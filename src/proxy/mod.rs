/// System proxy module.
/// Windows: Registry (HKCU\...\Internet Settings) + wininet notification.
/// Linux: Environment variable instructions.
#[cfg(windows)]
use anyhow::Context;

#[derive(Debug, Clone)]
pub struct ProxyState {
    pub enabled: bool,
    pub host: String,
    pub port: u16,
    pub bypass: String,
}

impl Default for ProxyState {
    fn default() -> Self {
        Self {
            enabled: false,
            host: "127.0.0.1".into(),
            port: 7890,
            bypass: "<local>".into(),
        }
    }
}

impl ProxyState {
    pub fn address(&self) -> String {
        format!("{}:{}", self.host, self.port)
    }

    /// Enable the system proxy.
    pub fn enable(&self) -> anyhow::Result<()> {
        set_system_proxy(true, &self.host, self.port, &self.bypass)
    }

    /// Disable the system proxy.
    pub fn disable(&self) -> anyhow::Result<()> {
        set_system_proxy(false, "", 0, "")
    }

    /// Toggle proxy on/off based on current state.
    pub fn toggle(&mut self) -> anyhow::Result<()> {
        if self.enabled {
            self.disable()?;
            self.enabled = false;
        } else {
            self.enable()?;
            self.enabled = true;
        }
        Ok(())
    }

    /// Check current system proxy state.
    pub fn detect() -> anyhow::Result<Self> {
        get_system_proxy()
    }
}

#[cfg(windows)]
fn set_system_proxy(enable: bool, host: &str, port: u16, bypass: &str) -> anyhow::Result<()> {
    use winreg::enums::*;
    use winreg::RegKey;

    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    let path = r"Software\Microsoft\Windows\CurrentVersion\Internet Settings";
    let key = hkcu
        .open_subkey_with_flags(path, KEY_SET_VALUE | KEY_QUERY_VALUE)
        .with_context(|| "Failed to open Internet Settings registry key")?;

    if enable {
        let server = format!("{}:{}", host, port);
        key.set_value("ProxyEnable", &1u32)?;
        key.set_value("ProxyServer", &server)?;
        if !bypass.is_empty() {
            key.set_value("ProxyOverride", &bypass)?;
        }
        tracing::debug!("System proxy enabled: {} (bypass: {})", server, bypass);
    } else {
        key.set_value("ProxyEnable", &0u32)?;
        tracing::debug!("System proxy disabled");
    }

    // Notify the system that settings changed
    notify_system()?;

    Ok(())
}

#[cfg(windows)]
fn get_system_proxy() -> anyhow::Result<ProxyState> {
    use winreg::enums::*;
    use winreg::RegKey;

    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    let path = r"Software\Microsoft\Windows\CurrentVersion\Internet Settings";
    let key = hkcu.open_subkey_with_flags(path, KEY_READ)?;

    let enabled: u32 = key.get_value("ProxyEnable").unwrap_or(0);
    let server: String = key.get_value("ProxyServer").unwrap_or_default();
    let bypass: String = key.get_value("ProxyOverride").unwrap_or_default();

    let (host, port) = parse_host_port(&server);

    Ok(ProxyState {
        enabled: enabled != 0,
        host: host.into(),
        port,
        bypass,
    })
}

#[cfg(windows)]
fn notify_system() -> anyhow::Result<()> {
    // Call InternetSetOptionW to notify wininet of proxy changes
    extern "system" {
        fn InternetSetOptionW(
            hinternet: *mut std::ffi::c_void,
            dwoption: u32,
            lpbuffer: *const std::ffi::c_void,
            dwbufferlength: u32,
        ) -> i32;
    }

    const INTERNET_OPTION_SETTINGS_CHANGED: u32 = 39;
    const INTERNET_OPTION_REFRESH: u32 = 37;

    unsafe {
        InternetSetOptionW(
            std::ptr::null_mut(),
            INTERNET_OPTION_SETTINGS_CHANGED,
            std::ptr::null(),
            0,
        );
        InternetSetOptionW(
            std::ptr::null_mut(),
            INTERNET_OPTION_REFRESH,
            std::ptr::null(),
            0,
        );
    }

    tracing::debug!("System notified of proxy changes");
    Ok(())
}

/// Parse "host:port" or "host" from proxy server string
#[cfg(any(windows, test))]
fn parse_host_port(server: &str) -> (&str, u16) {
    if server.is_empty() {
        return ("127.0.0.1", 7890);
    }
    // Handle "http://host:port" or "host:port" formats
    let s = server
        .trim_start_matches("http://")
        .trim_start_matches("https://");
    if let Some(colon) = s.rfind(':') {
        let host = &s[..colon];
        let port = s[colon + 1..].parse::<u16>().unwrap_or(7890);
        (host, port)
    } else {
        (s, 7890)
    }
}

// ============================
// Linux fallback (env vars)
// ============================

#[cfg(not(windows))]
fn set_system_proxy(enable: bool, host: &str, port: u16, _bypass: &str) -> anyhow::Result<()> {
    let addr = format!("http://{}:{}", host, port);
    if enable {
        tracing::debug!("System proxy env would be set to: {}", addr);
        tracing::info!(
            "Run: export HTTP_PROXY={} HTTPS_PROXY={} ALL_PROXY={}",
            addr,
            addr,
            addr
        );
        // On Linux, we typically can't set the caller's env vars.
        // Write to a shell-export file instead.
        if let Ok(export_path) = crate::config::AppConfig::config_dir().map(|d| d.join("proxy.env"))
        {
            let content = format!(
                "export HTTP_PROXY={}\nexport HTTPS_PROXY={}\nexport ALL_PROXY={}\n",
                addr, addr, addr
            );
            std::fs::write(&export_path, content)?;
            tracing::debug!("Proxy env written to {}", export_path.display());
        }
    } else {
        if let Ok(export_path) = crate::config::AppConfig::config_dir().map(|d| d.join("proxy.env"))
        {
            std::fs::write(
                &export_path,
                "unset HTTP_PROXY\nunset HTTPS_PROXY\nunset ALL_PROXY\n",
            )?;
        }
    }
    Ok(())
}

#[cfg(not(windows))]
fn get_system_proxy() -> anyhow::Result<ProxyState> {
    Ok(ProxyState::default())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_host_port() {
        assert_eq!(parse_host_port("127.0.0.1:7890"), ("127.0.0.1", 7890));
        assert_eq!(parse_host_port("localhost:8080"), ("localhost", 8080));
        assert_eq!(
            parse_host_port("http://127.0.0.1:7890"),
            ("127.0.0.1", 7890)
        );
        assert_eq!(parse_host_port(""), ("127.0.0.1", 7890));
        assert_eq!(parse_host_port("192.168.1.1"), ("192.168.1.1", 7890));
    }
}
