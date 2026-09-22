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

fn window_sel(address: &str) -> String {
    if let Some(rest) = address.strip_prefix("address:") {
        format!("address:{rest}")
    } else {
        format!("address:{address}")
    }
}
