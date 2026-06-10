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
        self.source_maps
            .read()
            .expect("Lock")
            .get(path)?
            .map
            .get(&(line, col))
            .cloned()
    }
}

pub struct SourceMap {
    map: HashMap<(u32, u32), (u32, u32)>,
}

impl FromIterator<((u32, u32), (u32, u32))> for SourceMap {
    fn from_iter<T: IntoIterator<Item = ((u32, u32), (u32, u32))>>(iter: T) -> Self {
        SourceMap {
            map: iter.into_iter().collect(),
        }
    }
}
