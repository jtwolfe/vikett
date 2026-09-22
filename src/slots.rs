use std::collections::{BTreeMap, BTreeSet};

use crate::classes::app_for_class;
use crate::text::{contains_phrase, tokens};
use crate::types::{Page, Snap};

#[derive(Clone, Debug, Default)]
pub struct SlotFill {
    pub slots: BTreeMap<String, String>,
    pub missing: Vec<String>,
    /// Slot ids filled from the focused class, not the utterance.
    pub defaulted: BTreeSet<String>,
}

pub fn fill(page: &Page, utterance: &str, snap: &Snap) -> SlotFill {
    let mut slots = BTreeMap::new();
    let mut missing = Vec::new();
    let mut defaulted = BTreeSet::new();
    let utokens = tokens(utterance);

    for slot in &page.slots {
        let mut hit: Option<String> = None;
        let mut best = 0usize;
        for v in &slot.values {
            let mut candidates = Vec::with_capacity(2 + v.aliases.len());
            candidates.push(v.id.as_str());
            candidates.push(v.label.as_str());
            candidates.extend(v.aliases.iter().map(String::as_str));
            // The playlist id `focus` is also the wm verb. Only a phrase that
            // says playlist may fill it, or `focus loupe` binds the wrong slot.
            if slot.id == "playlist" {
                candidates.retain(|a| contains_phrase(a, "playlist"));
            }
            for a in candidates {
                if !contains_phrase(utterance, a)
                    && !utokens.iter().any(|t| t == &crate::text::norm(a))
                {
                    continue;
                }
                let score = a.len();
                if score >= best {
                    best = score;
                    hit = Some(v.id.clone());
                }
            }
        }
        if hit.is_none() && slot.id == "folder" && page.id == "browser.bookmark" {
            hit = list_hit(snap, "bookmark_folder", utterance);
        }
        if let Some(id) = hit {
            slots.insert(slot.id.clone(), id);
        } else if slot.id == "app" && crate::drivers::is_family(&page.module) {
            // Focused class only. `launch.app` is not a family and must not default.
            if let Some(app) = focused_app(snap) {
                if slot.values.iter().any(|v| v.id == app) {
                    slots.insert(slot.id.clone(), app.to_string());
                    defaulted.insert(slot.id.clone());
                } else {
                    missing.push(slot.id.clone());
                }
            } else if slot.required {
                missing.push(slot.id.clone());
            }
        } else if slot.required {
            missing.push(slot.id.clone());
        } else if slot.id == "target" && contains_phrase(utterance, "this") {
            slots.insert("target".into(), "active".into());
        } else if slot.id == "amount" {
            slots.insert("amount".into(), "little".into());
        }
    }

    SlotFill {
        slots,
        missing,
        defaulted,
    }
}

fn focused_app(snap: &Snap) -> Option<&'static str> {
    let class = snap.clients.iter().find(|c| c.focused)?.class.as_str();
    app_for_class(class)
}

fn list_hit(snap: &Snap, key: &str, utterance: &str) -> Option<String> {
    let names = snap.lists.get(key)?;
    let mut best: Option<&str> = None;
    for name in names {
        if name.is_empty() || !contains_phrase(utterance, name) {
            continue;
        }
        let longer = best.is_none_or(|b| name.len() > b.len());
        if longer {
            best = Some(name);
        }
    }
    best.map(str::to_string)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::catalog::Catalog;

    #[test]
    fn fills_volume_notch() {
        let cat = Catalog::load();
        let page = cat.page("audio.bump").unwrap();
        let snap = cat.snap("desk").unwrap();
        let fill = fill(page, "increase the volume a bit", snap);
        assert_eq!(fill.slots.get("direction").map(String::as_str), Some("up"));
        assert_eq!(fill.slots.get("amount").map(String::as_str), Some("little"));
        assert!(fill.missing.is_empty());
    }
}
