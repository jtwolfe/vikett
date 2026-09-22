//! Screen record. `wf-recorder` wins when it is in `bins`. The other name is
//! `gpu-screen-recorder`, used only when wf-recorder is absent.
//! No `nvidia-smi`, no `--list-monitors`, no `-encoder`. Screenshot pages stay
//! in the v0 walk.

use crate::types::Snap;

pub fn is_live(snap: &Snap) -> (bool, String) {
    match recorder(snap) {
        Some(bin) => (true, format!("recorder {bin}")),
        None => (false, "no screen recorder in bins".into()),
    }
}

pub fn command(snap: &Snap, stop: bool) -> String {
    let Some(bin) = recorder(snap) else {
        return "reserved".into();
    };
    if stop {
        return format!("pkill -INT -x {bin}");
    }
    match bin {
        "wf-recorder" => "wf-recorder -f ~/Videos/vikett.mp4".into(),
        "gpu-screen-recorder" => "gpu-screen-recorder -w focused -o ~/Videos/vikett.mp4".into(),
        _ => "reserved".into(),
    }
}

fn recorder(snap: &Snap) -> Option<&'static str> {
    if snap.bins.iter().any(|b| b == "wf-recorder") {
        return Some("wf-recorder");
    }
    // gpu-screen-recorder is the other binary. Presence is `bins`, not a GPU probe.
    if snap.bins.iter().any(|b| b == "gpu-screen-recorder") {
        return Some("gpu-screen-recorder");
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::catalog::Catalog;
    use crate::decide::decide;
    use crate::types::RefereeKind;

    fn no_probe(cmd: &str) {
        assert!(!cmd.contains("nvidia"), "{cmd}");
        assert!(!cmd.contains("--list"), "{cmd}");
        assert!(!cmd.contains("-encoder"), "{cmd}");
        assert!(!cmd.contains("portal"), "{cmd}");
        assert!(!cmd.contains("dispatch exec"), "{cmd}");
        assert!(!cmd.contains("slurp"), "{cmd}");
        assert!(!cmd.contains("grim"), "{cmd}");
    }

    #[test]
    fn wf_recorder_first_and_neither_is_dead() {
        let cat = Catalog::load();
        let snap = cat.snap("desk-shelf").unwrap();
        let start = cat.page("capture.record_start").unwrap();
        let stop = cat.page("capture.record_stop").unwrap();
        assert!(start.confirm && stop.confirm);
        assert_ne!(cat.page("capture.screenshot").unwrap().id, start.id);

        let go = decide(&cat, "start screen recording", snap, RefereeKind::Lexical);
        assert_eq!(go.take.page_id.as_deref(), Some("capture.record_start"));
        assert!(go.confirm);
        let cmd = go.walk.unwrap().command;
        assert!(cmd.starts_with("wf-recorder "), "{cmd}");
        assert!(!cmd.contains("gpu-screen-recorder"), "{cmd}");
        no_probe(&cmd);

        let end = decide(&cat, "stop screen recording", snap, RefereeKind::Lexical);
        assert_eq!(end.take.page_id.as_deref(), Some("capture.record_stop"));
        assert!(end.confirm);
        assert_eq!(end.walk.unwrap().command, "pkill -INT -x wf-recorder");

        let shot = decide(&cat, "screenshot", snap, RefereeKind::Lexical);
        assert_eq!(shot.take.page_id.as_deref(), Some("capture.screenshot"));
        assert!(shot.walk.unwrap().command.contains("grim"),);
        let region = decide(&cat, "grab a region", snap, RefereeKind::Lexical);
        assert_eq!(region.take.page_id.as_deref(), Some("capture.region"));

        let mut gpu = snap.clone();
        gpu.bins.retain(|b| b != "wf-recorder");
        gpu.bins.push("gpu-screen-recorder".into());
        let cmd = command(&gpu, false);
        assert!(cmd.starts_with("gpu-screen-recorder -w focused"), "{cmd}");
        no_probe(&cmd);
        assert_eq!(command(&gpu, true), "pkill -INT -x gpu-screen-recorder");

        let mut none = snap.clone();
        none.bins
            .retain(|b| b != "wf-recorder" && b != "gpu-screen-recorder");
        let (ok, why) = is_live(&none);
        assert!(!ok, "{why}");
        let dead = decide(&cat, "start screen recording", &none, RefereeKind::Lexical);
        assert!(dead.take.page_id.is_none(), "{:?}", dead.take);
        assert!(dead.walk.is_none());

        let desk = cat.snap("desk").unwrap();
        let (ok, _) = is_live(desk);
        assert!(!ok);
        let still = decide(&cat, "screenshot", desk, RefereeKind::Lexical);
        assert_eq!(still.take.page_id.as_deref(), Some("capture.screenshot"));
    }
}
