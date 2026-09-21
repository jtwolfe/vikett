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
