//! Power profile is `powerprofilesctl set` of one authored id.
//! Battery ask is a bucket. `None` is dead. There is no fan walk.

use std::collections::BTreeMap;

use serde_json::{json, Value};

use crate::types::{Page, Snap, WalkPlan};

pub fn is_live(page: &Page, snap: &Snap, slots: &BTreeMap<String, String>) -> (bool, String) {
    match page.id.as_str() {
        "power.profile" => profile_live(snap, slots),
        "power.ask_battery" => match battery_bucket(snap) {
            Some(bucket) => (true, format!("battery {bucket}")),
            None => (false, "battery missing".into()),
        },
        _ => (false, "unknown power page".into()),
    }
}

pub fn fill_walk(page: &Page, slots: &BTreeMap<String, String>, snap: &Snap) -> WalkPlan {
    let command = match page.id.as_str() {
        "power.profile" => profile_cmd(snap, slots).unwrap_or("reserved"),
        _ => "reserved",
    };
    WalkPlan {
        command: command.into(),
        driver: "power".into(),
    }
}

pub fn fill_ask(page: &Page, _slots: &BTreeMap<String, String>, snap: &Snap) -> Value {
    match page.id.as_str() {
        "power.ask_battery" => match battery_bucket(snap) {
            Some(bucket) => json!({ "bucket": bucket }),
            None => json!({ "unarmed": "power.ask_battery" }),
        },
        _ => json!({ "unarmed": page.id }),
    }
}

fn profile_live(snap: &Snap, slots: &BTreeMap<String, String>) -> (bool, String) {
    let Some(id) = slots.get("profile").map(String::as_str) else {
        return (false, "profile missing".into());
    };
    if profile_cmd(snap, slots).is_none() {
        if !matches!(id, "power-saver" | "balanced" | "performance") {
            return (false, "profile not authored".into());
        }
        if !listed(snap, id) {
            return (false, "profile not in list".into());
        }
        return (false, "reserved — powerprofilesctl absent".into());
    }
    (true, format!("profile {id}"))
}

fn profile_cmd(snap: &Snap, slots: &BTreeMap<String, String>) -> Option<&'static str> {
    if !snap.bins.iter().any(|b| b == "powerprofilesctl") {
        return None;
    }
    let id = slots.get("profile").map(String::as_str)?;
    if !listed(snap, id) {
        return None;
    }
    match id {
        "power-saver" => Some("powerprofilesctl set power-saver"),
        "balanced" => Some("powerprofilesctl set balanced"),
        "performance" => Some("powerprofilesctl set performance"),
        _ => None,
    }
}

fn listed(snap: &Snap, id: &str) -> bool {
    snap.lists
        .get("power_profile")
        .is_some_and(|names| names.iter().any(|n| n == id))
}

/// `None` is dead even on AC. On AC the bucket is `ac`. Otherwise the fraction.
fn battery_bucket(snap: &Snap) -> Option<&'static str> {
    let level = snap.battery?;
    if !(0.0..=1.0).contains(&level) {
        return None;
    }
    if snap.on_ac {
        return Some("ac");
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
    fn profile_is_the_enum_and_battery_is_a_bucket() {
        let cat = Catalog::load();
        let snap = cat.snap("desk-rig").unwrap();
        let saver = decide(&cat, "power saver profile", snap, RefereeKind::Lexical);
        assert_eq!(saver.take.page_id.as_deref(), Some("power.profile"));
        assert_eq!(
            saver.take.slots.get("profile").map(String::as_str),
            Some("power-saver")
        );
        assert!(!saver.confirm);
        let cmd = saver.walk.unwrap().command;
        assert_eq!(cmd, "powerprofilesctl set power-saver");
        assert!(!cmd.contains("fan"));
        assert!(!cmd.contains('%'));
        assert!(!cmd.contains("systemctl"));

        let balanced = decide(&cat, "balanced power profile", snap, RefereeKind::Lexical);
        assert_eq!(
            balanced.walk.unwrap().command,
            "powerprofilesctl set balanced"
        );
        let perf = decide(
            &cat,
            "performance power profile",
            snap,
            RefereeKind::Lexical,
        );
        assert_eq!(
            perf.walk.unwrap().command,
            "powerprofilesctl set performance"
        );

        let gaming = decide(&cat, "gaming power profile", snap, RefereeKind::Lexical);
        assert!(gaming.take.page_id.is_none(), "{:?}", gaming.take);
        assert!(gaming.walk.is_none());

        let ask = decide(&cat, "how's the battery", snap, RefereeKind::Lexical);
        assert_eq!(ask.take.page_id.as_deref(), Some("power.ask_battery"));
        assert!(ask.walk.is_none());
        let body = ask.answer.unwrap();
        assert_eq!(body, json!({ "bucket": "mid" }));
        assert!(body.get("percent").is_none(), "{body}");
        assert!(body.get("battery").is_none(), "{body}");

        let page = cat.page("power.ask_battery").unwrap();
        let desk = cat.snap("desk").unwrap();
        assert!(desk.battery.is_none());
        let (ok, why) = is_live(page, desk, &Default::default());
        assert!(!ok, "{why}");
        assert!(why.contains("missing"), "{why}");
        let dead = decide(&cat, "how's the battery", desk, RefereeKind::Lexical);
        assert!(dead.take.page_id.is_none(), "{:?}", dead.take);
        assert!(dead.answer.is_none());
        assert!(dead.walk.is_none());

        let mut plugged = snap.clone();
        plugged.on_ac = true;
        plugged.battery = None;
        assert!(battery_bucket(&plugged).is_none());
        plugged.battery = Some(0.1);
        assert_eq!(battery_bucket(&plugged), Some("ac"));
        plugged.on_ac = false;
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
            plugged.battery = Some(level);
            assert_eq!(battery_bucket(&plugged), Some(bucket), "{level}");
        }
        plugged.battery = Some(1.4);
        assert!(battery_bucket(&plugged).is_none());

        let profile = cat.page("power.profile").unwrap();
        let mut no_bin = snap.clone();
        no_bin.bins.retain(|b| b != "powerprofilesctl");
        let (ok, why) = is_live(profile, &no_bin, &saver.take.slots);
        assert!(!ok, "{why}");
        assert!(why.contains("powerprofilesctl"), "{why}");
        assert_eq!(
            fill_walk(profile, &saver.take.slots, &no_bin).command,
            "reserved"
        );
    }
}
