use std::collections::BTreeMap;

use crate::slots;
use crate::types::{DeadVikett, LiveVikett, Page, Policy, Snap};

#[derive(Clone, Debug)]
pub struct Liveness {
    pub ok: bool,
    pub why: String,
}

pub fn client_match<'a>(snap: &'a Snap, target: Option<&str>) -> Option<&'a crate::types::Client> {
    match target {
        None | Some("active") => snap
            .clients
            .iter()
            .find(|c| c.focused)
            .or_else(|| snap.clients.first()),
        Some(t) => {
            let t = t.to_lowercase();
            snap.clients.iter().find(|c| {
                c.class.to_lowercase().contains(&t) || c.title.to_lowercase().contains(&t)
            })
        }
    }
}

/// Guest flag, blank `who`, or `unknown` (any case). Not the owner.
pub fn as_guest(snap: &Snap) -> bool {
    if snap.guest {
        return true;
    }
    let who = snap.who.trim();
    who.is_empty() || who.eq_ignore_ascii_case("unknown")
}

fn as_guest_why(snap: &Snap) -> &'static str {
    if snap.guest {
        "guest flag"
    } else if snap.who.trim().is_empty() {
        "who empty"
    } else {
        "who unknown"
    }
}

pub fn is_live(page: &Page, snap: &Snap, slots: &BTreeMap<String, String>) -> Liveness {
    if (page.policy == Policy::Private || page.module == "mail") && as_guest(snap) {
        return Liveness {
            ok: false,
            why: format!("private pages hidden — {}", as_guest_why(snap)),
        };
    }
    if page.policy == Policy::Owner && as_guest(snap) {
        return Liveness {
            ok: false,
            why: format!("owner-only page — {}", as_guest_why(snap)),
        };
    }
    if page.policy == Policy::Owner && snap.who != snap.owner_id() {
        return Liveness {
            ok: false,
            why: "owner-only page".into(),
        };
    }

    if crate::drivers::is_family(&page.module) {
        let (ok, why) = crate::drivers::family_live(page, snap, slots);
        return Liveness { ok, why };
    }

    match page.id.as_str() {
        "wm.focus" => {
            let c = client_match(snap, slots.get("target").map(String::as_str));
            match c {
                Some(c) => Liveness {
                    ok: true,
                    why: format!("client {}", c.class),
                },
                None => Liveness {
                    ok: false,
                    why: "no matching client".into(),
                },
            }
        }
        "wm.fullscreen" => {
            let c = client_match(
                snap,
                slots.get("target").map(String::as_str).or(Some("active")),
            );
            match c {
                None => Liveness {
                    ok: false,
                    why: "no client".into(),
                },
                Some(c) if c.fullscreen => Liveness {
                    ok: false,
                    why: "already fullscreen".into(),
                },
                Some(c) => Liveness {
                    ok: true,
                    why: format!("can fullscreen {}", c.class),
                },
            }
        }
        "wm.move_ws" | "wm.float" | "wm.close" | "wm.pin" | "wm.center" => {
            if snap.clients.iter().any(|c| c.focused) {
                Liveness {
                    ok: true,
                    why: "focused client".into(),
                }
            } else {
                Liveness {
                    ok: false,
                    why: "nothing focused".into(),
                }
            }
        }
        "wm.workspace" | "wm.cycle_next" => Liveness {
            ok: true,
            why: "workspace/cycle is always legal".into(),
        },
        "wm.movefocus" => Liveness {
            ok: snap.clients.len() >= 2,
            why: if snap.clients.len() >= 2 {
                "multiple clients".into()
            } else {
                "nothing to move focus toward".into()
            },
        },
        "wm.monitor" | "wm.swap_monitor" => Liveness {
            ok: snap.monitor_count() > 1,
            why: if snap.monitor_count() > 1 {
                format!("{} monitors", snap.monitor_count())
            } else {
                "single monitor".into()
            },
        },
        "wm.split_beside" => Liveness {
            ok: true,
            why: "allowlisted split".into(),
        },
        "wm.split_ratio" | "wm.group_next" | "wm.layout" => Liveness {
            ok: true,
            why: "compositor layout door".into(),
        },
        "audio.bump" => {
            if slots.get("direction").map(String::as_str) == Some("up") && snap.volume >= 0.99 {
                Liveness {
                    ok: false,
                    why: "already max".into(),
                }
            } else {
                Liveness {
                    ok: true,
                    why: format!("sink {} @ {}", snap.default_sink, snap.volume),
                }
            }
        }
        "audio.mute" => {
            if snap.muted {
                Liveness {
                    ok: false,
                    why: "already muted".into(),
                }
            } else {
                Liveness {
                    ok: true,
                    why: "can mute".into(),
                }
            }
        }
        "audio.unmute" => {
            if snap.muted {
                Liveness {
                    ok: true,
                    why: "muted".into(),
                }
            } else {
                Liveness {
                    ok: false,
                    why: "not muted".into(),
                }
            }
        }
        "audio.headphones" | "audio.speakers" | "audio.tv" => Liveness {
            ok: true,
            why: "sink switch offered".into(),
        },
        "audio.mic_mute" => Liveness {
            ok: !snap.mic_muted,
            why: if snap.mic_muted {
                "mic already muted".into()
            } else {
                "can mute source".into()
            },
        },
        "audio.mic_unmute" => Liveness {
            ok: snap.mic_muted,
            why: if snap.mic_muted {
                "mic muted".into()
            } else {
                "mic not muted".into()
            },
        },
        "launch.app" => {
            let Some(app) = slots.get("app") else {
                return Liveness {
                    ok: false,
                    why: "no app slot".into(),
                };
            };
            if !snap.allowlist.iter().any(|a| a == app) {
                Liveness {
                    ok: false,
                    why: "not allowlisted".into(),
                }
            } else if snap.running.iter().any(|a| a == app) {
                Liveness {
                    ok: true,
                    why: "already running → focus".into(),
                }
            } else {
                Liveness {
                    ok: true,
                    why: "exec".into(),
                }
            }
        }
        "mail.from" | "mail.unread" | "mail.flag" | "mail.open_last" | "mail.next_unread"
        | "mail.archive" | "mail.mark_read" => {
            if !snap.mail_online {
                Liveness {
                    ok: false,
                    why: "mail offline".into(),
                }
            } else {
                Liveness {
                    ok: true,
                    why: "account bound".into(),
                }
            }
        }
        "media.play_pause" | "media.next" | "media.prev" | "media.ask_now" => {
            if snap.now_playing.is_some() || snap.media_playing {
                Liveness {
                    ok: true,
                    why: snap.now_playing.clone().unwrap_or_else(|| "player".into()),
                }
            } else {
                Liveness {
                    ok: false,
                    why: "no player".into(),
                }
            }
        }
        "timer.add" | "timer.ask" | "timer.cancel" => match snap.timer_sec {
            Some(sec) => Liveness {
                ok: true,
                why: format!("{sec}s left"),
            },
            None => Liveness {
                ok: false,
                why: "no timer".into(),
            },
        },
        "timer.start" => Liveness {
            ok: true,
            why: "timer daemon".into(),
        },
        "bucky.hide" => Liveness {
            ok: true,
            why: "overlay".into(),
        },
        "scene.handoff" => Liveness {
            ok: false,
            why: "handoff driver not wired (page reserved)".into(),
        },
        "calendar.ask_next" | "calendar.ask_today" => Liveness {
            ok: true,
            why: snap
                .next_event
                .as_ref()
                .map(|e| e.title.clone())
                .unwrap_or_else(|| "calendar empty".into()),
        },

        "climate.bump" | "climate.off" => match snap.climate {
            Some(c) => Liveness {
                ok: true,
                why: format!("{c}C"),
            },
            None => Liveness {
                ok: false,
                why: "no climate in this room".into(),
            },
        },
        "capture.screenshot" | "capture.region" => Liveness {
            ok: true,
            why: "focused output".into(),
        },
        "capture.record_start" | "capture.record_stop" => {
            let (ok, why) = crate::drivers::capture::is_live(snap);
            Liveness { ok, why }
        }
        "weather.ask" => match &snap.weather {
            Some(w) => Liveness {
                ok: true,
                why: w.clone(),
            },
            None => Liveness {
                ok: false,
                why: "weather offline".into(),
            },
        },
        "notify.read_last" | "notify.dismiss" | "notify.dismiss_all" => match &snap.notification {
            Some(n) => Liveness {
                ok: true,
                why: n.clone(),
            },
            None => Liveness {
                ok: false,
                why: "no notification history".into(),
            },
        },
        _ => Liveness {
            ok: true,
            why: "default live".into(),
        },
    }
}

pub fn live_and_dead(
    pages: &[Page],
    snap: &Snap,
    utterance: Option<&str>,
) -> (Vec<LiveVikett>, Vec<DeadVikett>) {
    let mut live = Vec::new();
    let mut dead = Vec::new();
    for page in pages {
        let filled = utterance
            .map(|u| slots::fill(page, u, snap))
            .unwrap_or_default();
        let status = is_live(page, snap, &filled.slots);
        if status.ok {
            live.push(LiveVikett {
                page_id: page.id.clone(),
                label: page.title.clone(),
                slots: filled.slots,
                why: status.why,
            });
        } else {
            dead.push(DeadVikett {
                page_id: page.id.clone(),
                why: status.why,
            });
        }
    }
    (live, dead)
}
