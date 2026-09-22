//! Video players. Seek is a `playerctl` notch. Chapter and subs are
//! `hl.dsp.send_shortcut` only where a modifier chord is known (VLC Shift+N
//! and Shift+V). mpv and jellyfin chapter keys are unmodified, so those stay
//! reserved. No free title. Focus may exec; nothing else may.

use std::collections::BTreeMap;

use crate::classes::{bin_for_app, client_for_app};
use crate::drivers::hl;
use crate::keymap;
use crate::types::{Page, Snap, WalkPlan};

pub fn is_live(page: &Page, snap: &Snap, slots: &BTreeMap<String, String>) -> (bool, String) {
    let Some(app) = slots.get("app").map(String::as_str) else {
        return (false, "app missing".into());
    };
    if video_player(app).is_none() {
        return (false, "not a video player".into());
    }
    match page.id.as_str() {
        "video.focus" => focus_live(snap, app),
        "video.seek" => mapped(snap, app),
        "video.chapter" | "video.subs" | "video.fullscreen" | "video.pip" => {
            chord_live(page, snap, app, slots)
        }
        _ => (false, "unknown video page".into()),
    }
}

pub fn fill_walk(page: &Page, slots: &BTreeMap<String, String>, snap: &Snap) -> WalkPlan {
    let app = slots.get("app").map(String::as_str).unwrap_or("");
    if page.id == "video.focus" {
        return focus_walk(app, snap);
    }
    let Some(player) = video_player(app) else {
        return reserved();
    };
    let Some(c) = client_for_app(snap, app) else {
        return reserved();
    };
    if page.id == "video.seek" {
        return WalkPlan {
            command: format!("playerctl -p {player} position {}", seek_offset(slots)),
            driver: "video".into(),
        };
    }
    let Some(chord) = keymap::chord_for(&page.id, app, &snap.id, slots) else {
        return reserved();
    };
    WalkPlan {
        command: format!(
            "{} && {}",
            hl::focus_cmd(&c.address),
            hl::shortcut_cmd(&chord, &c.address)
        ),
        driver: "video".into(),
    }
}

fn video_player(app: &str) -> Option<&'static str> {
    Some(match app {
        "mpv" => "mpv",
        "vlc" => "vlc",
        "jellyfin" => "jellyfin",
        _ => return None,
    })
}

fn seek_offset(slots: &BTreeMap<String, String>) -> &'static str {
    let back = slots.get("direction").map(String::as_str) == Some("down");
    let lot = slots.get("amount").map(String::as_str) == Some("lot");
    match (back, lot) {
        (false, false) => "10+",
        (false, true) => "30+",
        (true, false) => "10-",
        (true, true) => "30-",
    }
}

fn mapped(snap: &Snap, app: &str) -> (bool, String) {
    match client_for_app(snap, app) {
        Some(c) => (true, format!("client {}", c.class)),
        None => (false, "no matching client".into()),
    }
}

fn focus_live(snap: &Snap, app: &str) -> (bool, String) {
    if client_for_app(snap, app).is_some() {
        return (true, format!("client {app}"));
    }
    let Some(bin) = bin_for_app(app) else {
        return (false, "video player not mapped".into());
    };
    if snap.allowlist.iter().any(|a| a == app) && snap.bins.iter().any(|b| b == bin) {
        (true, format!("exec {bin}"))
    } else {
        (false, "video player not mapped".into())
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

fn focus_walk(app: &str, snap: &Snap) -> WalkPlan {
    if let Some(c) = client_for_app(snap, app) {
        return WalkPlan {
            command: hl::focus_cmd(&c.address),
            driver: "video".into(),
        };
    }
    let command = bin_for_app(app)
        .filter(|bin| snap.allowlist.iter().any(|a| a == app) && snap.bins.iter().any(|b| b == bin))
        .map(hl::exec_cmd)
        .unwrap_or_else(|| "reserved".into());
    WalkPlan {
        command,
        driver: "video".into(),
    }
}

fn reserved() -> WalkPlan {
    WalkPlan {
        command: "reserved".into(),
        driver: "video".into(),
    }
}

#[cfg(test)]
mod tests {
    use crate::catalog::Catalog;
    use crate::decide::decide;
    use crate::types::RefereeKind;

    #[test]
    fn chapter_guest_stays_silent_and_seek_is_a_notch() {
        let cat = Catalog::load();
        let guest = cat.snap("guest-living").unwrap();
        let chapter = decide(&cat, "next chapter", guest, RefereeKind::Lexical);
        assert!(chapter.take.page_id.is_none(), "{:?}", chapter.take);
        assert!(chapter.walk.is_none());
        let next = decide(&cat, "next", guest, RefereeKind::Lexical);
        assert_eq!(next.take.page_id.as_deref(), Some("media.next"));

        let seek = decide(&cat, "seek the video forward", guest, RefereeKind::Lexical);
        assert_eq!(seek.take.page_id.as_deref(), Some("video.seek"));
        let walk = seek.walk.unwrap().command;
        assert!(
            walk.contains("playerctl -p jellyfin position 10+"),
            "{walk}"
        );
        assert!(!walk.contains("dispatch exec"), "{walk}");
        assert!(decide(&cat, "play the movie", guest, RefereeKind::Lexical)
            .take
            .page_id
            .is_none());
    }
}
