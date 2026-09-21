//! Mods and key names for a dry-run chord.
//!
//! Empty `mods` is a chord only when `key` is `F11`. `CTRL + SHIFT` (spaces
//! around `+`) is the string we print, not a mask confirmed on this compositor.

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Chord {
    pub mods: String,
    pub key: String,
}

/// Not a bool. `ForceDead` wins over a builtin; `Use` is armed only when [`chord_ok`].
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ChordOverride {
    ForceDead,
    Use { mods: String, key: String },
}

pub fn chord_ok(mods: &str, key: &str) -> bool {
    !key.is_empty() && (!mods.is_empty() || key == "F11")
}

impl ChordOverride {
    pub fn chord(&self) -> Option<Chord> {
        match self {
            Self::ForceDead => None,
            Self::Use { mods, key } if chord_ok(mods, key) => Some(Chord {
                mods: mods.clone(),
                key: key.clone(),
            }),
            Self::Use { .. } => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_mods_only_f11() {
        assert!(chord_ok("", "F11"));
        assert!(!chord_ok("", "T"));
        assert!(!chord_ok("", ""));
        assert!(!chord_ok("", "f11"));
        assert!(chord_ok("CTRL", "T"));
        assert!(chord_ok("CTRL + SHIFT", "P"));
    }

    #[test]
    fn overlay_is_not_a_bool() {
        assert!(ChordOverride::ForceDead.chord().is_none());
        assert!(ChordOverride::Use {
            mods: String::new(),
            key: "T".into(),
        }
        .chord()
        .is_none());
        let f11 = ChordOverride::Use {
            mods: String::new(),
            key: "F11".into(),
        }
        .chord()
        .expect("F11");
        assert_eq!(f11.mods, "");
        assert_eq!(f11.key, "F11");
    }
}
