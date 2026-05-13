//! Built-in language specs.
//!
//! Each submodule declares one or more `pub const LanguageSpec`
//! values. The registry in [`super::registry`] aggregates them into a
//! single lookup slice.

pub mod c_family;
pub mod erlang;
pub mod hash_line;
pub mod haskell;
pub mod julia;
pub mod lisp;
pub mod lua;
pub mod markup;
pub mod misc;
pub mod ml;
pub mod sql;

#[cfg(test)]
pub(crate) fn assert_strip(spec: &super::spec::LanguageSpec, input: &str, expected: &str) {
    assert_eq!(super::stripper::strip(input, spec), expected);
}
