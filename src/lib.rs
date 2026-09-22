//! Vikett interpreter: authored pages, live prune, typed take, walk.
//! The model never invents argv. Silence is a first-class result.

pub mod ask;
pub mod catalog;
pub mod classes;
pub mod decide;
pub mod drivers;
pub mod host;
pub mod keymap;
pub mod lexical;
pub mod model;
pub mod ontology;
pub mod prune;
pub mod refuse;
pub mod slots;
pub mod suite;
pub mod text;
pub mod trace;
pub mod types;
pub mod walk;

pub mod tui;

pub use catalog::Catalog;
pub use decide::{decide, decide_with_model, run_golden, GoldenRun};
pub use types::{EngineResult, RefereeKind, Snap, Take};

use std::sync::LazyLock;

pub static CATALOG: LazyLock<Catalog> = LazyLock::new(Catalog::load);

pub fn eval(utterance: &str, snap_id: &str, referee: RefereeKind) -> anyhow::Result<EngineResult> {
    let snap = CATALOG
        .snap(snap_id)
        .ok_or_else(|| anyhow::anyhow!("unknown snap {snap_id}"))?;
    eval_on(&CATALOG, utterance, snap, referee)
}

pub fn eval_on(
    cat: &Catalog,
    utterance: &str,
    snap: &Snap,
    referee: RefereeKind,
) -> anyhow::Result<EngineResult> {
    if referee == RefereeKind::Lexical {
        return Ok(decide(cat, utterance, snap, referee));
    }
    let (live, _dead) = prune::live_and_dead(&cat.pages, snap, Some(utterance));
    let model_take = model::take(referee, utterance, snap, &live, &cat.pages)?;
    Ok(decide_with_model(
        cat,
        utterance,
        snap,
        referee,
        Some(model_take),
    ))
}

pub fn dump_ontology(cat: &Catalog) -> anyhow::Result<()> {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("ontology");
    std::fs::write(root.join("pages.json"), pretty(&cat.pages)?)?;
    std::fs::write(root.join("modules.json"), pretty(&cat.modules)?)?;
    std::fs::write(root.join("snaps.json"), pretty(&cat.snaps)?)?;
    std::fs::write(root.join("goldens.json"), pretty(&cat.goldens)?)?;
    std::fs::write(root.join("phrases.json"), pretty(&cat.phrases)?)?;
    Ok(())
}

fn pretty<T: serde::Serialize>(v: &T) -> anyhow::Result<String> {
    Ok(serde_json::to_string_pretty(v)? + "\n")
}
