use std::{collections::HashMap, sync::RwLock};

pub struct SourceMaps {
    source_maps: RwLock<HashMap<String, SourceMap>>,
}

impl SourceMaps {
    pub fn new() -> Self {
        Self {
            source_maps: RwLock::new(HashMap::new()),
        }
    }

    pub fn insert(&self, path: String, map: SourceMap) {
        self.source_maps.write().expect("Lock").insert(path, map);
    }

    pub fn lookup(&self, path: &str, line: u32, col: u32) -> Option<(u32, u32)> {
        let lock = self.source_maps.read().expect("Lock");

        let sourcemap = lock.get(path)?;

        let (_, raw) = greatest_lower_bound(&sourcemap.map, &(line - 1, col - 1), |t| {
            (t.dst_line, t.dst_col)
        })?;

        Some((raw.src_line, raw.src_col))
    }
}

#[derive(Debug)]
pub struct Mapping {
    pub src_line: u32,
    pub src_col: u32,
    pub dst_line: u32,
    pub dst_col: u32,
}

#[derive(Debug)]
pub struct SourceMap {
    map: Vec<Mapping>,
}

impl FromIterator<((u32, u32), (u32, u32))> for SourceMap {
    fn from_iter<T: IntoIterator<Item = ((u32, u32), (u32, u32))>>(iter: T) -> Self {
        SourceMap {
            map: iter
                .into_iter()
                .map(|((src_line, src_col), (dst_line, dst_col))| Mapping {
                    src_line,
                    src_col,
                    dst_line,
                    dst_col,
                })
                .collect(),
        }
    }
}

pub fn greatest_lower_bound<'a, T, K: Ord, F: Fn(&'a T) -> K>(
    slice: &'a [T],
    key: &K,
    map: F,
) -> Option<(usize, &'a T)> {
    let mut idx = match slice.binary_search_by_key(key, &map) {
        Ok(index) => index,
        Err(index) => {
            // If there is no match, then we know for certain that the index is where we
            // should insert a new token, and that the token directly before is
            // the greatest lower bound.
            return slice.get(index.checked_sub(1)?).map(|res| (index, res));
        }
    };

    // If we get an exact match, then we need to continue looking at previous tokens
    // to see if they also match. We use a linear search because the number of
    // exact matches is generally very small, and almost certainly smaller than
    // the number of tokens before the index.
    for i in (0..idx).rev() {
        if map(&slice[i]) == *key {
            idx = i;
        } else {
            break;
        }
    }
    slice.get(idx).map(|res| (idx, res))
}
