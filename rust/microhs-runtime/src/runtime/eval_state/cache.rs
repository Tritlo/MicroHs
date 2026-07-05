//! Cached roots for primitive and compound nodes.
use super::*;

#[derive(Clone, Debug, Default)]
pub(in crate::runtime) struct PrimCache {
    pub(in crate::runtime) a: Option<NodeId>,
    pub(in crate::runtime) b: Option<NodeId>,
    pub(in crate::runtime) c: Option<NodeId>,
    pub(in crate::runtime) i: Option<NodeId>,
    pub(in crate::runtime) k: Option<NodeId>,
    pub(in crate::runtime) k2: Option<NodeId>,
    pub(in crate::runtime) k3: Option<NodeId>,
    pub(in crate::runtime) o: Option<NodeId>,
    pub(in crate::runtime) p: Option<NodeId>,
    pub(in crate::runtime) u: Option<NodeId>,
    pub(in crate::runtime) y: Option<NodeId>,
    pub(in crate::runtime) z: Option<NodeId>,
    pub(in crate::runtime) io_bind: Option<NodeId>,
    pub(in crate::runtime) io_perform_io: Option<NodeId>,
}

#[derive(Clone, Debug, Default)]
pub(in crate::runtime) struct CompoundCache {
    pub(in crate::runtime) fst: Option<NodeId>,
    pub(in crate::runtime) snd: Option<NodeId>,
    pub(in crate::runtime) just: Option<NodeId>,
    pub(in crate::runtime) pair_unit: Option<NodeId>,
}

pub(in crate::runtime) fn small_int_index(value: i64) -> Option<usize> {
    if (SMALL_INT_MIN..=SMALL_INT_MAX).contains(&value) {
        Some((value - SMALL_INT_MIN) as usize)
    } else {
        None
    }
}
