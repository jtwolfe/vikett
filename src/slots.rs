use std::collections::BTreeMap;

use crate::text::{contains_phrase, tokens};
use crate::types::Page;

#[derive(Clone, Debug, Default)]
pub struct SlotFill {
    pub slots: BTreeMap<String, String>,
    pub missing: Vec<String>,
}

pub fn fill(page: &Page, utterance: &str) -> SlotFill {
    let mut slots = BTreeMap::new();
    let mut missing = Vec::new();
    let utokens = tokens(utterance);

    for slot in &page.slots {
        let mut hit: Option<String> = None;
        let mut best = 0usize;
        for v in &slot.values {
            let mut candidates = Vec::with_capacity(2 + v.aliases.len());
            candidates.push(v.id.as_str());
            candidates.push(v.label.as_str());
            candidates.extend(v.aliases.iter().map(String::as_str));
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
        if let Some(id) = hit {
            slots.insert(slot.id.clone(), id);
        } else if slot.required {
            missing.push(slot.id.clone());
        } else if slot.id == "target" && contains_phrase(utterance, "this") {
            slots.insert("target".into(), "active".into());
        } else if slot.id == "amount" {
            slots.insert("amount".into(), "little".into());
        }
    }

    SlotFill { slots, missing }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::catalog::Catalog;

    #[test]
    fn fills_volume_notch() {
        let cat = Catalog::load();
        let page = cat.page("audio.bump").unwrap();
        let fill = fill(page, "increase the volume a bit");
        assert_eq!(fill.slots.get("direction").map(String::as_str), Some("up"));
        assert_eq!(fill.slots.get("amount").map(String::as_str), Some("little"));
        assert!(fill.missing.is_empty());
    }
}
