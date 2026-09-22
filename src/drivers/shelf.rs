//! Launcher and clipboard. The launcher walk is `fuzzel` or `walker` with no
//! query: the user types. Clear confirms and never prints clipboard bytes.
//! `cliphist list` and `wl-paste` are not commands here.

use std::collections::BTreeMap;

use serde_json::{json, Value};

use crate::types::{Page, Snap, WalkPlan};

pub fn is_live(page: &Page, snap: &Snap, slots: &BTreeMap<String, String>) -> (bool, String) {
    match page.id.as_str() {
        "shelf.fuzzel" => match launcher(snap, slots) {
            Some(bin) => (true, format!("launcher {bin}")),
            None => (false, "no fuzzel or walker in bins".into()),
        },
        "clip.ask" => match clip_kind(snap) {
            Some(kind) => (true, format!("clipboard {kind}")),
            None => (false, "clipboard kind missing".into()),
        },
        "clip.clear" => match clear_cmd(snap) {
            Some(cmd) => (true, cmd.into()),
            None => (false, "no clipboard clear tool in bins".into()),
        },
        _ => (false, "unknown shelf page".into()),
    }
}

pub fn fill_walk(page: &Page, slots: &BTreeMap<String, String>, snap: &Snap) -> WalkPlan {
    let command = match page.id.as_str() {
        "shelf.fuzzel" => launcher(snap, slots).unwrap_or("reserved").to_string(),
        "clip.clear" => clear_cmd(snap).unwrap_or("reserved").to_string(),
        _ => "reserved".into(),
    };
    WalkPlan {
        command,
        driver: "shelf".into(),
    }
}

pub fn fill_ask(page: &Page, _slots: &BTreeMap<String, String>, snap: &Snap) -> Value {
    match page.id.as_str() {
        "clip.ask" => {
            let Some(kind) = clip_kind(snap) else {
                return json!({ "unarmed": "clip.ask" });
            };
            json!({
                "kind": kind,
                "bucket": bucket(kind, snap.clipboard_count),
            })
        }
        _ => json!({ "unarmed": page.id }),
    }
}

/// Named launcher wins. Otherwise fuzzel, then walker. No arguments.
fn launcher(snap: &Snap, slots: &BTreeMap<String, String>) -> Option<&'static str> {
    if let Some(name) = slots.get("launcher").map(String::as_str) {
        return match name {
            "fuzzel" if listed(snap, "fuzzel") => Some("fuzzel"),
            "walker" if listed(snap, "walker") => Some("walker"),
            _ => None,
        };
    }
    if listed(snap, "fuzzel") {
        Some("fuzzel")
    } else if listed(snap, "walker") {
        Some("walker")
    } else {
        None
    }
}

/// `cliphist wipe` when that binary is listed, else `wl-copy -c`.
/// Neither prints the clipboard. `cliphist list` is not the other probe.
fn clear_cmd(snap: &Snap) -> Option<&'static str> {
    if listed(snap, "cliphist") {
        Some("cliphist wipe")
    } else if listed(snap, "wl-copy") {
        Some("wl-copy -c")
    } else {
        None
    }
}

fn listed(snap: &Snap, bin: &str) -> bool {
    snap.bins.iter().any(|b| b == bin)
}

/// `text`, `image`, or `empty`. Anything else, including a missing field, is dead.
fn clip_kind(snap: &Snap) -> Option<&'static str> {
    match snap.clipboard_kind.as_deref() {
        Some("text") => Some("text"),
        Some("image") => Some("image"),
        Some("empty") => Some("empty"),
        _ => None,
    }
}

fn bucket(kind: &str, count: Option<u32>) -> &'static str {
    if kind == "empty" {
        return "none";
    }
    match count.unwrap_or(0) {
        0 => "none",
        1..=3 => "few",
        _ => "many",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::catalog::Catalog;
    use crate::decide::decide;
    use crate::prune;
    use crate::types::RefereeKind;

    #[test]
    fn launcher_is_the_binary_and_the_user_types() {
        let cat = Catalog::load();
        let snap = cat.snap("desk-shelf").unwrap();
        let named = decide(&cat, "show walker", snap, RefereeKind::Lexical);
        assert_eq!(named.take.page_id.as_deref(), Some("shelf.fuzzel"));
        assert_eq!(
            named.take.slots.get("launcher").map(String::as_str),
            Some("walker")
        );
        let walk = named.walk.unwrap().command;
        assert_eq!(walk, "walker");
        assert!(!walk.contains(' '));
        assert!(!walk.contains("hyprctl"));
        assert!(!walk.contains("dispatch exec"));
        assert!(!walk.contains("exec_cmd"));

        let plain = decide(&cat, "show the launcher", snap, RefereeKind::Lexical);
        assert_eq!(plain.walk.unwrap().command, "fuzzel");

        let mut only_walker = snap.clone();
        only_walker.bins.retain(|b| b != "fuzzel");
        let r = decide(&cat, "show fuzzel", &only_walker, RefereeKind::Lexical);
        assert!(r.take.page_id.is_none(), "{:?}", r.take);
        assert!(r.walk.is_none());

        let desk = cat.snap("desk").unwrap();
        let dead = decide(&cat, "show the launcher", desk, RefereeKind::Lexical);
        assert!(dead.take.page_id.is_none(), "{:?}", dead.take);
        assert!(dead.walk.is_none());
    }

    #[test]
    fn clipboard_kind_bucket_and_clear_confirms() {
        let cat = Catalog::load();
        let snap = cat.snap("desk-shelf").unwrap();
        let ask = cat.page("clip.ask").unwrap();
        assert_eq!(ask.policy, crate::types::Policy::Private);
        let r = decide(&cat, "what's on the clipboard", snap, RefereeKind::Lexical);
        assert_eq!(r.take.page_id.as_deref(), Some("clip.ask"));
        assert!(r.walk.is_none());
        let body = r.answer.unwrap();
        assert_eq!(body, json!({ "kind": "text", "bucket": "few" }));
        assert!(body.get("text").is_none());
        assert!(body.get("bytes").is_none());
        assert!(body.get("password").is_none());

        let clear = decide(&cat, "clear the clipboard", snap, RefereeKind::Lexical);
        assert_eq!(clear.take.page_id.as_deref(), Some("clip.clear"));
        assert!(clear.confirm);
        assert_eq!(clear.walk.unwrap().command, "wl-copy -c");

        let mut both = snap.clone();
        both.bins.insert(0, "cliphist".into());
        let wiped = decide(&cat, "clear the clipboard", &both, RefereeKind::Lexical);
        assert_eq!(wiped.walk.unwrap().command, "cliphist wipe");

        let mut empty = snap.clone();
        empty.clipboard_kind = Some("empty".into());
        empty.clipboard_count = Some(9);
        let body = fill_ask(ask, &Default::default(), &empty);
        assert_eq!(body, json!({ "kind": "empty", "bucket": "none" }));

        let mut image = snap.clone();
        image.clipboard_kind = Some("image".into());
        image.clipboard_count = Some(4);
        let body = fill_ask(ask, &Default::default(), &image);
        assert_eq!(body, json!({ "kind": "image", "bucket": "many" }));

        for kind in [None, Some("other".into()), Some("password".into())] {
            let mut bad = snap.clone();
            bad.clipboard_kind = kind.clone();
            let (ok, why) = is_live(ask, &bad, &Default::default());
            assert!(!ok, "{kind:?} {why}");
            assert!(why.contains("missing"), "{why}");
        }

        let guest = cat.snap("guest-living").unwrap();
        let status = prune::is_live(ask, guest, &Default::default());
        assert!(!status.ok);
        assert!(status.why.contains("guest flag"), "{}", status.why);
        let hidden = decide(&cat, "what's on the clipboard", guest, RefereeKind::Lexical);
        assert!(hidden.take.page_id.is_none(), "{:?}", hidden.take);
        assert!(hidden.answer.is_none());
        assert!(hidden.walk.is_none());

        let refused = decide(&cat, "copy a password", snap, RefereeKind::Lexical);
        assert!(refused.take.page_id.is_none(), "{:?}", refused.take);
        assert!(refused.walk.is_none());
        assert!(refused.answer.is_none());
    }
}
