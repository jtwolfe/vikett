//! Family drivers. v0 pages stay in the prune/walk/ask matches.

use std::collections::BTreeMap;

use serde_json::Value;

use crate::types::{Page, Snap, WalkPlan};

pub mod bluetooth;
pub mod boxes;
pub mod browser;
pub mod capture;
pub mod chat;
pub mod disk;
pub mod display;
pub mod draw;
pub mod edit;
pub mod files;
pub mod fx;
pub mod games;
pub(crate) mod hl;
pub mod image;
pub mod input;
pub mod look;
pub mod music;
pub mod network;
pub mod notes;
pub mod obs;
pub mod office;
pub mod power;
pub mod print;
pub mod read;
pub mod secrets;
pub mod session;
pub mod shelf;
pub mod sync;
pub mod term;
pub mod updates;
pub mod video;

pub fn is_family(module: &str) -> bool {
    matches!(
        module,
        "browser"
            | "term"
            | "files"
            | "notes"
            | "read"
            | "chat"
            | "session"
            | "display"
            | "music"
            | "video"
            | "image"
            | "edit"
            | "office"
            | "draw"
            | "shelf"
            | "look"
            | "fx"
            | "network"
            | "bluetooth"
            | "power"
            | "disk"
            | "updates"
            | "secrets"
            | "sync"
            | "boxes"
            | "games"
            | "obs"
            | "print"
            | "input"
    )
}

pub fn family_live(page: &Page, snap: &Snap, slots: &BTreeMap<String, String>) -> (bool, String) {
    match page.module.as_str() {
        "browser" => browser::is_live(page, snap, slots),
        "term" => term::is_live(page, snap, slots),
        "files" => files::is_live(page, snap, slots),
        "notes" => notes::is_live(page, snap, slots),
        "read" => read::is_live(page, snap, slots),
        "chat" => chat::is_live(page, snap, slots),
        "session" => session::is_live(page, snap, slots),
        "display" => display::is_live(page, snap, slots),
        "music" => music::is_live(page, snap, slots),
        "video" => video::is_live(page, snap, slots),
        "image" => image::is_live(page, snap, slots),
        "edit" => edit::is_live(page, snap, slots),
        "office" => office::is_live(page, snap, slots),
        "draw" => draw::is_live(page, snap, slots),
        "shelf" => shelf::is_live(page, snap, slots),
        "look" => look::is_live(page, snap, slots),
        "fx" => fx::is_live(page, snap, slots),
        "network" => network::is_live(page, snap, slots),
        "bluetooth" => bluetooth::is_live(page, snap, slots),
        "power" => power::is_live(page, snap, slots),
        "disk" => disk::is_live(page, snap, slots),
        "updates" => updates::is_live(page, snap, slots),
        "secrets" => secrets::is_live(page, snap, slots),
        "sync" => sync::is_live(page, snap, slots),
        "boxes" => boxes::is_live(page, snap, slots),
        "games" => games::is_live(page, snap, slots),
        "obs" => obs::is_live(page, snap, slots),
        "print" => print::is_live(page, snap, slots),
        "input" => input::is_live(page, snap, slots),
        _ => (false, "not a family".into()),
    }
}

pub fn family_walk(page: &Page, slots: &BTreeMap<String, String>, snap: &Snap) -> WalkPlan {
    match page.module.as_str() {
        "browser" => browser::fill_walk(page, slots, snap),
        "term" => term::fill_walk(page, slots, snap),
        "files" => files::fill_walk(page, slots, snap),
        "notes" => notes::fill_walk(page, slots, snap),
        "read" => read::fill_walk(page, slots, snap),
        "chat" => chat::fill_walk(page, slots, snap),
        "session" => session::fill_walk(page, slots, snap),
        "display" => display::fill_walk(page, slots, snap),
        "music" => music::fill_walk(page, slots, snap),
        "video" => video::fill_walk(page, slots, snap),
        "image" => image::fill_walk(page, slots, snap),
        "edit" => edit::fill_walk(page, slots, snap),
        "office" => office::fill_walk(page, slots, snap),
        "draw" => draw::fill_walk(page, slots, snap),
        "shelf" => shelf::fill_walk(page, slots, snap),
        "look" => look::fill_walk(page, slots, snap),
        "fx" => fx::fill_walk(page, slots, snap),
        "network" => network::fill_walk(page, slots, snap),
        "bluetooth" => bluetooth::fill_walk(page, slots, snap),
        "power" => power::fill_walk(page, slots, snap),
        "disk" => disk::fill_walk(page, slots, snap),
        "updates" => updates::fill_walk(page, slots, snap),
        "secrets" => secrets::fill_walk(page, slots, snap),
        "sync" => sync::fill_walk(page, slots, snap),
        "boxes" => boxes::fill_walk(page, slots, snap),
        "games" => games::fill_walk(page, slots, snap),
        "obs" => obs::fill_walk(page, slots, snap),
        "print" => print::fill_walk(page, slots, snap),
        "input" => input::fill_walk(page, slots, snap),
        _ => WalkPlan {
            command: "UNARMED".into(),
            driver: page.module.clone(),
        },
    }
}

pub fn family_ask(page: &Page, slots: &BTreeMap<String, String>, snap: &Snap) -> Value {
    match page.module.as_str() {
        "browser" => browser::fill_ask(page, slots, snap),
        "chat" => chat::fill_ask(page, slots, snap),
        "shelf" => shelf::fill_ask(page, slots, snap),
        "look" => look::fill_ask(page, slots, snap),
        "network" => network::fill_ask(page, slots, snap),
        "bluetooth" => bluetooth::fill_ask(page, slots, snap),
        "power" => power::fill_ask(page, slots, snap),
        "disk" => disk::fill_ask(page, slots, snap),
        "updates" => updates::fill_ask(page, slots, snap),
        "secrets" => secrets::fill_ask(page, slots, snap),
        "input" => input::fill_ask(page, slots, snap),
        // No term page is an ask. A later ask should get its own fill_ask.
        _ => serde_json::json!({ "unarmed": page.id }),
    }
}
