//! Cross-platform agent harness discovery engine.

use super::models::{DiscoveredHarness, PlatformTarget};
use std::path::{Path, PathBuf};

/// Discovers installed AI agent harnesses in a specific home directory for target platform.
pub fn discover_harnesses_in(home_dir: &Path, platform: PlatformTarget) -> Vec<DiscoveredHarness> {
    let mut harnesses = Vec::new();

    // 1. Google Antigravity IDE
    let (antigravity_config, antigravity_plugin) = match platform {
        PlatformTarget::MacOS => {
            let app_support = home_dir.join("Library/Application Support/Google/Antigravity");
            let default_gemini = home_dir.join(".gemini/config");
            let config = if app_support.exists() {
                app_support
            } else {
                default_gemini
            };
            let plugin = home_dir.join(".gemini/config/plugins/xgauntlet");
            (config, plugin)
        }
        PlatformTarget::Windows => {
            let app_data = home_dir.join("AppData/Roaming/Google/Antigravity");
            let default_gemini = home_dir.join(".gemini/config");
            let config = if app_data.exists() {
                app_data
            } else {
                default_gemini
            };
            let plugin = home_dir.join(".gemini/config/plugins/xgauntlet");
            (config, plugin)
        }
        PlatformTarget::Linux => {
            let config = home_dir.join(".gemini");
            let plugin = home_dir.join(".gemini/config/plugins/xgauntlet");
            (config, plugin)
        }
    };

    let antigravity_detected = antigravity_config.exists()
        || home_dir.join(".gemini").exists()
        || antigravity_plugin.exists();
    let antigravity_installed = is_plugin_installed(&antigravity_plugin);

    harnesses.push(DiscoveredHarness {
        name: "antigravity".to_string(),
        detected: antigravity_detected,
        config_dir: antigravity_config,
        plugin_installed: antigravity_installed,
        plugin_dir: antigravity_plugin,
    });

    // 2. Claude Code
    let (claude_config, claude_plugin) = match platform {
        PlatformTarget::Windows => {
            let app_data = home_dir.join("AppData/Roaming/Claude");
            let dot_claude = home_dir.join(".claude");
            let config = if app_data.exists() {
                app_data
            } else {
                dot_claude
            };
            let plugin = home_dir.join(".claude/skills/xgauntlet");
            (config, plugin)
        }
        _ => {
            let config = home_dir.join(".claude");
            let plugin = home_dir.join(".claude/skills/xgauntlet");
            (config, plugin)
        }
    };

    let claude_detected =
        claude_config.exists() || home_dir.join(".claude").exists() || claude_plugin.exists();
    let claude_installed = is_plugin_installed(&claude_plugin);

    harnesses.push(DiscoveredHarness {
        name: "claude_code".to_string(),
        detected: claude_detected,
        config_dir: claude_config,
        plugin_installed: claude_installed,
        plugin_dir: claude_plugin,
    });

    // 3. OpenAI Codex
    let codex_config = home_dir.join(".codex");
    let codex_plugin = home_dir.join(".codex/plugins/xgauntlet");
    let codex_detected = codex_config.exists() || codex_plugin.exists();
    let codex_installed = is_plugin_installed(&codex_plugin);

    harnesses.push(DiscoveredHarness {
        name: "codex".to_string(),
        detected: codex_detected,
        config_dir: codex_config,
        plugin_installed: codex_installed,
        plugin_dir: codex_plugin,
    });

    // 4. Mistral Vibe
    let vibe_config = home_dir.join(".vibe");
    let vibe_plugin = home_dir.join(".vibe/plugins/xgauntlet");
    let vibe_detected = vibe_config.exists() || vibe_plugin.exists();
    let vibe_installed = is_plugin_installed(&vibe_plugin);

    harnesses.push(DiscoveredHarness {
        name: "mistral".to_string(),
        detected: vibe_detected,
        config_dir: vibe_config,
        plugin_installed: vibe_installed,
        plugin_dir: vibe_plugin,
    });

    harnesses
}

/// Discovers installed AI agent harnesses on current host platform.
pub fn discover_installed_harnesses() -> Vec<DiscoveredHarness> {
    let home = std::env::var("HOME")
        .or_else(|_| std::env::var("USERPROFILE"))
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from("."));

    discover_harnesses_in(&home, PlatformTarget::current())
}

fn is_plugin_installed(dir: &Path) -> bool {
    dir.is_dir()
}
