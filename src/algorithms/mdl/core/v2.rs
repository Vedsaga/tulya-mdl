//! MDL v2 – Fast descent operator using bit-parallel energy updates.
//!
//! This version maintains covered bitsets and incrementally updates the
//! MDL energy, significantly speeding up the neighbor evaluation phase.

use bitvec::prelude::*;
use super::types::{Block, MatchTable, block_overlaps, lambda, KAPPA};

pub struct FastBlockFamily {
    pub blocks: Vec<Block>,
    covered_s: BitVec,
    covered_t: BitVec,
    energy: usize,
    s_len: usize,
    t_len: usize,
}

impl FastBlockFamily {
    pub fn new(s_len: usize, t_len: usize) -> Self {
        Self {
            blocks: Vec::new(),
            covered_s: bitvec![0; s_len],
            covered_t: bitvec![0; t_len],
            energy: KAPPA * (s_len + t_len) + 1,
            s_len,
            t_len,
        }
    }

    #[allow(dead_code)]
    pub fn energy(&self, _s: &[u8], _t: &[u8]) -> usize {
        self.energy
    }

    #[inline(always)]
    
    #[allow(dead_code)]
    fn compute_energy(&self) -> usize {
        let u_s = self.covered_s.count_zeros();
        let u_t = self.covered_t.count_zeros();
        let mut l_m = lambda(self.blocks.len());
        for b in &self.blocks {
            l_m += lambda(b.i + 1) + lambda(b.j + 1) + lambda(b.len);
        }
        l_m + KAPPA * u_s + KAPPA * u_t
    }

    #[inline(always)]
    fn admissible(&self, cand: &Block) -> bool {
        !self.blocks.iter().any(|b| block_overlaps(b, cand))
    }

    fn try_insert(&self, cand: &Block) -> Option<usize> {
        if !self.admissible(cand) {
            return None;
        }
        let mut delta: isize = 0;
        for k in cand.i..cand.i + cand.len {
            if !self.covered_s[k] {
                delta -= KAPPA as isize;
            }
        }
        for k in cand.j..cand.j + cand.len {
            if !self.covered_t[k] {
                delta -= KAPPA as isize;
            }
        }

        delta += (lambda(cand.i + 1) + lambda(cand.j + 1) + lambda(cand.len)) as isize;
        let old_lm = lambda(self.blocks.len());
        let new_lm = lambda(self.blocks.len() + 1);
        delta += (new_lm as isize) - (old_lm as isize);

        let new_e = (self.energy as isize + delta) as usize;
        Some(new_e)
    }

    fn try_remove(&self, idx: usize) -> usize {
        let b = self.blocks[idx];
        let mut delta: isize = 0;

        let mut other_s = bitvec![0; self.s_len];
        let mut other_t = bitvec![0; self.t_len];
        for (i, ob) in self.blocks.iter().enumerate() {
            if i == idx { continue; }
            other_s[ob.i..ob.i + ob.len].fill(true);
            other_t[ob.j..ob.j + ob.len].fill(true);
        }

        for k in b.i..b.i + b.len {
            if !other_s[k] {
                delta += KAPPA as isize;
            }
        }
        for k in b.j..b.j + b.len {
            if !other_t[k] {
                delta += KAPPA as isize;
            }
        }

        delta -= (lambda(b.i + 1) + lambda(b.j + 1) + lambda(b.len)) as isize;
        let old_lm = lambda(self.blocks.len());
        let new_lm = lambda(self.blocks.len() - 1);
        delta += (new_lm as isize) - (old_lm as isize);

        (self.energy as isize + delta) as usize
    }

    fn apply_insert(&mut self, cand: Block, new_e: usize) {
        self.covered_s[cand.i..cand.i + cand.len].fill(true);
        self.covered_t[cand.j..cand.j + cand.len].fill(true);
        self.blocks.push(cand);
        self.energy = new_e;
    }

    fn apply_remove(&mut self, idx: usize, new_e: usize) {
        self.blocks.remove(idx);
        self.covered_s.fill(false);
        self.covered_t.fill(false);
        for b in &self.blocks {
            self.covered_s[b.i..b.i + b.len].fill(true);
            self.covered_t[b.j..b.j + b.len].fill(true);
        }
        self.energy = new_e;
    }

    pub fn descent_step(&mut self, s: &[u8], t: &[u8], mt: &MatchTable) -> bool {
        let mut best_move: Option<(Move, usize, Vec<Block>)> = None;
        let mut best_e = self.energy;
        let mut best_key = {
            let mut k = self.blocks.clone();
            k.sort();
            k
        };

        // Try insertions
        for i in 0..s.len() {
            for j in 0..t.len() {
                let max_l = mt.max_len(i, j);
                for len in 1..=max_l {
                    let cand = Block { i, j, len };
                    if let Some(new_e) = self.try_insert(&cand) {
                        let mut next_key = self.blocks.clone();
                        next_key.push(cand);
                        next_key.sort();

                        if new_e < best_e || (new_e == best_e && next_key < best_key) {
                            best_e = new_e;
                            best_key = next_key;
                            best_move = Some((Move::Insert(cand), new_e, best_key.clone()));
                        }
                    }
                }
            }
        }

        // Try deletions
        for i in 0..self.blocks.len() {
            let new_e = self.try_remove(i);
            let mut next_key = self.blocks.clone();
            next_key.remove(i);
            next_key.sort();

            if new_e < best_e || (new_e == best_e && next_key < best_key) {
                best_e = new_e;
                best_key = next_key;
                best_move = Some((Move::Remove(i), new_e, best_key.clone()));
            }
        }

        if let Some((m, e, _)) = best_move {
            match m {
                Move::Insert(b) => self.apply_insert(b, e),
                Move::Remove(i) => self.apply_remove(i, e),
            }
            true
        } else {
            false
        }
    }

    pub fn run(&mut self, s: &[u8], t: &[u8], mt: &MatchTable) {
        while self.descent_step(s, t, mt) {}
    }
}

enum Move {
    Insert(Block),
    Remove(usize),
}
