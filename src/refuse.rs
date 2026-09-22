use regex::Regex;
use std::sync::LazyLock;

use crate::text::norm;

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
    "change dns",
    "pair the headphones",
    "suspend the computer",
    "shut down",
    "close everything",
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
    "write a paragraph",
    "write me a paragraph",
    "delete the file",
    "delete this file",
    "delete these files",
    "permanently delete",
    "remove the file",
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
    if has_percent || REFUSE_RE.is_match(&n) {
        return Some("refused — no page for compose/send/buy/click/free-number");
    }
    for phrase in ADVERSARIAL {
        if crate::text::contains_phrase(&n, phrase) {
            return Some("refused — adversarial utterance, silence");
        }
    }
    None
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
    }
}
