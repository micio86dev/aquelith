//! Prototype P1: persistent ids, content hashes and validated atomic patches.
//!
//! Toy model of ADR-0002 (source authority) and ADR-0004 (stable identities).
//! Not part of the AQELITH compiler. The hash is FNV-1a: it is NOT
//! collision-resistant and must be replaced before any integrity use.

use std::collections::BTreeMap;

const FNV_OFFSET: u64 = 0xcbf2_9ce4_8422_2325;
const FNV_PRIME: u64 = 0x0000_0100_0000_01b3;

fn fnv1a(bytes: &[u8]) -> u64 {
    bytes.iter().fold(FNV_OFFSET, |h, b| (h ^ u64::from(*b)).wrapping_mul(FNV_PRIME))
}

/// Opaque identity assigned once, never derived from position or name, never reused.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct PersistentId(u64);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Ty {
    I32,
    U32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Node {
    pub name: String,
    pub ty: Ty,
    pub value: i64,
}

impl Node {
    /// Content hash used for staleness checks. Changes when name, type or value change.
    pub fn content_hash(&self) -> u64 {
        let mut bytes = self.name.as_bytes().to_vec();
        bytes.push(0);
        bytes.push(match self.ty {
            Ty::I32 => 1,
            Ty::U32 => 2,
        });
        bytes.extend_from_slice(&self.value.to_le_bytes());
        fnv1a(&bytes)
    }
}

/// Program store: nodes keyed by persistent id, plus program order (the text position).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Graph {
    next_id: u64,
    nodes: BTreeMap<PersistentId, Node>,
    order: Vec<PersistentId>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Op {
    Rename { target: PersistentId, expect_name: String, to: String },
    SetValue { target: PersistentId, expect_value: i64, to: i64 },
    Insert { at: usize, name: String, ty: Ty, value: i64 },
    Delete { target: PersistentId, expect_hash: u64 },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Patch {
    pub base: u64,
    pub ops: Vec<Op>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Rejection {
    /// The patch was built against a different graph state.
    Stale { expected_base: u64, actual_base: u64 },
    UnknownTarget(PersistentId),
    /// Precondition on op `op` failed; nothing was applied.
    PreconditionFailed { op: usize, detail: String },
    /// Validation after op `op` failed; nothing was applied.
    Invalid { op: usize, code: &'static str, detail: String },
}

impl Graph {
    pub fn new() -> Self {
        Graph { next_id: 1, nodes: BTreeMap::new(), order: Vec::new() }
    }

    /// Host-side setup that bypasses patches. Tests use it to build fixtures.
    pub fn insert_direct(&mut self, at: usize, name: &str, ty: Ty, value: i64) -> PersistentId {
        let id = PersistentId(self.next_id);
        self.next_id += 1;
        self.nodes.insert(id, Node { name: name.to_string(), ty, value });
        self.order.insert(at, id);
        id
    }

    pub fn ids(&self) -> Vec<PersistentId> {
        self.order.clone()
    }

    pub fn node(&self, id: PersistentId) -> Option<&Node> {
        self.nodes.get(&id)
    }

    /// Position-derived id, the scheme Zerolang uses (per its audit). Shown only to compare.
    pub fn positional_id(&self, id: PersistentId) -> Option<String> {
        self.order.iter().position(|x| *x == id).map(|i| format!("pos:{i}"))
    }

    /// Hash over program order and node contents: the observable program state.
    pub fn graph_hash(&self) -> u64 {
        let mut bytes = Vec::new();
        for id in &self.order {
            let node = &self.nodes[id];
            bytes.extend_from_slice(&node.content_hash().to_le_bytes());
        }
        fnv1a(&bytes)
    }

    /// Check the patch and return the resulting hash without changing the store.
    pub fn dry_run(&self, patch: &Patch) -> Result<u64, Rejection> {
        self.plan(patch).map(|next| next.graph_hash())
    }

    /// Apply all ops or none. Returns the new graph hash.
    pub fn apply(&mut self, patch: &Patch) -> Result<u64, Rejection> {
        let next = self.plan(patch)?;
        let hash = next.graph_hash();
        *self = next;
        Ok(hash)
    }

    fn plan(&self, patch: &Patch) -> Result<Graph, Rejection> {
        let current = self.graph_hash();
        if patch.base != current {
            return Err(Rejection::Stale { expected_base: patch.base, actual_base: current });
        }
        let mut next = self.clone();
        for (i, op) in patch.ops.iter().enumerate() {
            next.apply_op(i, op)?;
            next.validate(i)?;
        }
        Ok(next)
    }

    fn apply_op(&mut self, index: usize, op: &Op) -> Result<(), Rejection> {
        match op {
            Op::Rename { target, expect_name, to } => {
                let node = self.nodes.get_mut(target).ok_or(Rejection::UnknownTarget(*target))?;
                if &node.name != expect_name {
                    return Err(Rejection::PreconditionFailed {
                        op: index,
                        detail: format!("name is {:?}, expected {:?}", node.name, expect_name),
                    });
                }
                node.name = to.clone();
            }
            Op::SetValue { target, expect_value, to } => {
                let node = self.nodes.get_mut(target).ok_or(Rejection::UnknownTarget(*target))?;
                if node.value != *expect_value {
                    return Err(Rejection::PreconditionFailed {
                        op: index,
                        detail: format!("value is {}, expected {}", node.value, expect_value),
                    });
                }
                node.value = *to;
            }
            Op::Insert { at, name, ty, value } => {
                if *at > self.order.len() {
                    return Err(Rejection::PreconditionFailed {
                        op: index,
                        detail: format!("position {at} out of range"),
                    });
                }
                let id = PersistentId(self.next_id);
                self.next_id += 1;
                self.nodes.insert(id, Node { name: name.clone(), ty: *ty, value: *value });
                self.order.insert(*at, id);
            }
            Op::Delete { target, expect_hash } => {
                let node = self.nodes.get(target).ok_or(Rejection::UnknownTarget(*target))?;
                if node.content_hash() != *expect_hash {
                    return Err(Rejection::PreconditionFailed {
                        op: index,
                        detail: "content hash differs".to_string(),
                    });
                }
                self.nodes.remove(target);
                self.order.retain(|x| x != target);
                // next_id is never decremented: ids are not reused.
            }
        }
        Ok(())
    }

    /// Stand-in for the full checker. The same check runs for every op, so no path is weaker.
    fn validate(&self, op: usize) -> Result<(), Rejection> {
        for node in self.nodes.values() {
            if node.ty == Ty::U32 && !(0..=i64::from(u32::MAX)).contains(&node.value) {
                return Err(Rejection::Invalid {
                    op,
                    code: "TYP-U32-RANGE",
                    detail: format!("{} is not a u32: {}", node.name, node.value),
                });
            }
        }
        Ok(())
    }
}

impl Default for Graph {
    fn default() -> Self {
        Self::new()
    }
}
