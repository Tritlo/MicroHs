//! Mark-sweep GC over the node arena and cold payload tables.
use super::*;

const GC_EVENTS_LIMIT: usize = 1024;

enum GcRedRewrite {
    Indir(NodeId),
    Prim(Prim),
}

impl Program {
    pub fn gc_stats(&self) -> GcStats {
        GcStats {
            collections: self.gc_collections,
            freed_nodes_total: self.gc_freed_nodes_total,
            last_live_nodes: self.gc_last_live_nodes,
            last_free_nodes: self.gc_last_free_nodes,
            high_water_nodes: self.gc_high_water_nodes,
            current_nodes: self.nodes.len(),
            current_free_nodes: self.free_nodes,
            last_pause_nanos: self.gc_last_pause_nanos,
            total_pause_nanos: self.gc_total_pause_nanos,
            #[cfg(feature = "gc-phase-profile")]
            last_mark_nanos: self.gc_last_mark_nanos,
            #[cfg(feature = "gc-phase-profile")]
            total_mark_nanos: self.gc_total_mark_nanos,
            #[cfg(feature = "gc-phase-profile")]
            last_sweep_nanos: self.gc_last_sweep_nanos,
            #[cfg(feature = "gc-phase-profile")]
            total_sweep_nanos: self.gc_total_sweep_nanos,
            last_allocations_since_collect: self.gc_last_allocations_since_collect,
            current_allocations_since_collect: self.gc_allocations_since_collect,
            events: self.gc_events.clone(),
        }
    }

    // Nested bounds/mark checks benchmark faster in the GC mark loop.
    #[allow(clippy::collapsible_if)]
    pub(in crate::runtime) fn mark_node_id(
        marked: &mut [bool],
        work: &mut Vec<NodeId>,
        id: NodeId,
    ) {
        if let Some(mark) = marked.get_mut(id.index()) {
            if !*mark {
                *mark = true;
                work.push(id);
            }
        }
    }

    pub(in crate::runtime) fn node_pointer_target(&self, ptr: i64) -> Option<NodeId> {
        if ptr <= 0 {
            return None;
        }
        let slot_word = usize::try_from(ptr >> 32).ok()?;
        if slot_word == 0 {
            return None;
        }
        self.node_pointers.get(slot_word - 1).copied()
    }

    pub(in crate::runtime) fn mark_pointer_target(
        &self,
        marked: &mut [bool],
        work: &mut Vec<NodeId>,
        ptr: i64,
    ) {
        if let Some(id) = self.node_pointer_target(ptr) {
            Self::mark_node_id(marked, work, id);
        }
    }

    pub(in crate::runtime) fn mark_machine_stack(
        &self,
        marked: &mut [bool],
        work: &mut Vec<NodeId>,
        stack: &EvalStack,
    ) {
        for app in &stack.apps {
            Self::mark_node_id(marked, work, *app);
        }
        for frame in &stack.frames {
            match frame {
                StackFrame::Whnf(frame) => {
                    Self::mark_node_id(marked, work, frame.redex);
                    match &frame.kind {
                        WhnfFrameKind::Seq { result } => Self::mark_node_id(marked, work, *result),
                        WhnfFrameKind::IoStrict { action, value } => {
                            Self::mark_node_id(marked, work, *action);
                            Self::mark_node_id(marked, work, *value);
                        }
                        WhnfFrameKind::IsInt => {}
                    }
                }
                StackFrame::Int(frame) => match &frame.kind {
                    IntFrameKind::BinSecond { x, .. } => {
                        Self::mark_node_id(marked, work, frame.redex);
                        Self::mark_node_id(marked, work, *x);
                    }
                    IntFrameKind::BinFirst { .. } | IntFrameKind::Un { .. } => {
                        Self::mark_node_id(marked, work, frame.redex);
                    }
                },
                StackFrame::Int64(frame) => {
                    if let Int64FrameKind::BinSecond { x, .. } = &frame.kind {
                        Self::mark_node_id(marked, work, *x);
                    }
                }
                StackFrame::Int64Shift(frame) => {
                    Self::mark_node_id(marked, work, frame.x);
                }
                StackFrame::Float64(frame) => {
                    if let Float64FrameKind::BinSecond { x, .. } = &frame.kind {
                        Self::mark_node_id(marked, work, *x);
                    }
                }
                StackFrame::Float32(frame) => {
                    if let Float32FrameKind::BinSecond { x, .. } = &frame.kind {
                        Self::mark_node_id(marked, work, *x);
                    }
                }
                StackFrame::Bytes(frame) => match &frame.kind {
                    BytesFrameKind::BinSecond { x, .. } => Self::mark_node_id(marked, work, *x),
                    BytesFrameKind::BinFirst { y, .. } => Self::mark_node_id(marked, work, *y),
                },
                StackFrame::Conversion(_) => {}
            }
        }
    }

    pub(in crate::runtime) fn mark_eval_spine(
        marked: &mut [bool],
        work: &mut Vec<NodeId>,
        spine: &EvalSpine,
    ) {
        for idx in 0..spine.len() {
            Self::mark_node_id(marked, work, spine.desc_arg(idx));
            Self::mark_node_id(marked, work, spine.desc_app(idx));
        }
    }

    // Keep the independent root sets borrowed directly instead of wrapping them for one call.
    #[allow(clippy::too_many_arguments)]
    pub(in crate::runtime) fn mark_program_roots(
        &self,
        marked: &mut [bool],
        work: &mut Vec<NodeId>,
        current_root: NodeId,
        eval_spine: &EvalSpine,
        scratch_args: &[NodeId],
        scratch_apps: &[NodeId],
        machine_stack: Option<&EvalStack>,
    ) {
        Self::mark_node_id(marked, work, self.root);
        Self::mark_node_id(marked, work, current_root);
        // Every live thread's continuation is a root, else a suspended thread's graph
        // would be collected out from under it.
        for thread in self.threads.iter().flatten() {
            Self::mark_node_id(marked, work, thread.root);
            if let Some(value) = thread.delivered_value {
                Self::mark_node_id(marked, work, value);
            }
            if let Some(exn) = thread.pending_exception {
                Self::mark_node_id(marked, work, exn);
            }
        }
        for queues in self.mvar_waiters.values() {
            for slot in queues.takeput.iter().chain(&queues.read) {
                if let Some(thread) = self.threads.get(*slot).and_then(Option::as_ref) {
                    Self::mark_node_id(marked, work, thread.root);
                    if let Some(value) = thread.delivered_value {
                        Self::mark_node_id(marked, work, value);
                    }
                    if let Some(exn) = thread.pending_exception {
                        Self::mark_node_id(marked, work, exn);
                    }
                }
            }
        }
        for id in self.stable_ptrs.iter().flatten() {
            Self::mark_node_id(marked, work, *id);
        }
        for bfile in self.bfiles.iter().flatten() {
            if let BFileKind::ReadOnlyMemoryView { base, .. } = &bfile.kind {
                Self::mark_node_id(marked, work, *base);
            }
        }
        if let Some(id) = self.arg_ref_array {
            Self::mark_node_id(marked, work, id);
        }
        if let Some(id) = self.world {
            Self::mark_node_id(marked, work, id);
        }
        for id in self.small_ints.iter().flatten() {
            Self::mark_node_id(marked, work, *id);
        }
        for id in [
            self.prim_cache.a,
            self.prim_cache.b,
            self.prim_cache.c,
            self.prim_cache.i,
            self.prim_cache.k,
            self.prim_cache.k2,
            self.prim_cache.k3,
            self.prim_cache.o,
            self.prim_cache.p,
            self.prim_cache.u,
            self.prim_cache.y,
            self.prim_cache.z,
            self.prim_cache.io_bind,
            self.prim_cache.io_perform_io,
            self.compound_cache.fst,
            self.compound_cache.snd,
            self.compound_cache.just,
            self.compound_cache.pair_unit,
        ]
        .into_iter()
        .flatten()
        {
            Self::mark_node_id(marked, work, id);
        }
        if let Some(machine_stack) = machine_stack {
            self.mark_machine_stack(marked, work, machine_stack);
        }
        // The innermost reducer's stack is `machine_stack`; a caller may
        // still use its entry node after a nested slice returns (for example
        // `catch` after a step limit), so every entry is a root.
        let outer = self.active_reducers.len().saturating_sub(1);
        for (depth, reducer) in self.active_reducers.iter().enumerate() {
            Self::mark_node_id(marked, work, reducer.entry);
            if depth < outer {
                // SAFETY: see `ActiveReducer`; the outer reducer's local
                // outlives this collection and is not accessed meanwhile.
                let stack = unsafe { &*reducer.stack };
                self.mark_machine_stack(marked, work, stack);
            }
        }
        Self::mark_eval_spine(marked, work, eval_spine);
        for id in scratch_args.iter().chain(scratch_apps) {
            Self::mark_node_id(marked, work, *id);
        }
    }

    pub(in crate::runtime) fn compress_marked_indirection(&mut self, id: NodeId) -> Option<NodeId> {
        let mut current = id;
        let mut depth = 0usize;
        loop {
            match self.nodes.get(current.index()).map(|cell| cell.tag()) {
                Some(CellTag::Indir) => {
                    let next = self.nodes[current.index()].option_id_word1()?;
                    current = next;
                    depth += 1;
                    if depth > self.nodes.len() {
                        return None;
                    }
                }
                Some(CellTag::Free) | None => return None,
                Some(_) => break,
            }
        }
        if depth > 1 {
            self.set_cell_at(id.index(), Cell::indir_trusted(current));
        }
        Some(current)
    }

    // Nested canonicalization checks benchmark faster in the GC mark loop.
    #[allow(clippy::collapsible_if)]
    pub(in crate::runtime) fn canonical_gc_target(&mut self, id: NodeId) -> Option<NodeId> {
        let cell = self.nodes.get(id.index()).copied()?;
        let tag = cell.tag_bits();
        let target = if tag == CellTag::Indir.bits() {
            self.compress_marked_indirection(id)?
        } else if tag == CellTag::Free.bits() {
            return None;
        } else {
            id
        };
        if let Some(value) = self.cell_int_value(target) {
            if let Some(slot) = small_int_index(value) {
                if let Some(canonical) = self.small_ints[slot] {
                    if canonical != target {
                        self.set_cell_at(target.index(), Cell::indir_trusted(canonical));
                    }
                    return Some(canonical);
                }
            }
        }
        Some(target)
    }

    pub(in crate::runtime) fn mark_canonical_child(
        &mut self,
        marked: &mut [bool],
        work: &mut Vec<NodeId>,
        child: NodeId,
    ) -> NodeId {
        let target = self.canonical_gc_target(child).unwrap_or(child);
        Self::mark_node_id(marked, work, target);
        target
    }

    fn gc_red_resolved_id(&self, id: NodeId) -> Option<NodeId> {
        let mut current = id;
        for _ in 0..self.nodes.len() {
            let cell = self.nodes.get(current.index())?;
            match cell.tag() {
                CellTag::Indir => current = cell.option_id_word1()?,
                CellTag::Free => return None,
                _ => return Some(current),
            }
        }
        None
    }

    fn gc_red_prim(&self, id: NodeId) -> Option<Prim> {
        let id = self.gc_red_resolved_id(id)?;
        self.nodes.get(id.index())?.prim()
    }

    fn gc_red_app_fields(&self, id: NodeId) -> Option<(NodeId, NodeId)> {
        let id = self.gc_red_resolved_id(id)?;
        self.nodes.get(id.index())?.app_fields()
    }

    fn gc_red_flipped_prim(prim: Prim) -> Option<Prim> {
        match prim {
            Prim::Known(KnownPrim::K) => Some(Prim::Known(KnownPrim::A)),
            Prim::Known(KnownPrim::A) => Some(Prim::Known(KnownPrim::K)),
            Prim::Runtime(runtime) => {
                let flipped = match runtime.name() {
                    "+" => "+",
                    "-" => "subtract",
                    "*" => "*",
                    "u+" => "u+",
                    "u-" => "usubtract",
                    "u*" => "u*",
                    "subtract" => "-",
                    "usubtract" => "u-",
                    "and" => "and",
                    "or" => "or",
                    "xor" => "xor",
                    "d+" => "d+",
                    "d*" => "d*",
                    "d==" => "d==",
                    "d/=" => "d/=",
                    "d<" => "d>",
                    "d<=" => "d>=",
                    "d>" => "d<",
                    "d>=" => "d<=",
                    "f+" => "f+",
                    "f*" => "f*",
                    "f==" => "f==",
                    "f/=" => "f/=",
                    "f<" => "f>",
                    "f<=" => "f>=",
                    "f>" => "f<",
                    "f>=" => "f<=",
                    "bs==" => "bs==",
                    "bs/=" => "bs/=",
                    "bs<" => "bs>",
                    "bs<=" => "bs>=",
                    "bs>" => "bs<",
                    "bs>=" => "bs<=",
                    "==" => "==",
                    "/=" => "/=",
                    "<" => ">",
                    "u<" => "u>",
                    "u<=" => "u>=",
                    "u>" => "u<",
                    "u>=" => "u<=",
                    "<=" => ">=",
                    ">" => "<",
                    ">=" => "<=",
                    "I+" => "I+",
                    "I-" => "Isubtract",
                    "I*" => "I*",
                    "Iu+" => "Iu+",
                    "Iu-" => "Iusubtract",
                    "Iu*" => "Iu*",
                    "Isubtract" => "I-",
                    "Iusubtract" => "Iu-",
                    "Iand" => "Iand",
                    "Ior" => "Ior",
                    "Ixor" => "Ixor",
                    "I==" => "I==",
                    "I/=" => "I/=",
                    "I<" => "I>",
                    "Iu<" => "Iu>",
                    "Iu<=" => "Iu>=",
                    "Iu>" => "Iu<",
                    "Iu>=" => "Iu<=",
                    "I<=" => "I>=",
                    "I>" => "I<",
                    "I>=" => "I<=",
                    _ => return None,
                };
                Prim::from_name(flipped)
            }
            _ => None,
        }
    }

    fn gc_red_rewrite(&self, fun: NodeId, arg: NodeId) -> Option<GcRedRewrite> {
        use KnownPrim::*;

        let funt = self.gc_red_prim(fun);
        let argt = self.gc_red_prim(arg);
        let fun_app = self.gc_red_app_fields(fun);
        let funfunt = fun_app.and_then(|(fun_fun, _)| self.gc_red_prim(fun_fun));
        let arg_app = self.gc_red_app_fields(arg);
        let arg_fun_t = arg_app.and_then(|(arg_fun, _)| self.gc_red_prim(arg_fun));

        if funfunt == Some(Prim::Known(A)) {
            return Some(GcRedRewrite::Indir(arg));
        }
        if funfunt == Some(Prim::Known(K)) {
            return Some(GcRedRewrite::Indir(fun_app?.1));
        }
        if funt == Some(Prim::Known(I)) {
            return Some(GcRedRewrite::Indir(arg));
        }
        if funt == Some(Prim::Known(CPrime)) && argt == Some(Prim::Known(I)) {
            return Some(GcRedRewrite::Prim(Prim::Known(C)));
        }
        if funt == Some(Prim::Known(CPrimeB))
            && let Some((arg_fun, arg_arg)) = arg_app
        {
            let arg_is_p = self.gc_red_prim(arg_arg) == Some(Prim::Known(P));
            let arg_fun_is_bc = self
                .gc_red_app_fields(arg_fun)
                .map(|(bc_fun, bc_arg)| {
                    self.gc_red_prim(bc_fun) == Some(Prim::Known(B))
                        && self.gc_red_prim(bc_arg) == Some(Prim::Known(C))
                })
                .unwrap_or(false);
            if arg_is_p && arg_fun_is_bc {
                return Some(GcRedRewrite::Prim(Prim::Known(C)));
            }
        }
        if funt == Some(Prim::Known(B)) && argt == Some(Prim::Known(I)) {
            return Some(GcRedRewrite::Prim(Prim::Known(I)));
        }
        if funfunt == Some(Prim::Known(B)) && argt == Some(Prim::Known(I)) {
            return Some(GcRedRewrite::Indir(fun_app?.1));
        }
        if funfunt == Some(Prim::Known(CPrimeB)) && argt == Some(Prim::Known(I)) {
            return Some(GcRedRewrite::Indir(fun_app?.1));
        }
        if funt == Some(Prim::Known(C)) && arg_fun_t == Some(Prim::Known(C)) {
            return Some(GcRedRewrite::Indir(arg_app?.1));
        }
        if funt == Some(Prim::Known(C))
            && let Some(flipped) = argt.and_then(Self::gc_red_flipped_prim)
        {
            return Some(GcRedRewrite::Prim(flipped));
        }
        None
    }

    // Nested optional-finalizer checks keep the measured GC loop shape.
    #[allow(clippy::collapsible_if)]
    pub(in crate::runtime) fn mark_reachable<const REDUCE_APPS: bool>(
        &mut self,
        marked: &mut [bool],
        work: &mut Vec<NodeId>,
        foreign_finalizer_marked: &mut [bool],
    ) {
        while let Some(id) = work.pop() {
            let Some(cell) = self.nodes.get(id.index()).copied() else {
                continue;
            };
            let tag = cell.tag_bits();
            if tag == CellTag::Indir.bits() {
                // Compress the chain in both modes. Cells in the middle of a
                // chain are reachable only through another indirection, so no
                // suspended caller can hold a valid id for one of them.
                if let Some(target) = self.compress_marked_indirection(id) {
                    Self::mark_node_id(marked, work, target);
                }
                continue;
            }
            if tag == CellTag::App.bits() {
                let fun = cell.id_payload();
                let arg = cell.id_word1();
                if !self.gc_shortcut {
                    // Keep the children as they are: an outer caller may
                    // hold the id of an indirection or non-canonical int here.
                    Self::mark_node_id(marked, work, fun);
                    Self::mark_node_id(marked, work, arg);
                    continue;
                }
                let fun = self.canonical_gc_target(fun).unwrap_or(fun);
                let arg = self.canonical_gc_target(arg).unwrap_or(arg);
                if REDUCE_APPS {
                    if let Some(rewrite) = self.gc_red_rewrite(fun, arg) {
                        match rewrite {
                            GcRedRewrite::Indir(target) => {
                                let target = self.mark_canonical_child(marked, work, target);
                                self.set_app_cell_at(id.index(), Cell::indir_trusted(target));
                            }
                            GcRedRewrite::Prim(Prim::Known(known)) => {
                                self.set_app_cell_at(id.index(), Cell::known_prim_trusted(known))
                            }
                            GcRedRewrite::Prim(Prim::Runtime(runtime)) => {
                                self.set_app_cell_at(id.index(), Cell::runtime_prim(runtime))
                            }
                        }
                        continue;
                    }
                }
                Self::mark_node_id(marked, work, fun);
                Self::mark_node_id(marked, work, arg);
                if fun != cell.id_payload() || arg != cell.id_word1() {
                    self.set_app_cell_at(id.index(), Cell::app_trusted(fun, arg));
                }
                continue;
            }
            if tag == CellTag::Cold.bits() {
                let cold = cell.payload0() as usize;
                match self.cold_nodes.get(cold) {
                    Some(Node::Ptr(ptr) | Node::RawFunPtr(ptr)) => {
                        self.mark_pointer_target(marked, work, *ptr);
                    }
                    Some(Node::ForeignPtr(foreign_ptr)) => {
                        if let Some(finalizer) = foreign_ptr.finalizer {
                            if let Some(marked) = foreign_finalizer_marked.get_mut(finalizer) {
                                *marked = true;
                            }
                        }
                        self.mark_pointer_target(marked, work, foreign_ptr.ptr);
                    }
                    Some(Node::Weak(_)) => {}
                    Some(Node::BytesView(view)) => Self::mark_node_id(marked, work, view.base),
                    Some(Node::MVar(Some(value))) => Self::mark_node_id(marked, work, *value),
                    Some(Node::Array(items)) => {
                        for item in items.iter() {
                            Self::mark_node_id(marked, work, *item);
                        }
                    }
                    _ => {}
                }
            }
        }
    }

    pub(in crate::runtime) fn weak_key_target(&mut self, key: NodeId) -> Option<NodeId> {
        if matches!(
            self.nodes.get(key.index()).map(|cell| cell.tag()),
            Some(CellTag::Indir)
        ) {
            self.compress_marked_indirection(key)
        } else if matches!(
            self.nodes.get(key.index()).map(|cell| cell.tag()),
            Some(CellTag::Free) | None
        ) {
            None
        } else {
            Some(key)
        }
    }

    // Nested mark checks keep the measured weak-sweep loop shape.
    #[allow(clippy::collapsible_if)]
    pub(in crate::runtime) fn sweep_weaks_after_mark<const REDUCE_APPS: bool>(
        &mut self,
        marked: &mut [bool],
        work: &mut Vec<NodeId>,
        foreign_finalizer_marked: &mut [bool],
    ) -> Vec<NodeId> {
        if self.weak_nodes.is_empty() {
            return Vec::new();
        }

        let mut weak_nodes = std::mem::take(&mut self.weak_nodes)
            .into_iter()
            .filter(|id| matches!(self.cold_node(*id), Some(Node::Weak(_))))
            .collect::<Vec<_>>();
        weak_nodes.sort_unstable_by_key(|id| id.index());
        weak_nodes.dedup();
        if weak_nodes.is_empty() {
            return Vec::new();
        }

        loop {
            let mut added_marks = false;
            for id in weak_nodes.iter().copied() {
                let index = id.index();
                let Some((key, value, finalizer)) = self.cold_node(id).and_then(|node| {
                    if let Node::Weak(weak) = node {
                        Some((weak.key, weak.value, weak.finalizer))
                    } else {
                        None
                    }
                }) else {
                    continue;
                };
                let Some(value) = value else {
                    continue;
                };
                let key_live = key
                    .and_then(|key| self.weak_key_target(key))
                    .and_then(|target| {
                        marked
                            .get(target.index())
                            .copied()
                            .filter(|live| *live)
                            .map(|_| target)
                    });
                if let Some(target) = key_live {
                    if let Some(Node::Weak(weak)) = self.cold_node_mut(id) {
                        weak.key = Some(target);
                    }
                    if let Some(mark) = marked.get_mut(index) {
                        if !*mark {
                            *mark = true;
                            work.push(id);
                            added_marks = true;
                        }
                    }
                    for child in [Some(value), finalizer].into_iter().flatten() {
                        if marked
                            .get(child.index())
                            .is_some_and(|already_marked| !*already_marked)
                        {
                            Self::mark_node_id(marked, work, child);
                            added_marks = true;
                        }
                    }
                } else if let Some(Node::Weak(weak)) = self.cold_node_mut(id) {
                    weak.key = None;
                    weak.value = None;
                }
            }
            if !added_marks {
                break;
            }
            self.mark_reachable::<REDUCE_APPS>(marked, work, foreign_finalizer_marked);
        }

        let mut finalizers = Vec::new();
        for id in weak_nodes.iter().copied() {
            let finalizer = match self.cold_node_mut(id) {
                Some(Node::Weak(weak)) if weak.value.is_none() => {
                    weak.key = None;
                    weak.finalizer.take()
                }
                _ => None,
            };
            if let Some(finalizer) = finalizer {
                Self::mark_node_id(marked, work, finalizer);
                finalizers.push(finalizer);
            }
        }
        if !work.is_empty() {
            self.mark_reachable::<REDUCE_APPS>(marked, work, foreign_finalizer_marked);
        }
        self.weak_nodes = weak_nodes;
        finalizers
    }

    pub(in crate::runtime) fn run_foreign_finalizer(
        &mut self,
        finalizer: ForeignFinalizer,
        arg: i64,
    ) -> Result<(), EvalError> {
        match finalizer {
            ForeignFinalizer::Free => self.free_memory(arg),
            ForeignFinalizer::CloseB => self.close_bfile(arg),
            ForeignFinalizer::RawZero => Ok(()),
        }
    }

    pub(in crate::runtime) fn run_dead_foreign_finalizers(
        &mut self,
        marked: &[bool],
    ) -> Result<(), EvalError> {
        for index in 0..self.foreign_finalizers.len() {
            if marked.get(index).copied().unwrap_or(false) {
                continue;
            }
            let Some(state) = self.foreign_finalizers[index].take() else {
                continue;
            };
            if let Some(finalizer) = state.finalizer {
                self.run_foreign_finalizer(finalizer, state.arg)?;
            }
            self.foreign_finalizer_free.push(index);
        }
        Ok(())
    }

    pub(in crate::runtime) fn collect_garbage<const REDUCE_APPS: bool>(
        &mut self,
        current_root: NodeId,
        eval_spine: &EvalSpine,
        scratch_args: &[NodeId],
        scratch_apps: &[NodeId],
        machine_stack: Option<&EvalStack>,
    ) -> Result<usize, EvalError> {
        let started = Instant::now();
        // A nested collection must leave every reachable node in place:
        // outer callers hold plain `NodeId`s that are not roots.
        self.gc_shortcut = self.reduce_depth <= 1;
        let allocations_since_collect = self.gc_allocations_since_collect;
        let mut marked = std::mem::take(&mut self.gc_marked);
        if marked.len() < self.nodes.len() {
            marked.resize(self.nodes.len(), false);
        }
        let mut work = std::mem::take(&mut self.gc_mark_work);
        work.clear();
        let mut foreign_finalizer_marked = std::mem::take(&mut self.gc_foreign_finalizer_marked);
        foreign_finalizer_marked.clear();
        foreign_finalizer_marked.resize(self.foreign_finalizers.len(), false);
        #[cfg(feature = "gc-phase-profile")]
        let mark_started = Instant::now();
        self.mark_program_roots(
            &mut marked,
            &mut work,
            current_root,
            eval_spine,
            scratch_args,
            scratch_apps,
            machine_stack,
        );
        self.mark_reachable::<REDUCE_APPS>(&mut marked, &mut work, &mut foreign_finalizer_marked);
        let weak_finalizers = self.sweep_weaks_after_mark::<REDUCE_APPS>(
            &mut marked,
            &mut work,
            &mut foreign_finalizer_marked,
        );
        #[cfg(feature = "gc-phase-profile")]
        let mark_nanos = mark_started.elapsed().as_nanos();
        work.clear();
        self.labels
            .retain(|_, id| marked.get(id.index()).copied().unwrap_or(false));
        self.run_dead_foreign_finalizers(&foreign_finalizer_marked)?;

        #[cfg(feature = "gc-phase-profile")]
        let sweep_started = Instant::now();
        self.free_head = None;
        self.free_nodes = 0;
        let mut freed = 0;
        let mut live = 0;
        // Indexed sweep benchmarks faster; index is also the reclaimed node id.
        #[allow(clippy::needless_range_loop)]
        for index in 0..marked.len() {
            if marked[index] {
                marked[index] = false;
                live += 1;
                continue;
            }
            freed += 1;
            self.push_free_node(index);
        }
        #[cfg(feature = "gc-phase-profile")]
        let sweep_nanos = sweep_started.elapsed().as_nanos();
        self.gc_marked = marked;
        self.gc_mark_work = work;
        self.gc_foreign_finalizer_marked = foreign_finalizer_marked;
        let pause_nanos = started.elapsed().as_nanos();
        self.gc_collections += 1;
        self.gc_freed_nodes_total = self.gc_freed_nodes_total.saturating_add(freed);
        self.gc_last_live_nodes = live;
        self.gc_last_free_nodes = self.free_nodes;
        self.gc_last_pause_nanos = pause_nanos;
        self.gc_total_pause_nanos = self.gc_total_pause_nanos.saturating_add(pause_nanos);
        #[cfg(feature = "gc-phase-profile")]
        {
            self.gc_last_mark_nanos = mark_nanos;
            self.gc_total_mark_nanos = self.gc_total_mark_nanos.saturating_add(mark_nanos);
            self.gc_last_sweep_nanos = sweep_nanos;
            self.gc_total_sweep_nanos = self.gc_total_sweep_nanos.saturating_add(sweep_nanos);
        }
        self.gc_last_allocations_since_collect = allocations_since_collect;
        self.gc_allocations_since_collect = 0;
        if self.gc_events.len() == GC_EVENTS_LIMIT {
            self.gc_events.remove(0);
        }
        self.gc_events.push(GcEventStats {
            collection: self.gc_collections,
            pause_nanos,
            #[cfg(feature = "gc-phase-profile")]
            mark_nanos,
            #[cfg(feature = "gc-phase-profile")]
            sweep_nanos,
            live_nodes: live,
            free_nodes: self.free_nodes,
            arena_nodes: self.nodes.len(),
            freed_nodes: freed,
            allocations_since_collect,
        });
        // Like eval.c's sweep_weaks: each dead weak's finalizer becomes a thread
        // of its own, so it runs at the next scheduling point, isolated from the
        // thread that triggered the collection, and a weak observed dead by
        // deRefWeak runs its finalizer only afterwards.
        if !weak_finalizers.is_empty() && !self.threads.is_empty() {
            for finalizer in weak_finalizers {
                self.spawn_thread(finalizer);
            }
            // End a lone thread's unbounded slice so slicing starts; the
            // finalizer then runs at the next slice boundary or yield, as in C.
            self.reschedule_now = true;
        }
        Ok(freed)
    }

    /// Run one non-moving mark-sweep collection between evaluator steps.
    ///
    /// The caller passes the active reducer roots because they are not all
    /// stored on `Program` while a reduction slice is running.
    pub(in crate::runtime) fn collect_garbage_between_steps(
        &mut self,
        current_root: NodeId,
        eval_spine: &EvalSpine,
        scratch_args: &[NodeId],
        scratch_apps: &[NodeId],
        machine_stack: Option<&EvalStack>,
    ) -> Result<usize, EvalError> {
        self.collect_garbage::<false>(
            current_root,
            eval_spine,
            scratch_args,
            scratch_apps,
            machine_stack,
        )
    }

    /// Match the C runtime's two post-parse, allocation-free GCRED passes.
    pub(crate) fn collect_garbage_after_parse(&mut self) {
        let root = self.root;
        let eval_spine = EvalSpine::default();
        for _ in 0..2 {
            self.collect_garbage::<true>(root, &eval_spine, &[], &[], None)
                .expect("a freshly parsed program has no fallible GC finalizers");
        }
    }

    pub(in crate::runtime) fn maybe_collect_garbage_between_steps(
        &mut self,
        current_root: NodeId,
        eval_spine: &EvalSpine,
        scratch_args: &[NodeId],
        scratch_apps: &[NodeId],
        machine_stack: Option<&EvalStack>,
    ) -> Result<(), EvalError> {
        debug_assert_eq!(self.active_reducers.len(), self.reduce_depth);
        if !self.force_gc {
            if self.gc_node_interval == 0 {
                return Ok(());
            }
            if self.gc_allocations_since_collect < self.gc_node_interval {
                return Ok(());
            }
        }
        self.force_gc = false;
        self.collect_garbage_between_steps(
            current_root,
            eval_spine,
            scratch_args,
            scratch_apps,
            machine_stack,
        )?;
        Ok(())
    }
}
