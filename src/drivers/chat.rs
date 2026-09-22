//! Signal, Element, and Vesktop. Class `discord` is Vesktop.
//! Focus is `hl.dsp.focus` only when that class is mapped. It does not exec.
//! Mute and mark-read have no modifier chord: Discord Ctrl+Shift+M is the
//! microphone, not the app stream, and the others are unset or unmodified.
//! Those doors stay reserved. Unread is a private bucket from `chat_unread`.
//! A missing key is dead, not zero. There is no compose page.

use std::collections::BTreeMap;

use serde_json::{json, Value};

use crate::classes::client_for_app;
use crate::drivers::files::{chord_walk, reserved};
use crate::keymap;
use crate::types::{Page, Snap, WalkPlan};

pub fn is_live(page: &Page, snap: &Snap, slots: &BTreeMap<String, String>) -> (bool, String) {
    let Some(app) = slots.get("app") else {
        return (false, "app missing".into());
    };
    if !is_chat_app(app) {
        return (false, "not a chat app".into());
    }
    match page.id.as_str() {
        "chat.focus" => match client_for_app(snap, app) {
            Some(c) => (true, format!("client {}", c.class)),
            None => (false, "chat app not mapped".into()),
        },
        "chat.mute_app" | "chat.mark_read" => chord_live(page, snap, app, slots),
        "chat.ask_unread" => match snap.chat_unread.get(app) {
            Some(n) => (true, format!("{n} unread")),
            None => (false, "unread unknown".into()),
        },
        _ => (false, "unknown chat page".into()),
    }
}

pub fn fill_walk(page: &Page, slots: &BTreeMap<String, String>, snap: &Snap) -> WalkPlan {
    let app = slots.get("app").map(String::as_str).unwrap_or("");
    if page.id == "chat.focus" {
        return match client_for_app(snap, app) {
            Some(c) => WalkPlan {
                command: crate::drivers::hl::focus_cmd(&c.address),
                driver: "chat".into(),
            },
            None => reserved("chat"),
        };
    }
    chord_walk(&page.id, app, slots, snap, "chat")
}

pub fn fill_ask(page: &Page, slots: &BTreeMap<String, String>, snap: &Snap) -> Value {
    let app = slots.get("app").map(String::as_str).unwrap_or("");
    match page.id.as_str() {
        "chat.ask_unread" => {
            let Some(n) = snap.chat_unread.get(app) else {
                return json!({ "unarmed": "chat.ask_unread" });
            };
            json!({ "app": app, "bucket": bucket(*n) })
        }
        _ => json!({ "unarmed": page.id }),
    }
}

fn is_chat_app(app: &str) -> bool {
    matches!(app, "signal" | "element" | "vesktop")
}

fn bucket(n: u32) -> &'static str {
    if n == 0 {
        "none"
    } else if n <= 3 {
        "few"
    } else {
        "many"
    }
}

fn chord_live(
    page: &Page,
    snap: &Snap,
    app: &str,
    slots: &BTreeMap<String, String>,
) -> (bool, String) {
    let Some(c) = client_for_app(snap, app) else {
        return (false, "no matching client".into());
    };
    if keymap::chord_for(&page.id, app, &snap.id, slots).is_none() {
        return (false, "reserved — no chord".into());
    }
    (true, format!("client {}", c.class))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::catalog::Catalog;
    use crate::prune;

    #[test]
    fn focus_is_mapped_address_and_mute_stays_reserved() {
        let cat = Catalog::load();
        let snap = cat.snap("desk-chat").unwrap();
        let focus = cat.page("chat.focus").unwrap();
        let mut slots = BTreeMap::new();
        slots.insert("app".into(), "signal".into());
        let plan = fill_walk(focus, &slots, snap);
        assert!(plan.command.contains("hl.dsp.focus"), "{}", plan.command);
        assert!(plan.command.contains("address:0xsig"), "{}", plan.command);
        assert!(!plan.command.contains("exec_cmd"), "{}", plan.command);
        assert!(!plan.command.contains("dispatch exec"), "{}", plan.command);
        assert!(!plan.command.contains("focuswindow"), "{}", plan.command);

        slots.insert("app".into(), "vesktop".into());
        let plan = fill_walk(focus, &slots, snap);
        assert!(plan.command.contains("address:0xdisc"), "{}", plan.command);

        slots.insert("app".into(), "element".into());
        let (ok, why) = is_live(focus, snap, &slots);
        assert!(!ok, "{why}");
        assert!(why.contains("not mapped"), "{why}");

        let mute = cat.page("chat.mute_app").unwrap();
        slots.insert("app".into(), "signal".into());
        let (ok, why) = is_live(mute, snap, &slots);
        assert!(!ok, "{why}");
        assert!(why.contains("chord"), "{why}");
        assert_eq!(fill_walk(mute, &slots, snap).command, "reserved");
        assert_eq!(
            fill_walk(cat.page("chat.mark_read").unwrap(), &slots, snap).command,
            "reserved"
        );
    }

    #[test]
    fn unread_bucket_or_dead_never_ok() {
        let cat = Catalog::load();
        let snap = cat.snap("desk-chat").unwrap();
        let page = cat.page("chat.ask_unread").unwrap();
        assert_eq!(page.policy, crate::types::Policy::Private);
        let mut slots = BTreeMap::new();
        slots.insert("app".into(), "signal".into());
        assert_eq!(
            fill_ask(page, &slots, snap),
            json!({ "app": "signal", "bucket": "few" })
        );
        slots.insert("app".into(), "vesktop".into());
        assert_eq!(
            fill_ask(page, &slots, snap),
            json!({ "app": "vesktop", "bucket": "none" })
        );
        slots.insert("app".into(), "element".into());
        let (ok, why) = is_live(page, snap, &slots);
        assert!(!ok, "{why}");
        assert!(why.contains("unknown"), "{why}");
        let body = fill_ask(page, &slots, snap);
        assert_ne!(body, json!({ "ok": true }));
        assert!(body.get("bucket").is_none(), "{body}");

        let mut many = snap.clone();
        many.chat_unread.insert("signal".into(), 4);
        slots.insert("app".into(), "signal".into());
        assert_eq!(fill_ask(page, &slots, &many)["bucket"], json!("many"));

        let guest = cat.snap("guest-living").unwrap();
        let status = prune::is_live(page, guest, &slots);
        assert!(!status.ok);
        assert!(status.why.contains("guest"), "{}", status.why);
    }
}
