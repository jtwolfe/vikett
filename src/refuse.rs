use regex::Regex;
use std::sync::LazyLock;

use crate::text::norm;

static REFUSE_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r"(?x)
        \b(email|mail|message)\b.*\b(that|to\ say|saying)\b
        | \bsend\b
        | \bbuy\b
        | \bclick\b
        | \bpurchase\b
        | \bschedule\b
        | \binvite\b
        | \b\d+(\.\d+)?\s*(percent|%|degrees|celsius|kelvin)\b
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
];

pub fn refused(utterance: &str) -> Option<&'static str> {
    let n = norm(utterance);
    // `%` is stripped by norm(); catch "40%" on the raw string.
    let has_percent = utterance.contains('%') && utterance.chars().any(|c| c.is_ascii_digit());
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
    }
}
