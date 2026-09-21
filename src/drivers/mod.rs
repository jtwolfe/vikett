//! Family drivers. v0 pages stay in the prune/walk/ask matches.

use std::collections::BTreeMap;

use serde_json::Value;

use crate::types::{Page, Snap, WalkPlan};

pub mod browser;
pub mod term;

pub fn is_family(module: &str) -> bool {
    matches!(module, "browser" | "term")
}

pub fn family_live(page: &Page, snap: &Snap, slots: &BTreeMap<String, String>) -> (bool, String) {
    match page.module.as_str() {
        "browser" => browser::is_live(page, snap, slots),
        "term" => term::is_live(page, snap, slots),
        _ => (false, "not a family".into()),
    }
}

pub fn family_walk(page: &Page, slots: &BTreeMap<String, String>, snap: &Snap) -> WalkPlan {
    match page.module.as_str() {
        "browser" => browser::fill_walk(page, slots, snap),
        "term" => term::fill_walk(page, slots, snap),
        _ => WalkPlan {
            command: "UNARMED".into(),
            driver: page.module.clone(),
        },
    }
}

pub fn family_ask(page: &Page, slots: &BTreeMap<String, String>, snap: &Snap) -> Value {
    match page.module.as_str() {
        "browser" => browser::fill_ask(page, slots, snap),
        // No term page is an ask. A later ask should get its own fill_ask.
        _ => serde_json::json!({ "unarmed": page.id }),
    }
}
