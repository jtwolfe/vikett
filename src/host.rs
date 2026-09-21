//! Build a Snap from the live Hyprland/PipeWire box. Dry-run only — never walks.

use std::collections::BTreeMap;
use std::process::Command;

use anyhow::{Context, Result};
use serde::Deserialize;

use crate::types::{Client, Snap};

#[derive(Deserialize)]
struct HyprWorkspace {
    #[allow(dead_code)]
    id: i64,
    name: String,
}

#[derive(Deserialize)]
struct HyprClient {
    address: String,
    class: String,
    title: String,
    workspace: HyprWorkspace,
    #[serde(default)]
    fullscreen: serde_json::Value,
    #[serde(default)]
    floating: bool,
}

#[derive(Deserialize)]
struct HyprMonitor {
    name: String,
    #[serde(default)]
    focused: bool,
}

#[derive(Deserialize)]
struct HyprActive {
    address: Option<String>,
}

pub fn live_snap() -> Result<Snap> {
    let clients_raw = hyprctl_json("clients")?;
    let monitors_raw = hyprctl_json("monitors")?;
    let workspaces_raw = hyprctl_json("workspaces")?;
    let active_raw = hyprctl_json("activewindow").unwrap_or(serde_json::json!({}));

    let hypr_clients: Vec<HyprClient> = serde_json::from_value(clients_raw)?;
    let monitors: Vec<HyprMonitor> = serde_json::from_value(monitors_raw)?;
    let workspaces: Vec<HyprWorkspace> = serde_json::from_value(workspaces_raw)?;
    let active: HyprActive =
        serde_json::from_value(active_raw).unwrap_or(HyprActive { address: None });

    let focused_addr = active.address.unwrap_or_default();
    let clients: Vec<Client> = hypr_clients
        .into_iter()
        .map(|c| {
            let fullscreen = match c.fullscreen {
                serde_json::Value::Bool(b) => b,
                serde_json::Value::Number(n) => n.as_i64().unwrap_or(0) != 0,
                _ => false,
            };
            Client {
                focused: c.address == focused_addr,
                address: c.address,
                class: c.class,
                title: c.title,
                workspace: c.workspace.name,
                fullscreen,
                floating: c.floating,
            }
        })
        .collect();

    let mut running = Vec::new();
    for c in &clients {
        let id = class_to_app(&c.class);
        if !running.iter().any(|r| r == &id) {
            running.push(id);
        }
    }

    let (volume, muted) = wpctl_volume();
    let sinks = wpctl_sink_names();
    let default_sink = if sinks.iter().any(|s| s.contains("hdmi") || s == "tv") && sinks.len() > 1 {
        // Prefer the named default: analog -> speakers, hdmi -> tv
        if default_is_hdmi() {
            "tv".into()
        } else {
            "speakers".into()
        }
    } else {
        "speakers".into()
    };

    let focused_output = monitors
        .iter()
        .find(|m| m.focused)
        .map(|m| m.name.clone())
        .or_else(|| monitors.first().map(|m| m.name.clone()));

    let mut allowlist = vec![
        "grokbot".into(),
        "firefox".into(),
        "kitty".into(),
        "jellyfin".into(),
        "thunderbird".into(),
        "files".into(),
        "code".into(),
        "zen".into(),
        "foot".into(),
    ];
    for r in &running {
        if !allowlist.contains(r) {
            allowlist.push(r.clone());
        }
    }

    Ok(Snap {
        id: "live".into(),
        title: "Live host".into(),
        who: whoami(),
        place: "desk".into(),
        occupants: vec![whoami()],
        guest: false,
        clients,
        workspaces: workspaces.into_iter().map(|w| w.name).collect(),
        active_workspace: monitors
            .iter()
            .find(|m| m.focused)
            .map(|_| {
                // filled below
                String::new()
            })
            .unwrap_or_default(),
        volume,
        muted,
        default_sink,
        allowlist,
        running,
        mail_online: false,
        contacts: vec!["dave".into(), "school".into()],
        unread_from: BTreeMap::new(),
        media_playing: false,
        now_playing: None,
        scene: None,
        lights: 0.5,
        timer_sec: None,
        bucky_listening: false,
        brightness: 0.7,
        climate: None,
        next_event: None,
        weather: None,
        owner: whoami(),
        monitors: monitors.len() as u32,
        lock_available: which("hyprlock"),
        notification: None,
        network: nmcli_active(),
        bluetooth_on: bluetooth_powered(),
        mic_muted: false,
        sinks,
        focused_output,
    }
    .with_active_workspace())
}

impl Snap {
    fn with_active_workspace(mut self) -> Self {
        if self.active_workspace.is_empty() {
            if let Some(c) = self.clients.iter().find(|c| c.focused) {
                self.active_workspace = c.workspace.clone();
            } else if let Some(ws) = self.workspaces.first() {
                self.active_workspace = ws.clone();
            }
        }
        self
    }
}

fn hyprctl_json(cmd: &str) -> Result<serde_json::Value> {
    let out = Command::new("hyprctl")
        .args(["-j", cmd])
        .output()
        .with_context(|| format!("hyprctl -j {cmd}"))?;
    if !out.status.success() {
        anyhow::bail!("hyprctl {cmd} failed");
    }
    Ok(serde_json::from_slice(&out.stdout)?)
}

fn wpctl_volume() -> (f32, bool) {
    let out = Command::new("wpctl")
        .args(["get-volume", "@DEFAULT_AUDIO_SINK@"])
        .output();
    let Ok(out) = out else { return (0.4, false) };
    let s = String::from_utf8_lossy(&out.stdout);
    let muted = s.contains("MUTED");
    let vol = s
        .split_whitespace()
        .filter_map(|t| t.parse::<f32>().ok())
        .next()
        .unwrap_or(0.4);
    (vol, muted)
}

fn wpctl_sink_names() -> Vec<String> {
    let out = Command::new("wpctl").arg("status").output();
    let Ok(out) = out else {
        return vec!["speakers".into()];
    };
    let s = String::from_utf8_lossy(&out.stdout);
    let mut names = Vec::new();
    let mut in_sinks = false;
    for line in s.lines() {
        if line.contains("Sinks:") {
            in_sinks = true;
            continue;
        }
        if in_sinks && (line.contains("Sources:") || line.contains("Filters:")) {
            break;
        }
        if in_sinks {
            if line.to_lowercase().contains("hdmi") {
                names.push("tv".into());
            } else if line.contains("Analog") || line.contains("Speaker") {
                names.push("speakers".into());
            }
        }
    }
    if names.is_empty() {
        names.push("speakers".into());
    }
    names
}

fn default_is_hdmi() -> bool {
    let out = Command::new("wpctl").arg("status").output();
    let Ok(out) = out else { return false };
    let s = String::from_utf8_lossy(&out.stdout);
    let mut in_sinks = false;
    for line in s.lines() {
        if line.contains("Sinks:") {
            in_sinks = true;
            continue;
        }
        if in_sinks && line.contains("Sources:") {
            break;
        }
        if in_sinks && line.contains('*') {
            return line.to_lowercase().contains("hdmi");
        }
    }
    false
}

fn nmcli_active() -> Option<String> {
    let out = Command::new("nmcli")
        .args([
            "-t",
            "-f",
            "TYPE,NAME,STATE",
            "connection",
            "show",
            "--active",
        ])
        .output()
        .ok()?;
    let s = String::from_utf8_lossy(&out.stdout);
    for line in s.lines() {
        if line.contains("ethernet") {
            return Some("ethernet".into());
        }
        if line.contains("wireless") || line.contains("wifi") {
            return Some("wifi".into());
        }
    }
    if s.trim().is_empty() {
        None
    } else {
        Some("online".into())
    }
}

fn bluetooth_powered() -> bool {
    let out = Command::new("bluetoothctl").arg("show").output();
    let Ok(out) = out else { return false };
    String::from_utf8_lossy(&out.stdout)
        .lines()
        .any(|l| l.contains("Powered: yes"))
}

fn which(bin: &str) -> bool {
    Command::new("which")
        .arg(bin)
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

fn class_to_app(class: &str) -> String {
    let c = class.to_lowercase();
    if c.contains("zen") {
        "zen".into()
    } else if c.contains("foot") {
        "foot".into()
    } else if c.contains("firefox") {
        "firefox".into()
    } else if c.contains("kitty") {
        "kitty".into()
    } else if c.contains("jellyfin") {
        "jellyfin".into()
    } else if c.contains("code") || c.contains("codium") {
        "code".into()
    } else if c.contains("thunderbird") {
        "thunderbird".into()
    } else {
        c
    }
}

fn whoami() -> String {
    std::env::var("USER").unwrap_or_else(|_| "jim".into())
}

#[cfg(test)]
mod tests {
    use super::class_to_app;

    #[test]
    fn maps_zen_and_foot() {
        assert_eq!(class_to_app("zen"), "zen");
        assert_eq!(class_to_app("foot"), "foot");
    }
}
