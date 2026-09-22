//! Pending updates are a bucket. `None` is dead.
//! Upgrade is one fixed full-upgrade command, owner and confirm.
//! A package name is not an argument.

use std::collections::BTreeMap;

use serde_json::{json, Value};

use crate::types::{Page, Snap, WalkPlan};

pub fn is_live(page: &Page, snap: &Snap, _slots: &BTreeMap<String, String>) -> (bool, String) {
    match page.id.as_str() {
        "updates.ask" => match snap.updates_pending {
            Some(n) => (true, format!("pending {n}")),
            None => (false, "updates pending missing".into()),
        },
        "updates.upgrade" => match upgrade_cmd(snap) {
            Some(cmd) => (true, cmd.into()),
            None => (false, "reserved — no full upgrade command".into()),
        },
        _ => (false, "unknown updates page".into()),
    }
}

pub fn fill_walk(page: &Page, _slots: &BTreeMap<String, String>, snap: &Snap) -> WalkPlan {
    let command = match page.id.as_str() {
        "updates.upgrade" => upgrade_cmd(snap).unwrap_or("reserved"),
        _ => "reserved",
    };
    WalkPlan {
        command: command.into(),
        driver: "updates".into(),
    }
}

pub fn fill_ask(page: &Page, _slots: &BTreeMap<String, String>, snap: &Snap) -> Value {
    match page.id.as_str() {
        "updates.ask" => match snap.updates_pending {
            Some(n) => json!({ "bucket": pending_bucket(n) }),
            None => json!({ "unarmed": "updates.ask" }),
        },
        _ => json!({ "unarmed": page.id }),
    }
}

/// Known full-upgrade commands only. `apt upgrade` is partial and is not used.
fn upgrade_cmd(snap: &Snap) -> Option<&'static str> {
    if snap.bins.iter().any(|b| b == "pacman") {
        Some("pacman -Syu")
    } else if snap.bins.iter().any(|b| b == "dnf") {
        Some("dnf upgrade")
    } else if snap.bins.iter().any(|b| b == "apt") {
        Some("apt full-upgrade")
    } else {
        None
    }
}

fn pending_bucket(n: u32) -> &'static str {
    if n == 0 {
        "none"
    } else if n <= 3 {
        "few"
    } else {
        "many"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::catalog::Catalog;
    use crate::decide::decide;
    use crate::prune;
    use crate::types::{Policy, RefereeKind};

    #[test]
    fn pending_bucket_and_upgrade_is_owner_confirm() {
        let cat = Catalog::load();
        let snap = cat.snap("desk-rig").unwrap();
        let page = cat.page("updates.upgrade").unwrap();
        assert_eq!(page.policy, Policy::Owner);
        assert!(page.confirm);
        assert_ne!(page.policy, Policy::Confirm);

        let up = decide(&cat, "upgrade the system", snap, RefereeKind::Lexical);
        assert_eq!(up.take.page_id.as_deref(), Some("updates.upgrade"));
        assert!(up.confirm);
        let cmd = up.walk.unwrap().command;
        assert_eq!(cmd, "pacman -Syu");
        assert!(!cmd.contains("firefox"));
        assert!(!cmd.contains("sudo"));
        assert!(!cmd.contains("-y"));
        assert!(!cmd.contains("noconfirm"));
        assert!(!cmd.contains("install"));

        let ask = decide(&cat, "how many updates", snap, RefereeKind::Lexical);
        assert_eq!(ask.take.page_id.as_deref(), Some("updates.ask"));
        assert!(ask.walk.is_none());
        let body = ask.answer.unwrap();
        assert_eq!(body, json!({ "bucket": "few" }));
        assert!(body.get("packages").is_none(), "{body}");
        assert!(body.get("percent").is_none(), "{body}");

        let ask_page = cat.page("updates.ask").unwrap();
        let mut levels = snap.clone();
        for (n, bucket) in [(0, "none"), (1, "few"), (3, "few"), (4, "many")] {
            levels.updates_pending = Some(n);
            assert_eq!(
                fill_ask(ask_page, &Default::default(), &levels)["bucket"],
                json!(bucket),
                "{n}"
            );
        }
        levels.updates_pending = None;
        let (ok, why) = is_live(ask_page, &levels, &Default::default());
        assert!(!ok, "{why}");
        assert!(why.contains("missing"), "{why}");

        let desk = cat.snap("desk").unwrap();
        let dead = decide(&cat, "how many updates", desk, RefereeKind::Lexical);
        assert!(dead.take.page_id.is_none(), "{:?}", dead.take);
        assert!(dead.answer.is_none());
        assert!(dead.walk.is_none());

        let guest = cat.snap("guest-living").unwrap();
        assert_eq!(guest.who, "jim");
        let status = prune::is_live(page, guest, &Default::default());
        assert!(!status.ok, "{}", status.why);
        assert!(status.why.contains("guest flag"), "{}", status.why);
        let hidden = decide(&cat, "upgrade the system", guest, RefereeKind::Lexical);
        assert!(hidden.take.page_id.is_none(), "{:?}", hidden.take);
        assert!(hidden.walk.is_none());

        let mut blank = snap.clone();
        blank.guest = false;
        blank.who = String::new();
        let status = prune::is_live(page, &blank, &Default::default());
        assert!(!status.ok);
        assert!(status.why.contains("who empty"), "{}", status.why);

        let mut no_bin = snap.clone();
        no_bin.bins.retain(|b| b != "pacman");
        no_bin.bins.push("apt".into());
        assert_eq!(upgrade_cmd(&no_bin), Some("apt full-upgrade"));
        assert!(!upgrade_cmd(&no_bin).unwrap().contains("install"));
        no_bin.bins.retain(|b| b != "apt");
        no_bin.bins.push("dnf".into());
        assert_eq!(upgrade_cmd(&no_bin), Some("dnf upgrade"));
        no_bin.bins.clear();
        let (ok, why) = is_live(page, &no_bin, &Default::default());
        assert!(!ok, "{why}");
        assert!(why.contains("reserved"), "{why}");
        assert_eq!(
            fill_walk(page, &Default::default(), &no_bin).command,
            "reserved"
        );
    }
}
