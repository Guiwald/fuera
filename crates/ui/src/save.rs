//! Saved progress and settings: a plain `key=value` text, written by the host where it can (file, browser storage).

use std::collections::BTreeMap;

#[derive(Clone, Debug, Default)]
pub struct Save {
    map: BTreeMap<String, String>,
    pub dirty: bool,
}

impl Save {
    pub fn parse(text: &str) -> Save {
        let mut map = BTreeMap::new();
        for line in text.lines() {
            if let Some((k, v)) = line.split_once('=') {
                map.insert(k.trim().to_string(), v.trim().to_string());
            }
        }
        Save { map, dirty: false }
    }

    pub fn to_text(&self) -> String {
        let mut s = String::new();
        for (k, v) in &self.map {
            s.push_str(k);
            s.push('=');
            s.push_str(v);
            s.push('\n');
        }
        s
    }

    pub fn get(&self, k: &str) -> Option<&str> {
        self.map.get(k).map(|s| s.as_str())
    }

    pub fn get_u32(&self, k: &str) -> u32 {
        self.get(k).and_then(|v| v.parse().ok()).unwrap_or(0)
    }

    pub fn get_bool(&self, k: &str, default: bool) -> bool {
        self.get(k).map(|v| v == "1").unwrap_or(default)
    }

    pub fn set(&mut self, k: &str, v: impl ToString) {
        let v = v.to_string();
        if self.map.get(k) != Some(&v) {
            self.map.insert(k.to_string(), v);
            self.dirty = true;
        }
    }

    pub fn keys_with_prefix<'a>(&'a self, p: &'a str) -> impl Iterator<Item = (&'a str, &'a str)> + 'a {
        self.map.range(p.to_string()..).take_while(move |(k, _)| k.starts_with(p)).map(|(k, v)| (k.as_str(), v.as_str()))
    }
}
