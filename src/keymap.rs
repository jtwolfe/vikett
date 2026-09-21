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

/// Builtin, then the empty-mods gate. Success is the final `Some`.
pub fn chord_for(page_id: &str, app: &str) -> Option<Chord> {
    arm(builtin(page_id, app)?)
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
        let fs = chord_for("browser.fullscreen", "firefox").unwrap();
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
}
