use std::{
    cmp::Reverse,
    collections::HashMap,
    sync::{Arc, Mutex},
};

const N: usize = 3;
const THRESHOLD: usize = 5;

pub struct Part {
    pub enabled: bool,
    index: HashMap<u64, Vec<usize>>,
}

impl Part {
    pub fn new(candidates: &[Arc<String>], offset: usize) -> Self {
        let mut index: HashMap<u64, Vec<usize>> = HashMap::new();
        candidates.iter().enumerate().for_each(|(id, c)| {
            ngram_signature(c).into_iter().for_each(|h| {
                index.entry(h).or_default().push(id + offset);
            });
        });
        Self {
            enabled: true,
            index,
        }
    }

    fn count(&self, counter: &mut HashMap<usize, usize>, signature: &Vec<u64>) {
        signature
            .into_iter()
            .filter_map(|h| self.index.get(h))
            .flatten()
            .for_each(|&id| {
                *counter.entry(id).or_insert(0) += 1;
            });
    }
}

pub struct Matcher {
    candidates: Vec<Arc<String>>,
    parts: Vec<Arc<Mutex<Part>>>,
    count: usize,
}

impl Matcher {
    pub fn new() -> Self {
        Self {
            candidates: vec![],
            parts: vec![],
            count: 0,
        }
    }

    pub fn add(&mut self, candidates: Vec<Arc<String>>) -> Arc<Mutex<Part>> {
        let part = Arc::new(Mutex::new(Part::new(&candidates, self.count)));
        self.count += candidates.len();
        self.candidates.extend(candidates);
        self.parts.push(part.clone());
        part
    }

    pub fn find<'a>(&'a self, target: &str) -> Option<&'a str> {
        let mut counter = HashMap::new();
        let signature = ngram_signature(target);
        self.parts
            .iter()
            .filter_map(|p| p.lock().ok())
            .filter(|p| p.enabled)
            .for_each(|p| p.count(&mut counter, &signature));
        let indices: Vec<usize> = if counter.is_empty() {
            (0..self.candidates.len()).collect()
        } else {
            let mut v: Vec<(usize, usize)> = counter.into_iter().collect();
            v.sort_unstable_by_key(|&(_, c)| Reverse(c));
            v.into_iter().take(30).map(|(i, _)| i).collect()
        };
        indices
            .into_iter()
            .filter_map(|i| {
                let cand = self.candidates[i].as_str();
                levenshtein_distance(target, cand).map(|d| (d, cand))
            })
            .min_by_key(|&(d, _)| d)
            .and_then(|(d, cand)| (d < THRESHOLD).then_some(cand))
    }
}

fn ngram_signature(s: &str) -> Vec<u64> {
    let chars: Vec<char> = s.chars().collect();
    (chars.len() < N)
        .then(|| chars.iter().map(|&c| c as u64).collect())
        .unwrap_or_else(|| {
            (0..=chars.len() - N)
                .map(|i| {
                    (0..N).fold(0u64, |h, j| {
                        h.wrapping_mul(31).wrapping_add(chars[i + j] as u64)
                    })
                })
                .collect()
        })
}

fn levenshtein_distance(s1: &str, s2: &str) -> Option<usize> {
    let (l1, l2) = (s1.chars().count(), s2.chars().count());
    let (long, short, max, min) = (l1 < l2)
        .then_some((s2, s1, l2, l1))
        .unwrap_or((s1, s2, l1, l2));
    match (max, min) {
        (max, 0) => (max > THRESHOLD).then_some(max),
        (max, min) if max - min > THRESHOLD => None,
        (_, min) => {
            let mut prev: Vec<usize> = (0..=min).collect();
            let mut curr = vec![0; min + 1];
            let shorts: Vec<char> = short.chars().collect();
            for (i, lc) in long.chars().enumerate() {
                curr[0] = i + 1;
                let mut mini = curr[0];
                for (j, &sc) in shorts.iter().enumerate() {
                    curr[j + 1] = (prev[j + 1] + 1)
                        .min(curr[j] + 1)
                        .min(prev[j] + (sc != lc) as usize);
                    mini = mini.min(curr[j + 1]);
                }
                if mini > THRESHOLD {
                    return None;
                }
                std::mem::swap(&mut prev, &mut curr);
            }
            (prev[min] < THRESHOLD).then_some(prev[min])
        }
    }
}
