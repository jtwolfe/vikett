//! Family drivers. v0 pages stay in the prune/walk/ask matches.

use std::collections::BTreeMap;

use serde_json::Value;

use crate::types::{Page, Snap, WalkPlan};

pub mod browser;
pub mod capture;
pub mod chat;
pub mod display;
pub mod draw;
pub mod edit;
pub mod files;
pub mod fx;
pub(crate) mod hl;
pub mod image;
pub mod look;
pub mod music;
pub mod notes;
pub mod office;
pub mod read;
pub mod session;
pub mod shelf;
pub mod term;
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
        // No term page is an ask. A later ask should get its own fill_ask.
        _ => serde_json::json!({ "unarmed": page.id }),
    }
}
