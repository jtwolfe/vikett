use std::collections::HashSet;
use std::sync::LazyLock;

static STOP: LazyLock<HashSet<&'static str>> = LazyLock::new(|| {
    HashSet::from([
        "a", "the", "to", "on", "in", "it", "me", "i", "am", "is", "are", "any", "of", "and",
        "please", "up", "my",
    ])
});

pub fn norm(s: &str) -> String {
    let lowered = s.to_lowercase().replace(['’', '\''], " ");
    let mut out = String::with_capacity(lowered.len());
    let mut prev_space = false;
    for ch in lowered.chars() {
        if ch.is_ascii_alphanumeric() || ch == '+' || ch == '\'' {
            out.push(ch);
            prev_space = false;
        } else if !prev_space {
            out.push(' ');
            prev_space = true;
        }
    }
    out.split_whitespace().collect::<Vec<_>>().join(" ")
}

pub fn tokens(s: &str) -> Vec<String> {
    norm(s)
        .split(' ')
        .filter(|t| !t.is_empty() && !STOP.contains(t))
        .map(|t| t.to_string())
        .collect()
}

pub fn contains_phrase(hay: &str, needle: &str) -> bool {
    let h = format!(" {} ", norm(hay));
    let n = format!(" {} ", norm(needle));
    if n.trim().is_empty() {
        return false;
    }
    h.contains(&n)
}

/// Split compound speech on `and then` / `and`. A single clause is returned as-is.
pub fn split_compound(utterance: &str) -> Vec<String> {
    let n = norm(utterance);
    let mut parts: Vec<String> = Vec::new();
    let mut rest = n.as_str();
    while let Some(idx) = find_split(rest) {
        let (left, right) = rest.split_at(idx);
        let left = left.trim();
        if !left.is_empty() {
            parts.push(left.to_string());
        }
        rest = skip_splitter(right);
    }
    let last = rest.trim();
    if !last.is_empty() {
        parts.push(last.to_string());
    }
    if parts.len() > 1 {
        parts
    } else {
        vec![utterance.to_string()]
    }
}

fn find_split(s: &str) -> Option<usize> {
    let and_then = s.find(" and then ");
    let and = s.find(" and ");
    match (and_then, and) {
        (Some(a), Some(b)) => Some(a.min(b)),
        (Some(a), None) => Some(a),
        (None, Some(b)) => Some(b),
        (None, None) => None,
    }
}

fn skip_splitter(s: &str) -> &str {
    if let Some(rest) = s.strip_prefix(" and then ") {
        rest
    } else if let Some(rest) = s.strip_prefix(" and ") {
        rest
    } else {
        s.trim_start()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn splits_compound() {
        assert_eq!(
            split_compound("switch to jellyfin and fullscreen"),
            vec!["switch to jellyfin", "fullscreen"]
        );
        assert_eq!(
            split_compound("switch to jellyfin and then fullscreen"),
            vec!["switch to jellyfin", "fullscreen"]
        );
    }

    #[test]
    fn leaves_single_clause() {
        assert_eq!(split_compound("mute"), vec!["mute"]);
    }

    #[test]
    fn phrase_match_is_bounded() {
        assert!(contains_phrase("increase the volume a bit", "a bit"));
        assert!(!contains_phrase("email dave", "mail"));
    }
}
