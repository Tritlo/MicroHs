//! Feature-gated heap evacuation scaffold for moving GC.

use super::*;

impl Program {
    #[cfg_attr(feature = "moving-gc", allow(dead_code))]
    pub(in crate::runtime) fn evacuate_marked_heap_for_moving_gc(
        &mut self,
        marked: &[bool],
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

        let old_nodes = std::mem::take(&mut self.nodes);
        let old_cold_nodes = std::mem::take(&mut self.cold_nodes);
        let old_len = old_nodes.len();
        let live = marked.iter().filter(|marked| **marked).count();
        let mut remap_table = vec![None; old_len];
        let mut new_nodes = Vec::with_capacity(live);
        let mut new_cold_nodes = Vec::new();

        for (old_index, cell) in old_nodes.iter().copied().enumerate() {
            if !marked[old_index] {
                continue;
            }
            debug_assert_ne!(
                cell.tag(),
                CellTag::Free,
                "moving GC cannot evacuate a free cell as live"
            );
            let new_id = NodeId::from_index(new_nodes.len());
            remap_table[old_index] = Some(new_id);
            let cell = if cell.has_tag(CellTag::Cold) {
                Cell::from_node(cell.to_node(&old_cold_nodes), &mut new_cold_nodes)
            } else {
                cell
            };
            new_nodes.push(cell);
        }

        self.nodes = new_nodes;
        self.cold_nodes = new_cold_nodes;
        self.free_head = None;
        self.free_nodes = 0;
        #[cfg(feature = "moving-gc")]
        {
            self.gc_nursery_start = self.nodes.len();
        }

        self.labels
            .retain(|_, id| remap_table.get(id.index()).is_some_and(Option::is_some));
        self.weak_nodes
            .retain(|id| remap_table.get(id.index()).is_some_and(Option::is_some));
        self.gc_mark_work.clear();
        #[cfg(feature = "gc-phase-profile")]
        self.gc_young_profile_allocated_slots.clear();

        self.remap_node_ids_for_moving_gc(
            current_root,
            frame_stack,
            eval_spine,
            persistent_spine,
            scratch_args,
            scratch_apps,
            machine_stack,
            |id| {
                remap_table
                    .get(id.index())
                    .and_then(|id| *id)
                    .unwrap_or_else(|| panic!("moving GC missing live remap for {id:?}"))
            },
        );

        self.gc_marked.clear();
        self.gc_foreign_finalizer_marked.clear();

        old_len - self.nodes.len()
    }

    #[cfg_attr(not(feature = "moving-gc"), allow(dead_code))]
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
        let mut remap_table = vec![None; old_len];
        let mut new_nodes = Vec::with_capacity(old_len);
        new_nodes.extend_from_slice(&old_nodes[..nursery_start]);

        for (index, slot) in remap_table.iter_mut().take(nursery_start).enumerate() {
            *slot = Some(NodeId::from_index(index));
        }

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
            remap_table[index] = Some(new_id);
            new_nodes.push(cell);
        }

        self.nodes = new_nodes;
        self.free_head = None;
        self.free_nodes = 0;

        self.labels
            .retain(|_, id| remap_table.get(id.index()).is_some_and(Option::is_some));
        self.weak_nodes
            .retain(|id| remap_table.get(id.index()).is_some_and(Option::is_some));
        self.gc_mark_work.clear();
        #[cfg(feature = "gc-phase-profile")]
        self.gc_young_profile_allocated_slots.clear();

        self.remap_node_ids_for_moving_gc(
            current_root,
            frame_stack,
            eval_spine,
            persistent_spine,
            scratch_args,
            scratch_apps,
            machine_stack,
            |id| {
                remap_table
                    .get(id.index())
                    .and_then(|id| *id)
                    .unwrap_or_else(|| panic!("moving GC missing nursery remap for {id:?}"))
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
