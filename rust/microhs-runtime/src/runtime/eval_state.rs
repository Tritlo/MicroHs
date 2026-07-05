#[derive(Clone, Debug, Default)]
struct PrimCache {
    a: Option<NodeId>,
    b: Option<NodeId>,
    c: Option<NodeId>,
    i: Option<NodeId>,
    k: Option<NodeId>,
    k2: Option<NodeId>,
    k3: Option<NodeId>,
    o: Option<NodeId>,
    p: Option<NodeId>,
    u: Option<NodeId>,
    y: Option<NodeId>,
    z: Option<NodeId>,
    io_bind: Option<NodeId>,
    io_perform_io: Option<NodeId>,
}

#[derive(Clone, Debug, Default)]
struct CompoundCache {
    fst: Option<NodeId>,
    snd: Option<NodeId>,
    just: Option<NodeId>,
    pair_unit: Option<NodeId>,
}

struct Spine {
    head: NodeId,
    storage: SpineStorage,
}

struct EvalSpine {
    inline_args: [MaybeUninit<NodeId>; INLINE_SPINE],
    inline_apps: [MaybeUninit<NodeId>; INLINE_SPINE],
    inline_len: usize,
    heap_args: Vec<NodeId>,
    heap_apps: Vec<NodeId>,
    heap: bool,
}

enum SpineStorage {
    Inline {
        args: [MaybeUninit<NodeId>; INLINE_SPINE],
        apps: [MaybeUninit<NodeId>; INLINE_SPINE],
        len: usize,
    },
    Heap {
        args: Vec<NodeId>,
        apps: Vec<NodeId>,
    },
}

#[derive(Default)]
struct PersistentSpine {
    apps: VecDeque<NodeId>,
}

enum PersistentStep {
    Reduced { node: NodeId, reductions: usize },
    Force { node: NodeId },
    Whnf { node: NodeId },
    Fallback { root: NodeId },
}

enum PersistentHead {
    Ffi(String),
    JsCall { tags: String, body: Vec<u8> },
    JsWrap { tags: String },
    Known(KnownPrim),
    Other(StrictPrimitiveAction),
    Whnf,
}

enum EvalHead {
    Ffi(String),
    JsCall {
        tags: String,
        body: Vec<u8>,
    },
    JsWrap {
        tags: String,
    },
    Known(KnownPrim),
    Other {
        action: StrictPrimitiveAction,
        fallback_name: Option<&'static str>,
    },
    Whnf,
}

#[derive(Clone, Copy, Debug)]
enum StrictPrimitiveAction {
    None,
    IntBin(IntBinOp),
    IntUn(IntUnOp),
    Int64Bin(Int64BinOp),
    Int64Un(Int64UnOp),
    Float64Bin(Float64BinOp),
    Float64Un(Float64UnOp),
    Float32Bin(Float32BinOp),
    Float32Un(Float32UnOp),
    BytesBin(BytesBinOp),
    Conversion(ConversionFrameKind),
}

const INT_BIN_RUNTIME_START: u16 = 0;
const INT_BIN_RUNTIME_OPS: [IntBinOp; 30] = [
    IntBinOp::Add,
    IntBinOp::Sub,
    IntBinOp::Mul,
    IntBinOp::Quot,
    IntBinOp::Rem,
    IntBinOp::SubR,
    IntBinOp::UAdd,
    IntBinOp::USub,
    IntBinOp::UMul,
    IntBinOp::UQuot,
    IntBinOp::URem,
    IntBinOp::USubR,
    IntBinOp::And,
    IntBinOp::Or,
    IntBinOp::Xor,
    IntBinOp::Shl,
    IntBinOp::Shr,
    IntBinOp::Ashr,
    IntBinOp::Eq,
    IntBinOp::Ne,
    IntBinOp::Lt,
    IntBinOp::Le,
    IntBinOp::Gt,
    IntBinOp::Ge,
    IntBinOp::Ult,
    IntBinOp::Ule,
    IntBinOp::Ugt,
    IntBinOp::Uge,
    IntBinOp::ICmp,
    IntBinOp::UCmp,
];

const INT_UN_RUNTIME_START: u16 = 30;
const INT_UN_RUNTIME_OPS: [IntUnOp; 6] = [
    IntUnOp::Neg,
    IntUnOp::UNeg,
    IntUnOp::Inv,
    IntUnOp::PopCount,
    IntUnOp::Clz,
    IntUnOp::Ctz,
];

const INT64_BIN_RUNTIME_START: u16 = 36;
const INT64_BIN_RUNTIME_OPS: [Int64BinOp; 30] = [
    Int64BinOp::Add,
    Int64BinOp::Sub,
    Int64BinOp::Mul,
    Int64BinOp::Quot,
    Int64BinOp::Rem,
    Int64BinOp::SubR,
    Int64BinOp::UAdd,
    Int64BinOp::USub,
    Int64BinOp::UMul,
    Int64BinOp::UQuot,
    Int64BinOp::URem,
    Int64BinOp::USubR,
    Int64BinOp::And,
    Int64BinOp::Or,
    Int64BinOp::Xor,
    Int64BinOp::Shl,
    Int64BinOp::Shr,
    Int64BinOp::Ashr,
    Int64BinOp::Eq,
    Int64BinOp::Ne,
    Int64BinOp::Lt,
    Int64BinOp::Le,
    Int64BinOp::Gt,
    Int64BinOp::Ge,
    Int64BinOp::Ult,
    Int64BinOp::Ule,
    Int64BinOp::Ugt,
    Int64BinOp::Uge,
    Int64BinOp::ICmp,
    Int64BinOp::UCmp,
];

const INT64_UN_RUNTIME_START: u16 = 66;
const INT64_UN_RUNTIME_OPS: [Int64UnOp; 6] = [
    Int64UnOp::Neg,
    Int64UnOp::UNeg,
    Int64UnOp::Inv,
    Int64UnOp::PopCount,
    Int64UnOp::Clz,
    Int64UnOp::Ctz,
];

const FLOAT64_BIN_RUNTIME_START: u16 = 72;
const FLOAT64_BIN_RUNTIME_OPS: [Float64BinOp; 10] = [
    Float64BinOp::Add,
    Float64BinOp::Sub,
    Float64BinOp::Mul,
    Float64BinOp::Div,
    Float64BinOp::Eq,
    Float64BinOp::Ne,
    Float64BinOp::Lt,
    Float64BinOp::Le,
    Float64BinOp::Gt,
    Float64BinOp::Ge,
];

const FLOAT64_UN_RUNTIME_START: u16 = 82;
const FLOAT64_UN_RUNTIME_OPS: [Float64UnOp; 1] = [Float64UnOp::Neg];

const FLOAT32_BIN_RUNTIME_START: u16 = 83;
const FLOAT32_BIN_RUNTIME_OPS: [Float32BinOp; 10] = [
    Float32BinOp::Add,
    Float32BinOp::Sub,
    Float32BinOp::Mul,
    Float32BinOp::Div,
    Float32BinOp::Eq,
    Float32BinOp::Ne,
    Float32BinOp::Lt,
    Float32BinOp::Le,
    Float32BinOp::Gt,
    Float32BinOp::Ge,
];

const FLOAT32_UN_RUNTIME_START: u16 = 93;
const FLOAT32_UN_RUNTIME_OPS: [Float32UnOp; 1] = [Float32UnOp::Neg];

const CONVERSION_RUNTIME_START: u16 = 94;
const CONVERSION_RUNTIME_OPS: [ConversionFrameKind; 18] = [
    ConversionFrameKind::IntToInt64,
    ConversionFrameKind::IntToInt64,
    ConversionFrameKind::Int64ToInt,
    ConversionFrameKind::Int64ToInt,
    ConversionFrameKind::IntToFloat64 { unsigned: false },
    ConversionFrameKind::IntToFloat64 { unsigned: true },
    ConversionFrameKind::Int64ToFloat64,
    ConversionFrameKind::Float64ToInt,
    ConversionFrameKind::IntToFloat32 { unsigned: false },
    ConversionFrameKind::IntToFloat32 { unsigned: true },
    ConversionFrameKind::Int64ToFloat32,
    ConversionFrameKind::Float32ToInt,
    ConversionFrameKind::Float64ToFloat32,
    ConversionFrameKind::Float32ToFloat64,
    ConversionFrameKind::Int64BitsToFloat64,
    ConversionFrameKind::Float64BitsToInt64,
    ConversionFrameKind::IntBitsToFloat32,
    ConversionFrameKind::Float32BitsToInt,
];

const BYTES_BIN_RUNTIME_START: u16 = 145;
const BYTES_BIN_RUNTIME_OPS: [BytesBinOp; 9] = [
    BytesBinOp::Append,
    BytesBinOp::AppendDot,
    BytesBinOp::Eq,
    BytesBinOp::Ne,
    BytesBinOp::Lt,
    BytesBinOp::Le,
    BytesBinOp::Gt,
    BytesBinOp::Ge,
    BytesBinOp::Cmp,
];

#[inline(always)]
fn runtime_prim_op<T: Copy, const N: usize>(index: u16, start: u16, ops: &[T; N]) -> Option<T> {
    let offset = index.checked_sub(start)? as usize;
    ops.get(offset).copied()
}

impl RuntimePrim {
    #[inline(always)]
    fn strict_action(self, args_len: usize) -> StrictPrimitiveAction {
        let index = self.0;

        if args_len >= 2 {
            if let Some(op) = runtime_prim_op(index, INT_BIN_RUNTIME_START, &INT_BIN_RUNTIME_OPS) {
                return StrictPrimitiveAction::IntBin(op);
            }
        }
        if args_len >= 1 {
            if let Some(op) = runtime_prim_op(index, INT_UN_RUNTIME_START, &INT_UN_RUNTIME_OPS) {
                return StrictPrimitiveAction::IntUn(op);
            }
        }
        if args_len >= 2 {
            if let Some(op) =
                runtime_prim_op(index, INT64_BIN_RUNTIME_START, &INT64_BIN_RUNTIME_OPS)
            {
                return StrictPrimitiveAction::Int64Bin(op);
            }
        }
        if args_len >= 1 {
            if let Some(op) = runtime_prim_op(index, INT64_UN_RUNTIME_START, &INT64_UN_RUNTIME_OPS)
            {
                return StrictPrimitiveAction::Int64Un(op);
            }
        }
        if args_len >= 2 {
            if let Some(op) =
                runtime_prim_op(index, FLOAT64_BIN_RUNTIME_START, &FLOAT64_BIN_RUNTIME_OPS)
            {
                return StrictPrimitiveAction::Float64Bin(op);
            }
        }
        if args_len >= 1 {
            if let Some(op) =
                runtime_prim_op(index, FLOAT64_UN_RUNTIME_START, &FLOAT64_UN_RUNTIME_OPS)
            {
                return StrictPrimitiveAction::Float64Un(op);
            }
        }
        if args_len >= 2 {
            if let Some(op) =
                runtime_prim_op(index, FLOAT32_BIN_RUNTIME_START, &FLOAT32_BIN_RUNTIME_OPS)
            {
                return StrictPrimitiveAction::Float32Bin(op);
            }
        }
        if args_len >= 1 {
            if let Some(op) =
                runtime_prim_op(index, FLOAT32_UN_RUNTIME_START, &FLOAT32_UN_RUNTIME_OPS)
            {
                return StrictPrimitiveAction::Float32Un(op);
            }
        }
        if args_len >= 2 {
            if let Some(op) =
                runtime_prim_op(index, BYTES_BIN_RUNTIME_START, &BYTES_BIN_RUNTIME_OPS)
            {
                return StrictPrimitiveAction::BytesBin(op);
            }
        }
        if args_len >= 1 {
            if let Some(kind) =
                runtime_prim_op(index, CONVERSION_RUNTIME_START, &CONVERSION_RUNTIME_OPS)
            {
                return StrictPrimitiveAction::Conversion(kind);
            }
        }

        StrictPrimitiveAction::None
    }
}

impl Default for EvalSpine {
    fn default() -> Self {
        Self {
            inline_args: [const { MaybeUninit::uninit() }; INLINE_SPINE],
            inline_apps: [const { MaybeUninit::uninit() }; INLINE_SPINE],
            inline_len: 0,
            heap_args: Vec::new(),
            heap_apps: Vec::new(),
            heap: false,
        }
    }
}

impl EvalSpine {
    fn clear(&mut self) {
        self.inline_len = 0;
        self.heap = false;
        self.heap_args.clear();
        self.heap_apps.clear();
    }

    fn len(&self) -> usize {
        if self.heap {
            self.heap_args.len()
        } else {
            self.inline_len
        }
    }

    fn is_heap(&self) -> bool {
        self.heap
    }

    fn push_desc(&mut self, arg: NodeId, app: NodeId) {
        if self.heap {
            self.heap_args.push(arg);
            self.heap_apps.push(app);
        } else if self.inline_len < INLINE_SPINE {
            self.inline_args[self.inline_len].write(arg);
            self.inline_apps[self.inline_len].write(app);
            self.inline_len += 1;
        } else {
            self.heap = true;
            self.heap_args.reserve(INLINE_SPINE * 2);
            self.heap_apps.reserve(INLINE_SPINE * 2);
            for idx in 0..self.inline_len {
                // SAFETY: indices below inline_len were written before heap promotion.
                self.heap_args
                    .push(unsafe { self.inline_args[idx].assume_init() });
                // SAFETY: indices below inline_len were written before heap promotion.
                self.heap_apps
                    .push(unsafe { self.inline_apps[idx].assume_init() });
            }
            self.heap_args.push(arg);
            self.heap_apps.push(app);
        }
    }

    fn desc_arg(&self, desc_idx: usize) -> NodeId {
        if self.heap {
            self.heap_args[desc_idx]
        } else {
            debug_assert!(desc_idx < self.inline_len);
            // SAFETY: desc_idx is below inline_len, so the slot was initialized.
            unsafe { self.inline_args[desc_idx].assume_init() }
        }
    }

    fn desc_app(&self, desc_idx: usize) -> NodeId {
        if self.heap {
            self.heap_apps[desc_idx]
        } else {
            debug_assert!(desc_idx < self.inline_len);
            // SAFETY: desc_idx is below inline_len, so the slot was initialized.
            unsafe { self.inline_apps[desc_idx].assume_init() }
        }
    }

    fn arg(&self, head_idx: usize) -> NodeId {
        let len = self.len();
        debug_assert!(head_idx < len);
        self.desc_arg(len - head_idx - 1)
    }

    fn app(&self, head_idx: usize) -> NodeId {
        let len = self.len();
        debug_assert!(head_idx < len);
        self.desc_app(len - head_idx - 1)
    }

    fn write_args_head_order(&self, args: &mut Vec<NodeId>) {
        args.clear();
        let len = self.len();
        args.reserve(len);
        for desc_idx in (0..len).rev() {
            args.push(self.desc_arg(desc_idx));
        }
    }

    fn write_args_head_order_prefix(&self, args: &mut Vec<NodeId>, limit: usize) {
        args.clear();
        let len = self.len().min(limit);
        args.reserve(len);
        for head_idx in 0..len {
            args.push(self.arg(head_idx));
        }
    }

    fn write_apps_head_order(&self, apps: &mut Vec<NodeId>) {
        apps.clear();
        let len = self.len();
        apps.reserve(len);
        for desc_idx in (0..len).rev() {
            apps.push(self.desc_app(desc_idx));
        }
    }
}

impl PersistentSpine {
    fn clear(&mut self) {
        self.apps.clear();
    }

    fn len(&self) -> usize {
        self.apps.len()
    }

    fn push_front(&mut self, app: NodeId) {
        self.apps.push_front(app);
    }

    fn consume(&mut self, used: usize) {
        debug_assert!(used <= self.len());
        for _ in 0..used {
            self.apps.pop_front();
        }
    }

    fn arg(&self, nodes: &[Cell], index: usize) -> Result<NodeId, EvalError> {
        let app = self.app(index);
        nodes
            .get(app.index())
            .and_then(|cell| cell.app_fields())
            .map(|(_, arg)| arg)
            .ok_or(EvalError::DanglingIndirection(app))
    }

    fn app(&self, index: usize) -> NodeId {
        self.apps[index]
    }

    fn write_args_head_order(
        &self,
        nodes: &[Cell],
        args: &mut Vec<NodeId>,
    ) -> Result<(), EvalError> {
        args.clear();
        args.reserve(self.len());
        for index in 0..self.len() {
            args.push(self.arg(nodes, index)?);
        }
        Ok(())
    }

    fn outer_root(&self, head: NodeId) -> NodeId {
        self.apps.back().copied().unwrap_or(head)
    }

    fn remaining_apps_contain(&self, start: usize, node: NodeId) -> bool {
        self.apps.iter().skip(start).any(|app| *app == node)
    }
}

impl Spine {
    fn args(&self) -> &[NodeId] {
        match &self.storage {
            SpineStorage::Inline { args, len, .. } => initialized_node_slice(args, *len),
            SpineStorage::Heap { args, .. } => args,
        }
    }

    fn apps(&self) -> &[NodeId] {
        match &self.storage {
            SpineStorage::Inline { apps, len, .. } => initialized_node_slice(apps, *len),
            SpineStorage::Heap { apps, .. } => apps,
        }
    }
}

fn initialized_node_slice(storage: &[MaybeUninit<NodeId>], len: usize) -> &[NodeId] {
    debug_assert!(len <= storage.len());
    // SAFETY: Spine::spine writes exactly the first `len` elements before storing
    // an Inline spine, and NodeId is Copy with no drop glue.
    unsafe { std::slice::from_raw_parts(storage.as_ptr().cast::<NodeId>(), len) }
}

fn small_int_index(value: i64) -> Option<usize> {
    if (SMALL_INT_MIN..=SMALL_INT_MAX).contains(&value) {
        Some((value - SMALL_INT_MIN) as usize)
    } else {
        None
    }
}

struct EvalLoopStep {
    node: NodeId,
    reductions: usize,
}

type ProfileHead = Option<NodeId>;

struct WhnfFrame {
    redex: StrictRedex,
    profile_head: ProfileHead,
    kind: WhnfFrameKind,
}

enum WhnfFrameKind {
    Seq { result: NodeId },
    IoStrict { action: NodeId, value: NodeId },
    IsInt,
}

struct IntFrame {
    redex: StrictRedex,
    profile_head: ProfileHead,
    kind: IntFrameKind,
}

enum StrictRedex {
    Root(NodeId),
    Spine {
        root: NodeId,
        used: usize,
        apps: Vec<NodeId>,
    },
}

enum IntFrameKind {
    BinSecond { op: IntBinOp, x: NodeId },
    BinFirst { op: IntBinOp, y: i64 },
    Un { op: IntUnOp },
}

struct Int64Frame {
    redex: StrictRedex,
    profile_head: ProfileHead,
    kind: Int64FrameKind,
}

enum Int64FrameKind {
    BinSecond { op: Int64BinOp, x: NodeId },
    BinFirst { op: Int64BinOp, y: i64 },
    ShiftFirst { op: Int64BinOp, y: i64 },
    Un { op: Int64UnOp },
}

struct Int64ShiftFrame {
    redex: StrictRedex,
    profile_head: ProfileHead,
    op: Int64BinOp,
    x: NodeId,
}

struct Float64Frame {
    redex: StrictRedex,
    profile_head: ProfileHead,
    kind: Float64FrameKind,
}

enum Float64FrameKind {
    BinSecond { op: Float64BinOp, x: NodeId },
    BinFirst { op: Float64BinOp, y: f64 },
    Un { op: Float64UnOp },
}

struct Float32Frame {
    redex: StrictRedex,
    profile_head: ProfileHead,
    kind: Float32FrameKind,
}

enum Float32FrameKind {
    BinSecond { op: Float32BinOp, x: NodeId },
    BinFirst { op: Float32BinOp, y: f32 },
    Un { op: Float32UnOp },
}

struct BytesFrame {
    redex: StrictRedex,
    profile_head: ProfileHead,
    kind: BytesFrameKind,
}

enum BytesFrameKind {
    BinSecond { op: BytesBinOp, x: NodeId },
    BinFirst { op: BytesBinOp, y: NodeId },
}

struct ConversionFrame {
    redex: StrictRedex,
    profile_head: ProfileHead,
    kind: ConversionFrameKind,
}

#[derive(Clone, Copy, Debug)]
enum ConversionFrameKind {
    IntToInt64,
    Int64ToInt,
    IntToFloat64 { unsigned: bool },
    Int64ToFloat64,
    Float64ToInt,
    IntToFloat32 { unsigned: bool },
    Int64ToFloat32,
    Float32ToInt,
    Float64ToFloat32,
    Float32ToFloat64,
    Int64BitsToFloat64,
    Float64BitsToInt64,
    IntBitsToFloat32,
    Float32BitsToInt,
}

enum ConversionValue {
    Int(i64),
    Int64(i64),
    Float64(f64),
    Float32(f32),
}

enum EvalFrame {
    Whnf(WhnfFrame),
    Int(IntFrame),
    Int64(Int64Frame),
    Int64Shift(Int64ShiftFrame),
    Float64(Float64Frame),
    Float32(Float32Frame),
    Bytes(BytesFrame),
    Conversion(ConversionFrame),
}

struct FrameStack<T> {
    top: Option<T>,
    rest: Vec<T>,
}

impl<T> Default for FrameStack<T> {
    fn default() -> Self {
        Self {
            top: None,
            rest: Vec::new(),
        }
    }
}

impl<T> FrameStack<T> {
    fn push(&mut self, frame: T) {
        if let Some(top) = self.top.replace(frame) {
            self.rest.push(top);
        }
    }

    fn pop(&mut self) -> Option<T> {
        let frame = self.top.take()?;
        self.top = self.rest.pop();
        Some(frame)
    }

    fn peek(&self) -> Option<&T> {
        self.top.as_ref()
    }
}

type EvalFrameStack = FrameStack<EvalFrame>;

struct StackWhnfFrame {
    prev_app_base: usize,
    redex: NodeId,
    used: usize,
    profile_head: ProfileHead,
    kind: WhnfFrameKind,
}

struct StackIntFrame {
    prev_app_base: usize,
    redex: NodeId,
    profile_head: ProfileHead,
    kind: IntFrameKind,
}

struct StackInt64Frame {
    prev_app_base: usize,
    app_end: usize,
    used: usize,
    profile_head: ProfileHead,
    kind: Int64FrameKind,
}

struct StackInt64ShiftFrame {
    prev_app_base: usize,
    app_end: usize,
    used: usize,
    profile_head: ProfileHead,
    op: Int64BinOp,
    x: NodeId,
}

struct StackFloat64Frame {
    prev_app_base: usize,
    app_end: usize,
    used: usize,
    profile_head: ProfileHead,
    kind: Float64FrameKind,
}

struct StackFloat32Frame {
    prev_app_base: usize,
    app_end: usize,
    used: usize,
    profile_head: ProfileHead,
    kind: Float32FrameKind,
}

struct StackBytesFrame {
    prev_app_base: usize,
    app_end: usize,
    used: usize,
    profile_head: ProfileHead,
    kind: BytesFrameKind,
}

struct StackConversionFrame {
    prev_app_base: usize,
    app_end: usize,
    used: usize,
    profile_head: ProfileHead,
    kind: ConversionFrameKind,
}

enum StackFrame {
    Whnf(StackWhnfFrame),
    Int(StackIntFrame),
    Int64(StackInt64Frame),
    Int64Shift(StackInt64ShiftFrame),
    Float64(StackFloat64Frame),
    Float32(StackFloat32Frame),
    Bytes(StackBytesFrame),
    Conversion(StackConversionFrame),
}

#[derive(Default)]
struct EvalStack {
    apps: Vec<NodeId>,
    frames: Vec<StackFrame>,
    app_base: usize,
}

enum StackStep {
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
    fn prev_app_base(&self) -> usize {
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
    fn push_app(&mut self, app: NodeId) {
        self.apps.push(app);
    }

    fn app_unchecked(&self, index: usize) -> NodeId {
        debug_assert!(index < self.apps.len());
        unsafe { *self.apps.get_unchecked(index) }
    }

    fn app_base(&self) -> usize {
        self.app_base
    }

    fn app_len(&self) -> usize {
        self.apps.len() - self.app_base()
    }

    fn arg(&self, nodes: &[Cell], index: usize) -> NodeId {
        let app_index = self.apps.len() - index - 1;
        self.arg_at_app(nodes, app_index)
    }

    fn arg_at_app(&self, nodes: &[Cell], app_index: usize) -> NodeId {
        debug_assert!(app_index < self.apps.len());
        // The stack app segment is built only by descending through App
        // cells. Match eval.c's ARG(TOP(i)) discipline in the hot path.
        unsafe {
            let app = *self.apps.get_unchecked(app_index);
            nodes.get_unchecked(app.index()).id_word1()
        }
    }

    fn arg_from_app(nodes: &[Cell], app: NodeId) -> NodeId {
        debug_assert!(app.index() < nodes.len());
        debug_assert_eq!(nodes[app.index()].tag(), CellTag::App);
        unsafe { nodes.get_unchecked(app.index()).id_word1() }
    }

    fn take_args1(&mut self, nodes: &[Cell]) -> (NodeId, NodeId) {
        let end = self.apps.len();
        debug_assert!(self.app_len() >= 1);
        let redex_index = end - 1;
        let redex = unsafe { *self.apps.get_unchecked(redex_index) };
        let x = Self::arg_from_app(nodes, redex);
        self.apps.truncate(redex_index);
        (redex, x)
    }

    fn take_args2(&mut self, nodes: &[Cell]) -> (NodeId, NodeId, NodeId) {
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

    fn take_args3(&mut self, nodes: &[Cell]) -> (NodeId, NodeId, NodeId, NodeId) {
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

    fn take_args4(&mut self, nodes: &[Cell]) -> (NodeId, NodeId, NodeId, NodeId, NodeId) {
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

    fn take_args5(&mut self, nodes: &[Cell]) -> (NodeId, NodeId, NodeId, NodeId, NodeId, NodeId) {
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

    fn outer_root(&self, head: NodeId) -> NodeId {
        let base = self.app_base();
        if base == self.apps.len() {
            return head;
        }
        self.apps[base]
    }

    fn has_frame_below_apps(&self) -> bool {
        !self.frames.is_empty()
    }

    fn top_is_frame(&self) -> bool {
        !self.frames.is_empty() && self.apps.len() == self.app_base
    }

    fn peek_frame(&self) -> Option<&StackFrame> {
        if self.top_is_frame() {
            self.frames.last()
        } else {
            None
        }
    }

    fn push_whnf_frame(
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

    fn push_int_frame(&mut self, redex: NodeId, profile_head: ProfileHead, kind: IntFrameKind) {
        let prev_app_base = self.app_base;
        self.frames.push(StackFrame::Int(StackIntFrame {
            prev_app_base,
            redex,
            profile_head,
            kind,
        }));
        self.app_base = self.apps.len();
    }

    fn push_int64_frame(
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

    fn push_int64_shift_frame(
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

    fn push_float64_frame(
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

    fn push_float32_frame(
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

    fn push_bytes_frame(
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

    fn push_conversion_frame(
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

    fn pop_frame(&mut self) -> Option<StackFrame> {
        if self.top_is_frame() {
            let frame = self.frames.pop().expect("frame marker must have payload");
            self.app_base = frame.prev_app_base();
            Some(frame)
        } else {
            None
        }
    }

    fn write_args_head_order(
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

    fn write_args_head_order_prefix(
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

impl ConversionFrameKind {
    fn ready_cell_value(self, cell: Cell) -> Option<ConversionValue> {
        match self {
            Self::IntToInt64
            | Self::IntToFloat64 { .. }
            | Self::IntToFloat32 { .. }
            | Self::IntBitsToFloat32 => cell.int_value().map(ConversionValue::Int),
            Self::Int64ToInt
            | Self::Int64ToFloat64
            | Self::Int64ToFloat32
            | Self::Int64BitsToFloat64 => cell.int64_value().map(ConversionValue::Int64),
            Self::Float64ToInt | Self::Float64ToFloat32 | Self::Float64BitsToInt64 => {
                cell.float64_value().map(ConversionValue::Float64)
            }
            Self::Float32ToInt | Self::Float32ToFloat64 | Self::Float32BitsToInt => {
                cell.float32_value().map(ConversionValue::Float32)
            }
        }
    }

    fn expected_error(self, current: NodeId) -> EvalError {
        match self {
            Self::IntToInt64
            | Self::IntToFloat64 { .. }
            | Self::IntToFloat32 { .. }
            | Self::IntBitsToFloat32 => EvalError::ExpectedInt(current),
            Self::Int64ToInt
            | Self::Int64ToFloat64
            | Self::Int64ToFloat32
            | Self::Int64BitsToFloat64 => EvalError::ExpectedInt64(current),
            Self::Float64ToInt | Self::Float64ToFloat32 | Self::Float64BitsToInt64 => {
                EvalError::ExpectedFloat64(current)
            }
            Self::Float32ToInt | Self::Float32ToFloat64 | Self::Float32BitsToInt => {
                EvalError::ExpectedFloat32(current)
            }
        }
    }
}
