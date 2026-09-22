//! Hyprland 0.56 dry-run strings. `exec_cmd` is bash -c, so a binary plus an
//! authored path is one string. Never the old `dispatch exec`.

use crate::keymap::Chord;

pub fn focus_cmd(address: &str) -> String {
    let window = window_sel(address);
    format!("hyprctl dispatch 'hl.dsp.focus({{ window = \"{window}\" }})'")
}

pub fn shortcut_cmd(chord: &Chord, address: &str) -> String {
    let window = window_sel(address);
    format!(
        "hyprctl dispatch 'hl.dsp.send_shortcut({{ mods = \"{mods}\", key = \"{key}\", window = \"{window}\" }})'",
        mods = chord.mods,
        key = chord.key,
    )
}

pub fn exec_cmd(cmd: &str) -> String {
    format!("hyprctl dispatch 'hl.dsp.exec_cmd(\"{cmd}\")'")
}

/// Dwindle `splitratio` is a layout message (`"splitratio +0.1"`), not
/// `dispatch splitratio`. Little is 0.1, lot is 0.25. The sign is the notch
/// direction. `hl.dsp.layout` does not switch the layout algorithm; the enum
/// message below is the authored token for `wm.layout`, not a mode string.
pub fn layout_msg(message: &str) -> String {
    format!("hyprctl dispatch 'hl.dsp.layout(\"{message}\")'")
}

pub fn split_ratio_msg(wider: bool, lot: bool) -> String {
    let sign = if wider { "+" } else { "-" };
    let mag = if lot { "0.25" } else { "0.1" };
    layout_msg(&format!("splitratio {sign}{mag}"))
}

pub fn group_next_cmd() -> String {
    "hyprctl dispatch 'hl.dsp.group.next()'".into()
}

fn window_sel(address: &str) -> String {
    if let Some(rest) = address.strip_prefix("address:") {
        format!("address:{rest}")
    } else {
        format!("address:{address}")
    }
}
