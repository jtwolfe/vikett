//! Stock chords and the one gate that arms them.
//! Hyprland 0.56 wants `hl.dsp.send_shortcut`, not `dispatch sendshortcut`.
//! `CTRL + SHIFT` is the dry-run spelling (spaces around `+`). Empty `mods` is
//! only legal for F11 — the field is still required by the dispatcher.

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

/// Zen index binds are unset. `desk-browsers` walks one fiction chord so the
/// suite can see a key. Not a snap overlay, and not this Hyprland config.
/// Live snaps use id `live`, so this never arms a real session.
fn suite_fiction(snap_id: &str, page_id: &str, app: &str) -> Option<Chord> {
    if snap_id == "desk-browsers" && page_id == "browser.zen_ws" && app == "zen" {
        Some(chord("ALT + CTRL", "1"))
    } else {
        None
    }
}

/// Fiction chord, else builtin, then the empty-mods gate. Success is the final `Some`.
pub fn chord_for(page_id: &str, app: &str, snap_id: &str) -> Option<Chord> {
    let candidate = suite_fiction(snap_id, page_id, app).or_else(|| builtin(page_id, app))?;
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
        let fs = chord_for("browser.fullscreen", "firefox", "").unwrap();
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
        let next = chord_for("browser.zen_ws_next", "zen", "").unwrap();
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
        assert!(chord_for("browser.compact", "zen", "desk-browsers").is_none());
        assert!(chord_for("browser.glance_close", "zen", "desk-browsers").is_none());
        let fiction = chord_for("browser.zen_ws", "zen", "desk-browsers").unwrap();
        assert_eq!(fiction.mods, "ALT + CTRL");
        assert_eq!(fiction.key, "1");
        assert!(chord_for("browser.zen_ws", "zen", "desk-zen").is_none());
        assert!(chord_for("browser.zen_ws", "zen", "live").is_none());
        assert!(chord_for("browser.zen_ws", "firefox", "desk-browsers").is_none());
    }
}
