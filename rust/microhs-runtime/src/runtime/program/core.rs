//! Core Program construction, heap access, allocation, and resolution.
use super::*;

impl Program {
    pub(crate) fn new(nodes: Vec<Node>, root: NodeId, labels: HashMap<usize, NodeId>) -> Self {
        let boot_time_micro = current_time_micro();
        #[cfg(target_os = "wasi")]
        let default_gc_node_interval = WASI_GC_NODE_INTERVAL;
        #[cfg(not(target_os = "wasi"))]
        let default_gc_node_interval = GC_NODE_INTERVAL;
        let gc_node_interval = std::env::var("MHS_GC_NODE_INTERVAL")
            .ok()
            .and_then(|value| value.parse::<usize>().ok())
            .unwrap_or(default_gc_node_interval);
        let high_water_nodes = nodes.len();
        let mut cold_nodes = ColdNodes::default();
        let nodes = nodes
            .into_iter()
            .map(|node| Cell::from_node(node, &mut cold_nodes))
            .collect::<Vec<_>>();
        let mut small_ints = [None; SMALL_INT_COUNT];
        let weak_nodes = nodes
            .iter()
            .enumerate()
            .filter_map(|(index, cell)| {
                let cold = cell.cold_index()?;
                matches!(cold_nodes.get(cold), Some(Node::Weak(_)))
                    .then(|| NodeId::from_index(index))
            })
            .collect();
        for (index, node) in nodes.iter().enumerate() {
            if let Some(value) = node.int_value()
                && let Some(slot) = small_int_index(value)
            {
                small_ints[slot].get_or_insert(NodeId::from_index(index));
            }
        }
        Self {
            nodes,
            cold_nodes,
            root,
            labels,
            node_pointers: Vec::new(),
            node_pointer_slots: HashMap::new(),
            free_head: None,
            free_nodes: 0,
            gc_node_interval,
            force_gc: false,
            gc_allocations_since_collect: 0,
            gc_last_allocations_since_collect: 0,
            gc_collections: 0,
            gc_freed_nodes_total: 0,
            gc_last_live_nodes: high_water_nodes,
            gc_last_free_nodes: 0,
            gc_high_water_nodes: high_water_nodes,
            gc_last_pause_nanos: 0,
            gc_total_pause_nanos: 0,
            #[cfg(feature = "gc-phase-profile")]
            gc_last_mark_nanos: 0,
            #[cfg(feature = "gc-phase-profile")]
            gc_total_mark_nanos: 0,
            #[cfg(feature = "gc-phase-profile")]
            gc_last_sweep_nanos: 0,
            #[cfg(feature = "gc-phase-profile")]
            gc_total_sweep_nanos: 0,
            gc_marked: Vec::new(),
            gc_mark_work: Vec::new(),
            active_reducers: Vec::new(),
            gc_shortcut: true,
            gc_foreign_finalizer_marked: Vec::new(),
            gc_events: Vec::new(),
            stable_ptrs: vec![None],
            stable_ptr_first_free: 1,
            weak_nodes,
            pending_weak_finalizers: Vec::new(),
            foreign_finalizers: Vec::new(),
            foreign_finalizer_free: Vec::new(),
            allocations: Vec::new(),
            allocation_first_free: 0,
            bfiles: Vec::new(),
            bfile_first_free: 0,
            dirs: Vec::new(),
            dir_first_free: 0,
            program_args: Vec::new(),
            executable_path: None,
            arg_ref_array: None,
            boot_time_micro,
            errno_value: 0,
            errno_ptr: None,
            masking_state: 0,
            reductions: 0,
            js_program_handle: None,
            js_wrapper_tags: Vec::new(),
            prim_cache: PrimCache::default(),
            compound_cache: CompoundCache::default(),
            small_ints,
            world: None,
            #[cfg(feature = "profile")]
            profile: None,
            reduce_depth: 0,
            threads: Vec::new(),
            live_thread_count: 0,
            pending_async_count: 0,
            thread_states: Vec::new(),
            thread_ids: Vec::new(),
            run_queue: std::collections::VecDeque::new(),
            mvar_waiters: HashMap::new(),
            delay_wakeups: HashMap::new(),
            scheduler_epoch: Instant::now(),
            current_thread: 0,
            next_thread_id: 1,
            reschedule_now: false,
            preserve_thread_root_once: false,
        }
    }

    pub fn root(&self) -> NodeId {
        self.root
    }

    pub fn node_count(&self) -> usize {
        self.nodes.len()
    }

    pub(in crate::runtime) fn node_for_debug(&self, id: NodeId) -> Node {
        self.nodes[id.index()].to_node(&self.cold_nodes)
    }

    pub(in crate::runtime) fn cell(&self, id: NodeId) -> Cell {
        self.nodes[id.index()]
    }

    #[inline]
    pub(in crate::runtime) fn cell_trusted(&self, id: NodeId) -> Cell {
        debug_assert!(id.index() < self.nodes.len());
        unsafe { *self.nodes.get_unchecked(id.index()) }
    }

    #[inline]
    pub(in crate::runtime) fn cell_int_value(&self, id: NodeId) -> Option<i64> {
        let cell = self.cell_trusted(id);
        if let Some(value) = cell.int_value() {
            return Some(value);
        }
        if let Some(Node::Int(value)) = self.cold_node(id) {
            return Some(*value);
        }
        None
    }

    #[inline]
    pub(in crate::runtime) fn cell_int64_value(&self, id: NodeId) -> Option<i64> {
        if let Some(Node::Int64(value)) = self.cold_node(id) {
            return Some(*value);
        }
        None
    }

    #[inline]
    pub(in crate::runtime) fn cell_float64_value(&self, id: NodeId) -> Option<f64> {
        if let Some(Node::Float64(value)) = self.cold_node(id) {
            return Some(*value);
        }
        None
    }

    #[inline]
    pub(in crate::runtime) fn cell_float32_value(&self, id: NodeId) -> Option<f32> {
        let cell = self.cell_trusted(id);
        if let Some(value) = cell.float32_value() {
            return Some(value);
        }
        None
    }

    #[inline]
    pub(in crate::runtime) fn cell_thread_id_value(&self, id: NodeId) -> Option<i64> {
        let cell = self.cell_trusted(id);
        if let Some(value) = cell.thread_id_value() {
            return Some(value);
        }
        if let Some(Node::ThreadId(value)) = self.cold_node(id) {
            return Some(*value);
        }
        None
    }

    #[inline]
    pub(in crate::runtime) fn cell_ptr_value(&self, id: NodeId) -> Option<i64> {
        if let Some(Node::Ptr(value)) = self.cold_node(id) {
            return Some(*value);
        }
        None
    }

    #[inline]
    pub(in crate::runtime) fn cell_raw_fun_ptr_value(&self, id: NodeId) -> Option<i64> {
        if let Some(Node::RawFunPtr(value)) = self.cold_node(id) {
            return Some(*value);
        }
        None
    }

    #[inline]
    pub(in crate::runtime) fn app_fun_trusted(&self, id: NodeId) -> Option<NodeId> {
        self.cell_trusted(id).app_fun_trusted()
    }

    pub(in crate::runtime) fn set_cell_at(&mut self, index: usize, cell: Cell) {
        self.drop_cold_payload(index);
        debug_assert!(index < self.nodes.len());
        // SAFETY: callers rewrite existing heap slots selected by NodeId/index.
        unsafe {
            *self.nodes.get_unchecked_mut(index) = cell;
        }
    }

    #[inline(always)]
    pub(in crate::runtime) fn set_app_cell_at(&mut self, index: usize, cell: Cell) {
        debug_assert!(index < self.nodes.len());
        debug_assert_eq!(self.nodes[index].tag(), CellTag::App);
        // SAFETY: callers only pass redex/stack-app indices produced by the
        // reducer, and the debug assertions document both bounds and App-ness.
        unsafe {
            *self.nodes.get_unchecked_mut(index) = cell;
        }
    }

    #[inline(always)]
    pub(in crate::runtime) fn set_app_node_at(&mut self, index: usize, node: Node) {
        debug_assert!(index < self.nodes.len());
        debug_assert_eq!(self.nodes[index].tag(), CellTag::App);
        let cell = Cell::from_node(node, &mut self.cold_nodes);
        // SAFETY: callers only rewrite existing application redexes. App cells
        // have no cold payload to drop; the debug assertions check the invariant.
        unsafe {
            *self.nodes.get_unchecked_mut(index) = cell;
        }
    }

    pub(in crate::runtime) fn set_node_at(&mut self, index: usize, node: Node) {
        self.drop_cold_payload(index);
        debug_assert!(index < self.nodes.len());
        let cell = Cell::from_node(node, &mut self.cold_nodes);
        // SAFETY: callers rewrite existing heap slots selected by NodeId/index.
        unsafe {
            *self.nodes.get_unchecked_mut(index) = cell;
        }
    }

    /// Append a cell and return its `NodeId`. This is the ONLY place the node
    /// arena grows, and the only place `NodeId`s are minted, which makes it the
    /// enforcement point for two invariants the hot-path `unsafe` relies on:
    ///
    /// 1. Cell-id fit: the `assert!` keeps `nodes.len() < CELL_NONE_ID`, so
    ///    every id ever produced fits the active cell id field. This is what makes
    ///    the `Cell::*_trusted` constructors sound.
    /// 2. Permanent index validity: the arena never shrinks (freed slots stay
    ///    in `nodes` tagged `Free`), so any `NodeId` handed out is a valid
    ///    `nodes` index for the rest of the run. That is what lets the reducer
    ///    index `nodes` with `get_unchecked` given only a `NodeId`.
    ///
    /// The `assert!` is a real (non-debug) check so the invariant holds in
    /// release; the cap is ~1e9 cells for the default packed representation and
    /// ~4.3e9 cells with the `wide-cell` feature.
    pub(in crate::runtime) fn push_cell(&mut self, cell: Cell) -> NodeId {
        std::cfg_select! {
            feature = "wide-cell" => {
                const CAP_MSG: &str = "node arena exceeded the wide 16-byte cell cap \
                    (~4.29B cells; u32 index limit)";
            }
            _ => {
                const CAP_MSG: &str = "node arena exceeded the packed 8-byte cell cap \
                    (~1.07B cells; 2^30 index limit) — rebuild with `--features wide-cell` \
                    for a 16-byte cell and a ~4.29B-cell arena";
            }
        }
        assert!((self.nodes.len() as u64) < CELL_NONE_ID, "{CAP_MSG}");
        let id = NodeId::from_index(self.nodes.len());
        self.nodes.push(cell);
        self.gc_high_water_nodes = self.nodes.len();
        id
    }

    pub(in crate::runtime) fn cold_node(&self, id: NodeId) -> Option<&Node> {
        let cold = self.cell_trusted(id).cold_index()?;
        self.cold_nodes.get(cold)
    }

    pub(in crate::runtime) fn cold_node_mut(&mut self, id: NodeId) -> Option<&mut Node> {
        let cold = self.cell_trusted(id).cold_index()?;
        self.cold_nodes.get_mut(cold)
    }

    pub(in crate::runtime) fn drop_cold_payload(&mut self, index: usize) {
        debug_assert!(index < self.nodes.len());
        // SAFETY: callers pass existing heap slot indices.
        let cell = unsafe { *self.nodes.get_unchecked(index) };
        if let Some(cold) = cell.cold_index() {
            self.cold_nodes.release(cold);
        }
    }

    pub fn set_program_args(&mut self, args: Vec<Vec<u8>>) {
        self.program_args = args;
        self.arg_ref_array = None;
    }

    pub fn set_executable_path(&mut self, path: Option<Vec<u8>>) {
        self.executable_path = path;
    }

    #[cfg(feature = "profile")]
    pub fn enable_profile(&mut self) {
        self.profile = Some(EvalProfile::default());
    }

    #[cfg(feature = "profile")]
    pub fn take_profile(&mut self) -> Option<EvalProfile> {
        self.profile.take()
    }

    #[cfg(all(target_arch = "wasm32", not(target_os = "wasi")))]
    pub(crate) fn set_js_program_handle(&mut self, handle: u32) {
        self.js_program_handle = Some(handle);
    }

    #[inline(always)]
    pub(in crate::runtime) fn pop_free_node_nonempty(&mut self) -> usize {
        debug_assert!(self.free_nodes > 0);
        let head = match self.free_head {
            Some(head) => head,
            // SAFETY: `free_head` and `free_nodes` are maintained together
            // (see push_free_node / pop_free_node): `free_nodes > 0` iff the
            // free list is non-empty iff `free_head.is_some()`. This function's
            // contract is that the caller already checked `free_nodes > 0`
            // (debug_assert above), so `None` here is unreachable.
            None => unsafe {
                std::hint::unreachable_unchecked();
            },
        };
        let index = head.index();
        debug_assert!(index < self.nodes.len());
        // SAFETY: free_head is maintained exclusively from indices in nodes.
        let cell = unsafe { *self.nodes.get_unchecked(index) };
        debug_assert_eq!(
            cell.tag(),
            CellTag::Free,
            "free-list head did not point to a free node"
        );
        self.free_head = cell.option_id_word1();
        self.free_nodes -= 1;
        index
    }

    pub(in crate::runtime) fn push_free_node(&mut self, index: usize) {
        debug_assert!(index < self.nodes.len());
        // SAFETY: sweep passes indices from 0..nodes.len(), and other callers
        // free existing heap slots.
        let old = unsafe { *self.nodes.get_unchecked(index) };
        if let Some(cold) = old.cold_index() {
            self.cold_nodes.release(cold);
        }
        // SAFETY: same bounds invariant as above.
        unsafe {
            *self.nodes.get_unchecked_mut(index) = Cell::free(self.free_head);
        }
        self.free_head = Some(NodeId::from_index(index));
        self.free_nodes += 1;
    }

    pub(in crate::runtime) fn push_node(&mut self, node: Node) -> NodeId {
        self.gc_allocations_since_collect += 1;
        if self.free_nodes == 0 {
            let cell = Cell::from_node(node, &mut self.cold_nodes);
            self.push_cell(cell)
        } else {
            let index = self.pop_free_node_nonempty();
            debug_assert_eq!(self.nodes[index].tag(), CellTag::Free);
            self.nodes[index] = Cell::from_node(node, &mut self.cold_nodes);
            NodeId::from_index(index)
        }
    }

    #[inline(always)]
    pub(in crate::runtime) fn push_app_node(&mut self, fun: NodeId, arg: NodeId) -> NodeId {
        self.gc_allocations_since_collect += 1;
        if self.free_nodes == 0 {
            self.push_cell(Cell::app_trusted(fun, arg))
        } else {
            let index = self.pop_free_node_nonempty();
            debug_assert_eq!(self.nodes[index].tag(), CellTag::Free);
            self.nodes[index] = Cell::app_trusted(fun, arg);
            NodeId::from_index(index)
        }
    }
}
