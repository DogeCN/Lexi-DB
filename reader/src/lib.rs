use db::DBReader;
use entry::Entry;
use pyo3::prelude::*;
use rand::seq::IteratorRandom;
use std::sync::{Arc, Mutex, MutexGuard};
use std::{cmp::Reverse, collections::HashMap};

const N: usize = 3;
const THRESHOLD: usize = 5;

#[pyclass]
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum DataState {
    Unloaded,
    Loading,
    Loaded,
    Error,
}

#[pyclass]
pub struct State {
    pub enabled: bool,
    pub data_state: DataState,
}

impl State {
    pub fn new() -> Self {
        State {
            enabled: false,
            data_state: DataState::Unloaded,
        }
    }
}

pub struct Part {
    pub state: State,
    candidates: Vec<Arc<String>>,
    index: HashMap<u64, Vec<usize>>,
}

impl Part {
    pub fn new(candidates: Vec<Arc<String>>, state: &State) -> Self {
        let mut index: HashMap<u64, Vec<usize>> = HashMap::new();
        for (id, c) in candidates.iter().enumerate() {
            ngram_signature(c).into_iter().for_each(|h| {
                index.entry(h).or_default().push(id);
            });
        }
        Self {
            state: State {
                enabled: state.enabled,
                data_state: state.data_state,
            },
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
                *counter.entry(self.candidates[id].clone()).or_insert(0) += 1;
            });
    }
}

pub struct Matcher {
    parts: Vec<Mutex<Part>>,
}

impl Matcher {
    pub fn new() -> Self {
        Self { parts: vec![] }
    }

    pub fn add(&mut self, candidates: Vec<Arc<String>>, state: &State) {
        let part = Mutex::new(Part::new(candidates, state));
        self.parts.push(part);
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
            .filter_map(|(c, _)| levenshtein_distance(target, c.as_str()).map(|d| (d, c)))
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
            .filter(|p| p.state.enabled)
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

#[pyclass]
struct PyDBReader {
    db: Mutex<DBReader<Entry>>,
    #[pyo3(get)]
    path: String,
    enabled: bool,
    data_state: DataState,
}

#[pymethods]
impl PyDBReader {
    #[new]
    fn new(path: &str, temp: &str) -> PyResult<Self> {
        Ok(PyDBReader {
            db: Mutex::new(DBReader::from(path, temp)?),
            path: path.to_string(),
            enabled: false,
            data_state: DataState::Unloaded,
        })
    }

    fn enabled(&self) -> bool {
        self.enabled
    }

    fn set_enabled(&mut self, enabled: bool) {
        if enabled && self.data_state != DataState::Loaded {
            self.data_state = DataState::Loading;
            // 不能在 setter 里用 py.allow_threads，只能在 load 方法里做
        }
        self.enabled = enabled;
    }

    // 新增 load 方法，带 py 参数，做真正的加载
    fn load(&mut self, py: Python<'_>) {
        if self.data_state != DataState::Loaded {
            self.data_state = DataState::Loading;
            let result = py.allow_threads(|| self.db.lock().unwrap().load().is_ok());
            if result {
                self.data_state = DataState::Loaded;
                self.enabled = true;
            } else {
                self.data_state = DataState::Error;
                self.enabled = false;
            }
        }
    }

    fn data_state(&self) -> DataState {
        self.data_state
    }

    fn name(&self) -> String {
        if self.data_state != DataState::Loaded {
            return String::new();
        }
        self.db.lock().unwrap().name.clone()
    }

    fn name_zh(&self) -> String {
        if self.data_state != DataState::Loaded {
            return String::new();
        }
        self.db.lock().unwrap().name_zh.clone()
    }

    fn filter(&self, _py: Python<'_>, word: &str, seps: Vec<char>) -> Vec<String> {
        if self.data_state != DataState::Loaded {
            return vec![];
        }
        self.db.lock().unwrap().filter_keys(word, &seps)
    }

    fn __getitem__(&mut self, key: &str) -> Option<Entry> {
        if self.data_state != DataState::Loaded {
            return None;
        }
        self.db.lock().unwrap().get(key).map(|e| e.into())
    }

    fn __len__(&self) -> usize {
        if self.data_state != DataState::Loaded {
            return 0;
        }
        self.db.lock().unwrap().len()
    }

    fn __contains__(&self, key: &str) -> bool {
        if self.data_state != DataState::Loaded {
            return false;
        }
        self.db.lock().unwrap().contains(key)
    }
}

#[pyclass]
struct PyMatcher {
    matcher: Matcher,
}

#[pymethods]
impl PyMatcher {
    #[new]
    fn new() -> Self {
        PyMatcher {
            matcher: Matcher::new(),
        }
    }

    fn combine(&mut self, py: Python<'_>, reader: &mut PyDBReader) {
        let state = State {
            enabled: reader.enabled,
            data_state: reader.data_state,
        };
        py.allow_threads(|| {
            if let Ok(g) = reader.db.lock() {
                self.matcher.add(g.keys(), &state);
            }
        });
    }

    fn find(&self, py: Python<'_>, word: &str) -> Option<String> {
        py.allow_threads(|| self.matcher.find(word))
    }

    fn random(&self) -> Option<String> {
        self.matcher.random()
    }
}

#[pymodule]
fn reader(m: &Bound<PyModule>) -> PyResult<()> {
    m.add_class::<PyMatcher>()?;
    Ok(())
}
