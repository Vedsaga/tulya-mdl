//! MDL v1 – Baseline descent operator over a full candidate set.
//!
//! This version generates all possible common blocks and uses a greedy
//! descent step to find a local minimum of the MDL energy.

use super::types::{Block, MatchTable, block_overlaps, lambda, KAPPA};
use colored::Colorize;

/// An admissible family of non-overlapping blocks.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct BlockFamily {
    pub blocks: Vec<Block>,
}

use super::diagnostics::{EnergyBreakdown, BlockInsight};

impl BlockFamily {
    pub fn is_admissible(&self) -> bool {
        for i in 0..self.blocks.len() {
            for j in i + 1..self.blocks.len() {
                if block_overlaps(&self.blocks[i], &self.blocks[j]) {
                    return false;
                }
            }
        }
        true
    }

    pub fn energy(&self, s: &[u8], t: &[u8]) -> usize {
        let mut covered_s = vec![false; s.len()];
        let mut covered_t = vec![false; t.len()];

        for b in &self.blocks {
            for k in 0..b.len {
                covered_s[b.i + k] = true;
                covered_t[b.j + k] = true;
            }
        }

        let u_x = covered_s.iter().filter(|&&x| !x).count();
        let u_y = covered_t.iter().filter(|&&x| !x).count();

        let mut l_m = lambda(self.blocks.len());
        for b in &self.blocks {
            l_m += lambda(b.i + 1) + lambda(b.j + 1) + lambda(b.len);
        }

        l_m + KAPPA * u_x + KAPPA * u_y
    }

    fn canonical_key(&self) -> Vec<Block> {
        let mut sorted = self.blocks.clone();
        sorted.sort();
        sorted
    }

    fn all_candidate_blocks(s: &[u8], t: &[u8], mt: &MatchTable) -> Vec<Block> {
        let mut cands = Vec::new();
        for i in 0..s.len() {
            for j in 0..t.len() {
                let max_l = mt.max_len(i, j);
                for len in 1..=max_l {
                    cands.push(Block { i, j, len });
                }
            }
        }
        cands
    }

    pub fn neighbors(&self, s: &[u8], t: &[u8], mt: &MatchTable) -> Vec<Self> {
        let mut neighbors = Vec::new();
        let candidates = Self::all_candidate_blocks(s, t, mt);

        for cand in candidates {
            if self.blocks.contains(&cand) {
                // Neighbors via deletion
                let mut next = self.clone();
                next.blocks.retain(|&b| b != cand);
                neighbors.push(next);
            } else {
                // Neighbors via insertion
                let mut next = self.clone();
                next.blocks.push(cand);
                if next.is_admissible() {
                    neighbors.push(next);
                }
            }
        }
        neighbors
    }

    pub fn descent_step(&mut self, s: &[u8], t: &[u8], mt: &MatchTable) -> bool {
        let current_e = self.energy(s, t);
        let mut best = self.clone();
        let mut best_e = current_e;
        let mut best_key = self.canonical_key();

        for n in self.neighbors(s, t, mt) {
            let e = n.energy(s, t);
            let k = n.canonical_key();
            if e < best_e || (e == best_e && k < best_key) {
                best = n;
                best_e = e;
                best_key = k;
            }
        }

        if best_e < current_e || (best_e == current_e && best_key < self.canonical_key()) {
            *self = best;
            true
        } else {
            false
        }
    }

    pub fn run_to_fixed_point(&mut self, s: &[u8], t: &[u8], mt: &MatchTable) {
        while self.descent_step(s, t, mt) {}
    }

    pub fn analyze(&self, s: &[u8], t: &[u8]) -> EnergyBreakdown {
        EnergyBreakdown::calculate(&self.blocks, s, t)
    }

    pub fn block_insights(&self, s: &[u8], t: &[u8]) -> Vec<BlockInsight> {
        BlockInsight::calculate_all(&self.blocks, s, t)
    }

    pub fn descent_step_with_log(&mut self, s: &[u8], t: &[u8], mt: &MatchTable, step: usize) -> bool {
        let current_e = self.energy(s, t);
        let mut best = self.clone();
        let mut best_e = current_e;
        let mut best_key = self.canonical_key();
        let mut last_move = None;

        let candidates = Self::all_candidate_blocks(s, t, mt);

        for cand in candidates {
            if self.blocks.contains(&cand) {
                // Neighbors via deletion
                let mut next = self.clone();
                next.blocks.retain(|&b| b != cand);
                let e = next.energy(s, t);
                let k = next.canonical_key();
                if e < best_e || (e == best_e && k < best_key) {
                    best = next;
                    best_e = e;
                    best_key = k;
                    last_move = Some(format!("{} (i={}, j={}, len={})", "DELETE".red(), cand.i, cand.j, cand.len));
                }
            } else {
                // Neighbors via insertion
                let mut next = self.clone();
                next.blocks.push(cand);
                if next.is_admissible() {
                    let e = next.energy(s, t);
                    let k = next.canonical_key();
                    if e < best_e || (e == best_e && k < best_key) {
                        best = next;
                        best_e = e;
                        best_key = k;
                        last_move = Some(format!("{} (i={}, j={}, len={})", "ADD".green(), cand.i, cand.j, cand.len));
                    }
                }
            }
        }

        if best_e < current_e || (best_e == current_e && best_key < self.canonical_key()) {
            println!("[Step {}]", step);
            println!("Energy: {} → {}", current_e, best_e);
            if let Some(m) = last_move {
                println!("* {}", m);
            }
            println!();
            *self = best;
            true
        } else {
            false
        }
    }

    pub fn run_with_diagnostics(&mut self, s: &[u8], t: &[u8], mt: &MatchTable, debug: bool) {
        let mut step = 1;
        if debug {
            while self.descent_step_with_log(s, t, mt, step) {
                step += 1;
            }
        } else {
            self.run_to_fixed_point(s, t, mt);
        }
    }
}
