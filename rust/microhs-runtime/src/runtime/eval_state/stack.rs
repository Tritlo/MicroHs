//! Explicit WHNF application stack and strict-frame stack operations.
use super::*;

pub(in crate::runtime) struct EvalLoopStep {
    pub(in crate::runtime) node: NodeId,
    pub(in crate::runtime) reductions: usize,
}

/// Explicit WHNF machine stack.
///
/// `apps` is the current application spine and `frames` holds strict primitive
/// continuations. `app_base` lets nested strict frames reserve their own app
/// segment without copying the outer spine.
#[derive(Default)]
pub(in crate::runtime) struct EvalStack {
    pub(in crate::runtime) apps: Vec<NodeId>,
    pub(in crate::runtime) frames: Vec<StackFrame>,
    pub(in crate::runtime) app_base: usize,
}

/// Result of one bounded stack-reducer slice.
pub(in crate::runtime) enum StackStep {
    Reduced {
        node: NodeId,
        reductions: usize,
    },
    Whnf {
        node: NodeId,
        head: NodeId,
        reductions: usize,
    },
    Fallback {
        root: NodeId,
        head: NodeId,
        reductions: usize,
    },
}

impl StackFrame {
    pub(in crate::runtime) fn prev_app_base(&self) -> usize {
        match self {
            Self::Whnf(frame) => frame.prev_app_base,
            Self::Int(frame) => frame.prev_app_base,
            Self::Int64(frame) => frame.prev_app_base,
            Self::Int64Shift(frame) => frame.prev_app_base,
            Self::Float64(frame) => frame.prev_app_base,
            Self::Float32(frame) => frame.prev_app_base,
            Self::Bytes(frame) => frame.prev_app_base,
            Self::Conversion(frame) => frame.prev_app_base,
        }
    }
}

impl EvalStack {
    pub(in crate::runtime) fn push_app(&mut self, app: NodeId) {
        self.apps.push(app);
    }

    pub(in crate::runtime) fn app_unchecked(&self, index: usize) -> NodeId {
        debug_assert!(index < self.apps.len());
        unsafe { *self.apps.get_unchecked(index) }
    }

    pub(in crate::runtime) fn app_base(&self) -> usize {
        self.app_base
    }

    pub(in crate::runtime) fn app_len(&self) -> usize {
        self.apps.len() - self.app_base()
    }

    pub(in crate::runtime) fn arg(&self, nodes: &[Cell], index: usize) -> NodeId {
        let app_index = self.apps.len() - index - 1;
        self.arg_at_app(nodes, app_index)
    }

    pub(in crate::runtime) fn arg_at_app(&self, nodes: &[Cell], app_index: usize) -> NodeId {
        debug_assert!(app_index < self.apps.len());
        // The stack app segment is built only by descending through App
        // cells. Match eval.c's ARG(TOP(i)) discipline in the hot path.
        unsafe {
            let app = *self.apps.get_unchecked(app_index);
            nodes.get_unchecked(app.index()).id_word1()
        }
    }

    pub(in crate::runtime) fn arg_from_app(nodes: &[Cell], app: NodeId) -> NodeId {
        debug_assert!(app.index() < nodes.len());
        debug_assert_eq!(nodes[app.index()].tag(), CellTag::App);
        unsafe { nodes.get_unchecked(app.index()).id_word1() }
    }

    pub(in crate::runtime) fn take_args1(&mut self, nodes: &[Cell]) -> (NodeId, NodeId) {
        let end = self.apps.len();
        debug_assert!(self.app_len() >= 1);
        let redex_index = end - 1;
        let redex = unsafe { *self.apps.get_unchecked(redex_index) };
        let x = Self::arg_from_app(nodes, redex);
        self.apps.truncate(redex_index);
        (redex, x)
    }

    pub(in crate::runtime) fn take_args2(&mut self, nodes: &[Cell]) -> (NodeId, NodeId, NodeId) {
        let end = self.apps.len();
        debug_assert!(self.app_len() >= 2);
        let redex_index = end - 2;
        let x_app = unsafe { *self.apps.get_unchecked(end - 1) };
        let redex = unsafe { *self.apps.get_unchecked(redex_index) };
        let x = Self::arg_from_app(nodes, x_app);
        let y = Self::arg_from_app(nodes, redex);
        self.apps.truncate(redex_index);
        (redex, x, y)
    }

    pub(in crate::runtime) fn take_args3(
        &mut self,
        nodes: &[Cell],
    ) -> (NodeId, NodeId, NodeId, NodeId) {
        let end = self.apps.len();
        debug_assert!(self.app_len() >= 3);
        let redex_index = end - 3;
        let x_app = unsafe { *self.apps.get_unchecked(end - 1) };
        let y_app = unsafe { *self.apps.get_unchecked(end - 2) };
        let redex = unsafe { *self.apps.get_unchecked(redex_index) };
        let x = Self::arg_from_app(nodes, x_app);
        let y = Self::arg_from_app(nodes, y_app);
        let z = Self::arg_from_app(nodes, redex);
        self.apps.truncate(redex_index);
        (redex, x, y, z)
    }

    pub(in crate::runtime) fn take_args4(
        &mut self,
        nodes: &[Cell],
    ) -> (NodeId, NodeId, NodeId, NodeId, NodeId) {
        let end = self.apps.len();
        debug_assert!(self.app_len() >= 4);
        let redex_index = end - 4;
        let x_app = unsafe { *self.apps.get_unchecked(end - 1) };
        let y_app = unsafe { *self.apps.get_unchecked(end - 2) };
        let z_app = unsafe { *self.apps.get_unchecked(end - 3) };
        let redex = unsafe { *self.apps.get_unchecked(redex_index) };
        let x = Self::arg_from_app(nodes, x_app);
        let y = Self::arg_from_app(nodes, y_app);
        let z = Self::arg_from_app(nodes, z_app);
        let w = Self::arg_from_app(nodes, redex);
        self.apps.truncate(redex_index);
        (redex, x, y, z, w)
    }

    pub(in crate::runtime) fn take_args5(
        &mut self,
        nodes: &[Cell],
    ) -> (NodeId, NodeId, NodeId, NodeId, NodeId, NodeId) {
        let end = self.apps.len();
        debug_assert!(self.app_len() >= 5);
        let redex_index = end - 5;
        let x_app = unsafe { *self.apps.get_unchecked(end - 1) };
        let y_app = unsafe { *self.apps.get_unchecked(end - 2) };
        let z_app = unsafe { *self.apps.get_unchecked(end - 3) };
        let w_app = unsafe { *self.apps.get_unchecked(end - 4) };
        let redex = unsafe { *self.apps.get_unchecked(redex_index) };
        let x = Self::arg_from_app(nodes, x_app);
        let y = Self::arg_from_app(nodes, y_app);
        let z = Self::arg_from_app(nodes, z_app);
        let w = Self::arg_from_app(nodes, w_app);
        let v = Self::arg_from_app(nodes, redex);
        self.apps.truncate(redex_index);
        (redex, x, y, z, w, v)
    }

    pub(in crate::runtime) fn outer_root(&self, head: NodeId) -> NodeId {
        let base = self.app_base();
        if base == self.apps.len() {
            return head;
        }
        self.apps[base]
    }

    pub(in crate::runtime) fn has_frame_below_apps(&self) -> bool {
        !self.frames.is_empty()
    }

    pub(in crate::runtime) fn top_is_frame(&self) -> bool {
        !self.frames.is_empty() && self.apps.len() == self.app_base
    }

    pub(in crate::runtime) fn peek_frame(&self) -> Option<&StackFrame> {
        if self.top_is_frame() {
            self.frames.last()
        } else {
            None
        }
    }

    pub(in crate::runtime) fn push_whnf_frame(
        &mut self,
        redex: NodeId,
        used: usize,
        profile_head: ProfileHead,
        kind: WhnfFrameKind,
    ) {
        let prev_app_base = self.app_base;
        self.frames.push(StackFrame::Whnf(StackWhnfFrame {
            prev_app_base,
            redex,
            used,
            profile_head,
            kind,
        }));
        self.app_base = self.apps.len();
    }

    pub(in crate::runtime) fn push_int_frame(
        &mut self,
        redex: NodeId,
        profile_head: ProfileHead,
        kind: IntFrameKind,
    ) {
        let prev_app_base = self.app_base;
        self.frames.push(StackFrame::Int(StackIntFrame {
            prev_app_base,
            redex,
            profile_head,
            kind,
        }));
        self.app_base = self.apps.len();
    }

    pub(in crate::runtime) fn push_int64_frame(
        &mut self,
        app_end: usize,
        used: usize,
        profile_head: ProfileHead,
        kind: Int64FrameKind,
    ) {
        let prev_app_base = self.app_base;
        self.frames.push(StackFrame::Int64(StackInt64Frame {
            prev_app_base,
            app_end,
            used,
            profile_head,
            kind,
        }));
        self.app_base = self.apps.len();
    }

    pub(in crate::runtime) fn push_int64_shift_frame(
        &mut self,
        app_end: usize,
        used: usize,
        profile_head: ProfileHead,
        op: Int64BinOp,
        x: NodeId,
    ) {
        let prev_app_base = self.app_base;
        self.frames
            .push(StackFrame::Int64Shift(StackInt64ShiftFrame {
                prev_app_base,
                app_end,
                used,
                profile_head,
                op,
                x,
            }));
        self.app_base = self.apps.len();
    }

    pub(in crate::runtime) fn push_float64_frame(
        &mut self,
        app_end: usize,
        used: usize,
        profile_head: ProfileHead,
        kind: Float64FrameKind,
    ) {
        let prev_app_base = self.app_base;
        self.frames.push(StackFrame::Float64(StackFloat64Frame {
            prev_app_base,
            app_end,
            used,
            profile_head,
            kind,
        }));
        self.app_base = self.apps.len();
    }

    pub(in crate::runtime) fn push_float32_frame(
        &mut self,
        app_end: usize,
        used: usize,
        profile_head: ProfileHead,
        kind: Float32FrameKind,
    ) {
        let prev_app_base = self.app_base;
        self.frames.push(StackFrame::Float32(StackFloat32Frame {
            prev_app_base,
            app_end,
            used,
            profile_head,
            kind,
        }));
        self.app_base = self.apps.len();
    }

    pub(in crate::runtime) fn push_bytes_frame(
        &mut self,
        app_end: usize,
        used: usize,
        profile_head: ProfileHead,
        kind: BytesFrameKind,
    ) {
        let prev_app_base = self.app_base;
        self.frames.push(StackFrame::Bytes(StackBytesFrame {
            prev_app_base,
            app_end,
            used,
            profile_head,
            kind,
        }));
        self.app_base = self.apps.len();
    }

    pub(in crate::runtime) fn push_conversion_frame(
        &mut self,
        app_end: usize,
        used: usize,
        profile_head: ProfileHead,
        kind: ConversionFrameKind,
    ) {
        let prev_app_base = self.app_base;
        self.frames
            .push(StackFrame::Conversion(StackConversionFrame {
                prev_app_base,
                app_end,
                used,
                profile_head,
                kind,
            }));
        self.app_base = self.apps.len();
    }

    pub(in crate::runtime) fn pop_frame(&mut self) -> Option<StackFrame> {
        if self.top_is_frame() {
            let frame = self.frames.pop().expect("frame marker must have payload");
            self.app_base = frame.prev_app_base();
            Some(frame)
        } else {
            None
        }
    }

    pub(in crate::runtime) fn write_args_head_order(
        &self,
        nodes: &[Cell],
        args: &mut Vec<NodeId>,
    ) -> Result<(), EvalError> {
        args.clear();
        args.reserve(self.app_len());
        for index in 0..self.app_len() {
            args.push(self.arg(nodes, index));
        }
        Ok(())
    }

    pub(in crate::runtime) fn write_args_head_order_prefix(
        &self,
        nodes: &[Cell],
        args: &mut Vec<NodeId>,
        limit: usize,
    ) -> Result<(), EvalError> {
        args.clear();
        let len = self.app_len().min(limit);
        args.reserve(len);
        for index in 0..len {
            args.push(self.arg(nodes, index));
        }
        Ok(())
    }
}
