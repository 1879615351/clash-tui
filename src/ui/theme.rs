use anyhow::Context;
use ratatui::style::Color;
use serde::Deserialize;

use crate::config::AppConfig;

#[derive(Debug, Clone, Deserialize)]
struct ThemeFile {
    name: Option<String>,
    bg: Option<String>,
    fg: Option<String>,
    accent: Option<String>,
    surface: Option<String>,
    surface_light: Option<String>,
    success: Option<String>,
    warning: Option<String>,
    error: Option<String>,
    info: Option<String>,
    latency_low: Option<String>,
    latency_mid: Option<String>,
    latency_high: Option<String>,
    latency_offline: Option<String>,
    tab_active: Option<String>,
    tab_inactive: Option<String>,
}

/// Color theme for the TUI
#[derive(Debug, Clone)]
pub struct Theme {
    pub name: String,

    pub bg: Color,
    pub fg: Color,
    pub accent: Color,
    pub surface: Color,
    pub surface_light: Color,

    pub success: Color,
    pub warning: Color,
    pub error: Color,
    pub info: Color,

    pub latency_low: Color,
    pub latency_mid: Color,
    pub latency_high: Color,
    pub latency_offline: Color,

    pub tab_active: Color,
    pub tab_inactive: Color,
}

impl Theme {
    pub fn tokyo_night() -> Self {
        Self {
            name: "tokyo-night".into(),
            bg: hex("#1a1b26"),
            fg: hex("#c0caf5"),
            accent: hex("#7aa2f7"),
            surface: hex("#24283b"),
            surface_light: hex("#414868"),
            success: hex("#9ece6a"),
            warning: hex("#e0af68"),
            error: hex("#f7768e"),
            info: hex("#7dcfff"),
            latency_low: hex("#9ece6a"),
            latency_mid: hex("#e0af68"),
            latency_high: hex("#f7768e"),
            latency_offline: hex("#565f89"),
            tab_active: hex("#7aa2f7"),
            tab_inactive: hex("#565f89"),
        }
    }

    pub fn catppuccin() -> Self {
        Self {
            name: "catppuccin".into(),
            bg: hex("#1e1e2e"),
            fg: hex("#cdd6f4"),
            accent: hex("#cba6f7"),
            surface: hex("#313244"),
            surface_light: hex("#45475a"),
            success: hex("#a6e3a1"),
            warning: hex("#f9e2af"),
            error: hex("#f38ba8"),
            info: hex("#89dceb"),
            latency_low: hex("#a6e3a1"),
            latency_mid: hex("#f9e2af"),
            latency_high: hex("#f38ba8"),
            latency_offline: hex("#585b70"),
            tab_active: hex("#cba6f7"),
            tab_inactive: hex("#585b70"),
        }
    }

    pub fn gruvbox() -> Self {
        Self {
            name: "gruvbox".into(),
            bg: hex("#282828"),
            fg: hex("#ebdbb2"),
            accent: hex("#fabd2f"),
            surface: hex("#3c3836"),
            surface_light: hex("#504945"),
            success: hex("#b8bb26"),
            warning: hex("#fabd2f"),
            error: hex("#fb4934"),
            info: hex("#83a598"),
            latency_low: hex("#b8bb26"),
            latency_mid: hex("#fabd2f"),
            latency_high: hex("#fb4934"),
            latency_offline: hex("#665c54"),
            tab_active: hex("#fabd2f"),
            tab_inactive: hex("#665c54"),
        }
    }

    /// Load theme by name. Tries file first, then built-in, then Tokyo Night fallback.
    pub fn load(name: &str) -> anyhow::Result<Self> {
        // Try loading from themes directory
        if let Ok(themes_dir) = AppConfig::themes_dir() {
            let theme_path = themes_dir.join(format!("{}.toml", name));
            if theme_path.exists() {
                let content =
                    std::fs::read_to_string(&theme_path).context("Failed to read theme file")?;
                let file: ThemeFile =
                    toml::from_str(&content).context("Failed to parse theme file")?;
                return Ok(Self::from_file(name, file));
            }
        }

        // Fall back to built-in
        match name.to_lowercase().as_str() {
            "tokyo-night" | "tokyo_night" => Ok(Self::tokyo_night()),
            "catppuccin" => Ok(Self::catppuccin()),
            "gruvbox" => Ok(Self::gruvbox()),
            _ => {
                tracing::info!("Theme '{}' not found, using tokyo-night fallback", name);
                Ok(Self::tokyo_night())
            }
        }
    }

    fn from_file(name: &str, file: ThemeFile) -> Self {
        let base = Self::tokyo_night(); // defaults for missing fields
        Self {
            name: file.name.unwrap_or_else(|| name.to_string()),
            bg: file.bg.map(|s| hex(&s)).unwrap_or(base.bg),
            fg: file.fg.map(|s| hex(&s)).unwrap_or(base.fg),
            accent: file.accent.map(|s| hex(&s)).unwrap_or(base.accent),
            surface: file.surface.map(|s| hex(&s)).unwrap_or(base.surface),
            surface_light: file
                .surface_light
                .map(|s| hex(&s))
                .unwrap_or(base.surface_light),
            success: file.success.map(|s| hex(&s)).unwrap_or(base.success),
            warning: file.warning.map(|s| hex(&s)).unwrap_or(base.warning),
            error: file.error.map(|s| hex(&s)).unwrap_or(base.error),
            info: file.info.map(|s| hex(&s)).unwrap_or(base.info),
            latency_low: file
                .latency_low
                .map(|s| hex(&s))
                .unwrap_or(base.latency_low),
            latency_mid: file
                .latency_mid
                .map(|s| hex(&s))
                .unwrap_or(base.latency_mid),
            latency_high: file
                .latency_high
                .map(|s| hex(&s))
                .unwrap_or(base.latency_high),
            latency_offline: file
                .latency_offline
                .map(|s| hex(&s))
                .unwrap_or(base.latency_offline),
            tab_active: file.tab_active.map(|s| hex(&s)).unwrap_or(base.tab_active),
            tab_inactive: file
                .tab_inactive
                .map(|s| hex(&s))
                .unwrap_or(base.tab_inactive),
        }
    }
}

fn hex(s: &str) -> Color {
    let s = s.trim_start_matches('#');
    if s.len() == 6 {
        let r = u8::from_str_radix(&s[0..2], 16).unwrap_or(0);
        let g = u8::from_str_radix(&s[2..4], 16).unwrap_or(0);
        let b = u8::from_str_radix(&s[4..6], 16).unwrap_or(0);
        Color::Rgb(r, g, b)
    } else {
        Color::Reset
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hex_color() {
        assert_eq!(hex("#ff0000"), Color::Rgb(255, 0, 0));
        assert_eq!(hex("00ff00"), Color::Rgb(0, 255, 0));
        assert_eq!(hex("#1a1b26"), Color::Rgb(26, 27, 38));
    }

    #[test]
    fn test_builtin_themes() {
        let t = Theme::tokyo_night();
        assert_eq!(t.name, "tokyo-night");

        let t = Theme::catppuccin();
        assert_eq!(t.name, "catppuccin");

        let t = Theme::gruvbox();
        assert_eq!(t.name, "gruvbox");
    }

    #[test]
    fn test_theme_fallback() {
        let t = Theme::load("nonexistent").unwrap();
        assert_eq!(t.name, "tokyo-night");
    }
}
