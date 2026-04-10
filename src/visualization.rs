//! Visualization utilities for MDL block alignment

use colored::*;
use comfy_table::{Attribute, Cell, Color, Table};

use crate::algorithms::{ByteCluster, MappingResult, Segment, SegmentType};
use crate::cli::SortMode;

/// Arguments for the visualize function.
pub struct VisualizeParams<'a> {
    pub source: &'a [u8],
    pub target: &'a [u8],
    pub segments: &'a [Segment],
    pub clusters: &'a [ByteCluster],
    pub mapping: &'a MappingResult,
    pub sort_mode: SortMode,
    pub show_mapping: bool,
}

pub fn visualize(params: VisualizeParams) {
    let VisualizeParams {
        source,
        target,
        segments,
        clusters,
        mapping,
        sort_mode,
        show_mapping,
    } = params;

    // Sort segments
    let mut sorted_segments = segments.to_vec();
    match sort_mode {
        SortMode::LengthDesc => {
            sorted_segments.sort_by(|a, b| b.byte_len.cmp(&a.byte_len));
        }
        SortMode::Density | SortMode::Fragmentation => {
            // These apply to clusters, not segments
        }
        SortMode::Label => {
            sorted_segments.sort_by(|a, b| {
                let order = |l: &SegmentType| match l {
                    SegmentType::Common => 0,
                    SegmentType::Missing => 1,
                    SegmentType::Added => 2,
                };
                order(&a.label)
                    .cmp(&order(&b.label))
                    .then_with(|| b.byte_len.cmp(&a.byte_len))
            });
        }
    }

    // Re-assign IDs after sorting
    for (i, seg) in sorted_segments.iter_mut().enumerate() {
        seg.id = i;
    }

    // Count by type
    let mut common_count = 0;
    let mut missing_count = 0;
    let mut added_count = 0;

    for seg in &sorted_segments {
        match seg.label {
            SegmentType::Common => common_count += 1,
            SegmentType::Missing => missing_count += 1,
            SegmentType::Added => added_count += 1,
        }
    }

    // ---- Table 1: Segments ----
    eprintln!(
        "\n{}",
        "╔══════════════════════════════════════════════════════════════════╗".bold()
    );
    eprintln!(
        "{}",
        "║                      SEGMENT SUMMARY                             ║".bold()
    );
    eprintln!(
        "{}",
        "╚══════════════════════════════════════════════════════════════════╝".bold()
    );

    let mut segment_table = Table::new();
    segment_table.set_header(vec![
        Cell::new("ID").add_attribute(Attribute::Bold),
        Cell::new("Type").add_attribute(Attribute::Bold),
        Cell::new("Length").add_attribute(Attribute::Bold),
        Cell::new("Source Range").add_attribute(Attribute::Bold),
        Cell::new("Target Range").add_attribute(Attribute::Bold),
        Cell::new("Content").add_attribute(Attribute::Bold),
    ]);

    for seg in &sorted_segments {
        let color = match seg.label {
            SegmentType::Common => Color::Green,
            SegmentType::Missing => Color::Yellow,
            SegmentType::Added => Color::Red,
        };

        let type_str = match seg.label {
            SegmentType::Common => "✓ Common",
            SegmentType::Missing => "⏺ Missing",
            SegmentType::Added => "+ Added",
        };

        let src_range_str = seg
            .source_range
            .map_or("—".to_string(), |(s, e)| format!("{}..{}", s, e));
        let tgt_range_str = seg
            .target_range
            .map_or("—".to_string(), |(s, e)| format!("{}..{}", s, e));

        segment_table.add_row(vec![
            Cell::new(seg.id.to_string()).fg(color),
            Cell::new(type_str).fg(color),
            Cell::new(seg.byte_len.to_string()).fg(color),
            Cell::new(src_range_str).fg(color),
            Cell::new(tgt_range_str).fg(color),
            Cell::new(&seg.content).fg(color),
        ]);
    }

    eprintln!("{}", segment_table);

    // ---- Table 2: Clusters ----
    eprintln!(
        "\n{}",
        "╔══════════════════════════════════════════════════════════════════╗".bold()
    );
    eprintln!(
        "{}",
        "║                      CLUSTER SUMMARY                             ║".bold()
    );
    eprintln!(
        "{}",
        "╚══════════════════════════════════════════════════════════════════╝".bold()
    );

    let mut cluster_table = Table::new();
    cluster_table.set_header(vec![
        Cell::new("ID").add_attribute(Attribute::Bold),
        Cell::new("Length").add_attribute(Attribute::Bold),
        Cell::new("Source Range").add_attribute(Attribute::Bold),
        Cell::new("Target Range").add_attribute(Attribute::Bold),
        Cell::new("Density").add_attribute(Attribute::Bold),
        Cell::new("Fragments").add_attribute(Attribute::Bold),
        Cell::new("Content").add_attribute(Attribute::Bold),
    ]);

    for (_i, cluster) in clusters.iter().enumerate() {
        let density_color = if cluster.density > 0.75 {
            Color::Green
        } else if cluster.density > 0.5 {
            Color::Yellow
        } else {
            Color::Red
        };

        cluster_table.add_row(vec![
            Cell::new(format!("{}", cluster.id)),
            Cell::new(cluster.byte_len.to_string()),
            Cell::new(format!(
                "{}..{}",
                cluster.source_range.0, cluster.source_range.1
            )),
            Cell::new(format!(
                "{}..{}",
                cluster.target_range.0, cluster.target_range.1
            )),
            Cell::new(format!("{:.2}", cluster.density)).fg(density_color),
            Cell::new(cluster.fragment_count.to_string()),
            Cell::new(&cluster.content),
        ]);
    }

    eprintln!("{}", cluster_table);

    // ---- Table 3: Mapping (Debug) ----
    if show_mapping {
        eprintln!(
            "\n{}",
            "╔══════════════════════════════════════════════════════════════════╗".bold()
        );
        eprintln!(
            "{}",
            "║                    MAPPING TABLE (Source → Target)                ║".bold()
        );
        eprintln!(
            "{}",
            "╚══════════════════════════════════════════════════════════════════╝".bold()
        );

        let mut mapping_table = Table::new();
        mapping_table.set_header(vec![
            Cell::new("Source Idx").add_attribute(Attribute::Bold),
            Cell::new("Target Idx(s)").add_attribute(Attribute::Bold),
            Cell::new("Source Byte").add_attribute(Attribute::Bold),
        ]);

        let mut src_indices: Vec<usize> = mapping.source_to_target.keys().cloned().collect();
        src_indices.sort();

        for &s_idx in &src_indices {
            if let Some(t_indices) = mapping.source_to_target.get(&s_idx) {
                let mut t_strs: Vec<String> = t_indices.iter().map(|t: &usize| t.to_string()).collect();
                t_strs.sort();
                let byte_repr = String::from_utf8_lossy(&[source[s_idx]]).into_owned();

                mapping_table.add_row(vec![
                    Cell::new(format!("{}", s_idx)),
                    Cell::new(t_strs.join(", ")),
                    Cell::new(byte_repr),
                ]);
            }
        }

        eprintln!("{}", mapping_table);
    }

    // ---- Summary ----
    eprintln!(
        "\n{}",
        "╔══════════════════════════════════════════════════════════════════╗".bold()
    );
    eprintln!(
        "{}",
        "║                         SUMMARY                                   ║".bold()
    );
    eprintln!(
        "{}",
        "╚══════════════════════════════════════════════════════════════════╝".bold()
    );
    eprintln!(
        "Total bytes: Source={}, Target={}",
        source.len(),
        target.len()
    );
    eprintln!(
        "Unmapped: Source={}, Target={}",
        mapping.unmapped_source.len(),
        mapping.unmapped_target.len()
    );
    eprintln!(
        "Segments: {} (Common: {}, Missing: {}, Added: {})",
        sorted_segments.len(),
        common_count,
        missing_count,
        added_count
    );
    eprintln!("Clusters: {}", clusters.len());
}
