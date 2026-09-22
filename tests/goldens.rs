use vikett::decide::run_golden;
use vikett::prune::live_and_dead;
use vikett::types::RefereeKind;
use vikett::{decide, Catalog};

fn cat() -> Catalog {
    Catalog::load()
}

#[test]
fn l1_all_goldens() {
    let cat = cat();
    let mut failed = Vec::new();
    for g in &cat.goldens {
        let r = run_golden(&cat, g);
        if !r.pass {
            failed.push(format!(
                "{}: got {:?} expected {:?} slots_ok={} ({})",
                r.id, r.got, r.expected, r.slots_ok, r.detail
            ));
        }
    }
    assert!(
        failed.is_empty(),
        "{} goldens failed:\n{}",
        failed.len(),
        failed.join("\n")
    );
}

#[test]
fn l0_guest_hides_mail_and_calendar() {
    let cat = cat();
    let snap = cat.snap("guest-living").expect("guest snap");
    assert_eq!(snap.who, "jim");
    assert!(snap.guest);
    let (live, _) = live_and_dead(&cat.pages, snap, None);
    let leaked: Vec<_> = live
        .iter()
        .filter(|l| {
            l.page_id.starts_with("mail.")
                || l.page_id.starts_with("calendar.")
                || cat
                    .page(&l.page_id)
                    .map(|p| p.policy == vikett::types::Policy::Private)
                    .unwrap_or(false)
        })
        .map(|l| l.page_id.as_str())
        .collect();
    assert!(leaked.is_empty(), "guest leaked private pages: {leaked:?}");
    let (_, dead) = live_and_dead(&cat.pages, snap, Some("shared only"));
    let lock = dead
        .iter()
        .find(|d| d.page_id == "scene.lock_private")
        .expect("lock-private dead");
    assert!(lock.why.contains("guest flag"), "lock why {}", lock.why);
    let tree = decide(&cat, "shared only", snap, RefereeKind::Lexical)
        .trace
        .render();
    assert!(tree.contains("scene.lock_private"), "{tree}");
    assert!(tree.contains("scene.guest"), "{tree}");
    assert!(tree.contains("guest flag"), "{tree}");
    let (live, _) = live_and_dead(&cat.pages, snap, Some("how many downloads"));
    assert!(
        !live.iter().any(|l| l.page_id == "browser.downloads"),
        "guest saw browser.downloads"
    );
    let (_, dead) = live_and_dead(&cat.pages, snap, None);
    for id in [
        "mail.next_unread",
        "mail.archive",
        "mail.mark_read",
        "chat.ask_unread",
        "calendar.ask_today",
        "session.suspend",
        "session.reboot",
        "session.poweroff",
    ] {
        let row = dead
            .iter()
            .find(|d| d.page_id == id)
            .unwrap_or_else(|| panic!("{id} missing from dead"));
        assert!(row.why.contains("guest flag"), "{id} {}", row.why);
    }
}

#[test]
fn l0_blank_or_unknown_who_hides_mail_and_private() {
    let cat = cat();
    let desk = cat.snap("desk").expect("desk").clone();
    assert!(!desk.guest);
    let (live, _) = live_and_dead(&cat.pages, &desk, None);
    assert!(
        live.iter().any(|l| l.page_id.starts_with("mail.")),
        "desk should still show mail before who is cleared"
    );
    for who in ["", "unknown", "UNKNOWN"] {
        let mut snap = desk.clone();
        snap.who = who.into();
        snap.guest = false;
        let (live, dead) = live_and_dead(&cat.pages, &snap, None);
        let because = if who.is_empty() {
            "who empty"
        } else {
            "who unknown"
        };
        let leaked: Vec<_> = live
            .iter()
            .filter(|l| {
                l.page_id.starts_with("mail.")
                    || cat
                        .page(&l.page_id)
                        .is_some_and(|p| p.policy == vikett::types::Policy::Private)
            })
            .map(|l| l.page_id.clone())
            .collect();
        assert!(
            leaked.is_empty(),
            "who={who:?} leaked mail or private: {leaked:?}"
        );
        assert!(
            live.iter().any(|l| l.page_id == "wm.workspace"),
            "who={who:?} hid household pages"
        );
        let owner: Vec<_> = live
            .iter()
            .filter(|l| {
                cat.page(&l.page_id)
                    .is_some_and(|p| p.policy == vikett::types::Policy::Owner)
            })
            .map(|l| l.page_id.clone())
            .collect();
        assert!(
            owner.is_empty(),
            "who={who:?} leaked owner pages: {owner:?}"
        );
        assert!(
            dead.iter()
                .any(|d| d.page_id.starts_with("mail.") && d.why.contains(because)),
            "who={who:?} mail why missing {because}: {dead:?}"
        );
        for id in [
            "mail.next_unread",
            "mail.archive",
            "mail.mark_read",
            "chat.ask_unread",
            "session.suspend",
            "session.reboot",
            "session.poweroff",
        ] {
            assert!(
                dead.iter()
                    .any(|d| d.page_id == id && d.why.contains(because)),
                "who={who:?} {id} why missing {because}"
            );
        }
        assert!(
            dead.iter()
                .any(|d| d.page_id == "scene.lock_private" && d.why.contains(because)),
            "who={who:?} owner why missing {because}"
        );
    }
}

#[test]
fn l0_timer_dead_on_desk() {
    let cat = cat();
    let snap = cat.snap("desk").unwrap();
    let (live, dead) = live_and_dead(&cat.pages, snap, Some("add five minutes"));
    assert!(
        !live.iter().any(|l| l.page_id == "timer.add"),
        "timer.add must be dead without a running timer"
    );
    assert!(dead.iter().any(|d| d.page_id == "timer.add"));
}

#[test]
fn l0_handoff_always_denied() {
    let cat = cat();
    for snap in &cat.snaps {
        let (live, dead) = live_and_dead(&cat.pages, snap, Some("put it on the tv"));
        assert!(
            !live.iter().any(|l| l.page_id == "scene.handoff"),
            "handoff live on {}",
            snap.id
        );
        assert!(dead.iter().any(|d| d.page_id == "scene.handoff"));
    }
}

#[test]
fn compound_first_take_only() {
    let cat = cat();
    let snap = cat.snap("desk-jellyfin").unwrap();
    let r = decide(
        &cat,
        "switch to jellyfin and fullscreen",
        snap,
        RefereeKind::Lexical,
    );
    assert_eq!(r.take.page_id.as_deref(), Some("wm.focus"));
    assert_eq!(r.remaining, vec!["fullscreen"]);
}

#[test]
fn tree_is_nonempty() {
    let cat = cat();
    let snap = cat.snap("desk").unwrap();
    let r = decide(&cat, "mute", snap, RefereeKind::Lexical);
    let tree = r.trace.render();
    assert!(tree.contains("prune"));
    assert!(tree.contains("take"));
}

#[test]
fn slots_asserted_on_volume() {
    let cat = cat();
    let g = cat.goldens.iter().find(|g| g.id == "vol-little").unwrap();
    let r = run_golden(&cat, g);
    assert!(r.pass, "{r:?}");
    assert!(r.slots_ok);
}

#[test]
fn browser_new_tab_uses_send_shortcut() {
    let cat = cat();
    let snap = cat.snap("desk-zen").unwrap();
    let r = decide(&cat, "new tab", snap, RefereeKind::Lexical);
    assert_eq!(r.take.page_id.as_deref(), Some("browser.tab_new"));
    let walk = r.walk.unwrap().command;
    assert!(walk.contains("hl.dsp.send_shortcut"), "{walk}");
    assert!(walk.contains("address:0xzen"), "{walk}");
    assert!(walk.contains("mods = \"CTRL\""), "{walk}");
    assert!(walk.contains("key = \"T\""), "{walk}");
    assert!(!walk.contains("focuswindow"), "{walk}");
    assert!(!walk.contains("dispatch exec "), "{walk}");
}

#[test]
fn browser_fullscreen_keeps_empty_mods() {
    let cat = cat();
    let snap = cat.snap("desk-zen").unwrap();
    let r = decide(&cat, "fullscreen the page", snap, RefereeKind::Lexical);
    let walk = r.walk.unwrap().command;
    assert!(walk.contains("mods = \"\""), "{walk}");
    assert!(walk.contains("key = \"F11\""), "{walk}");
    assert!(!walk.contains("dispatch fullscreen"), "{walk}");
}

#[test]
fn browser_focus_exec_or_address() {
    let cat = cat();
    let snap = cat.snap("desk-zen").unwrap();
    let mapped = decide(&cat, "focus the browser", snap, RefereeKind::Lexical);
    let focus = mapped.walk.unwrap().command;
    assert_eq!(
        focus,
        "hyprctl dispatch 'hl.dsp.focus({ window = \"address:0xzen\" })'"
    );
    let unmapped = decide(&cat, "firefox window", snap, RefereeKind::Lexical);
    assert_eq!(unmapped.take.page_id.as_deref(), Some("browser.focus"));
    assert_eq!(
        unmapped.walk.unwrap().command,
        "hyprctl dispatch 'hl.dsp.exec_cmd(\"firefox\")'"
    );
}

#[test]
fn browser_zoom_lot_repeats_chord() {
    let cat = cat();
    let snap = cat.snap("desk-zen").unwrap();
    let little = decide(&cat, "zoom in", snap, RefereeKind::Lexical);
    let lot = decide(&cat, "zoom in a lot", snap, RefereeKind::Lexical);
    assert_eq!(
        little
            .walk
            .as_ref()
            .unwrap()
            .command
            .matches("send_shortcut")
            .count(),
        1
    );
    assert_eq!(
        lot.walk
            .as_ref()
            .unwrap()
            .command
            .matches("send_shortcut")
            .count(),
        3
    );
    assert!(lot.walk.unwrap().command.contains("key = \"plus\""));
}

#[test]
fn browser_downloads_bucket_not_ok() {
    let cat = cat();
    let snap = cat.snap("desk-zen").unwrap();
    let r = decide(&cat, "how many downloads", snap, RefereeKind::Lexical);
    assert_eq!(r.take.page_id.as_deref(), Some("browser.downloads"));
    assert_eq!(r.answer, Some(serde_json::json!({ "bucket": "few" })));
}

#[test]
fn mute_does_not_score_browser_pages() {
    let cat = cat();
    let snap = cat.snap("desk").unwrap();
    let (live, _) = live_and_dead(&cat.pages, snap, Some("mute"));
    let ids: std::collections::HashSet<&str> = live.iter().map(|l| l.page_id.as_str()).collect();
    let scored = vikett::lexical::score_live(&cat.pages, &ids, "mute", snap);
    assert!(
        scored
            .iter()
            .all(|s| !s.page_id.starts_with("browser.") && !s.page_id.starts_with("term.")),
        "{:?}",
        scored.iter().map(|s| &s.page_id).collect::<Vec<_>>()
    );
}

#[test]
fn foot_only_dead_alias_silences() {
    let cat = cat();
    let mut snap = cat.snap("desk").unwrap().clone();
    snap.clients = vec![vikett::types::Client {
        address: "0xfoot".into(),
        class: "foot".into(),
        title: "~".into(),
        workspace: "1".into(),
        focused: true,
        fullscreen: false,
        floating: false,
    }];
    snap.media_playing = false;
    snap.now_playing = None;
    for utt in ["close tab", "next tab", "fullscreen the page"] {
        let r = decide(&cat, utt, &snap, RefereeKind::Lexical);
        assert!(r.take.page_id.is_none(), "{utt} got {:?}", r.take);
        assert!(r.walk.is_none(), "{utt}");
        assert!(r.answer.is_none(), "{utt}");
    }
}

#[test]
fn browser_reserved_and_chrome_private_stay_dead() {
    let cat = cat();
    let zen = cat.snap("desk-zen").unwrap();
    for (utt, id) in [
        ("pin tab", "browser.tab_pin"),
        ("mute tab", "browser.tab_mute"),
        ("reader mode", "browser.reader"),
        ("picture in picture", "browser.pip"),
        ("bookmark this page", "browser.bookmark"),
        ("translate this page", "browser.translate"),
        ("cancel the download", "browser.download_cancel"),
    ] {
        let (live, dead) = live_and_dead(&cat.pages, zen, Some(utt));
        assert!(!live.iter().any(|l| l.page_id == id), "{id} live");
        let why = &dead.iter().find(|d| d.page_id == id).unwrap().why;
        assert!(
            why.contains("no chord") || why.contains("reserved"),
            "{id} {why}"
        );
    }
    let browsers = cat.snap("desk-browsers").unwrap();
    let (live, dead) = live_and_dead(&cat.pages, browsers, Some("chrome private window"));
    assert!(!live.iter().any(|l| l.page_id == "browser.private"));
    let why = &dead
        .iter()
        .find(|d| d.page_id == "browser.private")
        .unwrap()
        .why;
    assert!(
        why.contains("no chord") || why.contains("reserved"),
        "{why}"
    );
}

#[test]
fn vendor_doors_follow_class_and_zen_index_stays_reserved() {
    let cat = cat();
    let zen = cat.snap("desk-zen").unwrap();
    let desk = cat.snap("desk").unwrap();
    let browsers = cat.snap("desk-browsers").unwrap();
    let kitchen = cat.snap("kitchen").unwrap();

    let dead_why = |snap: &vikett::Snap, utt: &str, id: &str| -> String {
        let (live, dead) = live_and_dead(&cat.pages, snap, Some(utt));
        assert!(!live.iter().any(|l| l.page_id == id), "{id} live on {utt}");
        dead.iter()
            .find(|d| d.page_id == id)
            .unwrap_or_else(|| panic!("{id} missing from dead on {utt}"))
            .why
            .clone()
    };

    let why = dead_why(desk, "firefox container work", "browser.container");
    assert!(
        why.contains("no chord") || why.contains("reserved"),
        "{why}"
    );
    let why = dead_why(zen, "firefox container work", "browser.container");
    assert!(why.contains("no matching client"), "{why}");
    let why = dead_why(browsers, "chrome profile work", "browser.profile");
    assert!(
        why.contains("no chord") || why.contains("reserved"),
        "{why}"
    );
    let why = dead_why(kitchen, "collapse tab group", "browser.tab_group_collapse");
    assert!(
        why.contains("no chord") || why.contains("reserved"),
        "{why}"
    );
    for id in [
        "browser.zen_ws",
        "browser.compact",
        "browser.glance_open",
        "browser.glance_close",
        "browser.zen_ws_new",
        "browser.web_panel",
        "browser.essential",
        "browser.tab_move_ws",
    ] {
        let (live, dead) = live_and_dead(&cat.pages, zen, Some("zen workspace two"));
        assert!(!live.iter().any(|l| l.page_id == id), "{id}");
        let why = &dead.iter().find(|d| d.page_id == id).unwrap().why;
        assert!(
            why.contains("no chord") || why.contains("reserved") || why.contains("app missing"),
            "{id} {why}"
        );
    }

    let silenced = decide(&cat, "zen workspace two", zen, RefereeKind::Lexical);
    assert!(silenced.take.page_id.is_none(), "{:?}", silenced.take);
    assert!(silenced.walk.is_none());

    let hypr = decide(&cat, "workspace two zen", zen, RefereeKind::Lexical);
    assert_eq!(hypr.take.page_id.as_deref(), Some("wm.workspace"));
    assert_eq!(hypr.take.slots.get("ws").map(String::as_str), Some("2"));

    let walked = decide(&cat, "next workspace", zen, RefereeKind::Lexical);
    assert_eq!(walked.take.page_id.as_deref(), Some("browser.zen_ws_next"));
    let walk = walked.walk.unwrap().command;
    assert!(walk.contains("hl.dsp.send_shortcut"), "{walk}");
    assert!(walk.contains("address:0xzen"), "{walk}");
    assert!(
        walk.contains("mods = \"ALT + CTRL\", key = \"E\""),
        "{walk}"
    );
    assert!(!walk.contains("focuswindow"), "{walk}");

    let index = decide(&cat, "zen workspace two", browsers, RefereeKind::Lexical);
    assert_eq!(index.take.page_id.as_deref(), Some("browser.zen_ws"));
    assert_eq!(index.take.slots.get("ws").map(String::as_str), Some("2"));
    let walk = index.walk.unwrap().command;
    assert!(walk.contains("hl.dsp.send_shortcut"), "{walk}");
    assert!(walk.contains("address:"), "{walk}");
    assert!(walk.contains("key = \"2\""), "{walk}");
    assert!(!walk.contains("key = \"1\""), "{walk}");

    let page = cat.page("browser.container").unwrap();
    let mut slots = std::collections::BTreeMap::new();
    slots.insert("app".into(), "zen".into());
    let (ok, why) = vikett::drivers::browser::is_live(page, zen, &slots);
    assert!(!ok);
    assert!(why.contains("wrong browser"), "{why}");

    let banking = vikett::slots::fill(page, "firefox container banking", desk);
    assert_eq!(
        banking.slots.get("container").map(String::as_str),
        Some("banking")
    );
    assert_eq!(
        banking.slots.get("app").map(String::as_str),
        Some("firefox")
    );
    let profile = cat.page("browser.profile").unwrap();
    let filled = vikett::slots::fill(profile, "chrome profile work", browsers);
    assert_eq!(
        filled.slots.get("profile").map(String::as_str),
        Some("work")
    );
    assert_eq!(filled.slots.get("app").map(String::as_str), Some("chrome"));
}

#[test]
fn term_focus_is_address_or_exec_cmd() {
    let cat = cat();
    let zen = cat.snap("desk-zen").unwrap();
    let mapped = decide(&cat, "focus the foot terminal", zen, RefereeKind::Lexical);
    assert_eq!(mapped.take.page_id.as_deref(), Some("term.focus"));
    let focus = mapped.walk.unwrap().command;
    assert_eq!(
        focus,
        "hyprctl dispatch 'hl.dsp.focus({ window = \"address:0xfoot\" })'"
    );
    assert!(!focus.contains("dispatch exec"), "{focus}");
    assert!(!focus.contains("exec_cmd"), "{focus}");

    let desk = cat.snap("desk").unwrap();
    let unmapped = decide(&cat, "focus the foot terminal", desk, RefereeKind::Lexical);
    assert_eq!(unmapped.take.page_id.as_deref(), Some("term.focus"));
    assert_eq!(
        unmapped.walk.unwrap().command,
        "hyprctl dispatch 'hl.dsp.exec_cmd(\"foot\")'"
    );
}

#[test]
fn term_new_window_never_execs() {
    let cat = cat();
    let zen = cat.snap("desk-zen").unwrap();
    let mapped = decide(&cat, "new foot window", zen, RefereeKind::Lexical);
    assert_eq!(mapped.take.page_id.as_deref(), Some("term.new_window"));
    let walk = mapped.walk.unwrap().command;
    assert!(walk.contains("hl.dsp.send_shortcut"), "{walk}");
    assert!(walk.contains("address:0xfoot"), "{walk}");
    assert!(walk.contains("key = \"N\""), "{walk}");
    assert!(!walk.contains("exec_cmd"), "{walk}");
    assert!(!walk.contains("dispatch exec"), "{walk}");

    let desk = cat.snap("desk").unwrap();
    let unmapped = decide(&cat, "new foot window", desk, RefereeKind::Lexical);
    assert!(unmapped.take.page_id.is_none(), "{:?}", unmapped.take);
    assert!(unmapped.walk.is_none());
}

#[test]
fn term_font_lot_repeats_and_little_is_default() {
    let cat = cat();
    let snap = cat.snap("desk-zen").unwrap();
    let little = decide(&cat, "bigger foot font", snap, RefereeKind::Lexical);
    let lot = decide(&cat, "bigger foot font a lot", snap, RefereeKind::Lexical);
    assert_eq!(
        little.take.slots.get("amount").map(String::as_str),
        Some("little")
    );
    assert_eq!(
        little
            .walk
            .as_ref()
            .unwrap()
            .command
            .matches("send_shortcut")
            .count(),
        1
    );
    assert_eq!(
        lot.walk
            .as_ref()
            .unwrap()
            .command
            .matches("send_shortcut")
            .count(),
        3
    );
    assert!(lot.walk.unwrap().command.contains("key = \"plus\""));
}

#[test]
fn desk_zen_next_tab_stays_browser() {
    let cat = cat();
    let snap = cat.snap("desk-zen").unwrap();
    let r = decide(&cat, "next tab", snap, RefereeKind::Lexical);
    assert_eq!(r.take.page_id.as_deref(), Some("browser.tab_next"));
}

#[test]
fn how_busy_stays_ask_today_with_bucket() {
    let cat = cat();
    assert!(cat.page("calendar.ask_busy").is_none());
    let snap = cat.snap("desk").unwrap();
    let r = decide(&cat, "how busy am I today", snap, RefereeKind::Lexical);
    assert_eq!(r.take.page_id.as_deref(), Some("calendar.ask_today"));
    let answer = r.answer.unwrap();
    assert_eq!(answer["remaining"], 1);
    assert_eq!(answer["bucket"], "light");
    assert!(answer.get("title").is_none(), "{answer}");
}

#[test]
fn mail_archive_confirms_and_does_not_exec() {
    let cat = cat();
    let snap = cat.snap("desk").unwrap();
    let r = decide(&cat, "archive the mail", snap, RefereeKind::Lexical);
    assert_eq!(r.take.page_id.as_deref(), Some("mail.archive"));
    assert!(r.confirm);
    let walk = r.walk.unwrap().command;
    assert!(walk.contains("notmuch"), "{walk}");
    assert!(!walk.contains("dispatch exec"), "{walk}");
    assert!(!walk.contains("send"), "{walk}");
}

#[test]
fn chat_unread_answer_is_a_bucket() {
    let cat = cat();
    let snap = cat.snap("desk-chat").unwrap();
    let r = decide(&cat, "unread on signal", snap, RefereeKind::Lexical);
    assert_eq!(r.take.page_id.as_deref(), Some("chat.ask_unread"));
    let answer = r.answer.unwrap();
    assert_eq!(answer["app"], "signal");
    assert_eq!(answer["bucket"], "few");
    assert_ne!(answer, serde_json::json!({ "ok": true }));
    let guest = cat.snap("guest-living").unwrap();
    assert_eq!(guest.who, "jim");
    let hidden = decide(&cat, "unread on signal", guest, RefereeKind::Lexical);
    assert!(hidden.take.page_id.is_none(), "{:?}", hidden.take);
    assert!(hidden.answer.is_none());
    assert!(hidden.walk.is_none());
}
