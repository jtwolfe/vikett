//! Adapter ask stays `{ powered }`. Connect and disconnect take one allowlisted
//! device id. The walk is a fixed `bluetoothctl` verb for that id. Pair is not a walk.

use std::collections::BTreeMap;

use serde_json::{json, Value};

use crate::types::{Page, Snap, WalkPlan};

pub fn is_live(page: &Page, snap: &Snap, slots: &BTreeMap<String, String>) -> (bool, String) {
    match page.id.as_str() {
        "bluetooth.ask" => (
            snap.bluetooth_on,
            if snap.bluetooth_on {
                "adapter on".into()
            } else {
                "bluetooth off or missing".into()
            },
        ),
        "bluetooth.connect" => device_live(snap, slots, "connect"),
        "bluetooth.disconnect" => device_live(snap, slots, "disconnect"),
        _ => (false, "unknown bluetooth page".into()),
    }
}

pub fn fill_walk(page: &Page, slots: &BTreeMap<String, String>, snap: &Snap) -> WalkPlan {
    let command = match page.id.as_str() {
        "bluetooth.connect" => device_cmd(snap, slots, "connect").unwrap_or("reserved"),
        "bluetooth.disconnect" => device_cmd(snap, slots, "disconnect").unwrap_or("reserved"),
        _ => "reserved",
    };
    WalkPlan {
        command: command.into(),
        driver: "bluetooth".into(),
    }
}

pub fn fill_ask(page: &Page, _slots: &BTreeMap<String, String>, snap: &Snap) -> Value {
    match page.id.as_str() {
        "bluetooth.ask" => json!({ "powered": snap.bluetooth_on }),
        _ => json!({ "unarmed": page.id }),
    }
}

fn device_live(snap: &Snap, slots: &BTreeMap<String, String>, verb: &str) -> (bool, String) {
    if !snap.bluetooth_on {
        return (false, "bluetooth off or missing".into());
    }
    let Some(id) = slots.get("device").map(String::as_str) else {
        return (false, "device missing".into());
    };
    if device_cmd(snap, slots, verb).is_none() {
        if !matches!(id, "headphones") {
            return (false, "device not authored".into());
        }
        if !listed(snap, id) {
            return (false, "device not in list".into());
        }
        return (false, "reserved — bluetoothctl absent".into());
    }
    (true, format!("{verb} {id}"))
}

/// Fixed argv. `headphones` is the only allowlisted id. No MAC from the utterance and no pair.
fn device_cmd(snap: &Snap, slots: &BTreeMap<String, String>, verb: &str) -> Option<&'static str> {
    if !snap.bluetooth_on || !snap.bins.iter().any(|b| b == "bluetoothctl") {
        return None;
    }
    let id = slots.get("device").map(String::as_str)?;
    if !listed(snap, id) {
        return None;
    }
    match (verb, id) {
        ("connect", "headphones") => Some("bluetoothctl connect headphones"),
        ("disconnect", "headphones") => Some("bluetoothctl disconnect headphones"),
        _ => None,
    }
}

fn listed(snap: &Snap, id: &str) -> bool {
    snap.lists
        .get("bt_device")
        .is_some_and(|names| names.iter().any(|n| n == id))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::catalog::Catalog;
    use crate::decide::decide;
    use crate::types::RefereeKind;

    #[test]
    fn connect_and_disconnect_are_the_headphones_enum() {
        let cat = Catalog::load();
        let snap = cat.snap("desk-rig").unwrap();
        let on = decide(&cat, "connect the headphones", snap, RefereeKind::Lexical);
        assert_eq!(on.take.page_id.as_deref(), Some("bluetooth.connect"));
        assert_eq!(
            on.take.slots.get("device").map(String::as_str),
            Some("headphones")
        );
        assert!(!on.confirm);
        let cmd = on.walk.unwrap().command;
        assert_eq!(cmd, "bluetoothctl connect headphones");
        assert!(!cmd.contains("pair"));
        assert!(!cmd.contains("trust"));
        assert!(!cmd.contains("remove"));
        assert!(!cmd.contains("nmcli"));

        let off = decide(
            &cat,
            "disconnect the headphones",
            snap,
            RefereeKind::Lexical,
        );
        assert_eq!(off.take.page_id.as_deref(), Some("bluetooth.disconnect"));
        assert_eq!(
            off.walk.unwrap().command,
            "bluetoothctl disconnect headphones"
        );

        let speaker = decide(&cat, "connect the speaker", snap, RefereeKind::Lexical);
        assert!(speaker.take.page_id.is_none(), "{:?}", speaker.take);
        assert!(speaker.walk.is_none());

        let mut unlisted = snap.clone();
        unlisted.lists.insert("bt_device".into(), Vec::new());
        let dead = decide(
            &cat,
            "connect the headphones",
            &unlisted,
            RefereeKind::Lexical,
        );
        assert!(dead.take.page_id.is_none(), "{:?}", dead.take);
        assert!(dead.walk.is_none());

        let mut no_bin = snap.clone();
        no_bin.bins.retain(|b| b != "bluetoothctl");
        let page = cat.page("bluetooth.connect").unwrap();
        let (ok, why) = is_live(page, &no_bin, &on.take.slots);
        assert!(!ok, "{why}");
        assert!(why.contains("bluetoothctl"), "{why}");
        assert_eq!(fill_walk(page, &on.take.slots, &no_bin).command, "reserved");

        let mut powered = snap.clone();
        powered.bluetooth_on = false;
        let (ok, why) = is_live(page, &powered, &on.take.slots);
        assert!(!ok, "{why}");
        assert!(why.contains("off"), "{why}");

        let desk = cat.snap("desk").unwrap();
        let ask = decide(&cat, "bluetooth status", desk, RefereeKind::Lexical);
        assert_eq!(ask.take.page_id.as_deref(), Some("bluetooth.ask"));
        assert!(ask.walk.is_none());
        assert_eq!(ask.answer.unwrap(), json!({ "powered": true }));
        let headphones = decide(&cat, "headphones", snap, RefereeKind::Lexical);
        assert_eq!(headphones.take.page_id.as_deref(), Some("audio.headphones"));
    }
}
