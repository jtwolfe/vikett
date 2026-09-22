use std::collections::BTreeMap;

use crate::prune::client_match;
use crate::types::{Page, Snap, WalkPlan};

pub fn fill_walk(page: &Page, slots: &BTreeMap<String, String>, snap: &Snap) -> WalkPlan {
    if crate::drivers::is_family(&page.module) {
        return crate::drivers::browser::fill_walk(page, slots, snap);
    }
    let command = match page.id.as_str() {
        "audio.bump" => {
            let dir = if slots.get("direction").map(String::as_str) == Some("down") {
                "-"
            } else {
                "+"
            };
            let step = if slots.get("amount").map(String::as_str) == Some("lot") {
                "10%"
            } else {
                "5%"
            };
            format!("wpctl set-volume -l 1.0 @DEFAULT_AUDIO_SINK@ {step}{dir}")
        }
        "audio.mute" => "wpctl set-mute @DEFAULT_AUDIO_SINK@ 1".into(),
        "audio.unmute" => "wpctl set-mute @DEFAULT_AUDIO_SINK@ 0".into(),
        "audio.mic_mute" => "wpctl set-mute @DEFAULT_AUDIO_SOURCE@ 1".into(),
        "audio.mic_unmute" => "wpctl set-mute @DEFAULT_AUDIO_SOURCE@ 0".into(),
        "audio.headphones" => "wpctl set-default <headphones-id>".into(),
        "audio.speakers" => "wpctl set-default <speakers-id>".into(),
        "audio.tv" => "wpctl set-default <hdmi-id>".into(),
        "wm.focus" => {
            let c = client_match(snap, slots.get("target").map(String::as_str));
            match c {
                Some(c) => format!("hyprctl dispatch focuswindow address:{}", c.address),
                None => page.walk.clone().unwrap_or_default(),
            }
        }
        "wm.fullscreen" => "hyprctl dispatch fullscreen 1".into(),
        "wm.workspace" => format!(
            "hyprctl dispatch workspace {}",
            slots.get("ws").map(String::as_str).unwrap_or("1")
        ),
        "wm.move_ws" => format!(
            "hyprctl dispatch movetoworkspace {}",
            slots.get("ws").map(String::as_str).unwrap_or("1")
        ),
        "wm.close" => "hyprctl dispatch killactive".into(),
        "wm.float" => "hyprctl dispatch togglefloating".into(),
        "wm.monitor" => "hyprctl dispatch focusmonitor +1".into(),
        "wm.swap_monitor" => "hyprctl dispatch swapactiveworkspaces current +1".into(),
        "wm.cycle_next" => "hyprctl dispatch cyclenext".into(),
        "wm.movefocus" => {
            let dir = match slots.get("dir").map(String::as_str) {
                Some("left") => "l",
                Some("right") => "r",
                Some("up") => "u",
                Some("down") => "d",
                _ => "r",
            };
            format!("hyprctl dispatch movefocus {dir}")
        }
        "wm.pin" => "hyprctl dispatch pin".into(),
        "wm.center" => "hyprctl dispatch centerwindow".into(),
        "launch.app" => {
            let app = slots.get("app").map(String::as_str).unwrap_or("");
            if snap.running.iter().any(|r| r == app) {
                if let Some(c) = client_match(snap, Some(app)) {
                    format!("hyprctl dispatch focuswindow address:{}", c.address)
                } else {
                    format!("focus {app}")
                }
            } else {
                format!("hyprctl dispatch exec {app}")
            }
        }
        "display.bump" => {
            let dir = if slots.get("direction").map(String::as_str) == Some("down") {
                "-"
            } else {
                "+"
            };
            let step = if slots.get("amount").map(String::as_str) == Some("lot") {
                "10%"
            } else {
                "5%"
            };
            format!("brightnessctl set {step}{dir}")
        }
        "display.night" => "hyprctl hyprsunset temperature 3500".into(),
        "climate.bump" => {
            let step = if slots.get("direction").map(String::as_str) == Some("down") {
                -1
            } else {
                1
            };
            format!("climate.set_temperature step {step}")
        }
        "climate.off" => "climate.turn_off".into(),
        "capture.screenshot" => {
            let out = snap.focused_output.as_deref().unwrap_or("focused");
            format!("grim -o {out} ~/Pictures/vikett.png")
        }
        "capture.region" => "grim -g \"$(slurp)\" ~/Pictures/vikett.png".into(),
        "session.lock" => "hyprlock".into(),
        "session.dpms" => {
            if slots.get("action").map(String::as_str) == Some("on") {
                "hyprctl dispatch dpms on".into()
            } else {
                "hyprctl dispatch dpms off".into()
            }
        }
        "notify.dismiss" => "makoctl dismiss".into(),
        "media.play_pause" => "playerctl play-pause".into(),
        "media.next" => "playerctl next".into(),
        "media.prev" => "playerctl previous".into(),
        "timer.add" => "timer daemon add 300".into(),
        "timer.start" => format!(
            "timer daemon start {}m",
            slots.get("mins").map(String::as_str).unwrap_or("5")
        ),
        "timer.cancel" => "timer daemon cancel".into(),
        "bucky.hide" => "buckyboi --hide".into(),
        "scene.apply" => format!(
            "Surface API apply_scene {}",
            slots.get("scene").map(String::as_str).unwrap_or("")
        ),
        "scene.lock_private" => "Surface API lock_private".into(),
        "scene.guest" => "session.privacy_state = shared_only".into(),
        "lights.bump" => "HA light.turn_on brightness_step".into(),
        "lights.off" => "HA light.turn_off".into(),
        _ => page.walk.clone().unwrap_or_else(|| page.id.clone()),
    };

    let driver = page.module.clone();
    WalkPlan { command, driver }
}
