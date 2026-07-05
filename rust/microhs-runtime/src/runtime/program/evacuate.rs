//! Feature-gated nursery evacuation scaffold for moving GC.

use super::*;

impl Program {
    pub(in crate::runtime) fn evacuate_marked_nursery_for_moving_gc(
        &mut self,
        marked: &[bool],
        nursery_start: usize,
        current_root: &mut NodeId,
        frame_stack: &mut EvalFrameStack,
        eval_spine: &mut EvalSpine,
        persistent_spine: &mut PersistentSpine,
        scratch_args: &mut [NodeId],
        scratch_apps: &mut [NodeId],
        machine_stack: Option<&mut EvalStack>,
    ) -> usize {
        assert_eq!(
            marked.len(),
            self.nodes.len(),
            "moving GC mark bitmap must match heap size"
        );

        let old_len = self.nodes.len();
        let nursery_start = nursery_start.min(old_len);
        if nursery_start == old_len {
            #[cfg(feature = "moving-gc")]
            {
                self.gc_nursery_start = old_len;
            }
            return 0;
        }

        let old_nodes = std::mem::take(&mut self.nodes);
        let mut young_remap = HashMap::new();
        let mut new_nodes = Vec::with_capacity(old_len);
        new_nodes.extend_from_slice(&old_nodes[..nursery_start]);

        for (index, cell) in old_nodes.iter().copied().enumerate().skip(nursery_start) {
            if !marked[index] {
                if let Some(cold) = cell.cold_index() {
                    if let Some(slot) = self.cold_nodes.get_mut(cold) {
                        *slot = None;
                    }
                }
                continue;
            }
            debug_assert_ne!(
                cell.tag(),
                CellTag::Free,
                "moving GC cannot evacuate a free cell as live"
            );
            let new_id = NodeId::from_index(new_nodes.len());
            young_remap.insert(index, new_id);
            new_nodes.push(cell);
        }

        self.nodes = new_nodes;
        self.free_head = None;
        self.free_nodes = 0;

        self.labels
            .retain(|_, id| id.index() < nursery_start || young_remap.contains_key(&id.index()));
        self.weak_nodes
            .retain(|id| id.index() < nursery_start || young_remap.contains_key(&id.index()));
        self.gc_mark_work.clear();

        self.remap_node_ids_for_moving_gc(
            current_root,
            frame_stack,
            eval_spine,
            persistent_spine,
            scratch_args,
            scratch_apps,
            machine_stack,
            |id| {
                if id.index() < nursery_start {
                    id
                } else {
                    young_remap
                        .get(&id.index())
                        .copied()
                        .unwrap_or_else(|| panic!("moving GC missing nursery remap for {id:?}"))
                }
            },
        );

        self.gc_marked.clear();
        self.gc_foreign_finalizer_marked.clear();
        #[cfg(feature = "moving-gc")]
        {
            self.gc_nursery_start = self.nodes.len();
        }

        old_len - self.nodes.len()
    }
}
