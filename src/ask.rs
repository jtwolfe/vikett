use std::collections::BTreeMap;

use serde_json::{json, Value};

use crate::types::{Page, Snap};

pub fn fill_ask(page: &Page, slots: &BTreeMap<String, String>, snap: &Snap) -> Value {
    if crate::drivers::is_family(&page.module) {
        return crate::drivers::browser::fill_ask(page, slots, snap);
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
        "calendar.ask_today" => json!({
            "remaining": if snap.next_event.is_some() { 1 } else { 0 }
        }),
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
}
