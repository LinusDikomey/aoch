pub mod grid;

pub use color_format::*;
pub use itertools::Itertools;
pub use std::collections::{BTreeSet, HashMap, HashSet};
pub use vecm::*;

pub use grid::{Grid, Side};

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

pub fn transitive_closure<I: IntoIterator<Item = T>, T, F: FnMut(&T, &T) -> bool>(
    it: I,
    mut relation: F,
) -> Vec<Vec<T>> {
    let mut sets: Vec<Vec<T>> = Vec::new();
    for item in it {
        let mut found_in = Vec::new();
        for (i, set) in sets.iter().enumerate() {
            if set.iter().any(|other| relation(&item, other)) {
                found_in.push(i);
            }
        }
        match found_in.split_first() {
            None => sets.push(vec![item]),
            Some((&first, others)) => {
                sets[first].push(item);
                for &other in others {
                    let mut other = sets.remove(other);
                    sets[first].append(&mut other);
                }
            }
        }
    }
    sets
}
