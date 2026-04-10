//! Shared types and utilities for MDL block-packing algorithms.
use crate::algorithms::types::MappingResult;
use std::collections::HashMap;

pub const KAPPA: usize = 8;

/// A candidate block: exact diagonal match between source[i..i+len] and target[j..j+len].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Block {
    pub i: usize, // source start (0-indexed)
    pub j: usize, // target start (0-indexed)
    pub len: usize,
}

/// Precomputed longest common prefix table for fast candidate generation.
pub struct MatchTable {
    lcp: Vec<Vec<usize>>,
}

impl MatchTable {
    /// Build the LCP table for strings `s` and `t`.
    pub fn new(s: &[u8], t: &[u8]) -> Self {
        let n = s.len();
        let m = t.len();
        let mut lcp = vec![vec![0; m + 1]; n + 1];
        for i in (0..n).rev() {
            for j in (0..m).rev() {
                if s[i] == t[j] {
                    lcp[i][j] = 1 + lcp[i + 1][j + 1];
                }
            }
        }
        Self { lcp }
    }

    /// Length of the longest exact match starting at (i, j).
    #[inline(always)]
    pub fn max_len(&self, i: usize, j: usize) -> usize {
        self.lcp[i][j]
    }
}

/// MDL length function (log2-like).
#[inline(always)]
pub fn lambda(x: usize) -> usize {
    if x <= 1 {
        1
    } else {
        (usize::BITS - x.leading_zeros()) as usize
    }
}

/// Check if two blocks overlap in either source or target space.
#[inline(always)]
pub fn block_overlaps(a: &Block, b: &Block) -> bool {
    // Range intersection test: [a.i, a.i+a.len) overlaps [b.i, b.i+b.len)
    // AND [a.j, a.j+a.len) overlaps [b.j, b.j+b.len)
    a.i < b.i + b.len && b.i < a.i + a.len && a.j < b.j + b.len && b.j < a.j + a.len
}

/// Convert MDL blocks to a standard MappingResult for unified testing/visualization.
pub fn convert_mdl_to_mapping(
    source_len: usize,
    target_len: usize,
    blocks: &[Block],
) -> MappingResult {
    let mut s2t: HashMap<usize, Vec<usize>> = HashMap::new();
    let mut t2s: HashMap<usize, Vec<usize>> = HashMap::new();

    for b in blocks {
        for k in 0..b.len {
            let si = b.i + k;
            let tj = b.j + k;
            s2t.entry(si).or_default().push(tj);
            t2s.entry(tj).or_default().push(si);
        }
    }

    for v in s2t.values_mut() {
        v.sort();
    }
    for v in t2s.values_mut() {
        v.sort();
    }

    let unmapped_source = (0..source_len).filter(|i| !s2t.contains_key(i)).collect();
    let unmapped_target = (0..target_len).filter(|j| !t2s.contains_key(j)).collect();

    MappingResult {
        source_to_target: s2t,
        target_to_source: t2s,
        unmapped_source,
        unmapped_target,
    }
}
