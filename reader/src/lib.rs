use db::DBReader;
use entry::Entry;
use pyo3::prelude::*;
use rand::seq::IteratorRandom;
use std::io::Result;
use std::sync::{Arc, Mutex, MutexGuard};
use std::{cmp::Reverse, collections::HashMap};

const N: usize = 3;
const THRESHOLD: usize = 5;

pub enum State {
    Unloaded(),
    Loading(),
    Loaded(bool),
    Error(),
}

#[pyclass]
struct Manager {
    readers: Vec<Arc<Mutex<Reader>>>,
}

#[pymethods]
impl Manager {
    #[new]
    fn new() -> Self {
        Self { readers: vec![] }
    }

    fn create(&mut self, path: &str, temp: &str, callback: Py<PyAny>) -> PyResult<Handle> {
        let reader = Arc::new(Mutex::new(Reader::new(path, temp)?));
        self.readers.push(reader.clone());
        Ok(Handle { reader, callback })
    }

    fn get(&mut self, py: Python, word: &str) -> Option<Entry> {
        py.detach(|| {
            for mut p in self.enabled() {
                match p.db.get(word) {
                    Some(e) => return Some(e),
                    _ => (),
                }
            }
            None
        })
    }

    fn find(&self, py: Python, target: &str) -> Option<String> {
        py.detach(|| {
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
        })
    }

    fn filter(&self, py: Python, word: &str, seps: Vec<char>) -> Vec<String> {
        py.detach(|| {
            let mut result: Vec<String> = Vec::new();
            for sep in seps {
                let words: Vec<&str> = word.split(sep).collect();
                if word.contains(sep) {
                    for &p in &words {
                        result.push(p.to_owned());
                    }
                }
                for p in self.enabled() {
                    for k in &p.candidates {
                        let k = k.as_str();
                        let keys: Vec<&str> = k.split(sep).collect();
                        if word != k
                            && keys.len() >= words.len()
                            && words.iter().all(|p| keys.contains(p))
                        {
                            result.push(k.to_owned());
                        }
                    }
                }
            }
            result
        })
    }

    fn random(&self) -> Option<String> {
        let mut rng = rand::rng();
        self.enabled()
            .choose(&mut rng)?
            .candidates
            .iter()
            .choose(&mut rng)
            .map(|s| s.to_string())
    }

    fn clear(&mut self) {
        self.readers.clear();
    }
}

impl Manager {
    fn enabled<'a>(&'a self) -> impl Iterator<Item = MutexGuard<'a, Reader>> {
        self.readers
            .iter()
            .filter_map(|p| p.lock().ok())
            .filter(|p| matches!(p.state, State::Loaded(true)))
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
struct Handle {
    reader: Arc<Mutex<Reader>>,
    callback: Py<PyAny>,
}

impl Handle {
    fn callback(&self, state: &State) {
        Python::attach(|py| {
            let _ = self.callback.call1(
                py,
                (match state {
                    State::Unloaded() => 0,
                    State::Loading() => 1,
                    State::Loaded(false) => 2,
                    State::Loaded(true) => 3,
                    State::Error() => 4,
                },),
            );
        });
    }
}

#[pymethods]
impl Handle {
    fn switch(&mut self, py: Python<'_>) -> PyResult<()> {
        py.detach(|| {
            let mut reader = self.reader.lock().unwrap();
            match reader.state {
                State::Unloaded() | State::Error() => {
                    reader.state = State::Loading();
                    self.callback(&reader.state);
                    reader.load()?;
                    reader.state = State::Loaded(true);
                }
                State::Loaded(l) => reader.state = State::Loaded(!l),
                _ => (),
            }
            self.callback(&reader.state);
            Ok(())
        })
    }

    fn update(&mut self) -> PyResult<()> {
        self.callback(&self.reader.lock().unwrap().state);
        Ok(())
    }

    #[getter]
    fn name(&self) -> String {
        self.reader.lock().unwrap().db.name.clone()
    }

    #[getter]
    fn name_zh(&self) -> String {
        self.reader.lock().unwrap().db.name_zh.clone()
    }

    fn __len__(&self) -> usize {
        self.reader.lock().unwrap().db.len()
    }
}

struct Reader {
    db: DBReader<Entry>,
    state: State,
    candidates: Vec<Arc<String>>,
    index: HashMap<u64, Vec<usize>>,
}

impl Reader {
    fn new(path: &str, temp: &str) -> Result<Self> {
        Ok(Reader {
            db: DBReader::from(path, temp)?,
            state: State::Unloaded(),
            candidates: Vec::new(),
            index: HashMap::new(),
        })
    }

    fn load(&mut self) -> Result<()> {
        self.db.load()?;
        for (id, c) in self.db.indexes.keys().enumerate() {
            ngram_signature(c).into_iter().for_each(|h| {
                self.index.entry(h).or_default().push(id);
            });
            self.candidates.push(c.clone());
        }
        Ok(())
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

#[pymodule]
fn reader(m: &Bound<PyModule>) -> PyResult<()> {
    m.add_class::<Manager>()?;
    m.add_class::<Handle>()?;
    m.add_class::<Entry>()?;
    Ok(())
}
