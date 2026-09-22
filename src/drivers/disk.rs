//! Disk ask is a free-space bucket. `None` is dead.
//! Timeshift create is owner and confirm. Format and delete are not walks.

use std::collections::BTreeMap;

use serde_json::{json, Value};

use crate::types::{Page, Snap, WalkPlan};

const TIMESHIFT_CREATE: &str = "timeshift --create --scripted";

pub fn is_live(page: &Page, snap: &Snap, _slots: &BTreeMap<String, String>) -> (bool, String) {
    match page.id.as_str() {
        "disk.ask" => match disk_bucket(snap.disk_free) {
            Some(bucket) => (true, format!("disk {bucket}")),
            None => (false, "disk free missing".into()),
        },
        "disk.timeshift" => {
            if snap.bins.iter().any(|b| b == "timeshift") {
                (true, "timeshift create".into())
            } else {
                (false, "reserved — timeshift absent".into())
            }
        }
        _ => (false, "unknown disk page".into()),
    }
}

pub fn fill_walk(page: &Page, _slots: &BTreeMap<String, String>, snap: &Snap) -> WalkPlan {
    let command = match page.id.as_str() {
        "disk.timeshift" if snap.bins.iter().any(|b| b == "timeshift") => TIMESHIFT_CREATE,
        _ => "reserved",
    };
    WalkPlan {
        command: command.into(),
        driver: "disk".into(),
    }
}

pub fn fill_ask(page: &Page, _slots: &BTreeMap<String, String>, snap: &Snap) -> Value {
    match page.id.as_str() {
        "disk.ask" => match disk_bucket(snap.disk_free) {
            Some(bucket) => json!({ "bucket": bucket }),
            None => json!({ "unarmed": "disk.ask" }),
        },
        _ => json!({ "unarmed": page.id }),
    }
}

/// Fraction free. Under 0.10 the disk is full, under 0.20 low, else ok.
fn disk_bucket(free: Option<f32>) -> Option<&'static str> {
    let free = free?;
    if !(0.0..=1.0).contains(&free) {
        return None;
    }
    Some(if free < 0.10 {
        "full"
    } else if free < 0.20 {
        "low"
    } else {
        "ok"
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::catalog::Catalog;
    use crate::decide::decide;
    use crate::prune;
    use crate::types::{Policy, RefereeKind};

    #[test]
    fn disk_bucket_and_timeshift_create_is_owner_confirm() {
        let cat = Catalog::load();
        let snap = cat.snap("desk-rig").unwrap();
        let page = cat.page("disk.timeshift").unwrap();
        assert_eq!(page.policy, Policy::Owner);
        assert!(page.confirm);
        assert_ne!(page.policy, Policy::Confirm);

        let shot = decide(&cat, "take a snapshot", snap, RefereeKind::Lexical);
        assert_eq!(shot.take.page_id.as_deref(), Some("disk.timeshift"));
        assert!(shot.confirm);
        let cmd = shot.walk.unwrap().command;
        assert_eq!(cmd, TIMESHIFT_CREATE);
        assert!(!cmd.contains("--delete"));
        assert!(!cmd.contains("format"));
        assert!(!cmd.contains("mkfs"));
        assert!(!cmd.contains("--restore"));
        assert!(!cmd.contains("sudo"));
        assert!(!cmd.contains("--comments"));

        let ask = decide(&cat, "how's the disk", snap, RefereeKind::Lexical);
        assert_eq!(ask.take.page_id.as_deref(), Some("disk.ask"));
        assert!(ask.walk.is_none());
        let body = ask.answer.unwrap();
        assert_eq!(body, json!({ "bucket": "ok" }));
        assert!(body.get("percent").is_none(), "{body}");
        assert!(body.get("path").is_none(), "{body}");

        let ask_page = cat.page("disk.ask").unwrap();
        let mut levels = snap.clone();
        for (free, bucket) in [
            (0.0, "full"),
            (0.09, "full"),
            (0.10, "low"),
            (0.19, "low"),
            (0.20, "ok"),
            (1.0, "ok"),
        ] {
            levels.disk_free = Some(free);
            assert_eq!(disk_bucket(levels.disk_free), Some(bucket), "{free}");
        }
        levels.disk_free = None;
        let (ok, why) = is_live(ask_page, &levels, &Default::default());
        assert!(!ok, "{why}");
        assert!(why.contains("missing"), "{why}");
        levels.disk_free = Some(1.5);
        assert!(disk_bucket(levels.disk_free).is_none());

        let desk = cat.snap("desk").unwrap();
        let dead = decide(&cat, "how's the disk", desk, RefereeKind::Lexical);
        assert!(dead.take.page_id.is_none(), "{:?}", dead.take);
        assert!(dead.answer.is_none());

        let guest = cat.snap("guest-living").unwrap();
        let status = prune::is_live(page, guest, &Default::default());
        assert!(!status.ok, "{}", status.why);
        assert!(status.why.contains("guest flag"), "{}", status.why);
        let hidden = decide(&cat, "take a snapshot", guest, RefereeKind::Lexical);
        assert!(hidden.take.page_id.is_none(), "{:?}", hidden.take);
        assert!(hidden.walk.is_none());

        let mut no_bin = snap.clone();
        no_bin.bins.retain(|b| b != "timeshift");
        let (ok, why) = is_live(page, &no_bin, &Default::default());
        assert!(!ok, "{why}");
        assert!(why.contains("timeshift"), "{why}");
        assert_eq!(
            fill_walk(page, &Default::default(), &no_bin).command,
            "reserved"
        );

        let mut other = snap.clone();
        other.who = "dave".into();
        let status = prune::is_live(page, &other, &Default::default());
        assert!(!status.ok);
        assert!(status.why.contains("owner-only"), "{}", status.why);
    }
}
