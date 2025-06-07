use std::collections::HashMap;
use std::ops::AddAssign;
use std::sync::Arc;

pub struct Matcher {
    candidates: Vec<Arc<String>>,
    index: HashMap<u64, Vec<usize>>,
    n: usize,
}

impl Matcher {
    pub fn new(candidates: Vec<Arc<String>>) -> Self {
        let n = 2;
        let mut index: HashMap<u64, Vec<usize>> = HashMap::new();
        for (idx, cand) in candidates.iter().enumerate() {
            for hash in Self::ngram_signature(cand, n) {
                index.entry(hash).or_default().push(idx);
            }
        }
        Self {
            candidates,
            index,
            n,
        }
    }

    pub fn find<'a>(&'a self, target: &str) -> Option<&'a str> {
        let mut counter: HashMap<usize, usize> = HashMap::new();
        Self::ngram_signature(target, self.n)
            .iter()
            .filter_map(|hash| self.index.get(hash))
            .flat_map(|indices| indices.iter())
            .for_each(|idx| {
                counter
                    .entry(*idx)
                    .and_modify(|e| e.add_assign(1))
                    .or_insert(1);
            });
        counter
            .is_empty()
            .then(|| (0..self.candidates.len()).collect::<Vec<_>>())
            .unwrap_or_else(|| {
                let mut v: Vec<_> = counter.into_iter().collect();
                v.sort_unstable_by_key(|&(_, c)| std::cmp::Reverse(c));
                v.into_iter().take(20).map(|(i, _)| i).collect()
            })
            .into_iter()
            .map(|i| self.candidates[i].as_str())
            .min_by_key(|&cand| Self::levenshtein_distance(target, cand))
    }

    fn ngram_signature(s: &str, n: usize) -> Vec<u64> {
        let mut sig = Vec::new();
        let chars: Vec<char> = s.chars().collect();
        if chars.len() < n {
            return sig;
        }
        for i in 0..=chars.len() - n {
            let mut hash = 0u64;
            for j in 0..n {
                hash = hash.wrapping_mul(31).wrapping_add(chars[i + j] as u64);
            }
            sig.push(hash);
        }
        sig
    }

    fn levenshtein_distance(s1: &str, s2: &str) -> usize {
        let (l1, l2) = (s1.chars().count(), s2.chars().count());
        let (long, short, max, min) = (l1 < l2)
            .then(|| (s2, s1, l2, l1))
            .unwrap_or_else(|| (s1, s2, l1, l2));
        match (max, min) {
            (max, 0) => max,
            (max, min) if max - min > 3 => max,
            (_, min) => {
                let mut prev: Vec<usize> = (0..=min).collect();
                let mut curr = vec![0; min + 1];
                let shorts = short.chars().enumerate();
                for (i, lc) in long.chars().enumerate() {
                    curr[0] = i + 1;
                    for (j, sc) in shorts.clone() {
                        curr[j + 1] = (prev[j + 1] + 1)
                            .min(curr[j] + 1)
                            .min(prev[j] + (sc != lc).then_some(1).unwrap_or_default());
                    }
                    std::mem::swap(&mut prev, &mut curr);
                }
                prev[min]
            }
        }
    }
}
