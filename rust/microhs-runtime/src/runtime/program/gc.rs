//! Mark-sweep GC over the node arena and cold payload tables.
use super::*;

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
            #[cfg(feature = "gc-phase-profile")]
            red_i_opportunities: self.gc_red_i_opportunities,
            #[cfg(feature = "gc-phase-profile")]
            red_k_opportunities: self.gc_red_k_opportunities,
            #[cfg(feature = "gc-phase-profile")]
            red_a_opportunities: self.gc_red_a_opportunities,
            #[cfg(feature = "gc-phase-profile")]
            red_bi_opportunities: self.gc_red_bi_opportunities,
            #[cfg(feature = "gc-phase-profile")]
            red_bxi_opportunities: self.gc_red_bxi_opportunities,
            #[cfg(feature = "gc-phase-profile")]
            red_ccbi_opportunities: self.gc_red_ccbi_opportunities,
            #[cfg(feature = "gc-phase-profile")]
            red_cc_opportunities: self.gc_red_cc_opportunities,
            #[cfg(feature = "gc-phase-profile")]
            red_cci_opportunities: self.gc_red_cci_opportunities,
            #[cfg(feature = "gc-phase-profile")]
            red_ccbbcp_opportunities: self.gc_red_ccbbcp_opportunities,
            #[cfg(feature = "gc-phase-profile")]
            red_flip_opportunities: self.gc_red_flip_opportunities,
            #[cfg(feature = "gc-phase-profile")]
            young_profile_last_slots: self.gc_young_profile_last_slots,
            #[cfg(feature = "gc-phase-profile")]
            young_profile_last_live: self.gc_young_profile_last_live,
            #[cfg(feature = "gc-phase-profile")]
            young_profile_last_dead: self.gc_young_profile_last_dead,
            #[cfg(feature = "gc-phase-profile")]
            young_profile_last_old_to_young_sources: self
                .gc_young_profile_last_old_to_young_sources,
            #[cfg(feature = "gc-phase-profile")]
            young_profile_last_old_to_young_edges: self.gc_young_profile_last_old_to_young_edges,
            #[cfg(feature = "gc-phase-profile")]
            young_profile_total_slots: self.gc_young_profile_total_slots,
            #[cfg(feature = "gc-phase-profile")]
            young_profile_total_live: self.gc_young_profile_total_live,
            #[cfg(feature = "gc-phase-profile")]
            young_profile_total_dead: self.gc_young_profile_total_dead,
            #[cfg(feature = "gc-phase-profile")]
            young_profile_total_old_to_young_sources: self
                .gc_young_profile_total_old_to_young_sources,
            #[cfg(feature = "gc-phase-profile")]
            young_profile_total_old_to_young_edges: self.gc_young_profile_total_old_to_young_edges,
            last_allocations_since_collect: self.gc_last_allocations_since_collect,
            current_allocations_since_collect: self.gc_allocations_since_collect,
            events: self.gc_events.clone(),
        }
    }

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

    #[cfg(feature = "gc-phase-profile")]
    pub(in crate::runtime) fn gc_profile_young_edge(young: &[bool], id: NodeId) -> usize {
        young.get(id.index()).copied().unwrap_or(false) as usize
    }

    #[cfg(feature = "gc-phase-profile")]
    pub(in crate::runtime) fn gc_profile_young_pointer_edge(
        &self,
        young: &[bool],
        ptr: i64,
    ) -> usize {
        self.node_pointer_target(ptr)
            .map(|id| Self::gc_profile_young_edge(young, id))
            .unwrap_or(0)
    }

    #[cfg(feature = "gc-phase-profile")]
    pub(in crate::runtime) fn gc_profile_old_to_young_edges_for_cell(
        &self,
        index: usize,
        cell: Cell,
        young: &[bool],
    ) -> usize {
        if let Some((fun, arg)) = cell.app_fields() {
            return Self::gc_profile_young_edge(young, fun)
                + Self::gc_profile_young_edge(young, arg);
        }
        match cell.tag() {
            CellTag::Indir => cell
                .option_id_word1()
                .map(|id| Self::gc_profile_young_edge(young, id))
                .unwrap_or(0),
            CellTag::Ptr | CellTag::RawFunPtr => cell
                .pointer_payload()
                .map(|ptr| self.gc_profile_young_pointer_edge(young, ptr))
                .unwrap_or(0),
            CellTag::Cold => match self.cold_node(NodeId::from_index(index)) {
                Some(Node::Ptr(ptr) | Node::RawFunPtr(ptr)) => {
                    self.gc_profile_young_pointer_edge(young, *ptr)
                }
                Some(Node::ForeignPtr(foreign_ptr)) => {
                    self.gc_profile_young_pointer_edge(young, foreign_ptr.ptr)
                }
                Some(Node::Weak(weak)) => {
                    weak.value
                        .map(|id| Self::gc_profile_young_edge(young, id))
                        .unwrap_or(0)
                        + weak
                            .finalizer
                            .map(|id| Self::gc_profile_young_edge(young, id))
                            .unwrap_or(0)
                }
                Some(Node::BytesView(view)) => Self::gc_profile_young_edge(young, view.base),
                Some(Node::MVar(Some(value))) => Self::gc_profile_young_edge(young, *value),
                Some(Node::Array(items)) => items
                    .iter()
                    .map(|id| Self::gc_profile_young_edge(young, *id))
                    .sum(),
                _ => 0,
            },
            CellTag::App
            | CellTag::Free
            | CellTag::KnownPrim
            | CellTag::RuntimePrim
            | CellTag::Int
            | CellTag::Int64
            | CellTag::Float64
            | CellTag::Float32
            | CellTag::ThreadId => 0,
        }
    }

    #[cfg(feature = "gc-phase-profile")]
    pub(in crate::runtime) fn gc_profile_young_candidate_stats(
        &self,
        marked: &[bool],
    ) -> (usize, usize, usize, usize, usize) {
        let slots = self.gc_young_profile_allocated_slots.len();
        let mut young = vec![false; marked.len()];
        for id in self.gc_young_profile_allocated_slots.iter().copied() {
            if let Some(slot) = young.get_mut(id.index()) {
                *slot = true;
            }
        }

        let live = self
            .gc_young_profile_allocated_slots
            .iter()
            .filter(|id| marked.get(id.index()).copied().unwrap_or(false))
            .count();
        let dead = slots.saturating_sub(live);
        let mut old_to_young_sources = 0usize;
        let mut old_to_young_edges = 0usize;
        for (index, cell) in self.nodes.iter().copied().enumerate() {
            if !marked.get(index).copied().unwrap_or(false)
                || young.get(index).copied().unwrap_or(false)
            {
                continue;
            }
            let edges = self.gc_profile_old_to_young_edges_for_cell(index, cell, &young);
            if edges != 0 {
                old_to_young_sources += 1;
                old_to_young_edges += edges;
            }
        }
        (slots, live, dead, old_to_young_sources, old_to_young_edges)
    }

    pub(in crate::runtime) fn mark_strict_redex(
        marked: &mut [bool],
        work: &mut Vec<NodeId>,
        redex: &StrictRedex,
    ) {
        match redex {
            StrictRedex::Root(root) => Self::mark_node_id(marked, work, *root),
            StrictRedex::Spine { root, apps, .. } => {
                Self::mark_node_id(marked, work, *root);
                for id in apps {
                    Self::mark_node_id(marked, work, *id);
                }
            }
        }
    }

    pub(in crate::runtime) fn mark_eval_frame(
        &self,
        marked: &mut [bool],
        work: &mut Vec<NodeId>,
        frame: &EvalFrame,
    ) {
        match frame {
            EvalFrame::Whnf(frame) => {
                Self::mark_strict_redex(marked, work, &frame.redex);
                match &frame.kind {
                    WhnfFrameKind::Seq { result } => Self::mark_node_id(marked, work, *result),
                    WhnfFrameKind::IoStrict { action, value } => {
                        Self::mark_node_id(marked, work, *action);
                        Self::mark_node_id(marked, work, *value);
                    }
                    WhnfFrameKind::IsInt => {}
                }
            }
            EvalFrame::Int(frame) => {
                Self::mark_strict_redex(marked, work, &frame.redex);
                match &frame.kind {
                    IntFrameKind::BinSecond { x, .. } => Self::mark_node_id(marked, work, *x),
                    IntFrameKind::BinFirst { .. } | IntFrameKind::Un { .. } => {}
                }
            }
            EvalFrame::Int64(frame) => {
                Self::mark_strict_redex(marked, work, &frame.redex);
                match &frame.kind {
                    Int64FrameKind::BinSecond { x, .. } => Self::mark_node_id(marked, work, *x),
                    Int64FrameKind::BinFirst { .. }
                    | Int64FrameKind::ShiftFirst { .. }
                    | Int64FrameKind::Un { .. } => {}
                }
            }
            EvalFrame::Int64Shift(frame) => {
                Self::mark_strict_redex(marked, work, &frame.redex);
                Self::mark_node_id(marked, work, frame.x);
            }
            EvalFrame::Float64(frame) => {
                Self::mark_strict_redex(marked, work, &frame.redex);
                if let Float64FrameKind::BinSecond { x, .. } = &frame.kind {
                    Self::mark_node_id(marked, work, *x);
                }
            }
            EvalFrame::Float32(frame) => {
                Self::mark_strict_redex(marked, work, &frame.redex);
                if let Float32FrameKind::BinSecond { x, .. } = &frame.kind {
                    Self::mark_node_id(marked, work, *x);
                }
            }
            EvalFrame::Bytes(frame) => {
                Self::mark_strict_redex(marked, work, &frame.redex);
                match &frame.kind {
                    BytesFrameKind::BinSecond { x, .. } => Self::mark_node_id(marked, work, *x),
                    BytesFrameKind::BinFirst { y, .. } => Self::mark_node_id(marked, work, *y),
                }
            }
            EvalFrame::Conversion(frame) => {
                Self::mark_strict_redex(marked, work, &frame.redex);
            }
        }
    }

    pub(in crate::runtime) fn mark_eval_stack(
        &self,
        marked: &mut [bool],
        work: &mut Vec<NodeId>,
        stack: &EvalFrameStack,
    ) {
        if let Some(frame) = &stack.top {
            self.mark_eval_frame(marked, work, frame);
        }
        for frame in &stack.rest {
            self.mark_eval_frame(marked, work, frame);
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

    pub(in crate::runtime) fn mark_persistent_spine(
        marked: &mut [bool],
        work: &mut Vec<NodeId>,
        spine: &PersistentSpine,
    ) {
        for id in &spine.apps {
            Self::mark_node_id(marked, work, *id);
        }
    }

    pub(in crate::runtime) fn mark_program_roots(
        &self,
        marked: &mut [bool],
        work: &mut Vec<NodeId>,
        current_root: NodeId,
        frame_stack: &EvalFrameStack,
        eval_spine: &EvalSpine,
        persistent_spine: &PersistentSpine,
        scratch_args: &[NodeId],
        scratch_apps: &[NodeId],
        machine_stack: Option<&EvalStack>,
    ) {
        Self::mark_node_id(marked, work, self.root);
        Self::mark_node_id(marked, work, current_root);
        for id in self.stable_ptrs.iter().flatten() {
            Self::mark_node_id(marked, work, *id);
        }
        for id in &self.pending_weak_finalizers {
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
        self.mark_eval_stack(marked, work, frame_stack);
        if let Some(machine_stack) = machine_stack {
            self.mark_machine_stack(marked, work, machine_stack);
        }
        Self::mark_eval_spine(marked, work, eval_spine);
        Self::mark_persistent_spine(marked, work, persistent_spine);
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
                    let Some(next) = self.nodes[current.index()].option_id_word1() else {
                        return None;
                    };
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
            self.set_cell_at(id.index(), Cell::indir(Some(current)));
        }
        Some(current)
    }

    pub(in crate::runtime) fn canonical_gc_target(&mut self, id: NodeId) -> Option<NodeId> {
        let target = if matches!(
            self.nodes.get(id.index()).map(|cell| cell.tag()),
            Some(CellTag::Indir)
        ) {
            self.compress_marked_indirection(id)?
        } else {
            id
        };
        if let Some(value) = self.cell_int_value(target) {
            if let Some(slot) = small_int_index(value) {
                if let Some(canonical) = self.small_ints[slot] {
                    if canonical != target {
                        self.set_cell_at(target.index(), Cell::indir(Some(canonical)));
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

    #[cfg(any(feature = "gc-phase-profile", feature = "eval-phase-profile"))]
    pub(in crate::runtime) fn gc_profile_resolved_id(&self, id: NodeId) -> Option<NodeId> {
        let mut current = id;
        for _ in 0..self.nodes.len() {
            let cell = *self.nodes.get(current.index())?;
            if !cell.has_tag(CellTag::Indir) {
                return (!cell.has_tag(CellTag::Free)).then_some(current);
            }
            current = cell.option_id_word1()?;
        }
        None
    }

    #[cfg(any(feature = "gc-phase-profile", feature = "eval-phase-profile"))]
    pub(in crate::runtime) fn gc_profile_prim(&self, id: NodeId) -> Option<Prim> {
        let id = self.gc_profile_resolved_id(id)?;
        self.nodes.get(id.index())?.prim()
    }

    #[cfg(any(feature = "gc-phase-profile", feature = "eval-phase-profile"))]
    pub(in crate::runtime) fn gc_profile_app_fields(&self, id: NodeId) -> Option<(NodeId, NodeId)> {
        let id = self.gc_profile_resolved_id(id)?;
        self.nodes.get(id.index())?.app_fields()
    }

    #[cfg(any(feature = "gc-phase-profile", feature = "eval-phase-profile"))]
    pub(in crate::runtime) fn gc_profile_flipped_prim(prim: Prim) -> Option<Prim> {
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

    #[cfg(feature = "gc-phase-profile")]
    pub(in crate::runtime) fn profile_gc_red_opportunities(&mut self, fun: NodeId, arg: NodeId) {
        use KnownPrim::*;

        let funt = self.gc_profile_prim(fun);
        let argt = self.gc_profile_prim(arg);
        let fun_app = self.gc_profile_app_fields(fun);
        let funfunt = fun_app.and_then(|(fun_fun, _)| self.gc_profile_prim(fun_fun));
        let arg_app = self.gc_profile_app_fields(arg);
        let arg_fun_t = arg_app.and_then(|(arg_fun, _)| self.gc_profile_prim(arg_fun));

        if funt == Some(Prim::Known(I)) {
            self.gc_red_i_opportunities += 1;
        }
        if funfunt == Some(Prim::Known(K)) {
            self.gc_red_k_opportunities += 1;
        }
        if funfunt == Some(Prim::Known(A)) {
            self.gc_red_a_opportunities += 1;
        }
        if funt == Some(Prim::Known(B)) && argt == Some(Prim::Known(I)) {
            self.gc_red_bi_opportunities += 1;
        }
        if funfunt == Some(Prim::Known(B)) && argt == Some(Prim::Known(I)) {
            self.gc_red_bxi_opportunities += 1;
        }
        if funfunt == Some(Prim::Known(CPrimeB)) && argt == Some(Prim::Known(I)) {
            self.gc_red_ccbi_opportunities += 1;
        }
        if funt == Some(Prim::Known(C)) && arg_fun_t == Some(Prim::Known(C)) {
            self.gc_red_cc_opportunities += 1;
        }
        if funt == Some(Prim::Known(CPrime)) && argt == Some(Prim::Known(I)) {
            self.gc_red_cci_opportunities += 1;
        }
        if funt == Some(Prim::Known(CPrimeB)) {
            if let Some((arg_fun, arg_arg)) = arg_app {
                let fun_arg_is_p = self.gc_profile_prim(arg_arg) == Some(Prim::Known(P));
                let fun_arg_is_bc = self
                    .gc_profile_app_fields(arg_fun)
                    .map(|(bc_fun, bc_arg)| {
                        self.gc_profile_prim(bc_fun) == Some(Prim::Known(B))
                            && self.gc_profile_prim(bc_arg) == Some(Prim::Known(C))
                    })
                    .unwrap_or(false);
                if fun_arg_is_p && fun_arg_is_bc {
                    self.gc_red_ccbbcp_opportunities += 1;
                }
            }
        }
        if funt == Some(Prim::Known(C)) && argt.and_then(Self::gc_profile_flipped_prim).is_some() {
            self.gc_red_flip_opportunities += 1;
        }
    }

    pub(in crate::runtime) fn mark_reachable(
        &mut self,
        marked: &mut [bool],
        work: &mut Vec<NodeId>,
        foreign_finalizer_marked: &mut [bool],
    ) {
        while let Some(id) = work.pop() {
            if matches!(
                self.nodes.get(id.index()).map(|cell| cell.tag()),
                Some(CellTag::Indir)
            ) {
                if let Some(target) = self.compress_marked_indirection(id) {
                    Self::mark_node_id(marked, work, target);
                }
                continue;
            }
            let Some(cell) = self.nodes.get(id.index()).copied() else {
                continue;
            };
            if let Some((fun, arg)) = cell.app_fields() {
                let fun = self.mark_canonical_child(marked, work, fun);
                let arg = self.mark_canonical_child(marked, work, arg);
                #[cfg(feature = "gc-phase-profile")]
                self.profile_gc_red_opportunities(fun, arg);
                if fun != cell.id_payload() || arg != cell.id_word1() {
                    self.set_app_cell_at(id.index(), Cell::app(fun, arg));
                }
                continue;
            }
            match cell.tag() {
                CellTag::Ptr | CellTag::RawFunPtr => {
                    if let Some(ptr) = cell.pointer_payload() {
                        self.mark_pointer_target(marked, work, ptr);
                    }
                }
                CellTag::Cold => match self.cold_node(id) {
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
                },
                CellTag::App
                | CellTag::Indir
                | CellTag::Free
                | CellTag::KnownPrim
                | CellTag::RuntimePrim
                | CellTag::Int
                | CellTag::Int64
                | CellTag::Float64
                | CellTag::Float32
                | CellTag::ThreadId => {}
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

    pub(in crate::runtime) fn sweep_weaks_after_mark(
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
            self.mark_reachable(marked, work, foreign_finalizer_marked);
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
            self.mark_reachable(marked, work, foreign_finalizer_marked);
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

    pub(in crate::runtime) fn collect_garbage_between_steps(
        &mut self,
        current_root: NodeId,
        frame_stack: &EvalFrameStack,
        eval_spine: &EvalSpine,
        persistent_spine: &PersistentSpine,
        scratch_args: &[NodeId],
        scratch_apps: &[NodeId],
        machine_stack: Option<&EvalStack>,
    ) -> Result<usize, EvalError> {
        let started = Instant::now();
        let allocations_since_collect = self.gc_allocations_since_collect;
        let mut marked = std::mem::take(&mut self.gc_marked);
        marked.clear();
        marked.resize(self.nodes.len(), false);
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
            frame_stack,
            eval_spine,
            persistent_spine,
            scratch_args,
            scratch_apps,
            machine_stack,
        );
        self.mark_reachable(&mut marked, &mut work, &mut foreign_finalizer_marked);
        let weak_finalizers =
            self.sweep_weaks_after_mark(&mut marked, &mut work, &mut foreign_finalizer_marked);
        #[cfg(feature = "gc-phase-profile")]
        let mark_nanos = mark_started.elapsed().as_nanos();
        work.clear();
        self.labels
            .retain(|_, id| marked.get(id.index()).copied().unwrap_or(false));
        self.run_dead_foreign_finalizers(&foreign_finalizer_marked)?;
        #[cfg(feature = "gc-phase-profile")]
        let (
            young_profile_slots,
            young_profile_live,
            young_profile_dead,
            young_profile_old_to_young_sources,
            young_profile_old_to_young_edges,
        ) = self.gc_profile_young_candidate_stats(&marked);

        #[cfg(feature = "gc-phase-profile")]
        let sweep_started = Instant::now();
        self.free_head = None;
        self.free_nodes = 0;
        let mut freed = 0;
        let mut live = 0;
        for index in 0..marked.len() {
            if marked[index] {
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
            self.gc_young_profile_last_slots = young_profile_slots;
            self.gc_young_profile_last_live = young_profile_live;
            self.gc_young_profile_last_dead = young_profile_dead;
            self.gc_young_profile_last_old_to_young_sources = young_profile_old_to_young_sources;
            self.gc_young_profile_last_old_to_young_edges = young_profile_old_to_young_edges;
            self.gc_young_profile_total_slots = self
                .gc_young_profile_total_slots
                .saturating_add(young_profile_slots);
            self.gc_young_profile_total_live = self
                .gc_young_profile_total_live
                .saturating_add(young_profile_live);
            self.gc_young_profile_total_dead = self
                .gc_young_profile_total_dead
                .saturating_add(young_profile_dead);
            self.gc_young_profile_total_old_to_young_sources = self
                .gc_young_profile_total_old_to_young_sources
                .saturating_add(young_profile_old_to_young_sources);
            self.gc_young_profile_total_old_to_young_edges = self
                .gc_young_profile_total_old_to_young_edges
                .saturating_add(young_profile_old_to_young_edges);
        }
        self.gc_last_allocations_since_collect = allocations_since_collect;
        self.gc_allocations_since_collect = 0;
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
            #[cfg(feature = "gc-phase-profile")]
            young_profile_slots,
            #[cfg(feature = "gc-phase-profile")]
            young_profile_live,
            #[cfg(feature = "gc-phase-profile")]
            young_profile_dead,
            #[cfg(feature = "gc-phase-profile")]
            young_profile_old_to_young_sources,
            #[cfg(feature = "gc-phase-profile")]
            young_profile_old_to_young_edges,
        });
        #[cfg(feature = "gc-phase-profile")]
        self.gc_young_profile_allocated_slots.clear();
        self.pending_weak_finalizers.extend(weak_finalizers);
        while let Some(finalizer) = self.pending_weak_finalizers.pop() {
            self.reduce_node_whnf(finalizer, FORCE_REDUCTION_LIMIT)?;
        }
        Ok(freed)
    }

    pub(in crate::runtime) fn maybe_collect_garbage_between_steps(
        &mut self,
        current_root: NodeId,
        frame_stack: &EvalFrameStack,
        eval_spine: &EvalSpine,
        persistent_spine: &PersistentSpine,
        scratch_args: &[NodeId],
        scratch_apps: &[NodeId],
        machine_stack: Option<&EvalStack>,
    ) -> Result<(), EvalError> {
        if self.gc_node_interval == 0 {
            return Ok(());
        }
        if self.reduce_depth != 1 {
            return Ok(());
        }
        if self.gc_allocations_since_collect < self.gc_node_interval {
            return Ok(());
        }
        self.collect_garbage_between_steps(
            current_root,
            frame_stack,
            eval_spine,
            persistent_spine,
            scratch_args,
            scratch_apps,
            machine_stack,
        )?;
        Ok(())
    }
}
