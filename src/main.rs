//! Tulya - MDL Block Alignment Tool
//!
//! A deterministic algorithm that aligns two strings by minimizing description length.

mod algorithms;
mod cli;
mod visualization;

use algorithms::{
    phase2_clustering, phase3_segments, ByteCluster, MappingResult, Segment,
    SegmentType,
};
use clap::Parser;
use cli::Args;

fn main() {
    let args = Args::parse();

    if args.no_color {
        colored::control::set_override(false);
    }

    // Resolve input mode
    let (source, target) = cli::resolve_input(&args);

    // Edge case: empty input
    if source.is_empty() && target.is_empty() {
        if !args.quiet {
            eprintln!("Both inputs are empty.");
        }
        return;
    }

    // Edge case: identical input
    if source == target {
        if !args.quiet {
            eprintln!("Source and target are identical.");
            let cluster = ByteCluster {
                id: 0,
                source_indices: (0..source.len()).collect(),
                source_range: (0, source.len().saturating_sub(1)),
                target_range: (0, target.len().saturating_sub(1)),
                byte_len: source.len(),
                density: 1.0,
                fragment_count: 1,
                content: String::from_utf8_lossy(&source).into_owned(),
            };
            let mapping = MappingResult {
                source_to_target: Default::default(),
                target_to_source: Default::default(),
                unmapped_source: Vec::new(),
                unmapped_target: Vec::new(),
            };
            visualization::visualize(visualization::VisualizeParams {
                source: &source,
                target: &target,
                segments: &[Segment {
                    id: 0,
                    label: SegmentType::Common,
                    byte_len: source.len(),
                    source_range: Some((0, source.len().saturating_sub(1))),
                    target_range: Some((0, target.len().saturating_sub(1))),
                    content: String::from_utf8_lossy(&source).into_owned(),
                }],
                clusters: &[cluster],
                mapping: &mapping,
                sort_mode: args.sort,
                show_mapping: args.show_mapping,
            });
        }
        return;
    }

    // Core processing
    let mapping: MappingResult;
    use algorithms::mdl::core::*;

    let mt = MatchTable::new(&source, &target);
    match args.version.as_str() {
        "v1" | "mdl" => {
            let mut fam = MdlV1::default();
            // Show steps only if --debug
            fam.run_with_diagnostics(&source, &target, &mt, args.debug);
            
            if !args.summary_only && !args.quiet {
                let insights = fam.block_insights(&source, &target);
                algorithms::mdl::core::diagnostics::print_block_table(&insights);
                algorithms::mdl::core::diagnostics::print_ascii_alignment(&source, &target, &fam.blocks);
            }
            
            let breakdown = fam.analyze(&source, &target);
            if !args.quiet {
                breakdown.print();
            }

            mapping = algorithms::mdl::core::convert_mdl_to_mapping(source.len(), target.len(), &fam.blocks);
        }
        "v2" | "fast" => {
            let mut fam = MdlV2::new(source.len(), target.len());
            fam.run(&source, &target, &mt);
            mapping = algorithms::mdl::core::convert_mdl_to_mapping(source.len(), target.len(), &fam.blocks);
        }
        other => {
            if !args.quiet {
                eprintln!("Invalid MDL version: {}. Defaulting to v1.", other);
            }
            let mut fam = MdlV1::default();
            fam.run_with_diagnostics(&source, &target, &mt, args.debug);
            mapping = algorithms::mdl::core::convert_mdl_to_mapping(source.len(), target.len(), &fam.blocks);
        }
    }

    if !args.quiet {
        let clusters = phase2_clustering(&source, &mapping);
        let segments = phase3_segments(&source, &target, &mapping);

        visualization::visualize(visualization::VisualizeParams {
            source: &source,
            target: &target,
            segments: &segments,
            clusters: &clusters,
            mapping: &mapping,
            sort_mode: args.sort,
            show_mapping: args.show_mapping,
        });
    }
}
