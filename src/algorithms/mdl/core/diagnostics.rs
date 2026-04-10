use super::types::{Block, lambda, KAPPA};
use colored::*;

#[derive(Debug, Clone)]
pub struct EnergyBreakdown {
    pub total_energy: usize,
    pub desc_cost: usize,
    pub residual_cost: usize,
    pub uncovered_s: usize,
    pub uncovered_t: usize,
    pub covered_s: usize,
    pub covered_t: usize,
    pub coverage_ratio_s: f64,
    pub coverage_ratio_t: f64,
    pub num_blocks: usize,
    pub total_block_len: usize,
    pub avg_block_len: f64,
    pub empty_energy: usize,
    pub bits_saved: i64,
    pub compression_ratio: f64,
}

#[derive(Debug, Clone)]
pub struct BlockInsight {
    pub block: Block,
    pub desc_cost: usize,
    pub saved_bits: usize,
    pub net_gain: i64,
}

impl BlockInsight {
    pub fn calculate_all(blocks: &[Block], _s: &[u8], _t: &[u8]) -> Vec<Self> {
        let mut insights = Vec::new();
        for &b in blocks {
            let desc_cost = lambda(b.i + 1) + lambda(b.j + 1) + lambda(b.len);
            let saved_bits = KAPPA * (2 * b.len); // Ideal saving
            let net_gain = saved_bits as i64 - desc_cost as i64;
            insights.push(Self {
                block: b,
                desc_cost,
                saved_bits,
                net_gain,
            });
        }
        // Highest net_gain first
        insights.sort_by(|a, b| b.net_gain.cmp(&a.net_gain));
        insights
    }
}

impl EnergyBreakdown {
    pub fn calculate(blocks: &[Block], s: &[u8], t: &[u8]) -> Self {
        let mut covered_s = vec![false; s.len()];
        let mut covered_t = vec![false; t.len()];

        for b in blocks {
            for k in 0..b.len {
                covered_s[b.i + k] = true;
                covered_t[b.j + k] = true;
            }
        }

        let u_s = covered_s.iter().filter(|&&x| !x).count();
        let u_t = covered_t.iter().filter(|&&x| !x).count();
        let c_s = s.len() - u_s;
        let c_t = t.len() - u_t;

        let mut l_m = lambda(blocks.len());
        let mut total_block_len = 0;
        for b in blocks {
            l_m += lambda(b.i + 1) + lambda(b.j + 1) + lambda(b.len);
            total_block_len += b.len;
        }

        let total_energy = l_m + KAPPA * (u_s + u_t);
        let empty_energy = lambda(0) + KAPPA * (s.len() + t.len());

        Self {
            total_energy,
            desc_cost: l_m,
            residual_cost: KAPPA * (u_s + u_t),
            uncovered_s: u_s,
            uncovered_t: u_t,
            covered_s: c_s,
            covered_t: c_t,
            coverage_ratio_s: c_s as f64 / s.len() as f64,
            coverage_ratio_t: c_t as f64 / t.len() as f64,
            num_blocks: blocks.len(),
            total_block_len,
            avg_block_len: if blocks.is_empty() { 0.0 } else { total_block_len as f64 / blocks.len() as f64 },
            empty_energy,
            bits_saved: empty_energy as i64 - total_energy as i64,
            compression_ratio: total_energy as f64 / empty_energy as f64,
        }
    }

    pub fn print(&self) {
        println!("{}", "╔══════════════════════════════════════════════════════════════════╗".blue());
        println!("{}", "║                      ENERGY BREAKDOWN                            ║".blue());
        println!("{}", "╚══════════════════════════════════════════════════════════════════╝".blue());
        println!("{:<25} : {}", "Total Energy", self.total_energy.to_string().yellow());
        println!("{:<25} : {}", "Description Cost (L_m)", self.desc_cost);
        println!("{:<25} : {}", "Residual Cost (κ*U)", self.residual_cost);
        println!("{:<25} : {} / {} ({:.1}%)", "S Coverage", self.covered_s, self.covered_s + self.uncovered_s, self.coverage_ratio_s * 100.0);
        println!("{:<25} : {} / {} ({:.1}%)", "T Coverage", self.covered_t, self.covered_t + self.uncovered_t, self.coverage_ratio_t * 100.0);
        println!("{:<25} : {}", "Number of Blocks", self.num_blocks);
        println!("{:<25} : {}", "Total Block Length", self.total_block_len);
        println!("{:<25} : {:.2}", "Avg Block Length", self.avg_block_len);
        println!("{:<25} : {}", "Empty Energy (Initial)", self.empty_energy);
        println!("{:<25} : {}", "Bits Saved", if self.bits_saved > 0 { self.bits_saved.to_string().green() } else { self.bits_saved.to_string().red() });
        println!("{:<25} : {:.3}", "Compression Ratio", self.compression_ratio);
        println!();
    }
}

pub fn print_block_table(insights: &[BlockInsight]) {
    println!("{}", "╔══════════════════════════════════════════════════════════════════╗".blue());
    println!("{}", "║                         BLOCK TABLE                              ║".blue());
    println!("{}", "╚══════════════════════════════════════════════════════════════════╝".blue());
    println!("{:<4} | {:<12} | {:<12} | {:<4} | {:<10} | {:<10} | {:<8}", "ID", "S Range", "T Range", "Len", "Desc Cost", "Saved Bits", "Gain");
    println!("{}", "--------------------------------------------------------------------------------------------");
    for (idx, insight) in insights.iter().enumerate() {
        let b = &insight.block;
        let gain_str = if insight.net_gain >= 0 {
            format!("+{}", insight.net_gain).green()
        } else {
            insight.net_gain.to_string().red()
        };
        println!("{:<4} | [{:<2}..{:<2}) | [{:<2}..{:<2}) | {:<4} | {:<10} | {:<10} | {}", 
            idx, b.i, b.i + b.len, b.j, b.j + b.len, b.len, insight.desc_cost, insight.saved_bits, gain_str);
    }
    println!();
}

pub fn print_ascii_alignment(s: &[u8], t: &[u8], blocks: &[Block]) {
    let mut s_map = vec![None; s.len()];
    let mut t_map = vec![None; t.len()];

    for (idx, b) in blocks.iter().enumerate() {
        for k in 0..b.len {
            s_map[b.i + k] = Some(idx);
            t_map[b.j + k] = Some(idx);
        }
    }

    println!("{}", "╔══════════════════════════════════════════════════════════════════╗".blue());
    println!("{}", "║                      ASCII ALIGNMENT VIEW                        ║".blue());
    println!("{}", "╚══════════════════════════════════════════════════════════════════╝".blue());
    
    // Print S
    print!("S: ");
    for &c in s { print!("{}", String::from_utf8_lossy(&[c])); }
    println!();
    print!("   ");
    for &m in &s_map {
        match m {
            Some(id) => print!("{}", (id % 10).to_string().cyan()),
            None => print!("."),
        }
    }
    println!();

    println!();

    // Print T
    print!("T: ");
    for &c in t { print!("{}", String::from_utf8_lossy(&[c])); }
    println!();
    print!("   ");
    for &m in &t_map {
        match m {
            Some(id) => print!("{}", (id % 10).to_string().cyan()),
            None => print!("."),
        }
    }
    println!();
    println!();
}
