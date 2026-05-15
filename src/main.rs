use std::panic;

use clap::Parser;
use clash_tui::app::App;
use clash_tui::clash::client::ClashClient;
use clash_tui::clash::RefreshData;
use clash_tui::config::AppConfig;
use clash_tui::ui;
use clash_tui::ui::theme::Theme;
use tokio::sync::mpsc;

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
        let (data_tx, data_rx) = mpsc::unbounded_channel();
        run_app_with_client(config, theme, Box::new(clash_client), data_tx, data_rx, &rt)
    } else {
        tracing::info!("Monolithic mode: API at {}:{}", cli.host, cli.api_port);
        let clash_client = ClashClient::new(&cli.host, cli.api_port, config.api.secret.clone());
        let (data_tx, data_rx) = mpsc::unbounded_channel::<RefreshData>();
        // Spawn mihomo in background; it will push RefreshData once the API is up
        let bg_tx = data_tx.clone();
        let host = cli.host.clone();
        let port = cli.api_port;
        rt.spawn(async move {
            start_mihomo_background(&host, port, bg_tx).await;
        });
        run_app_with_client(config, theme, Box::new(clash_client), data_tx, data_rx, &rt)
    }
}

fn run_app_with_client(
    config: AppConfig,
    theme: Theme,
    clash_client: Box<dyn clash_tui::clash::ClashApi>,
    data_tx: mpsc::UnboundedSender<RefreshData>,
    data_rx: mpsc::UnboundedReceiver<RefreshData>,
    rt: &tokio::runtime::Runtime,
) -> anyhow::Result<()> {
    let mut terminal = ui::init()?;

    let mut app = App::with_client(config, theme, clash_client, data_tx, data_rx, rt.handle().clone());
    let result = rt.block_on(app.run_async(&mut terminal));

    ui::restore()?;
    cleanup_mihomo();
    result?;
    tracing::info!("Clash TUI exited normally.");
    Ok(())
}

/// Start mihomo in the background. Once the API is reachable, pushes initial
/// RefreshData through `data_tx` so the TUI updates its connection status immediately.
async fn start_mihomo_background(host: &str, port: u16, data_tx: mpsc::UnboundedSender<RefreshData>) {
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
    // Always regenerate config from available subscription files.
    // If sub_*.yaml files are missing (e.g. deleted since last session),
    // a minimal valid config is written so mihomo can start cleanly.
    if let Err(e) = clash_tui::core::CoreManager::regenerate_main_config(host, port) {
        tracing::error!("Failed to regenerate config: {}", e);
    }

    let log_file = std::fs::File::create(core_dir.join("mihomo.log"))
        .unwrap_or_else(|_| std::fs::File::create("mihomo.log").unwrap());
    let spawn_result = std::process::Command::new(&core_binary)
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
    match spawn_result {
        Ok(_child) => {
            tracing::info!("Mihomo spawned, waiting for API...");
        }
        Err(e) => {
            tracing::error!("Failed to spawn mihomo: {}", e);
            return;
        }
    }

    // Poll /version with a fast, lightweight HTTP check (not refresh_all).
    // Using refresh_all here risks 10s hangs if mihomo's TCP port is open
    // but the HTTP layer isn't ready yet — each of the 8 concurrent requests
    // would block for the full timeout.
    let check_url = format!("http://{}:{}/version", host, port);
    for _ in 0..40 {
        tokio::time::sleep(std::time::Duration::from_millis(500)).await;
        match reqwest::Client::new()
            .get(&check_url)
            .timeout(std::time::Duration::from_secs(2))
            .send()
            .await
        {
            Ok(resp) if resp.status().is_success() => {
                tracing::info!("Mihomo API ready at {}:{}", host, port);
                // Push a lightweight signal — the refresh loop will deliver full data
                let signal = RefreshData {
                    core_version: "pending".into(),
                    api_reachable: true,
                    ..Default::default()
                };
                let _ = data_tx.send(signal);
                return;
            }
            Ok(_) => {
                // HTTP 4xx/5xx — mihomo initialising
            }
            Err(_) => {
                // Not reachable yet
            }
        }
    }
    tracing::warn!("Mihomo start timeout after 20s — will keep retrying via refresh loop");
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
