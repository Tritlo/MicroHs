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

std::cfg_select! {
    feature = "wide-cell" => {
        /// One hot heap slot.
        ///
        /// The `wide-cell` feature is an opt-in correctness escape hatch: it
        /// keeps the same cold-node policy but widens each slot to 16 bytes so
        /// App/Indir/Free ids are stored directly as u32 payloads and the arena
        /// cap rises to u32::MAX.
        #[repr(C, align(8))]
        #[derive(Clone, Copy, Debug)]
        pub(in crate::runtime) struct Cell {
            pub(in crate::runtime) tag: u8,
            pub(in crate::runtime) _pad: [u8; 3],
            pub(in crate::runtime) left: u32,
            pub(in crate::runtime) right: u64,
        }
    }
    _ => {
        /// One hot heap slot.
        ///
        /// The default representation stores the common graph shapes in one
        /// packed word. Values that do not fit in the 4-bit tag plus 60-bit
        /// payload are represented as `CellTag::Cold` and live in
        /// `Program::cold_nodes`.
        #[derive(Clone, Copy, Debug)]
        pub(in crate::runtime) struct Cell {
            pub(in crate::runtime) word: u64,
        }
    }
}

/// Packed-cell tags for the hot arena representation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(in crate::runtime) enum CellTag {
    App,
    Indir,
    Free,
    KnownPrim,
    RuntimePrim,
    Int,
    Float32,
    ThreadId,
    Cold,
}

std::cfg_select! {
    feature = "wide-cell" => {
        pub(in crate::runtime) const CELL_TAG_BITS: u64 = 0xff;
        pub(in crate::runtime) const CELL_NONE_ID: u64 = u32::MAX as u64;
    }
    _ => {
        pub(in crate::runtime) const CELL_TAG_BITS: u64 = 0x0f;
        pub(in crate::runtime) const CELL_PAYLOAD_SHIFT: u64 = 4;
        pub(in crate::runtime) const CELL_NONE_ID: u64 = (1_u64 << PACKED_ID_BITS) - 1;
        pub(in crate::runtime) const PACKED_ID_BITS: u64 = 30;
        pub(in crate::runtime) const PACKED_ID_MASK: u64 = (1_u64 << PACKED_ID_BITS) - 1;
        pub(in crate::runtime) const PACKED_PAYLOAD1_SHIFT: u64 =
            CELL_PAYLOAD_SHIFT + PACKED_ID_BITS;
        pub(in crate::runtime) const PACKED_SCALAR_PAYLOAD_MASK: u64 = (1_u64 << 60) - 1;
    }
}
pub(in crate::runtime) const PACKED_INT_MIN: i64 = -(1_i64 << 59);
pub(in crate::runtime) const PACKED_INT_MAX: i64 = (1_i64 << 59) - 1;

std::cfg_select! {
    feature = "wide-cell" => {
        const _: [(); 16] = [(); std::mem::size_of::<Cell>()];
    }
    _ => {
        const _: [(); 8] = [(); std::mem::size_of::<Cell>()];
    }
}

impl CellTag {
    pub(in crate::runtime) fn from_bits(bits: u64) -> Self {
        match bits & CELL_TAG_BITS {
            0 => Self::App,
            1 => Self::Indir,
            2 => Self::Free,
            3 => Self::KnownPrim,
            4 => Self::RuntimePrim,
            5 => Self::Int,
            6 => Self::Float32,
            7 => Self::ThreadId,
            8 => Self::Cold,
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
            Self::Float32 => 6,
            Self::ThreadId => 7,
            Self::Cold => 8,
        }
    }
}

impl Cell {
    #[inline]
    pub(in crate::runtime) fn tag_bits(self) -> u64 {
        std::cfg_select! {
            feature = "wide-cell" => {
                u64::from(self.tag)
            }
            _ => {
                self.word & CELL_TAG_BITS
            }
        }
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
            Node::Int(value) if can_inline_int(value) => Self::int(value),
            Node::Float32(value) => Self::float32(value),
            Node::ThreadId(value) if can_inline_int(value) => Self::thread_id(value),
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
            CellTag::KnownPrim => {
                Node::Prim(Prim::Known(decode_known_prim(self.payload0() as u16)))
            }
            CellTag::RuntimePrim => Node::Prim(Prim::Runtime(RuntimePrim(self.payload0() as u16))),
            CellTag::Int => Node::Int(self.int_value().expect("Int cell must decode")),
            CellTag::Float32 => {
                Node::Float32(self.float32_value().expect("Float32 cell must decode"))
            }
            CellTag::ThreadId => {
                Node::ThreadId(self.thread_id_value().expect("ThreadId cell must decode"))
            }
            CellTag::Cold => cold_nodes[self.cold_index().expect("Cold cell must decode")]
                .as_ref()
                .expect("live cold cell pointed at freed cold node")
                .clone(),
        }
    }

    pub(in crate::runtime) fn tag(self) -> CellTag {
        CellTag::from_bits(self.tag_bits())
    }

    pub(in crate::runtime) fn app(fun: NodeId, arg: NodeId) -> Self {
        Self::two_id_payloads(CellTag::App, fun, arg)
    }

    /// `Cell::app` for ids that are known to fit the active arena-id field.
    ///
    /// INVARIANT (arena-id fit): every `NodeId` created by the runtime is
    /// `< CELL_NONE_ID`. `NodeId`s are only minted by the node arena, and
    /// [`Program::push_cell`] asserts `nodes.len() < CELL_NONE_ID` on every
    /// growth, so no id can ever overflow the active cell field. That lets the
    /// hot rewrite/allocation paths skip the id-range handling `Cell::app`
    /// would otherwise do. The `debug_assert`s re-check the invariant in debug
    /// builds (exercised by the debug-build test suite); if it were violated
    /// the id would alias adjacent fields and later index the arena out
    /// of bounds, so callers must only pass existing heap `NodeId`s.
    #[inline(always)]
    pub(in crate::runtime) fn app_trusted(fun: NodeId, arg: NodeId) -> Self {
        debug_assert!(u64::from(fun.0) < CELL_NONE_ID);
        debug_assert!(u64::from(arg.0) < CELL_NONE_ID);
        std::cfg_select! {
            feature = "wide-cell" => {
                Self {
                    tag: CellTag::App.bits() as u8,
                    _pad: [0; 3],
                    left: fun.0,
                    right: u64::from(arg.0),
                }
            }
            _ => {
                Self {
                    word: (u64::from(arg.0) << PACKED_PAYLOAD1_SHIFT)
                        | (u64::from(fun.0) << CELL_PAYLOAD_SHIFT)
                        | CellTag::App.bits(),
                }
            }
        }
    }

    pub(in crate::runtime) fn indir(target: Option<NodeId>) -> Self {
        Self::with_payload0(CellTag::Indir, pack_option_id(target))
    }

    /// `Cell::indir(Some(target))` relying on the packed-id-fit invariant
    /// documented on [`Cell::app_trusted`].
    #[inline(always)]
    pub(in crate::runtime) fn indir_trusted(target: NodeId) -> Self {
        debug_assert!(u64::from(target.0) < CELL_NONE_ID);
        std::cfg_select! {
            feature = "wide-cell" => {
                Self {
                    tag: CellTag::Indir.bits() as u8,
                    _pad: [0; 3],
                    left: 0,
                    right: u64::from(target.0),
                }
            }
            _ => {
                Self {
                    word: (u64::from(target.0) << CELL_PAYLOAD_SHIFT) | CellTag::Indir.bits(),
                }
            }
        }
    }

    pub(in crate::runtime) fn free(next: Option<NodeId>) -> Self {
        Self::with_payload0(CellTag::Free, pack_option_id(next))
    }

    pub(in crate::runtime) fn known_prim(known: KnownPrim) -> Self {
        Self::with_payload0(CellTag::KnownPrim, u64::from(encode_known_prim(known)))
    }

    /// `Cell::known_prim` for the hot result paths. `encode_known_prim` always
    /// returns a small fixed code that fits the packed field, so this is
    /// unconditionally sound; it exists only to inline the fast construction.
    #[inline(always)]
    pub(in crate::runtime) fn known_prim_trusted(known: KnownPrim) -> Self {
        std::cfg_select! {
            feature = "wide-cell" => {
                Self {
                    tag: CellTag::KnownPrim.bits() as u8,
                    _pad: [0; 3],
                    left: 0,
                    right: u64::from(encode_known_prim(known)),
                }
            }
            _ => {
                Self {
                    word: (u64::from(encode_known_prim(known)) << CELL_PAYLOAD_SHIFT)
                        | CellTag::KnownPrim.bits(),
                }
            }
        }
    }

    pub(in crate::runtime) fn runtime_prim(runtime: RuntimePrim) -> Self {
        Self::with_payload0(CellTag::RuntimePrim, u64::from(runtime.0))
    }

    pub(in crate::runtime) fn int(value: i64) -> Self {
        debug_assert!(can_inline_int(value));
        Self::signed_payload(CellTag::Int, value)
    }

    /// `Cell::int` for callers that have already checked `can_inline_int`
    /// (i.e. the value fits the packed signed payload). The `debug_assert`
    /// re-checks it; an out-of-range value would corrupt the tag bits.
    #[inline(always)]
    pub(in crate::runtime) fn int_trusted(value: i64) -> Self {
        debug_assert!(can_inline_int(value));
        std::cfg_select! {
            feature = "wide-cell" => {
                Self {
                    tag: CellTag::Int.bits() as u8,
                    _pad: [0; 3],
                    left: 0,
                    right: value as u64,
                }
            }
            _ => {
                Self {
                    word: ((value as u64) << CELL_PAYLOAD_SHIFT) | CellTag::Int.bits(),
                }
            }
        }
    }

    pub(in crate::runtime) fn float32(value: f32) -> Self {
        Self::unsigned_payload(CellTag::Float32, u64::from(value.to_bits()))
    }

    pub(in crate::runtime) fn thread_id(value: i64) -> Self {
        debug_assert!(can_inline_int(value));
        Self::signed_payload(CellTag::ThreadId, value)
    }

    pub(in crate::runtime) fn cold(index: usize) -> Self {
        let index = u64::try_from(index).expect("cold node table exceeded u64");
        debug_assert!(index < CELL_NONE_ID);
        Self::with_payload0(CellTag::Cold, index)
    }

    pub(in crate::runtime) fn id_payload(self) -> NodeId {
        std::cfg_select! {
            feature = "wide-cell" => {
                debug_assert_eq!(self.tag_bits(), CellTag::App.bits());
                NodeId(self.left)
            }
            _ => {
                NodeId(self.payload0() as u32)
            }
        }
    }

    pub(in crate::runtime) fn id_word1(self) -> NodeId {
        std::cfg_select! {
            feature = "wide-cell" => {
                debug_assert_eq!(self.tag_bits(), CellTag::App.bits());
                NodeId(self.right as u32)
            }
            _ => {
                NodeId(self.payload1() as u32)
            }
        }
    }

    pub(in crate::runtime) fn option_id_word1(self) -> Option<NodeId> {
        unpack_option_id(self.payload0())
    }

    #[inline]
    pub(in crate::runtime) fn app_fun_trusted(self) -> Option<NodeId> {
        let tag = self.tag_bits();
        if tag == CellTag::App.bits() {
            Some(self.id_payload())
        } else {
            debug_assert_ne!(tag, CellTag::Indir.bits());
            debug_assert_ne!(tag, CellTag::Free.bits());
            None
        }
    }

    #[inline]
    pub(in crate::runtime) fn indir_target_trusted(self) -> Option<NodeId> {
        debug_assert_eq!(self.tag_bits(), CellTag::Indir.bits());
        self.option_id_word1()
    }

    pub(in crate::runtime) fn app_fields(self) -> Option<(NodeId, NodeId)> {
        self.has_tag(CellTag::App)
            .then(|| (self.id_payload(), self.id_word1()))
    }

    #[inline(always)]
    pub(in crate::runtime) fn prim(self) -> Option<Prim> {
        match self.tag_bits() {
            3 => Some(Prim::Known(decode_known_prim(self.payload0() as u16))),
            4 => Some(Prim::Runtime(RuntimePrim(self.payload0() as u16))),
            _ => None,
        }
    }

    pub(in crate::runtime) fn int_value(self) -> Option<i64> {
        self.has_tag(CellTag::Int)
            .then(|| self.signed_payload_value())
    }

    pub(in crate::runtime) fn thread_id_value(self) -> Option<i64> {
        self.has_tag(CellTag::ThreadId)
            .then(|| self.signed_payload_value())
    }

    pub(in crate::runtime) fn float32_value(self) -> Option<f32> {
        self.has_tag(CellTag::Float32)
            .then(|| f32::from_bits(self.unsigned_payload_value() as u32))
    }

    pub(in crate::runtime) fn cold_index(self) -> Option<usize> {
        self.has_tag(CellTag::Cold)
            .then_some(self.payload0() as usize)
    }

    #[inline]
    pub(in crate::runtime) fn payload0(self) -> u64 {
        std::cfg_select! {
            feature = "wide-cell" => {
                self.right
            }
            _ => {
                (self.word >> CELL_PAYLOAD_SHIFT) & PACKED_ID_MASK
            }
        }
    }

    #[inline]
    #[cfg_attr(feature = "wide-cell", allow(dead_code))]
    pub(in crate::runtime) fn payload1(self) -> u64 {
        std::cfg_select! {
            feature = "wide-cell" => {
                self.right
            }
            _ => {
                (self.word >> PACKED_PAYLOAD1_SHIFT) & PACKED_ID_MASK
            }
        }
    }

    #[inline]
    pub(in crate::runtime) fn signed_payload_value(self) -> i64 {
        std::cfg_select! {
            feature = "wide-cell" => {
                self.right as i64
            }
            _ => {
                (self.word as i64) >> CELL_PAYLOAD_SHIFT
            }
        }
    }

    #[inline]
    pub(in crate::runtime) fn unsigned_payload_value(self) -> u64 {
        std::cfg_select! {
            feature = "wide-cell" => {
                self.right
            }
            _ => {
                (self.word >> CELL_PAYLOAD_SHIFT) & PACKED_SCALAR_PAYLOAD_MASK
            }
        }
    }

    #[inline]
    pub(in crate::runtime) fn with_payload0(tag: CellTag, payload: u64) -> Self {
        std::cfg_select! {
            feature = "wide-cell" => {
                Self {
                    tag: tag.bits() as u8,
                    _pad: [0; 3],
                    left: 0,
                    right: payload,
                }
            }
            _ => {
                assert!(payload <= PACKED_ID_MASK, "packed cell payload overflow");
                Self {
                    word: (payload << CELL_PAYLOAD_SHIFT) | tag.bits(),
                }
            }
        }
    }

    #[inline]
    pub(in crate::runtime) fn two_id_payloads(tag: CellTag, left: NodeId, right: NodeId) -> Self {
        debug_assert_eq!(tag, CellTag::App);
        std::cfg_select! {
            feature = "wide-cell" => {
                let left = pack_id_payload(left);
                let right = pack_id_payload(right);
                Self {
                    tag: tag.bits() as u8,
                    _pad: [0; 3],
                    left: left as u32,
                    right,
                }
            }
            _ => {
                let left = pack_id_payload(left);
                let right = pack_id_payload(right);
                Self {
                    word: (right << PACKED_PAYLOAD1_SHIFT)
                        | (left << CELL_PAYLOAD_SHIFT)
                        | tag.bits(),
                }
            }
        }
    }

    #[inline]
    pub(in crate::runtime) fn signed_payload(tag: CellTag, value: i64) -> Self {
        assert!(can_inline_int(value), "packed signed cell payload overflow");
        std::cfg_select! {
            feature = "wide-cell" => {
                Self {
                    tag: tag.bits() as u8,
                    _pad: [0; 3],
                    left: 0,
                    right: value as u64,
                }
            }
            _ => {
                Self {
                    word: ((value as u64) << CELL_PAYLOAD_SHIFT) | tag.bits(),
                }
            }
        }
    }

    #[inline]
    pub(in crate::runtime) fn unsigned_payload(tag: CellTag, value: u64) -> Self {
        std::cfg_select! {
            feature = "wide-cell" => {
                Self {
                    tag: tag.bits() as u8,
                    _pad: [0; 3],
                    left: 0,
                    right: value,
                }
            }
            _ => {
                assert!(
                    value <= PACKED_SCALAR_PAYLOAD_MASK,
                    "packed unsigned cell payload overflow"
                );
                Self {
                    word: (value << CELL_PAYLOAD_SHIFT) | tag.bits(),
                }
            }
        }
    }
}

pub(in crate::runtime) fn pack_option_id(id: Option<NodeId>) -> u64 {
    id.map_or(CELL_NONE_ID, pack_id_payload)
}

pub(in crate::runtime) fn unpack_option_id(word: u64) -> Option<NodeId> {
    (word != CELL_NONE_ID).then_some(NodeId(word as u32))
}

#[inline]
pub(in crate::runtime) fn pack_id_payload(id: NodeId) -> u64 {
    let packed = u64::from(id.0);
    assert!(packed < CELL_NONE_ID, "cell NodeId overflow");
    packed
}

#[inline]
pub(in crate::runtime) fn can_inline_int(value: i64) -> bool {
    (PACKED_INT_MIN..=PACKED_INT_MAX).contains(&value)
}

#[derive(Default)]
pub(in crate::runtime) struct SerializationLabels {
    pub(in crate::runtime) shared: HashSet<NodeId>,
    pub(in crate::runtime) printed: HashSet<NodeId>,
}

#[derive(Clone, Debug)]
pub struct ForeignPtrNode {
    pub(crate) bytes: Option<Rc<[u8]>>,
    pub(crate) offset: usize,
    pub(crate) ptr: i64,
    pub(crate) finalizer: Option<usize>,
}

#[derive(Clone, Debug)]
pub(in crate::runtime) enum ForeignFinalizer {
    Free,
    CloseB,
    JsObjFree,
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
    pub(crate) fn prim(name: &str) -> Self {
        let prim = Prim::from_name(name)
            .unwrap_or_else(|| panic!("unknown runtime primitive requested internally: {name}"));
        Self::Prim(prim)
    }

    pub(crate) fn bigint(bytes: Vec<u8>) -> Self {
        Self::BigInt(Box::new(bytes))
    }

    pub(crate) fn bytes(bytes: Vec<u8>) -> Self {
        Self::Bytes(Box::new(bytes))
    }

    pub(in crate::runtime) fn bytes_view(base: NodeId, offset: usize, len: usize) -> Self {
        Self::BytesView(Box::new(BytesViewNode { base, offset, len }))
    }

    pub(crate) fn array(items: Vec<NodeId>) -> Self {
        Self::Array(Box::new(items))
    }

    pub(crate) fn ffi(name: String) -> Self {
        Self::Ffi(Box::new(name))
    }

    pub(crate) fn js_wrap(tags: String) -> Self {
        Self::JsWrap {
            tags: Box::new(tags),
        }
    }

    pub(crate) fn fun_ptr(name: impl Into<String>) -> Self {
        Self::FunPtr(Box::new(name.into()))
    }

    pub(crate) fn tick(bytes: Vec<u8>) -> Self {
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
pub(in crate::runtime) const REDUCTION_SLICE: usize = 100_000;
pub(in crate::runtime) const RTS_EXN_DIVIDE_BY_ZERO: i64 = 4;
pub(in crate::runtime) const RTS_EXN_OVERFLOW: i64 = 7;
pub(in crate::runtime) const RTS_EXN_SERIALIZE: i64 = 8;
pub(in crate::runtime) const RTS_EXN_DESERIALIZE: i64 = 9;
pub(in crate::runtime) const MASK_UNMASKED: i64 = 0;
pub(in crate::runtime) const MASK_INTERRUPTIBLE: i64 = 1;
pub(in crate::runtime) const MASK_UNINTERRUPTIBLE: i64 = 2;
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
// Default cells between GCs when MHS_GC_NODE_INTERVAL is unset: a lean 16M on
// wasm32-wasi (memory-constrained hosts; ~180MB peak on the heavy self-host) vs
// 75M native (~C's default footprint). Both packed-cell counts, overridable.
pub(in crate::runtime) const GC_NODE_INTERVAL: usize = std::cfg_select! {
    target_os = "wasi" => { 16 * 1024 * 1024 }
    _ => { 75 * 1024 * 1024 }
};

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
pub(in crate::runtime) const NATIVE_FILE_WRITE_BUFFER_CAPACITY: usize = 4096;

#[cfg(any(not(target_arch = "wasm32"), target_os = "wasi"))]
#[derive(Clone, Debug)]
pub(in crate::runtime) struct NativeFileHandle {
    inner: std::rc::Rc<std::cell::RefCell<NativeFileState>>,
}

#[cfg(any(not(target_arch = "wasm32"), target_os = "wasi"))]
#[derive(Debug)]
pub(in crate::runtime) struct NativeFileState {
    file: std::fs::File,
    write_buffer: Option<Vec<u8>>,
}

#[cfg(any(not(target_arch = "wasm32"), target_os = "wasi"))]
impl NativeFileHandle {
    pub(in crate::runtime) fn raw(file: std::fs::File) -> Self {
        Self {
            inner: std::rc::Rc::new(std::cell::RefCell::new(NativeFileState {
                file,
                write_buffer: None,
            })),
        }
    }

    pub(in crate::runtime) fn buffered_write(file: std::fs::File) -> Self {
        Self {
            inner: std::rc::Rc::new(std::cell::RefCell::new(NativeFileState {
                file,
                write_buffer: Some(Vec::with_capacity(NATIVE_FILE_WRITE_BUFFER_CAPACITY)),
            })),
        }
    }

    pub(in crate::runtime) fn read_byte(&self) -> std::io::Result<Option<u8>> {
        use std::io::Read as _;

        let mut state = self.inner.borrow_mut();
        let mut byte = [0];
        match state.file.read(&mut byte)? {
            0 => Ok(None),
            _ => Ok(Some(byte[0])),
        }
    }

    pub(in crate::runtime) fn read(&self, bytes: &mut [u8]) -> std::io::Result<usize> {
        use std::io::Read as _;

        self.inner.borrow_mut().file.read(bytes)
    }

    pub(in crate::runtime) fn write_byte(&self, byte: u8) -> std::io::Result<()> {
        use std::io::Write as _;

        let mut state = self.inner.borrow_mut();
        if state.write_buffer.is_some() {
            if state.write_buffer.as_ref().unwrap().len() == NATIVE_FILE_WRITE_BUFFER_CAPACITY {
                state.flush_write_buffer()?;
            }
            state.write_buffer.as_mut().unwrap().push(byte);
            Ok(())
        } else {
            state.file.write_all(&[byte])
        }
    }

    pub(in crate::runtime) fn write_all(&self, bytes: &[u8]) -> std::io::Result<()> {
        use std::io::Write as _;

        let mut state = self.inner.borrow_mut();
        let Some(buffer) = state.write_buffer.as_ref() else {
            return state.file.write_all(bytes);
        };
        if bytes.len() >= NATIVE_FILE_WRITE_BUFFER_CAPACITY {
            state.flush_write_buffer()?;
            return state.file.write_all(bytes);
        }
        if buffer.len() + bytes.len() > NATIVE_FILE_WRITE_BUFFER_CAPACITY {
            state.flush_write_buffer()?;
        }
        state
            .write_buffer
            .as_mut()
            .unwrap()
            .extend_from_slice(bytes);
        Ok(())
    }

    pub(in crate::runtime) fn flush(&self) -> std::io::Result<()> {
        use std::io::Write as _;

        let mut state = self.inner.borrow_mut();
        state.flush_write_buffer()?;
        state.file.flush()
    }
}

#[cfg(any(not(target_arch = "wasm32"), target_os = "wasi"))]
impl NativeFileState {
    fn flush_write_buffer(&mut self) -> std::io::Result<()> {
        use std::io::Write as _;

        if let Some(buffer) = &mut self.write_buffer
            && !buffer.is_empty()
        {
            self.file.write_all(buffer)?;
            buffer.clear();
        }
        Ok(())
    }
}

#[cfg(any(not(target_arch = "wasm32"), target_os = "wasi"))]
impl Drop for NativeFileState {
    fn drop(&mut self) {
        let _ = self.flush_write_buffer();
        use std::io::Write as _;
        let _ = self.file.flush();
    }
}

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
    StepLimit {
        limit: usize,
    },
    #[cfg(feature = "embedded")]
    Cancelled,
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
    Blocked(BlockReason),
    InvalidStablePtr,
    InvalidMVar,
    InvalidHandle,
    UnknownPrim(String),
    UnknownFfi(String),
    UnsupportedForeignFinalizer(String),
    UnsupportedJsFfi,
    UnsupportedSerialization(NodeId),
    Deadlock,
}

#[derive(Clone, Copy, Debug)]
pub enum BlockReason {
    TakeMVar(NodeId),
    PutMVar(NodeId),
    ReadMVar(NodeId),
    Delay(u128),
}

impl fmt::Display for EvalError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::StepLimit { limit } => write!(f, "reduction step limit reached ({limit})"),
            #[cfg(feature = "embedded")]
            Self::Cancelled => write!(f, "reduction cancelled by host"),
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
            Self::Blocked(_) => write!(f, "thread blocked"),
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
            Self::Deadlock => write!(f, "all threads blocked indefinitely"),
        }
    }
}

/// A cooperative green thread: an IO computation reduced in slices by the
/// scheduler. `root` is the stable node whose in-place reduction advances the
/// thread's continuation; re-reducing it resumes where a slice left off.
#[derive(Clone, Debug)]
pub(in crate::runtime) struct ThreadControl {
    pub(in crate::runtime) id: i64,
    pub(in crate::runtime) root: NodeId,
    pub(in crate::runtime) delivered_value: Option<NodeId>,
    pub(in crate::runtime) pending_exception: Option<NodeId>,
    pub(in crate::runtime) delay_ready: bool,
    pub(in crate::runtime) masking_state: i64,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(in crate::runtime) enum ThreadState {
    Runnable,
    BlockedMVar,
    BlockedOther,
    Finished,
}

#[derive(Clone, Debug, Default)]
pub(in crate::runtime) struct MVarWaitQueues {
    pub(in crate::runtime) takeput: std::collections::VecDeque<usize>,
    pub(in crate::runtime) read: std::collections::VecDeque<usize>,
}

impl std::error::Error for EvalError {}

/// Loaded comb program plus all mutable runtime state.
///
/// `Program` owns the packed heap, cold payload table, host resources, caches,
/// GC scratch buffers, and reduction counters. Methods are split across
/// `runtime/program/*` modules, but the state intentionally stays together so
/// the evaluator can mutate the heap and host tables without extra indirection.
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
    /// Set by `performGC` (IO.gc) to force a full collection at the next top-level
    /// step boundary, regardless of the allocation threshold.
    pub(in crate::runtime) force_gc: bool,
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
    pub(in crate::runtime) gc_marked: Vec<bool>,
    pub(in crate::runtime) gc_mark_work: Vec<NodeId>,
    pub(in crate::runtime) gc_foreign_finalizer_marked: Vec<bool>,
    pub(in crate::runtime) gc_events: Vec<GcEventStats>,
    pub(in crate::runtime) stable_ptrs: Vec<Option<NodeId>>,
    pub(in crate::runtime) stable_ptr_first_free: usize,
    pub(in crate::runtime) weak_nodes: Vec<NodeId>,
    pub(in crate::runtime) pending_weak_finalizers: Vec<NodeId>,
    pub(in crate::runtime) foreign_finalizers: Vec<Option<ForeignFinalizerState>>,
    pub(in crate::runtime) foreign_finalizer_free: Vec<usize>,
    pub(in crate::runtime) allocations: Vec<Option<Vec<u8>>>,
    pub(in crate::runtime) allocation_first_free: usize,
    pub(in crate::runtime) bfiles: Vec<Option<BFile>>,
    pub(in crate::runtime) bfile_first_free: usize,
    pub(in crate::runtime) dirs: Vec<Option<DirHandle>>,
    pub(in crate::runtime) dir_first_free: usize,
    pub(in crate::runtime) program_args: Vec<Vec<u8>>,
    pub(in crate::runtime) executable_path: Option<Vec<u8>>,
    pub(in crate::runtime) arg_ref_array: Option<NodeId>,
    pub(in crate::runtime) boot_time_micro: i64,
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
    #[cfg(feature = "profile")]
    pub(in crate::runtime) profile: Option<EvalProfile>,
    pub(in crate::runtime) reduce_depth: usize,
    /// Green threads, indexed by slot; `None` is a reaped thread. Slot 0 is `main`.
    pub(in crate::runtime) threads: Vec<Option<ThreadControl>>,
    /// Number of live, non-reaped green threads.
    pub(in crate::runtime) live_thread_count: usize,
    /// Number of live threads with a pending async exception.
    pub(in crate::runtime) pending_async_count: usize,
    /// C-visible scheduler state per thread slot (`threadStatus` reports this).
    pub(in crate::runtime) thread_states: Vec<ThreadState>,
    /// Stable thread ids per slot, retained after a thread has been reaped.
    pub(in crate::runtime) thread_ids: Vec<i64>,
    /// Runnable thread slots in round-robin order.
    pub(in crate::runtime) run_queue: std::collections::VecDeque<usize>,
    /// MVar wait queues live outside `Node::MVar`; the node only stores the value.
    pub(in crate::runtime) mvar_waiters: HashMap<NodeId, MVarWaitQueues>,
    /// Absolute scheduler times, in microseconds since `scheduler_epoch`.
    pub(in crate::runtime) delay_wakeups: HashMap<usize, u128>,
    pub(in crate::runtime) scheduler_epoch: Instant,
    /// Slot of the thread currently being reduced.
    pub(in crate::runtime) current_thread: usize,
    /// Monotonic thread-id counter; `main` is 1 (matching the C runtime).
    pub(in crate::runtime) next_thread_id: i64,
    /// Set when the running thread should yield to the scheduler at the next step
    /// boundary (e.g. right after a `forkIO` that makes the program multi-threaded),
    /// so the reducer can leave an otherwise-unbounded single-thread slice.
    pub(in crate::runtime) reschedule_now: bool,
    /// Set when a nested reducer already installed the precise restart root for a
    /// scheduler yield (currently `catchr` preserving a handler around a sliced action).
    pub(in crate::runtime) preserve_thread_root_once: bool,
}
