//! Laya / EdgeJev referee.
//!
//! This is a System 1 typed-decision call (`choice` + `noul`), not a chat LLM.
//! The engine never sends walk/argv. Criteria are live page labels only.
//!
//! Runtime: POST `{LAYA_URL}/v1/systemone` (EdgeJev `serve`, official Jev-compatible).
//! Python is used to *export/quantize/fine-tune* weights, not to interpret Vikett.

use std::collections::{BTreeMap, HashSet};
use std::net::ToSocketAddrs;
use std::time::Duration;

use anyhow::{anyhow, Context, Result};
use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::lexical::{self, Scored};
use crate::types::{LiveVikett, Page, RefereeKind, Snap, Take};

const SHORTLIST: usize = 12;
const CHOICE_FLOOR: f32 = 0.62;
const CHOICE_MARGIN: f32 = 0.08;
const COMPOUND_NOUL: f32 = 0.55;

#[derive(Clone, Debug)]
pub struct ModelConfig {
    pub kind: RefereeKind,
    pub laya_url: String,
}

impl Default for ModelConfig {
    fn default() -> Self {
        let laya_url = std::env::var("VIKETT_LAYA_URL")
            .or_else(|_| std::env::var("TYPESAFE_BASE_URL"))
            .unwrap_or_else(|_| "http://127.0.0.1:8009".into());
        Self {
            kind: RefereeKind::Laya,
            laya_url,
        }
    }
}

pub fn available(kind: RefereeKind) -> bool {
    match kind {
        RefereeKind::Lexical => true,
        RefereeKind::Laya => laya_up(&ModelConfig::default().laya_url),
    }
}

fn laya_up(base: &str) -> bool {
    let url = format!("{}/health", base.trim_end_matches('/'));
    if ping_http(&url) {
        return true;
    }
    // EdgeJev may not expose /health; the port being open is enough to try.
    if let Some((host, port)) = host_port(base) {
        let addr = format!("{host}:{port}");
        if let Ok(mut addrs) = addr.to_socket_addrs() {
            if let Some(a) = addrs.next() {
                return std::net::TcpStream::connect_timeout(&a, Duration::from_millis(80)).is_ok();
            }
        }
    }
    false
}

fn host_port(base: &str) -> Option<(String, u16)> {
    let rest = base
        .trim()
        .trim_start_matches("http://")
        .trim_start_matches("https://");
    let rest = rest.split('/').next()?;
    match rest.rsplit_once(':') {
        Some((h, p)) => Some((h.to_string(), p.parse().ok()?)),
        None => Some((rest.to_string(), 80)),
    }
}

fn ping_http(url: &str) -> bool {
    client()
        .ok()
        .and_then(|c| c.get(url).send().ok())
        .map(|r| r.status().is_success() || r.status().as_u16() == 404)
        .unwrap_or(false)
}

pub fn take(
    kind: RefereeKind,
    utterance: &str,
    snap: &Snap,
    live: &[LiveVikett],
    pages: &[Page],
) -> Result<Take> {
    match kind {
        RefereeKind::Lexical => Err(anyhow!("lexical referee does not call a model")),
        RefereeKind::Laya => laya_take(utterance, snap, live, pages),
    }
}

pub fn laya_take(
    utterance: &str,
    snap: &Snap,
    live: &[LiveVikett],
    pages: &[Page],
) -> Result<Take> {
    let cfg = ModelConfig::default();
    let live_ids: HashSet<&str> = live.iter().map(|l| l.page_id.as_str()).collect();
    let scored = lexical::score_live(pages, &live_ids, utterance, snap);
    // Lexical is the shortlist. An empty shortlist must not be padded with
    // unrelated live pages (that made Laya walk wm.focus on dead/guest doors).
    if scored.is_empty() {
        return Ok(Take::silence(
            "no lexical shortlist — Laya not offered live pages",
        ));
    }
    let criteria = criteria_map(live, &scored);
    let shortlist: Vec<&LiveVikett> = scored
        .iter()
        .take(SHORTLIST)
        .filter_map(|s| live.iter().find(|l| l.page_id == s.page_id))
        .collect();
    let state = json!({
        "who": snap.who,
        "where": snap.place,
        "guest": snap.guest,
        "utterance": utterance,
        "live": shortlist.iter().map(|l| json!({
            "id": l.page_id,
            "label": l.label,
        })).collect::<Vec<_>>(),
    });
    let questions = json!({
        "page": {
            "type": "choice",
            "instructions": "Which live vikett did they mean? Pick none if unsure, if the door is not live, or if they asked to send/buy/click/invent a number.",
            "criteria": criteria,
        },
        "compound": {
            "type": "noul",
            "instructions": "Does the utterance contain a second action after this take (e.g. 'and then fullscreen')?",
        },
    });
    let body = json!({
        "model": "laya",
        "state": state,
        "questions": questions,
    });
    let url = format!("{}/v1/systemone", cfg.laya_url.trim_end_matches('/'));
    let resp: SystemOneResponse = client()?
        .post(&url)
        .header(
            "Authorization",
            format!(
                "Bearer {}",
                std::env::var("TYPESAFE_API_KEY").unwrap_or_else(|_| "local".into())
            ),
        )
        .json(&body)
        .send()
        .with_context(|| {
            format!(
                "Laya POST {url} — no referee on that port. Build once with ./scripts/setup_edgejev.sh then: edgejev serve --model ./jev-int8 --port 8009"
            )
        })?
        .error_for_status()
        .with_context(|| "Laya /v1/systemone")?
        .json()
        .context("parse Laya response")?;

    let page = resp
        .answers
        .get("page")
        .ok_or_else(|| anyhow!("Laya response missing answers.page"))?;
    let choice = page
        .choice
        .as_deref()
        .filter(|id| *id != "none" && !id.is_empty());
    let probs = page.probabilities.clone().unwrap_or_default();
    let top_p = choice
        .and_then(|id| probs.get(id).copied())
        .unwrap_or(page.confidence.unwrap_or(0.0));
    let second_p = probs
        .iter()
        .filter(|(k, _)| Some(k.as_str()) != choice)
        .map(|(_, v)| *v)
        .fold(0.0f32, f32::max);

    if choice.is_none() {
        return Ok(Take::silence("Laya chose none"));
    }
    // Walk on the choice probability, not Laya's separate `confidence` scalar
    // (that field is often ~0.1–0.4 even when P(choice) is 0.85).
    if top_p < CHOICE_FLOOR || (top_p - second_p) < CHOICE_MARGIN {
        return Ok(Take {
            page_id: None,
            slots: BTreeMap::new(),
            confidence: top_p,
            reason: format!("Laya below floor/margin (p={top_p:.2} second={second_p:.2})"),
        });
    }
    let id = choice.unwrap();
    if !live.iter().any(|l| l.page_id == id) {
        return Ok(Take::silence(format!("Laya picked {id} which is not live")));
    }
    let mut slots = BTreeMap::new();
    if let Some(lv) = live.iter().find(|l| l.page_id == id) {
        slots = lv.slots.clone();
    }
    let compound = resp
        .answers
        .get("compound")
        .and_then(|a| a.noul)
        .unwrap_or(0.0);
    let reason = if compound >= COMPOUND_NOUL {
        format!("Laya choice {id} (compound noul={compound:.2})")
    } else {
        format!("Laya choice {id}")
    };
    Ok(Take {
        page_id: Some(id.to_string()),
        slots,
        confidence: top_p,
        reason,
    })
}

fn criteria_map(live: &[LiveVikett], scored: &[Scored]) -> BTreeMap<String, String> {
    let mut out = BTreeMap::new();
    let mut order: Vec<&LiveVikett> = Vec::new();
    for s in scored.iter().take(SHORTLIST) {
        if let Some(l) = live.iter().find(|l| l.page_id == s.page_id) {
            order.push(l);
        }
    }
    for l in order {
        let label = if l.slots.is_empty() {
            l.label.clone()
        } else {
            let slots: Vec<String> = l.slots.iter().map(|(k, v)| format!("{k}={v}")).collect();
            format!("{} ({})", l.label, slots.join(", "))
        };
        out.insert(l.page_id.clone(), label);
    }
    out.insert(
        "none".into(),
        "refuse, not live, or not enough confidence".into(),
    );
    out
}

#[derive(Debug, Deserialize)]
struct SystemOneResponse {
    #[serde(default)]
    answers: BTreeMap<String, Answer>,
}

#[derive(Debug, Deserialize)]
struct Answer {
    #[serde(default)]
    choice: Option<String>,
    #[serde(default)]
    probabilities: Option<BTreeMap<String, f32>>,
    #[serde(default)]
    confidence: Option<f32>,
    #[serde(default)]
    noul: Option<f32>,
}

fn client() -> Result<reqwest::blocking::Client> {
    Ok(reqwest::blocking::Client::builder()
        .timeout(Duration::from_secs(8))
        .build()?)
}

#[derive(Serialize)]
pub struct ModelEvalRow {
    pub id: String,
    pub utterance: String,
    pub expected: Option<String>,
    pub got: Option<String>,
    pub pass: bool,
    pub reason: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::LiveVikett;

    #[test]
    fn criteria_always_include_none() {
        let live = [LiveVikett {
            page_id: "audio.bump".into(),
            label: "Nudge volume".into(),
            slots: BTreeMap::from([("direction".into(), "up".into())]),
            why: "sink".into(),
        }];
        let scored = [crate::lexical::Scored {
            page_id: "audio.bump".into(),
            score: 12,
            slots: BTreeMap::new(),
        }];
        let map = criteria_map(&live, &scored);
        assert!(map.contains_key("none"));
        assert!(map.contains_key("audio.bump"));
        assert!(!map
            .values()
            .any(|v| v.contains("hyprctl") || v.contains("wpctl")));
    }

    #[test]
    fn empty_shortlist_is_none_only() {
        let live = [LiveVikett {
            page_id: "wm.focus".into(),
            label: "Focus a window".into(),
            slots: BTreeMap::new(),
            why: "client".into(),
        }];
        let map = criteria_map(&live, &[]);
        assert_eq!(map.len(), 1);
        assert!(map.contains_key("none"));
        assert!(!map.contains_key("wm.focus"));
    }

    #[test]
    fn other_monitor_does_not_walk_focus() {
        let cat = crate::catalog::Catalog::load();
        let snap = cat.snap("desk").unwrap();
        let (live, _) = crate::prune::live_and_dead(&cat.pages, snap, Some("other monitor"));
        let take = laya_take("other monitor", snap, &live, &cat.pages).unwrap();
        assert!(take.page_id.is_none(), "got {:?}", take.page_id);
        assert!(take.reason.contains("shortlist"), "{}", take.reason);
    }

    struct LayaUrlGuard {
        prev: Option<String>,
    }

    impl LayaUrlGuard {
        fn set(url: &str) -> Self {
            let prev = std::env::var("VIKETT_LAYA_URL").ok();
            std::env::set_var("VIKETT_LAYA_URL", url);
            Self { prev }
        }
    }

    impl Drop for LayaUrlGuard {
        fn drop(&mut self) {
            match self.prev.take() {
                Some(v) => std::env::set_var("VIKETT_LAYA_URL", v),
                None => std::env::remove_var("VIKETT_LAYA_URL"),
            }
        }
    }

    #[test]
    fn empty_shortlist_does_not_post() {
        let cat = crate::catalog::Catalog::load();
        let snap = cat.snap("desk").unwrap();
        let (live, _) = crate::prune::live_and_dead(&cat.pages, snap, Some("other monitor"));
        // Port 1 refuses. A POST would error; silence must return before the client.
        let _guard = LayaUrlGuard::set("http://127.0.0.1:1");
        let take = laya_take("other monitor", snap, &live, &cat.pages)
            .expect("empty shortlist is silence, not a client error");
        assert!(take.page_id.is_none(), "{take:?}");
        assert!(take.reason.contains("shortlist"), "{}", take.reason);
    }

    #[test]
    fn guest_notify_does_not_walk_focus() {
        let cat = crate::catalog::Catalog::load();
        let snap = cat.snap("guest-living").unwrap();
        let (live, _) = crate::prune::live_and_dead(&cat.pages, snap, Some("last notification"));
        assert!(
            !live.iter().any(|l| l.page_id.starts_with("notify.")),
            "guest leaked notify"
        );
        let take = laya_take("last notification", snap, &live, &cat.pages).unwrap();
        assert!(take.page_id.is_none(), "got {:?}", take.page_id);
    }
}
