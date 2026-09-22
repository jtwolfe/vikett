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
}
