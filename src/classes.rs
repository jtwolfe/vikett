//! Window-class → app id. Longest matching substring wins (`google-chrome` over `chrome`).

const ARMS: &[(&str, &[&str])] = &[
    ("zen", &["zen-browser", "zen"]),
    ("firefox", &["firefox"]),
    ("chrome", &["google-chrome", "chrome"]),
    ("chromium", &["chromium"]),
    ("brave", &["brave"]),
    ("foot", &["foot"]),
    ("kitty", &["kitty"]),
    ("ghostty", &["com.mitchellh.ghostty", "ghostty"]),
    ("alacritty", &["alacritty"]),
    ("wezterm", &["org.wezfurlong.wezterm", "wezterm"]),
    ("jellyfin", &["jellyfin"]),
    ("code", &["codium", "code"]),
    ("thunderbird", &["thunderbird"]),
];

pub fn app_for_class(class: &str) -> Option<&'static str> {
    let c = class.to_lowercase();
    let mut best: Option<(&'static str, usize)> = None;
    for (app, substrs) in ARMS {
        for sub in *substrs {
            if sub.is_empty() || !c.contains(sub) {
                continue;
            }
            let longer = best.is_none_or(|(_, n)| sub.len() > n);
            if longer {
                best = Some((*app, sub.len()));
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

/// `which` name. Unmapped `browser.focus` may exec only when `snap.bins` contains it.
pub fn bin_for_app(app: &str) -> Option<&'static str> {
    match app {
        // This box: `which zen-browser`, class still `zen`.
        "zen" => Some("zen-browser"),
        "firefox" => Some("firefox"),
        "chrome" => Some("google-chrome"),
        "chromium" => Some("chromium"),
        "brave" => Some("brave"),
        "foot" => Some("foot"),
        "kitty" => Some("kitty"),
        "ghostty" => Some("ghostty"),
        "alacritty" => Some("alacritty"),
        "wezterm" => Some("wezterm"),
        _ => None,
    }
}

/// Class equality through `app_for_class` only. A title that mentions Firefox is not Firefox.
pub fn client_for_app<'a>(
    snap: &'a crate::types::Snap,
    app: &str,
) -> Option<&'a crate::types::Client> {
    let mut found = None;
    for c in &snap.clients {
        if app_for_class(&c.class) != Some(app) {
            continue;
        }
        if c.focused {
            return Some(c);
        }
        if found.is_none() {
            found = Some(c);
        }
    }
    found
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
        assert_eq!(class_to_app("ghostty"), "ghostty");
        assert_eq!(class_to_app("com.mitchellh.ghostty"), "ghostty");
        assert_eq!(class_to_app("Alacritty"), "alacritty");
        assert_eq!(class_to_app("org.wezfurlong.wezterm"), "wezterm");
        assert_eq!(class_to_app("NotMapped"), "notmapped");
        assert_eq!(bin_for_app("foot"), Some("foot"));
        assert_eq!(bin_for_app("wezterm"), Some("wezterm"));
    }

    #[test]
    fn longest_substr_beats_chrome_inside_chromium() {
        assert_eq!(app_for_class("chromium"), Some("chromium"));
        assert_eq!(app_for_class("google-chrome"), Some("chrome"));
    }
}
