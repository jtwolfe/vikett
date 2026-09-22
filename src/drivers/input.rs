//! Solaar battery is a bucket. `None` is dead. DPI stays reserved: there is
//! no confirmed chord and this driver does not invent `solaar config`.

use std::collections::BTreeMap;

use serde_json::{json, Value};

use crate::types::{Page, Snap, WalkPlan};

pub fn is_live(page: &Page, snap: &Snap, slots: &BTreeMap<String, String>) -> (bool, String) {
    match page.id.as_str() {
        "input.ask_battery" => match battery_bucket(snap.solaar_battery) {
            Some(bucket) => (true, format!("solaar {bucket}")),
            None => (false, "solaar battery missing".into()),
        },
        "input.dpi" => dpi_live(slots),
        _ => (false, "unknown input page".into()),
    }
}

pub fn fill_walk(_page: &Page, _slots: &BTreeMap<String, String>, _snap: &Snap) -> WalkPlan {
    WalkPlan {
        command: "reserved".into(),
        driver: "input".into(),
    }
}

pub fn fill_ask(page: &Page, _slots: &BTreeMap<String, String>, snap: &Snap) -> Value {
    match page.id.as_str() {
        "input.ask_battery" => match battery_bucket(snap.solaar_battery) {
            Some(bucket) => json!({ "bucket": bucket }),
            None => json!({ "unarmed": "input.ask_battery" }),
        },
        _ => json!({ "unarmed": page.id }),
    }
}

fn dpi_live(slots: &BTreeMap<String, String>) -> (bool, String) {
    let Some(app) = slots.get("app").map(String::as_str) else {
        return (false, "app missing".into());
    };
    if app != "solaar" {
        return (false, "not solaar".into());
    }
    (false, "reserved — no solaar chord".into())
}

/// Fraction. No AC bucket. `None` or out of range is dead.
fn battery_bucket(level: Option<f32>) -> Option<&'static str> {
    let level = level?;
    if !(0.0..=1.0).contains(&level) {
        return None;
    }
    Some(if level >= 0.90 {
        "full"
    } else if level >= 0.60 {
        "high"
    } else if level >= 0.30 {
        "mid"
    } else if level >= 0.15 {
        "low"
    } else {
        "critical"
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::catalog::Catalog;
    use crate::decide::decide;
    use crate::types::RefereeKind;

    #[test]
    fn solaar_battery_is_a_bucket_and_dpi_is_reserved() {
        let cat = Catalog::load();
        let snap = cat.snap("desk-rig").unwrap();
        let ask = decide(&cat, "mouse battery", snap, RefereeKind::Lexical);
        assert_eq!(ask.take.page_id.as_deref(), Some("input.ask_battery"));
        assert!(ask.walk.is_none());
        let body = ask.answer.unwrap();
        assert_eq!(body, json!({ "bucket": "high" }));
        assert!(body.get("percent").is_none(), "{body}");
        assert!(body.get("ok").is_none(), "{body}");

        let power = decide(&cat, "how's the battery", snap, RefereeKind::Lexical);
        assert_eq!(power.take.page_id.as_deref(), Some("power.ask_battery"));

        let page = cat.page("input.ask_battery").unwrap();
        let mut levels = snap.clone();
        for (level, bucket) in [
            (1.0, "full"),
            (0.90, "full"),
            (0.89, "high"),
            (0.60, "high"),
            (0.59, "mid"),
            (0.30, "mid"),
            (0.29, "low"),
            (0.15, "low"),
            (0.14, "critical"),
            (0.0, "critical"),
        ] {
            levels.solaar_battery = Some(level);
            assert_eq!(
                battery_bucket(levels.solaar_battery),
                Some(bucket),
                "{level}"
            );
        }
        levels.solaar_battery = None;
        let (ok, why) = is_live(page, &levels, &Default::default());
        assert!(!ok, "{why}");
        assert!(why.contains("missing"), "{why}");
        levels.solaar_battery = Some(1.4);
        assert!(battery_bucket(levels.solaar_battery).is_none());

        let desk = cat.snap("desk").unwrap();
        let dead = decide(&cat, "solaar battery", desk, RefereeKind::Lexical);
        assert!(dead.take.page_id.is_none(), "{:?}", dead.take);
        assert!(dead.answer.is_none());

        let dpi = cat.page("input.dpi").unwrap();
        let mut slots = BTreeMap::new();
        slots.insert("app".into(), "solaar".into());
        slots.insert("amount".into(), "little".into());
        let (ok, why) = is_live(dpi, snap, &slots);
        assert!(!ok, "{why}");
        assert!(why.contains("reserved"), "{why}");
        let walk = fill_walk(dpi, &slots, snap).command;
        assert_eq!(walk, "reserved");
        assert!(!walk.contains("solaar"));
        assert!(!walk.contains("dpi"));
        assert!(!walk.contains('%'));
        let raised = decide(&cat, "raise solaar dpi", snap, RefereeKind::Lexical);
        assert!(raised.take.page_id.is_none(), "{:?}", raised.take);
        assert!(raised.walk.is_none());
    }
}
