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
        scored.iter().all(|s| !s.page_id.starts_with("browser.")),
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
