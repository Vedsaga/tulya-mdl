#[cfg(test)]
#[macro_use]
mod macros;

#[cfg(test)]
#[macro_use]
#[path = "heuristic_macros.rs"]
mod heuristic_macros;

#[cfg(test)]
mod tests {
    use crate::algorithms::mdl::core::{MdlV1, MdlV2, MatchTable, convert_mdl_to_mapping};
    use crate::algorithms::types::{
        phase2_clustering, MappingResult,
    };

    // ── v1 ──────────────────────────────────────────────────────────────────
    mod v1 {
        use super::*;

        fn run_v1(s: &[u8], t: &[u8]) -> MdlV1 {
            let mt = MatchTable::new(s, t);
            let mut family = MdlV1::default();
            family.run_to_fixed_point(s, t, &mt);
            family
        }

        // Bridge to heuristic tests
        fn phase1_mapping(s: &[u8], t: &[u8]) -> MappingResult {
            let fam = run_v1(s, t);
            convert_mdl_to_mapping(s.len(), t.len(), &fam.blocks)
        }

        define_test_utils!();
        define_core_functional_tests!();
        define_complex_mapping_tests!();
        define_unicode_boundary_tests!();
        define_mdl_block_aligner_tests!(run_v1);
    }

    // ── v2 ──────────────────────────────────────────────────────────────────
    mod v2 {
        use super::*;

        fn run_v2(s: &[u8], t: &[u8]) -> MdlV2 {
            let mt = MatchTable::new(s, t);
            let mut family = MdlV2::new(s.len(), t.len());
            family.run(s, t, &mt);
            family
        }

        // Bridge to heuristic tests
        fn phase1_mapping(s: &[u8], t: &[u8]) -> MappingResult {
            let fam = run_v2(s, t);
            convert_mdl_to_mapping(s.len(), t.len(), &fam.blocks)
        }

        define_test_utils!();
        define_core_functional_tests!();
        define_complex_mapping_tests!();
        define_unicode_boundary_tests!();
        define_mdl_block_aligner_tests!(run_v2);
        define_mdl_block_aligner_comparison_tests!(run_v2);
    }
}
