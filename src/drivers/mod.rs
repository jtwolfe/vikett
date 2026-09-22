//! Family drivers. v0 pages stay in the prune/walk/ask matches.

pub mod browser;

pub fn is_family(module: &str) -> bool {
    module == "browser"
}
