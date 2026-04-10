// MDL specific test macros
// This file is located in src/algorithms/mdl/tests/macros.rs

macro_rules! define_mdl_block_aligner_tests {
    ($run_fn:expr) => {
        #[cfg(test)]
        mod shared_tests {
            use super::*;

            #[test]
            fn test_identical() {
                let s = b"abcabc";
                let t = b"abcabc";
                let result = $run_fn(s, t);
                assert!(result.energy(s, t) <= 8 * (s.len() + t.len()));
            }

            #[test]
            fn test_completely_different() {
                let s = b"abc";
                let t = b"xyz";
                let result = $run_fn(s, t);
                assert!(result.energy(s, t) >= 8 * (s.len() + t.len()));
            }

            #[test]
            fn test_empty() {
                let s = b"";
                let t = b"";
                let result = $run_fn(s, t);
                assert_eq!(result.blocks.len(), 0);
            }

            #[test]
            fn test_swap_blocks() {
                let s = b"abcXYZabc";
                let t = b"XYZabcabc";
                let fam = $run_fn(s, t);
                assert!(!fam.blocks.is_empty());
                assert!(fam.energy(s, t) < 8 * (s.len() + t.len()));
            }

            #[test]
            fn test_natural_language() {
                let s = b"That mango was eaten by Ram";
                let t = b"Ram ate that mango";
                let fam = $run_fn(s, t);
                assert!(!fam.blocks.is_empty());
                assert!(fam.energy(s, t) < 8 * (s.len() + t.len()));
            }

            #[test]
            fn test_noise_in_middle() {
                let s = b"AAAAxxxBBBB";
                let t = b"AAAAyyyyBBBB";
                let fam = $run_fn(s, t);
                assert!(!fam.blocks.is_empty());
                assert!(fam.energy(s, t) < 8 * (s.len() + t.len()));
            }

            #[test]
            fn test_deterministic() {
                let s = b"abcabcabc";
                let t = b"abcXYZabc";
                let a = $run_fn(s, t);
                let b = $run_fn(s, t);
                assert_eq!(a.blocks, b.blocks);
                assert_eq!(a.energy(s, t), b.energy(s, t));
            }
        }
    };
}

macro_rules! define_mdl_block_aligner_comparison_tests {
    ($fast_run_fn:expr) => {
        #[cfg(test)]
        mod comparison_tests {
            use super::*;

            #[test]
            fn test_identical() {
                let s = b"abcabc";
                let t = b"abcabc";
                let fast = $fast_run_fn(s, t);
                let reference = {
                    let mt = MatchTable::new(s, t);
                    let mut family = MdlV1::default();
                    family.run_to_fixed_point(s, t, &mt);
                    family
                };
                assert_eq!(
                    fast.energy(s, t),
                    reference.energy(s, t),
                    "Energy mismatch for identical strings: fast={}, reference={}",
                    fast.energy(s, t),
                    reference.energy(s, t)
                );
            }

            #[test]
            fn test_completely_different() {
                let s = b"abc";
                let t = b"xyz";
                let fast = $fast_run_fn(s, t);
                assert!(fast.energy(s, t) >= 8 * (s.len() + t.len()));
            }

            #[test]
            fn test_empty() {
                let s = b"";
                let t = b"";
                let fast = $fast_run_fn(s, t);
                assert_eq!(fast.blocks.len(), 0);
            }

            #[test]
            fn test_swap() {
                let s = b"abcXYZabc";
                let t = b"XYZabcabc";
                let fast = $fast_run_fn(s, t);
                let reference = {
                    let mt = MatchTable::new(s, t);
                    let mut family = MdlV1::default();
                    family.run_to_fixed_point(s, t, &mt);
                    family
                };
                assert_eq!(
                    fast.energy(s, t),
                    reference.energy(s, t),
                    "Energy mismatch for swap: fast={}, reference={}",
                    fast.energy(s, t),
                    reference.energy(s, t)
                );
            }

            #[test]
            fn test_partial_overlap() {
                let s = b"abcXXabc";
                let t = b"XXabcXXabc";
                let fast = $fast_run_fn(s, t);
                let reference = {
                    let mt = MatchTable::new(s, t);
                    let mut family = MdlV1::default();
                    family.run_to_fixed_point(s, t, &mt);
                    family
                };
                assert_eq!(
                    fast.energy(s, t),
                    reference.energy(s, t),
                    "Energy mismatch for partial overlap: fast={}, reference={}",
                    fast.energy(s, t),
                    reference.energy(s, t)
                );
            }

            #[test]
            fn test_insertion() {
                let s = b"abc";
                let t = b"abcXYZ";
                let fast = $fast_run_fn(s, t);
                let reference = {
                    let mt = MatchTable::new(s, t);
                    let mut family = MdlV1::default();
                    family.run_to_fixed_point(s, t, &mt);
                    family
                };
                assert_eq!(
                    fast.energy(s, t),
                    reference.energy(s, t),
                    "Energy mismatch for insertion: fast={}, reference={}",
                    fast.energy(s, t),
                    reference.energy(s, t)
                );
            }

            #[test]
            fn test_reorder() {
                let s = b"abc def";
                let t = b"def abc";
                let fast = $fast_run_fn(s, t);
                let reference = {
                    let mt = MatchTable::new(s, t);
                    let mut family = MdlV1::default();
                    family.run_to_fixed_point(s, t, &mt);
                    family
                };
                assert_eq!(
                    fast.energy(s, t),
                    reference.energy(s, t),
                    "Energy mismatch for reorder: fast={}, reference={}",
                    fast.energy(s, t),
                    reference.energy(s, t)
                );
            }

            #[test]
            fn test_deterministic() {
                let s = b"abcabcabc";
                let t = b"abcXYZabc";
                let a = $fast_run_fn(s, t);
                let b = $fast_run_fn(s, t);
                assert_eq!(a.blocks, b.blocks);
                assert_eq!(a.energy(s, t), b.energy(s, t));
            }
        }
    };
}
