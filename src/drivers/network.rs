//! Network ask stays the connection field. VPN connect is `nmcli connection up`
//! for a name already in `lists.vpn`. Never down, radio, wifi, or DNS.

use std::collections::BTreeMap;

use serde_json::{json, Value};

use crate::host::vpn_id_ok;
use crate::types::{Page, Snap, WalkPlan};

pub fn is_live(page: &Page, snap: &Snap, slots: &BTreeMap<String, String>) -> (bool, String) {
    match page.id.as_str() {
        "network.ask" => match &snap.network {
            Some(n) => (true, n.clone()),
            None => (false, "network snap empty".into()),
        },
        "network.vpn" => vpn_live(snap, slots),
        _ => (false, "unknown network page".into()),
    }
}

pub fn fill_walk(page: &Page, slots: &BTreeMap<String, String>, snap: &Snap) -> WalkPlan {
    let command = match page.id.as_str() {
        "network.vpn" => vpn_up(snap, slots).unwrap_or_else(|| "reserved".into()),
        _ => "reserved".into(),
    };
    WalkPlan {
        command,
        driver: "network".into(),
    }
}

pub fn fill_ask(page: &Page, _slots: &BTreeMap<String, String>, snap: &Snap) -> Value {
    match page.id.as_str() {
        "network.ask" => json!({ "connection": snap.network }),
        _ => json!({ "unarmed": page.id }),
    }
}

fn vpn_live(snap: &Snap, slots: &BTreeMap<String, String>) -> (bool, String) {
    let Some(id) = slots.get("vpn").map(String::as_str) else {
        return (false, "vpn missing".into());
    };
    if !listed(snap, id) {
        return (false, "vpn not in list".into());
    }
    if vpn_up(snap, slots).is_none() {
        if !snap.bins.iter().any(|b| b == "nmcli") {
            return (false, "reserved — nmcli absent".into());
        }
        return (false, "vpn name not a token".into());
    }
    (true, format!("vpn {id}"))
}

/// `nmcli connection up` only. The id is a list token, never the raw utterance.
fn vpn_up(snap: &Snap, slots: &BTreeMap<String, String>) -> Option<String> {
    if !snap.bins.iter().any(|b| b == "nmcli") {
        return None;
    }
    let id = slots.get("vpn").map(String::as_str)?;
    if !listed(snap, id) || !vpn_id_ok(id) {
        return None;
    }
    if id.contains(' ') {
        Some(format!("nmcli connection up '{id}'"))
    } else {
        Some(format!("nmcli connection up {id}"))
    }
}

fn listed(snap: &Snap, id: &str) -> bool {
    snap.lists
        .get("vpn")
        .is_some_and(|names| names.iter().any(|n| n == id))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::catalog::Catalog;
    use crate::decide::decide;
    use crate::prune;
    use crate::types::{Policy, RefereeKind};

    #[test]
    fn vpn_up_is_confirm_and_never_down() {
        let cat = Catalog::load();
        let snap = cat.snap("desk-rig").unwrap();
        let page = cat.page("network.vpn").unwrap();
        assert!(page.confirm);
        assert_eq!(page.policy, Policy::Household);
        assert_ne!(page.policy, Policy::Confirm);

        let r = decide(&cat, "connect the home vpn", snap, RefereeKind::Lexical);
        assert_eq!(r.take.page_id.as_deref(), Some("network.vpn"));
        assert_eq!(r.take.slots.get("vpn").map(String::as_str), Some("home"));
        assert!(r.confirm);
        assert!(r.answer.is_none());
        let cmd = r.walk.unwrap().command;
        assert_eq!(cmd, "nmcli connection up home");
        assert!(!cmd.contains("connection down"));
        assert!(!cmd.contains("radio"));
        assert!(!cmd.contains("wifi"));
        assert!(!cmd.contains("dns"));
        assert!(!cmd.contains("sudo"));
        assert!(!cmd.contains(';'));

        let mut spaced = snap.clone();
        spaced.lists.insert("vpn".into(), vec!["work vpn".into()]);
        let up = decide(&cat, "connect the work vpn", &spaced, RefereeKind::Lexical);
        assert_eq!(
            up.take.slots.get("vpn").map(String::as_str),
            Some("work vpn")
        );
        assert_eq!(up.walk.unwrap().command, "nmcli connection up 'work vpn'");

        let mut bare = snap.clone();
        bare.bins.retain(|b| b != "nmcli");
        let (ok, why) = is_live(page, &bare, &r.take.slots);
        assert!(!ok, "{why}");
        assert!(why.contains("nmcli"), "{why}");
        assert_eq!(fill_walk(page, &r.take.slots, &bare).command, "reserved");

        let missed = decide(&cat, "connect the office vpn", snap, RefereeKind::Lexical);
        assert!(missed.take.page_id.is_none(), "{:?}", missed.take);
        assert!(missed.walk.is_none());

        let desk = cat.snap("desk").unwrap();
        let dead = decide(&cat, "connect the home vpn", desk, RefereeKind::Lexical);
        assert!(dead.take.page_id.is_none(), "{:?}", dead.take);
        assert!(dead.walk.is_none());

        let ask = decide(&cat, "am i online", desk, RefereeKind::Lexical);
        assert_eq!(ask.take.page_id.as_deref(), Some("network.ask"));
        assert!(ask.walk.is_none());
        assert_eq!(ask.answer.unwrap()["connection"], "ethernet");

        let mut guest = snap.clone();
        guest.guest = true;
        let g = decide(&cat, "connect the home vpn", &guest, RefereeKind::Lexical);
        assert_eq!(g.take.page_id.as_deref(), Some("network.vpn"));
        assert!(g.confirm);

        let status = prune::is_live(page, desk, &r.take.slots);
        assert!(!status.ok, "{}", status.why);
    }
}
