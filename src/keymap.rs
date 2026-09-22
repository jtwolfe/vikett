//! Stock chords and the one gate that arms them.
//! Hyprland 0.56 wants `hl.dsp.send_shortcut`, not `dispatch sendshortcut`.
//! `CTRL + SHIFT` is the dry-run spelling (spaces around `+`). Empty `mods` is
//! only legal for F11 — the field is still required by the dispatcher.

use std::collections::BTreeMap;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Chord {
    pub mods: String,
    pub key: String,
}

fn chord(mods: &str, key: &str) -> Chord {
    Chord {
        mods: mods.to_string(),
        key: key.to_string(),
    }
}

fn browser_app(app: &str) -> bool {
    matches!(app, "zen" | "firefox" | "chrome" | "chromium" | "brave")
}

/// Builtin bind, if this page/app pair has one. No candidate means the door stays dead.
pub fn builtin(page_id: &str, app: &str) -> Option<Chord> {
    if let Some(chord) = term_builtin(page_id, app) {
        return Some(chord);
    }
    if let Some(chord) = files_builtin(page_id, app) {
        return Some(chord);
    }
    if let Some(chord) = notes_builtin(page_id, app) {
        return Some(chord);
    }
    if let Some(chord) = read_builtin(page_id, app) {
        return Some(chord);
    }
    if let Some(chord) = video_builtin(page_id, app) {
        return Some(chord);
    }
    if let Some(chord) = image_builtin(page_id, app) {
        return Some(chord);
    }
    if let Some(chord) = edit_builtin(page_id, app) {
        return Some(chord);
    }
    if let Some(chord) = office_builtin(page_id, app) {
        return Some(chord);
    }
    if let Some(chord) = draw_builtin(page_id, app) {
        return Some(chord);
    }
    if !browser_app(app) {
        return None;
    }
    if let Some(chord) = zen_builtin(page_id, app) {
        return Some(chord);
    }
    // Firefox private is Ctrl+Shift+P from upstream docs. Chrome/Chromium/Zen/Brave
    // stay reserved — that chord is not confirmed on those builds.
    if page_id == "browser.private" {
        return if app == "firefox" {
            Some(chord("CTRL + SHIFT", "P"))
        } else {
            None
        };
    }
    let pair = match page_id {
        "browser.tab_new" => ("CTRL", "T"),
        "browser.tab_close" => ("CTRL", "W"),
        "browser.tab_reopen" => ("CTRL + SHIFT", "T"),
        "browser.tab_next" => ("CTRL", "Tab"),
        "browser.tab_prev" => ("CTRL + SHIFT", "Tab"),
        "browser.back" => ("ALT", "Left"),
        "browser.forward" => ("ALT", "Right"),
        "browser.reload" => ("CTRL", "R"),
        "browser.reload_hard" => ("CTRL + SHIFT", "R"),
        "browser.home" => ("ALT", "Home"),
        "browser.find" => ("CTRL", "F"),
        "browser.zoom" => ("CTRL", "plus"),
        "browser.zoom_reset" => ("CTRL", "0"),
        "browser.fullscreen" => ("", "F11"),
        "browser.dev_tools" => ("CTRL + SHIFT", "I"),
        _ => return None,
    };
    Some(chord(pair.0, pair.1))
}

/// Zen shortcuts table only. Compact is Alt+Ctrl+C there and Ctrl+S on the
/// compact-mode page, so it is absent. Glance has no key; Escape is not one.
fn zen_builtin(page_id: &str, app: &str) -> Option<Chord> {
    if app != "zen" {
        return None;
    }
    let pair = match page_id {
        "browser.zen_ws_next" => ("ALT + CTRL", "E"),
        "browser.zen_ws_prev" => ("ALT + CTRL", "Q"),
        "browser.sidebar" => ("ALT + CTRL", "S"),
        "browser.split" => ("ALT + CTRL", "H"),
        "browser.unsplit" => ("ALT + CTRL", "U"),
        _ => return None,
    };
    Some(chord(pair.0, pair.1))
}

/// Linux defaults only. Alacritty has no SpawnNewInstance bind. Foot has no tabs.
/// Kitty's layout window is Ctrl+Shift+Enter; this door is the OS window
/// (Ctrl+Shift+N), same idea as foot `spawn-terminal`.
fn term_builtin(page_id: &str, app: &str) -> Option<Chord> {
    if !matches!(app, "foot" | "kitty" | "ghostty" | "alacritty" | "wezterm") {
        return None;
    }
    let pair: (&str, &str) = match page_id {
        "term.new_window" => match app {
            "foot" | "kitty" | "ghostty" | "wezterm" => ("CTRL + SHIFT", "N"),
            _ => return None,
        },
        "term.new_tab" => match app {
            "kitty" | "ghostty" | "wezterm" => ("CTRL + SHIFT", "T"),
            _ => return None,
        },
        "term.close" => match app {
            // kitty close_window, ghostty close_surface/close_tab, wezterm CloseCurrentTab.
            "kitty" | "ghostty" | "wezterm" => ("CTRL + SHIFT", "W"),
            _ => return None,
        },
        "term.next" => match app {
            "kitty" => ("CTRL + SHIFT", "Right"),
            "ghostty" | "wezterm" => ("CTRL", "Tab"),
            _ => return None,
        },
        "term.prev" => match app {
            "kitty" => ("CTRL + SHIFT", "Left"),
            "ghostty" | "wezterm" => ("CTRL + SHIFT", "Tab"),
            _ => return None,
        },
        "term.font" => match app {
            "kitty" => ("CTRL + SHIFT", "equal"),
            "wezterm" => ("CTRL", "equal"),
            "foot" | "ghostty" | "alacritty" => ("CTRL", "plus"),
            _ => return None,
        },
        _ => return None,
    };
    Some(chord(pair.0, pair.1))
}

/// Alt+Left / Alt+Up are Nautilus, Nemo, Thunar, and Dolphin.
/// Ctrl+H shows hidden files in Nautilus, Nemo, and Thunar. Dolphin's hidden
/// chord is not one key we can cite, and Yazi's defaults are unmodified.
/// Trash is Delete or an unconfirmed Ctrl+Delete, so it stays reserved.
fn files_builtin(page_id: &str, app: &str) -> Option<Chord> {
    if !matches!(app, "nautilus" | "nemo" | "thunar" | "dolphin" | "yazi") {
        return None;
    }
    let pair = match page_id {
        "files.back" if matches!(app, "nautilus" | "nemo" | "thunar" | "dolphin") => {
            ("ALT", "Left")
        }
        "files.up" if matches!(app, "nautilus" | "nemo" | "thunar" | "dolphin") => ("ALT", "Up"),
        "files.hidden" if matches!(app, "nautilus" | "nemo" | "thunar") => ("CTRL", "H"),
        _ => return None,
    };
    Some(chord(pair.0, pair.1))
}

/// Logseq's default `:go/journals` is Alt+J on Linux. Obsidian daily and the
/// sidebar have no default hotkey. Joplin's sidebar is F10, which cannot arm
/// (empty mods are only legal for F11). Joplin daily is a plugin chord.
fn notes_builtin(page_id: &str, app: &str) -> Option<Chord> {
    if page_id == "notes.daily" && app == "logseq" {
        Some(chord("ALT", "J"))
    } else {
        None
    }
}

/// Evince and Papers: Ctrl+Page Down / Up, Ctrl+plus (GNOME help).
/// Foliate zoom includes Ctrl+plus; its page keys are unmodified.
/// Zathura recolor is Ctrl+R. Zathura page and zoom keys are unmodified.
/// Chapter next/prev has no single default chord.
fn read_builtin(page_id: &str, app: &str) -> Option<Chord> {
    if !matches!(app, "zathura" | "evince" | "papers" | "foliate") {
        return None;
    }
    let pdf = matches!(app, "evince" | "papers");
    let pair = match page_id {
        "read.next_page" if pdf => ("CTRL", "Page_Down"),
        "read.prev_page" if pdf => ("CTRL", "Page_Up"),
        "read.zoom" if matches!(app, "evince" | "papers" | "foliate") => ("CTRL", "plus"),
        "read.dark" if app == "zathura" => ("CTRL", "R"),
        _ => return None,
    };
    Some(chord(pair.0, pair.1))
}

/// VLC Shift+N is next chapter and Shift+V hides subtitles (published hotkey
/// tables). mpv chapter is Page Up and its sub key is `v`, both unmodified,
/// so they stay reserved. Fullscreen `f` and PiP have no modifier chord.
fn video_builtin(page_id: &str, app: &str) -> Option<Chord> {
    if app != "vlc" {
        return None;
    }
    let pair = match page_id {
        "video.chapter" => ("SHIFT", "N"),
        "video.subs" => ("SHIFT", "v"),
        _ => return None,
    };
    Some(chord(pair.0, pair.1))
}

/// Loupe help: Ctrl+plus / Ctrl+minus. The bare plus key cannot arm.
/// imv's default config zooms in with Shift+plus. Zoom out there is an
/// unmodified minus, and next/trash arrows are unmodified, so those stay dead.
fn image_builtin(page_id: &str, app: &str) -> Option<Chord> {
    if page_id != "image.zoom" {
        return None;
    }
    let pair = match app {
        "loupe" => ("CTRL", "plus"),
        "imv" => ("SHIFT", "plus"),
        _ => return None,
    };
    Some(chord(pair.0, pair.1))
}

/// VS Code and VSCodium share the default keymap: Ctrl+S, Ctrl+W, Ctrl+PageDown,
/// Ctrl+backslash, Shift+Alt+F. Zed's linux defaults are ctrl-s, ctrl-w,
/// ctrl-pagedown, and ctrl-shift-i (`default-linux.json`). Zed's split is a
/// sequence. nvim, helix, and emacs are not one chord: helix Ctrl-S saves a
/// selection, emacs Ctrl-S is isearch, and `:w` / `C-x C-s` are sequences.
fn edit_builtin(page_id: &str, app: &str) -> Option<Chord> {
    if !matches!(app, "nvim" | "helix" | "code" | "codium" | "zed" | "emacs") {
        return None;
    }
    let vscode = matches!(app, "code" | "codium");
    let gui = vscode || app == "zed";
    let pair = match page_id {
        "edit.save" if gui => ("CTRL", "S"),
        "edit.close_tab" if gui => ("CTRL", "W"),
        "edit.next_tab" if gui => ("CTRL", "Page_Down"),
        "edit.split" if vscode => ("CTRL", "backslash"),
        "edit.format" if vscode => ("SHIFT + ALT", "F"),
        "edit.format" if app == "zed" => ("CTRL + SHIFT", "I"),
        _ => return None,
    };
    Some(chord(pair.0, pair.1))
}

/// Calc Guide: save is Ctrl+S, next sheet is Ctrl+PgDown. Zoom is the wheel.
fn office_builtin(page_id: &str, app: &str) -> Option<Chord> {
    if app != "libreoffice" {
        return None;
    }
    let pair = match page_id {
        "office.save" => ("CTRL", "S"),
        "office.next_sheet" => ("CTRL", "Page_Down"),
        _ => return None,
    };
    Some(chord(pair.0, pair.1))
}

/// GIMP, Krita, and Inkscape: Ctrl+S and Ctrl+Z. Krita zoom-in is Ctrl++
/// (`view_zoom_in` in `krita_default.shortcuts`). GIMP and Inkscape zoom keys
/// are unmodified. Inkscape export-to-png is Ctrl+Shift+E. Krita export has
/// no default. GIMP export is not png-specific. Darktable stays reserved.
fn draw_builtin(page_id: &str, app: &str) -> Option<Chord> {
    if !matches!(app, "gimp" | "krita" | "inkscape" | "darktable") {
        return None;
    }
    let painted = matches!(app, "gimp" | "krita" | "inkscape");
    let pair = match page_id {
        "draw.save" if painted => ("CTRL", "S"),
        "draw.undo" if painted => ("CTRL", "Z"),
        "draw.zoom" if app == "krita" => ("CTRL", "plus"),
        "draw.export_png" if app == "inkscape" => ("CTRL + SHIFT", "E"),
        _ => return None,
    };
    Some(chord(pair.0, pair.1))
}

/// Zen index binds are unset. `desk-browsers` walks a fiction chord whose key
/// is the filled `ws` id (`2`, not a constant `1`). Not a snap overlay, and
/// not this Hyprland config. Live snaps use id `live`.
fn suite_fiction(snap_id: &str, page_id: &str, app: &str, ws: Option<&str>) -> Option<Chord> {
    if snap_id == "desk-browsers" && page_id == "browser.zen_ws" && app == "zen" {
        let key = ws.filter(|k| matches!(*k, "1" | "2" | "3" | "4" | "5"))?;
        Some(chord("ALT + CTRL", key))
    } else {
        None
    }
}

/// Fiction chord, else builtin, then the empty-mods gate. Success is the final `Some`.
pub fn chord_for(
    page_id: &str,
    app: &str,
    snap_id: &str,
    slots: &BTreeMap<String, String>,
) -> Option<Chord> {
    let candidate = suite_fiction(snap_id, page_id, app, slots.get("ws").map(String::as_str))
        .or_else(|| builtin(page_id, app))?;
    arm(candidate)
}

/// Empty `mods` is dead unless the key is F11. Empty `key` is always dead.
fn arm(chord: Chord) -> Option<Chord> {
    if chord.key.is_empty() || (chord.mods.is_empty() && chord.key != "F11") {
        None
    } else {
        Some(chord)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn f11_empty_mods_and_private_firefox_only() {
        let fs = chord_for("browser.fullscreen", "firefox", "", &BTreeMap::new()).unwrap();
        assert_eq!(fs.mods, "");
        assert_eq!(fs.key, "F11");
        let p = builtin("browser.private", "firefox").unwrap();
        assert_eq!(p.mods, "CTRL + SHIFT");
        assert_eq!(p.key, "P");
        assert!(builtin("browser.private", "chrome").is_none());
        assert!(builtin("browser.private", "chromium").is_none());
        assert!(builtin("browser.private", "zen").is_none());
        assert!(builtin("browser.private", "brave").is_none());
        assert!(builtin("browser.tab_pin", "firefox").is_none());
    }

    #[test]
    fn empty_mods_other_than_f11_is_dead() {
        assert!(arm(chord("", "T")).is_none());
        assert!(arm(chord("CTRL", "")).is_none());
        let fs = arm(chord("", "F11")).unwrap();
        assert_eq!(fs.key, "F11");
        assert_eq!(fs.mods, "");
    }

    #[test]
    fn zen_chords_are_zen_only_and_index_is_fiction() {
        let next = chord_for("browser.zen_ws_next", "zen", "", &BTreeMap::new()).unwrap();
        assert_eq!(next.mods, "ALT + CTRL");
        assert_eq!(next.key, "E");
        let prev = builtin("browser.zen_ws_prev", "zen").unwrap();
        assert_eq!(prev.key, "Q");
        assert_eq!(builtin("browser.sidebar", "zen").unwrap().key, "S");
        assert_eq!(builtin("browser.split", "zen").unwrap().key, "H");
        assert_eq!(builtin("browser.unsplit", "zen").unwrap().key, "U");
        assert!(builtin("browser.zen_ws_next", "firefox").is_none());
        assert!(builtin("browser.compact", "zen").is_none());
        assert!(builtin("browser.glance_open", "zen").is_none());
        assert!(builtin("browser.glance_close", "zen").is_none());
        assert!(builtin("browser.zen_ws", "zen").is_none());
        assert!(builtin("browser.zen_ws_new", "zen").is_none());
        assert!(builtin("browser.container", "firefox").is_none());
        assert!(builtin("browser.container_tab", "firefox").is_none());
        assert!(builtin("browser.profile", "chrome").is_none());
        assert!(builtin("browser.profile_window", "chromium").is_none());
        assert!(builtin("browser.tab_group_collapse", "chrome").is_none());
        let empty = BTreeMap::new();
        assert!(chord_for("browser.compact", "zen", "desk-browsers", &empty).is_none());
        assert!(chord_for("browser.glance_close", "zen", "desk-browsers", &empty).is_none());
        assert!(chord_for("browser.zen_ws", "zen", "desk-browsers", &empty).is_none());
        let mut two = BTreeMap::new();
        two.insert("ws".into(), "2".into());
        let fiction = chord_for("browser.zen_ws", "zen", "desk-browsers", &two).unwrap();
        assert_eq!(fiction.mods, "ALT + CTRL");
        assert_eq!(fiction.key, "2");
        assert!(chord_for("browser.zen_ws", "zen", "desk-zen", &two).is_none());
        assert!(chord_for("browser.zen_ws", "zen", "live", &two).is_none());
        assert!(chord_for("browser.zen_ws", "firefox", "desk-browsers", &two).is_none());
    }

    #[test]
    fn term_chords_are_known_defaults_only() {
        let n = builtin("term.new_window", "foot").unwrap();
        assert_eq!(n.mods, "CTRL + SHIFT");
        assert_eq!(n.key, "N");
        assert_eq!(builtin("term.new_window", "kitty").unwrap().key, "N");
        assert_eq!(builtin("term.new_window", "ghostty").unwrap().key, "N");
        assert_eq!(builtin("term.new_window", "wezterm").unwrap().key, "N");
        assert!(builtin("term.new_window", "alacritty").is_none());
        assert!(builtin("term.new_tab", "foot").is_none());
        assert!(builtin("term.close", "foot").is_none());
        assert!(builtin("term.next", "foot").is_none());
        assert!(builtin("term.prev", "alacritty").is_none());
        assert_eq!(builtin("term.new_tab", "kitty").unwrap().key, "T");
        assert_eq!(builtin("term.close", "wezterm").unwrap().key, "W");
        assert_eq!(builtin("term.next", "kitty").unwrap().key, "Right");
        assert_eq!(builtin("term.prev", "kitty").unwrap().key, "Left");
        let gnext = builtin("term.next", "ghostty").unwrap();
        assert_eq!(gnext.mods, "CTRL");
        assert_eq!(gnext.key, "Tab");
        assert_eq!(
            builtin("term.prev", "wezterm").unwrap().mods,
            "CTRL + SHIFT"
        );
        assert_eq!(builtin("term.font", "foot").unwrap().key, "plus");
        assert_eq!(builtin("term.font", "alacritty").unwrap().mods, "CTRL");
        assert_eq!(builtin("term.font", "kitty").unwrap().key, "equal");
        assert_eq!(builtin("term.font", "kitty").unwrap().mods, "CTRL + SHIFT");
        assert_eq!(builtin("term.font", "wezterm").unwrap().key, "equal");
        assert!(builtin("term.font", "firefox").is_none());
        assert!(chord_for("term.new_tab", "foot", "desk-zen", &BTreeMap::new()).is_none());
    }

    #[test]
    fn desk_chords_are_documented_defaults_only() {
        assert_eq!(builtin("files.back", "nautilus").unwrap().key, "Left");
        assert_eq!(builtin("files.up", "dolphin").unwrap().mods, "ALT");
        assert_eq!(builtin("files.hidden", "thunar").unwrap().key, "H");
        assert!(builtin("files.hidden", "dolphin").is_none());
        assert!(builtin("files.back", "yazi").is_none());
        assert!(builtin("files.trash", "nautilus").is_none());
        assert!(builtin("files.sort", "nemo").is_none());
        assert_eq!(builtin("notes.daily", "logseq").unwrap().key, "J");
        assert!(builtin("notes.daily", "obsidian").is_none());
        assert!(builtin("notes.daily", "joplin").is_none());
        assert!(builtin("notes.sidebar", "joplin").is_none());
        assert!(builtin("notes.vault", "obsidian").is_none());
        assert_eq!(
            builtin("read.next_page", "evince").unwrap().key,
            "Page_Down"
        );
        assert_eq!(builtin("read.prev_page", "papers").unwrap().key, "Page_Up");
        assert_eq!(builtin("read.zoom", "foliate").unwrap().key, "plus");
        assert_eq!(builtin("read.dark", "zathura").unwrap().key, "R");
        assert!(builtin("read.next_page", "zathura").is_none());
        assert!(builtin("read.next_page", "foliate").is_none());
        assert!(builtin("read.zoom", "zathura").is_none());
        assert!(builtin("read.dark", "evince").is_none());
        assert!(builtin("read.dark", "papers").is_none());
        assert!(builtin("read.dark", "foliate").is_none());
        assert!(builtin("read.chapter_next", "foliate").is_none());
        assert!(builtin("read.chapter_prev", "evince").is_none());
    }

    #[test]
    fn edit_office_draw_chords_are_documented_defaults_only() {
        assert_eq!(builtin("edit.save", "code").unwrap().key, "S");
        assert_eq!(builtin("edit.save", "codium").unwrap().mods, "CTRL");
        assert_eq!(builtin("edit.save", "zed").unwrap().key, "S");
        assert!(builtin("edit.save", "nvim").is_none());
        assert!(builtin("edit.save", "helix").is_none());
        assert!(builtin("edit.save", "emacs").is_none());
        assert_eq!(builtin("edit.close_tab", "zed").unwrap().key, "W");
        assert_eq!(builtin("edit.next_tab", "code").unwrap().key, "Page_Down");
        assert_eq!(builtin("edit.split", "codium").unwrap().key, "backslash");
        assert!(builtin("edit.split", "zed").is_none());
        assert_eq!(builtin("edit.format", "code").unwrap().mods, "SHIFT + ALT");
        assert_eq!(builtin("edit.format", "zed").unwrap().key, "I");
        assert!(builtin("edit.format", "emacs").is_none());
        assert_eq!(builtin("office.save", "libreoffice").unwrap().key, "S");
        assert_eq!(
            builtin("office.next_sheet", "libreoffice").unwrap().key,
            "Page_Down"
        );
        assert!(builtin("office.zoom", "libreoffice").is_none());
        assert_eq!(builtin("draw.save", "gimp").unwrap().key, "S");
        assert_eq!(builtin("draw.undo", "krita").unwrap().key, "Z");
        assert_eq!(builtin("draw.zoom", "krita").unwrap().key, "plus");
        assert!(builtin("draw.zoom", "gimp").is_none());
        assert!(builtin("draw.zoom", "inkscape").is_none());
        assert!(builtin("draw.save", "darktable").is_none());
        assert!(builtin("draw.undo", "darktable").is_none());
        assert_eq!(builtin("draw.export_png", "inkscape").unwrap().key, "E");
        assert!(builtin("draw.export_png", "gimp").is_none());
        assert!(builtin("draw.export_png", "krita").is_none());
        assert!(builtin("draw.export_png", "darktable").is_none());
    }
}
