use regex::Regex;
use std::sync::LazyLock;

use crate::text::norm;

/// `play 'kind of blue'`. `norm` turns the quotes into spaces, so this sees the raw string.
/// A contraction (`what's`, `Dave's`) is not a quoted span.
static QUOTED_TITLE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?i)\b(play|watch)\b[^'‘’]*['‘][^'‘’]+['’]").expect("quoted title")
});

static REFUSE_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r"(?x)
        \b(email|mail|message)\b.*\b(that|to\ say|saying)\b
        | \bsend\b
        | \btell\b
        | \breply\b
        | \bbuy\b
        | \bclick\b
        | \bpurchase\b
        | \bschedule\b
        | \binvite\b
        | \b\d+(\.\d+)?\s*(percent|%|degrees|celsius|kelvin)\b
        | \bpage\s+\d+\b
        | \bjump\s+to\s+(?:page\s+)?\d+\b
        | \bgo\s+to\s+page\s+\d+\b
        | \b\d+\s+copies\b
        | \b(?:two|three|four|five|six|seven|eight|nine|ten)\s+copies\b
        | \b(?:run|start|launch)\b.*\bimage\b
        | \b(?:podman|docker|distrobox|toolbox)\s+run\b
        | \b\d{3,4}x\d{3,4}\b
        | \b\d+\s*hz\b
        | \bjoin\b.*\b(?:wifi|ssid)\b
        | \bconnect\s+to\b.*\b(?:wifi|ssid)\b
        | \bssid\b
        | \bdns\b
        | \bpair\b
        | \bformat\b.*\b(?:disk|drive)\b
        | \b(?:delete|remove)\b.*\bsnapshots?\b
        | \bset\s+(?:the\s+)?fan\b
        | \bfan\s+(?:speed|percent)\b
        ",
    )
    .expect("refuse regex")
});

/// Phrases that must never walk, even if a nearby page is live.
const ADVERSARIAL: &[&str] = &[
    "make it look nicer",
    "make it cozy",
    "whatever i usually do",
    "read me the whole thread",
    "open a random website",
    "run this curl",
    "run this command",
    "the red one",
    "click the second tab",
    "wake me at seven",
    "email that screenshot",
    "turn off wifi",
    "turn wifi off",
    "disable wifi",
    "wifi off",
    "change dns",
    "change the dns",
    "set the dns",
    "set dns",
    "join the wifi",
    "join wifi",
    "join the ssid",
    "join ssid",
    "connect to wifi",
    "connect to the wifi",
    "change the ssid",
    "change ssid",
    "pair the headphones",
    "pair a device",
    "pair the device",
    "pair bluetooth",
    "format the disk",
    "format this disk",
    "format the drive",
    "wipe the disk",
    "delete the snapshot",
    "delete snapshots",
    "delete a snapshot",
    "remove the snapshot",
    "remove snapshots",
    "set the fan",
    "set fan",
    "fan speed",
    "fan percent",
    "partial upgrade",
    "upgrade the package",
    "upgrade a package",
    "install the package",
    "close everything",
    "set the resolution",
    "change the refresh rate",
    "refresh rate",
    "put it on the tv",
    "put this on the tv",
    "put it on the living room",
    "open this link",
    "search the web",
    "google this",
    "fill the form",
    "fill in the form",
    "reveal the password",
    "show my password",
    "copy the password",
    "copy a password",
    "copy my password",
    "copy password",
    "what's my password",
    "whats my password",
    "what is my password",
    "what's the password",
    "what is the password",
    "reveal my password",
    "reveal password",
    "show the password",
    "run an arbitrary image",
    "run the image",
    "run this image",
    "start streaming",
    "start streaming the screen",
    "write a paragraph",
    "write me a paragraph",
    "delete the file",
    "delete this file",
    "delete these files",
    "permanently delete",
    "remove the file",
    "play the movie",
    "play the film",
    "play the album",
    "watch the film",
    "watch the movie",
    "watch the episode",
    "play this episode",
    "play this title",
    "put on the album",
    "commit",
    "push",
    "rewrite",
    "edit the cell",
    "edit cell",
    "select a layer",
    "select the layer",
    "draw a shape",
    "draw the shape",
];

pub fn refused(utterance: &str) -> Option<&'static str> {
    if free_path(utterance) {
        return Some("refused — no free path");
    }
    let n = norm(utterance);
    // `%` is stripped by norm(); catch "40%" on the raw string.
    let has_percent = utterance.contains('%') && utterance.chars().any(|c| c.is_ascii_digit());
    // `norm` turns `https://…` into tokens. The raw string still has the paste.
    let pasted = utterance.contains("://") || utterance.to_lowercase().contains("www.");
    if pasted {
        return Some("refused — no page for a pasted url");
    }
    // A quoted title is free text. Contractions stay. `norm` has already dropped `'`.
    if utterance.contains('"')
        || utterance.contains('“')
        || utterance.contains('”')
        || QUOTED_TITLE.is_match(utterance)
    {
        return Some("refused — no free title");
    }
    if has_percent || REFUSE_RE.is_match(&n) || named_package(&n) {
        return Some("refused — no page for compose/send/buy/click/free-number");
    }
    for phrase in ADVERSARIAL {
        if crate::text::contains_phrase(&n, phrase) {
            return Some("refused — adversarial utterance, silence");
        }
    }
    None
}

/// A full upgrade may only use these words. Any other token is a package name,
/// including one that follows "the system" or "everything".
const UPGRADE_WORDS: &[&str] = &[
    "upgrade",
    "update",
    "install",
    "the",
    "system",
    "everything",
    "all",
    "now",
    "status",
    "full",
];

fn named_package(n: &str) -> bool {
    let words: Vec<&str> = n.split_whitespace().collect();
    if !words
        .iter()
        .any(|w| matches!(*w, "upgrade" | "install" | "update"))
    {
        return false;
    }
    words.iter().any(|w| !UPGRADE_WORDS.contains(w))
}

/// `..`, `~/`, a backslash, or an absolute path. Enum dirs never look like this.
fn free_path(utterance: &str) -> bool {
    if utterance.contains("..") || utterance.contains("~/") || utterance.contains('\\') {
        return true;
    }
    let mut prev_space = true;
    for ch in utterance.chars() {
        if prev_space && ch == '/' {
            return true;
        }
        prev_space = ch.is_whitespace();
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn catches_send_and_percent() {
        assert!(refused("email Dave that I'll be late").is_some());
        assert!(refused("set volume to 37 percent").is_some());
        assert!(refused("set the volume to 40%").is_some());
        assert!(refused("buy the thing in that tab").is_some());
        assert!(refused("mute").is_none());
        assert!(refused("play the movie").is_some());
        assert!(refused("play the film").is_some());
        assert!(refused("play the album").is_some());
        assert!(refused("watch the episode").is_some());
        assert!(refused("play \"kind of blue\"").is_some());
        assert!(refused("play 'kind of blue'").is_some());
        assert!(refused("watch 'the bear'").is_some());
        assert!(refused("what's playing").is_none());
        assert!(refused("pause the show").is_none());
        assert!(refused("play the jazz playlist").is_none());
        assert!(refused("living room watch").is_none());
        assert!(refused("close everything").is_some());
        assert!(refused("put it on the TV").is_some());
        assert!(refused("open this link").is_some());
        assert!(refused("see https://example.com").is_some());
        assert!(refused("open www.example.com").is_some());
        assert!(refused("run this command").is_some());
        assert!(refused("page 12").is_some());
        assert!(refused("tell signal I'll be late").is_some());
        assert!(refused("reply to that mail").is_some());
        assert!(refused("message that I'm on my way").is_some());
        assert!(refused("browser forward").is_none());
        assert!(refused("forward the mail").is_none());
        assert!(refused("jump to 4").is_some());
        assert!(refused("go to page 3").is_some());
        assert!(refused("write a paragraph").is_some());
        assert!(refused("delete the file").is_some());
        assert!(refused("open ../secrets").is_some());
        assert!(refused("open /etc/passwd").is_some());
        assert!(refused("open ~/notes").is_some());
        assert!(refused("next page").is_none());
        assert!(refused("back in files").is_none());
        assert!(refused("open the downloads folder").is_none());
        assert!(refused("pixel click").is_some());
        assert!(refused("click at 40 12").is_some());
        assert!(refused("set the resolution").is_some());
        assert!(refused("1920x1080").is_some());
        assert!(refused("set 144hz").is_some());
        assert!(refused("suspend the computer").is_none());
        assert!(refused("shut down").is_none());
        assert!(refused("hdmi brighter").is_none());
        assert!(refused("commit this").is_some());
        assert!(refused("git commit").is_some());
        assert!(refused("push to origin").is_some());
        assert!(refused("rewrite the file").is_some());
        assert!(refused("edit the cell").is_some());
        assert!(refused("edit cell b2").is_some());
        assert!(refused("select a layer").is_some());
        assert!(refused("select the layer").is_some());
        assert!(refused("draw a shape").is_some());
        assert!(refused("copy the password").is_some());
        assert!(refused("copy a password").is_some());
        assert!(refused("start streaming").is_some());
        assert!(refused("start streaming the screen").is_some());
        assert!(refused("start recording").is_none());
        assert!(refused("start screen recording").is_none());
        assert!(refused("what's on the clipboard").is_none());
        assert!(refused("clear the clipboard").is_none());
        assert!(refused("next editor tab").is_none());
        assert!(refused("save in code").is_none());
        assert!(refused("next libreoffice sheet").is_none());
        assert!(refused("undo in gimp").is_none());
        assert!(refused("join the wifi").is_some());
        assert!(refused("join the office wifi").is_some());
        assert!(refused("connect to the wifi").is_some());
        assert!(refused("change the ssid").is_some());
        assert!(refused("set the dns").is_some());
        assert!(refused("disable wifi").is_some());
        assert!(refused("pair a device").is_some());
        assert!(refused("format the disk").is_some());
        assert!(refused("format my drive").is_some());
        assert!(refused("delete the snapshot").is_some());
        assert!(refused("delete snapshots").is_some());
        assert!(refused("set the fan").is_some());
        assert!(refused("set fan speed").is_some());
        assert!(refused("set the fan to 40%").is_some());
        assert!(refused("upgrade firefox").is_some());
        assert!(refused("update firefox").is_some());
        assert!(refused("install firefox").is_some());
        assert!(refused("partial upgrade").is_some());
        assert!(refused("apt install firefox").is_some());
        assert!(refused("upgrade the system").is_none());
        assert!(refused("update the system").is_none());
        assert!(refused("update status").is_none());
        assert!(refused("full system upgrade").is_none());
        assert!(refused("upgrade the system firefox").is_some());
        assert!(refused("update the system vim").is_some());
        assert!(refused("upgrade everything firefox").is_some());
        assert!(refused("partial system upgrade").is_some());
        assert!(refused("upgrade the firefox package").is_some());
        assert!(refused("how's the battery").is_none());
        assert!(refused("connect the home vpn").is_none());
        assert!(refused("connect the headphones").is_none());
        assert!(refused("disconnect the headphones").is_none());
        assert!(refused("take a snapshot").is_none());
        assert!(refused("power saver profile").is_none());
        assert!(refused("how's the disk").is_none());
        assert!(refused("how many updates").is_none());
        assert!(refused("screenshot").is_none());
        assert!(refused("lock the screen").is_none());
        assert!(refused("screens off").is_none());
        assert!(refused("what's my password").is_some());
        assert!(refused("whats my password").is_some());
        assert!(refused("what is my password").is_some());
        assert!(refused("reveal my password").is_some());
        assert!(refused("reveal the password").is_some());
        assert!(refused("copy password").is_some());
        assert!(refused("copy my password").is_some());
        assert!(refused("show the password").is_some());
        assert!(refused("is the vault unlocked").is_none());
        assert!(refused("run the ubuntu image").is_some());
        assert!(refused("run an arbitrary image").is_some());
        assert!(refused("podman run ubuntu").is_some());
        assert!(refused("docker run ubuntu").is_some());
        assert!(refused("print 3 copies").is_some());
        assert!(refused("scan 2 copies").is_some());
        assert!(refused("print two copies").is_some());
        assert!(refused("buy celeste").is_some());
        assert!(refused("start the arch box").is_none());
        assert!(refused("play celeste").is_none());
        assert!(refused("start obs recording").is_none());
        assert!(refused("start streaming").is_some());
        assert!(refused("mouse battery").is_none());
        assert!(refused("upgrade the system").is_none());
    }
}
