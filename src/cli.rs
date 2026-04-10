use clap::{Parser, ValueEnum};
use std::fs;

// ============================================================================
// CLI Arguments
// ============================================================================

#[derive(Parser, Debug)]
#[command(author, version, about = "Tulya–MDL: Deterministic MDL-Based Block Alignment", long_about = None, disable_version_flag = true)]
pub struct Args {
    /// Source input (string mode)
    #[arg(short = 's', long)]
    pub source: Option<String>,

    /// Target input (string mode)
    #[arg(short = 't', long)]
    pub target: Option<String>,

    /// Source input (hex byte mode, space-separated hex values)
    #[arg(long)]
    pub source_hex: Option<String>,

    /// Target input (hex byte mode, space-separated hex values)
    #[arg(long)]
    pub target_hex: Option<String>,

    /// Source input (raw binary file mode)
    #[arg(long)]
    pub source_file: Option<String>,

    /// Target input (raw binary file mode)
    #[arg(long)]
    pub target_file: Option<String>,

    /// Algorithm version (default: v1)
    #[arg(short = 'v', long, default_value = "v1")]
    pub version: String,

    /// Sort mode for output
    #[arg(long, value_enum, default_value_t = SortMode::Label)]
    pub sort: SortMode,

    /// Enable debug output (show steps of descent)
    #[arg(short, long)]
    pub debug: bool,

    /// Show mapping table (Source -> Target indices)
    #[arg(long)]
    pub show_mapping: bool,

    /// Disable ANSI colors
    #[arg(long)]
    pub no_color: bool,

    /// Only show final diagnostics summary
    #[arg(long)]
    pub summary_only: bool,

    /// Suppress most output
    #[arg(short, long)]
    pub quiet: bool,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, ValueEnum)]
pub enum SortMode {
    Label,
    LengthDesc,
    Density,
    Fragmentation,
}

// ============================================================================
// Input Parsing
// ============================================================================

pub fn parse_hex_string(hex_str: &str) -> Vec<u8> {
    hex_str
        .split_whitespace()
        .filter_map(|s| u8::from_str_radix(s, 16).ok())
        .collect()
}

pub fn read_binary_file(path: &str) -> Vec<u8> {
    fs::read(path).expect("Failed to read binary file")
}

pub fn resolve_input(args: &Args) -> (Vec<u8>, Vec<u8>) {
    let source: Vec<u8>;
    let target: Vec<u8>;

    // Priority: hex > file > string
    if let Some(ref hex) = args.source_hex {
        source = parse_hex_string(hex);
    } else if let Some(ref path) = args.source_file {
        source = read_binary_file(path);
    } else if let Some(ref s) = args.source {
        source = s.as_bytes().to_vec();
    } else {
        panic!("No source input provided");
    }

    if let Some(ref hex) = args.target_hex {
        target = parse_hex_string(hex);
    } else if let Some(ref path) = args.target_file {
        target = read_binary_file(path);
    } else if let Some(ref t) = args.target {
        target = t.as_bytes().to_vec();
    } else {
        panic!("No target input provided");
    }

    (source, target)
}
