//! Built-in language specs.
//!
//! Each submodule declares one or more `pub const LanguageSpec`
//! values. The registry in [`super::registry`] aggregates them into a
//! single lookup slice.

#[cfg(test)]
#[allow(dead_code)]
pub(crate) fn assert_strip(spec: &super::spec::LanguageSpec, input: &str, expected: &str) {
    assert_eq!(super::stripper::strip(input, spec), expected);
}
