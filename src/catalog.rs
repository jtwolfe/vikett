use crate::ontology::extras;
use crate::types::{Golden, ModuleDef, Page, Phrase, SlotValue, Snap};

pub struct Catalog {
    pub pages: Vec<Page>,
    pub modules: Vec<ModuleDef>,
    pub snaps: Vec<Snap>,
    pub goldens: Vec<Golden>,
    pub phrases: Vec<Phrase>,
}

impl Catalog {
    pub fn load() -> Self {
        let mut pages: Vec<Page> =
            serde_json::from_str(include_str!("../ontology/pages.json")).expect("pages.json");
        let mut modules: Vec<ModuleDef> =
            serde_json::from_str(include_str!("../ontology/modules.json")).expect("modules.json");
        let mut snaps: Vec<Snap> =
            serde_json::from_str(include_str!("../ontology/snaps.json")).expect("snaps.json");
        let mut goldens: Vec<Golden> =
            serde_json::from_str(include_str!("../ontology/goldens.json")).expect("goldens.json");
        let phrases: Vec<Phrase> =
            serde_json::from_str(include_str!("../ontology/phrases.json")).unwrap_or_default();

        extras::patch_pages(&mut pages);
        extras::patch_modules(&mut modules);
        extras::patch_snaps(&mut snaps);
        pages.extend(extras::pages());
        modules.extend(extras::modules());
        goldens.extend(extras::goldens());

        Self {
            pages,
            modules,
            snaps,
            goldens,
            phrases,
        }
    }

    pub fn page(&self, id: &str) -> Option<&Page> {
        self.pages.iter().find(|p| p.id == id)
    }

    pub fn snap(&self, id: &str) -> Option<&Snap> {
        self.snaps.iter().find(|s| s.id == id)
    }

    pub fn module(&self, id: &str) -> Option<&ModuleDef> {
        self.modules.iter().find(|m| m.id == id)
    }
}

pub fn slot_value(id: &str, label: &str, aliases: &[&str]) -> SlotValue {
    SlotValue {
        id: id.into(),
        label: label.into(),
        aliases: aliases.iter().map(|s| (*s).to_string()).collect(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn loads_v0_pages() {
        let cat = Catalog::load();
        assert!(cat.pages.len() >= 50);
        assert!(cat.page("wm.focus").is_some());
        assert!(cat.snap("desk").is_some());
        assert!(cat.goldens.iter().any(|g| g.id == "vol-little"));
    }
}
