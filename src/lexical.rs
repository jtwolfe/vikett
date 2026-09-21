use std::collections::BTreeMap;

use crate::slots;
use crate::text::{contains_phrase, tokens};
use crate::types::{Page, Snap};

#[derive(Clone, Debug)]
pub struct Scored {
    pub page_id: String,
    pub score: i32,
    pub slots: BTreeMap<String, String>,
}

pub fn alias_score(page: &Page, utterance: &str) -> i32 {
    let mut best = 0i32;
    for a in &page.aliases {
        if contains_phrase(utterance, a) {
            best = best.max(8 + a.len() as i32);
        }
    }
    let u_tok: Vec<String> = tokens(utterance);
    for a in &page.aliases {
        let a_norm = crate::text::norm(a);
        let raw_words: Vec<&str> = a_norm
            .split_whitespace()
            .filter(|w| !w.is_empty())
            .collect();
        let a_tok = tokens(a);
        let hits: Vec<&String> = a_tok
            .iter()
            .filter(|t| u_tok.iter().any(|u| u == *t))
            .collect();
        // "screens on" collapses to ["screens"] because "on" is a stop word.
        // Do not treat that as a single-token alias of "screens".
        let single_ok =
            a_tok.len() == 1 && raw_words.len() == 1 && hits.len() == 1 && hits[0].len() >= 4;
        if hits.len() >= 2 || single_ok {
            best = best.max(3 * hits.len() as i32);
        }
    }
    best
}

pub fn score_live(
    pages: &[Page],
    live_ids: &std::collections::HashSet<&str>,
    utterance: &str,
    snap: &Snap,
) -> Vec<Scored> {
    let mut scored = Vec::new();
    for page in pages {
        if !live_ids.contains(page.id.as_str()) {
            continue;
        }
        let filled = slots::fill(page, utterance, snap);
        if !filled.missing.is_empty() {
            continue;
        }
        let alias = alias_score(page, utterance);
        let mut a = alias;
        if page.id == "launch.app"
            && regex_is(
                utterance,
                r"(?i)\b(switch to|go to|focus|show me|bring up)\b",
            )
            && filled.slots.contains_key("app")
        {
            a = a.max(8);
        }
        if a <= 0 {
            if distinctive_slot_hit(page, utterance, &filled.slots) {
                a = 5;
            } else {
                continue;
            }
        }
        let mut score = a;
        for (k, v) in &filled.slots {
            if !v.is_empty() {
                score += 4 + k.len() as i32;
            }
        }
        // Before the `a <= 0` guard this would put every browser page on "mute".
        if alias > 0 && filled.defaulted.contains("app") {
            score += FOCUS_PRIOR;
        }
        if page.kind == crate::types::PageKind::Ask
            && regex_is(
                utterance,
                r"(?i)\b(have i|any|what|how|is|where|which|who)\b",
            )
        {
            score += 6;
        }
        if page.id == "launch.app"
            && regex_is(
                utterance,
                r"(?i)\b(switch to|go to|focus|show me|bring up)\b",
            )
        {
            if let Some(app) = filled.slots.get("app") {
                if !snap.running.iter().any(|r| r == app) && snap.allowlist.iter().any(|r| r == app)
                {
                    score += 20;
                }
            }
        }
        scored.push(Scored {
            page_id: page.id.clone(),
            score,
            slots: filled.slots,
        });
    }
    scored.sort_by_key(|y| std::cmp::Reverse(y.score));
    scored
}

fn default_fill(k: &str, v: &str) -> bool {
    (k == "amount" && v == "little") || (k == "target" && v == "active")
}

/// Slot-only score: the utterance named a specific enum value, not just
/// up/down/off/on which are shared across modules.
fn distinctive_slot_hit(page: &Page, utterance: &str, filled: &BTreeMap<String, String>) -> bool {
    for slot in &page.slots {
        if matches!(slot.id.as_str(), "direction" | "amount" | "action") {
            continue;
        }
        let Some(id) = filled.get(&slot.id) else {
            continue;
        };
        if default_fill(&slot.id, id) {
            continue;
        }
        let Some(val) = slot.values.iter().find(|v| v.id == *id) else {
            continue;
        };
        let mut names =
            std::iter::once(val.label.as_str()).chain(val.aliases.iter().map(String::as_str));
        if names.any(|n| n.len() >= 5 && contains_phrase(utterance, n)) {
            return true;
        }
    }
    false
}

fn regex_is(hay: &str, pat: &str) -> bool {
    regex::Regex::new(pat)
        .map(|re| re.is_match(hay))
        .unwrap_or(false)
}

pub const CONFIDENCE_FLOOR: f32 = 0.62;
pub const AMBIGUITY_MARGIN: i32 = 3;
/// Focused-class `app` prior. Only after a real alias hit, never on a zero score.
pub const FOCUS_PRIOR: i32 = 4;

pub fn confidence(score: i32) -> f32 {
    (0.55 + score as f32 / 40.0).min(0.98)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::catalog::Catalog;
    use std::collections::HashSet;

    #[test]
    fn screens_on_does_not_match_swap_screens() {
        let cat = Catalog::load();
        let page = cat.page("session.dpms").unwrap();
        assert_eq!(alias_score(page, "swap screens"), 0);
        assert!(alias_score(page, "screens off") > 0);
    }

    #[test]
    fn quiet_hours_scores_via_slot() {
        let cat = Catalog::load();
        let snap = cat.snap("desk").unwrap();
        let live: HashSet<&str> = cat.pages.iter().map(|p| p.id.as_str()).collect();
        let scored = score_live(&cat.pages, &live, "quiet hours", snap);
        assert!(
            scored.iter().any(|s| s.page_id == "scene.apply"),
            "{:?}",
            scored.iter().map(|s| &s.page_id).collect::<Vec<_>>()
        );
    }
}
