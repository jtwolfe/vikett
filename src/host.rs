//! Build a Snap from the live Hyprland/PipeWire box. Dry-run only — never walks.

use std::collections::BTreeMap;
use std::process::Command;

use anyhow::{Context, Result};
use serde::Deserialize;

use crate::classes::class_to_app;
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
        bins: present_bins(),
        lists: discover_lists(),
        downloads: None,
        chat_unread: BTreeMap::new(),
    }
    .with_active_workspace())
}

/// Bookmark folder titles only. The other three discovered lists land with their families.
pub fn discover_lists() -> BTreeMap<String, Vec<String>> {
    let mut lists = BTreeMap::new();
    lists.insert("bookmark_folder".into(), bookmark_folders());
    lists
}

fn present_bins() -> Vec<String> {
    [
        "zen-browser",
        "firefox",
        "google-chrome",
        "chromium",
        "brave",
    ]
    .into_iter()
    .filter(|bin| which(bin))
    .map(str::to_string)
    .collect()
}

fn bookmark_folders() -> Vec<String> {
    let Some(home) = std::env::var_os("HOME").map(std::path::PathBuf::from) else {
        return Vec::new();
    };
    let mut files = Vec::new();
    collect_places(&home.join(".zen"), &mut files);
    collect_places(&home.join(".mozilla").join("firefox"), &mut files);
    collect_places(&home.join(".config").join("zen"), &mut files);
    let mut out = Vec::new();
    for path in files {
        for title in sqlite_folders(&path) {
            let id = crate::text::norm(&title);
            if id.is_empty() || out.iter().any(|e| e == &id) {
                continue;
            }
            out.push(id);
        }
    }
    out
}

fn collect_places(dir: &std::path::Path, out: &mut Vec<std::path::PathBuf>) {
    let Ok(rd) = std::fs::read_dir(dir) else {
        return;
    };
    for ent in rd.flatten() {
        let places = ent.path().join("places.sqlite");
        if places.is_file() {
            out.push(places);
        }
    }
}

fn sqlite_folders(path: &std::path::Path) -> Vec<String> {
    let uri = sqlite_uri(path);
    let Ok(out) = Command::new("sqlite3")
        .args([
            "-batch",
            "-noinit",
            "-json",
            &uri,
            "SELECT title FROM moz_bookmarks WHERE type = 2 AND ifnull(title, '') != '';",
        ])
        .output()
    else {
        return Vec::new();
    };
    if !out.status.success() {
        return Vec::new();
    }
    let text = String::from_utf8_lossy(&out.stdout);
    let Ok(rows) = serde_json::from_str::<Vec<serde_json::Value>>(text.trim()) else {
        return Vec::new();
    };
    rows.into_iter()
        .filter_map(|row| row.get("title")?.as_str().map(str::to_string))
        .filter(|t| !t.trim().is_empty())
        .collect()
}

fn sqlite_uri(path: &std::path::Path) -> String {
    let mut enc = String::from("file:");
    for b in path.to_string_lossy().bytes() {
        match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'/' | b'.' | b'_' | b'-' => {
                enc.push(b as char);
            }
            _ => enc.push_str(&format!("%{b:02X}")),
        }
    }
    enc.push_str("?immutable=1");
    enc
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

fn whoami() -> String {
    std::env::var("USER").unwrap_or_else(|_| "jim".into())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn discover_lists_is_bookmark_folders_only() {
        let lists = discover_lists();
        assert_eq!(lists.len(), 1);
        assert!(lists.contains_key("bookmark_folder"));
    }
}
