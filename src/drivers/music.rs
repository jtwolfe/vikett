//! Music players. Transport is `playerctl -p <app>`, never a bare `playerctl`
//! and never a per-app volume. Focus may `exec_cmd` only when the app is
//! allowlisted, unmapped, and `bins` contains the binary.

use std::collections::BTreeMap;

use crate::classes::{bin_for_app, client_for_app};
use crate::drivers::hl;
use crate::types::{Page, Snap, WalkPlan};

pub fn is_live(page: &Page, snap: &Snap, slots: &BTreeMap<String, String>) -> (bool, String) {
    let Some(app) = slots.get("app").map(String::as_str) else {
        return (false, "app missing".into());
    };
    if music_player(app).is_none() {
        return (false, "not a music player".into());
    }
    match page.id.as_str() {
        "music.focus" => focus_live(snap, app),
        "music.play_pause" | "music.next" | "music.seek" => mapped(snap, app),
        "music.playlist" => playlist_live(snap, app, slots),
        _ => (false, "unknown music page".into()),
    }
}

pub fn fill_walk(page: &Page, slots: &BTreeMap<String, String>, snap: &Snap) -> WalkPlan {
    let app = slots.get("app").map(String::as_str).unwrap_or("");
    let driver = "music";
    if page.id == "music.focus" {
        return focus_walk(app, snap);
    }
    let Some(player) = music_player(app) else {
        return reserved();
    };
    if client_for_app(snap, app).is_none() {
        return reserved();
    }
    let command = match page.id.as_str() {
        "music.play_pause" => format!("playerctl -p {player} play-pause"),
        "music.next" => format!("playerctl -p {player} next"),
        "music.seek" => format!("playerctl -p {player} position {}", seek_offset(slots)),
        "music.playlist" => {
            let Some(id) = playlist_id(slots.get("playlist").map(String::as_str).unwrap_or(""))
            else {
                return reserved();
            };
            if !listed(snap, id) {
                return reserved();
            }
            format!("playerctl -p {player} open playlist:{id}")
        }
        _ => return reserved(),
    };
    WalkPlan {
        command,
        driver: driver.into(),
    }
}

fn music_player(app: &str) -> Option<&'static str> {
    Some(match app {
        "spotify" => "spotify",
        "ncspot" => "ncspot",
        "strawberry" => "strawberry",
        "amberol" => "amberol",
        "mpd" => "mpd",
        _ => return None,
    })
}

fn playlist_id(id: &str) -> Option<&'static str> {
    Some(match id {
        "focus" => "focus",
        "jazz" => "jazz",
        _ => return None,
    })
}

/// Little is 10s, lot is 30s. Not a percent.
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

fn listed(snap: &Snap, id: &str) -> bool {
    snap.lists
        .get("playlist")
        .is_some_and(|names| names.iter().any(|n| n == id))
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
        return (false, "music player not mapped".into());
    };
    if snap.allowlist.iter().any(|a| a == app) && snap.bins.iter().any(|b| b == bin) {
        (true, format!("exec {bin}"))
    } else {
        (false, "music player not mapped".into())
    }
}

fn playlist_live(snap: &Snap, app: &str, slots: &BTreeMap<String, String>) -> (bool, String) {
    let Some(id) = slots.get("playlist").map(String::as_str) else {
        return (false, "playlist missing".into());
    };
    if playlist_id(id).is_none() {
        return (false, "playlist not authored".into());
    }
    if !listed(snap, id) {
        return (false, "playlist not in list".into());
    }
    mapped(snap, app)
}

fn focus_walk(app: &str, snap: &Snap) -> WalkPlan {
    if let Some(c) = client_for_app(snap, app) {
        return WalkPlan {
            command: hl::focus_cmd(&c.address),
            driver: "music".into(),
        };
    }
    let command = bin_for_app(app)
        .filter(|bin| snap.allowlist.iter().any(|a| a == app) && snap.bins.iter().any(|b| b == bin))
        .map(hl::exec_cmd)
        .unwrap_or_else(|| "reserved".into());
    WalkPlan {
        command,
        driver: "music".into(),
    }
}

fn reserved() -> WalkPlan {
    WalkPlan {
        command: "reserved".into(),
        driver: "music".into(),
    }
}

#[cfg(test)]
mod tests {
    use crate::catalog::Catalog;
    use crate::decide::decide;
    use crate::types::RefereeKind;

    #[test]
    fn pause_spotify_does_not_steal_pause_or_set_volume() {
        let cat = Catalog::load();
        let guest = cat.snap("guest-living").unwrap();
        let paused = decide(&cat, "pause", guest, RefereeKind::Lexical);
        assert_eq!(paused.take.page_id.as_deref(), Some("media.play_pause"));
        let named = decide(&cat, "pause spotify", guest, RefereeKind::Lexical);
        assert!(named.take.page_id.is_none(), "{:?}", named.take);
        assert!(named.walk.is_none());

        let desk = cat.snap("desk-media").unwrap();
        let music = decide(&cat, "pause the music", desk, RefereeKind::Lexical);
        assert_eq!(music.take.page_id.as_deref(), Some("music.play_pause"));
        let walk = music.walk.unwrap().command;
        assert!(walk.contains("playerctl -p spotify play-pause"), "{walk}");
        assert!(!walk.contains("volume"), "{walk}");
        assert!(!walk.contains('%'), "{walk}");
        assert!(!walk.contains("dispatch exec"), "{walk}");

        let louder = decide(&cat, "louder", desk, RefereeKind::Lexical);
        assert_eq!(louder.take.page_id.as_deref(), Some("audio.bump"));
    }
}
