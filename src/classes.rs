//! Window-class → app id. Longest matching substring wins (`google-chrome` over `chrome`).

pub struct ClassBind {
    pub app: &'static str,
    pub family: &'static str,
    pub class_substr: &'static [&'static str],
    pub bin: &'static str,
}

static BINDS: &[ClassBind] = &[
    ClassBind {
        app: "zen",
        family: "browser",
        class_substr: &["zen-browser", "zen"],
        bin: "zen",
    },
    ClassBind {
        app: "firefox",
        family: "browser",
        class_substr: &["firefox"],
        bin: "firefox",
    },
    ClassBind {
        app: "chrome",
        family: "browser",
        class_substr: &["google-chrome", "chrome"],
        bin: "google-chrome",
    },
    ClassBind {
        app: "chromium",
        family: "browser",
        class_substr: &["chromium"],
        bin: "chromium",
    },
    ClassBind {
        app: "brave",
        family: "browser",
        class_substr: &["brave"],
        bin: "brave",
    },
    ClassBind {
        app: "foot",
        family: "term",
        class_substr: &["foot"],
        bin: "foot",
    },
    ClassBind {
        app: "kitty",
        family: "term",
        class_substr: &["kitty"],
        bin: "kitty",
    },
    ClassBind {
        app: "jellyfin",
        family: "media",
        class_substr: &["jellyfin"],
        bin: "jellyfin",
    },
    ClassBind {
        app: "code",
        family: "edit",
        class_substr: &["codium", "code"],
        bin: "code",
    },
    ClassBind {
        app: "thunderbird",
        family: "mail",
        class_substr: &["thunderbird"],
        bin: "thunderbird",
    },
];

pub fn bind_for_app(app: &str) -> Option<&'static ClassBind> {
    BINDS.iter().find(|b| b.app == app)
}

pub fn app_for_class(class: &str) -> Option<&'static str> {
    let c = class.to_lowercase();
    let mut best: Option<(&'static str, usize)> = None;
    for bind in BINDS {
        for sub in bind.class_substr {
            if sub.is_empty() || !c.contains(sub) {
                continue;
            }
            let longer = best.is_none_or(|(_, n)| sub.len() > n);
            if longer {
                best = Some((bind.app, sub.len()));
            }
        }
    }
    best.map(|(app, _)| app)
}

/// App id, or the lowercased class when nothing in the table matches.
pub fn class_to_app(class: &str) -> String {
    match app_for_class(class) {
        Some(app) => app.to_string(),
        None => class.to_lowercase(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn each_arm() {
        assert_eq!(class_to_app("zen"), "zen");
        assert_eq!(class_to_app("zen-browser"), "zen");
        assert_eq!(class_to_app("foot"), "foot");
        assert_eq!(class_to_app("firefox"), "firefox");
        assert_eq!(class_to_app("kitty"), "kitty");
        assert_eq!(class_to_app("jellyfin"), "jellyfin");
        assert_eq!(class_to_app("code"), "code");
        assert_eq!(class_to_app("codium"), "code");
        assert_eq!(class_to_app("VSCodium"), "code");
        assert_eq!(class_to_app("thunderbird"), "thunderbird");
        assert_eq!(class_to_app("google-chrome"), "chrome");
        assert_eq!(class_to_app("Google-Chrome"), "chrome");
        assert_eq!(class_to_app("chromium"), "chromium");
        assert_eq!(class_to_app("brave-browser"), "brave");
        assert_eq!(class_to_app("NotMapped"), "notmapped");
    }

    #[test]
    fn longest_substr_beats_chrome_inside_chromium() {
        assert_eq!(app_for_class("chromium"), Some("chromium"));
        assert_eq!(app_for_class("google-chrome"), Some("chrome"));
        assert_eq!(bind_for_app("zen").map(|b| b.family), Some("browser"));
        assert_eq!(bind_for_app("code").map(|b| b.bin), Some("code"));
        assert!(bind_for_app("code")
            .unwrap()
            .class_substr
            .contains(&"codium"));
    }
}
