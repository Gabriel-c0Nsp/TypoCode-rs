//! Core comment-stripping engine.
//!
//! The implementation lands in the next commit; this stub keeps the
//! module tree compiling while the registry and types take shape.

use super::spec::LanguageSpec;

/// Returns `raw` with comments removed according to `spec`. The stub
/// is a pass-through until the state machine arrives.
pub fn strip(raw: &str, _spec: &LanguageSpec) -> String {
    raw.to_string()
}
