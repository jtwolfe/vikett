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
        clipboard_kind: None,
        clipboard_count: None,
        picked_hex: None,
        battery: None,
        on_ac: false,
        disk_free: None,
        updates_pending: None,
        secrets_unlocked: None,
        solaar_battery: None,
    }
    .with_active_workspace())
}

/// Authored output class. The slot is never the raw connector.
/// `HDMI-A-1` is `hdmi`. `eDP-1` is `edp`. A DisplayPort name is neither.
pub fn output_class(name: &str) -> Option<&'static str> {
    let n = name.to_ascii_lowercase();
    if n.contains("edp") {
        Some("edp")
    } else if n.contains("hdmi") {
        Some("hdmi")
    } else {
        None
    }
}

/// Bookmark folder titles, project directory names, VPN connection names,
/// and installed game titles. Fixtures do not call this.
pub fn discover_lists() -> BTreeMap<String, Vec<String>> {
    let mut lists = BTreeMap::new();
    lists.insert("bookmark_folder".into(), bookmark_folders());
    lists.insert("project".into(), project_names());
    lists.insert("vpn".into(), vpn_names());
    lists.insert("game".into(), game_names());
    lists
}

/// Normed `nmcli` name: lowercase words, no shell metacharacters, not a flag.
pub fn vpn_id_ok(id: &str) -> bool {
    let mut chars = id.chars();
    let Some(first) = chars.next() else {
        return false;
    };
    if !first.is_ascii_lowercase() && !first.is_ascii_digit() {
        return false;
    }
    let len = id.chars().count();
    if !(2..=64).contains(&len) {
        return false;
    }
    if matches!(id, "wifi" | "ssid" | "dns" | "down" | "delete" | "radio") {
        return false;
    }
    let mut prev_space = false;
    for c in id.chars() {
        let ok = c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-' || c == '_' || c == ' ';
        if !ok || (c == ' ' && prev_space) {
            return false;
        }
        prev_space = c == ' ';
    }
    !id.ends_with(' ')
}

/// `nmcli -t -f NAME,TYPE connection show`. Type `vpn` or `wireguard` only.
/// The last colon separates name and type. Escaped names fail `vpn_id_ok`.
pub fn vpn_names_from_nmcli(text: &str) -> Vec<String> {
    let mut out = Vec::new();
    for line in text.lines() {
        let line = line.trim().trim_end_matches('\r');
        let Some((name, kind)) = line.rsplit_once(':') else {
            continue;
        };
        if !matches!(
            kind.trim().to_ascii_lowercase().as_str(),
            "vpn" | "wireguard"
        ) {
            continue;
        }
        let Some(id) = vpn_store_name(name) else {
            continue;
        };
        if out.iter().any(|e| e == &id) {
            continue;
        }
        out.push(id);
    }
    out.sort();
    out
}

fn vpn_names() -> Vec<String> {
    let Ok(out) = Command::new("nmcli")
        .args(["-t", "-f", "NAME,TYPE", "connection", "show"])
        .output()
    else {
        return Vec::new();
    };
    if !out.status.success() {
        return Vec::new();
    }
    vpn_names_from_nmcli(&String::from_utf8_lossy(&out.stdout))
}

fn vpn_store_name(raw: &str) -> Option<String> {
    let raw = raw.trim();
    if raw.is_empty()
        || raw.starts_with('-')
        || raw.starts_with(' ')
        || raw.ends_with(' ')
        || raw.contains("  ")
    {
        return None;
    }
    if raw
        .chars()
        .any(|c| !(c.is_ascii_alphanumeric() || c == ' ' || c == '-' || c == '_'))
    {
        return None;
    }
    let id = raw.to_lowercase();
    if id != crate::text::norm(raw) || !vpn_id_ok(&id) {
        return None;
    }
    Some(id)
}

/// Immediate child directory names of `~/storage` and `~/src`. No recursion.
/// The id is the normed name, never the path. Missing dirs yield an empty vec.
fn project_names() -> Vec<String> {
    let Some(home) = std::env::var_os("HOME").map(std::path::PathBuf::from) else {
        return Vec::new();
    };
    let mut out = Vec::new();
    for root in [home.join("storage"), home.join("src")] {
        let Ok(rd) = std::fs::read_dir(&root) else {
            continue;
        };
        for ent in rd.flatten() {
            if !ent.path().is_dir() {
                continue;
            }
            let name = ent.file_name();
            let Some(name) = name.to_str() else {
                continue;
            };
            if name.starts_with('.') {
                continue;
            }
            let id = crate::text::norm(name);
            if id.is_empty() || out.iter().any(|e| e == &id) {
                continue;
            }
            out.push(id);
        }
    }
    out.sort();
    out
}

/// Normed installed-game id. No path, URL, or store scheme.
pub fn game_id_ok(id: &str) -> bool {
    let mut chars = id.chars();
    let Some(first) = chars.next() else {
        return false;
    };
    if !first.is_ascii_lowercase() && !first.is_ascii_digit() {
        return false;
    }
    let len = id.chars().count();
    if !(2..=64).contains(&len) {
        return false;
    }
    if id.contains("http") || id.contains("://") {
        return false;
    }
    let mut prev_space = false;
    for c in id.chars() {
        let ok = c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-' || c == '_' || c == ' ';
        if !ok || (c == ' ' && prev_space) {
            return false;
        }
        prev_space = c == ' ';
    }
    !id.ends_with(' ')
}

/// `"name"` fields from one `appmanifest_*.acf`. `installdir` is not a title.
pub fn game_names_from_acf(text: &str) -> Vec<String> {
    let mut out = Vec::new();
    for line in text.lines() {
        let line = line.trim();
        let Some(rest) = line.strip_prefix("\"name\"") else {
            continue;
        };
        let rest = rest.trim().trim_start_matches('\t');
        let Some(rest) = rest.strip_prefix('"') else {
            continue;
        };
        let Some((name, _)) = rest.split_once('"') else {
            continue;
        };
        let Some(id) = game_store_name(name) else {
            continue;
        };
        push_game(&mut out, id);
    }
    out.sort();
    out
}

/// Installed Heroic or Legendary titles. A `title` without an install path
/// or executable is a store entry and is dropped. No network.
pub fn game_names_from_heroic(text: &str) -> Vec<String> {
    let Ok(value) = serde_json::from_str::<serde_json::Value>(text) else {
        return Vec::new();
    };
    let mut out = Vec::new();
    collect_installed_titles(&value, &mut out);
    out.sort();
    out
}

/// `"path"` values from `libraryfolders.vdf`. Absolute paths only. `..` is dropped.
pub fn steam_library_paths(text: &str) -> Vec<String> {
    let mut out = Vec::new();
    for line in text.lines() {
        let line = line.trim();
        let Some(rest) = line.strip_prefix("\"path\"") else {
            continue;
        };
        let rest = rest.trim().trim_start_matches('\t');
        let Some(rest) = rest.strip_prefix('"') else {
            continue;
        };
        let Some((path, _)) = rest.split_once('"') else {
            continue;
        };
        let path = path.replace("\\\\", "\\");
        if !path.starts_with('/') || path.contains("..") || path.contains('\0') {
            continue;
        }
        if out.iter().any(|e| e == &path) {
            continue;
        }
        out.push(path);
    }
    out
}

fn game_store_name(raw: &str) -> Option<String> {
    let raw = raw.trim();
    if raw.is_empty()
        || raw.contains("://")
        || raw.contains('/')
        || raw.contains('\\')
        || raw.contains("..")
        || raw.starts_with('-')
    {
        return None;
    }
    if raw.chars().any(|c| {
        matches!(
            c,
            '"' | '\'' | '`' | '$' | ';' | '|' | '&' | '*' | '<' | '>' | '(' | ')'
        )
    }) {
        return None;
    }
    let id = crate::text::norm(raw);
    if !game_id_ok(&id) {
        return None;
    }
    Some(id)
}

fn collect_installed_titles(v: &serde_json::Value, out: &mut Vec<String>) {
    match v {
        serde_json::Value::Array(items) => {
            for item in items {
                collect_installed_titles(item, out);
            }
        }
        serde_json::Value::Object(map) => {
            let installed = ["install_path", "installPath", "executable"]
                .iter()
                .any(|k| map.contains_key(*k));
            if installed {
                if let Some(id) = map
                    .get("title")
                    .and_then(|t| t.as_str())
                    .and_then(game_store_name)
                {
                    push_game(out, id);
                }
            }
            for (k, child) in map {
                if matches!(
                    k.as_str(),
                    "title" | "install_path" | "installPath" | "executable"
                ) {
                    continue;
                }
                if child.is_object() || child.is_array() {
                    collect_installed_titles(child, out);
                }
            }
        }
        _ => {}
    }
}

fn push_game(out: &mut Vec<String>, id: String) {
    if !out.iter().any(|e| e == &id) {
        out.push(id);
    }
}

/// Steam `appmanifest_*.acf` names plus Heroic/Legendary installed titles.
/// Missing files yield nothing. This does not query a store.
fn game_names() -> Vec<String> {
    let Some(home) = std::env::var_os("HOME").map(std::path::PathBuf::from) else {
        return Vec::new();
    };
    let mut roots = Vec::new();
    for rel in [
        ".steam/steam/steamapps",
        ".steam/root/steamapps",
        ".local/share/Steam/steamapps",
        ".var/app/com.valvesoftware.Steam/.local/share/Steam/steamapps",
    ] {
        roots.push(home.join(rel));
    }
    let mut extra = Vec::new();
    for root in &roots {
        let Ok(text) = std::fs::read_to_string(root.join("libraryfolders.vdf")) else {
            continue;
        };
        for path in steam_library_paths(&text) {
            extra.push(std::path::PathBuf::from(path).join("steamapps"));
        }
    }
    roots.extend(extra);
    let mut out = Vec::new();
    for root in roots {
        let Ok(rd) = std::fs::read_dir(&root) else {
            continue;
        };
        for ent in rd.flatten() {
            let name = ent.file_name();
            let Some(name) = name.to_str() else {
                continue;
            };
            if !(name.starts_with("appmanifest_") && name.ends_with(".acf")) {
                continue;
            }
            let Ok(text) = std::fs::read_to_string(ent.path()) else {
                continue;
            };
            for id in game_names_from_acf(&text) {
                push_game(&mut out, id);
            }
        }
    }
    for rel in [
        ".config/legendary/installed.json",
        ".config/heroic/gog_store/installed.json",
        ".var/app/com.heroicgameslauncher.hgl/config/legendary/installed.json",
        ".var/app/com.heroicgameslauncher.hgl/config/heroic/gog_store/installed.json",
    ] {
        let Ok(text) = std::fs::read_to_string(home.join(rel)) else {
            continue;
        };
        for id in game_names_from_heroic(&text) {
            push_game(&mut out, id);
        }
    }
    out.sort();
    out
}

/// `which` of known names for the live snap. Not `nvidia-smi` and not a monitor list.
/// Secrets, boxes, games, OBS, print, and Solaar are not probed.
fn present_bins() -> Vec<String> {
    [
        "zen-browser",
        "firefox",
        "google-chrome",
        "chromium",
        "brave",
        "fuzzel",
        "walker",
        "wf-recorder",
        "gpu-screen-recorder",
        "wl-copy",
        "cliphist",
        "swww",
        "nmcli",
        "bluetoothctl",
        "powerprofilesctl",
        "timeshift",
        "pacman",
        "dnf",
        "apt",
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
    fn discover_lists_has_folders_and_projects() {
        let lists = discover_lists();
        assert_eq!(lists.len(), 4);
        assert!(lists.contains_key("bookmark_folder"));
        assert!(lists.contains_key("project"));
        assert!(lists.contains_key("vpn"));
        assert!(lists.contains_key("game"));
        for id in &lists["game"] {
            assert!(game_id_ok(id), "{id}");
            assert!(!id.contains('/'), "{id}");
            assert!(!id.contains("http"), "{id}");
            assert!(!id.contains("steam://"), "{id}");
        }
        let projects = &lists["project"];
        assert!(projects.iter().all(|id| !id.contains('/')));
        assert!(projects.iter().all(|id| !id.starts_with('.')));
        let vpns = &lists["vpn"];
        assert!(vpns.iter().all(|id| vpn_id_ok(id)));
        assert!(vpns.iter().all(|id| !id.contains('/')));
        assert!(vpns.iter().all(|id| !id.contains(':')));
    }

    #[test]
    fn vpn_names_are_vpn_or_wireguard_only() {
        let text = "\
home:vpn
wg0:wireguard
cafe:802-11-wireless
Wired connection 1:802-3-ethernet
lo:loopback
Home:vpn
Work VPN:vpn
my\\:vpn:vpn
wifi:vpn
-evil:vpn
";
        let names = vpn_names_from_nmcli(text);
        assert_eq!(
            names,
            vec![
                "home".to_string(),
                "wg0".to_string(),
                "work vpn".to_string()
            ]
        );
        assert!(!names.iter().any(|id| id.contains(':')));
        assert!(vpn_id_ok("work vpn"));
        assert!(!vpn_id_ok("home'"));
        assert!(!vpn_id_ok("-flag"));
        assert!(!vpn_id_ok("wifi"));
        assert!(!vpn_id_ok("a"));
    }

    #[test]
    fn output_class_is_hdmi_or_edp_only() {
        assert_eq!(output_class("HDMI-A-1"), Some("hdmi"));
        assert_eq!(output_class("eDP-1"), Some("edp"));
        assert_eq!(output_class("DP-1"), None);
        assert_eq!(output_class("VGA-1"), None);
    }

    #[test]
    fn game_titles_come_from_manifests_not_a_store() {
        let acf = "\
\"AppState\"
{
\t\"appid\"\t\t\"504230\"
\t\"name\"\t\t\"Celeste\"
\t\"installdir\"\t\t\"DoNotUse\"
}
\"name\"\t\t\"../x\"
\"name\"\t\t\"http://evil\"
\"name\"\t\t\"Portal 2\"
\"name\"\t\t\"steam://rungameid/1\"
";
        assert_eq!(
            game_names_from_acf(acf),
            vec!["celeste".to_string(), "portal 2".to_string()]
        );
        assert!(game_id_ok("portal 2"));
        assert!(!game_id_ok("http://evil"));
        assert!(!game_id_ok("../x"));

        let heroic = r#"{
            "Celeste": {"title": "Celeste", "install_path": "/home/jim/Games/Celeste"},
            "Catalog": {"title": "Hades"},
            "Path": {"title": "/tmp/game", "executable": "game"}
        }"#;
        assert_eq!(game_names_from_heroic(heroic), vec!["celeste".to_string()]);
        let listed = r#"[{"title": "Into the Breach", "executable": "breach"}]"#;
        assert_eq!(
            game_names_from_heroic(listed),
            vec!["into the breach".to_string()]
        );
        assert!(game_names_from_heroic("not json").is_empty());

        let vdf = "\
\"path\"\t\t\"/mnt/games\"
\"path\"\t\t\"../etc\"
\"path\"\t\t\"relative\"
";
        assert_eq!(steam_library_paths(vdf), vec!["/mnt/games".to_string()]);
    }
}
