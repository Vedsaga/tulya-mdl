#![allow(unused_macros)]
// Heuristic and Mapping unified test macros
// This file is located in src/algorithms/heuristic/tests/macros.rs

macro_rules! define_test_utils {
    () => {
        fn assert_coverage(m: &MappingResult, sn: usize, tn: usize) {
            assert_eq!(m.source_to_target.len() + m.unmapped_source.len(), sn);
            assert_eq!(m.target_to_source.len() + m.unmapped_target.len(), tn);
        }
        fn assert_byte_eq(s: &[u8], t: &[u8], m: &MappingResult) {
            for (&si, tis) in &m.source_to_target {
                for &ti in tis {
                    assert_eq!(s[si], t[ti], "s[{}]!=t[{}]", si, ti);
                }
            }
        }
        fn assert_deterministic(s: &[u8], t: &[u8]) {
            let a = phase1_mapping(s, t);
            let b = phase1_mapping(s, t);
            assert_eq!(a.source_to_target, b.source_to_target);
        }
        fn _assert_range(m: &MappingResult, ss: usize, se: usize, ts: usize, te: usize) {
            assert_eq!(se - ss, te - ts, "range length mismatch");
            for i in 0..=(se - ss) {
                let tis = m
                    .source_to_target
                    .get(&(ss + i))
                    .expect(&format!("S[{}] unmapped", ss + i));
                assert!(
                    tis.contains(&(ts + i)),
                    "S[{}]->{:?} expected {}",
                    ss + i,
                    tis,
                    ts + i
                );
            }
        }
        fn assert_all_mapped(m: &MappingResult) {
            assert!(
                m.unmapped_source.is_empty(),
                "unmapped src: {:?}",
                m.unmapped_source
            );
            assert!(
                m.unmapped_target.is_empty(),
                "unmapped tgt: {:?}",
                m.unmapped_target
            );
        }
        fn assert_clusters_ok(s: &[u8], m: &MappingResult) {
            for c in &phase2_clustering(s, m) {
                for i in 1..c.source_indices.len() {
                    assert!(c.source_indices[i] > c.source_indices[i - 1]);
                }
                assert_eq!(
                    c.fragment_count, 1,
                    "cluster {} frag={}",
                    c.id, c.fragment_count
                );
            }
        }
        fn check(s: &[u8], t: &[u8]) {
            let m = phase1_mapping(s, t);
            assert_coverage(&m, s.len(), t.len());
            assert_byte_eq(s, t, &m);
            assert_deterministic(s, t);
            assert_clusters_ok(s, &m);
        }
    };
}

macro_rules! define_core_functional_tests {
    () => {
        #[test]
        fn test_identical() {
            let s = b"abc abc abc";
            let m = phase1_mapping(s, s);
            check(s, s);
            assert_all_mapped(&m);
            for i in 0..s.len() {
                assert!(m.source_to_target[&i].contains(&i));
            }
        }
        #[test]
        fn test_simple_reorder() {
            check(b"A B C D", b"C D A B");
            assert_all_mapped(&phase1_mapping(b"A B C D", b"C D A B"));
        }
        #[test]
        fn test_insertion() {
            let m = phase1_mapping(b"hello world", b"hello beautiful world");
            check(b"hello world", b"hello beautiful world");
            assert!(m.unmapped_source.is_empty());
            assert!(!m.unmapped_target.is_empty());
        }
        #[test]
        fn test_deletion() {
            let m = phase1_mapping(b"hello beautiful world", b"hello world");
            check(b"hello beautiful world", b"hello world");
            assert!(!m.unmapped_source.is_empty());
            assert!(m.unmapped_target.is_empty());
        }
        #[test]
        fn test_swap() {
            check(b"AB", b"BA");
            assert_all_mapped(&phase1_mapping(b"AB", b"BA"));
        }
        #[test]
        fn test_repetition() {
            let s = b"aaaaabaaaaa";
            let m = phase1_mapping(s, s);
            check(s, s);
            assert_all_mapped(&m);
            for i in 0..s.len() {
                assert!(m.source_to_target[&i].contains(&i));
            }
        }
        #[test]
        fn test_punct_move() {
            check(b"hello, world!", b"world! hello,");
            assert_all_mapped(&phase1_mapping(b"hello, world!", b"world! hello,"));
        }
    };
}

macro_rules! define_complex_mapping_tests {
    () => {
        #[test]
        fn test_duplication() {
            let s = b"abc";
            let t = b"abcabc";
            let m = phase1_mapping(s, t);
            assert_coverage(&m, s.len(), t.len());
            assert_byte_eq(s, t, &m);
            let t0 = m.source_to_target.get(&0).expect("src[0] unmapped");
            let t1 = m.source_to_target.get(&1).expect("src[1] unmapped");
            let t2 = m.source_to_target.get(&2).expect("src[2] unmapped");
            assert!(t0.contains(&0) && t0.contains(&3), "src[0]->{:?}", t0);
            assert!(t1.contains(&1) && t1.contains(&4), "src[1]->{:?}", t1);
            assert!(t2.contains(&2) && t2.contains(&5), "src[2]->{:?}", t2);
            assert!(m.unmapped_source.is_empty());
            assert!(m.unmapped_target.is_empty());
        }
        #[test]
        fn test_contraction() {
            let s = b"do not";
            let t = b"don't";
            let m = phase1_mapping(s, t);
            assert_coverage(&m, s.len(), t.len());
            assert_byte_eq(s, t, &m);
            assert!(m
                .source_to_target
                .get(&0)
                .map(|v: &Vec<usize>| v.contains(&0))
                .unwrap_or(false));
            assert!(m
                .source_to_target
                .get(&1)
                .map(|v: &Vec<usize>| v.contains(&1))
                .unwrap_or(false));
            assert!(m
                .source_to_target
                .get(&3)
                .map(|v: &Vec<usize>| v.contains(&2))
                .unwrap_or(false));
            assert!(m
                .source_to_target
                .get(&5)
                .map(|v: &Vec<usize>| v.contains(&4))
                .unwrap_or(false));
            assert!(m.unmapped_target.contains(&3), "apostrophe should be Added");
            assert!(m.unmapped_source.contains(&2), "space should be Missing");
        }
        #[test]
        fn test_duplication_multi_mapping() {
            let mapping = phase1_mapping(b"abc", b"abcabc");
            for i in 0..3 {
                assert!(mapping.source_to_target.get(&i).unwrap().len() >= 2);
            }
        }
        #[test]
        fn test_contraction_many_to_one() {
            let mapping = phase1_mapping(b"do not", b"don't");
            assert!(mapping.target_to_source.values().any(|v: &Vec<usize>| v.len() > 1));
        }
        #[test]
        fn test_duplicate_block_swap() {
            let source = b"task A task B";
            let target = b"task B task A";

            let mapping = phase1_mapping(source, target);
            let clusters = phase2_clustering(source, &mapping);
            let contents: Vec<String> = clusters.iter().map(|c| c.content.clone()).collect();

            println!("DEBUG clusters = {:#?}", contents);

            // Ensure at least one meaningful "task" block is detected
            assert!(
                contents.iter().any(|c: &String| c.contains("task")),
                "Expected at least one 'task' block, got: {:?}",
                contents
            );

            // Ensure no fragmentation into tiny pieces (structural coherence)
            assert!(
                contents.iter().any(|c: &String| c.len() >= 6), // "task A" or "task B"
                "Expected a reasonably large block (>=6 chars), got: {:?}",
                contents
            );
        }
        #[test]
        fn test_overlapping_patterns() {
            let mapping = phase1_mapping(b"abcabcX", b"abcXabc");
            assert!(mapping.unmapped_source.is_empty());
            assert!(mapping.unmapped_target.is_empty());
        }
        #[test]
        fn test_cross_block_reorder() {
            let mapping = phase1_mapping(b"A B C D", b"C D A B");
            let clusters = phase2_clustering(b"A B C D", &mapping);
            assert!(clusters.len() >= 2);
        }
        #[test]
        fn test_inversion_required_case() {
            let mapping = phase1_mapping(b"abc def ghi", b"ghi def abc");
            assert!(mapping.unmapped_source.is_empty());
        }
        #[test]
        fn test_duplicate_with_noise() {
            let mapping = phase1_mapping(b"foo bar foo baz", b"foo baz foo bar");
            let clusters = phase2_clustering(b"foo bar foo baz", &mapping);
            assert!(clusters.len() >= 2);
        }
        #[test]
        fn test_f1_coverage_maximization() {
            let s = b"aa";
            let t = b"a";
            let m = phase1_mapping(s, t);
            assert!(m.source_to_target.contains_key(&0));
            assert!(m.source_to_target.contains_key(&1));
            assert!(m.unmapped_source.is_empty());
        }
    };
}

macro_rules! define_unicode_boundary_tests {
    () => {
        #[test]
        fn test_hindi() {
            let s = "राम आम खाता है".as_bytes();
            let t = "आम राम खाता है".as_bytes();
            check(s, t);
            assert_all_mapped(&phase1_mapping(s, t));
        }
        #[test]
        fn test_cafe() {
            let m = phase1_mapping("café".as_bytes(), "café!".as_bytes());
            check("café".as_bytes(), "café!".as_bytes());
            assert!(m.unmapped_source.is_empty());
            assert!(!m.unmapped_target.is_empty());
        }
        #[test]
        fn test_unicode_stability_adv() {
            let mapping = phase1_mapping("café café".as_bytes(), "café café".as_bytes());
            assert!(mapping.unmapped_source.is_empty());
        }
        #[test]
        fn test_unicode_reorder_adv() {
            let mapping = phase1_mapping("café bar".as_bytes(), "bar café".as_bytes());
            assert!(!mapping.source_to_target.is_empty());
        }
        #[test]
        fn test_empty_src() {
            assert_eq!(phase1_mapping(b"", b"abc").unmapped_target.len(), 3);
        }
        #[test]
        fn test_empty_tgt() {
            assert_eq!(phase1_mapping(b"abc", b"").unmapped_source.len(), 3);
        }
        #[test]
        fn test_both_empty() {
            assert_coverage(&phase1_mapping(b"", b""), 0, 0);
        }
        #[test]
        fn test_disjoint() {
            let m = phase1_mapping(b"xyz", b"abc");
            check(b"xyz", b"abc");
            assert_eq!(m.unmapped_source.len(), 3);
            assert_eq!(m.unmapped_target.len(), 3);
        }
        #[test]
        fn test_empty_vs_nonempty_adv() {
            assert_eq!(phase1_mapping(b"", b"abc").unmapped_target.len(), 3);
        }
        #[test]
        fn test_nonempty_vs_empty_adv() {
            assert_eq!(phase1_mapping(b"abc", b"").unmapped_source.len(), 3);
        }
    };
}

macro_rules! define_stress_stability_tests {
    () => {
        #[test]
        fn test_high_repetition_stability() {
            let mapping = phase1_mapping(b"aaaaaaaaaaaa", b"aaaaaaaaaaaa");
            assert_eq!(mapping.unmapped_source.len(), 0);
            assert_eq!(mapping.unmapped_target.len(), 0);
            for i in 0..12 {
                assert!(mapping.source_to_target[&i].contains(&i));
            }
        }
        #[test]
        fn test_alternating_pattern_shift() {
            let mapping = phase1_mapping(b"abababab", b"babababa");
            assert!(mapping.unmapped_source.is_empty());
            assert!(mapping.unmapped_target.is_empty());
        }
        #[test]
        fn test_large_repetition_blocks_adv() {
            let s = "abc".repeat(100).into_bytes();
            let m = phase1_mapping(&s, &s);
            assert_eq!(m.unmapped_source.len(), 0);
        }
        #[test]
        fn test_shifted_large_block_adv() {
            let s = "aaaaabbbbbccccc".repeat(10).into_bytes();
            let t = "cccccaaaaabbbbb".repeat(10).into_bytes();
            let m = phase1_mapping(&s, &t);
            assert!(!m.source_to_target.is_empty());
        }
        #[test]
        fn test_noise_injection_adv() {
            let mapping = phase1_mapping(b"hello world", b"hello XXX world");
            assert!(mapping.unmapped_target.len() >= 3);
            assert!(mapping.source_to_target.contains_key(&0)); // 'h'
        }
        #[test]
        fn test_noise_deletion_adv() {
            let mapping = phase1_mapping(b"hello XXX world", b"hello world");
            assert!(mapping.unmapped_source.len() >= 3);
            assert!(mapping.target_to_source.contains_key(&0)); // 'h'
        }
        #[test]
        fn test_large_repeated_blocks_shifted() {
            let s = "abc".repeat(50).into_bytes();
            let t = "cab".repeat(50).into_bytes();
            let m = phase1_mapping(&s, &t);
            assert!(m.source_to_target.len() > s.len() / 2);
        }
    };
}

macro_rules! define_structural_integrity_tests {
    () => {
        #[test]
        fn test_delta_consistency_repeated_block() {
            let s = b"abcabc";
            let t = b"abcabc";
            let m = phase1_mapping(s, t);
            assert!(m.unmapped_source.is_empty());
            assert!(m.unmapped_target.is_empty());
            let mut deltas = Vec::new();
            for (&si, tis) in &m.source_to_target {
                for &ti in tis {
                    deltas.push(ti as isize - si as isize);
                }
            }
            deltas.sort();
            deltas.dedup();
            assert!(deltas.len() <= 2);
        }
        #[test]
        fn test_delta_consistency_shifted_pattern() {
            let s = b"abababab";
            let t = b"babababa";
            let m = phase1_mapping(s, t);
            assert!(m.unmapped_source.is_empty());
            assert!(m.unmapped_target.is_empty());
            let mut deltas = Vec::new();
            for (&si, tis) in &m.source_to_target {
                for &ti in tis {
                    deltas.push(ti as isize - si as isize);
                }
            }
            deltas.sort();
            deltas.dedup();
            assert!(deltas.len() <= 3);
        }
        #[test]
        fn test_contraction_run_coherence() {
            let s = b"don't";
            let t = b"do not";
            let m = phase1_mapping(s, t);
            let clusters = phase2_clustering(s, &m);
            assert!(clusters.len() <= 4);
        }
        #[test]
        fn test_long_run_preference() {
            let s = b"aaaaabaaaaa";
            let t = b"aaaaabaaaaa";
            let m = phase1_mapping(s, t);
            let clusters = phase2_clustering(s, &m);
            assert!(clusters.len() <= 3);
        }
        #[test]
        fn test_block_integrity_under_noise() {
            let s = b"abcXYZ";
            let t = b"abcX YZ";
            let m = phase1_mapping(s, t);
            let clusters = phase2_clustering(s, &m);
            assert!(clusters.len() <= 4);
        }
        #[test]
        fn test_local_optimality_simple_swap() {
            let s = b"abc";
            let t = b"bac";
            let m = phase1_mapping(s, t);
            assert!(m.source_to_target.get(&0).unwrap().contains(&1));
            assert!(m.source_to_target.get(&1).unwrap().contains(&0));
            assert!(m.source_to_target.get(&2).unwrap().contains(&2));
        }
        #[test]
        fn test_local_optimality_overlap() {
            let s = b"abab";
            let t = b"baba";
            let m = phase1_mapping(s, t);
            assert!(m.unmapped_source.is_empty());
            assert!(m.unmapped_target.is_empty());
        }
        #[test]
        fn test_unique_bytes_no_ambiguity() {
            let s = b"abcdefghijklmnopqrstuvwxyz";
            let t = b"abcdefghijklmnopqrstuvwxyz";
            let m = phase1_mapping(s, t);
            assert!(m.unmapped_source.is_empty());
            assert!(m.unmapped_target.is_empty());
        }
        #[test]
        fn test_bidirectional_consistency_contraction() {
            let s = b"do not";
            let t = b"don't";
            let m1 = phase1_mapping(s, t);
            let m2 = phase1_mapping(t, s);
            assert!(m1.source_to_target.len() >= 4);
            assert!(m2.source_to_target.len() >= 4);
        }
        #[test]
        fn test_uniform_string_stability() {
            let s = b"aaaaaaaaaaaa";
            let t = b"aaaaaaaaaaaa";
            let m = phase1_mapping(s, t);
            assert!(m.unmapped_source.is_empty());
            assert!(m.unmapped_target.is_empty());
        }
        #[test]
        fn test_partial_overlap_blocks_strict() {
            let s = b"abcabcX";
            let t = b"abcXabc";
            let m = phase1_mapping(s, t);
            assert!(m.unmapped_source.is_empty());
            assert!(m.unmapped_target.is_empty());
            let clusters = phase2_clustering(s, &m);
            assert!(clusters.len() >= 2);
        }
        #[test]
        fn test_partial_overlap_blocks() {
            let mapping = phase1_mapping(b"XYZabcXYZ", b"abcXYZabc");
            assert!(mapping.source_to_target.len() >= 6);
        }
        #[test]
        fn test_single_char_swap_adv() {
            assert_eq!(phase1_mapping(b"AB", b"BA").source_to_target.len(), 2);
        }
    };
}

macro_rules! define_evolutionary_scenario_tests {
    ($version:ident) => {
        #[test]
        fn test_case1() {
            let s = b"Hey do this task, not that task.";
            let t = b"Hey not that task, do this task?";
            let m = phase1_mapping(s, t);
            check(s, t);
            _assert_range(&m, 0, 2, 0, 2);
            _assert_range(&m, 3, 15, 18, 30);
            _assert_range(&m, 16, 16, 17, 17);
            _assert_range(&m, 17, 30, 3, 16);
            assert!(m.unmapped_source.contains(&31));
            assert!(m.unmapped_target.contains(&31));
        }
        #[test]
        fn test_case2() {
            let s = b"Hey do this task, not that task.";
            let t = b"Hey not that task, do this task.";
            let m = phase1_mapping(s, t);
            check(s, t);
            _assert_range(&m, 0, 2, 0, 2);
            _assert_range(&m, 3, 15, 18, 30);
            _assert_range(&m, 16, 16, 17, 17);
            _assert_range(&m, 17, 30, 3, 16);
            _assert_range(&m, 31, 31, 31, 31);
            assert_all_mapped(&m);
        }
        #[test]
        fn test_reorder_with_duplicates_strict() {
            let source = b"Hey do this task, not that task.".to_vec();
            let target = b"Hey not that task, do this task?".to_vec();
            let mapping = phase1_mapping(&source, &target);
            let clusters = phase2_clustering(&source, &mapping);
            let segments = phase3_segments(&source, &target, &mapping);
            let contents: Vec<String> = clusters.iter().map(|c| c.content.clone()).collect();
            assert!(contents.iter().any(|c: &String| c.contains("not that task")));
            assert!(contents.iter().any(|c: &String| c.contains("do this task")));
            assert!(clusters.len() >= 4);
            assert!(segments
                .iter()
                .any(|s| s.content == "." && s.label == SegmentType::Missing));
            assert!(segments
                .iter()
                .any(|s| s.content == "?" && s.label == SegmentType::Added));
        }
        #[test]
        fn test_long_context_bias_failure_case() {
            let mapping = phase1_mapping(b"task task task", b"task task");
            assert!(!mapping.unmapped_source.is_empty());
        }
    };
}

macro_rules! define_adversarial_invariant_tests {
    () => {
        #[test]
        fn strict_single_delta_only() {
            let s = b"abcabc";
            let t = b"abcabc";
            let m = phase1_mapping(s, t);
            let mut deltas = Vec::new();
            for (&si, tis) in &m.source_to_target {
                for &ti in tis {
                    deltas.push(ti as isize - si as isize);
                }
            }
            deltas.sort();
            deltas.dedup();
            assert!(
                deltas.len() == 1,
                "Expected single global alignment, got multiple: {:?}",
                deltas
            );
        }
        #[test]
        fn strict_no_crossing_edges() {
            let s = b"abcdabcd";
            let t = b"abcdabcd";
            let m = phase1_mapping(s, t);
            let mut pairs = Vec::new();
            for (&si, tis) in &m.source_to_target {
                for &ti in tis {
                    pairs.push((si, ti));
                }
            }
            pairs.sort();
            for i in 0..pairs.len() {
                for j in i + 1..pairs.len() {
                    let (s1, t1) = pairs[i];
                    let (s2, t2) = pairs[j];
                    if s1 < s2 {
                        assert!(
                            t1 < t2,
                            "Crossing detected: ({},{}) -> ({},{})",
                            s1,
                            t1,
                            s2,
                            t2
                        );
                    }
                }
            }
        }
        #[test]
        fn strict_block_consistency() {
            let s = b"xyzabcxyz";
            let t = b"xyzabcxyz";
            let m = phase1_mapping(s, t);
            let d0 = m.source_to_target[&0][0] as isize - 0;
            let d1 = m.source_to_target[&1][0] as isize - 1;
            let d2 = m.source_to_target[&2][0] as isize - 2;
            assert!(d0 == d1 && d1 == d2, "Inconsistent mapping in first block");
            let d6 = m.source_to_target[&6][0] as isize - 6;
            let d7 = m.source_to_target[&7][0] as isize - 7;
            let d8 = m.source_to_target[&8][0] as isize - 8;
            assert!(d6 == d7 && d7 == d8, "Inconsistent mapping in second block");
        }
        #[test]
        fn strict_no_fragmentation_allowed() {
            let s = b"aaaaaa";
            let t = b"aaaaaa";
            let m = phase1_mapping(s, t);
            let clusters = phase2_clustering(s, &m);
            assert!(clusters.len() == 1);
        }
        #[test]
        fn strict_symmetry_required() {
            let s = b"abcabc";
            let t = b"abcabc";
            let m1 = phase1_mapping(s, t);
            let m2 = phase1_mapping(t, s);
            for (&si, tis) in &m1.source_to_target {
                for &ti in tis {
                    let reverse = m2.source_to_target.get(&ti);
                    assert!(reverse.map_or(false, |v: &Vec<usize>| v.contains(&si)));
                }
            }
        }
        #[test]
        fn strict_global_choice_repetition() {
            let s = b"abcabcabc";
            let t = b"abcabcabc";
            let m = phase1_mapping(s, t);
            let mut deltas = Vec::new();
            for (&si, tis) in &m.source_to_target {
                for &ti in tis {
                    deltas.push(ti as isize - si as isize);
                }
            }
            deltas.sort();
            deltas.dedup();
            assert!(deltas.len() == 1);
        }
        #[test]
        fn fail_competing_structures() {
            let s = b"abcXYZabc";
            let t = b"abcabcXYZ";
            let m = phase1_mapping(s, t);
            let f = vec![
                m.source_to_target[&0][0],
                m.source_to_target[&1][0],
                m.source_to_target[&2][0],
            ];
            let s_tgt = vec![
                m.source_to_target[&6][0],
                m.source_to_target[&7][0],
                m.source_to_target[&8][0],
            ];
            println!("DEBUG fail_competing_structures: f={:?}, s={:?}", f, s_tgt);
            assert!(f != s_tgt);
        }
        #[test]
        fn fail_context_deception() {
            let s = b"aaaaab";
            let t = b"baaaaa";
            let m = phase1_mapping(s, t);
            assert_eq!(t[m.source_to_target[&5][0]], b'b');
        }
        #[test]
        fn fail_near_equal_choice() {
            let s = b"abc1abc2";
            let t = b"abc2abc1";
            let m = phase1_mapping(s, t);
            assert_eq!(t[m.source_to_target[&3][0]], b'1');
            assert_eq!(t[m.source_to_target[&7][0]], b'2');
        }
        #[test]
        fn fail_run_overfit() {
            let s = b"aaaaXaaaa";
            let t = b"aaaaYaaaa";
            let m = phase1_mapping(s, t);
            let xm = m.source_to_target.get(&4);
            assert!(xm.is_none() || t[xm.unwrap()[0]] != b'Y');
        }
        #[test]
        fn fail_reorder_ambiguity() {
            let s = b"A B C A B C";
            let t = b"C A B C A B";
            let m = phase1_mapping(s, t);
            let mut d = Vec::new();
            for (&si, tis) in &m.source_to_target {
                for &ti in tis {
                    d.push(ti as isize - si as isize);
                }
            }
            d.sort();
            d.dedup();
            assert!(d.len() >= 1);
        }
        #[test]
        fn fail_early_lock_wrong_structure() {
            let s = b"abcXXXabc";
            let t = b"XXXabcabc";
            let m = phase1_mapping(s, t);
            let f = vec![
                m.source_to_target[&0][0],
                m.source_to_target[&1][0],
                m.source_to_target[&2][0],
            ];
            let s_tgt = vec![
                m.source_to_target[&6][0],
                m.source_to_target[&7][0],
                m.source_to_target[&8][0],
            ];
            println!(
                "DEBUG fail_early_lock_wrong_structure: f={:?}, s_tgt={:?}",
                f, s_tgt
            );
            assert!(f != s_tgt);
        }
        #[test]
        fn fail_rare_signal_suppressed() {
            let s = b"aaaaab";
            let t = b"baaaaa";
            let m = phase1_mapping(s, t);
            assert_eq!(t[m.source_to_target[&5][0]], b'b');
        }
        #[test]
        fn fail_run_bias_overpowering_correctness() {
            let s = b"aaaaXaaaa";
            let t = b"aaaaYaaaa";
            let m = phase1_mapping(s, t);
            let xm = m.source_to_target.get(&4);
            assert!(xm.is_none() || t[xm.unwrap()[0]] != b'Y');
        }
        #[test]
        fn fail_delta_drift_between_blocks() {
            let s = b"abcDEFabc";
            let t = b"DEFabcabc";
            let m = phase1_mapping(s, t);
            let mut d = Vec::new();
            for (&si, tis) in &m.source_to_target {
                for &ti in tis {
                    d.push(ti as isize - si as isize);
                }
            }
            d.sort();
            d.dedup();
            println!("DEBUG {}: m = {:#?}", module_path!(), m);
            assert!(!d.is_empty());
        }
    };
}
