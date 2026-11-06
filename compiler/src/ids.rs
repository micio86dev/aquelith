//! Persistent ids for functions (ADR-0004, docs/compiler/STABLE_IDENTITIES.md §2–§4).
//!
//! Slice: functions only. An id is opaque, random (128 bits from the standard library's randomly
//! seeded hasher, so no platform device is needed), assigned once and not reused in practice (128-bit random space; no explicit check). An id is an
//! identity, not a security token (STABLE_IDENTITIES §7). It is stored in a sidecar index `<file>.ids.json`, because the
//! source language has no comments (ADR-0010) and must not carry metadata. A rename keeps the id;
//! a function that disappears loses its entry; its id is not reused in practice (no explicit check).
//! Not built: moves between modules, ids for nodes below function level, merge tooling (§6).

use crate::ast::Module;
use crate::diag::jstr;
use crate::json::{self, Json};

pub const API: &str = "aqelith.ids.v1";

/// Index entries as (id, name), in the order the functions appear in the module.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct IdIndex {
    entries: Vec<(String, String)>,
}

/// 128 bits as 32 lowercase hex digits from randomly seeded hashers. Portable (no platform device).
/// Not a cryptographic key generator: the standard library does not promise unpredictable output.
pub fn random_id() -> std::io::Result<String> {
    use std::collections::hash_map::RandomState;
    use std::hash::{BuildHasher, Hasher};
    let mut out = String::with_capacity(32);
    for word in 0u64..2 {
        let mut hasher = RandomState::new().build_hasher();
        hasher.write_u64(word);
        out.push_str(&format!("{:016x}", hasher.finish()));
    }
    Ok(out)
}

impl IdIndex {
    pub fn entries(&self) -> &[(String, String)] {
        &self.entries
    }

    pub fn id_of(&self, name: &str) -> Option<&str> {
        self.entries
            .iter()
            .find(|(_, n)| n == name)
            .map(|(id, _)| id.as_str())
    }

    pub fn name_of(&self, id: &str) -> Option<&str> {
        self.entries
            .iter()
            .find(|(i, _)| i == id)
            .map(|(_, n)| n.as_str())
    }

    /// Keeps the id of `old` and gives it the name `new`. Returns false if `old` is not indexed.
    pub fn rename(&mut self, old: &str, new: &str) -> bool {
        match self.entries.iter_mut().find(|(_, n)| n == old) {
            Some(entry) => {
                entry.1 = new.to_string();
                true
            }
            None => false,
        }
    }

    /// Brings the index in line with the module: keeps ids of functions still present, drops the
    /// entries of functions that are gone, and gives each new function a fresh id from `fresh`.
    pub fn sync(&mut self, m: &Module, fresh: &mut dyn FnMut() -> String) {
        let names: Vec<&str> = m.items.iter().map(|f| f.name.as_str()).collect();
        self.entries.retain(|(_, n)| names.contains(&n.as_str()));
        for name in names {
            if self.id_of(name).is_none() {
                let id = fresh();
                self.entries.push((id, name.to_string()));
            }
        }
    }

    /// Index for a module with no history: every function gets a fresh id.
    pub fn for_module(m: &Module, fresh: &mut dyn FnMut() -> String) -> IdIndex {
        let mut index = IdIndex::default();
        index.sync(m, fresh);
        index
    }

    pub fn to_json(&self) -> String {
        let items: Vec<String> = self
            .entries
            .iter()
            .map(|(id, name)| format!("{{\"id\":{},\"name\":{}}}", jstr(id), jstr(name)))
            .collect();
        format!(
            "{{\"api\":{},\"functions\":[{}]}}\n",
            jstr(API),
            items.join(",")
        )
    }

    /// Reads an index file. Rejects a wrong api version, malformed ids, and duplicate ids or names.
    pub fn from_json(text: &str) -> Result<IdIndex, String> {
        let doc = json::parse(text).map_err(|e| format!("invalid JSON: {e}"))?;
        if doc.get("api").and_then(Json::as_str) != Some(API) {
            return Err(format!("api must be \"{API}\""));
        }
        let items = doc
            .get("functions")
            .and_then(Json::as_array)
            .ok_or("functions must be an array")?;
        let mut index = IdIndex::default();
        for item in items {
            let id = item
                .get("id")
                .and_then(Json::as_str)
                .ok_or("each entry needs a string id")?;
            let name = item
                .get("name")
                .and_then(Json::as_str)
                .ok_or("each entry needs a string name")?;
            if id.len() != 32
                || !id
                    .bytes()
                    .all(|b| b.is_ascii_hexdigit() && !b.is_ascii_uppercase())
            {
                return Err(format!("malformed id '{id}'"));
            }
            if index.name_of(id).is_some() || index.id_of(name).is_some() {
                return Err(format!("duplicate id or name for '{name}'"));
            }
            index.entries.push((id.to_string(), name.to_string()));
        }
        Ok(index)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn counter() -> impl FnMut() -> String {
        let mut n = 0u128;
        move || {
            n += 1;
            format!("{n:032x}")
        }
    }

    #[test]
    fn json_round_trips() {
        let mut next = counter();
        let mut index = IdIndex::default();
        let m = Module {
            name: "m".into(),
            records: vec![],
            items: vec![],
            pages: vec![],
        };
        index.sync(&m, &mut next);
        assert_eq!(IdIndex::from_json(&index.to_json()).unwrap(), index);
    }

    #[test]
    fn malformed_files_are_rejected() {
        assert!(IdIndex::from_json("{}").is_err());
        assert!(IdIndex::from_json(
            r#"{"api":"aqelith.ids.v1","functions":[{"id":"XY","name":"f"}]}"#
        )
        .is_err());
        let dup = format!(
            r#"{{"api":"aqelith.ids.v1","functions":[{{"id":"{0}","name":"f"}},{{"id":"{0}","name":"g"}}]}}"#,
            "0".repeat(32)
        );
        assert!(IdIndex::from_json(&dup).is_err());
    }

    #[test]
    fn random_ids_are_32_hex_digits_and_differ() {
        let a = random_id().unwrap();
        let b = random_id().unwrap();
        assert_eq!(a.len(), 32);
        assert!(a
            .bytes()
            .all(|b| b.is_ascii_hexdigit() && !b.is_ascii_uppercase()));
        assert_ne!(a, b);
    }
}
