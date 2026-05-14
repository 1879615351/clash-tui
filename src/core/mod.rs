use std::path::PathBuf;

/// The mihomo core binary embedded at compile time (via build.rs).
/// If the build script couldn't download the core, this will be empty.
static EMBEDDED_CORE: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/mihomo.bin"));

pub struct CoreManager;

impl CoreManager {
    pub fn core_dir() -> anyhow::Result<PathBuf> {
        crate::config::AppConfig::config_dir().map(|d| d.join("core"))
    }

    pub fn core_binary_path() -> anyhow::Result<PathBuf> {
        let name = if cfg!(windows) {
            "mihomo.exe"
        } else {
            "mihomo"
        };
        Ok(Self::core_dir()?.join(name))
    }

    pub fn is_installed() -> bool {
        Self::core_binary_path()
            .map(|p| p.exists())
            .unwrap_or(false)
    }

    /// Extract the embedded core binary to disk.
    /// Returns the path to the extracted binary.
    pub fn install() -> anyhow::Result<PathBuf> {
        if EMBEDDED_CORE.len() < 1000 {
            anyhow::bail!(
                "Core binary not embedded (build without network).\n\
                 Download mihomo manually and place it in: {:?}",
                Self::core_dir()?
            );
        }

        let dir = Self::core_dir()?;
        std::fs::create_dir_all(&dir)?;
        let dest = Self::core_binary_path()?;

        // Only extract if not already present or if embedded version is newer
        if !dest.exists() {
            std::fs::write(&dest, EMBEDDED_CORE)?;
            tracing::info!(
                "Core extracted to {} ({} MB)",
                dest.display(),
                EMBEDDED_CORE.len() / 1024 / 1024
            );

            // Make executable on Unix
            #[cfg(not(windows))]
            {
                use std::os::unix::fs::PermissionsExt;
                let mut perms = std::fs::metadata(&dest)?.permissions();
                perms.set_mode(0o755);
                std::fs::set_permissions(&dest, perms)?;
            }
        }

        Ok(dest)
    }

    /// Get the default core path for config. Uses embedded binary location.
    pub fn default_core_path() -> String {
        Self::core_binary_path()
            .map(|p| p.display().to_string())
            .unwrap_or_else(|_| String::new())
    }

    /// Save subscription config and merged main config. Returns the main config path.
    pub fn save_subscription_config(name: &str, yaml: &str) -> anyhow::Result<String> {
        let dir = Self::core_dir()?;
        std::fs::create_dir_all(&dir)?;

        let sub_path = dir.join(format!("sub_{}.yaml", sanitize_name(name)));
        std::fs::write(&sub_path, yaml)?;

        let main_config = Self::build_main_config(&dir)?;
        let main_path = dir.join("config.yaml");
        std::fs::write(&main_path, &main_config)?;

        Ok(main_path.display().to_string())
    }

    /// Build the main clash config by merging all subscription files.
    /// Each subscription is included as a file-based proxy-provider.
    fn build_main_config(workdir: &std::path::Path) -> anyhow::Result<String> {
        let mut subs = Vec::new();
        if let Ok(entries) = std::fs::read_dir(workdir) {
            for entry in entries.flatten() {
                let name_str = entry.file_name().to_string_lossy().to_string();
                if name_str.starts_with("sub_") && name_str.ends_with(".yaml") {
                    let base = entry
                        .path()
                        .file_stem()
                        .unwrap()
                        .to_string_lossy()
                        .to_string();
                    let name = base.trim_start_matches("sub_").to_string();
                    subs.push((name, entry.path()));
                }
            }
        }

        if subs.is_empty() {
            // No subscriptions — return minimal config
            return Ok(String::from(
                "mixed-port: 7890\nallow-lan: true\nmode: rule\nlog-level: info\n",
            ));
        }

        // If there's exactly one subscription and it's a full config (has port/mode),
        // use it directly instead of wrapping in proxy-providers
        if subs.len() == 1 {
            let content = std::fs::read_to_string(&subs[0].1)?;
            if content.contains("port:") || content.contains("mixed-port:") {
                // Clean up problematic config entries:
                // - Remove authentication (it enables auth)
                // - Remove/comment GEOIP rules (require MMDB download, blocked in CN)
                let content = content
                    .lines()
                    .filter(|l| {
                        !l.starts_with("authentication:") && !l.trim_start().starts_with("- GEOIP,")
                    })
                    .collect::<Vec<_>>()
                    .join("\n");
                return Ok(content);
            }
        }

        // Multiple subscriptions — merge via proxy-providers
        let mut config = String::new();
        config.push_str("mixed-port: 7890\n");
        config.push_str("allow-lan: true\n");
        config.push_str("mode: rule\n");
        config.push_str("log-level: info\n\n");

        config.push_str("proxy-providers:\n");
        for (name, path) in &subs {
            config.push_str(&format!(
                "  {}:\n    type: file\n    path: {}\n    health-check:\n      enable: true\n      url: https://www.gstatic.com/generate_204\n      interval: 300\n\n",
                name,
                path.display(),
            ));
        }

        config.push_str("proxy-groups:\n");
        config.push_str("  - name: PROXY\n    type: select\n    proxies:\n");
        for (name, _) in &subs {
            config.push_str(&format!("      - {}\n", name));
        }
        config.push_str("      - DIRECT\n");
        config.push_str("\nrules:\n  - MATCH,PROXY\n");

        Ok(config)
    }
    /// Kill any running mihomo and restart it with the current config.
    pub fn restart_mihomo() -> anyhow::Result<()> {
        let core_dir = Self::core_dir()?;
        let binary = Self::core_binary_path()?;

        #[cfg(windows)]
        {
            // Wait for kill to complete
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

        std::thread::sleep(std::time::Duration::from_secs(1));

        let log_file = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(core_dir.join("mihomo.log"))?;
        std::process::Command::new(&binary)
            .arg("-d")
            .arg(&core_dir)
            .stdout(log_file.try_clone()?)
            .stderr(log_file.try_clone()?)
            .spawn()?;

        tracing::info!("Mihomo restarted");
        Ok(())
    }
}

fn sanitize_name(name: &str) -> String {
    name.chars()
        .map(|c| {
            if c.is_alphanumeric() || c == '-' || c == '_' {
                c
            } else {
                '_'
            }
        })
        .collect()
}
