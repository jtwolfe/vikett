//! Research-backed extensions on top of the v0 JSON catalogue.
//!
//! Hyprland 0.56.2 on this box: `hyprctl dispatch` walks windows/workspaces/dpms.
//! Everyday control surface from Omarchy/Hyprland keybind practice:
//! wpctl (PipeWire), brightnessctl, grim+slurp, hyprlock, hyprsunset,
//! nmcli, bluetoothctl. Slots stay enums. Walks stay driver-owned.

use crate::catalog::slot_value;
use crate::types::{Golden, ModuleDef, Page, PageKind, Policy, Slot, Snap};

pub fn patch_pages(pages: &mut [Page]) {
    for page in pages.iter_mut() {
        match page.id.as_str() {
            "wm.focus" => {
                if let Some(slot) = page.slots.iter_mut().find(|s| s.id == "target") {
                    remap_aliases(slot, "firefox", &["firefox"]);
                    remap_aliases(slot, "kitty", &["kitty"]);
                    push_unique(
                        slot,
                        slot_value("zen", "Zen Browser", &["zen", "zen browser", "browser"]),
                    );
                    push_unique(
                        slot,
                        slot_value("foot", "Foot", &["foot", "terminal", "shell"]),
                    );
                }
            }
            "launch.app" => {
                if let Some(slot) = page.slots.iter_mut().find(|s| s.id == "app") {
                    remap_aliases(slot, "firefox", &["firefox"]);
                    remap_aliases(slot, "kitty", &["kitty"]);
                    push_unique(
                        slot,
                        slot_value("zen", "Zen Browser", &["zen", "zen browser", "browser"]),
                    );
                    push_unique(
                        slot,
                        slot_value("foot", "Foot", &["foot", "terminal", "shell"]),
                    );
                }
            }
            "bucky.mic_mute" => {
                // Keep ASR mute distinct from PipeWire source mute.
                page.aliases = vec![
                    "mute the buddy mic".into(),
                    "stop listening".into(),
                    "mute asr".into(),
                ];
            }
            "audio.speakers" => {
                page.aliases = vec!["speakers".into(), "tv speakers".into()];
            }
            "scene.apply" => {
                for extra in ["quiet hours", "house idle", "bedside night", "living watch"] {
                    push_alias(page, extra);
                }
            }
            "display.bump" => push_alias(page, "brighten the screen"),
            "mail.unread" => {
                push_alias(page, "inbox");
                push_alias(page, "what's in my inbox");
            }
            "wm.workspace" => push_alias(page, "go to"),
            _ => {}
        }
    }
}

fn push_alias(page: &mut Page, alias: &str) {
    if !page.aliases.iter().any(|a| a == alias) {
        page.aliases.push(alias.into());
    }
}

fn push_unique(slot: &mut Slot, value: crate::types::SlotValue) {
    if !slot.values.iter().any(|v| v.id == value.id) {
        slot.values.push(value);
    }
}

fn remap_aliases(slot: &mut Slot, id: &str, aliases: &[&str]) {
    if let Some(v) = slot.values.iter_mut().find(|v| v.id == id) {
        v.aliases = aliases.iter().map(|s| (*s).to_string()).collect();
    }
}

pub fn patch_modules(_modules: &mut [ModuleDef]) {}

pub fn patch_snaps(snaps: &mut [Snap]) {
    for snap in snaps.iter_mut() {
        match snap.id.as_str() {
            "desk" | "desk-jellyfin" => {
                snap.lock_available = true;
                snap.monitors = 1;
                snap.bluetooth_on = true;
                snap.network = Some("ethernet".into());
                snap.notification = Some("Slack · standup in 10".into());
                snap.sinks = vec!["speakers".into(), "hdmi".into()];
                snap.focused_output = Some("HDMI-A-1".into());
                if !snap.allowlist.iter().any(|a| a == "zen") {
                    snap.allowlist.push("zen".into());
                }
                if !snap.allowlist.iter().any(|a| a == "foot") {
                    snap.allowlist.push("foot".into());
                }
            }
            "kitchen" => {
                snap.lock_available = true;
                snap.monitors = 1;
                snap.bluetooth_on = true;
                snap.network = Some("ethernet".into());
                snap.sinks = vec!["speakers".into()];
            }
            "guest-living" => {
                snap.lock_available = true;
                snap.monitors = 1;
                snap.bluetooth_on = true;
                snap.network = Some("ethernet".into());
                snap.sinks = vec!["tv".into()];
                snap.default_sink = if snap.default_sink.is_empty() {
                    "tv".into()
                } else {
                    snap.default_sink.clone()
                };
            }
            _ => {}
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn page(
    id: &str,
    module: &str,
    kind: PageKind,
    title: &str,
    summary: &str,
    aliases: &[&str],
    slots: Vec<Slot>,
    when: &str,
    policy: Policy,
    walk: Option<&str>,
    examples: &[&str],
    refuse: &[&str],
) -> Page {
    Page {
        id: id.into(),
        module: module.into(),
        kind,
        title: title.into(),
        summary: summary.into(),
        aliases: aliases.iter().map(|s| (*s).to_string()).collect(),
        slots,
        when: when.into(),
        policy,
        confirm: false,
        walk: walk.map(|s| s.into()),
        ask_shape: None,
        pose: None,
        examples: examples.iter().map(|s| (*s).to_string()).collect(),
        refuse: refuse.iter().map(|s| (*s).to_string()).collect(),
    }
}

#[allow(clippy::too_many_arguments)]
fn ask_page(
    id: &str,
    module: &str,
    title: &str,
    summary: &str,
    aliases: &[&str],
    slots: Vec<Slot>,
    when: &str,
    policy: Policy,
    ask_shape: &str,
    examples: &[&str],
    refuse: &[&str],
) -> Page {
    let mut p = page(
        id,
        module,
        PageKind::Ask,
        title,
        summary,
        aliases,
        slots,
        when,
        policy,
        None,
        examples,
        refuse,
    );
    p.ask_shape = Some(ask_shape.into());
    p
}

pub fn pages() -> Vec<Page> {
    let dir = Slot {
        id: "dir".into(),
        label: "direction".into(),
        required: true,
        values: vec![
            slot_value("left", "left", &["left", "port"]),
            slot_value("right", "right", &["right", "starboard"]),
            slot_value("up", "up", &["up", "above"]),
            slot_value("down", "down", &["down", "below"]),
        ],
    };
    let dpms = Slot {
        id: "action".into(),
        label: "dpms".into(),
        required: false,
        values: vec![
            slot_value("off", "off", &["off", "sleep", "screens off"]),
            slot_value("on", "on", &["on", "wake", "screens on"]),
        ],
    };

    vec![
        page(
            "wm.cycle_next",
            "wm",
            PageKind::Act,
            "Next window",
            "Cycle focus to the next client on this workspace (hyprctl cyclenext).",
            &["next window", "cycle windows", "other window"],
            vec![],
            "At least one client is mapped.",
            Policy::Household,
            Some("hyprctl dispatch cyclenext"),
            &["next window", "cycle windows"],
            &["click the second tab"],
        ),
        page(
            "wm.movefocus",
            "wm",
            PageKind::Act,
            "Move focus",
            "Move focus in a cardinal direction. Hyprland movefocus l/r/u/d.",
            &[
                "focus left",
                "focus right",
                "focus up",
                "focus down",
                "move focus",
            ],
            vec![dir],
            "Another client exists in that direction.",
            Policy::Household,
            Some("hyprctl dispatch movefocus <dir>"),
            &["focus left", "move focus right"],
            &[],
        ),
        page(
            "wm.pin",
            "wm",
            PageKind::Act,
            "Pin window",
            "Pin the focused window so it stays on every workspace.",
            &["pin this", "pin the window", "always on top"],
            vec![],
            "A focused client exists.",
            Policy::Household,
            Some("hyprctl dispatch pin"),
            &["pin this"],
            &[],
        ),
        page(
            "wm.center",
            "wm",
            PageKind::Act,
            "Center window",
            "Center the focused floating (or to-be-centered) window.",
            &["center this", "center the window"],
            vec![],
            "A focused client exists.",
            Policy::Household,
            Some("hyprctl dispatch centerwindow"),
            &["center this"],
            &[],
        ),
        page(
            "wm.swap_monitor",
            "wm",
            PageKind::Act,
            "Swap monitors",
            "Swap the active workspaces of this output and the next.",
            &["swap monitors", "swap screens", "swap displays"],
            vec![],
            "More than one monitor is connected.",
            Policy::Household,
            Some("hyprctl dispatch swapactiveworkspaces current +1"),
            &["swap monitors"],
            &[],
        ),
        page(
            "audio.mic_mute",
            "audio",
            PageKind::Act,
            "Mute microphone",
            "Mute the default PipeWire source. Distinct from sink mute and Bucky ASR mute.",
            &["mute the mic", "mute mic", "mute my microphone"],
            vec![],
            "Default source exists and is not muted.",
            Policy::Household,
            Some("wpctl set-mute @DEFAULT_AUDIO_SOURCE@ 1"),
            &["mute the mic"],
            &[],
        ),
        page(
            "audio.mic_unmute",
            "audio",
            PageKind::Act,
            "Unmute microphone",
            "Unmute the default PipeWire source.",
            &["unmute the mic", "unmute mic", "mic on"],
            vec![],
            "Default source is muted.",
            Policy::Household,
            Some("wpctl set-mute @DEFAULT_AUDIO_SOURCE@ 0"),
            &["unmute the mic"],
            &[],
        ),
        page(
            "audio.tv",
            "audio",
            PageKind::Act,
            "TV / HDMI sink",
            "Set the default sink to the HDMI/TV node (this box: Vega HDMI).",
            &["tv audio", "audio on the tv", "hdmi audio"],
            vec![],
            "An HDMI sink is in wpctl status.",
            Policy::Household,
            Some("wpctl set-default <hdmi-id>"),
            &["tv audio"],
            &[],
        ),
        page(
            "session.lock",
            "session",
            PageKind::Act,
            "Lock screen",
            "Lock with hyprlock. Never invent a password prompt.",
            &["lock the screen", "lock screen", "lock the box"],
            vec![],
            "hyprlock is installed.",
            Policy::Household,
            Some("hyprlock"),
            &["lock the screen"],
            &[],
        ),
        page(
            "session.dpms",
            "session",
            PageKind::Act,
            "Sleep the displays",
            "Hyprland DPMS off/on. Not a system suspend.",
            &["screens off", "screens on", "sleep the displays"],
            vec![dpms],
            "Hyprland is up.",
            Policy::Household,
            Some("hyprctl dispatch dpms off|on"),
            &["screens off", "screens on"],
            &[],
        ),
        page(
            "capture.region",
            "capture",
            PageKind::Act,
            "Region screenshot",
            "Select a region with slurp, capture with grim. Confirm if a guest is present.",
            &["screenshot a region", "capture a region", "grab a region"],
            vec![],
            "slurp and grim are present.",
            Policy::Confirm,
            Some("grim -g \"$(slurp)\" ~/Pictures/vikett.png"),
            &["screenshot a region"],
            &["email that screenshot"],
        ),
        page(
            "notify.dismiss",
            "notify",
            PageKind::Act,
            "Dismiss notification",
            "Dismiss the latest notification. No arbitrary D-Bus.",
            &["dismiss that", "clear the ping", "dismiss notification"],
            vec![],
            "Notification history is non-empty.",
            Policy::Household,
            Some("makoctl dismiss"),
            &["dismiss that"],
            &[],
        ),
        ask_page(
            "network.ask",
            "network",
            "Network status",
            "Return the active connection kind/name. Toggling Wi-Fi is not a v0 door.",
            &["what's the network", "am i online", "network status"],
            vec![],
            "nmcli reports an active connection.",
            Policy::Household,
            "{ connection }",
            &["am I online", "what's the network"],
            &["turn off wifi", "change dns"],
        ),
        ask_page(
            "bluetooth.ask",
            "bluetooth",
            "Bluetooth status",
            "Whether the adapter is powered. Pairing is not a page.",
            &["is bluetooth on", "bluetooth status"],
            vec![],
            "Adapter is present.",
            Policy::Household,
            "{ powered }",
            &["is bluetooth on"],
            &["pair the headphones", "connect to the speaker"],
        ),
    ]
}

pub fn modules() -> Vec<ModuleDef> {
    vec![
        ModuleDef {
            id: "session".into(),
            title: "Session".into(),
            summary: "Lock and DPMS stay. Suspend, reboot, and poweroff are owner plus confirm. Idle is reserved because force_idle is not an inhibitor."
                .into(),
            driver: "hyprlock ; hyprctl dispatch dpms ; systemctl suspend|reboot|poweroff".into(),
            snap: "lock binary present + dpms status".into(),
            priority: 22,
        },
        ModuleDef {
            id: "network".into(),
            title: "Network".into(),
            summary: "Ask stays the connection field. VPN connect is confirm and a lists.vpn name. No SSID, DNS, or nmcli down."
                .into(),
            driver: "nmcli connection show ; nmcli connection up <vpn>".into(),
            snap: "active connection type + name, lists.vpn".into(),
            priority: 11,
        },
        ModuleDef {
            id: "bluetooth".into(),
            title: "Bluetooth".into(),
            summary: "Ask is adapter power. Connect and disconnect an allowlisted device. Pair is not a door."
                .into(),
            driver: "bluetoothctl show ; bluetoothctl connect|disconnect <device>".into(),
            snap: "adapter Powered yes/no, lists.bt_device".into(),
            priority: 10,
        },
    ]
}

pub fn goldens() -> Vec<Golden> {
    use std::collections::BTreeMap;
    let slots = |pairs: &[(&str, &str)]| {
        Some(
            pairs
                .iter()
                .map(|(k, v)| ((*k).into(), (*v).into()))
                .collect::<BTreeMap<_, _>>(),
        )
    };
    vec![
        Golden {
            id: "next-window".into(),
            snap: "desk".into(),
            utterance: "next window".into(),
            expect_page: Some("wm.cycle_next".into()),
            expect_slots: None,
            notes: "Hyprland cyclenext.".into(),
        },
        Golden {
            id: "focus-left".into(),
            snap: "desk".into(),
            utterance: "focus left".into(),
            expect_page: Some("wm.movefocus".into()),
            expect_slots: slots(&[("dir", "left")]),
            notes: "Cardinal focus, not pixel click.".into(),
        },
        Golden {
            id: "pin-this".into(),
            snap: "desk".into(),
            utterance: "pin this".into(),
            expect_page: Some("wm.pin".into()),
            expect_slots: None,
            notes: "hyprctl pin.".into(),
        },
        Golden {
            id: "mute-mic".into(),
            snap: "desk".into(),
            utterance: "mute the mic".into(),
            expect_page: Some("audio.mic_mute".into()),
            expect_slots: None,
            notes: "Source mute, not sink mute.".into(),
        },
        Golden {
            id: "lock-screen".into(),
            snap: "desk".into(),
            utterance: "lock the screen".into(),
            expect_page: Some("session.lock".into()),
            expect_slots: None,
            notes: "hyprlock.".into(),
        },
        Golden {
            id: "screens-off".into(),
            snap: "desk".into(),
            utterance: "screens off".into(),
            expect_page: Some("session.dpms".into()),
            expect_slots: slots(&[("action", "off")]),
            notes: "DPMS, not suspend.".into(),
        },
        Golden {
            id: "region-shot".into(),
            snap: "desk".into(),
            utterance: "screenshot a region".into(),
            expect_page: Some("capture.region".into()),
            expect_slots: None,
            notes: "grim + slurp.".into(),
        },
        Golden {
            id: "am-i-online".into(),
            snap: "desk".into(),
            utterance: "am I online".into(),
            expect_page: Some("network.ask".into()),
            expect_slots: None,
            notes: "Ask, not nmcli down.".into(),
        },
        Golden {
            id: "bt-on".into(),
            snap: "desk".into(),
            utterance: "is bluetooth on".into(),
            expect_page: Some("bluetooth.ask".into()),
            expect_slots: None,
            notes: "Ask-only.".into(),
        },
        Golden {
            id: "refuse-wifi-off".into(),
            snap: "desk".into(),
            utterance: "turn off wifi".into(),
            expect_page: None,
            expect_slots: None,
            notes: "No free network act.".into(),
        },
        Golden {
            id: "swap-monitors-single".into(),
            snap: "desk".into(),
            utterance: "swap monitors".into(),
            expect_page: None,
            expect_slots: None,
            notes: "Single output on the fixture — page not live.".into(),
        },
        Golden {
            id: "tv-audio".into(),
            snap: "desk".into(),
            utterance: "tv audio".into(),
            expect_page: Some("audio.tv".into()),
            expect_slots: None,
            notes: "HDMI sink, this workstation has Vega HDMI.".into(),
        },
        Golden {
            id: "center-this".into(),
            snap: "desk".into(),
            utterance: "center this".into(),
            expect_page: Some("wm.center".into()),
            expect_slots: None,
            notes: "hyprctl centerwindow.".into(),
        },
        Golden {
            id: "dismiss-ping".into(),
            snap: "desk".into(),
            utterance: "dismiss that".into(),
            expect_page: Some("notify.dismiss".into()),
            expect_slots: None,
            notes: "History non-empty after snap patch.".into(),
        },
        Golden {
            id: "refuse-cozy".into(),
            snap: "desk".into(),
            utterance: "make it cozy".into(),
            expect_page: None,
            expect_slots: None,
            notes: "Scenes are named ids only.".into(),
        },
    ]
}
