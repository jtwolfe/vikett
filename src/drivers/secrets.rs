//! Vault ask is a bool. `None` is dead. Lock and autotype stay reserved.
//! Autotype would send secret bytes, so it is never a chord. The ask JSON
//! has `unlocked` only.

use std::collections::BTreeMap;

use serde_json::{json, Value};

use crate::types::{Page, Snap, WalkPlan};

pub fn is_live(page: &Page, snap: &Snap, _slots: &BTreeMap<String, String>) -> (bool, String) {
    match page.id.as_str() {
        "secrets.ask_unlocked" => match snap.secrets_unlocked {
            Some(unlocked) => (true, format!("unlocked {unlocked}")),
            None => (false, "secrets unlocked missing".into()),
        },
        "secrets.lock" => (false, "reserved — no vault lock chord".into()),
        "secrets.autotype" => (false, "reserved — would send secret bytes".into()),
        _ => (false, "unknown secrets page".into()),
    }
}

pub fn fill_walk(_page: &Page, _slots: &BTreeMap<String, String>, _snap: &Snap) -> WalkPlan {
    WalkPlan {
        command: "reserved".into(),
        driver: "secrets".into(),
    }
}

pub fn fill_ask(page: &Page, _slots: &BTreeMap<String, String>, snap: &Snap) -> Value {
    match page.id.as_str() {
        "secrets.ask_unlocked" => match snap.secrets_unlocked {
            Some(unlocked) => json!({ "unlocked": unlocked }),
            None => json!({ "unarmed": "secrets.ask_unlocked" }),
        },
        _ => json!({ "unarmed": page.id }),
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
    fn unlocked_is_a_bool_and_autotype_stays_reserved() {
        let cat = Catalog::load();
        let snap = cat.snap("desk-rig").unwrap();
        let ask_page = cat.page("secrets.ask_unlocked").unwrap();
        assert_eq!(ask_page.policy, Policy::Private);
        assert!(!ask_page.confirm);

        let ask = decide(&cat, "is the vault unlocked", snap, RefereeKind::Lexical);
        assert_eq!(ask.take.page_id.as_deref(), Some("secrets.ask_unlocked"));
        assert!(ask.walk.is_none());
        assert!(!ask.confirm);
        let body = ask.answer.unwrap();
        assert_eq!(body, json!({ "unlocked": true }));
        let obj = body.as_object().unwrap();
        assert_eq!(obj.len(), 1);
        assert!(obj.get("password").is_none(), "{body}");
        assert!(obj.get("secret").is_none(), "{body}");
        assert!(obj.get("ok").is_none(), "{body}");

        let desk = cat.snap("desk").unwrap();
        assert!(desk.secrets_unlocked.is_none());
        let (ok, why) = is_live(ask_page, desk, &Default::default());
        assert!(!ok, "{why}");
        assert!(why.contains("missing"), "{why}");
        let dead = decide(&cat, "is the vault unlocked", desk, RefereeKind::Lexical);
        assert!(dead.take.page_id.is_none(), "{:?}", dead.take);
        assert!(dead.answer.is_none());
        assert_ne!(
            fill_ask(ask_page, &Default::default(), desk),
            json!({ "ok": true })
        );

        let lock = cat.page("secrets.lock").unwrap();
        assert_eq!(lock.policy, Policy::Private);
        let (ok, why) = is_live(lock, snap, &Default::default());
        assert!(!ok, "{why}");
        assert!(why.contains("reserved"), "{why}");
        assert_eq!(
            fill_walk(lock, &Default::default(), snap).command,
            "reserved"
        );
        let locked = decide(&cat, "lock the vault", snap, RefereeKind::Lexical);
        assert!(locked.take.page_id.is_none(), "{:?}", locked.take);
        assert!(locked.walk.is_none());
        let screen = decide(&cat, "lock the screen", snap, RefereeKind::Lexical);
        assert_eq!(screen.take.page_id.as_deref(), Some("session.lock"));

        let auto = cat.page("secrets.autotype").unwrap();
        assert_eq!(auto.policy, Policy::Private);
        assert!(auto.confirm);
        assert_ne!(auto.policy, Policy::Confirm);
        let (ok, why) = is_live(auto, snap, &Default::default());
        assert!(!ok, "{why}");
        assert!(why.contains("secret"), "{why}");
        let walk = fill_walk(auto, &Default::default(), snap).command;
        assert_eq!(walk, "reserved");
        assert!(!walk.contains("password"));
        assert!(!walk.contains("secret"));
        let typed = decide(&cat, "autotype from the vault", snap, RefereeKind::Lexical);
        assert!(typed.take.page_id.is_none(), "{:?}", typed.take);
        assert!(typed.walk.is_none());
        assert!(typed.answer.is_none());

        let guest = cat.snap("guest-living").unwrap();
        let status = prune::is_live(ask_page, guest, &Default::default());
        assert!(!status.ok, "{}", status.why);
        assert!(status.why.contains("guest flag"), "{}", status.why);
        let hidden = decide(&cat, "is the vault unlocked", guest, RefereeKind::Lexical);
        assert!(hidden.take.page_id.is_none(), "{:?}", hidden.take);
        assert!(hidden.answer.is_none());

        let mut blank = snap.clone();
        blank.guest = false;
        blank.who = String::new();
        let status = prune::is_live(auto, &blank, &Default::default());
        assert!(!status.ok);
        assert!(status.why.contains("who empty"), "{}", status.why);
    }
}
