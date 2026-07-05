//! Feature-gated heap evacuation scaffold for moving GC.
#![allow(dead_code)]

use super::*;

impl Program {
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
}
