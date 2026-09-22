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
    ("nautilus", &["org.gnome.nautilus", "nautilus"]),
    ("nemo", &["nemo"]),
    ("thunar", &["thunar"]),
    ("dolphin", &["org.kde.dolphin", "dolphin"]),
    ("yazi", &["yazi"]),
    ("obsidian", &["obsidian"]),
    ("logseq", &["logseq"]),
    ("joplin", &["joplin"]),
    ("zathura", &["org.pwmt.zathura", "zathura"]),
    ("evince", &["org.gnome.evince", "evince"]),
    ("papers", &["org.gnome.papers", "papers"]),
    ("foliate", &["foliate"]),
    ("spotify", &["spotify"]),
    ("ncspot", &["ncspot"]),
    (
        "strawberry",
        &["org.strawberrymusicplayer.strawberry", "strawberry"],
    ),
    ("amberol", &["io.bassi.amberol", "amberol"]),
    ("mpd", &["mpd"]),
    ("mpv", &["mpv"]),
    ("vlc", &["vlc"]),
    ("loupe", &["org.gnome.loupe", "loupe"]),
    ("imv", &["imv"]),
    // `codium` stays on the `code` arm below. Neovide is the nvim GUI class.
    ("nvim", &["neovide", "nvim"]),
    ("helix", &["helix"]),
    ("zed", &["dev.zed.zed", "zed"]),
    ("emacs", &["emacs"]),
    ("libreoffice", &["libreoffice", "soffice"]),
    ("gimp", &["gimp"]),
    ("krita", &["org.kde.krita", "krita"]),
    ("inkscape", &["org.inkscape.inkscape", "inkscape"]),
    ("darktable", &["darktable"]),
    ("signal", &["signal"]),
    ("element", &["element"]),
    // Vesktop's class is `vesktop`. The Discord class is the same app id.
    ("vesktop", &["vesktop", "discord"]),
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
        "nautilus" => Some("nautilus"),
        "nemo" => Some("nemo"),
        "thunar" => Some("thunar"),
        "dolphin" => Some("dolphin"),
        "yazi" => Some("yazi"),
        "spotify" => Some("spotify"),
        "ncspot" => Some("ncspot"),
        "strawberry" => Some("strawberry"),
        "amberol" => Some("amberol"),
        "mpd" => Some("mpd"),
        "mpv" => Some("mpv"),
        "vlc" => Some("vlc"),
        "jellyfin" => Some("jellyfin"),
        "loupe" => Some("loupe"),
        "imv" => Some("imv"),
        "nvim" => Some("nvim"),
        // The app id is helix. `which` name is hx.
        "helix" => Some("hx"),
        "code" => Some("code"),
        "codium" => Some("codium"),
        "zed" => Some("zed"),
        "emacs" => Some("emacs"),
        "libreoffice" => Some("libreoffice"),
        "gimp" => Some("gimp"),
        "krita" => Some("krita"),
        "inkscape" => Some("inkscape"),
        "darktable" => Some("darktable"),
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
        assert_eq!(class_to_app("org.gnome.Nautilus"), "nautilus");
        assert_eq!(class_to_app("nemo"), "nemo");
        assert_eq!(class_to_app("thunar"), "thunar");
        assert_eq!(class_to_app("org.kde.dolphin"), "dolphin");
        assert_eq!(class_to_app("yazi"), "yazi");
        assert_eq!(class_to_app("obsidian"), "obsidian");
        assert_eq!(class_to_app("Logseq"), "logseq");
        assert_eq!(class_to_app("Joplin"), "joplin");
        assert_eq!(class_to_app("org.pwmt.zathura"), "zathura");
        assert_eq!(class_to_app("evince"), "evince");
        assert_eq!(class_to_app("org.gnome.Papers"), "papers");
        assert_eq!(class_to_app("com.github.johnfactotum.Foliate"), "foliate");
        assert_eq!(class_to_app("signal"), "signal");
        assert_eq!(class_to_app("org.signal.Signal"), "signal");
        assert_eq!(class_to_app("Element"), "element");
        assert_eq!(class_to_app("vesktop"), "vesktop");
        assert_eq!(class_to_app("discord"), "vesktop");
        assert_eq!(class_to_app("NotMapped"), "notmapped");
        assert_eq!(bin_for_app("nautilus"), Some("nautilus"));
        assert_eq!(bin_for_app("yazi"), Some("yazi"));
        assert_eq!(bin_for_app("zathura"), None);
        assert_eq!(bin_for_app("obsidian"), None);
        assert_eq!(class_to_app("Spotify"), "spotify");
        assert_eq!(class_to_app("ncspot"), "ncspot");
        assert_eq!(
            class_to_app("org.strawberrymusicplayer.strawberry"),
            "strawberry"
        );
        assert_eq!(class_to_app("io.bassi.Amberol"), "amberol");
        assert_eq!(class_to_app("mpv"), "mpv");
        assert_eq!(class_to_app("vlc"), "vlc");
        assert_eq!(class_to_app("org.gnome.Loupe"), "loupe");
        assert_eq!(class_to_app("imv"), "imv");
        assert_eq!(bin_for_app("spotify"), Some("spotify"));
        assert_eq!(bin_for_app("mpv"), Some("mpv"));
        assert_eq!(bin_for_app("jellyfin"), Some("jellyfin"));
        assert_eq!(bin_for_app("loupe"), Some("loupe"));
        assert_eq!(bin_for_app("imv"), Some("imv"));
        assert_eq!(class_to_app("nvim"), "nvim");
        assert_eq!(class_to_app("neovide"), "nvim");
        assert_eq!(class_to_app("helix"), "helix");
        assert_eq!(class_to_app("dev.zed.Zed"), "zed");
        assert_eq!(class_to_app("Emacs"), "emacs");
        assert_eq!(class_to_app("libreoffice-calc"), "libreoffice");
        assert_eq!(class_to_app("soffice"), "libreoffice");
        assert_eq!(class_to_app("gimp"), "gimp");
        assert_eq!(class_to_app("org.kde.krita"), "krita");
        assert_eq!(class_to_app("org.inkscape.Inkscape"), "inkscape");
        assert_eq!(class_to_app("darktable"), "darktable");
        assert_eq!(bin_for_app("nvim"), Some("nvim"));
        assert_eq!(bin_for_app("helix"), Some("hx"));
        assert_eq!(bin_for_app("code"), Some("code"));
        assert_eq!(bin_for_app("codium"), Some("codium"));
        assert_eq!(bin_for_app("zed"), Some("zed"));
        assert_eq!(bin_for_app("emacs"), Some("emacs"));
        assert_eq!(bin_for_app("libreoffice"), Some("libreoffice"));
        assert_eq!(bin_for_app("gimp"), Some("gimp"));
        assert_eq!(bin_for_app("krita"), Some("krita"));
        assert_eq!(bin_for_app("inkscape"), Some("inkscape"));
        assert_eq!(bin_for_app("darktable"), Some("darktable"));
        assert_eq!(bin_for_app("signal"), None);
        assert_eq!(bin_for_app("vesktop"), None);
        assert_eq!(bin_for_app("foot"), Some("foot"));
        assert_eq!(bin_for_app("wezterm"), Some("wezterm"));
    }

    #[test]
    fn longest_substr_beats_chrome_inside_chromium() {
        assert_eq!(app_for_class("chromium"), Some("chromium"));
        assert_eq!(app_for_class("google-chrome"), Some("chrome"));
    }
}
