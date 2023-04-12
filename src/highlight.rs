use aho_corasick::AhoCorasick;
use once_cell::sync::Lazy;

// Highlighting text, e.g. the search terms in search results
// The following characters are used as markers to encode highlighted text:
pub const HIGHLIGHT_BEGIN: &str = "˹";
pub const HIGHLIGHT_END: &str = "˺";

const HIGHLIGHT_SEARCH_PATTERNS: &[&str; 2] = &[HIGHLIGHT_BEGIN, HIGHLIGHT_END];
const HIGHILIGHT_REPLACE_PATTERNS: &[&str; 2] = &["<mark>", "</mark>"];
// AhoCorasick is an automaton for searching multiple strings in linear time.
// We'll reuse the same instance for all highlighting.
static AC: Lazy<AhoCorasick> = Lazy::new(|| AhoCorasick::new(HIGHLIGHT_SEARCH_PATTERNS));

// function to replace highlight markers with HTML tags
pub fn highlight_html(text: &str) -> String {
    AC.replace_all(text, HIGHILIGHT_REPLACE_PATTERNS)
}
