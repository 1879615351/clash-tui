use std::panic;

use clap::Parser;
use clash_tui::app::App;
use clash_tui::clash::client::ClashClient;
use clash_tui::config::AppConfig;
use clash_tui::ui;
use clash_tui::ui::theme::Theme;

#[derive(Parser, Debug)]
#[command(name = "clash-tui", version, about = "Clash TUI Client")]
struct Cli {
    #[arg(long)]
    daemon: bool,
    #[arg(long)]
    install_core: bool,
    #[arg(long)]
    port: Option<u16>,
    #[arg(long, default_value = "127.0.0.1")]
    host: String,
    #[arg(long, default_value_t = 9090)]
    api_port: u16,
}

fn main() -> anyhow::Result<()> {
    let default_hook = panic::take_hook();
    panic::set_hook(Box::new(move |info| {
        let _ = ui::restore();
        cleanup_mihomo();
        default_hook(info);
    }));

    let log_dir = clash_tui::config::AppConfig::config_dir()?;
    std::fs::create_dir_all(&log_dir).ok();
    let log_file = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(log_dir.join("clash-tui.log"))
        .unwrap_or_else(|_| std::fs::File::create("clash-tui.log").unwrap());

    tracing_subscriber::fmt()
        .with_writer(std::sync::Mutex::new(log_file))
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive("clash_tui=info".parse()?),
        )
        .with_ansi(false)
        .init();

    let cli = Cli::parse();
    tracing::info!("Starting Clash TUI v{}", env!("CARGO_PKG_VERSION"));
    let config = AppConfig::load()?;

    if cli.install_core {
        match clash_tui::core::CoreManager::install() {
            Ok(path) => {
                println!("Core installed to: {}", path.display());
                return Ok(());
            }
            Err(e) => {
                eprintln!("Failed to install core: {}", e);
                return Err(e);
            }
        }
    }

    if cli.daemon {
        return run_daemon(config);
    }

    run_tui(config, cli)
}

#[tokio::main]
async fn run_daemon(config: AppConfig) -> anyhow::Result<()> {
    tracing::info!("Starting daemon mode...");
    let daemon = clash_tui::daemon::Daemon::new(config).await?;
    daemon.run().await?;
    Ok(())
}

fn run_tui(config: AppConfig, cli: Cli) -> anyhow::Result<()> {
    let theme = Theme::load(&config.ui.theme)?;
    tracing::info!("Theme: {}", theme.name);

    let rt = tokio::runtime::Runtime::new()?;
    let _guard = rt.enter();

    if let Some(port) = cli.port {
        tracing::info!("Client mode: connecting to daemon on port {}", port);
        let clash_client = rt.block_on(async {
            clash_tui::clash::ipc_client::IpcClashClient::connect(port).await
        })?;
        run_app_with_client(config, theme, Box::new(clash_client), &rt)
    } else {
        // Start TUI immediately. Mihomo boots in background via startup task in app.rs.
        tracing::info!("Monolithic mode: API at {}:{}", cli.host, cli.api_port);
        let clash_client = ClashClient::new(&cli.host, cli.api_port, config.api.secret.clone());
        // Spawn mihomo in background (non-blocking)
        let host = cli.host.clone();
        let port = cli.api_port;
        rt.spawn(async move {
            start_mihomo_background(&host, port).await;
        });
        run_app_with_client(config, theme, Box::new(clash_client), &rt)
    }
}

fn run_app_with_client(
    config: AppConfig,
    theme: Theme,
    clash_client: Box<dyn clash_tui::clash::ClashApi>,
    rt: &tokio::runtime::Runtime,
) -> anyhow::Result<()> {
    let mut terminal = ui::init()?;

    let mut app = App::with_client(config, theme, clash_client, rt.handle().clone());
    let result = rt.block_on(app.run_async(&mut terminal));

    ui::restore()?;
    cleanup_mihomo();
    result?;
    tracing::info!("Clash TUI exited normally.");
    Ok(())
}

/// Start mihomo in the background. Returns when the API is reachable.
async fn start_mihomo_background(host: &str, port: u16) {
    // Kill any stale mihomo first
    #[cfg(windows)]
    {
        let _ = std::process::Command::new("taskkill")
            .args(["/F", "/IM", "mihomo.exe"])
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .output();
    }
    tokio::time::sleep(std::time::Duration::from_millis(500)).await;

    tracing::info!("Starting embedded mihomo...");
    let core_binary = match clash_tui::core::CoreManager::install() {
        Ok(b) => b,
        Err(e) => {
            tracing::error!("Failed to install core: {}", e);
            return;
        }
    };
    let core_dir = match clash_tui::core::CoreManager::core_dir() {
        Ok(d) => d,
        Err(e) => {
            tracing::error!("Failed to get core dir: {}", e);
            return;
        }
    };
    let config_yaml = core_dir.join("config.yaml");
    if !config_yaml.exists() {
        let default_config = format!(
            "mixed-port: 7890\nsocks-port: 7891\nexternal-controller: {}:{}\nallow-lan: false\nmode: rule\nlog-level: info\n",
            host, port
        );
        let _ = std::fs::create_dir_all(&core_dir);
        let _ = std::fs::write(&config_yaml, default_config);
    }

    let log_file = std::fs::File::create(core_dir.join("mihomo.log"))
        .unwrap_or_else(|_| std::fs::File::create("mihomo.log").unwrap());
    let _ = std::process::Command::new(&core_binary)
        .arg("-d")
        .arg(&core_dir)
        .stdout(
            log_file
                .try_clone()
                .unwrap_or_else(|_| std::fs::File::create("mihomo.log").unwrap()),
        )
        .stderr(
            log_file
                .try_clone()
                .unwrap_or_else(|_| std::fs::File::create("mihomo.log").unwrap()),
        )
        .spawn();

    tracing::info!("Mihomo spawned, waiting for API...");
    for _ in 0..20 {
        tokio::time::sleep(std::time::Duration::from_millis(500)).await;
        if is_mihomo_ready(host, port).await {
            tracing::info!("Mihomo API ready at {}:{}", host, port);
            return;
        }
    }
    tracing::warn!("Mihomo start timeout — will keep retrying in background");
}

async fn is_mihomo_ready(host: &str, port: u16) -> bool {
    let url = format!("http://{}:{}/version", host, port);
    match reqwest::Client::new()
        .get(&url)
        .timeout(std::time::Duration::from_secs(2))
        .send()
        .await
    {
        Ok(resp) => resp.status().is_success(),
        Err(_) => false,
    }
}

fn cleanup_mihomo() {
    #[cfg(windows)]
    {
        let _ = std::process::Command::new("taskkill")
            .args(["/F", "/IM", "mihomo.exe"])
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .output();
    }
    #[cfg(not(windows))]
    {
        let _ = std::process::Command::new("pkill")
            .args(["-9", "mihomo"])
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .output();
    }
}
