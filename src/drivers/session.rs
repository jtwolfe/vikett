//! Lock and DPMS keep their v0 dry-runs (`hyprlock`, `hyprctl dispatch dpms`).
//! Suspend, reboot, and poweroff are owner doors. `confirm` is a separate
//! bool; prune already treats a guest, an empty `who`, or `unknown` as not
//! the owner. `session.idle` stays reserved: `hl.dsp.force_idle` forces idle
//! and ignores inhibitors. It is not a latch, and this repo does not hold
//! `systemd-inhibit`.

use std::collections::BTreeMap;

use crate::types::{Page, Snap, WalkPlan};

pub fn is_live(page: &Page, snap: &Snap, _slots: &BTreeMap<String, String>) -> (bool, String) {
    match page.id.as_str() {
        "session.lock" => (
            snap.lock_available,
            if snap.lock_available {
                "hyprlock".into()
            } else {
                "lock binary not present".into()
            },
        ),
        "session.dpms" => (true, "compositor dpms".into()),
        "session.suspend" | "session.reboot" | "session.poweroff" => {
            (true, "owner power door".into())
        }
        "session.idle" => (false, "reserved — force_idle is not an inhibitor".into()),
        _ => (false, "unknown session page".into()),
    }
}

pub fn fill_walk(page: &Page, slots: &BTreeMap<String, String>, _snap: &Snap) -> WalkPlan {
    let command = match page.id.as_str() {
        "session.lock" => "hyprlock".into(),
        "session.dpms" => {
            if slots.get("action").map(String::as_str) == Some("on") {
                "hyprctl dispatch dpms on".into()
            } else {
                "hyprctl dispatch dpms off".into()
            }
        }
        "session.suspend" => "systemctl suspend".into(),
        "session.reboot" => "systemctl reboot".into(),
        "session.poweroff" => "systemctl poweroff".into(),
        _ => "reserved".into(),
    };
    WalkPlan {
        command,
        driver: "session".into(),
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
    fn power_is_owner_confirm_and_idle_is_not_force_idle() {
        let cat = Catalog::load();
        let desk = cat.snap("desk").unwrap();
        let guest = cat.snap("guest-living").unwrap();
        assert_eq!(guest.who, "jim");
        assert!(guest.guest);
        for id in ["session.suspend", "session.reboot", "session.poweroff"] {
            let page = cat.page(id).unwrap();
            assert_eq!(page.policy, Policy::Owner, "{id}");
            assert!(page.confirm, "{id}");
            assert_ne!(page.policy, Policy::Confirm, "{id}");
            let status = prune::is_live(page, guest, &Default::default());
            assert!(!status.ok, "{id} {}", status.why);
            assert!(status.why.contains("guest flag"), "{id} {}", status.why);
            let r = decide(&cat, page.aliases[0].as_str(), desk, RefereeKind::Lexical);
            assert_eq!(r.take.page_id.as_deref(), Some(id), "{:?}", r.take);
            assert!(r.confirm, "{id}");
            let cmd = r.walk.unwrap().command;
            assert!(cmd.starts_with("systemctl "), "{cmd}");
            assert!(!cmd.contains("force_idle"), "{cmd}");
            assert!(!cmd.contains("focuswindow"), "{cmd}");
            assert!(!cmd.contains("dispatch exec"), "{cmd}");
        }

        let idle = cat.page("session.idle").unwrap();
        let (ok, why) = is_live(idle, desk, &Default::default());
        assert!(!ok, "{why}");
        assert!(why.contains("force_idle"), "{why}");
        let plan = fill_walk(idle, &Default::default(), desk);
        assert_eq!(plan.command, "reserved");
        assert!(!plan.command.contains("force_idle"));
        let r = decide(&cat, "keep the screen awake", desk, RefereeKind::Lexical);
        assert!(r.take.page_id.is_none(), "{:?}", r.take);
        assert!(r.walk.is_none());

        let suspend = cat.page("session.suspend").unwrap();
        let mut blank = desk.clone();
        blank.guest = false;
        blank.who = String::new();
        let status = prune::is_live(suspend, &blank, &Default::default());
        assert!(!status.ok);
        assert!(status.why.contains("who empty"), "{}", status.why);
        blank.who = "Unknown".into();
        let status = prune::is_live(suspend, &blank, &Default::default());
        assert!(!status.ok);
        assert!(status.why.contains("who unknown"), "{}", status.why);
        blank.who = "dave".into();
        let status = prune::is_live(suspend, &blank, &Default::default());
        assert!(!status.ok);
        assert!(status.why.contains("owner-only"), "{}", status.why);

        let guest_take = decide(&cat, "suspend the computer", guest, RefereeKind::Lexical);
        assert!(guest_take.take.page_id.is_none(), "{:?}", guest_take.take);
        assert!(guest_take.walk.is_none());
        let shut = decide(&cat, "shut down", guest, RefereeKind::Lexical);
        assert!(shut.take.page_id.is_none(), "{:?}", shut.take);
        assert!(shut.walk.is_none());
    }

    #[test]
    fn lock_and_dpms_strings_stay() {
        let cat = Catalog::load();
        let desk = cat.snap("desk").unwrap();
        let lock = cat.page("session.lock").unwrap();
        assert_eq!(
            fill_walk(lock, &Default::default(), desk).command,
            "hyprlock"
        );
        let dpms = cat.page("session.dpms").unwrap();
        let mut slots = BTreeMap::new();
        slots.insert("action".into(), "on".into());
        assert_eq!(
            fill_walk(dpms, &slots, desk).command,
            "hyprctl dispatch dpms on"
        );
        slots.insert("action".into(), "off".into());
        assert_eq!(
            fill_walk(dpms, &slots, desk).command,
            "hyprctl dispatch dpms off"
        );
        let r = decide(&cat, "lock the screen", desk, RefereeKind::Lexical);
        assert_eq!(r.take.page_id.as_deref(), Some("session.lock"));
        let r = decide(&cat, "screens on", desk, RefereeKind::Lexical);
        assert_eq!(r.take.page_id.as_deref(), Some("session.dpms"));
        assert!(r.walk.unwrap().command.contains("dpms on"));
    }
}
