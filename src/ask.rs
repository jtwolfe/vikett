use std::collections::BTreeMap;

use serde_json::{json, Value};

use crate::types::{Page, Snap};

pub fn fill_ask(page: &Page, slots: &BTreeMap<String, String>, snap: &Snap) -> Value {
    if crate::drivers::is_family(&page.module) {
        return crate::drivers::family_ask(page, slots, snap);
    }
    match page.id.as_str() {
        "mail.from" => {
            let who = slots.get("who").map(String::as_str).unwrap_or("anyone");
            let row = snap.unread_from.get(who);
            json!({
                "count": row.map(|r| r.count).unwrap_or(0),
                "subjects": row.map(|r| r.subjects.clone()).unwrap_or_default(),
                "who": who,
            })
        }
        "mail.unread" => {
            let count: u32 = snap.unread_from.values().map(|r| r.count).sum();
            json!({ "unread": count })
        }
        "mail.next_unread" => match snap.unread_from.iter().find(|(_, row)| row.count > 0) {
            Some((who, row)) => json!({ "who": who, "count": row.count }),
            None => json!({ "who": Value::Null, "count": 0 }),
        },
        "audio.ask_level" => json!({
            "volume": snap.volume,
            "muted": snap.muted,
            "sink": snap.default_sink,
        }),
        "wm.ask_focused" => snap
            .clients
            .iter()
            .find(|c| c.focused)
            .map(|c| {
                json!({
                    "class": c.class,
                    "title": c.title,
                    "workspace": c.workspace,
                    "fullscreen": c.fullscreen,
                    "address": c.address,
                })
            })
            .unwrap_or_else(|| json!({ "none": true })),
        "launch.ask_running" => {
            let app = slots.get("app").cloned();
            json!({
                "running": app.as_ref().map(|a| snap.running.iter().any(|r| r == a)).unwrap_or(false),
                "app": app,
            })
        }
        "timer.ask" => json!({ "remaining_sec": snap.timer_sec }),
        "media.ask_now" => json!({
            "title": snap.now_playing,
            "playing": snap.media_playing,
        }),
        "scene.ask_active" => json!({
            "scene": snap.scene,
            "where": snap.place,
            "occupants": snap.occupants,
        }),
        "bucky.ask_who" => json!({
            "who": snap.who,
            "occupants": snap.occupants,
            "guest": snap.guest,
        }),
        "calendar.ask_next" => snap
            .next_event
            .as_ref()
            .map(|e| json!({ "title": e.title, "in_min": e.in_min }))
            .unwrap_or_else(|| json!({ "none": true })),
        "calendar.ask_today" => match &snap.next_event {
            None => json!({ "remaining": 0, "bucket": "free" }),
            Some(e) => json!({
                "remaining": 1,
                "bucket": today_bucket(e.in_min),
            }),
        },
        "weather.ask" => json!({ "condition": snap.weather }),
        "network.ask" => json!({ "connection": snap.network }),
        "bluetooth.ask" => json!({ "powered": snap.bluetooth_on }),
        "notify.read_last" => json!({ "summary": snap.notification }),
        "wm.ask_where" => {
            let target = slots.get("target").map(String::as_str).unwrap_or("");
            let c = snap.clients.iter().find(|c| {
                c.class.eq_ignore_ascii_case(target) || c.title.to_lowercase().contains(target)
            });
            match c {
                Some(c) => json!({
                    "workspace": c.workspace,
                    "address": c.address,
                    "focused": c.focused,
                }),
                None => json!({ "none": true }),
            }
        }
        _ => json!({ "ok": true }),
    }
}

/// `none` is no event. Otherwise the next event's `in_min`: under 30 busy,
/// under 120 light, else free. `remaining` stays a count, not the minutes.
fn today_bucket(in_min: i32) -> &'static str {
    if in_min < 30 {
        "busy"
    } else if in_min < 120 {
        "light"
    } else {
        "free"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::catalog::Catalog;
    use crate::types::PageKind;

    #[test]
    fn every_ask_has_an_ask_arm() {
        let cat = Catalog::load();
        let snap = cat.snap("desk").unwrap();
        for page in cat.pages.iter().filter(|p| p.kind == PageKind::Ask) {
            let body = fill_ask(page, &Default::default(), snap);
            assert_ne!(body, json!({ "ok": true }), "{}", page.id);
        }
    }

    #[test]
    fn today_bucket_from_in_min_and_mail_next_is_not_a_body() {
        let cat = Catalog::load();
        let page = cat.page("calendar.ask_today").unwrap();
        assert!(cat.page("calendar.ask_busy").is_none());
        let desk = cat.snap("desk").unwrap();
        let body = fill_ask(page, &Default::default(), desk);
        assert_eq!(body, json!({ "remaining": 1, "bucket": "light" }));
        assert!(body.get("title").is_none(), "{body}");

        let mut snap = desk.clone();
        snap.next_event = None;
        assert_eq!(
            fill_ask(page, &Default::default(), &snap),
            json!({ "remaining": 0, "bucket": "free" })
        );
        for (mins, bucket) in [
            (0, "busy"),
            (29, "busy"),
            (30, "light"),
            (119, "light"),
            (120, "free"),
        ] {
            snap.next_event = Some(crate::types::NextEvent {
                title: "standup".into(),
                in_min: mins,
            });
            let body = fill_ask(page, &Default::default(), &snap);
            assert_eq!(body["bucket"], json!(bucket), "{mins}");
            assert_eq!(body["remaining"], json!(1), "{mins}");
        }

        let next = cat.page("mail.next_unread").unwrap();
        let body = fill_ask(next, &Default::default(), desk);
        assert_eq!(body, json!({ "who": "dave", "count": 2 }));
        assert!(body.get("subjects").is_none(), "{body}");
        let mut empty = desk.clone();
        empty.unread_from.clear();
        assert_eq!(
            fill_ask(next, &Default::default(), &empty),
            json!({ "who": null, "count": 0 })
        );
        empty.mail_online = false;
        let status = crate::prune::is_live(next, &empty, &Default::default());
        assert!(!status.ok);
        assert!(status.why.contains("offline"), "{}", status.why);
        let archive = cat.page("mail.archive").unwrap();
        assert!(archive.confirm);
        assert_eq!(archive.policy, crate::types::Policy::Private);
    }
}
