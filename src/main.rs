use std::panic;

use anyhow::Context;
use clap::Parser;
use clash_tui::app::App;
use clash_tui::clash::client::ClashClient;
use clash_tui::config::AppConfig;
use clash_tui::ui;
use clash_tui::ui::theme::Theme;

#[derive(Parser, Debug)]
#[command(name = "clash-tui", version, about = "Clash TUI Client")]
struct Cli {
    /// Run as background daemon (serves Clash API over IPC)
    #[arg(long)]
    daemon: bool,

    /// Download and install mihomo core
    #[arg(long)]
    install_core: bool,

    /// Connect to daemon at the given port (client mode)
    #[arg(long)]
    port: Option<u16>,

    /// Clash API host (monolithic mode only)
    #[arg(long, default_value = "127.0.0.1")]
    host: String,

    /// Clash API port (monolithic mode only)
    #[arg(long, default_value_t = 9090)]
    api_port: u16,
}

fn main() -> anyhow::Result<()> {
    let default_hook = panic::take_hook();
    panic::set_hook(Box::new(move |info| {
        let _ = ui::restore();
        default_hook(info);
    }));

    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive("clash_tui=info".parse()?),
        )
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
        // Client mode: connect to daemon via IPC
        tracing::info!("Client mode: connecting to daemon on port {}", port);
        let clash_client = rt.block_on(async {
            clash_tui::clash::ipc_client::IpcClashClient::connect(port).await
        })?;

        // Box the IPC client
        run_app_with_client(config, theme, Box::new(clash_client), &rt)
    } else {
        // Monolithic mode: auto-start embedded mihomo if not already running
        rt.block_on(async {
            ensure_mihomo_running(&config).await?;
            Ok::<_, anyhow::Error>(())
        })?;
        tracing::info!("Monolithic mode: API at {}:{}", cli.host, cli.api_port);
        let clash_client = ClashClient::new(&cli.host, cli.api_port, config.api.secret.clone());
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

    // App takes the boxed client
    let mut app = App::with_client(config, theme, clash_client, rt.handle().clone());
    let result = rt.block_on(app.run_async(&mut terminal));

    ui::restore()?;
    result?;

    tracing::info!("Clash TUI exited normally.");
    Ok(())
}

/// Ensure mihomo is running at the configured API address.
/// If not, auto-start the embedded core.
async fn ensure_mihomo_running(config: &AppConfig) -> anyhow::Result<()> {
    let host = &config.api.host;
    let port = config.api.port;

    // Quick check if mihomo is already running
    if is_mihomo_running(host, port).await {
        tracing::info!("Mihomo already running at {}:{}", host, port);
        return Ok(());
    }

    // Install and start embedded mihomo
    tracing::info!("Starting embedded mihomo...");
    let core_binary = clash_tui::core::CoreManager::install()?;

    // Ensure a default config exists in the core dir
    let core_dir = clash_tui::core::CoreManager::core_dir()?;
    let config_yaml = core_dir.join("config.yaml");
    if !config_yaml.exists() {
        // API port (9090) ≠ proxy port (7890) — must be different
        let proxy_port = 7890u16;
        let default_config = format!(
            "mixed-port: {}\nsocks-port: {}\nexternal-controller: {}:{}\nallow-lan: false\nmode: rule\nlog-level: info\n",
            proxy_port,
            proxy_port + 1,
            host,
            port, // API port
        );
        std::fs::create_dir_all(&core_dir)?;
        std::fs::write(&config_yaml, default_config)?;
    }

    // Spawn mihomo in background, log to file for debugging
    let log_file = std::fs::File::create(core_dir.join("mihomo.log"))?;
    std::process::Command::new(&core_binary)
        .arg("-d")
        .arg(&core_dir)
        .stdout(log_file.try_clone()?)
        .stderr(log_file.try_clone()?)
        .spawn()
        .context("Failed to start mihomo")?;

    tracing::info!("Mihomo started, waiting for API...");

    // Wait for API to become available (up to 10 seconds)
    for _ in 0..20 {
        tokio::time::sleep(std::time::Duration::from_millis(500)).await;
        if is_mihomo_running(host, port).await {
            tracing::info!("Mihomo API ready at {}:{}", host, port);
            return Ok(());
        }
    }

    // If we get here, mihomo might still be starting. Don't block — let the TUI show.
    tracing::warn!("Mihomo start timeout — TUI will retry connection");
    Ok(())
}

async fn is_mihomo_running(host: &str, port: u16) -> bool {
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
