use std::collections::HashSet;

use crate::ask;
use crate::catalog::Catalog;
use crate::lexical::{self, AMBIGUITY_MARGIN, CONFIDENCE_FLOOR};
use crate::prune;
use crate::refuse;
use crate::text::{contains_phrase, norm, split_compound};
use crate::trace::{Trace, TraceNode, TraceStatus};
use crate::types::{EngineResult, PageKind, Policy, RefereeKind, Snap, Take, WalkPlan};
use crate::walk;

pub fn decide(cat: &Catalog, utterance: &str, snap: &Snap, referee: RefereeKind) -> EngineResult {
    decide_with_model(cat, utterance, snap, referee, None)
}

pub fn decide_with_model(
    cat: &Catalog,
    utterance: &str,
    snap: &Snap,
    referee: RefereeKind,
    model_take: Option<Take>,
) -> EngineResult {
    let mut trace = Trace::default();
    let n = norm(utterance);
    trace.push(TraceNode::new(
        "normalize",
        n.clone(),
        TraceStatus::Info,
        format!("snap={}", snap.id),
    ));

    let parts = split_compound(utterance);
    let remaining: Vec<String> = if parts.len() > 1 {
        parts[1..].to_vec()
    } else {
        Vec::new()
    };
    let first = parts.first().map(String::as_str).unwrap_or(utterance);
    if parts.len() > 1 {
        trace.push(TraceNode::new(
            "compound",
            format!("{} takes", parts.len()),
            TraceStatus::Info,
            format!("first={:?} rest={:?}", first, remaining),
        ));
    } else {
        trace.push(TraceNode::new(
            "compound",
            "single take",
            TraceStatus::Skip,
            "no and/and-then split",
        ));
    }

    if let Some(why) = refuse::refused(first) {
        let (live, dead) = prune::live_and_dead(&cat.pages, snap, Some(first));
        trace.push(TraceNode::new(
            "refuse",
            "hard refuse",
            TraceStatus::Fail,
            why,
        ));
        return EngineResult {
            utterance: utterance.into(),
            snap_id: snap.id.clone(),
            referee,
            live,
            dead,
            take: Take::silence(why),
            walk: None,
            answer: None,
            confirm: false,
            remaining,
            trace,
        };
    }
    trace.push(TraceNode::new(
        "refuse",
        "not a hard refuse",
        TraceStatus::Pass,
        "",
    ));

    let (live, dead) = prune::live_and_dead(&cat.pages, snap, Some(first));
    let live_ids: HashSet<&str> = live.iter().map(|l| l.page_id.as_str()).collect();
    let live_mods: HashSet<&str> = cat
        .pages
        .iter()
        .filter(|p| live_ids.contains(p.id.as_str()))
        .map(|p| p.module.as_str())
        .collect();

    let mut prune_node = TraceNode::new(
        "prune",
        format!("{} live / {} dead", live.len(), dead.len()),
        TraceStatus::Pass,
        format!("guest={} who={}", snap.guest, snap.who),
    );
    for l in live.iter().take(12) {
        prune_node = prune_node.child(TraceNode::new(
            "live",
            l.page_id.clone(),
            TraceStatus::Pass,
            l.why.clone(),
        ));
    }
    if live.len() > 12 {
        prune_node = prune_node.child(TraceNode::new(
            "live",
            format!("… {} more", live.len() - 12),
            TraceStatus::Info,
            "",
        ));
    }
    for d in dead.iter().filter(|d| {
        d.page_id.starts_with("mail.")
            || d.page_id.starts_with("calendar.")
            || d.page_id == "scene.handoff"
            || d.why.contains("guest")
            || d.why.contains("who empty")
            || d.why.contains("who unknown")
            || d.why.contains("owner-only")
            || d.why.contains("no timer")
    }) {
        prune_node = prune_node.child(TraceNode::new(
            "dead",
            d.page_id.clone(),
            TraceStatus::Fail,
            d.why.clone(),
        ));
    }
    trace.push(prune_node);

    let nfirst = norm(first);
    for (re, module) in [
        (r"\b(calendar|appointment|meetings?)\b", "calendar"),
        // `mail` is in the hint so a dead inbox does not slide onto media.next
        // via the bare alias `next` ("next unread mail").
        (r"\b(email|emails|inbox|mail)\b", "mail"),
    ] {
        if regex::Regex::new(re)
            .map(|r| r.is_match(&nfirst))
            .unwrap_or(false)
            && !live_mods.contains(module)
        {
            let why = format!("{module} not live — silence rather than a different door");
            trace.push(TraceNode::new(
                "module-hint",
                module,
                TraceStatus::Fail,
                &why,
            ));
            return EngineResult {
                utterance: utterance.into(),
                snap_id: snap.id.clone(),
                referee,
                live,
                dead,
                take: Take::silence(why),
                walk: None,
                answer: None,
                confirm: false,
                remaining,
                trace,
            };
        }
    }

    if let Some(why) = dead_alias_silence(cat, first, snap, &live_ids, &dead) {
        trace.push(TraceNode::new(
            "dead-alias",
            "silence",
            TraceStatus::Fail,
            why.clone(),
        ));
        return EngineResult {
            utterance: utterance.into(),
            snap_id: snap.id.clone(),
            referee,
            live,
            dead,
            take: Take::silence(why),
            walk: None,
            answer: None,
            confirm: false,
            remaining,
            trace,
        };
    }

    let mut take = match (referee, model_take) {
        (RefereeKind::Lexical, _) | (_, None) if referee == RefereeKind::Lexical => {
            lexical_take(cat, first, snap, &live, &live_ids, &mut trace)
        }
        (_, Some(model)) => {
            let mut node = TraceNode::new(
                "referee",
                referee.as_str(),
                TraceStatus::Info,
                model.reason.clone(),
            );
            node = node.child(TraceNode::new(
                "model",
                model.page_id.clone().unwrap_or_else(|| "none".into()),
                if model.page_id.is_some() {
                    TraceStatus::Pass
                } else {
                    TraceStatus::Fail
                },
                format!("conf={:.2}", model.confidence),
            ));
            trace.push(node);
            if let Some(id) = &model.page_id {
                if !live_ids.contains(id.as_str()) {
                    let why = format!("model picked {id} which is not live — silence");
                    trace.push(TraceNode::new("guard", "live-set", TraceStatus::Fail, &why));
                    Take::silence(why)
                } else if model.confidence < CONFIDENCE_FLOOR {
                    let why = "below threshold";
                    trace.push(TraceNode::new(
                        "threshold",
                        why,
                        TraceStatus::Fail,
                        format!("{}", model.confidence),
                    ));
                    Take {
                        page_id: None,
                        slots: Default::default(),
                        confidence: model.confidence,
                        reason: why.into(),
                    }
                } else {
                    merge_live_slots(model, &live)
                }
            } else {
                model
            }
        }
        _ => lexical_take(cat, first, snap, &live, &live_ids, &mut trace),
    };

    // "play the metal playlist" contains the bare alias "play". If that did
    // not take music.playlist, do not toggle whatever player is live.
    if contains_phrase(first, "playlist") && take.page_id.as_deref() != Some("music.playlist") {
        let why = "playlist utterance did not take music.playlist — silence";
        trace.push(TraceNode::new(
            "playlist",
            "silence",
            TraceStatus::Fail,
            why,
        ));
        take = Take::silence(why);
    }

    let mut walk_plan: Option<WalkPlan> = None;
    let mut answer = None;
    let mut confirm = false;
    if let Some(id) = &take.page_id {
        if let Some(page) = cat.page(id) {
            confirm = page.confirm || page.policy == Policy::Confirm;
            match page.kind {
                PageKind::Act => {
                    let plan = walk::fill_walk(page, &take.slots, snap);
                    trace.push(TraceNode::new(
                        "walk",
                        plan.command.clone(),
                        TraceStatus::Pass,
                        format!("driver={} confirm={confirm}", plan.driver),
                    ));
                    walk_plan = Some(plan);
                }
                PageKind::Ask => {
                    let val = ask::fill_ask(page, &take.slots, snap);
                    trace.push(TraceNode::new(
                        "ask",
                        page.id.clone(),
                        TraceStatus::Pass,
                        val.to_string(),
                    ));
                    answer = Some(val);
                }
            }
        }
    } else {
        trace.push(TraceNode::new(
            "take",
            "silence",
            TraceStatus::Fail,
            take.reason.clone(),
        ));
    }

    EngineResult {
        utterance: utterance.into(),
        snap_id: snap.id.clone(),
        referee,
        live,
        dead,
        take,
        walk: walk_plan,
        answer,
        confirm,
        remaining,
        trace,
    }
}

fn lexical_take(
    cat: &Catalog,
    first: &str,
    snap: &Snap,
    live: &[crate::types::LiveVikett],
    live_ids: &HashSet<&str>,
    trace: &mut Trace,
) -> Take {
    let scored = lexical::score_live(&cat.pages, live_ids, first, snap);
    let mut score_node = TraceNode::new(
        "referee",
        "lexical",
        TraceStatus::Info,
        format!("{} candidates above alias floor", scored.len()),
    );
    for s in scored.iter().take(8) {
        score_node = score_node.child(TraceNode::new(
            "score",
            format!("{}  {}", s.score, s.page_id),
            TraceStatus::Info,
            slot_brief(&s.slots),
        ));
    }
    trace.push(score_node);

    if scored.is_empty() {
        return Take::silence("no live door crossed threshold");
    }

    let top = &scored[0];
    if scored.len() > 1 {
        let second = &scored[1];
        if top.score - second.score < AMBIGUITY_MARGIN {
            let top_mod = module_of(cat, &top.page_id);
            let second_mod = module_of(cat, &second.page_id);
            if top_mod != second_mod {
                let why = format!("ambiguous {} vs {}", top.page_id, second.page_id);
                trace.push(TraceNode::new(
                    "ambiguity",
                    why.clone(),
                    TraceStatus::Fail,
                    format!("{} vs {}", top.score, second.score),
                ));
                return Take {
                    page_id: None,
                    slots: Default::default(),
                    confidence: 0.4,
                    reason: why,
                };
            }
        }
    }

    let lv = live.iter().find(|l| l.page_id == top.page_id);
    let mut slots = lv.map(|l| l.slots.clone()).unwrap_or_default();
    for (k, v) in &top.slots {
        slots.insert(k.clone(), v.clone());
    }
    let conf = lexical::confidence(top.score);
    if conf < CONFIDENCE_FLOOR {
        trace.push(TraceNode::new(
            "threshold",
            "below floor",
            TraceStatus::Fail,
            format!("{conf:.2} < {CONFIDENCE_FLOOR}"),
        ));
        return Take {
            page_id: None,
            slots: Default::default(),
            confidence: conf,
            reason: "below threshold".into(),
        };
    }

    trace.push(TraceNode::new(
        "take",
        top.page_id.clone(),
        TraceStatus::Pass,
        format!("conf={conf:.2} slots={}", slot_brief(&slots)),
    ));
    Take {
        page_id: Some(top.page_id.clone()),
        slots,
        confidence: conf,
        reason: "lexical referee (gold baseline — not a generative model)".into(),
    }
}

fn merge_live_slots(mut take: Take, live: &[crate::types::LiveVikett]) -> Take {
    if let Some(id) = &take.page_id {
        if let Some(lv) = live.iter().find(|l| l.page_id == *id) {
            for (k, v) in &lv.slots {
                take.slots.entry(k.clone()).or_insert_with(|| v.clone());
            }
        }
    }
    take
}

/// Longest `contains_phrase` alias on a dead family page silences when it is
/// strictly longer than every live alias. Equal length keeps the live door.
/// An app-missing dead page still counts in that guest case ("close tab");
/// it is skipped only when a live phrase already ties or beats its length,
/// so a dead `term.next` cannot veto a live `browser.tab_next`.
fn dead_alias_silence(
    cat: &Catalog,
    utterance: &str,
    snap: &Snap,
    live_ids: &HashSet<&str>,
    dead: &[crate::types::DeadVikett],
) -> Option<String> {
    let mut live_best = 0usize;
    for page in &cat.pages {
        if !live_ids.contains(page.id.as_str()) {
            continue;
        }
        for alias in &page.aliases {
            if contains_phrase(utterance, alias) {
                live_best = live_best.max(alias.len());
            }
        }
    }

    let mut best_len: Option<usize> = None;
    let mut best_count = 0usize;
    let mut best_alias = String::new();
    for d in dead {
        let Some(page) = cat.page(&d.page_id) else {
            continue;
        };
        if !crate::drivers::is_family(&page.module) {
            continue;
        }
        let filled = crate::slots::fill(page, utterance, snap);
        let app_missing =
            page.slots.iter().any(|s| s.id == "app") && !filled.slots.contains_key("app");
        for alias in &page.aliases {
            if !contains_phrase(utterance, alias) {
                continue;
            }
            if app_missing && live_best >= alias.len() {
                continue;
            }
            match best_len {
                Some(n) if alias.len() > n => {
                    best_len = Some(alias.len());
                    best_count = 1;
                    best_alias = alias.clone();
                }
                Some(n) if alias.len() == n => {
                    best_count += 1;
                }
                None => {
                    best_len = Some(alias.len());
                    best_count = 1;
                    best_alias = alias.clone();
                }
                _ => {}
            }
        }
    }

    let len = best_len?;
    if best_count != 1 || live_best >= len {
        return None;
    }
    Some(format!(
        "dead alias {best_alias} longer than live phrases — silence"
    ))
}

fn module_of<'a>(cat: &'a Catalog, id: &str) -> &'a str {
    cat.page(id).map(|p| p.module.as_str()).unwrap_or("")
}

fn slot_brief(slots: &std::collections::BTreeMap<String, String>) -> String {
    if slots.is_empty() {
        return "∅".into();
    }
    slots
        .iter()
        .map(|(k, v)| format!("{k}={v}"))
        .collect::<Vec<_>>()
        .join(" ")
}

pub fn run_golden(cat: &Catalog, g: &crate::types::Golden) -> GoldenRun {
    let Some(snap) = cat.snap(&g.snap) else {
        return GoldenRun {
            id: g.id.clone(),
            pass: false,
            got: Some("missing-snap".into()),
            expected: g.expect_page.clone(),
            slots_ok: true,
            detail: "snap not found".into(),
        };
    };
    let parts = split_compound(&g.utterance);
    let first = parts.first().map(String::as_str).unwrap_or(&g.utterance);
    let result = decide(cat, first, snap, RefereeKind::Lexical);
    let got = result.take.page_id.clone();
    let page_ok = got == g.expect_page;
    let mut slots_ok = true;
    if let Some(expected) = &g.expect_slots {
        for (k, v) in expected {
            if result.take.slots.get(k) != Some(v) {
                slots_ok = false;
            }
        }
    }
    GoldenRun {
        id: g.id.clone(),
        pass: page_ok && slots_ok,
        got,
        expected: g.expect_page.clone(),
        slots_ok,
        detail: result.take.reason.clone(),
    }
}

#[derive(Clone, Debug)]
pub struct GoldenRun {
    pub id: String,
    pub pass: bool,
    pub got: Option<String>,
    pub expected: Option<String>,
    pub slots_ok: bool,
    pub detail: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::catalog::Catalog;

    #[test]
    fn volume_little_takes_audio_bump() {
        let cat = Catalog::load();
        let snap = cat.snap("desk").unwrap();
        let r = decide(
            &cat,
            "increase the volume a bit",
            snap,
            RefereeKind::Lexical,
        );
        assert_eq!(r.take.page_id.as_deref(), Some("audio.bump"));
        assert_eq!(
            r.take.slots.get("direction").map(String::as_str),
            Some("up")
        );
        assert!(r.walk.as_ref().unwrap().command.contains("5%+"));
        assert!(r.trace.render().contains("audio.bump"));
    }

    #[test]
    fn confirm_is_flag_or_policy() {
        let mut cat = Catalog::load();
        let snap = cat.snap("desk").unwrap().clone();
        let closed = decide(&cat, "close this", &snap, RefereeKind::Lexical);
        assert_eq!(closed.take.page_id.as_deref(), Some("wm.close"));
        assert!(closed.confirm);
        let muted = decide(&cat, "mute", &snap, RefereeKind::Lexical);
        assert_eq!(muted.take.page_id.as_deref(), Some("audio.mute"));
        assert!(!muted.confirm);
        cat.pages
            .iter_mut()
            .find(|p| p.id == "audio.mute")
            .unwrap()
            .confirm = true;
        let flagged = decide(&cat, "mute", &snap, RefereeKind::Lexical);
        assert_eq!(flagged.take.page_id.as_deref(), Some("audio.mute"));
        assert!(flagged.confirm);
    }
}
