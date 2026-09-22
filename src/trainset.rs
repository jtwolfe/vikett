//! Split must-pass rows from holdout.
//!
//! `train_rows` reads paraphrases, goldens, and exact suite cases.
//! It does not read the holdout file. A separate call writes that file.

use std::collections::BTreeSet;
use std::fs;
use std::path::Path;

use anyhow::{bail, Result};
use serde::{Deserialize, Serialize};

use crate::catalog::Catalog;
use crate::suite;
use crate::types::Holdout;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TrainRow {
    pub id: String,
    pub snap: String,
    pub utterance: String,
    pub expect_page: Option<String>,
    #[serde(default)]
    pub expect_slots: Option<std::collections::BTreeMap<String, String>>,
    #[serde(default)]
    pub notes: String,
}

pub fn refuse_live(snap_id: &str) -> Result<()> {
    if snap_id == "live" {
        bail!("snap id live is refused");
    }
    Ok(())
}

/// Train inputs only. Does not read holdout rows or the holdout file.
pub fn train_rows(cat: &Catalog) -> Result<Vec<TrainRow>> {
    let mut out = Vec::new();
    let mut seen = BTreeSet::new();
    let paraphrases: Vec<TrainRow> = serde_json::from_str(include_str!("../ontology/train.json"))
        .map_err(|e| anyhow::anyhow!("train.json: {e}"))?;
    for row in paraphrases {
        push(&mut out, &mut seen, row)?;
    }
    for g in &cat.goldens {
        push(
            &mut out,
            &mut seen,
            TrainRow {
                id: g.id.clone(),
                snap: g.snap.clone(),
                utterance: g.utterance.clone(),
                expect_page: g.expect_page.clone(),
                expect_slots: g.expect_slots.clone(),
                notes: g.notes.clone(),
            },
        )?;
    }
    for case in suite::base_cases(cat) {
        if !case.tags.iter().any(|t| t == "exact") {
            continue;
        }
        let slots = if case.expect_slots.is_empty() {
            None
        } else {
            Some(case.expect_slots)
        };
        push(
            &mut out,
            &mut seen,
            TrainRow {
                id: case.id,
                snap: case.snap,
                utterance: case.utterance,
                expect_page: case.expect_page,
                expect_slots: slots,
                notes: case.notes,
            },
        )?;
    }
    Ok(out)
}

fn push(out: &mut Vec<TrainRow>, seen: &mut BTreeSet<String>, row: TrainRow) -> Result<()> {
    refuse_live(&row.snap)?;
    if seen.insert(row.id.clone()) {
        out.push(row);
    }
    Ok(())
}

pub fn write_train_file(cat: &Catalog, path: &Path) -> Result<()> {
    let rows = train_rows(cat)?;
    write_json(path, &rows)
}

pub fn write_holdout_file(holdouts: &[Holdout], path: &Path) -> Result<()> {
    for h in holdouts {
        refuse_live(&h.snap)?;
    }
    write_json(path, holdouts)
}

pub fn write_split(cat: &Catalog, train: &Path, holdout: &Path) -> Result<()> {
    write_train_file(cat, train)?;
    write_holdout_file(&cat.holdouts, holdout)
}

fn write_json<T: Serialize + ?Sized>(path: &Path, value: &T) -> Result<()> {
    if let Some(parent) = path.parent() {
        if !parent.as_os_str().is_empty() {
            fs::create_dir_all(parent)?;
        }
    }
    fs::write(path, serde_json::to_string_pretty(value)? + "\n")?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::text;
    use crate::types::Holdout;
    use std::collections::HashSet;

    #[test]
    fn train_and_holdout_utterances_are_disjoint() {
        let train: Vec<TrainRow> =
            serde_json::from_str(include_str!("../ontology/train.json")).expect("train.json");
        let hold: Vec<Holdout> =
            serde_json::from_str(include_str!("../ontology/holdout.json")).expect("holdout.json");
        assert!(!train.is_empty());
        assert!(!hold.is_empty());
        let hold_utt: HashSet<_> = hold.iter().map(|h| text::norm(&h.utterance)).collect();
        for row in &train {
            let n = text::norm(&row.utterance);
            assert!(!hold_utt.contains(&n), "overlap {}", row.utterance);
        }
    }

    #[test]
    fn train_file_omits_holdout_ids() {
        let cat = Catalog::load();
        let dir = std::env::temp_dir().join(format!("vikett-train-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        let train = dir.join("train.json");
        let holdout = dir.join("holdout.json");
        write_split(&cat, &train, &holdout).unwrap();

        let train_rows: Vec<TrainRow> =
            serde_json::from_str(&fs::read_to_string(&train).unwrap()).unwrap();
        let hold_rows: Vec<Holdout> =
            serde_json::from_str(&fs::read_to_string(&holdout).unwrap()).unwrap();
        assert!(!train_rows.is_empty());
        assert_eq!(hold_rows.len(), cat.holdouts.len());
        let train_ids: HashSet<_> = train_rows.iter().map(|r| r.id.as_str()).collect();
        for h in &hold_rows {
            assert!(
                !train_ids.contains(h.id.as_str()),
                "holdout id {} is in the train file",
                h.id
            );
        }
        let src = include_str!("../scripts/build_laya_set.py");
        assert!(
            !src.contains("holdout.json"),
            "trainer script names the holdout file"
        );
        assert!(!src.contains("torch"));
        assert!(!src.contains("edgejev"));
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn train_rows_refuse_live_snap() {
        let err = refuse_live("live").unwrap_err();
        assert!(err.to_string().contains("live"));
        refuse_live("desk").unwrap();
    }
}
