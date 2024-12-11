pub mod grid;

pub use color_format::*;
pub use itertools::Itertools;
pub use std::collections::{BTreeSet, HashMap, HashSet};
pub use vecm::*;

pub use grid::Grid;

pub fn int(s: &str) -> i64 {
    s.trim().parse().expect("failed to parse as int")
}

pub fn ints(s: &str) -> Vec<i64> {
    s.trim()
        .split(" ")
        .filter(|s| !s.is_empty())
        .map(int)
        .collect()
}
