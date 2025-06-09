use rand::seq::IteratorRandom;
use std::{
    cmp::Reverse,
    collections::HashMap,
    sync::{Arc, Mutex, MutexGuard},
};

const N: usize = 3;
const THRESHOLD: usize = 5;

pub struct Part {
    pub enabled: bool,
    candidates: Vec<Arc<String>>,
    index: HashMap<u64, Vec<usize>>,
}

impl Part {
    pub fn new(candidates: Vec<Arc<String>>) -> Self {
        let mut index: HashMap<u64, Vec<usize>> = HashMap::new();
        for (id, c) in candidates.iter().enumerate() {
            ngram_signature(c).into_iter().for_each(|h| {
                index.entry(h).or_default().push(id);
            });
        }
        Self {
            enabled: true,
            candidates,
            index,
        }
    }

    fn count(&self, counter: &mut HashMap<Arc<String>, usize>, signature: &Vec<u64>) {
        signature
            .iter()
            .filter_map(|h| self.index.get(h))
            .flatten()
            .for_each(|&id| {
                let cand = self.candidates[id].clone();
                *counter.entry(cand).or_insert(0) += 1;
            });
    }
}

pub struct Matcher {
    parts: Vec<Arc<Mutex<Part>>>,
}

impl Matcher {
    pub fn new() -> Self {
        Self { parts: vec![] }
    }

    pub fn add(&mut self, candidates: Vec<Arc<String>>) -> Arc<Mutex<Part>> {
        let part = Arc::new(Mutex::new(Part::new(candidates)));
        self.parts.push(part.clone());
        part
    }

    pub fn find(&self, target: &str) -> Option<String> {
        let mut counter: HashMap<Arc<String>, usize> = HashMap::new();
        let signature = ngram_signature(target);
        for p in self.enabled() {
            p.count(&mut counter, &signature);
        }
        let mut v: Vec<(Arc<String>, usize)> = counter.into_iter().collect();
        v.sort_unstable_by_key(|&(_, c)| Reverse(c));
        v.truncate(30);
        v.into_iter()
            .filter_map(|(c, _)| levenshtein_distance(target, &c).map(|d| (d, c)))
            .min_by_key(|&(d, _)| d)
            .and_then(|(d, c)| (d < THRESHOLD).then_some(c.to_string()))
    }

    pub fn random(&self) -> Option<String> {
        let mut rng = rand::rng();
        self.enabled()
            .choose(&mut rng)?
            .candidates
            .iter()
            .choose(&mut rng)
            .map(|s| s.to_string())
    }

    fn enabled<'a>(&'a self) -> impl Iterator<Item = MutexGuard<'a, Part>> {
        self.parts
            .iter()
            .filter_map(|p| p.lock().ok())
            .filter(|p| p.enabled)
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
