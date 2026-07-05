//! Core heap, node, handle, and error types for the runtime.
use super::*;

#[derive(Clone, Debug)]
pub enum Node {
    App(NodeId, NodeId),
    Indir(Option<NodeId>),
    Free(Option<NodeId>),
    Prim(Prim),
    Int(i64),
    Int64(i64),
    Float64(f64),
    Float32(f32),
    ThreadId(i64),
    Ptr(i64),
    RawFunPtr(i64),
    ForeignPtr(Box<ForeignPtrNode>),
    Weak(Box<WeakNode>),
    MVar(Option<NodeId>),
    BigInt(Box<Vec<u8>>),
    Bytes(Box<Vec<u8>>),
    BytesView(Box<BytesViewNode>),
    MutableBytes(Box<MutableBytesNode>),
    Array(Box<Vec<NodeId>>),
    Ffi(Box<String>),
    JsCall(Box<JsCallNode>),
    JsWrap { tags: Box<String> },
    FunPtr(Box<String>),
    Tick(Box<Vec<u8>>),
}

#[derive(Clone, Copy, Debug)]
pub(in crate::runtime) struct Cell {
    pub(in crate::runtime) word0: u64,
    pub(in crate::runtime) word1: u64,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(in crate::runtime) enum CellTag {
    App,
    Indir,
    Free,
    KnownPrim,
    RuntimePrim,
    Int,
    Int64,
    Float64,
    Float32,
    ThreadId,
    Ptr,
    RawFunPtr,
    Cold,
}

pub(in crate::runtime) const CELL_TAG_BITS: u64 = 0xff;
pub(in crate::runtime) const CELL_PAYLOAD_SHIFT: u64 = 8;
pub(in crate::runtime) const CELL_NONE_ID: u64 = u64::MAX;

impl CellTag {
    pub(in crate::runtime) fn from_bits(bits: u64) -> Self {
        match bits & CELL_TAG_BITS {
            0 => Self::App,
            1 => Self::Indir,
            2 => Self::Free,
            3 => Self::KnownPrim,
            4 => Self::RuntimePrim,
            5 => Self::Int,
            6 => Self::Int64,
            7 => Self::Float64,
            8 => Self::Float32,
            9 => Self::ThreadId,
            10 => Self::Ptr,
            11 => Self::RawFunPtr,
            12 => Self::Cold,
            _ => unreachable!("invalid cell tag"),
        }
    }

    pub(in crate::runtime) fn bits(self) -> u64 {
        match self {
            Self::App => 0,
            Self::Indir => 1,
            Self::Free => 2,
            Self::KnownPrim => 3,
            Self::RuntimePrim => 4,
            Self::Int => 5,
            Self::Int64 => 6,
            Self::Float64 => 7,
            Self::Float32 => 8,
            Self::ThreadId => 9,
            Self::Ptr => 10,
            Self::RawFunPtr => 11,
            Self::Cold => 12,
        }
    }
}

impl Cell {
    #[inline]
    pub(in crate::runtime) fn tag_bits(self) -> u64 {
        self.word0 & CELL_TAG_BITS
    }

    #[inline]
    pub(in crate::runtime) fn has_tag(self, tag: CellTag) -> bool {
        self.tag_bits() == tag.bits()
    }

    pub(in crate::runtime) fn from_node(node: Node, cold_nodes: &mut Vec<Option<Node>>) -> Self {
        match node {
            Node::App(fun, arg) => Self::app(fun, arg),
            Node::Indir(target) => Self::indir(target),
            Node::Free(next) => Self::free(next),
            Node::Prim(Prim::Known(known)) => Self::known_prim(known),
            Node::Prim(Prim::Runtime(runtime)) => Self::runtime_prim(runtime),
            Node::Int(value) => Self::int(value),
            Node::Int64(value) => Self::int64(value),
            Node::Float64(value) => Self::float64(value),
            Node::Float32(value) => Self::float32(value),
            Node::ThreadId(value) => Self::thread_id(value),
            Node::Ptr(value) => Self::ptr(value),
            Node::RawFunPtr(value) => Self::raw_fun_ptr(value),
            cold => {
                let index = cold_nodes.len();
                cold_nodes.push(Some(cold));
                Self::cold(index)
            }
        }
    }

    pub(in crate::runtime) fn to_node(self, cold_nodes: &[Option<Node>]) -> Node {
        match self.tag() {
            CellTag::App => Node::App(self.id_payload(), self.id_word1()),
            CellTag::Indir => Node::Indir(self.option_id_word1()),
            CellTag::Free => Node::Free(self.option_id_word1()),
            CellTag::KnownPrim => Node::Prim(Prim::Known(decode_known_prim(self.word1 as u16))),
            CellTag::RuntimePrim => Node::Prim(Prim::Runtime(RuntimePrim(self.word1 as u16))),
            CellTag::Int => Node::Int(self.word1 as i64),
            CellTag::Int64 => Node::Int64(self.word1 as i64),
            CellTag::Float64 => Node::Float64(f64::from_bits(self.word1)),
            CellTag::Float32 => Node::Float32(f32::from_bits(self.word1 as u32)),
            CellTag::ThreadId => Node::ThreadId(self.word1 as i64),
            CellTag::Ptr => Node::Ptr(self.word1 as i64),
            CellTag::RawFunPtr => Node::RawFunPtr(self.word1 as i64),
            CellTag::Cold => cold_nodes[self.word1 as usize]
                .as_ref()
                .expect("live cold cell pointed at freed cold node")
                .clone(),
        }
    }

    pub(in crate::runtime) fn tag(self) -> CellTag {
        CellTag::from_bits(self.tag_bits())
    }

    pub(in crate::runtime) fn app(fun: NodeId, arg: NodeId) -> Self {
        Self {
            word0: (u64::from(fun.0) << CELL_PAYLOAD_SHIFT) | CellTag::App.bits(),
            word1: u64::from(arg.0),
        }
    }

    pub(in crate::runtime) fn indir(target: Option<NodeId>) -> Self {
        Self {
            word0: CellTag::Indir.bits(),
            word1: pack_option_id(target),
        }
    }

    pub(in crate::runtime) fn free(next: Option<NodeId>) -> Self {
        Self {
            word0: CellTag::Free.bits(),
            word1: pack_option_id(next),
        }
    }

    pub(in crate::runtime) fn known_prim(known: KnownPrim) -> Self {
        Self {
            word0: CellTag::KnownPrim.bits(),
            word1: u64::from(encode_known_prim(known)),
        }
    }

    pub(in crate::runtime) fn runtime_prim(runtime: RuntimePrim) -> Self {
        Self {
            word0: CellTag::RuntimePrim.bits(),
            word1: u64::from(runtime.0),
        }
    }

    pub(in crate::runtime) fn int(value: i64) -> Self {
        Self {
            word0: CellTag::Int.bits(),
            word1: value as u64,
        }
    }

    pub(in crate::runtime) fn int64(value: i64) -> Self {
        Self {
            word0: CellTag::Int64.bits(),
            word1: value as u64,
        }
    }

    pub(in crate::runtime) fn float64(value: f64) -> Self {
        Self {
            word0: CellTag::Float64.bits(),
            word1: value.to_bits(),
        }
    }

    pub(in crate::runtime) fn float32(value: f32) -> Self {
        Self {
            word0: CellTag::Float32.bits(),
            word1: u64::from(value.to_bits()),
        }
    }

    pub(in crate::runtime) fn thread_id(value: i64) -> Self {
        Self {
            word0: CellTag::ThreadId.bits(),
            word1: value as u64,
        }
    }

    pub(in crate::runtime) fn ptr(value: i64) -> Self {
        Self {
            word0: CellTag::Ptr.bits(),
            word1: value as u64,
        }
    }

    pub(in crate::runtime) fn raw_fun_ptr(value: i64) -> Self {
        Self {
            word0: CellTag::RawFunPtr.bits(),
            word1: value as u64,
        }
    }

    pub(in crate::runtime) fn cold(index: usize) -> Self {
        Self {
            word0: CellTag::Cold.bits(),
            word1: u64::try_from(index).expect("cold node table exceeded u64"),
        }
    }

    pub(in crate::runtime) fn id_payload(self) -> NodeId {
        NodeId((self.word0 >> CELL_PAYLOAD_SHIFT) as u32)
    }

    pub(in crate::runtime) fn id_word1(self) -> NodeId {
        NodeId(self.word1 as u32)
    }

    pub(in crate::runtime) fn option_id_word1(self) -> Option<NodeId> {
        unpack_option_id(self.word1)
    }

    pub(in crate::runtime) fn app_fields(self) -> Option<(NodeId, NodeId)> {
        self.has_tag(CellTag::App)
            .then(|| (self.id_payload(), self.id_word1()))
    }

    #[inline(always)]
    pub(in crate::runtime) fn prim(self) -> Option<Prim> {
        match self.tag_bits() {
            3 => Some(Prim::Known(decode_known_prim(self.word1 as u16))),
            4 => Some(Prim::Runtime(RuntimePrim(self.word1 as u16))),
            _ => None,
        }
    }

    pub(in crate::runtime) fn int_value(self) -> Option<i64> {
        self.has_tag(CellTag::Int).then_some(self.word1 as i64)
    }

    pub(in crate::runtime) fn int64_value(self) -> Option<i64> {
        self.has_tag(CellTag::Int64).then_some(self.word1 as i64)
    }

    pub(in crate::runtime) fn thread_id_value(self) -> Option<i64> {
        self.has_tag(CellTag::ThreadId).then_some(self.word1 as i64)
    }

    pub(in crate::runtime) fn ptr_value(self) -> Option<i64> {
        self.has_tag(CellTag::Ptr).then_some(self.word1 as i64)
    }

    pub(in crate::runtime) fn raw_fun_ptr_value(self) -> Option<i64> {
        self.has_tag(CellTag::RawFunPtr)
            .then_some(self.word1 as i64)
    }

    pub(in crate::runtime) fn float64_value(self) -> Option<f64> {
        self.has_tag(CellTag::Float64)
            .then_some(f64::from_bits(self.word1))
    }

    pub(in crate::runtime) fn float32_value(self) -> Option<f32> {
        self.has_tag(CellTag::Float32)
            .then_some(f32::from_bits(self.word1 as u32))
    }

    pub(in crate::runtime) fn cold_index(self) -> Option<usize> {
        self.has_tag(CellTag::Cold).then_some(self.word1 as usize)
    }
}

pub(in crate::runtime) fn pack_option_id(id: Option<NodeId>) -> u64 {
    id.map_or(CELL_NONE_ID, |id| u64::from(id.0))
}

pub(in crate::runtime) fn unpack_option_id(word: u64) -> Option<NodeId> {
    (word != CELL_NONE_ID).then_some(NodeId(word as u32))
}

#[derive(Default)]
pub(in crate::runtime) struct SerializationLabels {
    pub(in crate::runtime) shared: HashSet<NodeId>,
    pub(in crate::runtime) printed: HashSet<NodeId>,
}

#[derive(Clone, Debug)]
pub struct ForeignPtrNode {
    pub(crate) bytes: Option<Vec<u8>>,
    pub(crate) offset: usize,
    pub(crate) ptr: i64,
    pub(crate) finalizer: Option<usize>,
}

#[derive(Clone, Debug)]
pub(in crate::runtime) enum ForeignFinalizer {
    Free,
    CloseB,
    RawZero,
}

#[derive(Clone, Debug)]
pub(in crate::runtime) struct ForeignFinalizerState {
    pub(in crate::runtime) arg: i64,
    pub(in crate::runtime) finalizer: Option<ForeignFinalizer>,
}

#[derive(Clone, Debug)]
pub struct BytesViewNode {
    pub(crate) base: NodeId,
    pub(crate) offset: usize,
    pub(crate) len: usize,
}

#[derive(Clone, Debug)]
pub struct JsCallNode {
    pub(crate) tags: String,
    pub(crate) body: Vec<u8>,
}

#[derive(Clone, Debug)]
pub struct WeakNode {
    pub(crate) key: Option<NodeId>,
    pub(crate) value: Option<NodeId>,
    pub(crate) finalizer: Option<NodeId>,
}

#[derive(Clone, Debug)]
pub struct MutableBytesNode {
    pub(crate) bytes: Vec<u8>,
    pub(crate) size: usize,
    pub(crate) capacity: usize,
}

impl MutableBytesNode {
    pub(in crate::runtime) fn visible(&self) -> &[u8] {
        &self.bytes[..self.size]
    }
}

impl Node {
    pub fn prim(name: &str) -> Self {
        let prim = Prim::from_name(name)
            .unwrap_or_else(|| panic!("unknown runtime primitive requested internally: {name}"));
        Self::Prim(prim)
    }

    pub fn bigint(bytes: Vec<u8>) -> Self {
        Self::BigInt(Box::new(bytes))
    }

    pub fn bytes(bytes: Vec<u8>) -> Self {
        Self::Bytes(Box::new(bytes))
    }

    pub(in crate::runtime) fn bytes_view(base: NodeId, offset: usize, len: usize) -> Self {
        Self::BytesView(Box::new(BytesViewNode { base, offset, len }))
    }

    pub fn array(items: Vec<NodeId>) -> Self {
        Self::Array(Box::new(items))
    }

    pub fn ffi(name: String) -> Self {
        Self::Ffi(Box::new(name))
    }

    pub fn js_wrap(tags: String) -> Self {
        Self::JsWrap {
            tags: Box::new(tags),
        }
    }

    pub fn fun_ptr(name: impl Into<String>) -> Self {
        Self::FunPtr(Box::new(name.into()))
    }

    pub fn tick(bytes: Vec<u8>) -> Self {
        Self::Tick(Box::new(bytes))
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(in crate::runtime) enum StdHandle {
    Stdin,
    Stdout,
    Stderr,
}

pub(in crate::runtime) const ALLOCATION_PTR_BASE: i64 = -(1_i64 << 62);
pub(in crate::runtime) const ALLOCATION_PTR_STRIDE: i64 = 1_i64 << 32;
pub(in crate::runtime) const NODE_PTR_STRIDE: i64 = 1_i64 << 32;
pub(in crate::runtime) const BFILE_PTR_BASE: i64 = i64::MIN + (1_i64 << 32);
pub(in crate::runtime) const FORCE_REDUCTION_LIMIT: usize = usize::MAX;
pub(in crate::runtime) const RTS_EXN_DIVIDE_BY_ZERO: i64 = 4;
pub(in crate::runtime) const RTS_EXN_OVERFLOW: i64 = 7;
pub(in crate::runtime) const MASK_INTERRUPTIBLE: i64 = 1;
pub(in crate::runtime) const BFILE_PTR_STRIDE: i64 = 1_i64 << 32;
pub(in crate::runtime) const DIR_PTR_BASE: i64 = i64::MIN + (1_i64 << 61);
pub(in crate::runtime) const DIR_PTR_STRIDE: i64 = 1_i64 << 32;
pub(in crate::runtime) const INLINE_SPINE: usize = 16;
pub(in crate::runtime) const FALLBACK_PRIM_ARG_PREFIX: usize = 4;
pub(in crate::runtime) const SMALL_INT_MIN: i64 = -10;
pub(in crate::runtime) const SMALL_INT_MAX: i64 = 255;
pub(in crate::runtime) const SMALL_INT_COUNT: usize = (SMALL_INT_MAX - SMALL_INT_MIN + 1) as usize;
pub(in crate::runtime) const IGNORED_IO_SHORTCUT_RECURSION_LIMIT: usize = 256;
pub(in crate::runtime) const UTF8_ASCII_REFILL: usize = 1024;
pub(in crate::runtime) const READ_ONLY_MEMORY_VIEW_MIN_LEN: usize = 8;
#[cfg(not(target_os = "wasi"))]
pub(in crate::runtime) const GC_NODE_INTERVAL: usize = 32 * 1024 * 1024;
#[cfg(target_os = "wasi")]
pub(in crate::runtime) const WASI_GC_NODE_INTERVAL: usize = 500_000;

#[derive(Clone, Debug)]
pub(in crate::runtime) struct BFile {
    pub(in crate::runtime) kind: BFileKind,
    pub(in crate::runtime) readable: bool,
    pub(in crate::runtime) writable: bool,
}

#[derive(Clone, Debug)]
pub(in crate::runtime) enum BFileKind {
    Memory {
        bytes: Vec<u8>,
        pos: usize,
    },
    ReadOnlyMemoryView {
        base: NodeId,
        offset: usize,
        len: usize,
        pos: usize,
    },
    #[cfg(any(not(target_arch = "wasm32"), target_os = "wasi"))]
    NativeFile {
        file: NativeFileHandle,
        ungot: Vec<u8>,
    },
    #[cfg(all(target_arch = "wasm32", not(target_os = "wasi")))]
    BrowserFile {
        handle: i64,
        ungot: Vec<u8>,
    },
    Utf8 {
        inner: i64,
        unget: Option<i64>,
        pending: Vec<u8>,
        pending_pos: usize,
    },
    Crlf {
        inner: i64,
    },
    Rle {
        inner: i64,
        read: bool,
        count: usize,
        byte: i64,
        unget: Option<i64>,
    },
    Base64 {
        inner: i64,
        read: bool,
        encbuf: [u8; 3],
        encpos: usize,
        linelen: usize,
        outcol: usize,
        unget: Option<i64>,
        outbuf: [u8; 3],
        outpos: usize,
        outlen: usize,
    },
    Lz77 {
        inner: i64,
        read: bool,
        buffer: Vec<u8>,
        pos: usize,
        numflush: usize,
    },
    Bwt {
        inner: i64,
        read: bool,
        buffer: Vec<u8>,
        pos: usize,
        numflush: usize,
    },
    Lzma {
        inner: i64,
        read: bool,
        buffer: Vec<u8>,
        pos: usize,
        numflush: usize,
    },
    Buf {
        inner: i64,
        unget: Option<i64>,
        buffer: Vec<u8>,
        cur: usize,
        pos: usize,
        linebuf: bool,
        read: bool,
    },
}

#[cfg(any(not(target_arch = "wasm32"), target_os = "wasi"))]
pub(in crate::runtime) type NativeFileHandle = std::rc::Rc<std::cell::RefCell<std::fs::File>>;

#[derive(Clone, Debug)]
pub(in crate::runtime) struct DirHandle {
    pub(in crate::runtime) entries: Vec<Vec<u8>>,
    pub(in crate::runtime) pos: usize,
}

#[cfg_attr(all(target_arch = "wasm32", not(target_os = "wasi")), allow(dead_code))]
#[derive(Clone, Copy, Debug)]
pub(in crate::runtime) struct NativeFileMode {
    pub(in crate::runtime) readable: bool,
    pub(in crate::runtime) writable: bool,
    pub(in crate::runtime) append: bool,
    pub(in crate::runtime) truncate: bool,
    pub(in crate::runtime) create: bool,
}

#[derive(Clone, Copy, Debug)]
pub(in crate::runtime) struct HostIntResult {
    pub(in crate::runtime) value: i64,
    pub(in crate::runtime) errno: Option<i32>,
}

#[cfg_attr(target_arch = "wasm32", allow(dead_code))]
impl HostIntResult {
    pub(in crate::runtime) fn ok(value: i64) -> Self {
        Self { value, errno: None }
    }

    pub(in crate::runtime) fn err(errno: i32) -> Self {
        Self {
            value: -1,
            errno: Some(errno),
        }
    }

    pub(in crate::runtime) fn os_err(errno: Option<i32>) -> Self {
        Self::err(errno.unwrap_or_else(|| errno_i32("EINVAL")))
    }
}

#[derive(Debug)]
pub enum EvalError {
    StepLimit { limit: usize },
    DanglingIndirection(NodeId),
    ExpectedInt(NodeId),
    ExpectedInt64(NodeId),
    ExpectedFloat64(NodeId),
    ExpectedFloat32(NodeId),
    ExpectedThreadId(NodeId),
    ExpectedPointer(NodeId),
    ExpectedForeignPtr(NodeId),
    ExpectedWeak(NodeId),
    ExpectedMVar(NodeId),
    ExpectedBytes(NodeId),
    ExpectedArray(NodeId),
    DivideByZero,
    Overflow,
    InvalidShift(i64),
    InvalidByteString,
    InvalidArray,
    Raised(NodeId),
    InvalidStablePtr,
    InvalidMVar,
    InvalidHandle,
    UnknownPrim(String),
    UnknownFfi(String),
    UnsupportedForeignFinalizer(String),
    UnsupportedJsFfi,
    UnsupportedSerialization(NodeId),
}

impl fmt::Display for EvalError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::StepLimit { limit } => write!(f, "reduction step limit reached ({limit})"),
            Self::DanglingIndirection(id) => write!(f, "dangling shared reference at node {id:?}"),
            Self::ExpectedInt(id) => write!(f, "expected Int at node {id:?}"),
            Self::ExpectedInt64(id) => write!(f, "expected Int64 at node {id:?}"),
            Self::ExpectedFloat64(id) => write!(f, "expected Float64 at node {id:?}"),
            Self::ExpectedFloat32(id) => write!(f, "expected Float32 at node {id:?}"),
            Self::ExpectedThreadId(id) => write!(f, "expected ThreadId at node {id:?}"),
            Self::ExpectedPointer(id) => write!(f, "expected pointer-like node {id:?}"),
            Self::ExpectedForeignPtr(id) => write!(f, "expected ForeignPtr at node {id:?}"),
            Self::ExpectedWeak(id) => write!(f, "expected Weak pointer at node {id:?}"),
            Self::ExpectedMVar(id) => write!(f, "expected MVar at node {id:?}"),
            Self::ExpectedBytes(id) => write!(f, "expected ByteString at node {id:?}"),
            Self::ExpectedArray(id) => write!(f, "expected Array at node {id:?}"),
            Self::DivideByZero => write!(f, "integer division by zero"),
            Self::Overflow => write!(f, "integer overflow"),
            Self::InvalidShift(n) => write!(f, "invalid shift amount {n}"),
            Self::InvalidByteString => write!(f, "invalid ByteString operation"),
            Self::InvalidArray => write!(f, "invalid Array operation"),
            Self::Raised(id) => write!(f, "uncaught exception at node {id:?}"),
            Self::InvalidStablePtr => write!(f, "invalid StablePtr operation"),
            Self::InvalidMVar => write!(f, "invalid MVar operation"),
            Self::InvalidHandle => write!(f, "invalid IO handle operation"),
            Self::UnknownPrim(name) => write!(f, "unknown primitive {name}"),
            Self::UnknownFfi(name) => write!(f, "unknown FFI symbol {name}"),
            Self::UnsupportedForeignFinalizer(name) => {
                write!(f, "unsupported ForeignPtr finalizer {name}")
            }
            Self::UnsupportedJsFfi => write!(f, "JavaScript FFI is not supported in this runtime"),
            Self::UnsupportedSerialization(id) => {
                write!(f, "cannot serialize node {id:?}")
            }
        }
    }
}

impl std::error::Error for EvalError {}

#[cfg_attr(
    not(all(target_arch = "wasm32", not(target_os = "wasi"))),
    allow(dead_code)
)]
pub(in crate::runtime) enum JsArg {
    Int(i32),
    UInt(u32),
    Double(f64),
    Object(u32),
    String(Vec<u8>),
}

#[derive(Clone, Debug, PartialEq)]
pub enum JsValue {
    Unit,
    Int(i32),
    UInt(u32),
    Double(f64),
    Float(f32),
    Bool(bool),
    Pointer(u32),
    Object(u32),
    Bytes(Vec<u8>),
}

#[derive(Clone, Debug)]
pub struct Program {
    pub(in crate::runtime) nodes: Vec<Cell>,
    pub(in crate::runtime) cold_nodes: Vec<Option<Node>>,
    pub(in crate::runtime) root: NodeId,
    pub(in crate::runtime) labels: HashMap<usize, NodeId>,
    pub(in crate::runtime) node_pointers: Vec<NodeId>,
    pub(in crate::runtime) node_pointer_slots: HashMap<NodeId, usize>,
    pub(in crate::runtime) free_head: Option<NodeId>,
    pub(in crate::runtime) free_nodes: usize,
    pub(in crate::runtime) gc_node_interval: usize,
    pub(in crate::runtime) gc_allocations_since_collect: usize,
    pub(in crate::runtime) gc_last_allocations_since_collect: usize,
    pub(in crate::runtime) gc_collections: usize,
    pub(in crate::runtime) gc_freed_nodes_total: usize,
    pub(in crate::runtime) gc_last_live_nodes: usize,
    pub(in crate::runtime) gc_last_free_nodes: usize,
    pub(in crate::runtime) gc_high_water_nodes: usize,
    pub(in crate::runtime) gc_last_pause_nanos: u128,
    pub(in crate::runtime) gc_total_pause_nanos: u128,
    #[cfg(feature = "gc-phase-profile")]
    pub(in crate::runtime) gc_last_mark_nanos: u128,
    #[cfg(feature = "gc-phase-profile")]
    pub(in crate::runtime) gc_total_mark_nanos: u128,
    #[cfg(feature = "gc-phase-profile")]
    pub(in crate::runtime) gc_last_sweep_nanos: u128,
    #[cfg(feature = "gc-phase-profile")]
    pub(in crate::runtime) gc_total_sweep_nanos: u128,
    #[cfg(feature = "gc-phase-profile")]
    pub(in crate::runtime) gc_red_i_opportunities: usize,
    #[cfg(feature = "gc-phase-profile")]
    pub(in crate::runtime) gc_red_k_opportunities: usize,
    #[cfg(feature = "gc-phase-profile")]
    pub(in crate::runtime) gc_red_a_opportunities: usize,
    #[cfg(feature = "gc-phase-profile")]
    pub(in crate::runtime) gc_red_bi_opportunities: usize,
    #[cfg(feature = "gc-phase-profile")]
    pub(in crate::runtime) gc_red_bxi_opportunities: usize,
    #[cfg(feature = "gc-phase-profile")]
    pub(in crate::runtime) gc_red_ccbi_opportunities: usize,
    #[cfg(feature = "gc-phase-profile")]
    pub(in crate::runtime) gc_red_cc_opportunities: usize,
    #[cfg(feature = "gc-phase-profile")]
    pub(in crate::runtime) gc_red_cci_opportunities: usize,
    #[cfg(feature = "gc-phase-profile")]
    pub(in crate::runtime) gc_red_ccbbcp_opportunities: usize,
    #[cfg(feature = "gc-phase-profile")]
    pub(in crate::runtime) gc_red_flip_opportunities: usize,
    #[cfg(feature = "gc-phase-profile")]
    pub(in crate::runtime) gc_young_profile_allocated_slots: Vec<NodeId>,
    #[cfg(feature = "gc-phase-profile")]
    pub(in crate::runtime) gc_young_profile_last_slots: usize,
    #[cfg(feature = "gc-phase-profile")]
    pub(in crate::runtime) gc_young_profile_last_live: usize,
    #[cfg(feature = "gc-phase-profile")]
    pub(in crate::runtime) gc_young_profile_last_dead: usize,
    #[cfg(feature = "gc-phase-profile")]
    pub(in crate::runtime) gc_young_profile_last_old_to_young_sources: usize,
    #[cfg(feature = "gc-phase-profile")]
    pub(in crate::runtime) gc_young_profile_last_old_to_young_edges: usize,
    #[cfg(feature = "gc-phase-profile")]
    pub(in crate::runtime) gc_young_profile_total_slots: usize,
    #[cfg(feature = "gc-phase-profile")]
    pub(in crate::runtime) gc_young_profile_total_live: usize,
    #[cfg(feature = "gc-phase-profile")]
    pub(in crate::runtime) gc_young_profile_total_dead: usize,
    #[cfg(feature = "gc-phase-profile")]
    pub(in crate::runtime) gc_young_profile_total_old_to_young_sources: usize,
    #[cfg(feature = "gc-phase-profile")]
    pub(in crate::runtime) gc_young_profile_total_old_to_young_edges: usize,
    pub(in crate::runtime) gc_marked: Vec<bool>,
    pub(in crate::runtime) gc_mark_work: Vec<NodeId>,
    pub(in crate::runtime) gc_foreign_finalizer_marked: Vec<bool>,
    pub(in crate::runtime) gc_events: Vec<GcEventStats>,
    pub(in crate::runtime) stable_ptrs: Vec<Option<NodeId>>,
    pub(in crate::runtime) weak_nodes: Vec<NodeId>,
    pub(in crate::runtime) pending_weak_finalizers: Vec<NodeId>,
    pub(in crate::runtime) foreign_finalizers: Vec<Option<ForeignFinalizerState>>,
    pub(in crate::runtime) foreign_finalizer_free: Vec<usize>,
    pub(in crate::runtime) allocations: Vec<Option<Vec<u8>>>,
    pub(in crate::runtime) bfiles: Vec<Option<BFile>>,
    pub(in crate::runtime) dirs: Vec<Option<DirHandle>>,
    pub(in crate::runtime) program_args: Vec<Vec<u8>>,
    pub(in crate::runtime) executable_path: Option<Vec<u8>>,
    pub(in crate::runtime) arg_ref_array: Option<NodeId>,
    pub(in crate::runtime) errno_value: i32,
    pub(in crate::runtime) errno_ptr: Option<i64>,
    pub(in crate::runtime) masking_state: i64,
    pub(in crate::runtime) reductions: usize,
    pub(in crate::runtime) js_program_handle: Option<u32>,
    pub(in crate::runtime) js_wrapper_tags: Vec<String>,
    pub(in crate::runtime) prim_cache: PrimCache,
    pub(in crate::runtime) compound_cache: CompoundCache,
    pub(in crate::runtime) small_ints: [Option<NodeId>; SMALL_INT_COUNT],
    pub(in crate::runtime) world: Option<NodeId>,
    pub(in crate::runtime) profile: Option<EvalProfile>,
    pub(in crate::runtime) trace_expected_bytes: bool,
    pub(in crate::runtime) reduce_depth: usize,
}
