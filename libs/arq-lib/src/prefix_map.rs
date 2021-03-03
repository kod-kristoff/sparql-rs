use lazy_static::lazy_static;

use regex::Regex;

pub struct PrefixMap {
    map: Vec<(String, String)>,
}

impl PrefixMap {
    pub fn new() -> Self {
        Self { map: Vec::new() }
    } 

    pub fn from_str(data: &str) -> Self {
        let mut map = Self::new();
        map.scan_and_add(data);
        map
    }

    pub fn scan_and_add(&mut self, data: &str) {
        lazy_static! {
            static ref PREFIX: Regex = Regex::new(r"PREFIX (\w+): <([a-zA-Z/:#\.]+)>").unwrap();
        }

        if let Some(caps) = PREFIX.captures(data) {
            println!("{} - {}", &caps[1], &caps[2]);
            self.map.push(
                (
                    caps.get(1).unwrap().as_str().to_owned(),
                    caps.get(2).unwrap().as_str().to_owned()
                )
            );
        }
    }
    pub fn has_prefix(&self, prefix: &str) -> bool {
        contains_key(&self.map, prefix)
    }

    pub fn replace_with_prefix(&self, s: &str) -> String {
        for (prefix, value) in &self.map {
            if s[1..].starts_with(value) {
                let index = value.len() + 1;
                return format!("{}:{}", prefix, &s[index..s.len()-1]);
            }
        }
        s.to_owned()
    }
}

fn contains_key(map: &[(String, String)], key: &str) -> bool {
    for (k, _v) in map {
        if k == key {
            return true;
        }
    }
    false
}
#[cfg(test)]
mod tests {
    use super::*;

    mod creation {
        use super::*;

        #[test]
        fn create_from_str() {
            let data = "PREFIX ab: <http://example.com/ns/test#>";

            let prefix_map = PrefixMap::from_str(data);

            assert!(prefix_map.has_prefix("ab"));

            assert_eq!(prefix_map.replace_with_prefix("<http://example.com/ns/test#kristoff>"), "ab:kristoff");
        }
    }

    #[test]
    fn replace_with_prefix_lets_through() {
        let prefix_map = PrefixMap::new();

        assert_eq!(prefix_map.replace_with_prefix("tyu"), "tyu");
    }
}
