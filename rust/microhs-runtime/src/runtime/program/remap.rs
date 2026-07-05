//! Mutable NodeId remapping for future moving GC.

use super::*;

impl Program {
    pub(in crate::runtime) fn remap_node_ids_for_moving_gc(
        &mut self,
        current_root: &mut NodeId,
        frame_stack: &mut EvalFrameStack,
        eval_spine: &mut EvalSpine,
        persistent_spine: &mut PersistentSpine,
        scratch_args: &mut [NodeId],
        scratch_apps: &mut [NodeId],
        machine_stack: Option<&mut EvalStack>,
        mut remap: impl FnMut(NodeId) -> NodeId,
    ) {
        for cell in &mut self.nodes {
            remap_cell_node_ids(cell, &mut remap);
        }
        for node in self.cold_nodes.iter_mut().flatten() {
            remap_node_payload_ids(node, &mut remap);
        }

        remap_id(&mut self.root, &mut remap);
        remap_id(current_root, &mut remap);
        for id in self.labels.values_mut() {
            remap_id(id, &mut remap);
        }
        for id in &mut self.node_pointers {
            remap_id(id, &mut remap);
        }
        self.node_pointer_slots.clear();
        self.node_pointer_slots.extend(
            self.node_pointers
                .iter()
                .copied()
                .enumerate()
                .map(|(slot, id)| (id, slot)),
        );
        for id in &mut self.gc_mark_work {
            remap_id(id, &mut remap);
        }
        for id in &mut self.stable_ptrs {
            remap_option_id(id, &mut remap);
        }
        for id in &mut self.weak_nodes {
            remap_id(id, &mut remap);
        }
        for id in &mut self.pending_weak_finalizers {
            remap_id(id, &mut remap);
        }
        for bfile in self.bfiles.iter_mut().flatten() {
            remap_bfile_node_ids(bfile, &mut remap);
        }
        remap_option_id(&mut self.arg_ref_array, &mut remap);
        remap_option_id(&mut self.world, &mut remap);
        remap_prim_cache_node_ids(&mut self.prim_cache, &mut remap);
        remap_compound_cache_node_ids(&mut self.compound_cache, &mut remap);
        for id in &mut self.small_ints {
            remap_option_id(id, &mut remap);
        }

        remap_eval_frame_stack_node_ids(frame_stack, &mut remap);
        remap_eval_spine_node_ids(eval_spine, &mut remap);
        remap_persistent_spine_node_ids(persistent_spine, &mut remap);
        for id in scratch_args.iter_mut().chain(scratch_apps) {
            remap_id(id, &mut remap);
        }
        if let Some(machine_stack) = machine_stack {
            remap_eval_stack_node_ids(machine_stack, &mut remap);
        }
    }
}

fn remap_id(id: &mut NodeId, remap: &mut impl FnMut(NodeId) -> NodeId) {
    *id = remap(*id);
}

fn remap_option_id(id: &mut Option<NodeId>, remap: &mut impl FnMut(NodeId) -> NodeId) {
    if let Some(id) = id {
        remap_id(id, remap);
    }
}

#[cfg(feature = "profile")]
fn remap_profile_head(head: &mut ProfileHead, remap: &mut impl FnMut(NodeId) -> NodeId) {
    remap_option_id(&mut head.0, remap);
}

#[cfg(not(feature = "profile"))]
fn remap_profile_head(_head: &mut ProfileHead, _remap: &mut impl FnMut(NodeId) -> NodeId) {}

fn remapped_option_id(
    id: Option<NodeId>,
    remap: &mut impl FnMut(NodeId) -> NodeId,
) -> Option<NodeId> {
    id.map(remap)
}

fn remap_cell_node_ids(cell: &mut Cell, remap: &mut impl FnMut(NodeId) -> NodeId) {
    match cell.tag() {
        CellTag::App => {
            *cell = Cell::app(remap(cell.id_payload()), remap(cell.id_word1()));
        }
        CellTag::Indir => {
            *cell = Cell::indir(remapped_option_id(cell.option_id_word1(), remap));
        }
        CellTag::Free => {
            // Free cells are not live roots; a moving collector must rebuild free state.
        }
        CellTag::KnownPrim
        | CellTag::RuntimePrim
        | CellTag::Int
        | CellTag::Float32
        | CellTag::ThreadId
        | CellTag::Cold => {}
    }
}

fn remap_node_payload_ids(node: &mut Node, remap: &mut impl FnMut(NodeId) -> NodeId) {
    match node {
        Node::App(fun, arg) => {
            remap_id(fun, remap);
            remap_id(arg, remap);
        }
        Node::Indir(target) | Node::MVar(target) => remap_option_id(target, remap),
        Node::Free(_) => {}
        Node::BytesView(view) => remap_id(&mut view.base, remap),
        Node::Array(items) => {
            for id in items.iter_mut() {
                remap_id(id, remap);
            }
        }
        Node::Weak(weak) => {
            remap_option_id(&mut weak.key, remap);
            remap_option_id(&mut weak.value, remap);
            remap_option_id(&mut weak.finalizer, remap);
        }
        Node::Prim(_)
        | Node::Int(_)
        | Node::Int64(_)
        | Node::Float64(_)
        | Node::Float32(_)
        | Node::ThreadId(_)
        | Node::Ptr(_)
        | Node::RawFunPtr(_)
        | Node::ForeignPtr(_)
        | Node::BigInt(_)
        | Node::Bytes(_)
        | Node::MutableBytes(_)
        | Node::Ffi(_)
        | Node::JsCall(_)
        | Node::JsWrap { .. }
        | Node::FunPtr(_)
        | Node::Tick(_) => {}
    }
}

fn remap_bfile_node_ids(bfile: &mut BFile, remap: &mut impl FnMut(NodeId) -> NodeId) {
    if let BFileKind::ReadOnlyMemoryView { base, .. } = &mut bfile.kind {
        remap_id(base, remap);
    }
}

fn remap_prim_cache_node_ids(cache: &mut PrimCache, remap: &mut impl FnMut(NodeId) -> NodeId) {
    for id in [
        &mut cache.a,
        &mut cache.b,
        &mut cache.c,
        &mut cache.i,
        &mut cache.k,
        &mut cache.k2,
        &mut cache.k3,
        &mut cache.o,
        &mut cache.p,
        &mut cache.u,
        &mut cache.y,
        &mut cache.z,
        &mut cache.io_bind,
        &mut cache.io_perform_io,
    ] {
        remap_option_id(id, remap);
    }
}

fn remap_compound_cache_node_ids(
    cache: &mut CompoundCache,
    remap: &mut impl FnMut(NodeId) -> NodeId,
) {
    for id in [
        &mut cache.fst,
        &mut cache.snd,
        &mut cache.just,
        &mut cache.pair_unit,
    ] {
        remap_option_id(id, remap);
    }
}

fn remap_strict_redex_node_ids(redex: &mut StrictRedex, remap: &mut impl FnMut(NodeId) -> NodeId) {
    match redex {
        StrictRedex::Root(root) => remap_id(root, remap),
        StrictRedex::Spine { root, apps, .. } => {
            remap_id(root, remap);
            for id in apps {
                remap_id(id, remap);
            }
        }
    }
}

fn remap_whnf_frame_kind_node_ids(
    kind: &mut WhnfFrameKind,
    remap: &mut impl FnMut(NodeId) -> NodeId,
) {
    match kind {
        WhnfFrameKind::Seq { result } => remap_id(result, remap),
        WhnfFrameKind::IoStrict { action, value } => {
            remap_id(action, remap);
            remap_id(value, remap);
        }
        WhnfFrameKind::IsInt => {}
    }
}

fn remap_int_frame_kind_node_ids(
    kind: &mut IntFrameKind,
    remap: &mut impl FnMut(NodeId) -> NodeId,
) {
    if let IntFrameKind::BinSecond { x, .. } = kind {
        remap_id(x, remap);
    }
}

fn remap_int64_frame_kind_node_ids(
    kind: &mut Int64FrameKind,
    remap: &mut impl FnMut(NodeId) -> NodeId,
) {
    if let Int64FrameKind::BinSecond { x, .. } = kind {
        remap_id(x, remap);
    }
}

fn remap_float64_frame_kind_node_ids(
    kind: &mut Float64FrameKind,
    remap: &mut impl FnMut(NodeId) -> NodeId,
) {
    if let Float64FrameKind::BinSecond { x, .. } = kind {
        remap_id(x, remap);
    }
}

fn remap_float32_frame_kind_node_ids(
    kind: &mut Float32FrameKind,
    remap: &mut impl FnMut(NodeId) -> NodeId,
) {
    if let Float32FrameKind::BinSecond { x, .. } = kind {
        remap_id(x, remap);
    }
}

fn remap_bytes_frame_kind_node_ids(
    kind: &mut BytesFrameKind,
    remap: &mut impl FnMut(NodeId) -> NodeId,
) {
    match kind {
        BytesFrameKind::BinSecond { x, .. } => remap_id(x, remap),
        BytesFrameKind::BinFirst { y, .. } => remap_id(y, remap),
    }
}

fn remap_eval_frame_node_ids(frame: &mut EvalFrame, remap: &mut impl FnMut(NodeId) -> NodeId) {
    match frame {
        EvalFrame::Whnf(frame) => {
            remap_strict_redex_node_ids(&mut frame.redex, remap);
            remap_profile_head(&mut frame.profile_head, remap);
            remap_whnf_frame_kind_node_ids(&mut frame.kind, remap);
        }
        EvalFrame::Int(frame) => {
            remap_strict_redex_node_ids(&mut frame.redex, remap);
            remap_profile_head(&mut frame.profile_head, remap);
            remap_int_frame_kind_node_ids(&mut frame.kind, remap);
        }
        EvalFrame::Int64(frame) => {
            remap_strict_redex_node_ids(&mut frame.redex, remap);
            remap_profile_head(&mut frame.profile_head, remap);
            remap_int64_frame_kind_node_ids(&mut frame.kind, remap);
        }
        EvalFrame::Int64Shift(frame) => {
            remap_strict_redex_node_ids(&mut frame.redex, remap);
            remap_profile_head(&mut frame.profile_head, remap);
            remap_id(&mut frame.x, remap);
        }
        EvalFrame::Float64(frame) => {
            remap_strict_redex_node_ids(&mut frame.redex, remap);
            remap_profile_head(&mut frame.profile_head, remap);
            remap_float64_frame_kind_node_ids(&mut frame.kind, remap);
        }
        EvalFrame::Float32(frame) => {
            remap_strict_redex_node_ids(&mut frame.redex, remap);
            remap_profile_head(&mut frame.profile_head, remap);
            remap_float32_frame_kind_node_ids(&mut frame.kind, remap);
        }
        EvalFrame::Bytes(frame) => {
            remap_strict_redex_node_ids(&mut frame.redex, remap);
            remap_profile_head(&mut frame.profile_head, remap);
            remap_bytes_frame_kind_node_ids(&mut frame.kind, remap);
        }
        EvalFrame::Conversion(frame) => {
            remap_strict_redex_node_ids(&mut frame.redex, remap);
            remap_profile_head(&mut frame.profile_head, remap);
        }
    }
}

fn remap_eval_frame_stack_node_ids(
    stack: &mut EvalFrameStack,
    remap: &mut impl FnMut(NodeId) -> NodeId,
) {
    if let Some(frame) = &mut stack.top {
        remap_eval_frame_node_ids(frame, remap);
    }
    for frame in &mut stack.rest {
        remap_eval_frame_node_ids(frame, remap);
    }
}

fn remap_eval_spine_node_ids(spine: &mut EvalSpine, remap: &mut impl FnMut(NodeId) -> NodeId) {
    if spine.heap {
        for id in spine.heap_args.iter_mut().chain(&mut spine.heap_apps) {
            remap_id(id, remap);
        }
    } else {
        for index in 0..spine.inline_len {
            // SAFETY: indices below inline_len were initialized by EvalSpine::push_desc.
            remap_id(unsafe { spine.inline_args[index].assume_init_mut() }, remap);
            // SAFETY: indices below inline_len were initialized by EvalSpine::push_desc.
            remap_id(unsafe { spine.inline_apps[index].assume_init_mut() }, remap);
        }
    }
}

fn remap_persistent_spine_node_ids(
    spine: &mut PersistentSpine,
    remap: &mut impl FnMut(NodeId) -> NodeId,
) {
    for id in &mut spine.apps {
        remap_id(id, remap);
    }
}

fn remap_stack_frame_node_ids(frame: &mut StackFrame, remap: &mut impl FnMut(NodeId) -> NodeId) {
    match frame {
        StackFrame::Whnf(frame) => {
            remap_id(&mut frame.redex, remap);
            remap_profile_head(&mut frame.profile_head, remap);
            remap_whnf_frame_kind_node_ids(&mut frame.kind, remap);
        }
        StackFrame::Int(frame) => {
            remap_id(&mut frame.redex, remap);
            remap_profile_head(&mut frame.profile_head, remap);
            remap_int_frame_kind_node_ids(&mut frame.kind, remap);
        }
        StackFrame::Int64(frame) => {
            remap_profile_head(&mut frame.profile_head, remap);
            remap_int64_frame_kind_node_ids(&mut frame.kind, remap);
        }
        StackFrame::Int64Shift(frame) => {
            remap_profile_head(&mut frame.profile_head, remap);
            remap_id(&mut frame.x, remap);
        }
        StackFrame::Float64(frame) => {
            remap_profile_head(&mut frame.profile_head, remap);
            remap_float64_frame_kind_node_ids(&mut frame.kind, remap);
        }
        StackFrame::Float32(frame) => {
            remap_profile_head(&mut frame.profile_head, remap);
            remap_float32_frame_kind_node_ids(&mut frame.kind, remap);
        }
        StackFrame::Bytes(frame) => {
            remap_profile_head(&mut frame.profile_head, remap);
            remap_bytes_frame_kind_node_ids(&mut frame.kind, remap);
        }
        StackFrame::Conversion(frame) => {
            remap_profile_head(&mut frame.profile_head, remap);
        }
    }
}

fn remap_eval_stack_node_ids(stack: &mut EvalStack, remap: &mut impl FnMut(NodeId) -> NodeId) {
    for id in &mut stack.apps {
        remap_id(id, remap);
    }
    for frame in &mut stack.frames {
        remap_stack_frame_node_ids(frame, remap);
    }
}
