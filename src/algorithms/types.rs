use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct MappingResult {
    pub source_to_target: HashMap<usize, Vec<usize>>,
    pub target_to_source: HashMap<usize, Vec<usize>>,
    pub unmapped_source: Vec<usize>,
    pub unmapped_target: Vec<usize>,
}

#[derive(Debug, Clone)]
pub struct ByteCluster {
    pub id: usize,
    #[allow(dead_code)]
    pub source_indices: Vec<usize>,
    pub source_range: (usize, usize),
    pub target_range: (usize, usize),
    pub byte_len: usize,
    pub density: f64,
    pub fragment_count: usize,
    pub content: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SegmentType {
    Common,
    Missing,
    Added,
}

#[derive(Debug, Clone)]
pub struct Segment {
    pub id: usize,
    pub label: SegmentType,
    pub byte_len: usize,
    pub source_range: Option<(usize, usize)>,
    pub target_range: Option<(usize, usize)>,
    pub content: String,
}

pub fn phase2_clustering(source: &[u8], mapping: &MappingResult) -> Vec<ByteCluster> {
    let mut pairs: Vec<(usize, usize)> = mapping
        .source_to_target
        .iter()
        .flat_map(|(&si, tis)| tis.iter().map(move |&ti| (si, ti)))
        .collect();
    pairs.sort_unstable_by_key(|&(_, t)| t);
    if pairs.is_empty() {
        return Vec::new();
    }
    let mut clusters = Vec::new();
    let mut group = vec![pairs[0]];
    for &(si, ti) in &pairs[1..] {
        let (ps, pt) = *group.last().unwrap();
        if ti == pt + 1 && si == ps + 1 {
            group.push((si, ti));
        } else {
            clusters.push(make_cluster(source, &group, clusters.len()));
            group = vec![(si, ti)];
        }
    }
    clusters.push(make_cluster(source, &group, clusters.len()));
    clusters
}

fn make_cluster(source: &[u8], pairs: &[(usize, usize)], id: usize) -> ByteCluster {
    let s_min = pairs.iter().map(|&(s, _)| s).min().unwrap();
    let s_max = pairs.iter().map(|&(s, _)| s).max().unwrap();
    let t_min = pairs.first().unwrap().1;
    let t_max = pairs.last().unwrap().1;
    let byte_len = pairs.len();
    let density = byte_len as f64 / (s_max - s_min + 1) as f64;
    let src: Vec<usize> = pairs.iter().map(|&(s, _)| s).collect();
    let mut frag = 1;
    for i in 1..src.len() {
        if src[i] != src[i - 1] + 1 {
            frag += 1;
        }
    }
    let content =
        String::from_utf8_lossy(&src.iter().map(|&s| source[s]).collect::<Vec<_>>()).into_owned();
    ByteCluster {
        id,
        source_indices: src,
        source_range: (s_min, s_max),
        target_range: (t_min, t_max),
        byte_len,
        density,
        fragment_count: frag,
        content,
    }
}

pub fn phase3_segments(source: &[u8], target: &[u8], mapping: &MappingResult) -> Vec<Segment> {
    let mut segments = Vec::new();
    let n = source.len();
    let m = target.len();
    let mut mapped_src = vec![false; n];
    let mut mapped_tgt = vec![false; m];
    for (&si, tis) in &mapping.source_to_target {
        mapped_src[si] = true;
        for &ti in tis {
            mapped_tgt[ti] = true;
        }
    }
    let mut t = 0;
    while t < m {
        if mapped_tgt[t] {
            let start = t;
            while t < m && mapped_tgt[t] {
                t += 1;
            }
            let mut pairs: Vec<(usize, usize)> = Vec::new();
            for ti in start..t {
                if let Some(sis) = mapping.target_to_source.get(&ti) {
                    pairs.push((ti, sis[0]));
                }
            }
            if !pairs.is_empty() {
                let mut ss = 0;
                for i in 1..pairs.len() {
                    if pairs[i].1 != pairs[i - 1].1 + 1 {
                        push_seg(&mut segments, target, &pairs[ss..i]);
                        ss = i;
                    }
                }
                push_seg(&mut segments, target, &pairs[ss..]);
            }
        } else {
            t += 1;
        }
    }
    let mut i = 0;
    while i < n {
        if !mapped_src[i] {
            let s = i;
            while i < n && !mapped_src[i] {
                i += 1;
            }
            segments.push(Segment {
                id: 0,
                label: SegmentType::Missing,
                byte_len: i - s,
                source_range: Some((s, i - 1)),
                target_range: None,
                content: String::from_utf8_lossy(&source[s..i]).into_owned(),
            });
        } else {
            i += 1;
        }
    }
    let mut i = 0;
    while i < m {
        if !mapped_tgt[i] {
            let s = i;
            while i < m && !mapped_tgt[i] {
                i += 1;
            }
            segments.push(Segment {
                id: 0,
                label: SegmentType::Added,
                byte_len: i - s,
                source_range: None,
                target_range: Some((s, i - 1)),
                content: String::from_utf8_lossy(&target[s..i]).into_owned(),
            });
        } else {
            i += 1;
        }
    }
    segments.sort_by_key(|s| match s.label {
        SegmentType::Missing => s.source_range.map(|(v, _)| v).unwrap_or(0),
        _ => s.target_range.map(|(v, _)| v).unwrap_or(0),
    });
    for (idx, seg) in segments.iter_mut().enumerate() {
        seg.id = idx;
    }
    segments
}

fn push_seg(segs: &mut Vec<Segment>, target: &[u8], pairs: &[(usize, usize)]) {
    if pairs.is_empty() {
        return;
    }
    let ts = pairs.first().unwrap().0;
    let te = pairs.last().unwrap().0;
    let mut srcs: Vec<usize> = pairs.iter().map(|&(_, s)| s).collect();
    srcs.sort_unstable();
    segs.push(Segment {
        id: 0,
        label: SegmentType::Common,
        byte_len: te - ts + 1,
        source_range: Some((*srcs.first().unwrap(), *srcs.last().unwrap())),
        target_range: Some((ts, te)),
        content: String::from_utf8_lossy(&target[ts..=te]).into_owned(),
    });
}
