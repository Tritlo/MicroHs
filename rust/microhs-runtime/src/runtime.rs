use std::cmp::Ordering;
use std::collections::{BTreeMap, HashMap, HashSet, VecDeque};
use std::fmt;
use std::mem::{MaybeUninit, size_of};
use std::time::Instant;

macro_rules! trace_invalid_bytes {
    ($program:expr, $($arg:tt)*) => {{
        if std::env::var_os("MHS_TRACE_INVALID_BYTES").is_some() {
            eprintln!("invalid bytes: reductions={}", $program.reductions);
            eprintln!($($arg)*);
        }
        EvalError::InvalidByteString
    }};
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct NodeId(pub u32);

impl NodeId {
    pub(crate) fn from_index(index: usize) -> Self {
        Self(u32::try_from(index).expect("node arena exceeded u32 ids"))
    }

    pub(crate) fn index(self) -> usize {
        self.0 as usize
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum Prim {
    Known(KnownPrim),
    Runtime(RuntimePrim),
}

impl Prim {
    pub fn from_name(name: &str) -> Option<Self> {
        KnownPrim::from_name(name)
            .map(Self::Known)
            .or_else(|| RuntimePrim::from_name(name).map(Self::Runtime))
    }

    pub fn known(&self) -> Option<KnownPrim> {
        match self {
            Self::Known(known) => Some(*known),
            Self::Runtime(_) => None,
        }
    }

    pub fn name(&self) -> &str {
        match self {
            Self::Known(known) => known.name(),
            Self::Runtime(runtime) => runtime.name(),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct RuntimePrim(u16);

impl RuntimePrim {
    fn from_name(name: &str) -> Option<Self> {
        RUNTIME_PRIM_NAMES
            .iter()
            .position(|candidate| *candidate == name)
            .map(|index| Self(index as u16))
    }

    fn name(self) -> &'static str {
        RUNTIME_PRIM_NAMES
            .get(self.0 as usize)
            .copied()
            .unwrap_or("<invalid-runtime-prim>")
    }
}

impl fmt::Display for Prim {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.name())
    }
}

impl PartialEq<&str> for Prim {
    fn eq(&self, other: &&str) -> bool {
        self.name() == *other
    }
}

impl PartialEq<&str> for &Prim {
    fn eq(&self, other: &&str) -> bool {
        self.name() == *other
    }
}

impl PartialEq<KnownPrim> for Prim {
    fn eq(&self, other: &KnownPrim) -> bool {
        self.known() == Some(*other)
    }
}

impl PartialEq<KnownPrim> for &Prim {
    fn eq(&self, other: &KnownPrim) -> bool {
        self.known() == Some(*other)
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum KnownPrim {
    A,
    B,
    BPrime,
    C,
    CPrime,
    CPrimeB,
    I,
    J,
    K,
    K2,
    K3,
    K4,
    KA,
    KK,
    L,
    O,
    P,
    R,
    S,
    SPrime,
    U,
    Y,
    Z,
    Tag(u8),
    Tuple(u8),
    Chr,
    Ord,
    Catch,
    CatchR,
    Dynsym,
    IsInt,
    Raise,
    Rnf,
    Seq,
    Thnum,
    IoAtomic,
    IoBind,
    IoGc,
    IoGetArgRef,
    IoGetMaskingState,
    IoLazyBind,
    IoNewMVar,
    IoPerformIo,
    IoPp,
    IoPrint,
    IoPutMVar,
    IoReadMVar,
    IoReturn,
    IoSerialize,
    IoDeserialize,
    IoSetMaskingState,
    IoStderr,
    IoStdin,
    IoStdout,
    IoStats,
    IoStrict,
    IoTakeMVar,
    IoThen,
    IoThid,
    IoThreadStatus,
    IoTryPutMVar,
    IoTryReadMVar,
    IoTryTakeMVar,
    IoYield,
}

impl KnownPrim {
    fn from_name(name: &str) -> Option<Self> {
        Some(match name {
            "A" => Self::A,
            "B" => Self::B,
            "B'" => Self::BPrime,
            "C" => Self::C,
            "C'" => Self::CPrime,
            "C'B" => Self::CPrimeB,
            "I" => Self::I,
            "J" => Self::J,
            "K" => Self::K,
            "K2" => Self::K2,
            "K3" => Self::K3,
            "K4" => Self::K4,
            "KA" => Self::KA,
            "KK" => Self::KK,
            "L" => Self::L,
            "O" => Self::O,
            "P" => Self::P,
            "R" => Self::R,
            "S" => Self::S,
            "S'" => Self::SPrime,
            "U" => Self::U,
            "Y" => Self::Y,
            "Z" => Self::Z,
            "chr" => Self::Chr,
            "ord" => Self::Ord,
            "catch" => Self::Catch,
            "catchr" => Self::CatchR,
            "dynsym" => Self::Dynsym,
            "isint" => Self::IsInt,
            "raise" => Self::Raise,
            "rnf" => Self::Rnf,
            "seq" => Self::Seq,
            "thnum" => Self::Thnum,
            "IO.atomic" => Self::IoAtomic,
            "IO.>>=" => Self::IoBind,
            "IO.gc" => Self::IoGc,
            "IO.getArgRef" => Self::IoGetArgRef,
            "IO.getmaskingstate" => Self::IoGetMaskingState,
            "IO.lazyBind" => Self::IoLazyBind,
            "IO.newmvar" => Self::IoNewMVar,
            "IO.performIO" => Self::IoPerformIo,
            "IO.pp" => Self::IoPp,
            "IO.print" => Self::IoPrint,
            "IO.putmvar" => Self::IoPutMVar,
            "IO.readmvar" => Self::IoReadMVar,
            "IO.return" => Self::IoReturn,
            "IO.serialize" => Self::IoSerialize,
            "IO.deserialize" => Self::IoDeserialize,
            "IO.setmaskingstate" => Self::IoSetMaskingState,
            "IO.stderr" => Self::IoStderr,
            "IO.stdin" => Self::IoStdin,
            "IO.stdout" => Self::IoStdout,
            "IO.stats" => Self::IoStats,
            "IO.strict" => Self::IoStrict,
            "IO.takemvar" => Self::IoTakeMVar,
            "IO.>>" => Self::IoThen,
            "IO.thid" => Self::IoThid,
            "IO.threadstatus" => Self::IoThreadStatus,
            "IO.tryputmvar" => Self::IoTryPutMVar,
            "IO.tryreadmvar" => Self::IoTryReadMVar,
            "IO.trytakemvar" => Self::IoTryTakeMVar,
            "IO.yield" => Self::IoYield,
            _ => {
                if let Some(tag) = tag_index(name) {
                    return Some(Self::Tag(tag as u8));
                }
                if let Some(fields) = tuple_fields(name) {
                    return Some(Self::Tuple(fields as u8));
                }
                return None;
            }
        })
    }

    fn name(self) -> &'static str {
        match self {
            Self::A => "A",
            Self::B => "B",
            Self::BPrime => "B'",
            Self::C => "C",
            Self::CPrime => "C'",
            Self::CPrimeB => "C'B",
            Self::I => "I",
            Self::J => "J",
            Self::K => "K",
            Self::K2 => "K2",
            Self::K3 => "K3",
            Self::K4 => "K4",
            Self::KA => "KA",
            Self::KK => "KK",
            Self::L => "L",
            Self::O => "O",
            Self::P => "P",
            Self::R => "R",
            Self::S => "S",
            Self::SPrime => "S'",
            Self::U => "U",
            Self::Y => "Y",
            Self::Z => "Z",
            Self::Tag(tag) => TAG_PRIM_NAMES
                .get(tag as usize)
                .copied()
                .unwrap_or("<invalid-tag>"),
            Self::Tuple(fields) => TUPLE_PRIM_NAMES
                .get(fields as usize)
                .copied()
                .filter(|name| !name.is_empty())
                .unwrap_or("<invalid-tuple>"),
            Self::Chr => "chr",
            Self::Ord => "ord",
            Self::Catch => "catch",
            Self::CatchR => "catchr",
            Self::Dynsym => "dynsym",
            Self::IsInt => "isint",
            Self::Raise => "raise",
            Self::Rnf => "rnf",
            Self::Seq => "seq",
            Self::Thnum => "thnum",
            Self::IoAtomic => "IO.atomic",
            Self::IoBind => "IO.>>=",
            Self::IoGc => "IO.gc",
            Self::IoGetArgRef => "IO.getArgRef",
            Self::IoGetMaskingState => "IO.getmaskingstate",
            Self::IoLazyBind => "IO.lazyBind",
            Self::IoNewMVar => "IO.newmvar",
            Self::IoPerformIo => "IO.performIO",
            Self::IoPp => "IO.pp",
            Self::IoPrint => "IO.print",
            Self::IoPutMVar => "IO.putmvar",
            Self::IoReadMVar => "IO.readmvar",
            Self::IoReturn => "IO.return",
            Self::IoSerialize => "IO.serialize",
            Self::IoDeserialize => "IO.deserialize",
            Self::IoSetMaskingState => "IO.setmaskingstate",
            Self::IoStderr => "IO.stderr",
            Self::IoStdin => "IO.stdin",
            Self::IoStdout => "IO.stdout",
            Self::IoStats => "IO.stats",
            Self::IoStrict => "IO.strict",
            Self::IoTakeMVar => "IO.takemvar",
            Self::IoThen => "IO.>>",
            Self::IoThid => "IO.thid",
            Self::IoThreadStatus => "IO.threadstatus",
            Self::IoTryPutMVar => "IO.tryputmvar",
            Self::IoTryReadMVar => "IO.tryreadmvar",
            Self::IoTryTakeMVar => "IO.trytakemvar",
            Self::IoYield => "IO.yield",
        }
    }
}

fn encode_known_prim(known: KnownPrim) -> u16 {
    match known {
        KnownPrim::A => 0,
        KnownPrim::B => 1,
        KnownPrim::BPrime => 2,
        KnownPrim::C => 3,
        KnownPrim::CPrime => 4,
        KnownPrim::CPrimeB => 5,
        KnownPrim::I => 6,
        KnownPrim::J => 7,
        KnownPrim::K => 8,
        KnownPrim::K2 => 9,
        KnownPrim::K3 => 10,
        KnownPrim::K4 => 11,
        KnownPrim::KA => 12,
        KnownPrim::KK => 13,
        KnownPrim::L => 14,
        KnownPrim::O => 15,
        KnownPrim::P => 16,
        KnownPrim::R => 17,
        KnownPrim::S => 18,
        KnownPrim::SPrime => 19,
        KnownPrim::U => 20,
        KnownPrim::Y => 21,
        KnownPrim::Z => 22,
        KnownPrim::Tag(tag) => 64 + u16::from(tag),
        KnownPrim::Tuple(fields) => 128 + u16::from(fields),
        KnownPrim::Chr => 23,
        KnownPrim::Ord => 24,
        KnownPrim::Catch => 25,
        KnownPrim::CatchR => 26,
        KnownPrim::Dynsym => 27,
        KnownPrim::IsInt => 28,
        KnownPrim::Raise => 29,
        KnownPrim::Rnf => 30,
        KnownPrim::Seq => 31,
        KnownPrim::Thnum => 32,
        KnownPrim::IoAtomic => 33,
        KnownPrim::IoBind => 34,
        KnownPrim::IoGc => 35,
        KnownPrim::IoGetArgRef => 36,
        KnownPrim::IoGetMaskingState => 37,
        KnownPrim::IoLazyBind => 38,
        KnownPrim::IoNewMVar => 39,
        KnownPrim::IoPerformIo => 40,
        KnownPrim::IoPp => 41,
        KnownPrim::IoPrint => 42,
        KnownPrim::IoPutMVar => 43,
        KnownPrim::IoReadMVar => 44,
        KnownPrim::IoReturn => 45,
        KnownPrim::IoSerialize => 46,
        KnownPrim::IoDeserialize => 61,
        KnownPrim::IoSetMaskingState => 47,
        KnownPrim::IoStderr => 48,
        KnownPrim::IoStdin => 49,
        KnownPrim::IoStdout => 50,
        KnownPrim::IoStats => 51,
        KnownPrim::IoStrict => 52,
        KnownPrim::IoTakeMVar => 53,
        KnownPrim::IoThen => 54,
        KnownPrim::IoThid => 55,
        KnownPrim::IoThreadStatus => 56,
        KnownPrim::IoTryPutMVar => 57,
        KnownPrim::IoTryReadMVar => 58,
        KnownPrim::IoTryTakeMVar => 59,
        KnownPrim::IoYield => 60,
    }
}

fn decode_known_prim(code: u16) -> KnownPrim {
    match code {
        0 => KnownPrim::A,
        1 => KnownPrim::B,
        2 => KnownPrim::BPrime,
        3 => KnownPrim::C,
        4 => KnownPrim::CPrime,
        5 => KnownPrim::CPrimeB,
        6 => KnownPrim::I,
        7 => KnownPrim::J,
        8 => KnownPrim::K,
        9 => KnownPrim::K2,
        10 => KnownPrim::K3,
        11 => KnownPrim::K4,
        12 => KnownPrim::KA,
        13 => KnownPrim::KK,
        14 => KnownPrim::L,
        15 => KnownPrim::O,
        16 => KnownPrim::P,
        17 => KnownPrim::R,
        18 => KnownPrim::S,
        19 => KnownPrim::SPrime,
        20 => KnownPrim::U,
        21 => KnownPrim::Y,
        22 => KnownPrim::Z,
        23 => KnownPrim::Chr,
        24 => KnownPrim::Ord,
        25 => KnownPrim::Catch,
        26 => KnownPrim::CatchR,
        27 => KnownPrim::Dynsym,
        28 => KnownPrim::IsInt,
        29 => KnownPrim::Raise,
        30 => KnownPrim::Rnf,
        31 => KnownPrim::Seq,
        32 => KnownPrim::Thnum,
        33 => KnownPrim::IoAtomic,
        34 => KnownPrim::IoBind,
        35 => KnownPrim::IoGc,
        36 => KnownPrim::IoGetArgRef,
        37 => KnownPrim::IoGetMaskingState,
        38 => KnownPrim::IoLazyBind,
        39 => KnownPrim::IoNewMVar,
        40 => KnownPrim::IoPerformIo,
        41 => KnownPrim::IoPp,
        42 => KnownPrim::IoPrint,
        43 => KnownPrim::IoPutMVar,
        44 => KnownPrim::IoReadMVar,
        45 => KnownPrim::IoReturn,
        46 => KnownPrim::IoSerialize,
        61 => KnownPrim::IoDeserialize,
        47 => KnownPrim::IoSetMaskingState,
        48 => KnownPrim::IoStderr,
        49 => KnownPrim::IoStdin,
        50 => KnownPrim::IoStdout,
        51 => KnownPrim::IoStats,
        52 => KnownPrim::IoStrict,
        53 => KnownPrim::IoTakeMVar,
        54 => KnownPrim::IoThen,
        55 => KnownPrim::IoThid,
        56 => KnownPrim::IoThreadStatus,
        57 => KnownPrim::IoTryPutMVar,
        58 => KnownPrim::IoTryReadMVar,
        59 => KnownPrim::IoTryTakeMVar,
        60 => KnownPrim::IoYield,
        64..=96 => KnownPrim::Tag((code - 64) as u8),
        128..=144 => KnownPrim::Tuple((code - 128) as u8),
        _ => unreachable!("invalid known prim code"),
    }
}

const TAG_PRIM_NAMES: [&str; 33] = [
    "TAG0", "TAG1", "TAG2", "TAG3", "TAG4", "TAG5", "TAG6", "TAG7", "TAG8", "TAG9", "TAG10",
    "TAG11", "TAG12", "TAG13", "TAG14", "TAG15", "TAG16", "TAG17", "TAG18", "TAG19", "TAG20",
    "TAG21", "TAG22", "TAG23", "TAG24", "TAG25", "TAG26", "TAG27", "TAG28", "TAG29", "TAG30",
    "TAG31", "TAG32",
];

const TUPLE_PRIM_NAMES: [&str; 17] = [
    "", "", "", "T3", "T4", "T5", "T6", "T7", "T8", "T9", "T10", "T11", "T12", "T13", "T14", "T15",
    "T16",
];

const RUNTIME_PRIM_NAMES: &[&str] = &[
    "+",
    "-",
    "*",
    "quot",
    "rem",
    "subtract",
    "u+",
    "u-",
    "u*",
    "uquot",
    "urem",
    "usubtract",
    "and",
    "or",
    "xor",
    "shl",
    "shr",
    "ashr",
    "==",
    "/=",
    "<",
    "<=",
    ">",
    ">=",
    "u<",
    "u<=",
    "u>",
    "u>=",
    "icmp",
    "ucmp",
    "neg",
    "uneg",
    "inv",
    "popcount",
    "clz",
    "ctz",
    "I+",
    "I-",
    "I*",
    "Iquot",
    "Irem",
    "Isubtract",
    "Iu+",
    "Iu-",
    "Iu*",
    "Iuquot",
    "Iurem",
    "Iusubtract",
    "Iand",
    "Ior",
    "Ixor",
    "Ishl",
    "Ishr",
    "Iashr",
    "I==",
    "I/=",
    "I<",
    "I<=",
    "I>",
    "I>=",
    "Iu<",
    "Iu<=",
    "Iu>",
    "Iu>=",
    "Iicmp",
    "Iucmp",
    "Ineg",
    "Iuneg",
    "Iinv",
    "Ipopcount",
    "Iclz",
    "Ictz",
    "d+",
    "d-",
    "d*",
    "d/",
    "d==",
    "d/=",
    "d<",
    "d<=",
    "d>",
    "d>=",
    "dneg",
    "f+",
    "f-",
    "f*",
    "f/",
    "f==",
    "f/=",
    "f<",
    "f<=",
    "f>",
    "f>=",
    "fneg",
    "itoI",
    "utoU",
    "Itoi",
    "Utou",
    "itod",
    "utod",
    "Itod",
    "dtoi",
    "itof",
    "utof",
    "Itof",
    "ftoi",
    "dtof",
    "ftod",
    "toDbl",
    "fromDbl",
    "toFlt",
    "fromFlt",
    "toInt",
    "toPtr",
    "toFunPtr",
    "fp+",
    "fp2bs",
    "fpnew",
    "fpfin",
    "bs2fp",
    "fp2p",
    "A.alloc",
    "A.read",
    "A.write",
    "A.trunc",
    "A.==",
    "A.copy",
    "A.size",
    "SPnew",
    "SPderef",
    "SPfree",
    "Wknewfin",
    "Wknew",
    "Wkderef",
    "Wkfinal",
    "packCString",
    "packCStringLen",
    "bsgrab",
    "bsgrablen",
    "bsnew",
    "bsread",
    "bswrite",
    "bsfreeze",
    "bsappbyte",
    "bsappchar",
    "bs++",
    "bs++.",
    "bs==",
    "bs/=",
    "bs<",
    "bs<=",
    "bs>",
    "bs>=",
    "bscmp",
    "bsreplicate",
    "bsindex",
    "bssubstr",
    "bslength",
    "headUTF8",
    "tailUTF8",
    "bsunpack",
    "fromUTF8",
    "IO.deserialize",
    "IO.fork",
    "IO.throwto",
    "IO.threaddelay",
    "IO.waitrdfd",
    "IO.waitwrfd",
];

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
struct Cell {
    word0: u64,
    word1: u64,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum CellTag {
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

const CELL_TAG_BITS: u64 = 0xff;
const CELL_PAYLOAD_SHIFT: u64 = 8;
const CELL_NONE_ID: u64 = u64::MAX;

impl CellTag {
    fn from_bits(bits: u64) -> Self {
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

    fn bits(self) -> u64 {
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
    fn tag_bits(self) -> u64 {
        self.word0 & CELL_TAG_BITS
    }

    #[inline]
    fn has_tag(self, tag: CellTag) -> bool {
        self.tag_bits() == tag.bits()
    }

    fn from_node(node: Node, cold_nodes: &mut Vec<Option<Node>>) -> Self {
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

    fn to_node(self, cold_nodes: &[Option<Node>]) -> Node {
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

    fn tag(self) -> CellTag {
        CellTag::from_bits(self.tag_bits())
    }

    fn app(fun: NodeId, arg: NodeId) -> Self {
        Self {
            word0: (u64::from(fun.0) << CELL_PAYLOAD_SHIFT) | CellTag::App.bits(),
            word1: u64::from(arg.0),
        }
    }

    fn indir(target: Option<NodeId>) -> Self {
        Self {
            word0: CellTag::Indir.bits(),
            word1: pack_option_id(target),
        }
    }

    fn free(next: Option<NodeId>) -> Self {
        Self {
            word0: CellTag::Free.bits(),
            word1: pack_option_id(next),
        }
    }

    fn known_prim(known: KnownPrim) -> Self {
        Self {
            word0: CellTag::KnownPrim.bits(),
            word1: u64::from(encode_known_prim(known)),
        }
    }

    fn runtime_prim(runtime: RuntimePrim) -> Self {
        Self {
            word0: CellTag::RuntimePrim.bits(),
            word1: u64::from(runtime.0),
        }
    }

    fn int(value: i64) -> Self {
        Self {
            word0: CellTag::Int.bits(),
            word1: value as u64,
        }
    }

    fn int64(value: i64) -> Self {
        Self {
            word0: CellTag::Int64.bits(),
            word1: value as u64,
        }
    }

    fn float64(value: f64) -> Self {
        Self {
            word0: CellTag::Float64.bits(),
            word1: value.to_bits(),
        }
    }

    fn float32(value: f32) -> Self {
        Self {
            word0: CellTag::Float32.bits(),
            word1: u64::from(value.to_bits()),
        }
    }

    fn thread_id(value: i64) -> Self {
        Self {
            word0: CellTag::ThreadId.bits(),
            word1: value as u64,
        }
    }

    fn ptr(value: i64) -> Self {
        Self {
            word0: CellTag::Ptr.bits(),
            word1: value as u64,
        }
    }

    fn raw_fun_ptr(value: i64) -> Self {
        Self {
            word0: CellTag::RawFunPtr.bits(),
            word1: value as u64,
        }
    }

    fn cold(index: usize) -> Self {
        Self {
            word0: CellTag::Cold.bits(),
            word1: u64::try_from(index).expect("cold node table exceeded u64"),
        }
    }

    fn id_payload(self) -> NodeId {
        NodeId((self.word0 >> CELL_PAYLOAD_SHIFT) as u32)
    }

    fn id_word1(self) -> NodeId {
        NodeId(self.word1 as u32)
    }

    fn option_id_word1(self) -> Option<NodeId> {
        unpack_option_id(self.word1)
    }

    fn app_fields(self) -> Option<(NodeId, NodeId)> {
        self.has_tag(CellTag::App)
            .then(|| (self.id_payload(), self.id_word1()))
    }

    fn prim(self) -> Option<Prim> {
        match self.tag_bits() {
            3 => Some(Prim::Known(decode_known_prim(self.word1 as u16))),
            4 => Some(Prim::Runtime(RuntimePrim(self.word1 as u16))),
            _ => None,
        }
    }

    fn int_value(self) -> Option<i64> {
        self.has_tag(CellTag::Int).then_some(self.word1 as i64)
    }

    fn int64_value(self) -> Option<i64> {
        self.has_tag(CellTag::Int64).then_some(self.word1 as i64)
    }

    fn thread_id_value(self) -> Option<i64> {
        self.has_tag(CellTag::ThreadId).then_some(self.word1 as i64)
    }

    fn ptr_value(self) -> Option<i64> {
        self.has_tag(CellTag::Ptr).then_some(self.word1 as i64)
    }

    fn raw_fun_ptr_value(self) -> Option<i64> {
        self.has_tag(CellTag::RawFunPtr)
            .then_some(self.word1 as i64)
    }

    fn float64_value(self) -> Option<f64> {
        self.has_tag(CellTag::Float64)
            .then_some(f64::from_bits(self.word1))
    }

    fn float32_value(self) -> Option<f32> {
        self.has_tag(CellTag::Float32)
            .then_some(f32::from_bits(self.word1 as u32))
    }

    fn cold_index(self) -> Option<usize> {
        self.has_tag(CellTag::Cold).then_some(self.word1 as usize)
    }
}

fn pack_option_id(id: Option<NodeId>) -> u64 {
    id.map_or(CELL_NONE_ID, |id| u64::from(id.0))
}

fn unpack_option_id(word: u64) -> Option<NodeId> {
    (word != CELL_NONE_ID).then_some(NodeId(word as u32))
}

#[derive(Default)]
struct SerializationLabels {
    shared: HashSet<NodeId>,
    printed: HashSet<NodeId>,
}

#[derive(Clone, Debug)]
pub struct ForeignPtrNode {
    pub(crate) bytes: Option<Vec<u8>>,
    pub(crate) offset: usize,
    pub(crate) ptr: i64,
    pub(crate) finalizer: Option<usize>,
}

#[derive(Clone, Debug)]
enum ForeignFinalizer {
    Free,
    CloseB,
    RawZero,
}

#[derive(Clone, Debug)]
struct ForeignFinalizerState {
    arg: i64,
    finalizer: Option<ForeignFinalizer>,
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
    fn visible(&self) -> &[u8] {
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

    fn bytes_view(base: NodeId, offset: usize, len: usize) -> Self {
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
enum StdHandle {
    Stdin,
    Stdout,
    Stderr,
}

const ALLOCATION_PTR_BASE: i64 = -(1_i64 << 62);
const ALLOCATION_PTR_STRIDE: i64 = 1_i64 << 32;
const NODE_PTR_STRIDE: i64 = 1_i64 << 32;
const BFILE_PTR_BASE: i64 = i64::MIN + (1_i64 << 32);
const FORCE_REDUCTION_LIMIT: usize = usize::MAX;
const RTS_EXN_DIVIDE_BY_ZERO: i64 = 4;
const RTS_EXN_OVERFLOW: i64 = 7;
const MASK_INTERRUPTIBLE: i64 = 1;
const BFILE_PTR_STRIDE: i64 = 1_i64 << 32;
const DIR_PTR_BASE: i64 = i64::MIN + (1_i64 << 61);
const DIR_PTR_STRIDE: i64 = 1_i64 << 32;
const INLINE_SPINE: usize = 16;
const FALLBACK_PRIM_ARG_PREFIX: usize = 4;
const SMALL_INT_MIN: i64 = -10;
const SMALL_INT_MAX: i64 = 255;
const SMALL_INT_COUNT: usize = (SMALL_INT_MAX - SMALL_INT_MIN + 1) as usize;
const IGNORED_IO_SHORTCUT_RECURSION_LIMIT: usize = 256;
const UTF8_ASCII_REFILL: usize = 1024;
const GC_NODE_INTERVAL: usize = 32 * 1024 * 1024;

#[derive(Clone, Debug)]
struct BFile {
    kind: BFileKind,
    readable: bool,
    writable: bool,
}

#[derive(Clone, Debug)]
enum BFileKind {
    Memory {
        bytes: Vec<u8>,
        pos: usize,
    },
    #[cfg(not(target_arch = "wasm32"))]
    NativeFile {
        file: NativeFileHandle,
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

#[cfg(not(target_arch = "wasm32"))]
type NativeFileHandle = std::rc::Rc<std::cell::RefCell<std::fs::File>>;

#[derive(Clone, Debug)]
struct DirHandle {
    entries: Vec<Vec<u8>>,
    pos: usize,
}

#[cfg(not(target_arch = "wasm32"))]
#[derive(Clone, Copy, Debug)]
struct NativeFileMode {
    readable: bool,
    writable: bool,
    append: bool,
    truncate: bool,
    create: bool,
}

#[derive(Clone, Copy, Debug)]
struct HostIntResult {
    value: i64,
    errno: Option<i32>,
}

#[cfg_attr(target_arch = "wasm32", allow(dead_code))]
impl HostIntResult {
    fn ok(value: i64) -> Self {
        Self { value, errno: None }
    }

    fn err(errno: i32) -> Self {
        Self {
            value: -1,
            errno: Some(errno),
        }
    }

    fn os_err(errno: Option<i32>) -> Self {
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

#[cfg_attr(not(target_arch = "wasm32"), allow(dead_code))]
enum JsArg {
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
    nodes: Vec<Cell>,
    cold_nodes: Vec<Option<Node>>,
    root: NodeId,
    labels: HashMap<usize, NodeId>,
    node_pointers: Vec<NodeId>,
    node_pointer_slots: HashMap<NodeId, usize>,
    free_head: Option<NodeId>,
    free_nodes: usize,
    gc_node_interval: usize,
    gc_allocations_since_collect: usize,
    gc_last_allocations_since_collect: usize,
    gc_collections: usize,
    gc_freed_nodes_total: usize,
    gc_last_live_nodes: usize,
    gc_last_free_nodes: usize,
    gc_high_water_nodes: usize,
    gc_last_pause_nanos: u128,
    gc_total_pause_nanos: u128,
    #[cfg(feature = "gc-phase-profile")]
    gc_last_mark_nanos: u128,
    #[cfg(feature = "gc-phase-profile")]
    gc_total_mark_nanos: u128,
    #[cfg(feature = "gc-phase-profile")]
    gc_last_sweep_nanos: u128,
    #[cfg(feature = "gc-phase-profile")]
    gc_total_sweep_nanos: u128,
    #[cfg(feature = "gc-phase-profile")]
    gc_red_i_opportunities: usize,
    #[cfg(feature = "gc-phase-profile")]
    gc_red_k_opportunities: usize,
    #[cfg(feature = "gc-phase-profile")]
    gc_red_a_opportunities: usize,
    #[cfg(feature = "gc-phase-profile")]
    gc_red_bi_opportunities: usize,
    #[cfg(feature = "gc-phase-profile")]
    gc_red_bxi_opportunities: usize,
    #[cfg(feature = "gc-phase-profile")]
    gc_red_ccbi_opportunities: usize,
    #[cfg(feature = "gc-phase-profile")]
    gc_red_cc_opportunities: usize,
    #[cfg(feature = "gc-phase-profile")]
    gc_red_cci_opportunities: usize,
    #[cfg(feature = "gc-phase-profile")]
    gc_red_ccbbcp_opportunities: usize,
    #[cfg(feature = "gc-phase-profile")]
    gc_red_flip_opportunities: usize,
    #[cfg(feature = "gc-phase-profile")]
    gc_young_profile_allocated_slots: Vec<NodeId>,
    #[cfg(feature = "gc-phase-profile")]
    gc_young_profile_last_slots: usize,
    #[cfg(feature = "gc-phase-profile")]
    gc_young_profile_last_live: usize,
    #[cfg(feature = "gc-phase-profile")]
    gc_young_profile_last_dead: usize,
    #[cfg(feature = "gc-phase-profile")]
    gc_young_profile_last_old_to_young_sources: usize,
    #[cfg(feature = "gc-phase-profile")]
    gc_young_profile_last_old_to_young_edges: usize,
    #[cfg(feature = "gc-phase-profile")]
    gc_young_profile_total_slots: usize,
    #[cfg(feature = "gc-phase-profile")]
    gc_young_profile_total_live: usize,
    #[cfg(feature = "gc-phase-profile")]
    gc_young_profile_total_dead: usize,
    #[cfg(feature = "gc-phase-profile")]
    gc_young_profile_total_old_to_young_sources: usize,
    #[cfg(feature = "gc-phase-profile")]
    gc_young_profile_total_old_to_young_edges: usize,
    gc_marked: Vec<bool>,
    gc_mark_work: Vec<NodeId>,
    gc_foreign_finalizer_marked: Vec<bool>,
    gc_events: Vec<GcEventStats>,
    stable_ptrs: Vec<Option<NodeId>>,
    weak_nodes: Vec<NodeId>,
    pending_weak_finalizers: Vec<NodeId>,
    foreign_finalizers: Vec<Option<ForeignFinalizerState>>,
    foreign_finalizer_free: Vec<usize>,
    allocations: Vec<Option<Vec<u8>>>,
    bfiles: Vec<Option<BFile>>,
    dirs: Vec<Option<DirHandle>>,
    program_args: Vec<Vec<u8>>,
    executable_path: Option<Vec<u8>>,
    arg_ref_array: Option<NodeId>,
    errno_value: i32,
    errno_ptr: Option<i64>,
    masking_state: i64,
    reductions: usize,
    js_program_handle: Option<u32>,
    js_wrapper_tags: Vec<String>,
    prim_cache: PrimCache,
    compound_cache: CompoundCache,
    small_ints: [Option<NodeId>; SMALL_INT_COUNT],
    world: Option<NodeId>,
    profile: Option<EvalProfile>,
    trace_expected_bytes: bool,
    reduce_depth: usize,
}

#[derive(Clone, Debug, Default)]
pub struct GcStats {
    pub collections: usize,
    pub freed_nodes_total: usize,
    pub last_live_nodes: usize,
    pub last_free_nodes: usize,
    pub high_water_nodes: usize,
    pub current_nodes: usize,
    pub current_free_nodes: usize,
    pub last_pause_nanos: u128,
    pub total_pause_nanos: u128,
    #[cfg(feature = "gc-phase-profile")]
    pub last_mark_nanos: u128,
    #[cfg(feature = "gc-phase-profile")]
    pub total_mark_nanos: u128,
    #[cfg(feature = "gc-phase-profile")]
    pub last_sweep_nanos: u128,
    #[cfg(feature = "gc-phase-profile")]
    pub total_sweep_nanos: u128,
    #[cfg(feature = "gc-phase-profile")]
    pub red_i_opportunities: usize,
    #[cfg(feature = "gc-phase-profile")]
    pub red_k_opportunities: usize,
    #[cfg(feature = "gc-phase-profile")]
    pub red_a_opportunities: usize,
    #[cfg(feature = "gc-phase-profile")]
    pub red_bi_opportunities: usize,
    #[cfg(feature = "gc-phase-profile")]
    pub red_bxi_opportunities: usize,
    #[cfg(feature = "gc-phase-profile")]
    pub red_ccbi_opportunities: usize,
    #[cfg(feature = "gc-phase-profile")]
    pub red_cc_opportunities: usize,
    #[cfg(feature = "gc-phase-profile")]
    pub red_cci_opportunities: usize,
    #[cfg(feature = "gc-phase-profile")]
    pub red_ccbbcp_opportunities: usize,
    #[cfg(feature = "gc-phase-profile")]
    pub red_flip_opportunities: usize,
    #[cfg(feature = "gc-phase-profile")]
    pub young_profile_last_slots: usize,
    #[cfg(feature = "gc-phase-profile")]
    pub young_profile_last_live: usize,
    #[cfg(feature = "gc-phase-profile")]
    pub young_profile_last_dead: usize,
    #[cfg(feature = "gc-phase-profile")]
    pub young_profile_last_old_to_young_sources: usize,
    #[cfg(feature = "gc-phase-profile")]
    pub young_profile_last_old_to_young_edges: usize,
    #[cfg(feature = "gc-phase-profile")]
    pub young_profile_total_slots: usize,
    #[cfg(feature = "gc-phase-profile")]
    pub young_profile_total_live: usize,
    #[cfg(feature = "gc-phase-profile")]
    pub young_profile_total_dead: usize,
    #[cfg(feature = "gc-phase-profile")]
    pub young_profile_total_old_to_young_sources: usize,
    #[cfg(feature = "gc-phase-profile")]
    pub young_profile_total_old_to_young_edges: usize,
    pub last_allocations_since_collect: usize,
    pub current_allocations_since_collect: usize,
    pub events: Vec<GcEventStats>,
}

#[derive(Clone, Copy, Debug, Default)]
pub struct GcEventStats {
    pub collection: usize,
    pub pause_nanos: u128,
    #[cfg(feature = "gc-phase-profile")]
    pub mark_nanos: u128,
    #[cfg(feature = "gc-phase-profile")]
    pub sweep_nanos: u128,
    pub live_nodes: usize,
    pub free_nodes: usize,
    pub arena_nodes: usize,
    pub freed_nodes: usize,
    pub allocations_since_collect: usize,
    #[cfg(feature = "gc-phase-profile")]
    pub young_profile_slots: usize,
    #[cfg(feature = "gc-phase-profile")]
    pub young_profile_live: usize,
    #[cfg(feature = "gc-phase-profile")]
    pub young_profile_dead: usize,
    #[cfg(feature = "gc-phase-profile")]
    pub young_profile_old_to_young_sources: usize,
    #[cfg(feature = "gc-phase-profile")]
    pub young_profile_old_to_young_edges: usize,
}

#[derive(Clone, Debug, Default)]
pub struct EvalProfile {
    pub step_attempts: usize,
    pub successful_steps: usize,
    pub reductions: usize,
    pub app_allocations: usize,
    pub arg_materializations: usize,
    pub arg_materialized_nodes: usize,
    pub spine_rewrites: usize,
    pub spine_rewrite_extra_args: usize,
    pub app_rewrites: usize,
    pub app_rewrite_extra_args: usize,
    pub stack_rewrites: usize,
    pub stack_rewrite_apps: usize,
    pub stack_rewrite_indirections: usize,
    pub stack_app_updates: usize,
    pub stack_app_update_apps: usize,
    pub stack_app_update_allocations: usize,
    pub stack_rethreads: usize,
    pub stack_rethread_apps: usize,
    pub stack_descent_pushes: usize,
    pub stack_arg_reads: usize,
    pub stack_arg_batches: usize,
    pub stack_loop_iterations: usize,
    pub stack_ready_checks: usize,
    pub stack_ready_successes: usize,
    pub stack_eval_step_calls: usize,
    pub stack_step_reduced: usize,
    pub stack_step_force: usize,
    pub stack_step_whnf: usize,
    pub stack_step_fallback: usize,
    pub stack_gc_check_nanos: u128,
    pub stack_resolve_nanos: u128,
    pub stack_ready_frame_nanos: u128,
    pub stack_descent_nanos: u128,
    pub stack_eval_step_nanos: u128,
    pub stack_whnf_finish_nanos: u128,
    pub stack_arg_read_nanos: u128,
    pub stack_app_alloc_nanos: u128,
    pub stack_apply_rewrite_nanos: u128,
    pub stack_apply_app_nanos: u128,
    pub stack_force_frame_nanos: u128,
    pub stack_inner_descent_nanos: u128,
    #[cfg(feature = "eval-phase-profile")]
    pub app_alloc_reused: usize,
    #[cfg(feature = "eval-phase-profile")]
    pub app_alloc_fresh: usize,
    #[cfg(feature = "eval-phase-profile")]
    pub app_alloc_free_pop_nanos: u128,
    #[cfg(feature = "eval-phase-profile")]
    pub app_alloc_reused_write_nanos: u128,
    #[cfg(feature = "eval-phase-profile")]
    pub app_alloc_fresh_push_nanos: u128,
    #[cfg(feature = "eval-phase-profile")]
    pub profile_step_nanos: u128,
    #[cfg(feature = "eval-phase-profile")]
    pub profile_reduction_nanos: u128,
    #[cfg(feature = "eval-phase-profile")]
    pub profile_stack_head_time_nanos: u128,
    #[cfg(feature = "eval-phase-profile")]
    pub profile_app_alloc_bookkeeping_nanos: u128,
    #[cfg(feature = "eval-phase-profile")]
    pub stack_arg_read_head_nanos: HashMap<String, u128>,
    #[cfg(feature = "eval-phase-profile")]
    pub stack_app_alloc_site_nanos: HashMap<String, u128>,
    #[cfg(feature = "eval-phase-profile")]
    pub stack_apply_rewrite_head_nanos: HashMap<String, u128>,
    #[cfg(feature = "eval-phase-profile")]
    pub stack_apply_app_head_nanos: HashMap<String, u128>,
    #[cfg(feature = "eval-phase-profile")]
    pub stack_force_frame_head_nanos: HashMap<String, u128>,
    #[cfg(feature = "eval-phase-profile")]
    pub stack_inner_descent_head_nanos: HashMap<String, u128>,
    pub persistent_forces: usize,
    pub persistent_fallbacks: usize,
    pub fallback_eval_loop_steps: usize,
    pub strict_redex_snapshots: usize,
    pub strict_redex_snapshot_apps: usize,
    pub remaining_app_scans: usize,
    pub remaining_app_scan_apps: usize,
    pub eval_frame_pushes: usize,
    pub small_int_cache_hits: usize,
    pub small_int_cache_misses: usize,
    pub non_small_int_allocations: usize,
    pub heap_spines: usize,
    pub max_spine_arity: usize,
    pub resolve_calls: usize,
    pub resolve_indirections: usize,
    pub max_resolve_chain: usize,
    pub head_attempts: HashMap<String, usize>,
    pub head_reductions: HashMap<String, usize>,
    pub spine_arity: BTreeMap<usize, usize>,
    pub resolve_chain: BTreeMap<usize, usize>,
    pub shortcut_hits: HashMap<String, usize>,
    pub primitive_dispatch_probes: HashMap<String, usize>,
    pub primitive_dispatch_hits: HashMap<String, usize>,
    pub node_allocations: HashMap<String, usize>,
    pub app_allocation_sites: HashMap<String, usize>,
    pub eval_frame_push_kinds: HashMap<String, usize>,
    pub stack_fallback_heads: HashMap<String, usize>,
    pub stack_eval_step_head_nanos: HashMap<String, u128>,
}

impl EvalProfile {
    pub fn top_head_attempts(&self, limit: usize) -> Vec<(&str, usize)> {
        sorted_profile_counts(&self.head_attempts, limit)
    }

    pub fn top_head_reductions(&self, limit: usize) -> Vec<(&str, usize)> {
        sorted_profile_counts(&self.head_reductions, limit)
    }

    pub fn top_shortcut_hits(&self, limit: usize) -> Vec<(&str, usize)> {
        sorted_profile_counts(&self.shortcut_hits, limit)
    }

    pub fn top_primitive_dispatch_probes(&self, limit: usize) -> Vec<(&str, usize)> {
        sorted_profile_counts(&self.primitive_dispatch_probes, limit)
    }

    pub fn top_primitive_dispatch_hits(&self, limit: usize) -> Vec<(&str, usize)> {
        sorted_profile_counts(&self.primitive_dispatch_hits, limit)
    }

    pub fn top_node_allocations(&self, limit: usize) -> Vec<(&str, usize)> {
        sorted_profile_counts(&self.node_allocations, limit)
    }

    pub fn top_app_allocation_sites(&self, limit: usize) -> Vec<(&str, usize)> {
        sorted_profile_counts(&self.app_allocation_sites, limit)
    }

    pub fn top_eval_frame_push_kinds(&self, limit: usize) -> Vec<(&str, usize)> {
        sorted_profile_counts(&self.eval_frame_push_kinds, limit)
    }

    pub fn top_stack_fallback_heads(&self, limit: usize) -> Vec<(&str, usize)> {
        sorted_profile_counts(&self.stack_fallback_heads, limit)
    }

    pub fn top_stack_eval_step_head_times(&self, limit: usize) -> Vec<(&str, u128)> {
        sorted_profile_times(&self.stack_eval_step_head_nanos, limit)
    }

    #[cfg(feature = "eval-phase-profile")]
    pub fn top_stack_arg_read_head_times(&self, limit: usize) -> Vec<(&str, u128)> {
        sorted_profile_times(&self.stack_arg_read_head_nanos, limit)
    }

    #[cfg(feature = "eval-phase-profile")]
    pub fn top_stack_app_alloc_site_times(&self, limit: usize) -> Vec<(&str, u128)> {
        sorted_profile_times(&self.stack_app_alloc_site_nanos, limit)
    }

    #[cfg(feature = "eval-phase-profile")]
    pub fn top_stack_apply_rewrite_head_times(&self, limit: usize) -> Vec<(&str, u128)> {
        sorted_profile_times(&self.stack_apply_rewrite_head_nanos, limit)
    }

    #[cfg(feature = "eval-phase-profile")]
    pub fn top_stack_apply_app_head_times(&self, limit: usize) -> Vec<(&str, u128)> {
        sorted_profile_times(&self.stack_apply_app_head_nanos, limit)
    }

    #[cfg(feature = "eval-phase-profile")]
    pub fn top_stack_force_frame_head_times(&self, limit: usize) -> Vec<(&str, u128)> {
        sorted_profile_times(&self.stack_force_frame_head_nanos, limit)
    }

    #[cfg(feature = "eval-phase-profile")]
    pub fn top_stack_inner_descent_head_times(&self, limit: usize) -> Vec<(&str, u128)> {
        sorted_profile_times(&self.stack_inner_descent_head_nanos, limit)
    }
}

fn sorted_profile_counts(map: &HashMap<String, usize>, limit: usize) -> Vec<(&str, usize)> {
    let mut counts: Vec<_> = map
        .iter()
        .map(|(key, value)| (key.as_str(), *value))
        .collect();
    counts.sort_unstable_by(|left, right| right.1.cmp(&left.1).then_with(|| left.0.cmp(right.0)));
    counts.truncate(limit);
    counts
}

fn sorted_profile_times(map: &HashMap<String, u128>, limit: usize) -> Vec<(&str, u128)> {
    let mut counts: Vec<_> = map
        .iter()
        .map(|(key, value)| (key.as_str(), *value))
        .collect();
    counts.sort_unstable_by(|left, right| right.1.cmp(&left.1).then_with(|| left.0.cmp(right.0)));
    counts.truncate(limit);
    counts
}

fn node_allocation_key(node: &Node) -> &'static str {
    match node {
        Node::App(_, _) => "App",
        Node::Indir(_) => "Indir",
        Node::Free(_) => "Free",
        Node::Prim(_) => "Prim",
        Node::Int(_) => "Int",
        Node::Int64(_) => "Int64",
        Node::Float64(_) => "Float64",
        Node::Float32(_) => "Float32",
        Node::ThreadId(_) => "ThreadId",
        Node::Ptr(_) => "Ptr",
        Node::RawFunPtr(_) => "RawFunPtr",
        Node::ForeignPtr(_) => "ForeignPtr",
        Node::Weak(_) => "Weak",
        Node::MVar(_) => "MVar",
        Node::BigInt(_) => "BigInt",
        Node::Bytes(_) => "Bytes",
        Node::BytesView(_) => "BytesView",
        Node::MutableBytes(_) => "MutableBytes",
        Node::Array(_) => "Array",
        Node::Ffi(_) => "Ffi",
        Node::JsCall(_) => "JsCall",
        Node::JsWrap { .. } => "JsWrap",
        Node::FunPtr(_) => "FunPtr",
        Node::Tick(_) => "Tick",
    }
}

fn serialization_shareable_node(node: &Node) -> bool {
    matches!(
        node,
        Node::App(_, _)
            | Node::ForeignPtr(_)
            | Node::BigInt(_)
            | Node::Bytes(_)
            | Node::MutableBytes(_)
            | Node::Array(_)
    )
}

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

impl Program {
    pub fn new(nodes: Vec<Node>, root: NodeId, labels: HashMap<usize, NodeId>) -> Self {
        let gc_node_interval = std::env::var("MHS_GC_NODE_INTERVAL")
            .ok()
            .and_then(|value| value.parse::<usize>().ok())
            .unwrap_or(GC_NODE_INTERVAL);
        let high_water_nodes = nodes.len();
        let mut cold_nodes = Vec::new();
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
                matches!(
                    cold_nodes.get(cold).and_then(Option::as_ref),
                    Some(Node::Weak(_))
                )
                .then(|| NodeId::from_index(index))
            })
            .collect();
        for (index, node) in nodes.iter().enumerate() {
            if let Some(value) = node.int_value() {
                if let Some(slot) = small_int_index(value) {
                    small_ints[slot].get_or_insert(NodeId::from_index(index));
                }
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
            #[cfg(feature = "gc-phase-profile")]
            gc_red_i_opportunities: 0,
            #[cfg(feature = "gc-phase-profile")]
            gc_red_k_opportunities: 0,
            #[cfg(feature = "gc-phase-profile")]
            gc_red_a_opportunities: 0,
            #[cfg(feature = "gc-phase-profile")]
            gc_red_bi_opportunities: 0,
            #[cfg(feature = "gc-phase-profile")]
            gc_red_bxi_opportunities: 0,
            #[cfg(feature = "gc-phase-profile")]
            gc_red_ccbi_opportunities: 0,
            #[cfg(feature = "gc-phase-profile")]
            gc_red_cc_opportunities: 0,
            #[cfg(feature = "gc-phase-profile")]
            gc_red_cci_opportunities: 0,
            #[cfg(feature = "gc-phase-profile")]
            gc_red_ccbbcp_opportunities: 0,
            #[cfg(feature = "gc-phase-profile")]
            gc_red_flip_opportunities: 0,
            #[cfg(feature = "gc-phase-profile")]
            gc_young_profile_allocated_slots: Vec::new(),
            #[cfg(feature = "gc-phase-profile")]
            gc_young_profile_last_slots: 0,
            #[cfg(feature = "gc-phase-profile")]
            gc_young_profile_last_live: 0,
            #[cfg(feature = "gc-phase-profile")]
            gc_young_profile_last_dead: 0,
            #[cfg(feature = "gc-phase-profile")]
            gc_young_profile_last_old_to_young_sources: 0,
            #[cfg(feature = "gc-phase-profile")]
            gc_young_profile_last_old_to_young_edges: 0,
            #[cfg(feature = "gc-phase-profile")]
            gc_young_profile_total_slots: 0,
            #[cfg(feature = "gc-phase-profile")]
            gc_young_profile_total_live: 0,
            #[cfg(feature = "gc-phase-profile")]
            gc_young_profile_total_dead: 0,
            #[cfg(feature = "gc-phase-profile")]
            gc_young_profile_total_old_to_young_sources: 0,
            #[cfg(feature = "gc-phase-profile")]
            gc_young_profile_total_old_to_young_edges: 0,
            gc_marked: Vec::new(),
            gc_mark_work: Vec::new(),
            gc_foreign_finalizer_marked: Vec::new(),
            gc_events: Vec::new(),
            stable_ptrs: vec![None],
            weak_nodes,
            pending_weak_finalizers: Vec::new(),
            foreign_finalizers: Vec::new(),
            foreign_finalizer_free: Vec::new(),
            allocations: Vec::new(),
            bfiles: Vec::new(),
            dirs: Vec::new(),
            program_args: Vec::new(),
            executable_path: None,
            arg_ref_array: None,
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
            profile: None,
            trace_expected_bytes: std::env::var_os("MHS_TRACE_EXPECTED_BYTES").is_some(),
            reduce_depth: 0,
        }
    }

    pub fn root(&self) -> NodeId {
        self.root
    }

    pub fn node_count(&self) -> usize {
        self.nodes.len()
    }

    pub fn node_for_debug(&self, id: NodeId) -> Node {
        self.nodes[id.index()].to_node(&self.cold_nodes)
    }

    pub fn nodes(&self) -> Vec<Node> {
        self.nodes
            .iter()
            .copied()
            .map(|cell| cell.to_node(&self.cold_nodes))
            .collect()
    }

    fn cell(&self, id: NodeId) -> Cell {
        self.nodes[id.index()]
    }

    fn cell_at(&self, index: usize) -> Cell {
        self.nodes[index]
    }

    #[inline]
    fn app_fun(&self, id: NodeId) -> Option<NodeId> {
        let word0 = self.nodes[id.index()].word0;
        ((word0 & CELL_TAG_BITS) == CellTag::App.bits())
            .then(|| NodeId((word0 >> CELL_PAYLOAD_SHIFT) as u32))
    }

    fn set_cell_at(&mut self, index: usize, cell: Cell) {
        self.drop_cold_payload(index);
        self.nodes[index] = cell;
    }

    fn set_app_cell_at(&mut self, index: usize, cell: Cell) {
        debug_assert_eq!(self.nodes[index].tag(), CellTag::App);
        self.nodes[index] = cell;
    }

    fn set_free_cell_at(&mut self, index: usize, cell: Cell) {
        debug_assert_eq!(self.nodes[index].tag(), CellTag::Free);
        self.nodes[index] = cell;
    }

    fn set_app_node_at(&mut self, index: usize, node: Node) {
        debug_assert_eq!(self.nodes[index].tag(), CellTag::App);
        self.nodes[index] = Cell::from_node(node, &mut self.cold_nodes);
    }

    fn set_node_at(&mut self, index: usize, node: Node) {
        self.drop_cold_payload(index);
        self.nodes[index] = Cell::from_node(node, &mut self.cold_nodes);
    }

    fn push_cell(&mut self, cell: Cell) -> NodeId {
        let id = NodeId::from_index(self.nodes.len());
        self.nodes.push(cell);
        self.gc_high_water_nodes = self.gc_high_water_nodes.max(self.nodes.len());
        id
    }

    #[cfg(feature = "gc-phase-profile")]
    fn gc_profile_record_allocated_slot(&mut self, id: NodeId) {
        self.gc_young_profile_allocated_slots.push(id);
    }

    fn push_node_fresh(&mut self, node: Node) -> NodeId {
        let cell = Cell::from_node(node, &mut self.cold_nodes);
        self.push_cell(cell)
    }

    fn cold_node(&self, id: NodeId) -> Option<&Node> {
        let cold = self.cell(id).cold_index()?;
        self.cold_nodes.get(cold)?.as_ref()
    }

    fn cold_node_mut(&mut self, id: NodeId) -> Option<&mut Node> {
        let cold = self.cell(id).cold_index()?;
        self.cold_nodes.get_mut(cold)?.as_mut()
    }

    fn drop_cold_payload(&mut self, index: usize) {
        if let Some(cold) = self.nodes[index].cold_index() {
            if let Some(slot) = self.cold_nodes.get_mut(cold) {
                *slot = None;
            }
        }
    }

    pub fn set_program_args(&mut self, args: Vec<Vec<u8>>) {
        self.program_args = args;
        self.arg_ref_array = None;
    }

    pub fn set_executable_path(&mut self, path: Option<Vec<u8>>) {
        self.executable_path = path;
    }

    pub fn enable_profile(&mut self) {
        self.profile = Some(EvalProfile::default());
    }

    pub fn take_profile(&mut self) -> Option<EvalProfile> {
        self.profile.take()
    }

    pub fn set_js_program_handle(&mut self, handle: u32) {
        self.js_program_handle = Some(handle);
    }

    pub fn label(&self, label: usize) -> Option<NodeId> {
        self.labels.get(&label).copied()
    }

    #[inline]
    fn pop_free_node(&mut self) -> Option<usize> {
        if self.free_nodes == 0 {
            return None;
        }
        let head = match self.free_head {
            Some(head) => head,
            None => unsafe {
                std::hint::unreachable_unchecked();
            },
        };
        let index = head.index();
        let cell = self.cell_at(index);
        debug_assert_eq!(
            cell.tag(),
            CellTag::Free,
            "free-list head did not point to a free node"
        );
        self.free_head = cell.option_id_word1();
        self.free_nodes -= 1;
        Some(index)
    }

    fn push_free_node(&mut self, index: usize) {
        self.set_cell_at(
            index,
            Cell {
                word0: CellTag::Free.bits(),
                word1: pack_option_id(self.free_head),
            },
        );
        self.free_head = Some(NodeId::from_index(index));
        self.free_nodes += 1;
    }

    pub fn push_node(&mut self, node: Node) -> NodeId {
        if self.profile.is_some() {
            self.profile_node_allocation(&node);
        }
        self.gc_allocations_since_collect = self.gc_allocations_since_collect.saturating_add(1);
        if let Some(index) = self.pop_free_node() {
            self.set_node_at(index, node);
            let id = NodeId::from_index(index);
            #[cfg(feature = "gc-phase-profile")]
            self.gc_profile_record_allocated_slot(id);
            id
        } else {
            let id = self.push_node_fresh(node);
            #[cfg(feature = "gc-phase-profile")]
            self.gc_profile_record_allocated_slot(id);
            id
        }
    }

    #[inline]
    fn push_app_node(&mut self, fun: NodeId, arg: NodeId) -> NodeId {
        self.gc_allocations_since_collect = self.gc_allocations_since_collect.saturating_add(1);
        #[cfg(feature = "eval-phase-profile")]
        let profiling = self.profile.is_some();
        #[cfg(feature = "eval-phase-profile")]
        let pop_started = profiling.then(Instant::now);
        let free_index = self.pop_free_node();
        #[cfg(feature = "eval-phase-profile")]
        if let Some(started) = pop_started {
            if let Some(profile) = self.profile.as_mut() {
                profile.app_alloc_free_pop_nanos = profile
                    .app_alloc_free_pop_nanos
                    .saturating_add(started.elapsed().as_nanos());
            }
        }
        if let Some(index) = free_index {
            #[cfg(feature = "eval-phase-profile")]
            if let Some(profile) = self.profile.as_mut() {
                profile.app_alloc_reused = profile.app_alloc_reused.saturating_add(1);
            }
            #[cfg(feature = "eval-phase-profile")]
            let write_started = profiling.then(Instant::now);
            self.set_free_cell_at(index, Cell::app(fun, arg));
            let id = NodeId::from_index(index);
            #[cfg(feature = "gc-phase-profile")]
            self.gc_profile_record_allocated_slot(id);
            #[cfg(feature = "eval-phase-profile")]
            if let Some(started) = write_started {
                if let Some(profile) = self.profile.as_mut() {
                    profile.app_alloc_reused_write_nanos = profile
                        .app_alloc_reused_write_nanos
                        .saturating_add(started.elapsed().as_nanos());
                }
            }
            id
        } else {
            #[cfg(feature = "eval-phase-profile")]
            if let Some(profile) = self.profile.as_mut() {
                profile.app_alloc_fresh = profile.app_alloc_fresh.saturating_add(1);
            }
            #[cfg(feature = "eval-phase-profile")]
            let push_started = profiling.then(Instant::now);
            let id = self.push_cell(Cell::app(fun, arg));
            #[cfg(feature = "gc-phase-profile")]
            self.gc_profile_record_allocated_slot(id);
            #[cfg(feature = "eval-phase-profile")]
            if let Some(started) = push_started {
                if let Some(profile) = self.profile.as_mut() {
                    profile.app_alloc_fresh_push_nanos = profile
                        .app_alloc_fresh_push_nanos
                        .saturating_add(started.elapsed().as_nanos());
                }
            }
            id
        }
    }

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

    fn mark_node_id(marked: &mut [bool], work: &mut Vec<NodeId>, id: NodeId) {
        if let Some(mark) = marked.get_mut(id.index()) {
            if !*mark {
                *mark = true;
                work.push(id);
            }
        }
    }

    fn node_pointer_target(&self, ptr: i64) -> Option<NodeId> {
        if ptr <= 0 {
            return None;
        }
        let slot_word = usize::try_from(ptr >> 32).ok()?;
        if slot_word == 0 {
            return None;
        }
        self.node_pointers.get(slot_word - 1).copied()
    }

    fn mark_pointer_target(&self, marked: &mut [bool], work: &mut Vec<NodeId>, ptr: i64) {
        if let Some(id) = self.node_pointer_target(ptr) {
            Self::mark_node_id(marked, work, id);
        }
    }

    #[cfg(feature = "gc-phase-profile")]
    fn gc_profile_young_edge(young: &[bool], id: NodeId) -> usize {
        young.get(id.index()).copied().unwrap_or(false) as usize
    }

    #[cfg(feature = "gc-phase-profile")]
    fn gc_profile_young_pointer_edge(&self, young: &[bool], ptr: i64) -> usize {
        self.node_pointer_target(ptr)
            .map(|id| Self::gc_profile_young_edge(young, id))
            .unwrap_or(0)
    }

    #[cfg(feature = "gc-phase-profile")]
    fn gc_profile_old_to_young_edges_for_cell(
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
            CellTag::Ptr | CellTag::RawFunPtr => {
                self.gc_profile_young_pointer_edge(young, cell.word1 as i64)
            }
            CellTag::Cold => match self.cold_node(NodeId::from_index(index)) {
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
    fn gc_profile_young_candidate_stats(
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

    fn mark_strict_redex(marked: &mut [bool], work: &mut Vec<NodeId>, redex: &StrictRedex) {
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

    fn mark_eval_frame(&self, marked: &mut [bool], work: &mut Vec<NodeId>, frame: &EvalFrame) {
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

    fn mark_eval_stack(&self, marked: &mut [bool], work: &mut Vec<NodeId>, stack: &EvalFrameStack) {
        if let Some(frame) = &stack.top {
            self.mark_eval_frame(marked, work, frame);
        }
        for frame in &stack.rest {
            self.mark_eval_frame(marked, work, frame);
        }
    }

    fn mark_machine_stack(&self, marked: &mut [bool], work: &mut Vec<NodeId>, stack: &EvalStack) {
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

    fn mark_eval_spine(marked: &mut [bool], work: &mut Vec<NodeId>, spine: &EvalSpine) {
        for idx in 0..spine.len() {
            Self::mark_node_id(marked, work, spine.desc_arg(idx));
            Self::mark_node_id(marked, work, spine.desc_app(idx));
        }
    }

    fn mark_persistent_spine(marked: &mut [bool], work: &mut Vec<NodeId>, spine: &PersistentSpine) {
        for id in &spine.apps {
            Self::mark_node_id(marked, work, *id);
        }
    }

    fn mark_program_roots(
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

    fn compress_marked_indirection(&mut self, id: NodeId) -> Option<NodeId> {
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

    fn canonical_gc_target(&mut self, id: NodeId) -> Option<NodeId> {
        let target = if matches!(
            self.nodes.get(id.index()).map(|cell| cell.tag()),
            Some(CellTag::Indir)
        ) {
            self.compress_marked_indirection(id)?
        } else {
            id
        };
        if let Some(value) = self.nodes.get(target.index())?.int_value() {
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

    fn mark_canonical_child(
        &mut self,
        marked: &mut [bool],
        work: &mut Vec<NodeId>,
        child: NodeId,
    ) -> NodeId {
        let target = self.canonical_gc_target(child).unwrap_or(child);
        Self::mark_node_id(marked, work, target);
        target
    }

    #[cfg(feature = "gc-phase-profile")]
    fn gc_profile_resolved_id(&self, id: NodeId) -> Option<NodeId> {
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

    #[cfg(feature = "gc-phase-profile")]
    fn gc_profile_prim(&self, id: NodeId) -> Option<Prim> {
        let id = self.gc_profile_resolved_id(id)?;
        self.nodes.get(id.index())?.prim()
    }

    #[cfg(feature = "gc-phase-profile")]
    fn gc_profile_app_fields(&self, id: NodeId) -> Option<(NodeId, NodeId)> {
        let id = self.gc_profile_resolved_id(id)?;
        self.nodes.get(id.index())?.app_fields()
    }

    #[cfg(feature = "gc-phase-profile")]
    fn gc_profile_flipped_prim(prim: Prim) -> Option<Prim> {
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
    fn profile_gc_red_opportunities(&mut self, fun: NodeId, arg: NodeId) {
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

    fn mark_reachable(
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
                    self.mark_pointer_target(marked, work, cell.word1 as i64);
                }
                CellTag::Cold => match self.cold_node(id) {
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

    fn weak_key_target(&mut self, key: NodeId) -> Option<NodeId> {
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

    fn sweep_weaks_after_mark(
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

    fn run_foreign_finalizer(
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

    fn run_dead_foreign_finalizers(&mut self, marked: &[bool]) -> Result<(), EvalError> {
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

    fn collect_garbage_between_steps(
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

    fn maybe_collect_garbage_between_steps(
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

    pub fn resolve(&self, mut id: NodeId) -> Result<NodeId, EvalError> {
        loop {
            let Some(cell) = self.nodes.get(id.index()).copied() else {
                return Err(EvalError::DanglingIndirection(id));
            };
            match cell.tag_bits() {
                tag if tag == CellTag::Indir.bits() => match cell.option_id_word1() {
                    Some(next) => id = next,
                    None => return Err(EvalError::DanglingIndirection(id)),
                },
                tag if tag == CellTag::Free.bits() => {
                    return Err(EvalError::DanglingIndirection(id));
                }
                _ => return Ok(id),
            }
        }
    }

    fn resolve_profiled(&mut self, mut id: NodeId) -> Result<NodeId, EvalError> {
        if self.profile.is_none() {
            return self.resolve(id);
        }

        let mut depth = 0;
        loop {
            let Some(cell) = self.nodes.get(id.index()).copied() else {
                return Err(EvalError::DanglingIndirection(id));
            };
            match cell.tag_bits() {
                tag if tag == CellTag::Indir.bits() => match cell.option_id_word1() {
                    Some(next) => {
                        id = next;
                        depth += 1;
                    }
                    None => return Err(EvalError::DanglingIndirection(id)),
                },
                tag if tag == CellTag::Free.bits() => {
                    return Err(EvalError::DanglingIndirection(id));
                }
                _ => {
                    self.profile_resolve_chain(depth);
                    return Ok(id);
                }
            }
        }
    }

    pub fn reduce_whnf(&mut self, limit: usize) -> Result<(NodeId, usize), EvalError> {
        let (root, steps) = self.reduce_whnf_from(self.root, limit, true, false)?;
        self.root = root;
        Ok((root, steps))
    }

    pub fn reduce_main(&mut self, limit: usize) -> Result<(NodeId, usize), EvalError> {
        let world = self.world();
        let root = self.app(self.root, world);
        let reductions = self.reductions;
        let root = self.reduce_node_whnf(root, limit)?;
        Ok((root, self.reductions - reductions))
    }

    pub fn reduction_count(&self) -> usize {
        self.reductions
    }

    pub fn uncaught_exception_message_bytes(&mut self, exn: NodeId) -> Result<Vec<u8>, EvalError> {
        let exn = self.resolve(exn)?;
        if let Some(code) = self.cell(exn).int_value() {
            return Ok(rts_exception_message(code).to_vec());
        }

        let u = self.prim("U");
        let k2 = self.prim("K2");
        let true_ = self.prim("A");
        let k2_true = self.app(k2, true_);
        let inner = self.app(u, k2_true);
        let show_exn = self.app(u, inner);
        let displayed = self.app(show_exn, exn);
        self.eval_string_bytes(displayed)
    }

    pub fn apply_stable_ptr_pointer(
        &mut self,
        stable_ptr: usize,
        arg: i64,
        limit: usize,
    ) -> Result<i64, EvalError> {
        let fun = self.deref_stable_ptr(stable_ptr)?;
        let arg = self.push_node(Node::Ptr(arg));
        let action = self.app(fun, arg);
        let perform_io = self.prim("IO.performIO");
        let root = self.app(perform_io, action);
        let root = self.reduce_node_whnf(root, limit)?;
        self.eval_pointer_value(root)
    }

    pub fn apply_js_wrapper(
        &mut self,
        tags: &str,
        stable_ptr: usize,
        args: &[JsValue],
        limit: usize,
    ) -> Result<JsValue, EvalError> {
        let tags = tags.as_bytes();
        validate_js_tags(tags)?;
        if args.len() != tags.len() - 1 {
            return Err(EvalError::InvalidArray);
        }

        let mut root = self.deref_stable_ptr(stable_ptr)?;
        for (tag, arg) in tags[1..].iter().copied().zip(args) {
            let arg = self.js_value_node(tag, arg)?;
            root = self.app(root, arg);
        }
        let perform_io = self.prim("IO.performIO");
        let root = self.app(perform_io, root);
        let root = self.reduce_node_whnf(root, limit)?;
        self.js_value_from_node(tags[0], root)
    }

    pub fn apply_js_wrapper_index(
        &mut self,
        wrapper_index: u32,
        stable_ptr: usize,
        args: &[JsValue],
        limit: usize,
    ) -> Result<JsValue, EvalError> {
        let tags = self.js_wrapper_tags(wrapper_index)?.to_owned();
        self.apply_js_wrapper(&tags, stable_ptr, args, limit)
    }

    pub fn js_wrapper_tags(&self, wrapper_index: u32) -> Result<&str, EvalError> {
        self.js_wrapper_tags
            .get(usize::try_from(wrapper_index).map_err(|_| EvalError::Overflow)?)
            .map(String::as_str)
            .ok_or(EvalError::InvalidArray)
    }

    #[cold]
    fn profile_step(&mut self, head: NodeId, arity: usize, heap_spine: bool) -> ProfileHead {
        #[cfg(feature = "eval-phase-profile")]
        let started = Instant::now();
        let key = self.profile_head_key(head);
        let profile = self.profile.as_mut().expect("profile checked");
        profile.step_attempts += 1;
        *profile.head_attempts.entry(key.clone()).or_default() += 1;
        *profile.spine_arity.entry(arity).or_default() += 1;
        if heap_spine {
            profile.heap_spines += 1;
        }
        profile.max_spine_arity = profile.max_spine_arity.max(arity);
        #[cfg(feature = "eval-phase-profile")]
        {
            profile.profile_step_nanos = profile
                .profile_step_nanos
                .saturating_add(started.elapsed().as_nanos());
        }
        Some(head)
    }

    #[cold]
    fn profile_reduction(&mut self, head: ProfileHead, reductions: usize) {
        let Some(head) = head else {
            return;
        };
        #[cfg(feature = "eval-phase-profile")]
        let started = Instant::now();
        let key = self.profile_head_key(head);
        let Some(profile) = self.profile.as_mut() else {
            return;
        };
        profile.successful_steps += 1;
        profile.reductions += reductions;
        *profile.head_reductions.entry(key).or_default() += reductions;
        #[cfg(feature = "eval-phase-profile")]
        {
            profile.profile_reduction_nanos = profile
                .profile_reduction_nanos
                .saturating_add(started.elapsed().as_nanos());
        }
    }

    #[cfg(feature = "eval-phase-profile")]
    #[cold]
    fn profile_stack_eval_step_head_time(&mut self, head: ProfileHead, nanos: u128) {
        let Some(head) = head else {
            return;
        };
        let started = Instant::now();
        let key = self.profile_head_key(head);
        let Some(profile) = self.profile.as_mut() else {
            return;
        };
        *profile.stack_eval_step_head_nanos.entry(key).or_default() += nanos;
        profile.profile_stack_head_time_nanos = profile
            .profile_stack_head_time_nanos
            .saturating_add(started.elapsed().as_nanos());
    }

    #[cfg(feature = "eval-phase-profile")]
    #[cold]
    fn profile_stack_arg_read_head_time(&mut self, head: ProfileHead, nanos: u128) {
        let Some(head) = head else {
            return;
        };
        let started = Instant::now();
        let key = self.profile_head_key(head);
        let Some(profile) = self.profile.as_mut() else {
            return;
        };
        *profile.stack_arg_read_head_nanos.entry(key).or_default() += nanos;
        profile.profile_stack_head_time_nanos = profile
            .profile_stack_head_time_nanos
            .saturating_add(started.elapsed().as_nanos());
    }

    #[cfg(feature = "eval-phase-profile")]
    #[cold]
    fn profile_stack_app_alloc_site_time(&mut self, site: &'static str, nanos: u128) {
        let started = Instant::now();
        let Some(profile) = self.profile.as_mut() else {
            return;
        };
        *profile
            .stack_app_alloc_site_nanos
            .entry(site.to_owned())
            .or_default() += nanos;
        profile.profile_stack_head_time_nanos = profile
            .profile_stack_head_time_nanos
            .saturating_add(started.elapsed().as_nanos());
    }

    #[cfg(feature = "eval-phase-profile")]
    #[cold]
    fn profile_stack_apply_rewrite_head_time(&mut self, head: ProfileHead, nanos: u128) {
        let Some(head) = head else {
            return;
        };
        let started = Instant::now();
        let key = self.profile_head_key(head);
        let Some(profile) = self.profile.as_mut() else {
            return;
        };
        *profile
            .stack_apply_rewrite_head_nanos
            .entry(key)
            .or_default() += nanos;
        profile.profile_stack_head_time_nanos = profile
            .profile_stack_head_time_nanos
            .saturating_add(started.elapsed().as_nanos());
    }

    #[cfg(feature = "eval-phase-profile")]
    #[cold]
    fn profile_stack_apply_app_head_time(&mut self, head: ProfileHead, nanos: u128) {
        let Some(head) = head else {
            return;
        };
        let started = Instant::now();
        let key = self.profile_head_key(head);
        let Some(profile) = self.profile.as_mut() else {
            return;
        };
        *profile.stack_apply_app_head_nanos.entry(key).or_default() += nanos;
        profile.profile_stack_head_time_nanos = profile
            .profile_stack_head_time_nanos
            .saturating_add(started.elapsed().as_nanos());
    }

    #[cfg(feature = "eval-phase-profile")]
    #[cold]
    fn profile_stack_force_frame_head_time(&mut self, head: ProfileHead, nanos: u128) {
        let Some(head) = head else {
            return;
        };
        let started = Instant::now();
        let key = self.profile_head_key(head);
        let Some(profile) = self.profile.as_mut() else {
            return;
        };
        *profile.stack_force_frame_head_nanos.entry(key).or_default() += nanos;
        profile.profile_stack_head_time_nanos = profile
            .profile_stack_head_time_nanos
            .saturating_add(started.elapsed().as_nanos());
    }

    #[cfg(feature = "eval-phase-profile")]
    #[cold]
    fn profile_stack_inner_descent_head_time(&mut self, head: ProfileHead, nanos: u128) {
        let Some(head) = head else {
            return;
        };
        let started = Instant::now();
        let key = self.profile_head_key(head);
        let Some(profile) = self.profile.as_mut() else {
            return;
        };
        *profile
            .stack_inner_descent_head_nanos
            .entry(key)
            .or_default() += nanos;
        profile.profile_stack_head_time_nanos = profile
            .profile_stack_head_time_nanos
            .saturating_add(started.elapsed().as_nanos());
    }

    #[cfg(feature = "eval-phase-profile")]
    #[cold]
    fn profile_stack_arg_read_time(&mut self, nanos: u128) {
        if let Some(profile) = self.profile.as_mut() {
            profile.stack_arg_read_nanos = profile.stack_arg_read_nanos.saturating_add(nanos);
        }
    }

    #[cfg(feature = "eval-phase-profile")]
    #[cold]
    fn profile_stack_app_alloc_time(&mut self, nanos: u128) {
        if let Some(profile) = self.profile.as_mut() {
            profile.stack_app_alloc_nanos = profile.stack_app_alloc_nanos.saturating_add(nanos);
        }
    }

    #[cfg(feature = "eval-phase-profile")]
    #[cold]
    fn profile_stack_apply_rewrite_time(&mut self, nanos: u128) {
        if let Some(profile) = self.profile.as_mut() {
            profile.stack_apply_rewrite_nanos =
                profile.stack_apply_rewrite_nanos.saturating_add(nanos);
        }
    }

    #[cfg(feature = "eval-phase-profile")]
    #[cold]
    fn profile_stack_apply_app_time(&mut self, nanos: u128) {
        if let Some(profile) = self.profile.as_mut() {
            profile.stack_apply_app_nanos = profile.stack_apply_app_nanos.saturating_add(nanos);
        }
    }

    #[cfg(feature = "eval-phase-profile")]
    #[cold]
    fn profile_stack_force_frame_time(&mut self, nanos: u128) {
        if let Some(profile) = self.profile.as_mut() {
            profile.stack_force_frame_nanos = profile.stack_force_frame_nanos.saturating_add(nanos);
        }
    }

    #[cfg(feature = "eval-phase-profile")]
    #[cold]
    fn profile_stack_inner_descent_time(&mut self, nanos: u128) {
        if let Some(profile) = self.profile.as_mut() {
            profile.stack_inner_descent_nanos =
                profile.stack_inner_descent_nanos.saturating_add(nanos);
        }
    }

    #[cfg(feature = "eval-phase-profile")]
    #[cold]
    fn profile_app_alloc_bookkeeping_time(&mut self, nanos: u128) {
        if let Some(profile) = self.profile.as_mut() {
            profile.profile_app_alloc_bookkeeping_nanos = profile
                .profile_app_alloc_bookkeeping_nanos
                .saturating_add(nanos);
        }
    }

    #[cold]
    fn profile_resolve_chain(&mut self, depth: usize) {
        let Some(profile) = self.profile.as_mut() else {
            return;
        };
        profile.resolve_calls += 1;
        profile.resolve_indirections += depth;
        profile.max_resolve_chain = profile.max_resolve_chain.max(depth);
        *profile.resolve_chain.entry(depth).or_default() += 1;
    }

    #[cold]
    fn profile_shortcut(&mut self, key: &'static str, count: usize) {
        if count == 0 {
            return;
        }
        let Some(profile) = self.profile.as_mut() else {
            return;
        };
        if let Some(existing) = profile.shortcut_hits.get_mut(key) {
            *existing += count;
        } else {
            profile.shortcut_hits.insert(key.to_owned(), count);
        }
    }

    #[cold]
    fn profile_arg_materialization(&mut self, nodes: usize) {
        if let Some(profile) = self.profile.as_mut() {
            profile.arg_materializations += 1;
            profile.arg_materialized_nodes += nodes;
        }
    }

    #[cold]
    fn profile_spine_rewrite(&mut self, extra_args: usize) {
        if let Some(profile) = self.profile.as_mut() {
            profile.spine_rewrites += 1;
            profile.spine_rewrite_extra_args += extra_args;
        }
    }

    #[cold]
    fn profile_app_rewrite(&mut self, extra_args: usize) {
        if let Some(profile) = self.profile.as_mut() {
            profile.app_rewrites += 1;
            profile.app_rewrite_extra_args += extra_args;
        }
    }

    #[cold]
    fn profile_stack_rewrite(&mut self, used: usize, wrote_indirection: bool) {
        if let Some(profile) = self.profile.as_mut() {
            profile.stack_rewrites += 1;
            profile.stack_rewrite_apps += used;
            if wrote_indirection {
                profile.stack_rewrite_indirections += 1;
            }
        }
    }

    #[cold]
    fn profile_stack_app_update(&mut self, used: usize) {
        if let Some(profile) = self.profile.as_mut() {
            profile.stack_app_updates += 1;
            profile.stack_app_update_apps += used;
            if used == 0 {
                profile.stack_app_update_allocations += 1;
            }
        }
    }

    #[cold]
    fn profile_stack_rethread(&mut self, apps: usize) {
        if let Some(profile) = self.profile.as_mut() {
            profile.stack_rethreads += 1;
            profile.stack_rethread_apps += apps;
        }
    }

    #[cold]
    fn profile_stack_descent_push(&mut self) {
        if let Some(profile) = self.profile.as_mut() {
            profile.stack_descent_pushes += 1;
        }
    }

    #[cold]
    fn profile_stack_arg_reads(&mut self, reads: usize) {
        if let Some(profile) = self.profile.as_mut() {
            profile.stack_arg_reads += reads;
        }
    }

    #[cold]
    fn profile_stack_arg_batch(&mut self) {
        if let Some(profile) = self.profile.as_mut() {
            profile.stack_arg_batches += 1;
        }
    }

    #[cold]
    fn profile_persistent_force(&mut self) {
        if let Some(profile) = self.profile.as_mut() {
            profile.persistent_forces += 1;
        }
    }

    #[cold]
    fn profile_persistent_fallback(&mut self) {
        if let Some(profile) = self.profile.as_mut() {
            profile.persistent_fallbacks += 1;
        }
    }

    #[cold]
    fn profile_stack_fallback_head(&mut self, head: NodeId) {
        if self.profile.is_some() {
            let key = self.profile_head_key(head);
            if let Some(profile) = self.profile.as_mut() {
                *profile.stack_fallback_heads.entry(key).or_insert(0) += 1;
            }
        }
    }

    #[cold]
    fn profile_fallback_eval_loop_step(&mut self) {
        if let Some(profile) = self.profile.as_mut() {
            profile.fallback_eval_loop_steps += 1;
        }
    }

    #[cold]
    fn profile_strict_redex_snapshot(&mut self, apps: usize) {
        if let Some(profile) = self.profile.as_mut() {
            profile.strict_redex_snapshots += 1;
            profile.strict_redex_snapshot_apps += apps;
        }
    }

    #[cold]
    fn profile_remaining_app_scan(&mut self, apps: usize) {
        if let Some(profile) = self.profile.as_mut() {
            profile.remaining_app_scans += 1;
            profile.remaining_app_scan_apps += apps;
        }
    }

    #[cold]
    fn profile_eval_frame_push(&mut self, kind: &'static str) {
        if let Some(profile) = self.profile.as_mut() {
            profile.eval_frame_pushes += 1;
            *profile
                .eval_frame_push_kinds
                .entry(kind.to_owned())
                .or_default() += 1;
        }
    }

    #[cold]
    fn profile_small_int_cache_hit(&mut self) {
        if let Some(profile) = self.profile.as_mut() {
            profile.small_int_cache_hits += 1;
        }
    }

    #[cold]
    fn profile_small_int_cache_miss(&mut self) {
        if let Some(profile) = self.profile.as_mut() {
            profile.small_int_cache_misses += 1;
        }
    }

    #[cold]
    fn profile_non_small_int_allocation(&mut self) {
        if let Some(profile) = self.profile.as_mut() {
            profile.non_small_int_allocations += 1;
        }
    }

    #[cold]
    fn profile_primitive_dispatch_probe(&mut self, key: &'static str) {
        if let Some(profile) = self.profile.as_mut() {
            *profile
                .primitive_dispatch_probes
                .entry(key.to_owned())
                .or_default() += 1;
        }
    }

    #[cold]
    fn profile_primitive_dispatch_hit(&mut self, key: &'static str) {
        if let Some(profile) = self.profile.as_mut() {
            *profile
                .primitive_dispatch_hits
                .entry(key.to_owned())
                .or_default() += 1;
        }
    }

    #[cold]
    fn profile_strict_primitive_dispatch(
        &mut self,
        args_len: usize,
        action: StrictPrimitiveAction,
    ) {
        macro_rules! probe {
            ($min_args:expr, $key:literal, $pattern:pat) => {
                if args_len >= $min_args {
                    self.profile_primitive_dispatch_probe($key);
                    if matches!(action, $pattern) {
                        self.profile_primitive_dispatch_hit($key);
                        return;
                    }
                }
            };
        }

        probe!(2, "strict_int_binop", StrictPrimitiveAction::IntBin(_));
        probe!(1, "strict_int_unop", StrictPrimitiveAction::IntUn(_));
        probe!(2, "strict_int64_binop", StrictPrimitiveAction::Int64Bin(_));
        probe!(1, "strict_int64_unop", StrictPrimitiveAction::Int64Un(_));
        probe!(
            2,
            "strict_float64_binop",
            StrictPrimitiveAction::Float64Bin(_)
        );
        probe!(
            1,
            "strict_float64_unop",
            StrictPrimitiveAction::Float64Un(_)
        );
        probe!(
            2,
            "strict_float32_binop",
            StrictPrimitiveAction::Float32Bin(_)
        );
        probe!(
            1,
            "strict_float32_unop",
            StrictPrimitiveAction::Float32Un(_)
        );
        probe!(2, "strict_bytes_binop", StrictPrimitiveAction::BytesBin(_));
        probe!(1, "strict_conversion", StrictPrimitiveAction::Conversion(_));
    }

    #[cold]
    fn profile_node_allocation(&mut self, node: &Node) {
        if let Some(profile) = self.profile.as_mut() {
            *profile
                .node_allocations
                .entry(node_allocation_key(node).to_owned())
                .or_default() += 1;
        }
    }

    fn eval_loop_result(
        &mut self,
        profile_head: ProfileHead,
        node: NodeId,
        reductions: usize,
    ) -> EvalLoopStep {
        if profile_head.is_some() {
            self.profile_reduction(profile_head, reductions);
        }
        EvalLoopStep { node, reductions }
    }

    fn fill_eval_spine(
        &mut self,
        mut node: NodeId,
        spine: &mut EvalSpine,
    ) -> Result<NodeId, EvalError> {
        spine.clear();
        while let Some((fun, arg)) = self.cell(node).app_fields() {
            spine.push_desc(arg, node);
            node = self.resolve_profiled(fun)?;
        }
        Ok(node)
    }

    fn apply_eval_spine_rewrite(
        &mut self,
        root: NodeId,
        spine: &EvalSpine,
        used: usize,
        mut node: NodeId,
    ) -> NodeId {
        let len = spine.len();
        debug_assert!(used <= len);
        if used == 0 && len == 0 {
            if node != root {
                self.set_cell_at(root.index(), Cell::indir(Some(node)));
            }
            return node;
        }
        if used > 0 {
            let redex = spine.app(used - 1);
            if node != redex {
                self.set_app_cell_at(redex.index(), Cell::indir(Some(node)));
            }
        }
        for head_idx in used..len {
            let app = spine.app(head_idx);
            let arg = spine.arg(head_idx);
            self.set_app_cell_at(app.index(), Cell::app(node, arg));
            node = app;
        }
        node
    }

    fn apply_eval_spine_app(
        &mut self,
        root: NodeId,
        spine: &EvalSpine,
        used: usize,
        fun: NodeId,
        arg: NodeId,
    ) -> NodeId {
        let len = spine.len();
        debug_assert!(used <= len);
        let mut node = if used == 0 {
            self.app(fun, arg)
        } else {
            let redex = spine.app(used - 1);
            self.set_app_cell_at(redex.index(), Cell::app(fun, arg));
            redex
        };
        if used == 0 && len == 0 {
            if node != root {
                self.set_cell_at(root.index(), Cell::indir(Some(node)));
            }
            return node;
        }
        for head_idx in used..len {
            let app = spine.app(head_idx);
            let arg = spine.arg(head_idx);
            self.set_app_cell_at(app.index(), Cell::app(node, arg));
            node = app;
        }
        node
    }

    fn eval_loop_app_result(
        &mut self,
        profile_head: ProfileHead,
        root: NodeId,
        spine: &EvalSpine,
        used: usize,
        fun: NodeId,
        arg: NodeId,
        reductions: usize,
    ) -> EvalLoopStep {
        if profile_head.is_some() {
            self.profile_app_rewrite(spine.len() - used);
        }
        let node = self.apply_eval_spine_app(root, spine, used, fun, arg);
        self.eval_loop_result(profile_head, node, reductions)
    }

    fn strict_redex_from_eval_spine(
        &mut self,
        root: NodeId,
        used: usize,
        spine: &EvalSpine,
        scratch_apps: &mut Vec<NodeId>,
    ) -> StrictRedex {
        if spine.len() == used {
            return StrictRedex::Root(root);
        }
        if self.profile.is_some() {
            self.profile_strict_redex_snapshot(spine.len());
        }
        spine.write_apps_head_order(scratch_apps);
        StrictRedex::Spine {
            root,
            used,
            apps: scratch_apps.clone(),
        }
    }

    fn strict_redex_from_persistent_spine(
        &mut self,
        root: NodeId,
        used: usize,
        spine: &PersistentSpine,
    ) -> StrictRedex {
        if spine.len() == used {
            return StrictRedex::Root(root);
        }
        if self.profile.is_some() {
            self.profile_strict_redex_snapshot(spine.len());
        }
        StrictRedex::Spine {
            root,
            used,
            apps: spine.apps.iter().copied().collect(),
        }
    }

    #[cold]
    fn profile_head_key(&self, head: NodeId) -> String {
        match self.node_for_debug(head) {
            Node::App(_, _) => "App".to_owned(),
            Node::Indir(_) => "Indir".to_owned(),
            Node::Free(_) => "Free".to_owned(),
            Node::Prim(name) => format!("Prim:{name}"),
            Node::Int(_) => "Int".to_owned(),
            Node::Int64(_) => "Int64".to_owned(),
            Node::Float64(_) => "Float64".to_owned(),
            Node::Float32(_) => "Float32".to_owned(),
            Node::ThreadId(_) => "ThreadId".to_owned(),
            Node::Ptr(_) => "Ptr".to_owned(),
            Node::RawFunPtr(_) => "RawFunPtr".to_owned(),
            Node::ForeignPtr(_) => "ForeignPtr".to_owned(),
            Node::Weak(_) => "Weak".to_owned(),
            Node::MVar(_) => "MVar".to_owned(),
            Node::BigInt(_) => "BigInt".to_owned(),
            Node::Bytes(_) => "Bytes".to_owned(),
            Node::BytesView(_) => "BytesView".to_owned(),
            Node::MutableBytes(_) => "MutableBytes".to_owned(),
            Node::Array(_) => "Array".to_owned(),
            Node::Ffi(name) => format!("Ffi:{name}"),
            Node::JsCall(call) => format!("JsCall:{}", call.tags),
            Node::JsWrap { tags } => format!("JsWrap:{tags}"),
            Node::FunPtr(name) => format!("FunPtr:{name}"),
            Node::Tick(_) => "Tick".to_owned(),
        }
    }

    fn eval_loop_step(
        &mut self,
        root: NodeId,
        budget: usize,
        spine: &mut EvalSpine,
        scratch_args: &mut Vec<NodeId>,
        scratch_apps: &mut Vec<NodeId>,
        frame_stack: &mut EvalFrameStack,
        strict_markers: bool,
    ) -> Result<Option<EvalLoopStep>, EvalError> {
        if self.profile.is_some() {
            self.profile_fallback_eval_loop_step();
        }
        let head = self.fill_eval_spine(root, spine)?;
        let args_len = spine.len();
        let profile_head = if self.profile.is_some() {
            self.profile_step(head, args_len, spine.is_heap())
        } else {
            None
        };

        macro_rules! arg {
            ($idx:expr) => {
                spine.arg($idx)
            };
        }
        macro_rules! app_step {
            ($used:expr, $fun:expr, $arg:expr) => {
                return Ok(Some(self.eval_loop_app_result(
                    profile_head,
                    root,
                    spine,
                    $used,
                    $fun,
                    $arg,
                    1,
                )));
            };
        }
        macro_rules! rewrite_step {
            ($used:expr, $node:expr, $reductions:expr) => {{
                if profile_head.is_some() {
                    self.profile_spine_rewrite(spine.len() - $used);
                }
                let node = self.apply_eval_spine_rewrite(root, spine, $used, $node);
                return Ok(Some(self.eval_loop_result(profile_head, node, $reductions)));
            }};
        }
        macro_rules! strict_marker_step {
            ($used:expr, $variant:ident, $frame:ident, $kind:expr, $next:expr) => {{
                let redex = self.strict_redex_from_eval_spine(root, $used, spine, scratch_apps);
                if self.profile.is_some() {
                    self.profile_eval_frame_push(stringify!($variant));
                }
                frame_stack.push(EvalFrame::$variant($frame {
                    redex,
                    profile_head,
                    kind: $kind,
                }));
                return Ok(Some(EvalLoopStep {
                    node: $next,
                    reductions: 0,
                }));
            }};
        }
        macro_rules! strict_int64_shift_marker_step {
            ($used:expr, $op:expr, $x:expr, $next:expr) => {{
                let redex = self.strict_redex_from_eval_spine(root, $used, spine, scratch_apps);
                if self.profile.is_some() {
                    self.profile_eval_frame_push("Int64Shift");
                }
                frame_stack.push(EvalFrame::Int64Shift(Int64ShiftFrame {
                    redex,
                    profile_head,
                    op: $op,
                    x: $x,
                }));
                return Ok(Some(EvalLoopStep {
                    node: $next,
                    reductions: 0,
                }));
            }};
        }

        let head_dispatch = match self.cell(head).prim() {
            Some(Prim::Known(known)) => EvalHead::Known(known),
            Some(Prim::Runtime(runtime)) => {
                let name = runtime.name();
                let action = runtime.strict_action(args_len);
                let needs_fallback_name =
                    !strict_markers || matches!(action, StrictPrimitiveAction::None);
                EvalHead::Other {
                    action,
                    fallback_name: needs_fallback_name.then_some(name),
                }
            }
            None => match self.cold_node(head) {
                Some(Node::Ffi(name)) => EvalHead::Ffi(name.to_string()),
                Some(Node::JsCall(call)) => EvalHead::JsCall {
                    tags: call.tags.clone(),
                    body: call.body.clone(),
                },
                Some(Node::JsWrap { tags }) => EvalHead::JsWrap {
                    tags: tags.to_string(),
                },
                _ => EvalHead::Whnf,
            },
        };

        let (known, strict_action, fallback_name) = match head_dispatch {
            EvalHead::Ffi(name) => {
                if self.profile.is_some() {
                    self.profile_arg_materialization(args_len);
                }
                spine.write_args_head_order(scratch_args);
                let Some((used, node)) = self.ffi_call(&name, scratch_args.as_slice())? else {
                    return Ok(None);
                };
                rewrite_step!(used, node, 1);
            }
            EvalHead::JsCall { tags, body } => {
                if self.profile.is_some() {
                    self.profile_arg_materialization(args_len);
                }
                spine.write_args_head_order(scratch_args);
                let Some((used, node)) = self.js_call(&tags, &body, scratch_args.as_slice())?
                else {
                    return Ok(None);
                };
                rewrite_step!(used, node, 1);
            }
            EvalHead::JsWrap { tags } => {
                if self.profile.is_some() {
                    self.profile_arg_materialization(args_len);
                }
                spine.write_args_head_order(scratch_args);
                let Some((used, node)) = self.js_wrap(&tags, scratch_args.as_slice())? else {
                    return Ok(None);
                };
                rewrite_step!(used, node, 1);
            }
            EvalHead::Known(known) => (Some(known), StrictPrimitiveAction::None, None),
            EvalHead::Other {
                action,
                fallback_name,
            } => (None, action, fallback_name),
            EvalHead::Whnf => return Ok(None),
        };
        use KnownPrim::*;

        if known == Some(U) && args_len >= 2 {
            if let Some(node) = self.selector_pair_field(arg!(0), arg!(1))? {
                self.profile_shortcut("selector_pair_field", 1);
                rewrite_step!(2, node, 1);
            }
        }

        if known == Some(IoThen) && args_len >= 3 && budget >= 2 {
            if let Some(reductions) = self.ignored_io_action_reductions(arg!(0), budget - 1)? {
                self.profile_shortcut("io_then_ignored_action", 1);
                let world = self
                    .run_ignored_io_action(arg!(0), arg!(2))?
                    .expect("preflighted ignored IO action should execute");
                let node = self.app(arg!(1), world);
                rewrite_step!(3, node, reductions + 1);
            }
            let k = self.prim("K");
            let then = self.app(k, arg!(1));
            let action = self.app(arg!(0), arg!(2));
            let node = self.app(action, then);
            rewrite_step!(3, node, 2);
        }

        if known == Some(IoBind) && args_len >= 3 {
            if let Some(result) = self.io_return_action_result(arg!(0))? {
                self.profile_shortcut("io_bind_return_action", 1);
                let next = self.app(arg!(1), result);
                let node = self.app(next, arg!(2));
                rewrite_step!(3, node, 2);
            }
        }

        if strict_markers {
            match known {
                Some(IoStrict) if args_len >= 2 => {
                    strict_marker_step!(
                        2,
                        Whnf,
                        WhnfFrame,
                        WhnfFrameKind::IoStrict {
                            action: arg!(0),
                            value: arg!(1),
                        },
                        arg!(1)
                    );
                }
                Some(Seq) if args_len >= 2 => {
                    strict_marker_step!(
                        2,
                        Whnf,
                        WhnfFrame,
                        WhnfFrameKind::Seq { result: arg!(1) },
                        arg!(0)
                    );
                }
                Some(IsInt) if args_len >= 1 => {
                    strict_marker_step!(1, Whnf, WhnfFrame, WhnfFrameKind::IsInt, arg!(0));
                }
                _ => {}
            }
        }

        if strict_markers && known.is_none() {
            if self.profile.is_some() {
                self.profile_strict_primitive_dispatch(args_len, strict_action);
            }
            match strict_action {
                StrictPrimitiveAction::IntBin(op) => {
                    let x = arg!(0);
                    strict_marker_step!(
                        2,
                        Int,
                        IntFrame,
                        IntFrameKind::BinSecond { op, x },
                        arg!(1)
                    );
                }
                StrictPrimitiveAction::IntUn(op) => {
                    strict_marker_step!(1, Int, IntFrame, IntFrameKind::Un { op }, arg!(0));
                }
                StrictPrimitiveAction::Int64Bin(op) => {
                    if op.rhs_is_shift() {
                        strict_int64_shift_marker_step!(2, op, arg!(0), arg!(1));
                    } else if op.driver_marker_safe() {
                        strict_marker_step!(
                            2,
                            Int64,
                            Int64Frame,
                            Int64FrameKind::BinSecond { op, x: arg!(0) },
                            arg!(1)
                        );
                    }
                }
                StrictPrimitiveAction::Int64Un(op) => {
                    strict_marker_step!(1, Int64, Int64Frame, Int64FrameKind::Un { op }, arg!(0));
                }
                StrictPrimitiveAction::Float64Bin(op) => {
                    strict_marker_step!(
                        2,
                        Float64,
                        Float64Frame,
                        Float64FrameKind::BinSecond { op, x: arg!(0) },
                        arg!(1)
                    );
                }
                StrictPrimitiveAction::Float64Un(op) => {
                    strict_marker_step!(
                        1,
                        Float64,
                        Float64Frame,
                        Float64FrameKind::Un { op },
                        arg!(0)
                    );
                }
                StrictPrimitiveAction::Float32Bin(op) => {
                    strict_marker_step!(
                        2,
                        Float32,
                        Float32Frame,
                        Float32FrameKind::BinSecond { op, x: arg!(0) },
                        arg!(1)
                    );
                }
                StrictPrimitiveAction::Float32Un(op) => {
                    strict_marker_step!(
                        1,
                        Float32,
                        Float32Frame,
                        Float32FrameKind::Un { op },
                        arg!(0)
                    );
                }
                StrictPrimitiveAction::BytesBin(op) => {
                    strict_marker_step!(
                        2,
                        Bytes,
                        BytesFrame,
                        BytesFrameKind::BinSecond { op, x: arg!(0) },
                        arg!(1)
                    );
                }
                StrictPrimitiveAction::Conversion(kind) => {
                    strict_marker_step!(1, Conversion, ConversionFrame, kind, arg!(0));
                }
                StrictPrimitiveAction::None => {}
            }
        }

        let rewrite = match known {
            Some(I | Ord | Chr) if args_len >= 1 => Some((1, arg!(0))),
            Some(K) if args_len >= 2 => Some((2, arg!(0))),
            Some(A) if args_len >= 2 => Some((2, arg!(1))),
            Some(U) if args_len >= 2 => {
                app_step!(2, arg!(1), arg!(0));
            }
            Some(IoPerformIo) if args_len >= 1 => {
                let world = self.world();
                let k = self.prim("K");
                let action = self.app(arg!(0), world);
                let n = self.app(action, k);
                Some((1, n))
            }
            Some(IoAtomic) if args_len >= 2 => {
                let k = self.prim("K");
                let action = self.app(arg!(0), arg!(1));
                let result = self.app(action, k);
                let pair = self.prim("P");
                let result_pair = self.app(pair, result);
                let n = self.app(result_pair, arg!(1));
                Some((2, n))
            }
            Some(IoBind) if args_len >= 3 => {
                let action = self.app(arg!(0), arg!(2));
                let n = self.app(action, arg!(1));
                Some((3, n))
            }
            Some(IoThen) if args_len >= 2 => {
                let bind = self.prim("IO.>>=");
                let bind_action = self.app(bind, arg!(0));
                let k = self.prim("K");
                let then = self.app(k, arg!(1));
                let n = self.app(bind_action, then);
                Some((2, n))
            }
            Some(IoReturn) if args_len >= 3 => {
                let kx = self.app(arg!(2), arg!(0));
                let n = self.app(kx, arg!(1));
                Some((3, n))
            }
            Some(IoLazyBind) if args_len >= 3 => {
                let world_result = self.app(arg!(0), arg!(2));
                let fst = self.fst();
                let snd = self.snd();
                let result = self.app(fst, world_result);
                let world = self.app(snd, world_result);
                let next = self.app(arg!(1), result);
                let n = self.app(next, world);
                Some((3, n))
            }
            Some(IoStrict) if args_len >= 2 => {
                let n = self.app(arg!(0), arg!(1));
                Some((2, n))
            }
            Some(IoGc) if args_len >= 2 => {
                let unit = self.prim("I");
                Some((2, self.pair(unit, arg!(1))))
            }
            Some(IoStats) if args_len >= 1 => {
                let alloc = self.int(i64::try_from(self.nodes.len()).unwrap_or(i64::MAX));
                let reductions = self.int(i64::try_from(self.reductions).unwrap_or(i64::MAX));
                let stats = self.pair(alloc, reductions);
                Some((1, self.pair(stats, arg!(0))))
            }
            Some(IoPp) if args_len >= 2 => {
                #[cfg(not(target_arch = "wasm32"))]
                {
                    let rendered = self.render(arg!(0));
                    eprintln!("{rendered}");
                }
                let unit = self.prim("I");
                Some((2, self.pair(unit, arg!(1))))
            }
            Some(IoPrint) if args_len >= 3 => {
                let ptr = self.eval_pointer_value(arg!(0))?;
                let value = self.reduce_node_whnf(arg!(1), FORCE_REDUCTION_LIMIT)?;
                let printed = self.print_program(value)?;
                self.write_bfile_bytes(ptr, &printed)?;
                let unit = self.prim("I");
                Some((3, self.pair(unit, arg!(2))))
            }
            Some(IoSerialize) if args_len >= 3 => {
                let ptr = self.eval_pointer_value(arg!(0))?;
                let value = self.reduce_node_whnf(arg!(1), FORCE_REDUCTION_LIMIT)?;
                let serialized = self.serialize_program(value)?;
                self.write_bfile_bytes(ptr, &serialized)?;
                let unit = self.prim("I");
                Some((3, self.pair(unit, arg!(2))))
            }
            Some(IoDeserialize) if args_len >= 2 => {
                let ptr = self.eval_pointer_value(arg!(0))?;
                let value = self.deserialize_bfile(ptr)?;
                Some((2, self.pair(value, arg!(1))))
            }
            Some(IoGetArgRef) if args_len >= 1 => {
                let arg_array = self.arg_ref_array();
                Some((1, self.pair(arg_array, arg!(0))))
            }
            Some(IoThid) if args_len >= 1 => {
                let thread = self.push_node(Node::ThreadId(1));
                Some((1, self.pair(thread, arg!(0))))
            }
            Some(IoYield) if args_len >= 1 => {
                let unit = self.prim("I");
                Some((1, self.pair(unit, arg!(0))))
            }
            Some(IoGetMaskingState) if args_len >= 1 => {
                let state = self.int(self.masking_state);
                Some((1, self.pair(state, arg!(0))))
            }
            Some(IoSetMaskingState) if args_len >= 2 => {
                self.masking_state = self.eval_int(arg!(0))?;
                let unit = self.prim("I");
                Some((2, self.pair(unit, arg!(1))))
            }
            Some(Dynsym) if args_len >= 1 => {
                let name = self.eval_ffi_name(arg!(0))?;
                Some((1, self.push_node(Node::ffi(name))))
            }
            Some(IoThreadStatus) if args_len >= 2 => {
                self.eval_thread_id(arg!(0))?;
                let status = self.int(0);
                Some((2, self.pair(status, arg!(1))))
            }
            Some(IoNewMVar) if args_len >= 1 => {
                let mvar = self.push_node(Node::MVar(None));
                Some((1, self.pair(mvar, arg!(0))))
            }
            Some(IoTakeMVar) if args_len >= 2 => {
                let mvar = self.eval_mvar_id(arg!(0))?;
                let value = self.take_mvar(mvar)?.ok_or(EvalError::InvalidMVar)?;
                Some((2, self.pair(value, arg!(1))))
            }
            Some(IoReadMVar) if args_len >= 2 => {
                let mvar = self.eval_mvar_id(arg!(0))?;
                let value = self.read_mvar(mvar)?.ok_or(EvalError::InvalidMVar)?;
                Some((2, self.pair(value, arg!(1))))
            }
            Some(IoPutMVar) if args_len >= 3 => {
                let mvar = self.eval_mvar_id(arg!(0))?;
                self.put_mvar(mvar, arg!(1))?;
                let unit = self.prim("I");
                Some((3, self.pair(unit, arg!(2))))
            }
            Some(IoTryTakeMVar) if args_len >= 2 => {
                let mvar = self.eval_mvar_id(arg!(0))?;
                let value = match self.take_mvar(mvar)? {
                    Some(value) => self.just(value),
                    None => self.nothing(),
                };
                Some((2, self.pair(value, arg!(1))))
            }
            Some(IoTryReadMVar) if args_len >= 2 => {
                let mvar = self.eval_mvar_id(arg!(0))?;
                let value = match self.read_mvar(mvar)? {
                    Some(value) => self.just(value),
                    None => self.nothing(),
                };
                Some((2, self.pair(value, arg!(1))))
            }
            Some(IoTryPutMVar) if args_len >= 3 => {
                let mvar = self.eval_mvar_id(arg!(0))?;
                let value = if self.try_put_mvar(mvar, arg!(1))? {
                    self.prim("A")
                } else {
                    self.prim("K")
                };
                Some((3, self.pair(value, arg!(2))))
            }
            Some(Catch) if args_len >= 3 => {
                let action = self.app(arg!(0), arg!(2));
                Some((3, self.catch_result(action, arg!(1), arg!(2))?))
            }
            Some(CatchR) if args_len >= 3 => {
                Some((3, self.catch_result(arg!(0), arg!(1), arg!(2))?))
            }
            Some(Raise) if args_len >= 1 => return Err(EvalError::Raised(arg!(0))),
            Some(Rnf) if args_len >= 2 => {
                let noerr = self.eval_int(arg!(0))? != 0;
                self.rnf(noerr, arg!(1))?;
                Some((2, self.prim("I")))
            }
            Some(Seq) if args_len >= 2 => Some((2, arg!(1))),
            Some(IsInt) if args_len >= 1 => {
                let root = self.resolve(arg!(0))?;
                let n = self.cell(root).int_value().unwrap_or(-1);
                Some((1, self.int(n)))
            }
            Some(Thnum) if args_len >= 1 => {
                let thread = self.eval_thread_id(arg!(0))?;
                Some((1, self.int(thread)))
            }
            Some(S) if args_len >= 3 => {
                let x = arg!(2);
                let left = self.app(arg!(0), x);
                let right = self.app(arg!(1), x);
                app_step!(3, left, right);
            }
            Some(SPrime) if args_len >= 4 => {
                let yw = self.app(arg!(1), arg!(3));
                let zw = self.app(arg!(2), arg!(3));
                let left = self.app(arg!(0), yw);
                app_step!(4, left, zw);
            }
            Some(B) if args_len >= 3 => {
                let yz = self.app(arg!(1), arg!(2));
                app_step!(3, arg!(0), yz);
            }
            Some(BPrime) if args_len >= 4 => {
                let zw = self.app(arg!(2), arg!(3));
                let xy = self.app(arg!(0), arg!(1));
                app_step!(4, xy, zw);
            }
            Some(BPrime) if args_len >= 2 => {
                let xy = self.app(arg!(0), arg!(1));
                let b = self.prim("B");
                app_step!(2, b, xy);
            }
            Some(Z) if args_len >= 3 => {
                app_step!(3, arg!(0), arg!(1));
            }
            Some(Z) if args_len >= 2 => {
                let xy = self.app(arg!(0), arg!(1));
                let k = self.prim("K");
                app_step!(2, k, xy);
            }
            Some(J) if args_len >= 3 => {
                app_step!(3, arg!(2), arg!(0));
            }
            Some(L) if args_len >= 3 => {
                app_step!(3, arg!(1), arg!(0));
            }
            Some(KK) if args_len >= 3 => Some((3, arg!(1))),
            Some(KA) if args_len >= 3 => Some((3, arg!(2))),
            Some(C) if args_len >= 3 => {
                let xz = self.app(arg!(0), arg!(2));
                app_step!(3, xz, arg!(1));
            }
            Some(CPrime) if args_len >= 4 => {
                let yw = self.app(arg!(1), arg!(3));
                let xyw = self.app(arg!(0), yw);
                app_step!(4, xyw, arg!(2));
            }
            Some(P) if args_len >= 3 => {
                let zx = self.app(arg!(2), arg!(0));
                app_step!(3, zx, arg!(1));
            }
            Some(R) if args_len >= 3 => {
                let yz = self.app(arg!(1), arg!(2));
                app_step!(3, yz, arg!(0));
            }
            Some(R) if args_len >= 2 => {
                let c = self.prim("C");
                let cy = self.app(c, arg!(1));
                app_step!(2, cy, arg!(0));
            }
            Some(O) if args_len >= 4 => {
                let wx = self.app(arg!(3), arg!(0));
                app_step!(4, wx, arg!(1));
            }
            Some(K2) if args_len >= 3 => Some((3, arg!(0))),
            Some(K2) if args_len >= 2 => {
                let k = self.prim("K");
                app_step!(2, k, arg!(0));
            }
            Some(K3) if args_len >= 4 => Some((4, arg!(0))),
            Some(K3) if args_len >= 2 => {
                let k2 = self.prim("K2");
                app_step!(2, k2, arg!(0));
            }
            Some(K4) if args_len >= 5 => Some((5, arg!(0))),
            Some(K4) if args_len >= 2 => {
                let k3 = self.prim("K3");
                app_step!(2, k3, arg!(0));
            }
            Some(CPrimeB) if args_len >= 4 => {
                let yw = self.app(arg!(1), arg!(3));
                let xz = self.app(arg!(0), arg!(2));
                app_step!(4, xz, yw);
            }
            Some(CPrimeB) if args_len >= 3 => {
                let xz = self.app(arg!(0), arg!(2));
                let b = self.prim("B");
                let bxz = self.app(b, xz);
                app_step!(3, bxz, arg!(1));
            }
            Some(Y) if args_len >= 1 => {
                app_step!(1, arg!(0), spine.app(0));
            }
            Some(Tag(tag)) if args_len >= 2 => {
                let tag = self.int(i64::from(tag));
                let ytag = self.app(arg!(1), tag);
                app_step!(2, ytag, arg!(0));
            }
            Some(Tuple(fields)) if args_len > usize::from(fields) => {
                let fields = usize::from(fields);
                if budget >= 2 {
                    let selector = arg!(fields);
                    let available_extra = args_len - fields - 1;
                    if let Some(extra_used) =
                        self.tuple_first_field_selector_extra(selector, fields, available_extra)?
                    {
                        self.profile_shortcut("tuple_first_field_selector", 1);
                        rewrite_step!(fields + 1 + extra_used, arg!(0), 2);
                    }
                }
                let mut n = arg!(fields);
                for idx in 0..fields - 1 {
                    n = self.app(n, arg!(idx));
                }
                app_step!(fields + 1, n, arg!(fields - 1));
            }
            _ if args_len >= 2 && fallback_name.is_some() => {
                let name = fallback_name.expect("fallback name checked above");
                let materialized_args = args_len.min(FALLBACK_PRIM_ARG_PREFIX);
                if self.profile.is_some() {
                    self.profile_arg_materialization(materialized_args);
                }
                spine.write_args_head_order_prefix(scratch_args, FALLBACK_PRIM_ARG_PREFIX);
                self.fallback_runtime_prim_rewrite(name, scratch_args.as_slice(), args_len)?
            }
            _ if args_len >= 1 && fallback_name.is_some() => {
                let name = fallback_name.expect("fallback name checked above");
                let materialized_args = args_len.min(FALLBACK_PRIM_ARG_PREFIX);
                if self.profile.is_some() {
                    self.profile_arg_materialization(materialized_args);
                }
                spine.write_args_head_order_prefix(scratch_args, FALLBACK_PRIM_ARG_PREFIX);
                self.fallback_runtime_prim_rewrite(name, scratch_args.as_slice(), args_len)?
            }
            _ => None,
        };

        let Some((mut used, mut node)) = rewrite else {
            if let Some(name) = fallback_name {
                if args_len != 0 && !is_supported_runtime_prim_name(name) {
                    return Err(EvalError::UnknownPrim(name.to_owned()));
                }
            }
            return Ok(None);
        };
        let mut reductions = 1;
        if matches!(known, Some(I | Ord | Chr)) {
            let mut alias_shortcuts = 0;
            while reductions < budget && used < args_len && self.is_identity_alias_node(node)? {
                node = arg!(used);
                used += 1;
                reductions += 1;
                alias_shortcuts += 1;
            }
            self.profile_shortcut("identity_alias_chain", alias_shortcuts);
        }
        if profile_head.is_some() {
            self.profile_spine_rewrite(args_len - used);
        }
        let node = self.apply_eval_spine_rewrite(root, spine, used, node);
        Ok(Some(self.eval_loop_result(profile_head, node, reductions)))
    }

    fn fallback_runtime_prim_rewrite(
        &mut self,
        name: &str,
        args: &[NodeId],
        args_len: usize,
    ) -> Result<Option<(usize, NodeId)>, EvalError> {
        macro_rules! dispatch_helper {
            ($key:literal, $expr:expr) => {{
                self.profile_primitive_dispatch_probe($key);
                let result = $expr?;
                if result.is_some() {
                    self.profile_primitive_dispatch_hit($key);
                }
                result
            }};
        }

        if args_len >= 2 {
            return Ok(
                dispatch_helper!("fallback_array_op", self.array_op(name, args))
                    .or(dispatch_helper!(
                        "fallback_foreign_ptr_op",
                        self.foreign_ptr_op(name, args)
                    ))
                    .or(dispatch_helper!(
                        "fallback_stable_ptr_op",
                        self.stable_ptr_op(name, args)
                    ))
                    .or(dispatch_helper!(
                        "fallback_weak_ptr_op",
                        self.weak_ptr_op(name, args)
                    ))
                    .or(dispatch_helper!(
                        "fallback_bytes_op",
                        self.bytes_op(name, args)
                    ))
                    .or(dispatch_helper!(
                        "fallback_float64_binop",
                        self.float64_binop(name, args)
                    ))
                    .or(dispatch_helper!(
                        "fallback_float32_binop",
                        self.float32_binop(name, args)
                    ))
                    .or(dispatch_helper!(
                        "fallback_int64_binop",
                        self.int64_binop(name, args)
                    ))
                    .or(dispatch_helper!(
                        "fallback_int_binop",
                        self.int_binop(name, args)
                    ))
                    .or(dispatch_helper!(
                        "fallback_array_unop",
                        self.array_unop(name, args)
                    ))
                    .or(dispatch_helper!(
                        "fallback_bytes_unop",
                        self.bytes_unop(name, args)
                    ))
                    .or(dispatch_helper!(
                        "fallback_float64_unop",
                        self.float64_unop(name, args)
                    ))
                    .or(dispatch_helper!(
                        "fallback_float32_unop",
                        self.float32_unop(name, args)
                    ))
                    .or(dispatch_helper!(
                        "fallback_pointer_conversion",
                        self.pointer_conversion(name, args)
                    ))
                    .or(dispatch_helper!(
                        "fallback_float_conversion",
                        self.float_conversion(name, args)
                    ))
                    .or(dispatch_helper!(
                        "fallback_int64_unop",
                        self.int64_unop(name, args)
                    ))
                    .or(dispatch_helper!(
                        "fallback_int_conversion",
                        self.int_conversion(name, args)
                    ))
                    .or(dispatch_helper!(
                        "fallback_int_unop",
                        self.int_unop(name, args)
                    )),
            );
        }

        if args_len >= 1 {
            return Ok(
                dispatch_helper!("fallback_array_unop", self.array_unop(name, args))
                    .or(dispatch_helper!(
                        "fallback_foreign_ptr_unop",
                        self.foreign_ptr_unop(name, args)
                    ))
                    .or(dispatch_helper!(
                        "fallback_stable_ptr_unop",
                        self.stable_ptr_unop(name, args)
                    ))
                    .or(dispatch_helper!(
                        "fallback_weak_ptr_unop",
                        self.weak_ptr_unop(name, args)
                    ))
                    .or(dispatch_helper!(
                        "fallback_bytes_unop",
                        self.bytes_unop(name, args)
                    ))
                    .or(dispatch_helper!(
                        "fallback_float64_unop",
                        self.float64_unop(name, args)
                    ))
                    .or(dispatch_helper!(
                        "fallback_float32_unop",
                        self.float32_unop(name, args)
                    ))
                    .or(dispatch_helper!(
                        "fallback_pointer_conversion",
                        self.pointer_conversion(name, args)
                    ))
                    .or(dispatch_helper!(
                        "fallback_float_conversion",
                        self.float_conversion(name, args)
                    ))
                    .or(dispatch_helper!(
                        "fallback_int64_unop",
                        self.int64_unop(name, args)
                    ))
                    .or(dispatch_helper!(
                        "fallback_int_conversion",
                        self.int_conversion(name, args)
                    ))
                    .or(dispatch_helper!(
                        "fallback_int_unop",
                        self.int_unop(name, args)
                    )),
            );
        }

        Ok(None)
    }

    fn fill_persistent_spine(
        &mut self,
        mut node: NodeId,
        spine: &mut PersistentSpine,
        profile_resolve: bool,
    ) -> Result<NodeId, EvalError> {
        while let Some((fun, _)) = self.cell(node).app_fields() {
            spine.push_front(node);
            node = self.resolve_for_whnf(fun, profile_resolve)?;
        }
        Ok(node)
    }

    fn persistent_eval_step(
        &mut self,
        head: NodeId,
        spine: &mut PersistentSpine,
        frame_stack: &mut EvalFrameStack,
        scratch_args: &mut Vec<NodeId>,
        budget: usize,
    ) -> Result<PersistentStep, EvalError> {
        let args_len = spine.len();
        let profile_head = if self.profile.is_some() {
            self.profile_step(head, args_len, false)
        } else {
            None
        };

        macro_rules! arg {
            ($idx:expr) => {
                spine.arg(&self.nodes, $idx)?
            };
        }
        macro_rules! app_site {
            ($key:literal, $fun:expr, $arg:expr) => {
                self.app_with_site($key, $fun, $arg)
            };
        }
        macro_rules! finish_reduction {
            ($node:expr, $reductions:expr) => {{
                if profile_head.is_some() {
                    self.profile_reduction(profile_head, $reductions);
                }
                return Ok(PersistentStep::Reduced {
                    node: $node,
                    reductions: $reductions,
                });
            }};
        }
        macro_rules! rewrite_step {
            ($used:expr, $node:expr, $reductions:expr) => {{
                let node = $node;
                if $used > 0 {
                    let redex = spine.app($used - 1);
                    if node != redex {
                        self.set_app_cell_at(redex.index(), Cell::indir(Some(node)));
                    }
                    if $used < spine.len() {
                        if self.profile.is_some() {
                            self.profile_remaining_app_scan(spine.len() - $used);
                        }
                    }
                    if $used < spine.len() && !spine.remaining_apps_contain($used, node) {
                        let app = spine.app($used);
                        let arg = spine.arg(&self.nodes, $used)?;
                        self.set_app_cell_at(app.index(), Cell::app(node, arg));
                    }
                }
                spine.consume($used);
                finish_reduction!(node, $reductions);
            }};
        }
        macro_rules! app_step_reductions {
            ($used:expr, $fun:expr, $arg:expr, $reductions:expr) => {{
                let fun = $fun;
                let arg = $arg;
                let node = if $used > 0 {
                    let redex = spine.app($used - 1);
                    self.set_app_cell_at(redex.index(), Cell::app(fun, arg));
                    redex
                } else {
                    app_site!("persistent_app_step_result", fun, arg)
                };
                spine.consume($used);
                finish_reduction!(node, $reductions);
            }};
        }
        macro_rules! app_step {
            ($used:expr, $fun:expr, $arg:expr) => {{
                app_step_reductions!($used, $fun, $arg, 1);
            }};
        }
        macro_rules! force_step {
            ($used:expr, $variant:ident, $frame:ident, $kind:expr, $next:expr) => {{
                let redex_root = spine.outer_root(head);
                let kind = $kind;
                let next = $next;
                let redex = self.strict_redex_from_persistent_spine(redex_root, $used, spine);
                if self.profile.is_some() {
                    self.profile_persistent_force();
                    self.profile_eval_frame_push(stringify!($variant));
                }
                frame_stack.push(EvalFrame::$variant($frame {
                    redex,
                    profile_head,
                    kind,
                }));
                return Ok(PersistentStep::Force { node: next });
            }};
        }
        macro_rules! force_int64_shift_step {
            ($used:expr, $op:expr, $x:expr, $next:expr) => {{
                let redex_root = spine.outer_root(head);
                let x = $x;
                let next = $next;
                let redex = self.strict_redex_from_persistent_spine(redex_root, $used, spine);
                if self.profile.is_some() {
                    self.profile_persistent_force();
                    self.profile_eval_frame_push("Int64Shift");
                }
                frame_stack.push(EvalFrame::Int64Shift(Int64ShiftFrame {
                    redex,
                    profile_head,
                    op: $op,
                    x,
                }));
                return Ok(PersistentStep::Force { node: next });
            }};
        }

        let head_dispatch = match self.cell(head).prim() {
            Some(Prim::Known(known)) => PersistentHead::Known(known),
            Some(Prim::Runtime(runtime)) => PersistentHead::Other(runtime.strict_action(args_len)),
            None if args_len > 0 => match self.cold_node(head) {
                Some(Node::Ffi(name)) => PersistentHead::Ffi(name.to_string()),
                Some(Node::JsCall(call)) => PersistentHead::JsCall {
                    tags: call.tags.clone(),
                    body: call.body.clone(),
                },
                Some(Node::JsWrap { tags }) => PersistentHead::JsWrap {
                    tags: tags.to_string(),
                },
                _ => PersistentHead::Whnf,
            },
            None => PersistentHead::Whnf,
        };

        let known = match head_dispatch {
            PersistentHead::Ffi(name) => {
                if self.profile.is_some() {
                    self.profile_arg_materialization(args_len);
                }
                spine.write_args_head_order(&self.nodes, scratch_args)?;
                let Some((used, node)) = self.ffi_call(&name, scratch_args.as_slice())? else {
                    return Ok(PersistentStep::Whnf {
                        node: spine.outer_root(head),
                    });
                };
                rewrite_step!(used, node, 1);
            }
            PersistentHead::JsCall { tags, body } => {
                if self.profile.is_some() {
                    self.profile_arg_materialization(args_len);
                }
                spine.write_args_head_order(&self.nodes, scratch_args)?;
                let Some((used, node)) = self.js_call(&tags, &body, scratch_args.as_slice())?
                else {
                    return Ok(PersistentStep::Whnf {
                        node: spine.outer_root(head),
                    });
                };
                rewrite_step!(used, node, 1);
            }
            PersistentHead::JsWrap { tags } => {
                if self.profile.is_some() {
                    self.profile_arg_materialization(args_len);
                }
                spine.write_args_head_order(&self.nodes, scratch_args)?;
                let Some((used, node)) = self.js_wrap(&tags, scratch_args.as_slice())? else {
                    return Ok(PersistentStep::Whnf {
                        node: spine.outer_root(head),
                    });
                };
                rewrite_step!(used, node, 1);
            }
            PersistentHead::Known(known) => known,
            PersistentHead::Other(action) => {
                if self.profile.is_some() {
                    self.profile_strict_primitive_dispatch(args_len, action);
                }
                match action {
                    StrictPrimitiveAction::IntBin(op) => {
                        let x = arg!(0);
                        force_step!(2, Int, IntFrame, IntFrameKind::BinSecond { op, x }, arg!(1));
                    }
                    StrictPrimitiveAction::IntUn(op) => {
                        force_step!(1, Int, IntFrame, IntFrameKind::Un { op }, arg!(0));
                    }
                    StrictPrimitiveAction::Int64Bin(op) => {
                        if op.rhs_is_shift() {
                            force_int64_shift_step!(2, op, arg!(0), arg!(1));
                        } else if op.driver_marker_safe() {
                            force_step!(
                                2,
                                Int64,
                                Int64Frame,
                                Int64FrameKind::BinSecond { op, x: arg!(0) },
                                arg!(1)
                            );
                        }
                    }
                    StrictPrimitiveAction::Int64Un(op) => {
                        force_step!(1, Int64, Int64Frame, Int64FrameKind::Un { op }, arg!(0));
                    }
                    StrictPrimitiveAction::Float64Bin(op) => {
                        force_step!(
                            2,
                            Float64,
                            Float64Frame,
                            Float64FrameKind::BinSecond { op, x: arg!(0) },
                            arg!(1)
                        );
                    }
                    StrictPrimitiveAction::Float64Un(op) => {
                        force_step!(
                            1,
                            Float64,
                            Float64Frame,
                            Float64FrameKind::Un { op },
                            arg!(0)
                        );
                    }
                    StrictPrimitiveAction::Float32Bin(op) => {
                        force_step!(
                            2,
                            Float32,
                            Float32Frame,
                            Float32FrameKind::BinSecond { op, x: arg!(0) },
                            arg!(1)
                        );
                    }
                    StrictPrimitiveAction::Float32Un(op) => {
                        force_step!(
                            1,
                            Float32,
                            Float32Frame,
                            Float32FrameKind::Un { op },
                            arg!(0)
                        );
                    }
                    StrictPrimitiveAction::BytesBin(op) => {
                        force_step!(
                            2,
                            Bytes,
                            BytesFrame,
                            BytesFrameKind::BinSecond { op, x: arg!(0) },
                            arg!(1)
                        );
                    }
                    StrictPrimitiveAction::Conversion(kind) => {
                        force_step!(1, Conversion, ConversionFrame, kind, arg!(0));
                    }
                    StrictPrimitiveAction::None => {
                        return Ok(PersistentStep::Fallback {
                            root: spine.outer_root(head),
                        });
                    }
                }
                return Ok(PersistentStep::Fallback {
                    root: spine.outer_root(head),
                });
            }
            PersistentHead::Whnf => {
                return Ok(PersistentStep::Whnf {
                    node: spine.outer_root(head),
                });
            }
        };
        use KnownPrim::*;

        match known {
            IoStrict if args_len >= 2 => {
                force_step!(
                    2,
                    Whnf,
                    WhnfFrame,
                    WhnfFrameKind::IoStrict {
                        action: arg!(0),
                        value: arg!(1),
                    },
                    arg!(1)
                );
            }
            Seq if args_len >= 2 => {
                force_step!(
                    2,
                    Whnf,
                    WhnfFrame,
                    WhnfFrameKind::Seq { result: arg!(1) },
                    arg!(0)
                );
            }
            IsInt if args_len >= 1 => {
                force_step!(1, Whnf, WhnfFrame, WhnfFrameKind::IsInt, arg!(0));
            }
            _ => {}
        }

        match known {
            IoPerformIo if args_len >= 1 => {
                let world = self.world();
                let k = self.prim("K");
                let io = arg!(0);
                let action = app_site!("IO.performIO.action", io, world);
                app_step!(1, action, k);
            }
            IoBind if args_len >= 3 => {
                let io = arg!(0);
                let k = arg!(1);
                let world = arg!(2);
                let action = app_site!("IO.bind.action", io, world);
                app_step!(3, action, k);
            }
            IoThen if args_len >= 3 && budget >= 2 => {
                let k = self.prim("K");
                let io = arg!(0);
                let y = arg!(1);
                let world = arg!(2);
                let then = app_site!("IO.then.k", k, y);
                let action = app_site!("IO.then.action", io, world);
                app_step_reductions!(3, action, then, 2);
            }
            IoThen if args_len >= 2 => {
                let bind = self.prim("IO.>>=");
                let io = arg!(0);
                let y = arg!(1);
                let bind_action = app_site!("IO.then.bind_action", bind, io);
                let k = self.prim("K");
                let then = app_site!("IO.then.k", k, y);
                app_step!(2, bind_action, then);
            }
            IoReturn if args_len >= 3 => {
                let x = arg!(0);
                let world = arg!(1);
                let k = arg!(2);
                let kx = app_site!("IO.return.kx", k, x);
                app_step!(3, kx, world);
            }
            I | Ord | Chr if args_len >= 1 => {
                let mut used = 1;
                let mut reductions = 1;
                let mut node = arg!(0);
                let mut alias_shortcuts = 0;
                while reductions < budget && used < args_len && self.is_identity_alias_node(node)? {
                    node = arg!(used);
                    used += 1;
                    reductions += 1;
                    alias_shortcuts += 1;
                }
                self.profile_shortcut("identity_alias_chain", alias_shortcuts);
                rewrite_step!(used, node, reductions);
            }
            K if args_len >= 2 => rewrite_step!(2, arg!(0), 1),
            A if args_len >= 2 => rewrite_step!(2, arg!(1), 1),
            U if args_len >= 2 => {
                app_step!(2, arg!(1), arg!(0));
            }
            S if args_len >= 3 => {
                let x = arg!(2);
                let left = app_site!("S.left", arg!(0), x);
                let right = app_site!("S.right", arg!(1), x);
                app_step!(3, left, right);
            }
            SPrime if args_len >= 4 => {
                let yw = app_site!("S'.yw", arg!(1), arg!(3));
                let zw = app_site!("S'.zw", arg!(2), arg!(3));
                let left = app_site!("S'.left", arg!(0), yw);
                app_step!(4, left, zw);
            }
            B if args_len >= 3 => {
                let yz = app_site!("B.yz", arg!(1), arg!(2));
                app_step!(3, arg!(0), yz);
            }
            BPrime if args_len >= 4 => {
                let zw = app_site!("B'.zw", arg!(2), arg!(3));
                let xy = app_site!("B'.xy", arg!(0), arg!(1));
                app_step!(4, xy, zw);
            }
            BPrime if args_len >= 2 => {
                let xy = app_site!("B'.xy_under", arg!(0), arg!(1));
                let b = self.prim("B");
                app_step!(2, b, xy);
            }
            Z if args_len >= 3 => {
                app_step!(3, arg!(0), arg!(1));
            }
            Z if args_len >= 2 => {
                let xy = app_site!("Z.xy_under", arg!(0), arg!(1));
                let k = self.prim("K");
                app_step!(2, k, xy);
            }
            J if args_len >= 3 => {
                app_step!(3, arg!(2), arg!(0));
            }
            L if args_len >= 3 => {
                app_step!(3, arg!(1), arg!(0));
            }
            KK if args_len >= 3 => rewrite_step!(3, arg!(1), 1),
            KA if args_len >= 3 => rewrite_step!(3, arg!(2), 1),
            C if args_len >= 3 => {
                let xz = app_site!("C.xz", arg!(0), arg!(2));
                app_step!(3, xz, arg!(1));
            }
            CPrime if args_len >= 4 => {
                let yw = app_site!("C'.yw", arg!(1), arg!(3));
                let xyw = app_site!("C'.xyw", arg!(0), yw);
                app_step!(4, xyw, arg!(2));
            }
            P if args_len >= 3 => {
                let zx = app_site!("P.zx", arg!(2), arg!(0));
                app_step!(3, zx, arg!(1));
            }
            R if args_len >= 3 => {
                let yz = app_site!("R.yz", arg!(1), arg!(2));
                app_step!(3, yz, arg!(0));
            }
            R if args_len >= 2 => {
                let c = self.prim("C");
                let cy = app_site!("R.cy_under", c, arg!(1));
                app_step!(2, cy, arg!(0));
            }
            O if args_len >= 4 => {
                let wx = app_site!("O.wx", arg!(3), arg!(0));
                app_step!(4, wx, arg!(1));
            }
            K2 if args_len >= 3 => rewrite_step!(3, arg!(0), 1),
            K2 if args_len >= 2 => {
                let k = self.prim("K");
                app_step!(2, k, arg!(0));
            }
            K3 if args_len >= 4 => rewrite_step!(4, arg!(0), 1),
            K3 if args_len >= 2 => {
                let k2 = self.prim("K2");
                app_step!(2, k2, arg!(0));
            }
            K4 if args_len >= 5 => rewrite_step!(5, arg!(0), 1),
            K4 if args_len >= 2 => {
                let k3 = self.prim("K3");
                app_step!(2, k3, arg!(0));
            }
            CPrimeB if args_len >= 4 => {
                let yw = app_site!("C'B.yw", arg!(1), arg!(3));
                let xz = app_site!("C'B.xz", arg!(0), arg!(2));
                app_step!(4, xz, yw);
            }
            CPrimeB if args_len >= 3 => {
                let xz = app_site!("C'B.xz_under", arg!(0), arg!(2));
                let b = self.prim("B");
                let bxz = app_site!("C'B.bxz_under", b, xz);
                app_step!(3, bxz, arg!(1));
            }
            Y if args_len >= 1 => {
                app_step!(1, arg!(0), spine.app(0));
            }
            Tag(tag) if args_len >= 2 => {
                let tag = self.int(i64::from(tag));
                let ytag = app_site!("Tag.ytag", arg!(1), tag);
                app_step!(2, ytag, arg!(0));
            }
            Tuple(fields) if args_len > usize::from(fields) => {
                let fields = usize::from(fields);
                let mut n = arg!(fields);
                for idx in 0..fields - 1 {
                    let arg = arg!(idx);
                    n = app_site!("Tuple.prefix", n, arg);
                }
                let last = arg!(fields - 1);
                app_step!(fields + 1, n, last);
            }
            I | Ord | Chr | K | A | U | S | SPrime | B | BPrime | Z | J | L | KK | KA | C
            | CPrime | P | R | O | K2 | K3 | K4 | CPrimeB | Y | Tag(_) | Tuple(_) | IoPerformIo
            | IoBind | IoThen | IoReturn => Ok(PersistentStep::Whnf {
                node: spine.outer_root(head),
            }),
            _ => Ok(PersistentStep::Fallback {
                root: spine.outer_root(head),
            }),
        }
    }

    fn stack_eval_step(
        &mut self,
        mut head: NodeId,
        stack: &mut EvalStack,
        scratch_args: &mut Vec<NodeId>,
        budget: usize,
        profile_resolve: bool,
    ) -> Result<StackStep, EvalError> {
        let mut carried_reductions = 0;
        'eval: loop {
            let profiling = self.profile.is_some();
            if stack.app_len() == 0 {
                if let Some((next, reductions)) = self.finish_ready_stack_frame(stack, head)? {
                    carried_reductions += reductions;
                    if carried_reductions >= budget {
                        return Ok(StackStep::Reduced {
                            node: next,
                            reductions: carried_reductions,
                        });
                    }
                    #[cfg(feature = "eval-phase-profile")]
                    let started = profiling.then(Instant::now);
                    head = self.descend_stack_from(next, stack, profile_resolve, profiling)?;
                    #[cfg(feature = "eval-phase-profile")]
                    if let Some(started) = started {
                        self.profile_stack_inner_descent_time(started.elapsed().as_nanos());
                    }
                    continue 'eval;
                }
            }

            let args_len = stack.app_len();
            let profile_head = if profiling {
                self.profile_step(head, args_len, false)
            } else {
                None
            };
            #[cfg(feature = "eval-phase-profile")]
            let stack_eval_step_head_started = profiling.then(Instant::now);

            macro_rules! arg {
                ($idx:expr) => {{
                    if profiling {
                        self.profile_stack_arg_reads(1);
                    }
                    #[cfg(feature = "eval-phase-profile")]
                    let started = profiling.then(Instant::now);
                    let arg = stack.arg(&self.nodes, $idx);
                    #[cfg(feature = "eval-phase-profile")]
                    if let Some(started) = started {
                        let nanos = started.elapsed().as_nanos();
                        self.profile_stack_arg_read_time(nanos);
                        self.profile_stack_arg_read_head_time(profile_head, nanos);
                    }
                    arg
                }};
            }
            macro_rules! take_args {
                ($reads:expr, $method:ident) => {{
                    if profiling {
                        self.profile_stack_arg_batch();
                        self.profile_stack_arg_reads($reads);
                    }
                    #[cfg(feature = "eval-phase-profile")]
                    let started = profiling.then(Instant::now);
                    let args = stack.$method(&self.nodes);
                    #[cfg(feature = "eval-phase-profile")]
                    if let Some(started) = started {
                        let nanos = started.elapsed().as_nanos();
                        self.profile_stack_arg_read_time(nanos);
                        self.profile_stack_arg_read_head_time(profile_head, nanos);
                    }
                    args
                }};
            }
            macro_rules! app_site {
                ($key:literal, $fun:expr, $arg:expr) => {
                    self.app_with_site($key, $fun, $arg)
                };
            }
            macro_rules! record_stack_head_time {
                () => {{
                    #[cfg(feature = "eval-phase-profile")]
                    {
                        if let Some(started) = stack_eval_step_head_started {
                            self.profile_stack_eval_step_head_time(
                                profile_head,
                                started.elapsed().as_nanos(),
                            );
                        }
                    }
                }};
            }
            macro_rules! finish_reduction {
                ($node:expr, $reductions:expr) => {{
                    record_stack_head_time!();
                    let reductions = carried_reductions + $reductions;
                    if profile_head.is_some() {
                        self.profile_reduction(profile_head, $reductions);
                    }
                    return Ok(StackStep::Reduced {
                        node: $node,
                        reductions,
                    });
                }};
            }
            macro_rules! rewrite_step {
                ($used:expr, $node:expr, $reductions:expr) => {{
                    #[cfg(feature = "eval-phase-profile")]
                    let started = profiling.then(Instant::now);
                    let node = self.apply_stack_rewrite(stack, $used, $node);
                    #[cfg(feature = "eval-phase-profile")]
                    if let Some(started) = started {
                        self.profile_stack_apply_rewrite_head_time(
                            profile_head,
                            started.elapsed().as_nanos(),
                        );
                    }
                    finish_reduction!(node, $reductions);
                }};
            }
            macro_rules! rewrite_continue_reductions {
                ($used:expr, $node:expr, $reductions:expr) => {{
                    #[cfg(feature = "eval-phase-profile")]
                    let started = profiling.then(Instant::now);
                    let node = self.apply_stack_rewrite(stack, $used, $node);
                    #[cfg(feature = "eval-phase-profile")]
                    if let Some(started) = started {
                        self.profile_stack_apply_rewrite_head_time(
                            profile_head,
                            started.elapsed().as_nanos(),
                        );
                    }
                    record_stack_head_time!();
                    if profile_head.is_some() {
                        self.profile_reduction(profile_head, $reductions);
                    }
                    carried_reductions += $reductions;
                    if carried_reductions >= budget {
                        return Ok(StackStep::Reduced {
                            node,
                            reductions: carried_reductions,
                        });
                    }
                    continue_with!(node);
                }};
            }
            macro_rules! continue_with {
                ($node:expr) => {{
                    #[cfg(feature = "eval-phase-profile")]
                    let started = profiling.then(Instant::now);
                    head = self.descend_stack_from($node, stack, profile_resolve, profiling)?;
                    #[cfg(feature = "eval-phase-profile")]
                    if let Some(started) = started {
                        let nanos = started.elapsed().as_nanos();
                        self.profile_stack_inner_descent_time(nanos);
                        self.profile_stack_inner_descent_head_time(profile_head, nanos);
                    }
                    continue 'eval;
                }};
            }
            macro_rules! goind_taken {
                ($redex:expr, $used:expr, $node:expr, $reductions:expr) => {{
                    #[cfg(feature = "eval-phase-profile")]
                    let started = profiling.then(Instant::now);
                    let redex = $redex;
                    let node = $node;
                    let wrote_indirection = node != redex;
                    if wrote_indirection {
                        self.set_app_cell_at(redex.index(), Cell::indir(Some(node)));
                    }
                    if profiling {
                        self.profile_stack_rewrite($used, wrote_indirection);
                    }
                    #[cfg(feature = "eval-phase-profile")]
                    if let Some(started) = started {
                        let nanos = started.elapsed().as_nanos();
                        self.profile_stack_apply_rewrite_time(nanos);
                        self.profile_stack_apply_rewrite_head_time(profile_head, nanos);
                    }
                    record_stack_head_time!();
                    if profile_head.is_some() {
                        self.profile_reduction(profile_head, $reductions);
                    }
                    carried_reductions += $reductions;
                    if carried_reductions >= budget {
                        return Ok(StackStep::Reduced {
                            node,
                            reductions: carried_reductions,
                        });
                    }
                    continue_with!(node);
                }};
            }
            macro_rules! app_step_reductions {
                ($used:expr, $fun:expr, $arg:expr, $reductions:expr) => {{
                    let fun = $fun;
                    let arg = $arg;
                    #[cfg(feature = "eval-phase-profile")]
                    let update_started = profiling.then(Instant::now);
                    let node = self.apply_stack_app(stack, $used, fun, arg);
                    #[cfg(feature = "eval-phase-profile")]
                    if let Some(started) = update_started {
                        self.profile_stack_apply_app_head_time(
                            profile_head,
                            started.elapsed().as_nanos(),
                        );
                    }
                    record_stack_head_time!();
                    if profile_head.is_some() {
                        self.profile_reduction(profile_head, $reductions);
                    }
                    carried_reductions += $reductions;
                    if carried_reductions >= budget {
                        return Ok(StackStep::Reduced {
                            node,
                            reductions: carried_reductions,
                        });
                    }
                    if profiling {
                        self.profile_stack_descent_push();
                    }
                    stack.push_app(node);
                    #[cfg(feature = "eval-phase-profile")]
                    let started = profiling.then(Instant::now);
                    head = self.descend_stack_from(fun, stack, profile_resolve, profiling)?;
                    #[cfg(feature = "eval-phase-profile")]
                    if let Some(started) = started {
                        let nanos = started.elapsed().as_nanos();
                        self.profile_stack_inner_descent_time(nanos);
                        self.profile_stack_inner_descent_head_time(profile_head, nanos);
                    }
                    continue 'eval;
                }};
            }
            macro_rules! app_taken_reductions {
                ($redex:expr, $used:expr, $fun:expr, $arg:expr, $reductions:expr) => {{
                    let redex = $redex;
                    let fun = $fun;
                    let arg = $arg;
                    #[cfg(feature = "eval-phase-profile")]
                    let started = profiling.then(Instant::now);
                    if profiling {
                        self.profile_stack_app_update($used);
                    }
                    self.set_app_cell_at(redex.index(), Cell::app(fun, arg));
                    #[cfg(feature = "eval-phase-profile")]
                    if let Some(started) = started {
                        let nanos = started.elapsed().as_nanos();
                        self.profile_stack_apply_app_time(nanos);
                        self.profile_stack_apply_app_head_time(profile_head, nanos);
                    }
                    record_stack_head_time!();
                    if profile_head.is_some() {
                        self.profile_reduction(profile_head, $reductions);
                    }
                    carried_reductions += $reductions;
                    if carried_reductions >= budget {
                        return Ok(StackStep::Reduced {
                            node: redex,
                            reductions: carried_reductions,
                        });
                    }
                    if profiling {
                        self.profile_stack_descent_push();
                    }
                    stack.push_app(redex);
                    #[cfg(feature = "eval-phase-profile")]
                    let started = profiling.then(Instant::now);
                    head = self.descend_stack_from(fun, stack, profile_resolve, profiling)?;
                    #[cfg(feature = "eval-phase-profile")]
                    if let Some(started) = started {
                        let nanos = started.elapsed().as_nanos();
                        self.profile_stack_inner_descent_time(nanos);
                        self.profile_stack_inner_descent_head_time(profile_head, nanos);
                    }
                    continue 'eval;
                }};
            }
            macro_rules! app_step {
                ($used:expr, $fun:expr, $arg:expr) => {{
                    app_step_reductions!($used, $fun, $arg, 1);
                }};
            }
            macro_rules! app_taken {
                ($redex:expr, $used:expr, $fun:expr, $arg:expr) => {{
                    app_taken_reductions!($redex, $used, $fun, $arg, 1);
                }};
            }
            macro_rules! force_step {
                ($used:expr, $push:ident, $kind:expr, $next:expr, $profile_kind:literal) => {{
                    let app_end = stack.apps.len();
                    let kind = $kind;
                    let next = $next;
                    if profiling {
                        self.profile_persistent_force();
                        self.profile_eval_frame_push($profile_kind);
                    }
                    record_stack_head_time!();
                    #[cfg(feature = "eval-phase-profile")]
                    let started = profiling.then(Instant::now);
                    stack.$push(app_end, $used, profile_head, kind);
                    #[cfg(feature = "eval-phase-profile")]
                    if let Some(started) = started {
                        let nanos = started.elapsed().as_nanos();
                        self.profile_stack_force_frame_time(nanos);
                        self.profile_stack_force_frame_head_time(profile_head, nanos);
                    }
                    continue_with!(next);
                }};
            }

            let head_dispatch = match self.cell(head).prim() {
                Some(Prim::Known(known)) => EvalHead::Known(known),
                Some(Prim::Runtime(runtime)) => {
                    let action = runtime.strict_action(args_len);
                    EvalHead::Other {
                        action,
                        fallback_name: matches!(action, StrictPrimitiveAction::None)
                            .then_some(runtime.name()),
                    }
                }
                None if args_len > 0 => match self.cold_node(head) {
                    Some(Node::Ffi(name)) => EvalHead::Ffi(name.to_string()),
                    Some(Node::JsCall(call)) => EvalHead::JsCall {
                        tags: call.tags.clone(),
                        body: call.body.clone(),
                    },
                    Some(Node::JsWrap { tags }) => EvalHead::JsWrap {
                        tags: tags.to_string(),
                    },
                    _ => EvalHead::Whnf,
                },
                None => EvalHead::Whnf,
            };

            let known = match head_dispatch {
                EvalHead::Ffi(name) => {
                    if self.profile.is_some() {
                        self.profile_arg_materialization(args_len);
                    }
                    stack.write_args_head_order(&self.nodes, scratch_args)?;
                    let Some((used, node)) = self.ffi_call(&name, scratch_args.as_slice())? else {
                        record_stack_head_time!();
                        return Ok(StackStep::Whnf {
                            node: stack.outer_root(head),
                            head,
                            reductions: carried_reductions,
                        });
                    };
                    rewrite_step!(used, node, 1);
                }
                EvalHead::JsCall { tags, body } => {
                    if self.profile.is_some() {
                        self.profile_arg_materialization(args_len);
                    }
                    stack.write_args_head_order(&self.nodes, scratch_args)?;
                    let Some((used, node)) = self.js_call(&tags, &body, scratch_args.as_slice())?
                    else {
                        record_stack_head_time!();
                        return Ok(StackStep::Whnf {
                            node: stack.outer_root(head),
                            head,
                            reductions: carried_reductions,
                        });
                    };
                    rewrite_step!(used, node, 1);
                }
                EvalHead::JsWrap { tags } => {
                    if self.profile.is_some() {
                        self.profile_arg_materialization(args_len);
                    }
                    stack.write_args_head_order(&self.nodes, scratch_args)?;
                    let Some((used, node)) = self.js_wrap(&tags, scratch_args.as_slice())? else {
                        record_stack_head_time!();
                        return Ok(StackStep::Whnf {
                            node: stack.outer_root(head),
                            head,
                            reductions: carried_reductions,
                        });
                    };
                    rewrite_step!(used, node, 1);
                }
                EvalHead::Known(known) => known,
                EvalHead::Other {
                    action,
                    fallback_name,
                } => {
                    if profiling {
                        self.profile_strict_primitive_dispatch(args_len, action);
                    }
                    match action {
                        StrictPrimitiveAction::IntBin(op) => {
                            let (redex, x, y) = take_args!(2, take_args2);
                            let y_immediate = self.cell(y).int_value();
                            if let Some(y_value) = y_immediate {
                                if let Some(x_value) = self.cell(x).int_value() {
                                    let result = op
                                        .apply(x_value, y_value)
                                        .map_err(|err| self.arithmetic_eval_error(err))?;
                                    let node = self.apply_stack_redex_value(
                                        redex,
                                        2,
                                        Self::int_result_value_node(result),
                                    );
                                    finish_reduction!(node, 1);
                                }
                                if profiling {
                                    self.profile_persistent_force();
                                    self.profile_eval_frame_push("Int");
                                }
                                record_stack_head_time!();
                                #[cfg(feature = "eval-phase-profile")]
                                let started = profiling.then(Instant::now);
                                stack.push_int_frame(
                                    redex,
                                    profile_head,
                                    IntFrameKind::BinFirst { op, y: y_value },
                                );
                                #[cfg(feature = "eval-phase-profile")]
                                if let Some(started) = started {
                                    let nanos = started.elapsed().as_nanos();
                                    self.profile_stack_force_frame_time(nanos);
                                    self.profile_stack_force_frame_head_time(profile_head, nanos);
                                }
                                continue_with!(x);
                            }
                            if profiling {
                                self.profile_persistent_force();
                                self.profile_eval_frame_push("Int");
                            }
                            record_stack_head_time!();
                            #[cfg(feature = "eval-phase-profile")]
                            let started = profiling.then(Instant::now);
                            stack.push_int_frame(
                                redex,
                                profile_head,
                                IntFrameKind::BinSecond { op, x },
                            );
                            #[cfg(feature = "eval-phase-profile")]
                            if let Some(started) = started {
                                let nanos = started.elapsed().as_nanos();
                                self.profile_stack_force_frame_time(nanos);
                                self.profile_stack_force_frame_head_time(profile_head, nanos);
                            }
                            continue_with!(y);
                        }
                        StrictPrimitiveAction::IntUn(op) => {
                            let (redex, x) = take_args!(1, take_args1);
                            if profiling {
                                self.profile_persistent_force();
                                self.profile_eval_frame_push("Int");
                            }
                            record_stack_head_time!();
                            #[cfg(feature = "eval-phase-profile")]
                            let started = profiling.then(Instant::now);
                            stack.push_int_frame(redex, profile_head, IntFrameKind::Un { op });
                            #[cfg(feature = "eval-phase-profile")]
                            if let Some(started) = started {
                                let nanos = started.elapsed().as_nanos();
                                self.profile_stack_force_frame_time(nanos);
                                self.profile_stack_force_frame_head_time(profile_head, nanos);
                            }
                            continue_with!(x);
                        }
                        StrictPrimitiveAction::Int64Bin(op) => {
                            if op.rhs_is_shift() {
                                let app_end = stack.apps.len();
                                let x = arg!(0);
                                let next = arg!(1);
                                if profiling {
                                    self.profile_persistent_force();
                                    self.profile_eval_frame_push("Int64Shift");
                                }
                                record_stack_head_time!();
                                #[cfg(feature = "eval-phase-profile")]
                                let started = profiling.then(Instant::now);
                                stack.push_int64_shift_frame(app_end, 2, profile_head, op, x);
                                #[cfg(feature = "eval-phase-profile")]
                                if let Some(started) = started {
                                    let nanos = started.elapsed().as_nanos();
                                    self.profile_stack_force_frame_time(nanos);
                                    self.profile_stack_force_frame_head_time(profile_head, nanos);
                                }
                                continue_with!(next);
                            } else if op.driver_marker_safe() {
                                force_step!(
                                    2,
                                    push_int64_frame,
                                    Int64FrameKind::BinSecond { op, x: arg!(0) },
                                    arg!(1),
                                    "Int64"
                                );
                            }
                        }
                        StrictPrimitiveAction::Int64Un(op) => {
                            force_step!(
                                1,
                                push_int64_frame,
                                Int64FrameKind::Un { op },
                                arg!(0),
                                "Int64"
                            );
                        }
                        StrictPrimitiveAction::Float64Bin(op) => {
                            force_step!(
                                2,
                                push_float64_frame,
                                Float64FrameKind::BinSecond { op, x: arg!(0) },
                                arg!(1),
                                "Float64"
                            );
                        }
                        StrictPrimitiveAction::Float64Un(op) => {
                            force_step!(
                                1,
                                push_float64_frame,
                                Float64FrameKind::Un { op },
                                arg!(0),
                                "Float64"
                            );
                        }
                        StrictPrimitiveAction::Float32Bin(op) => {
                            force_step!(
                                2,
                                push_float32_frame,
                                Float32FrameKind::BinSecond { op, x: arg!(0) },
                                arg!(1),
                                "Float32"
                            );
                        }
                        StrictPrimitiveAction::Float32Un(op) => {
                            force_step!(
                                1,
                                push_float32_frame,
                                Float32FrameKind::Un { op },
                                arg!(0),
                                "Float32"
                            );
                        }
                        StrictPrimitiveAction::BytesBin(op) => {
                            force_step!(
                                2,
                                push_bytes_frame,
                                BytesFrameKind::BinSecond { op, x: arg!(0) },
                                arg!(1),
                                "Bytes"
                            );
                        }
                        StrictPrimitiveAction::Conversion(kind) => {
                            force_step!(1, push_conversion_frame, kind, arg!(0), "Conversion");
                        }
                        StrictPrimitiveAction::None => {}
                    }
                    if let Some(name) = fallback_name {
                        let materialized_args = args_len.min(FALLBACK_PRIM_ARG_PREFIX);
                        if self.profile.is_some() {
                            self.profile_arg_materialization(materialized_args);
                        }
                        stack.write_args_head_order_prefix(
                            &self.nodes,
                            scratch_args,
                            FALLBACK_PRIM_ARG_PREFIX,
                        )?;
                        if let Some((used, node)) = self.fallback_runtime_prim_rewrite(
                            name,
                            scratch_args.as_slice(),
                            args_len,
                        )? {
                            rewrite_step!(used, node, 1);
                        }
                        if args_len != 0 && !is_supported_runtime_prim_name(name) {
                            return Err(EvalError::UnknownPrim(name.to_owned()));
                        }
                    }
                    record_stack_head_time!();
                    return Ok(StackStep::Whnf {
                        node: stack.outer_root(head),
                        head,
                        reductions: carried_reductions,
                    });
                }
                EvalHead::Whnf => {
                    record_stack_head_time!();
                    return Ok(StackStep::Whnf {
                        node: stack.outer_root(head),
                        head,
                        reductions: carried_reductions,
                    });
                }
            };
            use KnownPrim::*;

            match known {
                IoStrict if args_len >= 2 => {
                    let (redex, action, value) = take_args!(2, take_args2);
                    if profiling {
                        self.profile_persistent_force();
                        self.profile_eval_frame_push("Whnf");
                    }
                    record_stack_head_time!();
                    #[cfg(feature = "eval-phase-profile")]
                    let started = profiling.then(Instant::now);
                    stack.push_whnf_frame(
                        redex,
                        2,
                        profile_head,
                        WhnfFrameKind::IoStrict { action, value },
                    );
                    #[cfg(feature = "eval-phase-profile")]
                    if let Some(started) = started {
                        let nanos = started.elapsed().as_nanos();
                        self.profile_stack_force_frame_time(nanos);
                        self.profile_stack_force_frame_head_time(profile_head, nanos);
                    }
                    continue_with!(value);
                }
                Seq if args_len >= 2 => {
                    let (redex, x, result) = take_args!(2, take_args2);
                    if profiling {
                        self.profile_persistent_force();
                        self.profile_eval_frame_push("Whnf");
                    }
                    record_stack_head_time!();
                    #[cfg(feature = "eval-phase-profile")]
                    let started = profiling.then(Instant::now);
                    stack.push_whnf_frame(redex, 2, profile_head, WhnfFrameKind::Seq { result });
                    #[cfg(feature = "eval-phase-profile")]
                    if let Some(started) = started {
                        let nanos = started.elapsed().as_nanos();
                        self.profile_stack_force_frame_time(nanos);
                        self.profile_stack_force_frame_head_time(profile_head, nanos);
                    }
                    continue_with!(x);
                }
                IsInt if args_len >= 1 => {
                    let (redex, x) = take_args!(1, take_args1);
                    if profiling {
                        self.profile_persistent_force();
                        self.profile_eval_frame_push("Whnf");
                    }
                    record_stack_head_time!();
                    #[cfg(feature = "eval-phase-profile")]
                    let started = profiling.then(Instant::now);
                    stack.push_whnf_frame(redex, 1, profile_head, WhnfFrameKind::IsInt);
                    #[cfg(feature = "eval-phase-profile")]
                    if let Some(started) = started {
                        let nanos = started.elapsed().as_nanos();
                        self.profile_stack_force_frame_time(nanos);
                        self.profile_stack_force_frame_head_time(profile_head, nanos);
                    }
                    continue_with!(x);
                }
                _ => {}
            }

            match known {
                IoPerformIo if args_len >= 1 => {
                    let (redex, io) = take_args!(1, take_args1);
                    let world = self.world();
                    let k = self.prim("K");
                    let action = app_site!("IO.performIO.action", io, world);
                    app_taken!(redex, 1, action, k);
                }
                IoBind if args_len >= 3 => {
                    let (redex, io, k, world) = take_args!(3, take_args3);
                    let action = app_site!("IO.bind.action", io, world);
                    app_taken!(redex, 3, action, k);
                }
                IoThen if args_len >= 3 && budget >= 2 => {
                    let (redex, io, y, world) = take_args!(3, take_args3);
                    let k = self.prim("K");
                    let then = app_site!("IO.then.k", k, y);
                    let action = app_site!("IO.then.action", io, world);
                    app_taken_reductions!(redex, 3, action, then, 2);
                }
                IoThen if args_len >= 2 => {
                    let (redex, io, y) = take_args!(2, take_args2);
                    let bind = self.prim("IO.>>=");
                    let bind_action = app_site!("IO.then.bind_action", bind, io);
                    let k = self.prim("K");
                    let then = app_site!("IO.then.k", k, y);
                    app_taken!(redex, 2, bind_action, then);
                }
                IoReturn if args_len >= 3 => {
                    let (redex, x, world, k) = take_args!(3, take_args3);
                    let kx = app_site!("IO.return.kx", k, x);
                    app_taken!(redex, 3, kx, world);
                }
                I | Ord | Chr if args_len >= 1 => {
                    let mut used = 1;
                    let mut reductions = 1;
                    let mut node = arg!(0);
                    let mut alias_shortcuts = 0;
                    while reductions < budget
                        && used < args_len
                        && self.is_identity_alias_node(node)?
                    {
                        node = arg!(used);
                        used += 1;
                        reductions += 1;
                        alias_shortcuts += 1;
                    }
                    self.profile_shortcut("identity_alias_chain", alias_shortcuts);
                    rewrite_continue_reductions!(used, node, reductions);
                }
                K if args_len >= 2 => {
                    let (redex, x, _) = take_args!(2, take_args2);
                    goind_taken!(redex, 2, x, 1);
                }
                A if args_len >= 2 => {
                    let (redex, _, y) = take_args!(2, take_args2);
                    goind_taken!(redex, 2, y, 1);
                }
                U if args_len >= 2 => {
                    let (redex, x, y) = take_args!(2, take_args2);
                    app_taken!(redex, 2, y, x);
                }
                S if args_len >= 3 => {
                    let (redex, x, y, z) = take_args!(3, take_args3);
                    let left = app_site!("S.left", x, z);
                    let right = app_site!("S.right", y, z);
                    app_taken!(redex, 3, left, right);
                }
                SPrime if args_len >= 4 => {
                    let (redex, x, y, z, w) = take_args!(4, take_args4);
                    let yw = app_site!("S'.yw", y, w);
                    let zw = app_site!("S'.zw", z, w);
                    let left = app_site!("S'.left", x, yw);
                    app_taken!(redex, 4, left, zw);
                }
                B if args_len >= 3 => {
                    let (redex, x, y, z) = take_args!(3, take_args3);
                    let yz = app_site!("B.yz", y, z);
                    app_taken!(redex, 3, x, yz);
                }
                BPrime if args_len >= 4 => {
                    let (redex, x, y, z, w) = take_args!(4, take_args4);
                    let zw = app_site!("B'.zw", z, w);
                    let xy = app_site!("B'.xy", x, y);
                    app_taken!(redex, 4, xy, zw);
                }
                BPrime if args_len >= 2 => {
                    let (redex, x, y) = take_args!(2, take_args2);
                    let xy = app_site!("B'.xy_under", x, y);
                    let b = self.prim("B");
                    app_taken!(redex, 2, b, xy);
                }
                Z if args_len >= 3 => {
                    let (redex, x, y, _) = take_args!(3, take_args3);
                    app_taken!(redex, 3, x, y);
                }
                Z if args_len >= 2 => {
                    let (redex, x, y) = take_args!(2, take_args2);
                    let xy = app_site!("Z.xy_under", x, y);
                    let k = self.prim("K");
                    app_taken!(redex, 2, k, xy);
                }
                J if args_len >= 3 => {
                    let (redex, x, _, z) = take_args!(3, take_args3);
                    app_taken!(redex, 3, z, x);
                }
                L if args_len >= 3 => {
                    let (redex, x, y, _) = take_args!(3, take_args3);
                    app_taken!(redex, 3, y, x);
                }
                KK if args_len >= 3 => {
                    let (redex, _, y, _) = take_args!(3, take_args3);
                    goind_taken!(redex, 3, y, 1);
                }
                KA if args_len >= 3 => {
                    let (redex, _, _, z) = take_args!(3, take_args3);
                    goind_taken!(redex, 3, z, 1);
                }
                C if args_len >= 3 => {
                    let (redex, x, y, z) = take_args!(3, take_args3);
                    let xz = app_site!("C.xz", x, z);
                    app_taken!(redex, 3, xz, y);
                }
                CPrime if args_len >= 4 => {
                    let (redex, x, y, z, w) = take_args!(4, take_args4);
                    let yw = app_site!("C'.yw", y, w);
                    let xyw = app_site!("C'.xyw", x, yw);
                    app_taken!(redex, 4, xyw, z);
                }
                P if args_len >= 3 => {
                    let (redex, x, y, z) = take_args!(3, take_args3);
                    let zx = app_site!("P.zx", z, x);
                    app_taken!(redex, 3, zx, y);
                }
                R if args_len >= 3 => {
                    let (redex, x, y, z) = take_args!(3, take_args3);
                    let yz = app_site!("R.yz", y, z);
                    app_taken!(redex, 3, yz, x);
                }
                R if args_len >= 2 => {
                    let (redex, x, y) = take_args!(2, take_args2);
                    let c = self.prim("C");
                    let cy = app_site!("R.cy_under", c, y);
                    app_taken!(redex, 2, cy, x);
                }
                O if args_len >= 4 => {
                    let (redex, x, y, _, w) = take_args!(4, take_args4);
                    let wx = app_site!("O.wx", w, x);
                    app_taken!(redex, 4, wx, y);
                }
                K2 if args_len >= 3 => {
                    let (redex, x, _, _) = take_args!(3, take_args3);
                    goind_taken!(redex, 3, x, 1);
                }
                K2 if args_len >= 2 => {
                    let (redex, x, _) = take_args!(2, take_args2);
                    let k = self.prim("K");
                    app_taken!(redex, 2, k, x);
                }
                K3 if args_len >= 4 => {
                    let (redex, x, _, _, _) = take_args!(4, take_args4);
                    goind_taken!(redex, 4, x, 1);
                }
                K3 if args_len >= 2 => {
                    let (redex, x, _) = take_args!(2, take_args2);
                    let k2 = self.prim("K2");
                    app_taken!(redex, 2, k2, x);
                }
                K4 if args_len >= 5 => {
                    let (redex, x, _, _, _, _) = take_args!(5, take_args5);
                    goind_taken!(redex, 5, x, 1);
                }
                K4 if args_len >= 2 => {
                    let (redex, x, _) = take_args!(2, take_args2);
                    let k3 = self.prim("K3");
                    app_taken!(redex, 2, k3, x);
                }
                CPrimeB if args_len >= 4 => {
                    let (redex, x, y, z, w) = take_args!(4, take_args4);
                    let yw = app_site!("C'B.yw", y, w);
                    let xz = app_site!("C'B.xz", x, z);
                    app_taken!(redex, 4, xz, yw);
                }
                CPrimeB if args_len >= 3 => {
                    let (redex, x, y, z) = take_args!(3, take_args3);
                    let xz = app_site!("C'B.xz_under", x, z);
                    let b = self.prim("B");
                    let bxz = app_site!("C'B.bxz_under", b, xz);
                    app_taken!(redex, 3, bxz, y);
                }
                Y if args_len >= 1 => {
                    let (redex, x) = take_args!(1, take_args1);
                    app_taken!(redex, 1, x, redex);
                }
                Tag(tag) if args_len >= 2 => {
                    let (redex, x, y) = take_args!(2, take_args2);
                    let tag = self.int(i64::from(tag));
                    let ytag = app_site!("Tag.ytag", y, tag);
                    app_taken!(redex, 2, ytag, x);
                }
                Tuple(fields) if args_len > usize::from(fields) => {
                    let fields = usize::from(fields);
                    let mut n = arg!(fields);
                    for idx in 0..fields - 1 {
                        let arg = arg!(idx);
                        n = app_site!("Tuple.prefix", n, arg);
                    }
                    let last = arg!(fields - 1);
                    app_step!(fields + 1, n, last);
                }
                I | Ord | Chr | K | A | U | S | SPrime | B | BPrime | Z | J | L | KK | KA | C
                | CPrime | P | R | O | K2 | K3 | K4 | CPrimeB | Y | Tag(_) | Tuple(_)
                | IoPerformIo | IoBind | IoThen | IoReturn => {
                    record_stack_head_time!();
                    return Ok(StackStep::Whnf {
                        node: stack.outer_root(head),
                        head,
                        reductions: carried_reductions,
                    });
                }
                _ => {
                    self.profile_stack_fallback_head(head);
                    record_stack_head_time!();
                    return Ok(StackStep::Fallback {
                        root: stack.outer_root(head),
                        head,
                        reductions: carried_reductions,
                    });
                }
            }
        }
    }

    fn ignored_io_action_reductions(
        &mut self,
        action: NodeId,
        budget: usize,
    ) -> Result<Option<usize>, EvalError> {
        self.io_action_reductions(action, budget, IGNORED_IO_SHORTCUT_RECURSION_LIMIT)
    }

    fn io_action_reductions(
        &mut self,
        action: NodeId,
        budget: usize,
        depth: usize,
    ) -> Result<Option<usize>, EvalError> {
        if budget == 0 || depth == 0 {
            return Ok(None);
        }

        let spine = self.spine(action)?;
        let head = spine.head;
        let args = spine.args();
        use KnownPrim::*;
        match self.cell(head).prim() {
            Some(Prim::Known(IoReturn)) if args.len() == 1 => Ok(Some(1)),
            Some(Prim::Known(IoThen)) if args.len() == 2 && budget >= 2 => {
                let Some(right_reductions) =
                    self.io_action_reductions(args[1], budget - 1, depth - 1)?
                else {
                    return Ok(None);
                };
                let remaining_budget = budget.saturating_sub(right_reductions + 1);
                if remaining_budget == 0 {
                    return Ok(None);
                }
                let Some(left_reductions) =
                    self.io_action_reductions(args[0], remaining_budget, depth - 1)?
                else {
                    return Ok(None);
                };
                Ok(Some(left_reductions + right_reductions + 1))
            }
            Some(Prim::Known(IoLazyBind))
                if args.len() == 2
                    && budget >= 2
                    && self.direct_ffi_continuation_accepts_result(args[1])? =>
            {
                let Some(action_reductions) =
                    self.io_action_reductions(args[0], budget - 1, depth - 1)?
                else {
                    return Ok(None);
                };
                let remaining_budget = budget.saturating_sub(action_reductions + 1);
                if remaining_budget == 0 {
                    return Ok(None);
                }
                Ok(Some(action_reductions + 2))
            }
            Some(Prim::Known(IoGetArgRef | IoGetMaskingState | IoYield)) if args.is_empty() => {
                Ok(Some(1))
            }
            Some(Prim::Known(IoSetMaskingState)) if args.len() == 1 => Ok(Some(1)),
            _ if matches!(self.cold_node(head), Some(Node::Ffi(_))) => {
                let Some(Node::Ffi(name)) = self.cold_node(head).cloned() else {
                    unreachable!();
                };
                let arity =
                    ffi_arity(&name).ok_or_else(|| EvalError::UnknownFfi(name.to_string()))?;
                Ok((args.len() == arity).then_some(1))
            }
            _ => Ok(None),
        }
    }

    fn run_ignored_io_action(
        &mut self,
        action: NodeId,
        world: NodeId,
    ) -> Result<Option<NodeId>, EvalError> {
        Ok(self
            .run_io_action(action, world, IGNORED_IO_SHORTCUT_RECURSION_LIMIT)?
            .map(|(_, world)| world))
    }

    fn io_return_action_result(&mut self, action: NodeId) -> Result<Option<NodeId>, EvalError> {
        let action = self.resolve_profiled(action)?;
        let Some((fun, result)) = self.cell(action).app_fields() else {
            return Ok(None);
        };
        let fun = self.resolve_profiled(fun)?;
        Ok(match self.cell(fun).prim() {
            Some(Prim::Known(KnownPrim::IoReturn)) => Some(result),
            _ => None,
        })
    }

    fn run_io_action(
        &mut self,
        action: NodeId,
        world: NodeId,
        depth: usize,
    ) -> Result<Option<(NodeId, NodeId)>, EvalError> {
        if depth == 0 {
            return Ok(None);
        }

        let spine = self.spine(action)?;
        let head = spine.head;
        let args = spine.args();
        use KnownPrim::*;
        match self.cell(head).prim() {
            Some(Prim::Known(IoReturn)) if args.len() == 1 => Ok(Some((args[0], world))),
            Some(Prim::Known(IoThen)) if args.len() == 2 => {
                let Some((_, world)) = self.run_io_action(args[0], world, depth - 1)? else {
                    return Ok(None);
                };
                self.run_io_action(args[1], world, depth - 1)
            }
            Some(Prim::Known(IoLazyBind))
                if args.len() == 2 && self.direct_ffi_continuation_accepts_result(args[1])? =>
            {
                let Some((result, world)) = self.run_io_action(args[0], world, depth - 1)? else {
                    return Ok(None);
                };
                let next = self.app(args[1], result);
                self.run_io_action(next, world, depth - 1)
            }
            Some(Prim::Known(IoGetArgRef)) if args.is_empty() => {
                let result = self.arg_ref_array();
                Ok(Some((result, world)))
            }
            Some(Prim::Known(IoGetMaskingState)) if args.is_empty() => {
                let result = self.int(self.masking_state);
                Ok(Some((result, world)))
            }
            Some(Prim::Known(IoSetMaskingState)) if args.len() == 1 => {
                self.masking_state = self.eval_int(args[0])?;
                let result = self.prim("I");
                Ok(Some((result, world)))
            }
            Some(Prim::Known(IoYield)) if args.is_empty() => {
                let result = self.prim("I");
                Ok(Some((result, world)))
            }
            _ if matches!(self.cold_node(head), Some(Node::Ffi(_))) => {
                let Some(Node::Ffi(name)) = self.cold_node(head).cloned() else {
                    unreachable!();
                };
                let arity =
                    ffi_arity(&name).ok_or_else(|| EvalError::UnknownFfi(name.to_string()))?;
                if args.len() != arity {
                    return Ok(None);
                }
                let mut ffi_args = Vec::with_capacity(args.len() + 1);
                ffi_args.extend_from_slice(args);
                ffi_args.push(world);
                let Some((_, pair)) = self.ffi_call(&name, &ffi_args)? else {
                    return Ok(None);
                };
                self.pair_fields(pair)
            }
            _ => Ok(None),
        }
    }

    fn direct_ffi_continuation_accepts_result(&mut self, cont: NodeId) -> Result<bool, EvalError> {
        let cont = self.resolve_profiled(cont)?;
        let Some(Node::Ffi(name)) = self.cold_node(cont) else {
            return Ok(false);
        };
        let Some(arity) = ffi_arity(&name) else {
            return Err(EvalError::UnknownFfi(name.to_string()));
        };
        Ok(arity == 1)
    }

    fn pair_fields(&mut self, pair: NodeId) -> Result<Option<(NodeId, NodeId)>, EvalError> {
        let pair = self.resolve_profiled(pair)?;
        let Some((result_pair, world)) = self.cell(pair).app_fields() else {
            return Ok(None);
        };
        let result_pair = self.resolve_profiled(result_pair)?;
        let Some((pair_constructor, result)) = self.cell(result_pair).app_fields() else {
            return Ok(None);
        };
        let pair_constructor = self.resolve_profiled(pair_constructor)?;
        Ok(match self.cell(pair_constructor).prim() {
            Some(Prim::Known(KnownPrim::P)) => Some((result, world)),
            _ => None,
        })
    }

    fn selector_pair_field(
        &mut self,
        selector: NodeId,
        pair: NodeId,
    ) -> Result<Option<NodeId>, EvalError> {
        let selector = self.resolve_profiled(selector)?;
        use KnownPrim::*;
        let field = match self.cell(selector).prim() {
            Some(Prim::Known(K)) => 0,
            Some(Prim::Known(A)) => 1,
            _ => return Ok(None),
        };
        let pair = self.reduce_node_whnf(pair, FORCE_REDUCTION_LIMIT)?;
        let Some((result, world)) = self.pair_fields(pair)? else {
            return Ok(None);
        };
        Ok(Some(if field == 0 { result } else { world }))
    }

    fn tuple_first_field_selector_extra(
        &mut self,
        selector: NodeId,
        fields: usize,
        available_extra: usize,
    ) -> Result<Option<usize>, EvalError> {
        let selector = self.resolve_profiled(selector)?;
        use KnownPrim::*;
        let arity = match self.cell(selector).prim() {
            Some(Prim::Known(K2)) => 3,
            Some(Prim::Known(K3)) => 4,
            Some(Prim::Known(K4)) => 5,
            _ => return Ok(None),
        };
        if arity < fields {
            return Ok(None);
        }
        let extra = arity - fields;
        Ok((extra <= available_extra).then_some(extra))
    }

    fn spine(&mut self, root: NodeId) -> Result<Spine, EvalError> {
        let mut node = self.resolve_profiled(root)?;
        let mut inline_args = [const { MaybeUninit::uninit() }; INLINE_SPINE];
        let mut inline_apps = [const { MaybeUninit::uninit() }; INLINE_SPINE];
        let mut inline_len = 0;
        let mut heap: Option<(Vec<NodeId>, Vec<NodeId>)> = None;
        while let Some((fun, arg)) = self.cell(node).app_fields() {
            if let Some((args, apps)) = &mut heap {
                args.push(arg);
                apps.push(node);
            } else if inline_len < INLINE_SPINE {
                inline_args[inline_len].write(arg);
                inline_apps[inline_len].write(node);
                inline_len += 1;
            } else {
                let mut args = Vec::with_capacity(INLINE_SPINE * 2);
                let mut apps = Vec::with_capacity(INLINE_SPINE * 2);
                for idx in 0..inline_len {
                    // SAFETY: indices below inline_len were written above.
                    args.push(unsafe { inline_args[idx].assume_init() });
                    // SAFETY: indices below inline_len were written above.
                    apps.push(unsafe { inline_apps[idx].assume_init() });
                }
                args.push(arg);
                apps.push(node);
                heap = Some((args, apps));
            }
            node = self.resolve_profiled(fun)?;
        }
        let storage = if let Some((mut args, mut apps)) = heap {
            args.reverse();
            apps.reverse();
            SpineStorage::Heap { args, apps }
        } else {
            inline_args[..inline_len].reverse();
            inline_apps[..inline_len].reverse();
            SpineStorage::Inline {
                args: inline_args,
                apps: inline_apps,
                len: inline_len,
            }
        };
        Ok(Spine {
            head: node,
            storage,
        })
    }

    fn apply_reduction_spine(
        &mut self,
        node: &mut NodeId,
        used: usize,
        apps: &[NodeId],
    ) -> Result<bool, EvalError> {
        let mut in_place = false;
        // Match the C reducer's update point: the consumed redex root is shared
        // even when the current evaluation has extra arguments on the spine.
        if used > 0 && used < apps.len() {
            self.set_app_cell_at(apps[used - 1].index(), Cell::indir(Some(*node)));
            in_place = true;
        }
        for app in &apps[used..] {
            let Some((_, arg)) = self.cell(*app).app_fields() else {
                return Err(EvalError::DanglingIndirection(*app));
            };
            self.set_app_cell_at(app.index(), Cell::app(*node, arg));
            *node = *app;
            in_place = true;
        }
        Ok(in_place)
    }

    fn is_identity_alias_node(&mut self, id: NodeId) -> Result<bool, EvalError> {
        let id = self.resolve_profiled(id)?;
        Ok(matches!(
            self.cell(id).prim(),
            Some(Prim::Known(KnownPrim::I | KnownPrim::Ord | KnownPrim::Chr))
        ))
    }

    #[inline]
    fn app(&mut self, fun: NodeId, arg: NodeId) -> NodeId {
        #[cfg(feature = "eval-phase-profile")]
        let profile_started = self.profile.is_some().then(Instant::now);
        if let Some(profile) = self.profile.as_mut() {
            profile.app_allocations += 1;
            *profile
                .node_allocations
                .entry(node_allocation_key(&Node::App(fun, arg)).to_owned())
                .or_default() += 1;
            *profile
                .app_allocation_sites
                .entry("<generic app()>".to_owned())
                .or_default() += 1;
        }
        #[cfg(feature = "eval-phase-profile")]
        if let Some(started) = profile_started {
            self.profile_app_alloc_bookkeeping_time(started.elapsed().as_nanos());
        }
        #[cfg(feature = "eval-phase-profile")]
        let started = self.profile.is_some().then(Instant::now);
        let node = self.push_app_node(fun, arg);
        #[cfg(feature = "eval-phase-profile")]
        if let Some(started) = started {
            let nanos = started.elapsed().as_nanos();
            self.profile_stack_app_alloc_time(nanos);
            self.profile_stack_app_alloc_site_time("<generic app()>", nanos);
        }
        node
    }

    #[inline]
    fn app_with_site(&mut self, key: &'static str, fun: NodeId, arg: NodeId) -> NodeId {
        #[cfg(feature = "eval-phase-profile")]
        let profile_started = self.profile.is_some().then(Instant::now);
        if let Some(profile) = self.profile.as_mut() {
            profile.app_allocations += 1;
            *profile
                .node_allocations
                .entry(node_allocation_key(&Node::App(fun, arg)).to_owned())
                .or_default() += 1;
            *profile
                .app_allocation_sites
                .entry(key.to_owned())
                .or_default() += 1;
        }
        #[cfg(feature = "eval-phase-profile")]
        if let Some(started) = profile_started {
            self.profile_app_alloc_bookkeeping_time(started.elapsed().as_nanos());
        }
        #[cfg(feature = "eval-phase-profile")]
        let started = self.profile.is_some().then(Instant::now);
        let node = self.push_app_node(fun, arg);
        #[cfg(feature = "eval-phase-profile")]
        if let Some(started) = started {
            let nanos = started.elapsed().as_nanos();
            self.profile_stack_app_alloc_time(nanos);
            self.profile_stack_app_alloc_site_time(key, nanos);
        }
        node
    }

    fn prim(&mut self, name: &str) -> NodeId {
        let cached = match name {
            "A" => self.prim_cache.a,
            "B" => self.prim_cache.b,
            "C" => self.prim_cache.c,
            "I" => self.prim_cache.i,
            "K" => self.prim_cache.k,
            "K2" => self.prim_cache.k2,
            "K3" => self.prim_cache.k3,
            "O" => self.prim_cache.o,
            "P" => self.prim_cache.p,
            "U" => self.prim_cache.u,
            "Y" => self.prim_cache.y,
            "Z" => self.prim_cache.z,
            "IO.>>=" => self.prim_cache.io_bind,
            "IO.performIO" => self.prim_cache.io_perform_io,
            _ => None,
        };
        if let Some(id) = cached {
            return id;
        }

        let id = self.push_node(Node::prim(name));
        match name {
            "A" => self.prim_cache.a = Some(id),
            "B" => self.prim_cache.b = Some(id),
            "C" => self.prim_cache.c = Some(id),
            "I" => self.prim_cache.i = Some(id),
            "K" => self.prim_cache.k = Some(id),
            "K2" => self.prim_cache.k2 = Some(id),
            "K3" => self.prim_cache.k3 = Some(id),
            "O" => self.prim_cache.o = Some(id),
            "P" => self.prim_cache.p = Some(id),
            "U" => self.prim_cache.u = Some(id),
            "Y" => self.prim_cache.y = Some(id),
            "Z" => self.prim_cache.z = Some(id),
            "IO.>>=" => self.prim_cache.io_bind = Some(id),
            "IO.performIO" => self.prim_cache.io_perform_io = Some(id),
            _ => {}
        }
        id
    }

    fn int(&mut self, value: i64) -> NodeId {
        let Some(index) = small_int_index(value) else {
            if self.profile.is_some() {
                self.profile_non_small_int_allocation();
            }
            return self.push_node(Node::Int(value));
        };
        if let Some(id) = self.small_ints[index] {
            if self.profile.is_some() {
                self.profile_small_int_cache_hit();
            }
            return id;
        }
        if self.profile.is_some() {
            self.profile_small_int_cache_miss();
        }
        let id = self.push_node(Node::Int(value));
        self.small_ints[index] = Some(id);
        id
    }

    fn push_value_node(&mut self, node: Node) -> NodeId {
        match node {
            Node::Int(value) => self.int(value),
            Node::Prim(prim) => self.prim(prim.name()),
            node => self.push_node(node),
        }
    }

    fn world(&mut self) -> NodeId {
        if let Some(world) = self.world {
            return world;
        }
        let world = self.push_node(Node::Int(99_999));
        self.world = Some(world);
        world
    }

    fn fst(&mut self) -> NodeId {
        if let Some(fst) = self.compound_cache.fst {
            return fst;
        }
        let u = self.prim("U");
        let k = self.prim("K");
        let fst = self.app(u, k);
        self.compound_cache.fst = Some(fst);
        fst
    }

    fn snd(&mut self) -> NodeId {
        if let Some(snd) = self.compound_cache.snd {
            return snd;
        }
        let u = self.prim("U");
        let a = self.prim("A");
        let snd = self.app(u, a);
        self.compound_cache.snd = Some(snd);
        snd
    }

    fn pair(&mut self, result: NodeId, world: NodeId) -> NodeId {
        let pair = self.prim("P");
        let result_pair = self.app(pair, result);
        self.app(result_pair, world)
    }

    fn unit_pair(&mut self, world: NodeId) -> NodeId {
        let pair_unit = if let Some(pair_unit) = self.compound_cache.pair_unit {
            pair_unit
        } else {
            let pair = self.prim("P");
            let unit = self.prim("I");
            let pair_unit = self.app(pair, unit);
            self.compound_cache.pair_unit = Some(pair_unit);
            pair_unit
        };
        self.app(pair_unit, world)
    }

    fn just(&mut self, value: NodeId) -> NodeId {
        let just = if let Some(just) = self.compound_cache.just {
            just
        } else {
            let z = self.prim("Z");
            let u = self.prim("U");
            let just = self.app(z, u);
            self.compound_cache.just = Some(just);
            just
        };
        self.app(just, value)
    }

    fn nothing(&mut self) -> NodeId {
        self.prim("K")
    }

    fn catch_result(
        &mut self,
        action: NodeId,
        handler: NodeId,
        world: NodeId,
    ) -> Result<NodeId, EvalError> {
        let old_mask = self.masking_state;
        match self.reduce_node_whnf(action, FORCE_REDUCTION_LIMIT) {
            Ok(result) => Ok(result),
            Err(EvalError::Raised(exn)) => {
                self.masking_state = MASK_INTERRUPTIBLE;
                let handled = self.app(handler, exn);
                let bind = self.prim("IO.>>=");
                let handled_bind = self.app(bind, handled);
                let b_prime = self.prim("B'");
                let then = self.prim("IO.>>");
                let restore = self.prim("IO.setmaskingstate");
                let old_mask = self.int(old_mask);
                let restore = self.app(restore, old_mask);
                let restore_then = self.app(b_prime, then);
                let restore_then = self.app(restore_then, restore);
                let ret = self.prim("IO.return");
                let continuation = self.app(restore_then, ret);
                let caught = self.app(handled_bind, continuation);
                Ok(self.app(caught, world))
            }
            Err(err) => Err(err),
        }
    }

    fn rts_exception(&mut self, code: i64) -> EvalError {
        let exn = self.int(code);
        EvalError::Raised(exn)
    }

    fn arithmetic_eval_error(&mut self, err: EvalError) -> EvalError {
        match err {
            EvalError::DivideByZero => self.rts_exception(RTS_EXN_DIVIDE_BY_ZERO),
            EvalError::Overflow => self.rts_exception(RTS_EXN_OVERFLOW),
            other => other,
        }
    }

    fn ordering(&mut self, ord: Ordering) -> NodeId {
        let name = match ord {
            Ordering::Less => "K2",
            Ordering::Equal => "KK",
            Ordering::Greater => "KA",
        };
        self.prim(name)
    }

    fn bool_value_node(value: bool) -> Node {
        Node::Prim(Prim::Known(if value { KnownPrim::A } else { KnownPrim::K }))
    }

    fn ordering_value_node(ord: Ordering) -> Node {
        let known = match ord {
            Ordering::Less => KnownPrim::K2,
            Ordering::Equal => KnownPrim::KK,
            Ordering::Greater => KnownPrim::KA,
        };
        Node::Prim(Prim::Known(known))
    }

    fn int_result_node(&mut self, result: IntResult) -> NodeId {
        match result {
            IntResult::Int(n) => self.int(n),
            IntResult::Bool(b) => self.prim(if b { "A" } else { "K" }),
            IntResult::Ordering(ord) => self.ordering(ord),
        }
    }

    fn int_result_value_node(result: IntResult) -> Node {
        match result {
            IntResult::Int(n) => Node::Int(n),
            IntResult::Bool(b) => Self::bool_value_node(b),
            IntResult::Ordering(ord) => Self::ordering_value_node(ord),
        }
    }

    fn finish_int_frame(
        &mut self,
        frame: IntFrame,
        value: i64,
        stack: &mut EvalFrameStack,
    ) -> Result<(NodeId, usize), EvalError> {
        let result = match frame.kind {
            IntFrameKind::BinSecond { op, x } => {
                let next = x;
                let next_frame = IntFrame {
                    redex: frame.redex,
                    profile_head: frame.profile_head,
                    kind: IntFrameKind::BinFirst { op, y: value },
                };
                if self.profile.is_some() {
                    self.profile_eval_frame_push("Int");
                }
                stack.push(EvalFrame::Int(next_frame));
                return Ok((next, 0));
            }
            IntFrameKind::BinFirst { op, y } => op
                .apply(value, y)
                .map_err(|err| self.arithmetic_eval_error(err))?,
            IntFrameKind::Un { op } => {
                let n = op
                    .apply(value)
                    .map_err(|err| self.arithmetic_eval_error(err))?;
                IntResult::Int(n)
            }
        };

        if frame.profile_head.is_some() {
            self.profile_reduction(frame.profile_head, 1);
        }
        let result_node = self.int_result_node(result);
        let node = self.apply_strict_redex(frame.redex, result_node)?;
        Ok((node, 1))
    }

    fn int64_result_node(&mut self, result: Int64Result) -> NodeId {
        match result {
            Int64Result::Int64(n) => self.push_node(Node::Int64(n)),
            Int64Result::Bool(b) => self.prim(if b { "A" } else { "K" }),
            Int64Result::Ordering(ord) => self.ordering(ord),
        }
    }

    fn int64_result_value_node(result: Int64Result) -> Node {
        match result {
            Int64Result::Int64(n) => Node::Int64(n),
            Int64Result::Bool(b) => Self::bool_value_node(b),
            Int64Result::Ordering(ord) => Self::ordering_value_node(ord),
        }
    }

    fn int64_un_result_node(&mut self, result: Int64UnResult) -> NodeId {
        match result {
            Int64UnResult::Int64(n) => self.push_node(Node::Int64(n)),
            Int64UnResult::Int(n) => self.int(n),
        }
    }

    fn finish_int64_frame(
        &mut self,
        frame: Int64Frame,
        value: i64,
        stack: &mut EvalFrameStack,
    ) -> Result<(NodeId, usize), EvalError> {
        let node = match frame.kind {
            Int64FrameKind::BinSecond { op, x } => {
                let next = x;
                let next_frame = Int64Frame {
                    redex: frame.redex,
                    profile_head: frame.profile_head,
                    kind: Int64FrameKind::BinFirst { op, y: value },
                };
                if self.profile.is_some() {
                    self.profile_eval_frame_push("Int64");
                }
                stack.push(EvalFrame::Int64(next_frame));
                return Ok((next, 0));
            }
            Int64FrameKind::BinFirst { op, y } => {
                let result = op
                    .apply(value, y)
                    .map_err(|err| self.arithmetic_eval_error(err))?;
                self.int64_result_node(result)
            }
            Int64FrameKind::ShiftFirst { op, y } => {
                let result = op
                    .apply(value, y)
                    .map_err(|err| self.arithmetic_eval_error(err))?;
                self.int64_result_node(result)
            }
            Int64FrameKind::Un { op } => {
                let result = op
                    .apply(value)
                    .map_err(|err| self.arithmetic_eval_error(err))?;
                self.int64_un_result_node(result)
            }
        };

        if frame.profile_head.is_some() {
            self.profile_reduction(frame.profile_head, 1);
        }
        let node = self.apply_strict_redex(frame.redex, node)?;
        Ok((node, 1))
    }

    fn apply_strict_redex(
        &mut self,
        redex: StrictRedex,
        mut node: NodeId,
    ) -> Result<NodeId, EvalError> {
        match redex {
            StrictRedex::Root(root) => {
                if node != root {
                    self.set_cell_at(root.index(), Cell::indir(Some(node)));
                }
            }
            StrictRedex::Spine { root, used, apps } => {
                let in_place = self.apply_reduction_spine(&mut node, used, &apps)?;
                if !in_place && node != root {
                    self.set_cell_at(root.index(), Cell::indir(Some(node)));
                }
            }
        }
        Ok(node)
    }

    fn stack_entry_app(&self, stack: &EvalStack, index: usize) -> Result<NodeId, EvalError> {
        stack
            .apps
            .get(index)
            .copied()
            .ok_or_else(|| EvalError::DanglingIndirection(NodeId::from_index(index)))
    }

    fn apply_stack_rewrite(&mut self, stack: &mut EvalStack, used: usize, node: NodeId) -> NodeId {
        if used == 0 {
            return node;
        }
        let app_end = stack.apps.len();
        self.apply_stack_frame_rewrite(stack, app_end, used, node)
    }

    fn apply_stack_frame_rewrite(
        &mut self,
        stack: &mut EvalStack,
        app_end: usize,
        used: usize,
        node: NodeId,
    ) -> NodeId {
        #[cfg(feature = "eval-phase-profile")]
        let started = self.profile.is_some().then(Instant::now);
        debug_assert!(used > 0);
        debug_assert!(used <= app_end);
        let redex_index = app_end - used;
        let redex = stack.app_unchecked(redex_index);
        let wrote_indirection = node != redex;
        if wrote_indirection {
            self.set_app_cell_at(redex.index(), Cell::indir(Some(node)));
        }
        if self.profile.is_some() {
            self.profile_stack_rewrite(used, wrote_indirection);
        }
        stack.apps.truncate(redex_index);
        #[cfg(feature = "eval-phase-profile")]
        if let Some(started) = started {
            self.profile_stack_apply_rewrite_time(started.elapsed().as_nanos());
        }
        node
    }

    fn apply_stack_frame_value(
        &mut self,
        stack: &mut EvalStack,
        app_end: usize,
        used: usize,
        value: Node,
    ) -> NodeId {
        #[cfg(feature = "eval-phase-profile")]
        let started = self.profile.is_some().then(Instant::now);
        debug_assert!(used > 0);
        debug_assert!(used <= app_end);
        let redex_index = app_end - used;
        let redex = stack.app_unchecked(redex_index);
        self.set_app_node_at(redex.index(), value);
        if self.profile.is_some() {
            self.profile_stack_rewrite(used, false);
        }
        stack.apps.truncate(redex_index);
        #[cfg(feature = "eval-phase-profile")]
        if let Some(started) = started {
            self.profile_stack_apply_rewrite_time(started.elapsed().as_nanos());
        }
        redex
    }

    fn apply_stack_redex_value(&mut self, redex: NodeId, used: usize, value: Node) -> NodeId {
        #[cfg(feature = "eval-phase-profile")]
        let started = self.profile.is_some().then(Instant::now);
        self.set_app_node_at(redex.index(), value);
        if self.profile.is_some() {
            self.profile_stack_rewrite(used, false);
        }
        #[cfg(feature = "eval-phase-profile")]
        if let Some(started) = started {
            self.profile_stack_apply_rewrite_time(started.elapsed().as_nanos());
        }
        redex
    }

    fn apply_stack_app(
        &mut self,
        stack: &mut EvalStack,
        used: usize,
        fun: NodeId,
        arg: NodeId,
    ) -> NodeId {
        #[cfg(feature = "eval-phase-profile")]
        let started = self.profile.is_some().then(Instant::now);
        if self.profile.is_some() {
            self.profile_stack_app_update(used);
        }
        let node = if used == 0 {
            self.app(fun, arg)
        } else {
            let app_end = stack.apps.len();
            debug_assert!(used <= app_end);
            let redex_index = app_end - used;
            let redex = stack.app_unchecked(redex_index);
            self.set_app_cell_at(redex.index(), Cell::app(fun, arg));
            stack.apps.truncate(redex_index);
            redex
        };
        #[cfg(feature = "eval-phase-profile")]
        if let Some(started) = started {
            self.profile_stack_apply_app_time(started.elapsed().as_nanos());
        }
        node
    }

    fn rethread_stack_app_segment(
        &mut self,
        stack: &mut EvalStack,
        mut node: NodeId,
    ) -> Result<NodeId, EvalError> {
        let base = stack.app_base();
        if self.profile.is_some() {
            self.profile_stack_rethread(stack.apps.len().saturating_sub(base));
        }
        for index in (base..stack.apps.len()).rev() {
            let app = self.stack_entry_app(stack, index)?;
            let Some((_, arg)) = self.cell(app).app_fields() else {
                return Err(EvalError::DanglingIndirection(app));
            };
            self.set_app_cell_at(app.index(), Cell::app(node, arg));
            node = app;
        }
        stack.apps.truncate(base);
        Ok(node)
    }

    fn descend_stack_from(
        &mut self,
        mut current: NodeId,
        stack: &mut EvalStack,
        profile_resolve: bool,
        profiling: bool,
    ) -> Result<NodeId, EvalError> {
        current = self.resolve_for_whnf(current, profile_resolve)?;
        while let Some(fun) = self.app_fun(current) {
            if profiling {
                self.profile_stack_descent_push();
            }
            stack.push_app(current);
            current = self.resolve_for_whnf(fun, profile_resolve)?;
        }
        Ok(current)
    }

    fn finish_stack_whnf_frame(
        &mut self,
        frame: StackWhnfFrame,
        _stack: &mut EvalStack,
        value: NodeId,
    ) -> Result<(NodeId, usize), EvalError> {
        let node = match frame.kind {
            WhnfFrameKind::Seq { result } => {
                if frame.profile_head.is_some() {
                    self.profile_reduction(frame.profile_head, 1);
                }
                #[cfg(feature = "eval-phase-profile")]
                let started = self.profile.is_some().then(Instant::now);
                let wrote_indirection = result != frame.redex;
                if wrote_indirection {
                    self.set_app_cell_at(frame.redex.index(), Cell::indir(Some(result)));
                }
                if self.profile.is_some() {
                    self.profile_stack_rewrite(frame.used, wrote_indirection);
                }
                #[cfg(feature = "eval-phase-profile")]
                if let Some(started) = started {
                    self.profile_stack_apply_rewrite_time(started.elapsed().as_nanos());
                }
                return Ok((result, 1));
            }
            WhnfFrameKind::IoStrict { action, value } => {
                if frame.profile_head.is_some() {
                    self.profile_reduction(frame.profile_head, 1);
                }
                #[cfg(feature = "eval-phase-profile")]
                let started = self.profile.is_some().then(Instant::now);
                if self.profile.is_some() {
                    self.profile_stack_app_update(frame.used);
                }
                self.set_app_cell_at(frame.redex.index(), Cell::app(action, value));
                #[cfg(feature = "eval-phase-profile")]
                if let Some(started) = started {
                    self.profile_stack_apply_app_time(started.elapsed().as_nanos());
                }
                return Ok((frame.redex, 1));
            }
            WhnfFrameKind::IsInt => {
                let value = self.resolve(value)?;
                let n = self.cell(value).int_value().unwrap_or(-1);
                self.apply_stack_redex_value(frame.redex, frame.used, Node::Int(n))
            }
        };

        if frame.profile_head.is_some() {
            self.profile_reduction(frame.profile_head, 1);
        }
        Ok((node, 1))
    }

    fn finish_ready_stack_frame(
        &mut self,
        stack: &mut EvalStack,
        current: NodeId,
    ) -> Result<Option<(NodeId, usize)>, EvalError> {
        if !stack.top_is_frame() {
            return Ok(None);
        }

        enum ReadyFrame {
            Int(i64),
            Int64Shift(i64),
            Int64(i64),
            Float64(f64),
            Float32(f32),
            Bytes,
            Conversion(ConversionValue),
        }

        let current_cell = self.cell(current);
        let ready = match stack.peek_frame() {
            Some(StackFrame::Int(_)) => current_cell.int_value().map(ReadyFrame::Int),
            Some(StackFrame::Int64Shift(_)) => current_cell.int_value().map(ReadyFrame::Int64Shift),
            Some(StackFrame::Int64(_)) => current_cell.int64_value().map(ReadyFrame::Int64),
            Some(StackFrame::Float64(_)) => current_cell.float64_value().map(ReadyFrame::Float64),
            Some(StackFrame::Float32(_)) => current_cell.float32_value().map(ReadyFrame::Float32),
            Some(StackFrame::Bytes(_))
                if matches!(
                    self.cold_node(current),
                    Some(Node::Bytes(_) | Node::MutableBytes(_))
                ) =>
            {
                Some(ReadyFrame::Bytes)
            }
            Some(StackFrame::Conversion(frame)) => frame
                .kind
                .ready_cell_value(current_cell)
                .map(ReadyFrame::Conversion),
            _ => None,
        };
        let Some(ready) = ready else {
            return Ok(None);
        };

        let frame = stack.pop_frame().expect("ready stack frame must exist");
        let result = match (frame, ready) {
            (StackFrame::Int(frame), ReadyFrame::Int(value)) => {
                let used = match &frame.kind {
                    IntFrameKind::Un { .. } => 1,
                    IntFrameKind::BinSecond { .. } | IntFrameKind::BinFirst { .. } => 2,
                };
                let result = match frame.kind {
                    IntFrameKind::BinSecond { op, x } => {
                        stack.push_int_frame(
                            frame.redex,
                            frame.profile_head,
                            IntFrameKind::BinFirst { op, y: value },
                        );
                        if self.profile.is_some() {
                            self.profile_eval_frame_push("Int");
                        }
                        return Ok(Some((x, 0)));
                    }
                    IntFrameKind::BinFirst { op, y } => op
                        .apply(value, y)
                        .map_err(|err| self.arithmetic_eval_error(err))?,
                    IntFrameKind::Un { op } => {
                        let n = op
                            .apply(value)
                            .map_err(|err| self.arithmetic_eval_error(err))?;
                        IntResult::Int(n)
                    }
                };
                if frame.profile_head.is_some() {
                    self.profile_reduction(frame.profile_head, 1);
                }
                let node = self.apply_stack_redex_value(
                    frame.redex,
                    used,
                    Self::int_result_value_node(result),
                );
                (node, 1)
            }
            (StackFrame::Int64(frame), ReadyFrame::Int64(value)) => {
                let result = match frame.kind {
                    Int64FrameKind::BinSecond { op, x } => {
                        stack.push_int64_frame(
                            frame.app_end,
                            frame.used,
                            frame.profile_head,
                            Int64FrameKind::BinFirst { op, y: value },
                        );
                        if self.profile.is_some() {
                            self.profile_eval_frame_push("Int64");
                        }
                        return Ok(Some((x, 0)));
                    }
                    Int64FrameKind::BinFirst { op, y } => op
                        .apply(value, y)
                        .map_err(|err| self.arithmetic_eval_error(err))?,
                    Int64FrameKind::ShiftFirst { op, y } => op
                        .apply(value, y)
                        .map_err(|err| self.arithmetic_eval_error(err))?,
                    Int64FrameKind::Un { op } => {
                        let result = op
                            .apply(value)
                            .map_err(|err| self.arithmetic_eval_error(err))?;
                        match result {
                            Int64UnResult::Int64(n) => Int64Result::Int64(n),
                            Int64UnResult::Int(n) => {
                                if frame.profile_head.is_some() {
                                    self.profile_reduction(frame.profile_head, 1);
                                }
                                let node = self.apply_stack_frame_value(
                                    stack,
                                    frame.app_end,
                                    frame.used,
                                    Node::Int(n),
                                );
                                return Ok(Some((node, 1)));
                            }
                        }
                    }
                };
                if frame.profile_head.is_some() {
                    self.profile_reduction(frame.profile_head, 1);
                }
                let node = self.apply_stack_frame_value(
                    stack,
                    frame.app_end,
                    frame.used,
                    Self::int64_result_value_node(result),
                );
                (node, 1)
            }
            (StackFrame::Int64Shift(frame), ReadyFrame::Int64Shift(value)) => {
                stack.push_int64_frame(
                    frame.app_end,
                    frame.used,
                    frame.profile_head,
                    Int64FrameKind::ShiftFirst {
                        op: frame.op,
                        y: value,
                    },
                );
                if self.profile.is_some() {
                    self.profile_eval_frame_push("Int64");
                }
                (frame.x, 0)
            }
            (StackFrame::Float64(frame), ReadyFrame::Float64(value)) => {
                let result = match frame.kind {
                    Float64FrameKind::BinSecond { op, x } => {
                        stack.push_float64_frame(
                            frame.app_end,
                            frame.used,
                            frame.profile_head,
                            Float64FrameKind::BinFirst { op, y: value },
                        );
                        if self.profile.is_some() {
                            self.profile_eval_frame_push("Float64");
                        }
                        return Ok(Some((x, 0)));
                    }
                    Float64FrameKind::BinFirst { op, y } => op.apply(value, y),
                    Float64FrameKind::Un { op } => Float64Result::Float(op.apply(value)),
                };
                if frame.profile_head.is_some() {
                    self.profile_reduction(frame.profile_head, 1);
                }
                let value = match result {
                    Float64Result::Float(n) => Node::Float64(n),
                    Float64Result::Bool(b) => Self::bool_value_node(b),
                };
                let node = self.apply_stack_frame_value(stack, frame.app_end, frame.used, value);
                (node, 1)
            }
            (StackFrame::Float32(frame), ReadyFrame::Float32(value)) => {
                let result = match frame.kind {
                    Float32FrameKind::BinSecond { op, x } => {
                        stack.push_float32_frame(
                            frame.app_end,
                            frame.used,
                            frame.profile_head,
                            Float32FrameKind::BinFirst { op, y: value },
                        );
                        if self.profile.is_some() {
                            self.profile_eval_frame_push("Float32");
                        }
                        return Ok(Some((x, 0)));
                    }
                    Float32FrameKind::BinFirst { op, y } => op.apply(value, y),
                    Float32FrameKind::Un { op } => Float32Result::Float(op.apply(value)),
                };
                if frame.profile_head.is_some() {
                    self.profile_reduction(frame.profile_head, 1);
                }
                let value = match result {
                    Float32Result::Float(n) => Node::Float32(n),
                    Float32Result::Bool(b) => Self::bool_value_node(b),
                };
                let node = self.apply_stack_frame_value(stack, frame.app_end, frame.used, value);
                (node, 1)
            }
            (StackFrame::Bytes(frame), ReadyFrame::Bytes) => {
                let node = match frame.kind {
                    BytesFrameKind::BinSecond { op, x } => {
                        stack.push_bytes_frame(
                            frame.app_end,
                            frame.used,
                            frame.profile_head,
                            BytesFrameKind::BinFirst { op, y: current },
                        );
                        if self.profile.is_some() {
                            self.profile_eval_frame_push("Bytes");
                        }
                        return Ok(Some((x, 0)));
                    }
                    BytesFrameKind::BinFirst { op, y } => {
                        self.bytes_bin_result_node(op, current, y)?
                    }
                };
                if frame.profile_head.is_some() {
                    self.profile_reduction(frame.profile_head, 1);
                }
                let node = self.apply_stack_frame_rewrite(stack, frame.app_end, frame.used, node);
                (node, 1)
            }
            (StackFrame::Conversion(frame), ReadyFrame::Conversion(value)) => {
                if frame.profile_head.is_some() {
                    self.profile_reduction(frame.profile_head, 1);
                }
                let node = self.apply_stack_frame_value(
                    stack,
                    frame.app_end,
                    frame.used,
                    Self::conversion_result_value_node(frame.kind, value),
                );
                (node, 1)
            }
            _ => unreachable!("ready stack frame kind changed before pop"),
        };
        Ok(Some(result))
    }

    fn finish_whnf_stack_frame(
        &mut self,
        stack: &mut EvalStack,
        current: NodeId,
    ) -> Result<Option<(NodeId, usize)>, EvalError> {
        let Some(frame) = stack.pop_frame() else {
            return Ok(None);
        };
        let result = match frame {
            StackFrame::Whnf(frame) => self.finish_stack_whnf_frame(frame, stack, current)?,
            StackFrame::Int(_) => return Err(EvalError::ExpectedInt(current)),
            StackFrame::Int64Shift(_) => return Err(EvalError::ExpectedInt(current)),
            StackFrame::Int64(_) => return Err(EvalError::ExpectedInt64(current)),
            StackFrame::Float64(_) => return Err(EvalError::ExpectedFloat64(current)),
            StackFrame::Float32(_) => return Err(EvalError::ExpectedFloat32(current)),
            StackFrame::Bytes(_) => return Err(self.expected_bytes_error(current)),
            StackFrame::Conversion(frame) => return Err(frame.kind.expected_error(current)),
        };
        Ok(Some(result))
    }

    fn begin_whnf_force_frame(
        &mut self,
        root: NodeId,
    ) -> Result<Option<(WhnfFrame, NodeId)>, EvalError> {
        let spine = self.spine(root)?;
        let head = spine.head;
        let args = spine.args();
        let Some(Prim::Known(known)) = self.cell(head).prim() else {
            return Ok(None);
        };
        use KnownPrim::*;
        let force = match known {
            Seq if args.len() >= 2 => Some((2, WhnfFrameKind::Seq { result: args[1] }, args[0])),
            IoStrict if args.len() >= 2 => Some((
                2,
                WhnfFrameKind::IoStrict {
                    action: args[0],
                    value: args[1],
                },
                args[1],
            )),
            IsInt if !args.is_empty() => Some((1, WhnfFrameKind::IsInt, args[0])),
            _ => None,
        };
        let Some((used, kind, next)) = force else {
            return Ok(None);
        };

        let profile_head = if self.profile.is_some() {
            let heap_spine = matches!(&spine.storage, SpineStorage::Heap { .. });
            self.profile_step(head, args.len(), heap_spine)
        } else {
            None
        };
        let redex = if args.len() == used {
            StrictRedex::Root(root)
        } else {
            if self.profile.is_some() {
                self.profile_strict_redex_snapshot(args.len());
            }
            StrictRedex::Spine {
                root,
                used,
                apps: spine.apps().to_vec(),
            }
        };
        Ok(Some((
            WhnfFrame {
                redex,
                profile_head,
                kind,
            },
            next,
        )))
    }

    fn finish_whnf_frame(
        &mut self,
        frame: WhnfFrame,
        value: NodeId,
    ) -> Result<(NodeId, usize), EvalError> {
        let node = match frame.kind {
            WhnfFrameKind::Seq { result } => result,
            WhnfFrameKind::IoStrict { action, value } => self.app(action, value),
            WhnfFrameKind::IsInt => {
                let value = self.resolve(value)?;
                let n = self.cell(value).int_value().unwrap_or(-1);
                self.int(n)
            }
        };

        if frame.profile_head.is_some() {
            self.profile_reduction(frame.profile_head, 1);
        }
        let node = self.apply_strict_redex(frame.redex, node)?;
        Ok((node, 1))
    }

    fn conversion_result_node(
        &mut self,
        kind: ConversionFrameKind,
        value: ConversionValue,
    ) -> NodeId {
        match (kind, value) {
            (ConversionFrameKind::IntToInt64, ConversionValue::Int(n)) => {
                self.push_node(Node::Int64(n))
            }
            (ConversionFrameKind::Int64ToInt, ConversionValue::Int64(n)) => self.int(n),
            (ConversionFrameKind::IntToFloat64 { unsigned: false }, ConversionValue::Int(n)) => {
                self.push_node(Node::Float64(n as f64))
            }
            (ConversionFrameKind::IntToFloat64 { unsigned: true }, ConversionValue::Int(n)) => {
                self.push_node(Node::Float64((n as u64) as f64))
            }
            (ConversionFrameKind::Int64ToFloat64, ConversionValue::Int64(n)) => {
                self.push_node(Node::Float64(n as f64))
            }
            (ConversionFrameKind::Float64ToInt, ConversionValue::Float64(n)) => self.int(n as i64),
            (ConversionFrameKind::IntToFloat32 { unsigned: false }, ConversionValue::Int(n)) => {
                self.push_node(Node::Float32(n as f32))
            }
            (ConversionFrameKind::IntToFloat32 { unsigned: true }, ConversionValue::Int(n)) => {
                self.push_node(Node::Float32((n as u64) as f32))
            }
            (ConversionFrameKind::Int64ToFloat32, ConversionValue::Int64(n)) => {
                self.push_node(Node::Float32(n as f32))
            }
            (ConversionFrameKind::Float32ToInt, ConversionValue::Float32(n)) => self.int(n as i64),
            (ConversionFrameKind::Float64ToFloat32, ConversionValue::Float64(n)) => {
                self.push_node(Node::Float32(n as f32))
            }
            (ConversionFrameKind::Float32ToFloat64, ConversionValue::Float32(n)) => {
                self.push_node(Node::Float64(n as f64))
            }
            (ConversionFrameKind::Int64BitsToFloat64, ConversionValue::Int64(n)) => {
                self.push_node(Node::Float64(f64::from_bits(n as u64)))
            }
            (ConversionFrameKind::Float64BitsToInt64, ConversionValue::Float64(n)) => {
                self.push_node(Node::Int64(n.to_bits() as i64))
            }
            (ConversionFrameKind::IntBitsToFloat32, ConversionValue::Int(n)) => {
                self.push_node(Node::Float32(f32::from_bits(n as u32)))
            }
            (ConversionFrameKind::Float32BitsToInt, ConversionValue::Float32(n)) => {
                self.int((n.to_bits() as i32) as i64)
            }
            _ => unreachable!("conversion frame kind and value mismatch"),
        }
    }

    fn conversion_result_value_node(kind: ConversionFrameKind, value: ConversionValue) -> Node {
        match (kind, value) {
            (ConversionFrameKind::IntToInt64, ConversionValue::Int(n)) => Node::Int64(n),
            (ConversionFrameKind::Int64ToInt, ConversionValue::Int64(n)) => Node::Int(n),
            (ConversionFrameKind::IntToFloat64 { unsigned: false }, ConversionValue::Int(n)) => {
                Node::Float64(n as f64)
            }
            (ConversionFrameKind::IntToFloat64 { unsigned: true }, ConversionValue::Int(n)) => {
                Node::Float64((n as u64) as f64)
            }
            (ConversionFrameKind::Int64ToFloat64, ConversionValue::Int64(n)) => {
                Node::Float64(n as f64)
            }
            (ConversionFrameKind::Float64ToInt, ConversionValue::Float64(n)) => Node::Int(n as i64),
            (ConversionFrameKind::IntToFloat32 { unsigned: false }, ConversionValue::Int(n)) => {
                Node::Float32(n as f32)
            }
            (ConversionFrameKind::IntToFloat32 { unsigned: true }, ConversionValue::Int(n)) => {
                Node::Float32((n as u64) as f32)
            }
            (ConversionFrameKind::Int64ToFloat32, ConversionValue::Int64(n)) => {
                Node::Float32(n as f32)
            }
            (ConversionFrameKind::Float32ToInt, ConversionValue::Float32(n)) => Node::Int(n as i64),
            (ConversionFrameKind::Float64ToFloat32, ConversionValue::Float64(n)) => {
                Node::Float32(n as f32)
            }
            (ConversionFrameKind::Float32ToFloat64, ConversionValue::Float32(n)) => {
                Node::Float64(n as f64)
            }
            (ConversionFrameKind::Int64BitsToFloat64, ConversionValue::Int64(n)) => {
                Node::Float64(f64::from_bits(n as u64))
            }
            (ConversionFrameKind::Float64BitsToInt64, ConversionValue::Float64(n)) => {
                Node::Int64(n.to_bits() as i64)
            }
            (ConversionFrameKind::IntBitsToFloat32, ConversionValue::Int(n)) => {
                Node::Float32(f32::from_bits(n as u32))
            }
            (ConversionFrameKind::Float32BitsToInt, ConversionValue::Float32(n)) => {
                Node::Int((n.to_bits() as i32) as i64)
            }
            _ => unreachable!("conversion frame kind and value mismatch"),
        }
    }

    fn finish_conversion_frame(
        &mut self,
        frame: ConversionFrame,
        value: ConversionValue,
    ) -> Result<(NodeId, usize), EvalError> {
        let node = self.conversion_result_node(frame.kind, value);

        if frame.profile_head.is_some() {
            self.profile_reduction(frame.profile_head, 1);
        }
        let node = self.apply_strict_redex(frame.redex, node)?;
        Ok((node, 1))
    }

    fn finish_ready_eval_frame(
        &mut self,
        stack: &mut EvalFrameStack,
        current: NodeId,
    ) -> Result<Option<(NodeId, usize)>, EvalError> {
        enum ReadyFrame {
            Int(i64),
            Int64Shift(i64),
            Int64(i64),
            Float64(f64),
            Float32(f32),
            Bytes,
            Conversion(ConversionValue),
        }

        let current_cell = self.cell(current);
        let ready = match stack.peek() {
            Some(EvalFrame::Int(_)) => current_cell.int_value().map(ReadyFrame::Int),
            Some(EvalFrame::Int64Shift(_)) => current_cell.int_value().map(ReadyFrame::Int64Shift),
            Some(EvalFrame::Int64(_)) => current_cell.int64_value().map(ReadyFrame::Int64),
            Some(EvalFrame::Float64(_)) => current_cell.float64_value().map(ReadyFrame::Float64),
            Some(EvalFrame::Float32(_)) => current_cell.float32_value().map(ReadyFrame::Float32),
            Some(EvalFrame::Bytes(_))
                if matches!(
                    self.cold_node(current),
                    Some(Node::Bytes(_) | Node::MutableBytes(_))
                ) =>
            {
                Some(ReadyFrame::Bytes)
            }
            Some(EvalFrame::Conversion(frame)) => frame
                .kind
                .ready_cell_value(current_cell)
                .map(ReadyFrame::Conversion),
            _ => None,
        };
        let Some(ready) = ready else {
            return Ok(None);
        };

        let frame = stack.pop().expect("ready eval frame must have a frame");
        let result = match (frame, ready) {
            (EvalFrame::Int(frame), ReadyFrame::Int(value)) => {
                self.finish_int_frame(frame, value, stack)?
            }
            (EvalFrame::Int64(frame), ReadyFrame::Int64(value)) => {
                self.finish_int64_frame(frame, value, stack)?
            }
            (EvalFrame::Int64Shift(frame), ReadyFrame::Int64Shift(value)) => {
                let next = frame.x;
                stack.push(EvalFrame::Int64(Int64Frame {
                    redex: frame.redex,
                    profile_head: frame.profile_head,
                    kind: Int64FrameKind::ShiftFirst {
                        op: frame.op,
                        y: value,
                    },
                }));
                if self.profile.is_some() {
                    self.profile_eval_frame_push("Int64");
                }
                (next, 0)
            }
            (EvalFrame::Float64(frame), ReadyFrame::Float64(value)) => {
                self.finish_float64_frame(frame, value, stack)?
            }
            (EvalFrame::Float32(frame), ReadyFrame::Float32(value)) => {
                self.finish_float32_frame(frame, value, stack)?
            }
            (EvalFrame::Bytes(frame), ReadyFrame::Bytes) => {
                self.finish_bytes_frame(frame, current, stack)?
            }
            (EvalFrame::Conversion(frame), ReadyFrame::Conversion(value)) => {
                self.finish_conversion_frame(frame, value)?
            }
            _ => unreachable!("ready eval frame kind changed before pop"),
        };
        Ok(Some(result))
    }

    fn finish_whnf_eval_frame(
        &mut self,
        stack: &mut EvalFrameStack,
        current: NodeId,
    ) -> Result<Option<(NodeId, usize)>, EvalError> {
        let Some(frame) = stack.pop() else {
            return Ok(None);
        };
        let result = match frame {
            EvalFrame::Whnf(frame) => self.finish_whnf_frame(frame, current)?,
            EvalFrame::Int(_) => return Err(EvalError::ExpectedInt(current)),
            EvalFrame::Int64Shift(_) => return Err(EvalError::ExpectedInt(current)),
            EvalFrame::Int64(_) => return Err(EvalError::ExpectedInt64(current)),
            EvalFrame::Float64(_) => return Err(EvalError::ExpectedFloat64(current)),
            EvalFrame::Float32(_) => return Err(EvalError::ExpectedFloat32(current)),
            EvalFrame::Bytes(_) => return Err(self.expected_bytes_error(current)),
            EvalFrame::Conversion(frame) => return Err(frame.kind.expected_error(current)),
        };
        Ok(Some(result))
    }

    fn resolve_for_whnf(
        &mut self,
        root: NodeId,
        profile_resolve: bool,
    ) -> Result<NodeId, EvalError> {
        if profile_resolve {
            self.resolve_profiled(root)
        } else {
            self.resolve(root)
        }
    }

    fn reduce_whnf_from(
        &mut self,
        root: NodeId,
        limit: usize,
        profile_resolve: bool,
        whnf_frames: bool,
    ) -> Result<(NodeId, usize), EvalError> {
        self.reduce_depth += 1;
        let result = self.reduce_whnf_from_inner(root, limit, profile_resolve, whnf_frames);
        self.reduce_depth -= 1;
        result
    }

    fn reduce_whnf_from_inner(
        &mut self,
        mut root: NodeId,
        limit: usize,
        profile_resolve: bool,
        whnf_frames: bool,
    ) -> Result<(NodeId, usize), EvalError> {
        if !whnf_frames {
            return self.reduce_whnf_from_stack(root, limit, profile_resolve);
        }

        let mut steps = 0;
        let mut frame_stack = EvalFrameStack::default();
        let mut eval_spine = EvalSpine::default();
        let mut persistent_spine = PersistentSpine::default();
        let mut persistent_active = false;
        let mut scratch_args = Vec::new();
        let mut scratch_apps = Vec::new();
        while steps < limit {
            self.maybe_collect_garbage_between_steps(
                root,
                &frame_stack,
                &eval_spine,
                &persistent_spine,
                &scratch_args,
                &scratch_apps,
                None,
            )?;
            let mut current = self.resolve_for_whnf(root, profile_resolve)?;
            if let Some((next, reductions)) =
                self.finish_ready_eval_frame(&mut frame_stack, current)?
            {
                steps += reductions;
                self.reductions += reductions;
                if steps >= limit {
                    return Err(EvalError::StepLimit { limit });
                }
                root = next;
                persistent_active = false;
                persistent_spine.clear();
                continue;
            }

            if !whnf_frames {
                if !persistent_active {
                    persistent_spine.clear();
                }
                let head =
                    self.fill_persistent_spine(current, &mut persistent_spine, profile_resolve)?;
                match self.persistent_eval_step(
                    head,
                    &mut persistent_spine,
                    &mut frame_stack,
                    &mut scratch_args,
                    limit - steps,
                )? {
                    PersistentStep::Reduced { node, reductions } => {
                        steps += reductions;
                        self.reductions += reductions;
                        root = node;
                        persistent_active = true;
                        continue;
                    }
                    PersistentStep::Force { node } => {
                        root = node;
                        persistent_active = false;
                        persistent_spine.clear();
                        continue;
                    }
                    PersistentStep::Whnf { node } => {
                        let Some((next, reductions)) =
                            self.finish_whnf_eval_frame(&mut frame_stack, node)?
                        else {
                            return Ok((node, steps));
                        };
                        steps += reductions;
                        self.reductions += reductions;
                        if steps >= limit {
                            return Err(EvalError::StepLimit { limit });
                        }
                        root = next;
                        persistent_active = false;
                        persistent_spine.clear();
                        continue;
                    }
                    PersistentStep::Fallback { root: next_root } => {
                        if self.profile.is_some() {
                            self.profile_persistent_fallback();
                        }
                        root = next_root;
                        persistent_active = false;
                        persistent_spine.clear();
                        current = self.resolve_for_whnf(root, profile_resolve)?;
                    }
                }
            }

            if whnf_frames {
                if let Some((frame, next)) = self.begin_whnf_force_frame(current)? {
                    if self.profile.is_some() {
                        self.profile_eval_frame_push("Whnf");
                    }
                    frame_stack.push(EvalFrame::Whnf(frame));
                    root = next;
                    continue;
                }
            }

            let Some(step) = self.eval_loop_step(
                current,
                limit - steps,
                &mut eval_spine,
                &mut scratch_args,
                &mut scratch_apps,
                &mut frame_stack,
                true,
            )?
            else {
                let Some((next, reductions)) =
                    self.finish_whnf_eval_frame(&mut frame_stack, current)?
                else {
                    return Ok((current, steps));
                };
                steps += reductions;
                self.reductions += reductions;
                if steps >= limit {
                    return Err(EvalError::StepLimit { limit });
                }
                root = next;
                continue;
            };
            steps += step.reductions;
            self.reductions += step.reductions;
            root = step.node;
        }
        Err(EvalError::StepLimit { limit })
    }

    fn reduce_whnf_from_stack(
        &mut self,
        mut current: NodeId,
        limit: usize,
        profile_resolve: bool,
    ) -> Result<(NodeId, usize), EvalError> {
        let mut steps = 0;
        let mut stack = EvalStack::default();
        let mut fallback_frame_stack = EvalFrameStack::default();
        let mut eval_spine = EvalSpine::default();
        let persistent_spine = PersistentSpine::default();
        let mut scratch_args = Vec::new();
        let mut scratch_apps = Vec::new();
        let profiling = self.profile.is_some();

        #[cfg(feature = "eval-phase-profile")]
        macro_rules! stack_phase_start {
            () => {
                profiling.then(Instant::now)
            };
        }
        #[cfg(not(feature = "eval-phase-profile"))]
        macro_rules! stack_phase_start {
            () => {
                ()
            };
        }
        #[cfg(feature = "eval-phase-profile")]
        macro_rules! record_stack_time {
            ($field:ident, $started:expr) => {{
                if let Some(started) = $started {
                    if let Some(profile) = self.profile.as_mut() {
                        profile.$field =
                            profile.$field.saturating_add(started.elapsed().as_nanos());
                    }
                }
            }};
        }
        #[cfg(not(feature = "eval-phase-profile"))]
        macro_rules! record_stack_time {
            ($field:ident, $started:expr) => {{
                let _ = &$started;
            }};
        }
        #[cfg(feature = "eval-phase-profile")]
        macro_rules! profile_stack_counter {
            ($field:ident) => {{
                if let Some(profile) = self.profile.as_mut() {
                    profile.$field = profile.$field.saturating_add(1);
                }
            }};
        }
        #[cfg(not(feature = "eval-phase-profile"))]
        macro_rules! profile_stack_counter {
            ($field:ident) => {};
        }
        macro_rules! profile_stack_step_result {
            ($step:expr) => {{
                #[cfg(feature = "eval-phase-profile")]
                {
                    profile_stack_counter!(stack_eval_step_calls);
                    match &$step {
                        StackStep::Reduced { .. } => profile_stack_counter!(stack_step_reduced),
                        StackStep::Whnf { .. } => profile_stack_counter!(stack_step_whnf),
                        StackStep::Fallback { .. } => profile_stack_counter!(stack_step_fallback),
                    }
                }
            }};
        }

        while steps < limit {
            profile_stack_counter!(stack_loop_iterations);
            let gc_started = stack_phase_start!();
            self.maybe_collect_garbage_between_steps(
                current,
                &fallback_frame_stack,
                &eval_spine,
                &persistent_spine,
                &scratch_args,
                &scratch_apps,
                Some(&stack),
            )?;
            record_stack_time!(stack_gc_check_nanos, gc_started);

            let resolve_started = stack_phase_start!();
            current = self.resolve_for_whnf(current, profile_resolve)?;
            record_stack_time!(stack_resolve_nanos, resolve_started);

            if stack.app_len() == 0 {
                profile_stack_counter!(stack_ready_checks);
                let ready_started = stack_phase_start!();
                if let Some((next, reductions)) =
                    self.finish_ready_stack_frame(&mut stack, current)?
                {
                    record_stack_time!(stack_ready_frame_nanos, ready_started);
                    profile_stack_counter!(stack_ready_successes);
                    steps += reductions;
                    self.reductions += reductions;
                    if steps >= limit {
                        return Err(EvalError::StepLimit { limit });
                    }
                    current = next;
                    continue;
                }
                record_stack_time!(stack_ready_frame_nanos, ready_started);
            }

            let descent_started = stack_phase_start!();
            while let Some(fun) = self.app_fun(current) {
                if profiling {
                    self.profile_stack_descent_push();
                }
                stack.push_app(current);
                current = self.resolve_for_whnf(fun, profile_resolve)?;
            }
            record_stack_time!(stack_descent_nanos, descent_started);

            if stack.app_len() == 0 {
                profile_stack_counter!(stack_ready_checks);
                let ready_started = stack_phase_start!();
                if let Some((next, reductions)) =
                    self.finish_ready_stack_frame(&mut stack, current)?
                {
                    record_stack_time!(stack_ready_frame_nanos, ready_started);
                    profile_stack_counter!(stack_ready_successes);
                    steps += reductions;
                    self.reductions += reductions;
                    if steps >= limit {
                        return Err(EvalError::StepLimit { limit });
                    }
                    current = next;
                    continue;
                }
                record_stack_time!(stack_ready_frame_nanos, ready_started);
            }

            let step_started = stack_phase_start!();
            let step = self.stack_eval_step(
                current,
                &mut stack,
                &mut scratch_args,
                limit - steps,
                profile_resolve,
            )?;
            record_stack_time!(stack_eval_step_nanos, step_started);
            profile_stack_step_result!(step);

            match step {
                StackStep::Reduced { node, reductions } => {
                    steps += reductions;
                    self.reductions += reductions;
                    if steps >= limit {
                        return Err(EvalError::StepLimit { limit });
                    }
                    current = node;
                }
                StackStep::Whnf {
                    node,
                    head,
                    reductions,
                } => {
                    steps += reductions;
                    self.reductions += reductions;
                    if steps >= limit {
                        return Err(EvalError::StepLimit { limit });
                    }
                    let whnf_started = stack_phase_start!();
                    let value = if stack.has_frame_below_apps() {
                        self.rethread_stack_app_segment(&mut stack, head)?
                    } else {
                        node
                    };
                    let Some((next, reductions)) =
                        self.finish_whnf_stack_frame(&mut stack, value)?
                    else {
                        record_stack_time!(stack_whnf_finish_nanos, whnf_started);
                        return Ok((value, steps));
                    };
                    record_stack_time!(stack_whnf_finish_nanos, whnf_started);
                    steps += reductions;
                    self.reductions += reductions;
                    if steps >= limit {
                        return Err(EvalError::StepLimit { limit });
                    }
                    current = next;
                }
                StackStep::Fallback {
                    root,
                    head,
                    reductions,
                } => {
                    steps += reductions;
                    self.reductions += reductions;
                    if steps >= limit {
                        return Err(EvalError::StepLimit { limit });
                    }
                    if self.profile.is_some() {
                        self.profile_persistent_fallback();
                    }
                    let root = if stack.app_len() == 0 {
                        root
                    } else {
                        self.rethread_stack_app_segment(&mut stack, head)?
                    };
                    let Some(step) = self.eval_loop_step(
                        root,
                        limit - steps,
                        &mut eval_spine,
                        &mut scratch_args,
                        &mut scratch_apps,
                        &mut fallback_frame_stack,
                        false,
                    )?
                    else {
                        if stack.top_is_frame() {
                            let Some((next, reductions)) =
                                self.finish_whnf_stack_frame(&mut stack, root)?
                            else {
                                return Ok((root, steps));
                            };
                            steps += reductions;
                            self.reductions += reductions;
                            if steps >= limit {
                                return Err(EvalError::StepLimit { limit });
                            }
                            current = next;
                            continue;
                        }
                        return Ok((root, steps));
                    };
                    debug_assert!(
                        fallback_frame_stack.peek().is_none(),
                        "strict_markers=false fallback step should not keep frames"
                    );
                    steps += step.reductions;
                    self.reductions += step.reductions;
                    if steps >= limit {
                        return Err(EvalError::StepLimit { limit });
                    }
                    current = step.node;
                    eval_spine.clear();
                }
            }
        }
        Err(EvalError::StepLimit { limit })
    }

    fn float64_result_node(&mut self, result: Float64Result) -> NodeId {
        match result {
            Float64Result::Float(n) => self.push_node(Node::Float64(n)),
            Float64Result::Bool(b) => self.prim(if b { "A" } else { "K" }),
        }
    }

    fn finish_float64_frame(
        &mut self,
        frame: Float64Frame,
        value: f64,
        stack: &mut EvalFrameStack,
    ) -> Result<(NodeId, usize), EvalError> {
        let node = match frame.kind {
            Float64FrameKind::BinSecond { op, x } => {
                let next = x;
                let next_frame = Float64Frame {
                    redex: frame.redex,
                    profile_head: frame.profile_head,
                    kind: Float64FrameKind::BinFirst { op, y: value },
                };
                if self.profile.is_some() {
                    self.profile_eval_frame_push("Float64");
                }
                stack.push(EvalFrame::Float64(next_frame));
                return Ok((next, 0));
            }
            Float64FrameKind::BinFirst { op, y } => self.float64_result_node(op.apply(value, y)),
            Float64FrameKind::Un { op } => self.push_node(Node::Float64(op.apply(value))),
        };

        if frame.profile_head.is_some() {
            self.profile_reduction(frame.profile_head, 1);
        }
        let node = self.apply_strict_redex(frame.redex, node)?;
        Ok((node, 1))
    }

    fn float32_result_node(&mut self, result: Float32Result) -> NodeId {
        match result {
            Float32Result::Float(n) => self.push_node(Node::Float32(n)),
            Float32Result::Bool(b) => self.prim(if b { "A" } else { "K" }),
        }
    }

    fn finish_float32_frame(
        &mut self,
        frame: Float32Frame,
        value: f32,
        stack: &mut EvalFrameStack,
    ) -> Result<(NodeId, usize), EvalError> {
        let node = match frame.kind {
            Float32FrameKind::BinSecond { op, x } => {
                let next = x;
                let next_frame = Float32Frame {
                    redex: frame.redex,
                    profile_head: frame.profile_head,
                    kind: Float32FrameKind::BinFirst { op, y: value },
                };
                if self.profile.is_some() {
                    self.profile_eval_frame_push("Float32");
                }
                stack.push(EvalFrame::Float32(next_frame));
                return Ok((next, 0));
            }
            Float32FrameKind::BinFirst { op, y } => self.float32_result_node(op.apply(value, y)),
            Float32FrameKind::Un { op } => self.push_node(Node::Float32(op.apply(value))),
        };

        if frame.profile_head.is_some() {
            self.profile_reduction(frame.profile_head, 1);
        }
        let node = self.apply_strict_redex(frame.redex, node)?;
        Ok((node, 1))
    }

    fn bytes_bin_result_node(
        &mut self,
        op: BytesBinOp,
        x: NodeId,
        y: NodeId,
    ) -> Result<NodeId, EvalError> {
        let node = match op {
            BytesBinOp::Append => {
                let mut bytes = self.bytes(x)?.to_vec();
                bytes.extend(self.bytes(y)?);
                self.push_node(Node::bytes(bytes))
            }
            BytesBinOp::AppendDot => {
                let mut bytes = self.bytes(x)?.to_vec();
                bytes.push(b'.');
                bytes.extend(self.bytes(y)?);
                self.push_node(Node::bytes(bytes))
            }
            BytesBinOp::Eq
            | BytesBinOp::Ne
            | BytesBinOp::Lt
            | BytesBinOp::Le
            | BytesBinOp::Gt
            | BytesBinOp::Ge
            | BytesBinOp::Cmp => {
                let cmp = self.bytes(x)?.cmp(self.bytes(y)?);
                match op {
                    BytesBinOp::Eq => self.prim(if cmp == Ordering::Equal { "A" } else { "K" }),
                    BytesBinOp::Ne => self.prim(if cmp != Ordering::Equal { "A" } else { "K" }),
                    BytesBinOp::Lt => self.prim(if cmp == Ordering::Less { "A" } else { "K" }),
                    BytesBinOp::Le => self.prim(if cmp != Ordering::Greater { "A" } else { "K" }),
                    BytesBinOp::Gt => self.prim(if cmp == Ordering::Greater { "A" } else { "K" }),
                    BytesBinOp::Ge => self.prim(if cmp != Ordering::Less { "A" } else { "K" }),
                    BytesBinOp::Cmp => self.ordering(cmp),
                    BytesBinOp::Append | BytesBinOp::AppendDot => unreachable!(),
                }
            }
        };
        Ok(node)
    }

    fn finish_bytes_frame(
        &mut self,
        frame: BytesFrame,
        value: NodeId,
        stack: &mut EvalFrameStack,
    ) -> Result<(NodeId, usize), EvalError> {
        let node = match frame.kind {
            BytesFrameKind::BinSecond { op, x } => {
                let next = x;
                let next_frame = BytesFrame {
                    redex: frame.redex,
                    profile_head: frame.profile_head,
                    kind: BytesFrameKind::BinFirst { op, y: value },
                };
                if self.profile.is_some() {
                    self.profile_eval_frame_push("Bytes");
                }
                stack.push(EvalFrame::Bytes(next_frame));
                return Ok((next, 0));
            }
            BytesFrameKind::BinFirst { op, y } => self.bytes_bin_result_node(op, value, y)?,
        };

        if frame.profile_head.is_some() {
            self.profile_reduction(frame.profile_head, 1);
        }
        let node = self.apply_strict_redex(frame.redex, node)?;
        Ok((node, 1))
    }

    fn int_binop(
        &mut self,
        name: &str,
        args: &[NodeId],
    ) -> Result<Option<(usize, NodeId)>, EvalError> {
        let Some(op) = IntBinOp::from_prim(name) else {
            return Ok(None);
        };
        let x = self.eval_int(args[0])?;
        let y = self.eval_int(args[1])?;
        let result = op
            .apply(x, y)
            .map_err(|err| self.arithmetic_eval_error(err))?;
        let node = match result {
            IntResult::Int(n) => self.int(n),
            IntResult::Bool(b) => self.prim(if b { "A" } else { "K" }),
            IntResult::Ordering(ord) => self.ordering(ord),
        };
        Ok(Some((2, node)))
    }

    fn int_unop(
        &mut self,
        name: &str,
        args: &[NodeId],
    ) -> Result<Option<(usize, NodeId)>, EvalError> {
        let Some(op) = IntUnOp::from_prim(name) else {
            return Ok(None);
        };
        let x = self.eval_int(args[0])?;
        let n = op.apply(x).map_err(|err| self.arithmetic_eval_error(err))?;
        let node = self.int(n);
        Ok(Some((1, node)))
    }

    fn int64_binop(
        &mut self,
        name: &str,
        args: &[NodeId],
    ) -> Result<Option<(usize, NodeId)>, EvalError> {
        let Some(op) = Int64BinOp::from_prim(name) else {
            return Ok(None);
        };
        let x = self.eval_int64(args[0])?;
        let y = if op.rhs_is_shift() {
            self.eval_int(args[1])?
        } else {
            self.eval_int64(args[1])?
        };
        let result = op
            .apply(x, y)
            .map_err(|err| self.arithmetic_eval_error(err))?;
        let node = match result {
            Int64Result::Int64(n) => self.push_node(Node::Int64(n)),
            Int64Result::Bool(b) => self.prim(if b { "A" } else { "K" }),
            Int64Result::Ordering(ord) => self.ordering(ord),
        };
        Ok(Some((2, node)))
    }

    fn int64_unop(
        &mut self,
        name: &str,
        args: &[NodeId],
    ) -> Result<Option<(usize, NodeId)>, EvalError> {
        let Some(op) = Int64UnOp::from_prim(name) else {
            return Ok(None);
        };
        let x = self.eval_int64(args[0])?;
        let result = op.apply(x).map_err(|err| self.arithmetic_eval_error(err))?;
        let node = match result {
            Int64UnResult::Int64(n) => self.push_node(Node::Int64(n)),
            Int64UnResult::Int(n) => self.int(n),
        };
        Ok(Some((1, node)))
    }

    fn int_conversion(
        &mut self,
        name: &str,
        args: &[NodeId],
    ) -> Result<Option<(usize, NodeId)>, EvalError> {
        let node = match name {
            "itoI" | "utoU" => {
                let n = self.eval_int(args[0])?;
                self.push_node(Node::Int64(n))
            }
            "Itoi" | "Utou" => {
                let n = self.eval_int64(args[0])?;
                self.int(n)
            }
            _ => return Ok(None),
        };
        Ok(Some((1, node)))
    }

    fn float64_binop(
        &mut self,
        name: &str,
        args: &[NodeId],
    ) -> Result<Option<(usize, NodeId)>, EvalError> {
        let Some(op) = Float64BinOp::from_prim(name) else {
            return Ok(None);
        };
        let x = self.eval_float64(args[0])?;
        let y = self.eval_float64(args[1])?;
        let node = match op.apply(x, y) {
            Float64Result::Float(n) => self.push_node(Node::Float64(n)),
            Float64Result::Bool(b) => self.prim(if b { "A" } else { "K" }),
        };
        Ok(Some((2, node)))
    }

    fn float64_unop(
        &mut self,
        name: &str,
        args: &[NodeId],
    ) -> Result<Option<(usize, NodeId)>, EvalError> {
        let Some(op) = Float64UnOp::from_prim(name) else {
            return Ok(None);
        };
        let x = self.eval_float64(args[0])?;
        let node = self.push_node(Node::Float64(op.apply(x)));
        Ok(Some((1, node)))
    }

    fn float32_binop(
        &mut self,
        name: &str,
        args: &[NodeId],
    ) -> Result<Option<(usize, NodeId)>, EvalError> {
        let Some(op) = Float32BinOp::from_prim(name) else {
            return Ok(None);
        };
        let x = self.eval_float32(args[0])?;
        let y = self.eval_float32(args[1])?;
        let node = match op.apply(x, y) {
            Float32Result::Float(n) => self.push_node(Node::Float32(n)),
            Float32Result::Bool(b) => self.prim(if b { "A" } else { "K" }),
        };
        Ok(Some((2, node)))
    }

    fn float32_unop(
        &mut self,
        name: &str,
        args: &[NodeId],
    ) -> Result<Option<(usize, NodeId)>, EvalError> {
        let Some(op) = Float32UnOp::from_prim(name) else {
            return Ok(None);
        };
        let x = self.eval_float32(args[0])?;
        let node = self.push_node(Node::Float32(op.apply(x)));
        Ok(Some((1, node)))
    }

    fn float_conversion(
        &mut self,
        name: &str,
        args: &[NodeId],
    ) -> Result<Option<(usize, NodeId)>, EvalError> {
        let node = match name {
            "itod" => {
                let n = self.eval_int(args[0])?;
                self.push_node(Node::Float64(n as f64))
            }
            "utod" => {
                let n = self.eval_int(args[0])?;
                self.push_node(Node::Float64((n as u64) as f64))
            }
            "Itod" => {
                let n = self.eval_int64(args[0])?;
                self.push_node(Node::Float64(n as f64))
            }
            "dtoi" => {
                let n = self.eval_float64(args[0])?;
                self.int(n as i64)
            }
            "itof" => {
                let n = self.eval_int(args[0])?;
                self.push_node(Node::Float32(n as f32))
            }
            "utof" => {
                let n = self.eval_int(args[0])?;
                self.push_node(Node::Float32((n as u64) as f32))
            }
            "Itof" => {
                let n = self.eval_int64(args[0])?;
                self.push_node(Node::Float32(n as f32))
            }
            "ftoi" => {
                let n = self.eval_float32(args[0])?;
                self.int(n as i64)
            }
            "dtof" => {
                let n = self.eval_float64(args[0])?;
                self.push_node(Node::Float32(n as f32))
            }
            "ftod" => {
                let n = self.eval_float32(args[0])?;
                self.push_node(Node::Float64(n as f64))
            }
            "toDbl" => {
                let n = self.eval_int64(args[0])?;
                self.push_node(Node::Float64(f64::from_bits(n as u64)))
            }
            "fromDbl" => {
                let n = self.eval_float64(args[0])?;
                self.push_node(Node::Int64(n.to_bits() as i64))
            }
            "toFlt" => {
                let n = self.eval_int(args[0])?;
                self.push_node(Node::Float32(f32::from_bits(n as u32)))
            }
            "fromFlt" => {
                let n = self.eval_float32(args[0])?;
                self.int((n.to_bits() as i32) as i64)
            }
            _ => return Ok(None),
        };
        Ok(Some((1, node)))
    }

    fn node_trace_summary(&self, id: NodeId) -> String {
        let Some(cell) = self.nodes.get(id.index()).copied() else {
            return format!("{id:?}:<missing>");
        };
        let node = cell.to_node(&self.cold_nodes);
        match node {
            Node::Int(n) => format!("{id:?}:Int({n})"),
            Node::Int64(n) => format!("{id:?}:Int64({n})"),
            Node::Ptr(ptr) => format!("{id:?}:Ptr({ptr})"),
            Node::RawFunPtr(ptr) => format!("{id:?}:RawFunPtr({ptr})"),
            Node::ThreadId(n) => format!("{id:?}:ThreadId({n})"),
            Node::Prim(name) => format!("{id:?}:Prim({})", name.name()),
            Node::Ffi(name) => format!("{id:?}:Ffi({name})"),
            Node::Bytes(bytes) => format!("{id:?}:Bytes(len={})", bytes.len()),
            Node::BytesView(view) => format!(
                "{id:?}:BytesView(base={:?}, offset={}, len={})",
                view.base, view.offset, view.len
            ),
            Node::MutableBytes(bytes) => {
                format!(
                    "{id:?}:MutableBytes(size={}, capacity={})",
                    bytes.size, bytes.capacity
                )
            }
            Node::ForeignPtr(ptr) => format!(
                "{id:?}:ForeignPtr(ptr={}, offset={}, bytes={})",
                ptr.ptr,
                ptr.offset,
                ptr.bytes.as_ref().map_or(0, Vec::len)
            ),
            Node::App(fun, arg) => format!("{id:?}:App({fun:?},{arg:?})"),
            Node::Indir(target) => format!("{id:?}:Indir({target:?})"),
            Node::Free(next) => format!("{id:?}:Free({next:?})"),
            Node::BigInt(bytes) => format!("{id:?}:BigInt(len={})", bytes.len()),
            Node::Array(items) => format!("{id:?}:Array(len={})", items.len()),
            Node::Float64(n) => format!("{id:?}:Float64({n})"),
            Node::Float32(n) => format!("{id:?}:Float32({n})"),
            Node::Weak(_) => format!("{id:?}:Weak"),
            Node::MVar(_) => format!("{id:?}:MVar"),
            Node::JsCall(call) => format!("{id:?}:JsCall(tags={})", call.tags),
            Node::JsWrap { tags } => format!("{id:?}:JsWrap(tags={tags})"),
            Node::FunPtr(name) => format!("{id:?}:FunPtr({name})"),
            Node::Tick(bytes) => format!("{id:?}:Tick(len={})", bytes.len()),
        }
    }

    fn trace_invalid_op_error(
        &self,
        domain: &str,
        name: &str,
        args: &[NodeId],
        err: EvalError,
    ) -> EvalError {
        if matches!(err, EvalError::InvalidByteString)
            && std::env::var_os("MHS_TRACE_INVALID_BYTES").is_some()
        {
            eprintln!(
                "invalid bytes context: domain={domain} name={name} reductions={}",
                self.reductions
            );
            for (idx, arg) in args.iter().take(8).enumerate() {
                eprintln!("  arg{idx}: {}", self.node_trace_summary(*arg));
            }
            if args.len() > 8 {
                eprintln!("  ... {} more args", args.len() - 8);
            }
        }
        err
    }

    fn pointer_conversion(
        &mut self,
        name: &str,
        args: &[NodeId],
    ) -> Result<Option<(usize, NodeId)>, EvalError> {
        let value = match name {
            "toInt" => Node::Int(self.eval_pointer_value(args[0])?),
            "toPtr" => Node::Ptr(self.eval_pointer_value(args[0])?),
            "toFunPtr" => Node::RawFunPtr(self.eval_pointer_value(args[0])?),
            _ => return Ok(None),
        };
        Ok(Some((1, self.push_value_node(value))))
    }

    fn foreign_ptr_op(
        &mut self,
        name: &str,
        args: &[NodeId],
    ) -> Result<Option<(usize, NodeId)>, EvalError> {
        match self.foreign_ptr_op_inner(name, args) {
            Ok(result) => Ok(result),
            Err(err) => Err(self.trace_invalid_op_error("foreign_ptr_op", name, args, err)),
        }
    }

    fn foreign_ptr_op_inner(
        &mut self,
        name: &str,
        args: &[NodeId],
    ) -> Result<Option<(usize, NodeId)>, EvalError> {
        let rewrite = match name {
            "fp+" => {
                let foreign_ptr = self.eval_foreign_ptr_id(args[0])?;
                let offset = int_to_usize(self.eval_int(args[1])?)?;
                Some((2, self.offset_foreign_ptr(foreign_ptr, offset)?))
            }
            "fp2bs" => {
                let foreign_ptr = self.eval_foreign_ptr_id(args[0])?;
                let len = int_to_usize(self.eval_int(args[1])?)?;
                Some((2, self.foreign_ptr_to_bytes(foreign_ptr, len)?))
            }
            "fpnew" if args.len() >= 2 => {
                let ptr = self.eval_pointer_value(args[0])?;
                let node = self.foreign_ptr_node(None, 0, ptr);
                let foreign_ptr = self.push_node(node);
                Some((2, self.pair(foreign_ptr, args[1])))
            }
            "fpfin" if args.len() >= 3 => {
                let foreign_ptr = self.eval_foreign_ptr_id(args[1])?;
                self.set_foreign_ptr_finalizer(foreign_ptr, args[0])?;
                let unit = self.prim("I");
                Some((3, self.pair(unit, args[2])))
            }
            "fpfin" => {
                let foreign_ptr = self.eval_foreign_ptr_id(args[1])?;
                self.set_foreign_ptr_finalizer(foreign_ptr, args[0])?;
                Some((2, self.prim("I")))
            }
            _ => None,
        };
        Ok(rewrite)
    }

    fn foreign_ptr_unop(
        &mut self,
        name: &str,
        args: &[NodeId],
    ) -> Result<Option<(usize, NodeId)>, EvalError> {
        match self.foreign_ptr_unop_inner(name, args) {
            Ok(result) => Ok(result),
            Err(err) => Err(self.trace_invalid_op_error("foreign_ptr_unop", name, args, err)),
        }
    }

    fn foreign_ptr_unop_inner(
        &mut self,
        name: &str,
        args: &[NodeId],
    ) -> Result<Option<(usize, NodeId)>, EvalError> {
        let node = match name {
            "bs2fp" => {
                let bytes_id = self.eval_bytes_id(args[0])?;
                let bytes = self.bytes(bytes_id)?.to_vec();
                let ptr = self.pointer_for_node(bytes_id, 0)?;
                let node = self.foreign_ptr_node(Some(bytes), 0, ptr);
                self.push_node(node)
            }
            "fp2p" => {
                let foreign_ptr = self.eval_foreign_ptr_id(args[0])?;
                let ptr = self.foreign_ptr_value(foreign_ptr)?;
                self.push_node(Node::Ptr(ptr))
            }
            "fpnew" => {
                let ptr = self.eval_pointer_value(args[0])?;
                let node = self.foreign_ptr_node(None, 0, ptr);
                self.push_node(node)
            }
            _ => return Ok(None),
        };
        Ok(Some((1, node)))
    }

    fn array_op(
        &mut self,
        name: &str,
        args: &[NodeId],
    ) -> Result<Option<(usize, NodeId)>, EvalError> {
        let rewrite = match name {
            "A.alloc" if args.len() >= 3 => {
                let len = int_to_usize(self.eval_int(args[0])?)?;
                let array = self.push_node(Node::array(vec![args[1]; len]));
                Some((3, self.pair(array, args[2])))
            }
            "A.alloc" => {
                let len = int_to_usize(self.eval_int(args[0])?)?;
                Some((2, self.push_node(Node::array(vec![args[1]; len]))))
            }
            "A.read" if args.len() >= 3 => {
                let array = self.eval_array_id(args[0])?;
                let index = int_to_usize(self.eval_int(args[1])?)?;
                let item = *self
                    .array(array)?
                    .get(index)
                    .ok_or(EvalError::InvalidArray)?;
                Some((3, self.pair(item, args[2])))
            }
            "A.read" if args.len() >= 2 => {
                let array = self.eval_array_id(args[0])?;
                let index = int_to_usize(self.eval_int(args[1])?)?;
                let item = *self
                    .array(array)?
                    .get(index)
                    .ok_or(EvalError::InvalidArray)?;
                Some((2, item))
            }
            "A.write" if args.len() >= 4 => {
                let array = self.eval_array_id(args[0])?;
                let index = int_to_usize(self.eval_int(args[1])?)?;
                let items = self.array_mut(array)?;
                let slot = items.get_mut(index).ok_or(EvalError::InvalidArray)?;
                *slot = args[2];
                let unit = self.prim("I");
                Some((4, self.pair(unit, args[3])))
            }
            "A.write" if args.len() >= 3 => {
                let array = self.eval_array_id(args[0])?;
                let index = int_to_usize(self.eval_int(args[1])?)?;
                let items = self.array_mut(array)?;
                let slot = items.get_mut(index).ok_or(EvalError::InvalidArray)?;
                *slot = args[2];
                Some((3, self.prim("I")))
            }
            "A.trunc" if args.len() >= 3 => {
                let array = self.eval_array_id(args[0])?;
                let len = int_to_usize(self.eval_int(args[1])?)?;
                let items = self.array_mut(array)?;
                if len >= items.len() {
                    return Err(EvalError::InvalidArray);
                }
                items.truncate(len);
                let unit = self.prim("I");
                Some((3, self.pair(unit, args[2])))
            }
            "A.trunc" if args.len() >= 2 => {
                let array = self.eval_array_id(args[0])?;
                let len = int_to_usize(self.eval_int(args[1])?)?;
                let items = self.array_mut(array)?;
                if len >= items.len() {
                    return Err(EvalError::InvalidArray);
                }
                items.truncate(len);
                Some((2, self.prim("I")))
            }
            "A.==" => {
                let x = self.eval_array_id(args[0])?;
                let y = self.eval_array_id(args[1])?;
                Some((2, self.prim(if x == y { "A" } else { "K" })))
            }
            _ => None,
        };
        Ok(rewrite)
    }

    fn array_unop(
        &mut self,
        name: &str,
        args: &[NodeId],
    ) -> Result<Option<(usize, NodeId)>, EvalError> {
        let node = match name {
            "A.copy" if args.len() >= 2 => {
                let array = self.eval_array_id(args[0])?;
                let items = self.array(array)?.to_vec();
                let copy = self.push_node(Node::array(items));
                self.pair(copy, args[1])
            }
            "A.copy" => {
                let array = self.eval_array_id(args[0])?;
                let items = self.array(array)?.to_vec();
                self.push_node(Node::array(items))
            }
            "A.size" if args.len() >= 2 => {
                let array = self.eval_array_id(args[0])?;
                let len =
                    i64::try_from(self.array(array)?.len()).map_err(|_| EvalError::Overflow)?;
                let size = self.int(len);
                self.pair(size, args[1])
            }
            "A.size" => {
                let array = self.eval_array_id(args[0])?;
                let len =
                    i64::try_from(self.array(array)?.len()).map_err(|_| EvalError::Overflow)?;
                self.int(len)
            }
            _ => return Ok(None),
        };
        let used = if matches!(name, "A.copy" | "A.size") && args.len() >= 2 {
            2
        } else {
            1
        };
        Ok(Some((used, node)))
    }

    fn stable_ptr_op(
        &mut self,
        name: &str,
        args: &[NodeId],
    ) -> Result<Option<(usize, NodeId)>, EvalError> {
        let rewrite = match name {
            "SPnew" if args.len() >= 2 => {
                let handle = self.new_stable_ptr(args[0])?;
                Some((2, self.pair(handle, args[1])))
            }
            "SPderef" if args.len() >= 2 => {
                let handle = self.stable_ptr_handle(args[0])?;
                let value = self.deref_stable_ptr(handle)?;
                Some((2, self.pair(value, args[1])))
            }
            "SPfree" if args.len() >= 2 => {
                let handle = self.stable_ptr_handle(args[0])?;
                self.free_stable_ptr(handle)?;
                let unit = self.prim("I");
                Some((2, self.pair(unit, args[1])))
            }
            _ => None,
        };
        Ok(rewrite)
    }

    fn stable_ptr_unop(
        &mut self,
        name: &str,
        args: &[NodeId],
    ) -> Result<Option<(usize, NodeId)>, EvalError> {
        let node = match name {
            "SPnew" => self.new_stable_ptr(args[0])?,
            "SPderef" => {
                let handle = self.stable_ptr_handle(args[0])?;
                self.deref_stable_ptr(handle)?
            }
            "SPfree" => {
                let handle = self.stable_ptr_handle(args[0])?;
                self.free_stable_ptr(handle)?;
                self.prim("I")
            }
            _ => return Ok(None),
        };
        Ok(Some((1, node)))
    }

    fn weak_ptr_op(
        &mut self,
        name: &str,
        args: &[NodeId],
    ) -> Result<Option<(usize, NodeId)>, EvalError> {
        let rewrite = match name {
            "Wknewfin" if args.len() >= 4 => {
                let weak = self.new_weak_ptr(args[0], args[1], Some(args[2]));
                Some((4, self.pair(weak, args[3])))
            }
            "Wknewfin" if args.len() >= 3 => {
                let weak = self.new_weak_ptr(args[0], args[1], Some(args[2]));
                Some((3, weak))
            }
            "Wknew" if args.len() >= 3 => {
                let weak = self.new_weak_ptr(args[0], args[1], None);
                Some((3, self.pair(weak, args[2])))
            }
            "Wknew" if args.len() >= 2 => {
                let weak = self.new_weak_ptr(args[0], args[1], None);
                Some((2, weak))
            }
            "Wkderef" if args.len() >= 2 => {
                let value = self.deref_weak_ptr(args[0])?;
                Some((2, self.pair(value, args[1])))
            }
            "Wkfinal" if args.len() >= 2 => {
                self.finalize_weak_ptr(args[0])?;
                let unit = self.prim("I");
                Some((2, self.pair(unit, args[1])))
            }
            _ => None,
        };
        Ok(rewrite)
    }

    fn weak_ptr_unop(
        &mut self,
        name: &str,
        args: &[NodeId],
    ) -> Result<Option<(usize, NodeId)>, EvalError> {
        let node = match name {
            "Wkderef" => self.deref_weak_ptr(args[0])?,
            "Wkfinal" => {
                self.finalize_weak_ptr(args[0])?;
                self.prim("I")
            }
            _ => return Ok(None),
        };
        Ok(Some((1, node)))
    }

    fn bytes_op(
        &mut self,
        name: &str,
        args: &[NodeId],
    ) -> Result<Option<(usize, NodeId)>, EvalError> {
        match self.bytes_op_inner(name, args) {
            Ok(result) => Ok(result),
            Err(err) => Err(self.trace_invalid_op_error("bytes_op", name, args, err)),
        }
    }

    fn bytes_op_inner(
        &mut self,
        name: &str,
        args: &[NodeId],
    ) -> Result<Option<(usize, NodeId)>, EvalError> {
        macro_rules! invalid_bytes {
            ($($arg:tt)*) => {{
                if std::env::var_os("MHS_TRACE_INVALID_BYTES").is_some() {
                    eprintln!(
                        "invalid bytes op {name}: reductions={}",
                        self.reductions,
                    );
                    eprintln!($($arg)*);
                }
                EvalError::InvalidByteString
            }};
        }
        if let Some(op) = BytesBinOp::from_prim(name) {
            let x = self.eval_bytes_id(args[0])?;
            let y = self.eval_bytes_id(args[1])?;
            let node = self.bytes_bin_result_node(op, x, y)?;
            return Ok(Some((2, node)));
        }

        let rewrite = match name {
            "packCString" if args.len() >= 2 => {
                let ptr = self.eval_pointer_value(args[0])?;
                let bytes = self.read_c_string(ptr)?;
                let bytes = self.push_node(Node::bytes(bytes));
                Some((2, self.pair(bytes, args[1])))
            }
            "packCStringLen" if args.len() >= 3 => {
                let ptr = self.eval_pointer_value(args[0])?;
                let len = int_to_usize(self.eval_int(args[1])?)?;
                let bytes = self.read_pointer_bytes(ptr, len)?;
                let bytes = self.push_node(Node::bytes(bytes));
                Some((3, self.pair(bytes, args[2])))
            }
            "packCStringLen" => {
                let ptr = self.eval_pointer_value(args[0])?;
                let len = int_to_usize(self.eval_int(args[1])?)?;
                let bytes = self.read_pointer_bytes(ptr, len)?;
                Some((2, self.push_node(Node::bytes(bytes))))
            }
            "bsgrab" if args.len() >= 2 => {
                let ptr = self.eval_pointer_value(args[0])?;
                let bytes = self.read_c_string(ptr)?;
                let bytes = self.push_node(Node::bytes(bytes));
                Some((2, self.pair(bytes, args[1])))
            }
            "bsgrablen" if args.len() >= 3 => {
                let ptr = self.eval_pointer_value(args[0])?;
                let len = int_to_usize(self.eval_int(args[1])?)?;
                let bytes = self.read_pointer_bytes(ptr, len)?;
                let bytes = self.push_node(Node::bytes(bytes));
                Some((3, self.pair(bytes, args[2])))
            }
            "bsgrablen" => {
                let ptr = self.eval_pointer_value(args[0])?;
                let len = int_to_usize(self.eval_int(args[1])?)?;
                let bytes = self.read_pointer_bytes(ptr, len)?;
                Some((2, self.push_node(Node::bytes(bytes))))
            }
            "bsnew" if args.len() >= 3 => {
                let size = int_to_usize(self.eval_int(args[0])?)?;
                let capacity = int_to_usize(self.eval_int(args[1])?)?;
                let bytes = self.new_mutable_bytes(size, capacity)?;
                Some((3, self.pair(bytes, args[2])))
            }
            "bsnew" => {
                let size = int_to_usize(self.eval_int(args[0])?)?;
                let capacity = int_to_usize(self.eval_int(args[1])?)?;
                Some((2, self.new_mutable_bytes(size, capacity)?))
            }
            "bsread" if args.len() >= 3 => {
                let bytes = self.eval_bytes_id(args[0])?;
                let index = int_to_usize(self.eval_int(args[1])?)?;
                let (len, storage_len) = self.byte_prim_lengths(bytes)?;
                if index >= storage_len {
                    return Err(invalid_bytes!(
                        "bsread bytes={bytes:?} index={index} len={len} storage_len={storage_len}"
                    ));
                }
                let byte = self.read_byte_unchecked_prim(bytes, index)?;
                let byte = self.int(byte as i64);
                Some((3, self.pair(byte, args[2])))
            }
            "bsread" => {
                let bytes = self.eval_bytes_id(args[0])?;
                let index = int_to_usize(self.eval_int(args[1])?)?;
                let (len, storage_len) = self.byte_prim_lengths(bytes)?;
                if index >= storage_len {
                    return Err(invalid_bytes!(
                        "bsread bytes={bytes:?} index={index} len={len} storage_len={storage_len}"
                    ));
                }
                let byte = self.read_byte_unchecked_prim(bytes, index)?;
                Some((2, self.int(byte as i64)))
            }
            "bswrite" if args.len() >= 4 => {
                let bytes = self.eval_bytes_id(args[0])?;
                let index = int_to_usize(self.eval_int(args[1])?)?;
                let byte = self.eval_int(args[2])? as u8;
                let (len, storage_len) = self.byte_prim_lengths(bytes)?;
                if index >= storage_len {
                    return Err(invalid_bytes!(
                        "bswrite bytes={bytes:?} index={index} len={len} storage_len={storage_len}"
                    ));
                }
                self.write_byte_unchecked_prim(bytes, index, byte)?;
                let unit = self.prim("I");
                Some((4, self.pair(unit, args[3])))
            }
            "bswrite" if args.len() >= 3 => {
                let bytes = self.eval_bytes_id(args[0])?;
                let index = int_to_usize(self.eval_int(args[1])?)?;
                let byte = self.eval_int(args[2])? as u8;
                let (len, storage_len) = self.byte_prim_lengths(bytes)?;
                if index >= storage_len {
                    return Err(invalid_bytes!(
                        "bswrite bytes={bytes:?} index={index} len={len} storage_len={storage_len}"
                    ));
                }
                self.write_byte_unchecked_prim(bytes, index, byte)?;
                Some((3, self.prim("I")))
            }
            "bsfreeze" if args.len() >= 2 => {
                let bytes = self.eval_bytes_id(args[0])?;
                let bytes = self.freeze_bytes(bytes)?;
                Some((2, self.pair(bytes, args[1])))
            }
            "bsappbyte" if args.len() >= 3 => {
                let bytes = self.eval_bytes_id(args[0])?;
                let byte = self.eval_int(args[1])? as u8;
                self.append_byte(bytes, byte)?;
                let unit = self.prim("I");
                Some((3, self.pair(unit, args[2])))
            }
            "bsappbyte" => {
                let bytes = self.eval_bytes_id(args[0])?;
                let byte = self.eval_int(args[1])? as u8;
                self.append_byte(bytes, byte)?;
                Some((2, self.prim("I")))
            }
            "bsappchar" if args.len() >= 3 => {
                let bytes = self.eval_bytes_id(args[0])?;
                let encoded = modified_utf8(self.eval_int(args[1])?)?;
                self.append_bytes(bytes, &encoded)?;
                let unit = self.prim("I");
                Some((3, self.pair(unit, args[2])))
            }
            "bsappchar" => {
                let bytes = self.eval_bytes_id(args[0])?;
                let encoded = modified_utf8(self.eval_int(args[1])?)?;
                self.append_bytes(bytes, &encoded)?;
                Some((2, self.prim("I")))
            }
            "bsreplicate" => {
                let len = int_to_usize(self.eval_int(args[0])?)?;
                let byte = self.eval_int(args[1])? as u8;
                Some((2, self.push_node(Node::bytes(vec![byte; len]))))
            }
            "bsindex" => {
                let bytes_id = self.eval_bytes_id(args[0])?;
                let index = int_to_usize(self.eval_int(args[1])?)?;
                let bytes = self.bytes(bytes_id)?;
                let len = bytes.len();
                if index >= len {
                    return Err(invalid_bytes!("bsindex index={index} len={len}"));
                }
                let byte = bytes[index];
                Some((2, self.int(byte as i64)))
            }
            "bssubstr" if args.len() >= 3 => {
                let bytes_id = self.eval_bytes_id(args[0])?;
                let offset = int_to_usize(self.eval_int(args[1])?)?;
                let len = int_to_usize(self.eval_int(args[2])?)?;
                let bytes = self.bytes(bytes_id)?;
                let bytes_len = bytes.len();
                offset
                    .checked_add(len)
                    .filter(|end| *end <= bytes.len())
                    .ok_or_else(|| {
                        invalid_bytes!("bssubstr offset={offset} len={len} bytes_len={bytes_len}")
                    })?;
                let node = self.byte_slice_node(bytes_id, offset, len)?;
                Some((3, self.push_node(node)))
            }
            _ => None,
        };
        Ok(rewrite)
    }

    fn bytes_unop(
        &mut self,
        name: &str,
        args: &[NodeId],
    ) -> Result<Option<(usize, NodeId)>, EvalError> {
        match self.bytes_unop_inner(name, args) {
            Ok(result) => Ok(result),
            Err(err) => Err(self.trace_invalid_op_error("bytes_unop", name, args, err)),
        }
    }

    fn bytes_unop_inner(
        &mut self,
        name: &str,
        args: &[NodeId],
    ) -> Result<Option<(usize, NodeId)>, EvalError> {
        let node = match name {
            "packCString" => {
                let ptr = self.eval_pointer_value(args[0])?;
                let bytes = self.read_c_string(ptr)?;
                self.push_node(Node::bytes(bytes))
            }
            "bsgrab" => {
                let ptr = self.eval_pointer_value(args[0])?;
                let bytes = self.read_c_string(ptr)?;
                self.push_node(Node::bytes(bytes))
            }
            "bslength" => {
                let bytes_id = self.eval_bytes_id(args[0])?;
                let len =
                    i64::try_from(self.bytes(bytes_id)?.len()).map_err(|_| EvalError::Overflow)?;
                self.int(len)
            }
            "headUTF8" => {
                let bytes_id = self.eval_bytes_id(args[0])?;
                let (codepoint, _) = head_utf8(self.bytes(bytes_id)?)?;
                self.int(codepoint as i64)
            }
            "tailUTF8" => {
                let bytes_id = self.eval_bytes_id(args[0])?;
                let bytes = self.bytes(bytes_id)?;
                let (_, offset) = head_utf8(bytes)?;
                let len = bytes.len() - offset;
                let node = self.byte_slice_node(bytes_id, offset, len)?;
                self.push_node(node)
            }
            "bsunpack" => {
                let bytes = self.eval_bytes(args[0])?;
                let values = bytes.into_iter().map(i64::from);
                self.int_list(values)
            }
            "fromUTF8" => {
                let bytes = self.eval_bytes(args[0])?;
                let values = decode_utf8_string_bytes(&bytes)?;
                self.int_list(values.into_iter().map(i64::from))
            }
            "bsfreeze" => {
                let bytes = self.eval_bytes_id(args[0])?;
                self.freeze_bytes(bytes)?
            }
            _ => return Ok(None),
        };
        Ok(Some((1, node)))
    }

    fn ffi_call(
        &mut self,
        name: &str,
        args: &[NodeId],
    ) -> Result<Option<(usize, NodeId)>, EvalError> {
        match self.ffi_call_inner(name, args) {
            Ok(result) => Ok(result),
            Err(err) => Err(self.trace_invalid_op_error("ffi", name, args, err)),
        }
    }

    fn ffi_call_inner(
        &mut self,
        name: &str,
        args: &[NodeId],
    ) -> Result<Option<(usize, NodeId)>, EvalError> {
        if !args.is_empty() {
            if let Some(result) = self.zero_arity_ffi_result(name)? {
                let result = self.push_value_node(result);
                return Ok(Some((1, self.pair(result, args[0]))));
            }
        }
        if args.len() >= 2 && is_unary_math_ffi_candidate(name) {
            if let Some(result) = self.unary_math_ffi_result(name, args[0])? {
                let result = self.push_value_node(result);
                return Ok(Some((2, self.pair(result, args[1]))));
            }
        }
        let arity = ffi_arity(name).ok_or_else(|| EvalError::UnknownFfi(name.to_owned()))?;
        if args.len() < arity + 1 {
            return Ok(None);
        }

        let result = match name {
            name if errno_constant(name).is_some() => {
                Node::Int(errno_constant(name).expect("checked errno constant"))
            }
            name if host_constant(name).is_some() => {
                Node::Int(host_constant(name).expect("checked host constant"))
            }
            "GETRAW" => Node::Int(-1),
            "GETTIMEMICRO" => Node::Int(current_time_micro()),
            "islinux" => Node::Int(i64::from(cfg!(target_os = "linux"))),
            "ismacos" => Node::Int(i64::from(cfg!(target_os = "macos"))),
            "iswindows" => Node::Int(i64::from(cfg!(target_os = "windows"))),
            "sizeof_char" => Node::Int(size_of_i64::<std::os::raw::c_char>()),
            "sizeof_short" => Node::Int(size_of_i64::<std::os::raw::c_short>()),
            "sizeof_int" => Node::Int(size_of_i64::<std::os::raw::c_int>()),
            "sizeof_long" => Node::Int(size_of_i64::<std::os::raw::c_long>()),
            "sizeof_llong" => Node::Int(size_of_i64::<std::os::raw::c_longlong>()),
            "sizeof_size_t" => Node::Int(size_of_i64::<usize>()),
            "want_gmp" => Node::Int(0),
            "want_imath" => Node::Int(1),
            "js_debug" => {
                let ptr = self.eval_pointer_value(args[0])?;
                let bytes = self.read_c_string(ptr)?;
                host_js_debug(&bytes)?;
                Node::prim("I")
            }
            "js_eval_run" => {
                let ptr = self.eval_pointer_value(args[0])?;
                let bytes = self.read_c_string(ptr)?;
                host_js_eval_run(&bytes)?;
                Node::prim("I")
            }
            "js_eval_call" => {
                let ptr = self.eval_pointer_value(args[0])?;
                let bytes = self.read_c_string(ptr)?;
                let result = host_js_eval_call(&bytes)?;
                Node::Ptr(self.alloc_c_string_bytes(&result)?)
            }
            "js_set_haskellCallback" => {
                let callback = self.eval_int(args[0])?;
                host_js_set_haskell_callback(callback as i32)?;
                Node::prim("I")
            }
            "new_mpz" => self.new_mpz_node()?,
            "mpz_init_set_si" => {
                let ptr = self.eval_pointer_value(args[0])?;
                let value = self.eval_int(args[1])?;
                self.write_mpz_value(ptr, MpzValue::from_i64(value))?;
                Node::prim("I")
            }
            "mpz_init_set_ui" => {
                let ptr = self.eval_pointer_value(args[0])?;
                let value = self.eval_int(args[1])? as u64;
                self.write_mpz_value(ptr, MpzValue::from_u64(value))?;
                Node::prim("I")
            }
            "mpz_init_set_si64" => {
                let ptr = self.eval_pointer_value(args[0])?;
                let value = self.eval_int64(args[1])?;
                self.write_mpz_value(ptr, MpzValue::from_i64(value))?;
                Node::prim("I")
            }
            "mpz_init_set_ui64" => {
                let ptr = self.eval_pointer_value(args[0])?;
                let value = self.eval_int64(args[1])? as u64;
                self.write_mpz_value(ptr, MpzValue::from_u64(value))?;
                Node::prim("I")
            }
            "mpz_get_si" => {
                let ptr = self.eval_pointer_value(args[0])?;
                Node::Int(self.mpz_value(ptr)?.to_i64_wrapping())
            }
            "mpz_get_si64" => {
                let ptr = self.eval_pointer_value(args[0])?;
                Node::Int64(self.mpz_value(ptr)?.to_i64_wrapping())
            }
            "mpz_get_f" => {
                let ptr = self.eval_pointer_value(args[0])?;
                Node::Float32(self.mpz_value(ptr)?.to_f64() as f32)
            }
            "mpz_get_d" => {
                let ptr = self.eval_pointer_value(args[0])?;
                Node::Float64(self.mpz_value(ptr)?.to_f64())
            }
            "mpz_abs" => {
                let dst = self.eval_pointer_value(args[0])?;
                let src = self.eval_pointer_value(args[1])?;
                let mut value = self.mpz_value(src)?;
                value.negative = false;
                self.write_mpz_value(dst, value)?;
                Node::prim("I")
            }
            "mpz_neg" => {
                let dst = self.eval_pointer_value(args[0])?;
                let src = self.eval_pointer_value(args[1])?;
                let mut value = self.mpz_value(src)?;
                if !value.is_zero() {
                    value.negative = !value.negative;
                }
                self.write_mpz_value(dst, value)?;
                Node::prim("I")
            }
            "mpz_add" | "mpz_sub" | "mpz_mul" | "mpz_and" | "mpz_ior" | "mpz_xor" => {
                let dst = self.eval_pointer_value(args[0])?;
                let left_ptr = self.eval_pointer_value(args[1])?;
                let right_ptr = self.eval_pointer_value(args[2])?;
                let left = self.mpz_value(left_ptr)?;
                let right = self.mpz_value(right_ptr)?;
                let value = match name {
                    "mpz_add" => left.add(&right),
                    "mpz_sub" => left.sub(&right),
                    "mpz_mul" => left.mul(&right),
                    "mpz_and" => left.bitand(&right),
                    "mpz_ior" => left.bitor(&right),
                    "mpz_xor" => left.bitxor(&right),
                    _ => unreachable!("checked mpz binary op"),
                };
                self.write_mpz_value(dst, value)?;
                Node::prim("I")
            }
            "mpz_cmp" => {
                let left_ptr = self.eval_pointer_value(args[0])?;
                let right_ptr = self.eval_pointer_value(args[1])?;
                let left = self.mpz_value(left_ptr)?;
                let right = self.mpz_value(right_ptr)?;
                Node::Int(match left.cmp(&right) {
                    Ordering::Less => -1,
                    Ordering::Equal => 0,
                    Ordering::Greater => 1,
                })
            }
            "mpz_mul_2exp" => {
                let dst = self.eval_pointer_value(args[0])?;
                let src = self.eval_pointer_value(args[1])?;
                let shift = int_to_usize(self.eval_int(args[2])?)?;
                self.write_mpz_value(dst, self.mpz_value(src)?.shl_bits(shift))?;
                Node::prim("I")
            }
            "mpz_fdiv_q_2exp" => {
                let dst = self.eval_pointer_value(args[0])?;
                let src = self.eval_pointer_value(args[1])?;
                let shift = int_to_usize(self.eval_int(args[2])?)?;
                self.write_mpz_value(dst, self.mpz_value(src)?.fdiv_q_2exp(shift))?;
                Node::prim("I")
            }
            "mpz_tdiv_qr" => {
                let q_ptr = self.eval_pointer_value(args[0])?;
                let r_ptr = self.eval_pointer_value(args[1])?;
                let left_ptr = self.eval_pointer_value(args[2])?;
                let right_ptr = self.eval_pointer_value(args[3])?;
                let left = self.mpz_value(left_ptr)?;
                let right = self.mpz_value(right_ptr)?;
                let (quot, rem) = left.tdiv_qr(&right)?;
                self.write_mpz_value(q_ptr, quot)?;
                self.write_mpz_value(r_ptr, rem)?;
                Node::prim("I")
            }
            "mpz_popcount" => {
                let ptr = self.eval_pointer_value(args[0])?;
                Node::Int(self.mpz_value(ptr)?.signed_popcount()?)
            }
            "mpz_tstbit" => {
                let ptr = self.eval_pointer_value(args[0])?;
                let bit = int_to_usize(self.eval_int(args[1])?)?;
                Node::Int(i64::from(self.mpz_value(ptr)?.test_bit_signed(bit)))
            }
            "mpz_log2" => {
                let ptr = self.eval_pointer_value(args[0])?;
                Node::Int(self.mpz_value(ptr)?.log2()?)
            }
            "&closeb" => Node::fun_ptr("closeb"),
            "&free" => Node::fun_ptr("free"),
            "&errno" | "errno" => Node::Ptr(self.errno_ptr()?),
            "malloc" => {
                let size = int_to_usize(self.eval_int(args[0])?)?;
                Node::Ptr(self.alloc_memory(size)?)
            }
            "calloc" => {
                let count = int_to_usize(self.eval_int(args[0])?)?;
                let size = int_to_usize(self.eval_int(args[1])?)?;
                Node::Ptr(self.calloc_memory(count, size)?)
            }
            "realloc" => {
                let ptr = self.eval_pointer_value(args[0])?;
                let size = int_to_usize(self.eval_int(args[1])?)?;
                Node::Ptr(self.realloc_memory(ptr, size)?)
            }
            "free" => {
                let ptr = self.eval_pointer_value(args[0])?;
                self.free_memory(ptr)?;
                Node::prim("I")
            }
            "memcpy" | "memmove" => {
                let dst = self.eval_pointer_value(args[0])?;
                let src = self.eval_pointer_value(args[1])?;
                let len = int_to_usize(self.eval_int(args[2])?)?;
                let bytes = self.read_pointer_bytes(src, len)?;
                self.write_pointer_bytes(dst, &bytes)?;
                Node::prim("I")
            }
            "strcpy" => {
                let dst = self.eval_pointer_value(args[0])?;
                let src = self.eval_pointer_value(args[1])?;
                let mut bytes = self.read_c_string(src)?;
                bytes.push(0);
                self.write_pointer_bytes(dst, &bytes)?;
                Node::prim("I")
            }
            "strlen" => {
                let ptr = self.eval_pointer_value(args[0])?;
                let len = self.c_string_len(ptr)?;
                Node::Int(i64::try_from(len).map_err(|_| EvalError::Overflow)?)
            }
            "putchar" => {
                let byte = self.eval_int(args[0])?;
                self.write_io_handle_bytes(StdHandle::Stdout, &[byte as u8])?;
                Node::prim("I")
            }
            "md5String" => {
                let input = self.eval_pointer_value(args[0])?;
                let result = self.eval_pointer_value(args[1])?;
                let bytes = self.read_c_string(input)?;
                self.write_pointer_bytes(result, &md5_bytes(&bytes))?;
                Node::prim("I")
            }
            "md5Array" => {
                let input = self.eval_pointer_value(args[0])?;
                let result = self.eval_pointer_value(args[1])?;
                let len = int_to_usize(self.eval_int(args[2])?)?;
                let bytes = self.read_pointer_bytes(input, len)?;
                self.write_pointer_bytes(result, &md5_bytes(&bytes))?;
                Node::prim("I")
            }
            "md5BFILE" => {
                let ptr = self.eval_pointer_value(args[0])?;
                let result = self.eval_pointer_value(args[1])?;
                let mut ctx = Md5Context::new();
                loop {
                    let bytes = self.read_bfile_bytes(ptr, 1024)?;
                    if bytes.is_empty() {
                        break;
                    }
                    ctx.update(&bytes);
                }
                self.write_pointer_bytes(result, &ctx.finalize())?;
                Node::prim("I")
            }
            "getenv" => {
                let ptr = self.eval_pointer_value(args[0])?;
                let name = self.read_c_string(ptr)?;
                let ptr = if let Some(mut bytes) = getenv_bytes(&name) {
                    bytes.push(0);
                    let ptr = self.alloc_memory(bytes.len())?;
                    self.write_pointer_bytes(ptr, &bytes)?;
                    ptr
                } else {
                    0
                };
                Node::Ptr(ptr)
            }
            "setenv" => {
                let name_ptr = self.eval_pointer_value(args[0])?;
                let value_ptr = self.eval_pointer_value(args[1])?;
                let overwrite = self.eval_int(args[2])?;
                let name = self.read_c_string(name_ptr)?;
                let value = self.read_c_string(value_ptr)?;
                self.host_int_node(setenv_bytes(&name, &value, overwrite))?
            }
            "unsetenv" => {
                let name_ptr = self.eval_pointer_value(args[0])?;
                let name = self.read_c_string(name_ptr)?;
                self.host_int_node(unsetenv_bytes(&name))?
            }
            "environ" => Node::Ptr(self.alloc_environ()?),
            "strerror_r" => {
                let errno = int_to_i32(self.eval_int(args[0])?)?;
                let ptr = self.eval_pointer_value(args[1])?;
                let size = int_to_usize(self.eval_int(args[2])?)?;
                Node::Int(self.write_strerror(errno, ptr, size)?)
            }
            "remove" => {
                let ptr = self.eval_pointer_value(args[0])?;
                let path = self.read_c_string(ptr)?;
                self.host_int_node(remove_path_bytes(&path))?
            }
            "system" => {
                let ptr = self.eval_pointer_value(args[0])?;
                let command = if ptr == 0 {
                    None
                } else {
                    Some(self.read_c_string(ptr)?)
                };
                Node::Int(system_command_bytes(command.as_deref()))
            }
            "chdir" => {
                let ptr = self.eval_pointer_value(args[0])?;
                let path = self.read_c_string(ptr)?;
                self.host_int_node(chdir_path_bytes(&path))?
            }
            "mkdir" => {
                let ptr = self.eval_pointer_value(args[0])?;
                let path = self.read_c_string(ptr)?;
                let mode = self.eval_int(args[1])?;
                self.host_int_node(mkdir_path_bytes(&path, mode))?
            }
            "getcwd" => {
                let ptr = self.eval_pointer_value(args[0])?;
                let size = int_to_usize(self.eval_int(args[1])?)?;
                match current_dir_bytes() {
                    Ok(mut bytes) => {
                        bytes.push(0);
                        if bytes.len() <= size {
                            self.write_pointer_bytes(ptr, &bytes)?;
                            Node::Ptr(ptr)
                        } else {
                            self.set_errno_value(errno_i32("ERANGE"))?;
                            Node::Ptr(0)
                        }
                    }
                    Err(errno) => {
                        self.set_errno_value(errno)?;
                        Node::Ptr(0)
                    }
                }
            }
            "get_permissions" => {
                let ptr = self.eval_pointer_value(args[0])?;
                let path = self.read_c_string(ptr)?;
                self.host_int_node(get_permissions_path_bytes(&path))?
            }
            "set_permissions" => {
                let ptr = self.eval_pointer_value(args[0])?;
                let path = self.read_c_string(ptr)?;
                let permissions = self.eval_int(args[1])?;
                self.host_int_node(set_permissions_path_bytes(&path, permissions))?
            }
            "get_executable_path" => {
                let path = self
                    .executable_path
                    .clone()
                    .map(Ok)
                    .unwrap_or_else(executable_path_bytes);
                let ptr = match path {
                    Ok(mut bytes) => {
                        bytes.push(0);
                        let ptr = self.alloc_memory(bytes.len())?;
                        self.write_pointer_bytes(ptr, &bytes)?;
                        ptr
                    }
                    Err(errno) => {
                        self.set_errno_value(errno)?;
                        0
                    }
                };
                Node::Ptr(ptr)
            }
            "tmpname" => {
                let pre_ptr = self.eval_pointer_value(args[0])?;
                let suf_ptr = self.eval_pointer_value(args[1])?;
                let pre = self.read_c_string(pre_ptr)?;
                let suf = self.read_c_string(suf_ptr)?;
                match tmpname_bytes(&pre, &suf) {
                    Ok(mut bytes) => {
                        bytes.push(0);
                        let ptr = self.alloc_memory(bytes.len())?;
                        self.write_pointer_bytes(ptr, &bytes)?;
                        Node::Ptr(ptr)
                    }
                    Err(errno) => {
                        self.set_errno_value(errno)?;
                        Node::Ptr(0)
                    }
                }
            }
            "opendir" => {
                let ptr = self.eval_pointer_value(args[0])?;
                let path = self.read_c_string(ptr)?;
                match dir_entries_path_bytes(&path) {
                    Ok(entries) => Node::Ptr(self.alloc_dir(entries)?),
                    Err(errno) => {
                        self.set_errno_value(errno)?;
                        Node::Ptr(0)
                    }
                }
            }
            "readdir" => {
                let ptr = self.eval_pointer_value(args[0])?;
                Node::Ptr(self.read_dir_entry(ptr)?)
            }
            "closedir" => {
                let ptr = self.eval_pointer_value(args[0])?;
                if self.close_dir(ptr).is_ok() {
                    Node::Int(0)
                } else {
                    self.set_errno_value(errno_i32("EBADF"))?;
                    Node::Int(-1)
                }
            }
            "c_d_name" => {
                let ptr = self.eval_pointer_value(args[0])?;
                Node::Ptr(ptr)
            }
            "fopen" => {
                let path_ptr = self.eval_pointer_value(args[0])?;
                let mode_ptr = self.eval_pointer_value(args[1])?;
                let path = self.read_c_string(path_ptr)?;
                let mode = self.read_c_string(mode_ptr)?;
                match native_fopen_bfile(&path, &mode) {
                    Ok(bfile) => Node::Ptr(self.alloc_bfile(bfile)?),
                    Err(errno) => {
                        self.set_errno_value(errno)?;
                        Node::Ptr(0)
                    }
                }
            }
            "open" => {
                let path_ptr = self.eval_pointer_value(args[0])?;
                let flags = int_to_i32(self.eval_int(args[1])?)?;
                let mode = self.eval_int(args[2])?;
                let path = self.read_c_string(path_ptr)?;
                self.host_int_node(open_fd_path_bytes(&path, flags, mode))?
            }
            "add_FILE" => {
                let ptr = self.eval_pointer_value(args[0])?;
                if ptr != 0 && handle_from_ptr(ptr).is_none() {
                    self.bfile(ptr)?;
                }
                Node::Ptr(ptr)
            }
            "add_fd" => {
                let fd = int_to_i32(self.eval_int(args[0])?)?;
                match native_fd_bfile(fd) {
                    Ok(bfile) => Node::Ptr(self.alloc_bfile(bfile)?),
                    Err(errno) => {
                        self.set_errno_value(errno)?;
                        Node::Ptr(0)
                    }
                }
            }
            "add_utf8" => {
                let ptr = self.eval_pointer_value(args[0])?;
                Node::Ptr(self.add_utf8_bfile(ptr)?)
            }
            "add_crlf" => {
                let ptr = self.eval_pointer_value(args[0])?;
                Node::Ptr(self.add_crlf_bfile(ptr)?)
            }
            "add_rle_decompressor" => {
                let ptr = self.eval_pointer_value(args[0])?;
                Node::Ptr(self.add_rle_bfile(ptr, true)?)
            }
            "add_rle_compressor" => {
                let ptr = self.eval_pointer_value(args[0])?;
                Node::Ptr(self.add_rle_bfile(ptr, false)?)
            }
            "add_base64_decoder" => {
                let ptr = self.eval_pointer_value(args[0])?;
                Node::Ptr(self.add_base64_bfile(ptr, true)?)
            }
            "add_base64_encoder" => {
                let ptr = self.eval_pointer_value(args[0])?;
                Node::Ptr(self.add_base64_bfile(ptr, false)?)
            }
            "add_lz77_decompressor" => {
                let ptr = self.eval_pointer_value(args[0])?;
                Node::Ptr(self.add_lz77_bfile(ptr, true)?)
            }
            "add_lz77_compressor" => {
                let ptr = self.eval_pointer_value(args[0])?;
                Node::Ptr(self.add_lz77_bfile(ptr, false)?)
            }
            "add_bwt_decompressor" => {
                let ptr = self.eval_pointer_value(args[0])?;
                Node::Ptr(self.add_bwt_bfile(ptr, true)?)
            }
            "add_bwt_compressor" => {
                let ptr = self.eval_pointer_value(args[0])?;
                Node::Ptr(self.add_bwt_bfile(ptr, false)?)
            }
            "add_lzma_decompressor" => {
                let ptr = self.eval_pointer_value(args[0])?;
                Node::Ptr(self.add_lzma_bfile(ptr, true)?)
            }
            "add_lzma_compressor" => {
                let ptr = self.eval_pointer_value(args[0])?;
                Node::Ptr(self.add_lzma_bfile(ptr, false)?)
            }
            "add_buf" => {
                let ptr = self.eval_pointer_value(args[0])?;
                let size = self.eval_int(args[1])?;
                Node::Ptr(self.add_buf_bfile(ptr, size)?)
            }
            "openb_wr_mem" => Node::Ptr(self.alloc_bfile(BFile {
                kind: BFileKind::Memory {
                    bytes: Vec::new(),
                    pos: 0,
                },
                readable: false,
                writable: true,
            })?),
            "openb_rd_mem" => {
                let ptr = self.eval_pointer_value(args[0])?;
                let len = int_to_usize(self.eval_int(args[1])?)?;
                let bytes = self.read_pointer_bytes(ptr, len)?;
                Node::Ptr(self.alloc_bfile(BFile {
                    kind: BFileKind::Memory { bytes, pos: 0 },
                    readable: true,
                    writable: false,
                })?)
            }
            "get_mem" => {
                let bfile_ptr = self.eval_pointer_value(args[0])?;
                let bufp = self.eval_pointer_value(args[1])?;
                let lenp = self.eval_pointer_value(args[2])?;
                let buffer = self.bfile_output_bytes(bfile_ptr)?;
                let len = i64::try_from(buffer.len()).map_err(|_| EvalError::Overflow)?;
                let ptr = self.alloc_memory(buffer.len())?;
                self.write_pointer_bytes(ptr, &buffer)?;
                self.poke_signed(bufp, 8, ptr)?;
                self.poke_signed(lenp, 8, len)?;
                Node::prim("I")
            }
            "getcpu" => {
                let sec_ptr = self.eval_pointer_value(args[0])?;
                let nsec_ptr = self.eval_pointer_value(args[1])?;
                let (sec, nsec) = cpu_time();
                self.poke_unsigned(sec_ptr, size_of::<std::os::raw::c_ulong>(), sec)?;
                self.poke_unsigned(nsec_ptr, size_of::<std::os::raw::c_ulong>(), nsec)?;
                Node::prim("I")
            }
            "gettimeofday" => {
                let timeval_ptr = self.eval_pointer_value(args[0])?;
                let timezone_ptr = self.eval_pointer_value(args[1])?;
                self.gettimeofday_node(timeval_ptr, timezone_ptr)?
            }
            "accept" => {
                let fd = int_to_i32(self.eval_int(args[0])?)?;
                let addr_ptr = self.eval_pointer_value(args[1])?;
                let len_ptr = self.eval_pointer_value(args[2])?;
                self.accept_socket_node(fd, addr_ptr, len_ptr)?
            }
            "bind" => {
                let fd = int_to_i32(self.eval_int(args[0])?)?;
                let addr_ptr = self.eval_pointer_value(args[1])?;
                let len = int_to_usize(self.eval_int(args[2])?)?;
                let addr = self.read_pointer_bytes(addr_ptr, len)?;
                self.host_int_node(bind_socket(fd, &addr))?
            }
            "close" => {
                let fd = int_to_i32(self.eval_int(args[0])?)?;
                self.host_int_node(close_fd(fd))?
            }
            "connect" => {
                let fd = int_to_i32(self.eval_int(args[0])?)?;
                let addr_ptr = self.eval_pointer_value(args[1])?;
                let len = int_to_usize(self.eval_int(args[2])?)?;
                let addr = self.read_pointer_bytes(addr_ptr, len)?;
                self.host_int_node(connect_socket(fd, &addr))?
            }
            "fcntl" => {
                let fd = int_to_i32(self.eval_int(args[0])?)?;
                let cmd = int_to_i32(self.eval_int(args[1])?)?;
                let arg = int_to_i32(self.eval_int(args[2])?)?;
                self.host_int_node(fcntl_fd(fd, cmd, arg))?
            }
            "getsockopt" => {
                let fd = int_to_i32(self.eval_int(args[0])?)?;
                let level = int_to_i32(self.eval_int(args[1])?)?;
                let optname = int_to_i32(self.eval_int(args[2])?)?;
                let optval_ptr = self.eval_pointer_value(args[3])?;
                let optlen_ptr = self.eval_pointer_value(args[4])?;
                self.getsockopt_node(fd, level, optname, optval_ptr, optlen_ptr)?
            }
            "listen" => {
                let fd = int_to_i32(self.eval_int(args[0])?)?;
                let backlog = int_to_i32(self.eval_int(args[1])?)?;
                self.host_int_node(listen_socket(fd, backlog))?
            }
            "recv" => {
                let fd = int_to_i32(self.eval_int(args[0])?)?;
                let buf_ptr = self.eval_pointer_value(args[1])?;
                let len = int_to_usize(self.eval_int(args[2])?)?;
                let flags = int_to_i32(self.eval_int(args[3])?)?;
                self.recv_socket_node(fd, buf_ptr, len, flags)?
            }
            "send" => {
                let fd = int_to_i32(self.eval_int(args[0])?)?;
                let buf_ptr = self.eval_pointer_value(args[1])?;
                let len = int_to_usize(self.eval_int(args[2])?)?;
                let flags = int_to_i32(self.eval_int(args[3])?)?;
                self.send_socket_node(fd, buf_ptr, len, flags)?
            }
            "setsockopt" => {
                let fd = int_to_i32(self.eval_int(args[0])?)?;
                let level = int_to_i32(self.eval_int(args[1])?)?;
                let optname = int_to_i32(self.eval_int(args[2])?)?;
                let optval_ptr = self.eval_pointer_value(args[3])?;
                let optlen = int_to_usize(self.eval_int(args[4])?)?;
                let optval = self.read_pointer_bytes(optval_ptr, optlen)?;
                self.host_int_node(setsockopt_socket(fd, level, optname, &optval))?
            }
            "socket" => {
                let domain = int_to_i32(self.eval_int(args[0])?)?;
                let typ = int_to_i32(self.eval_int(args[1])?)?;
                let protocol = int_to_i32(self.eval_int(args[2])?)?;
                self.host_int_node(socket_fd(domain, typ, protocol))?
            }
            "closeb" => {
                let ptr = self.eval_pointer_value(args[0])?;
                self.close_bfile(ptr)?;
                Node::prim("I")
            }
            "flushb" => {
                let ptr = self.eval_pointer_value(args[0])?;
                self.flush_bfile(ptr)?;
                Node::prim("I")
            }
            "getb" => {
                let ptr = self.eval_pointer_value(args[0])?;
                Node::Int(self.get_bfile_byte(ptr)?)
            }
            "putb" => {
                let byte = self.eval_int(args[0])?;
                let ptr = self.eval_pointer_value(args[1])?;
                self.put_bfile_byte(ptr, byte)?;
                Node::prim("I")
            }
            "ungetb" => {
                let byte = self.eval_int(args[0])?;
                let ptr = self.eval_pointer_value(args[1])?;
                self.unget_bfile_byte(ptr, byte)?;
                Node::prim("I")
            }
            "readb" => {
                let dst = self.eval_pointer_value(args[0])?;
                let len = int_to_usize(self.eval_int(args[1])?)?;
                let ptr = self.eval_pointer_value(args[2])?;
                Node::Int(
                    i64::try_from(self.read_bfile(ptr, dst, len)?)
                        .map_err(|_| EvalError::Overflow)?,
                )
            }
            "writeb" => {
                let src = self.eval_pointer_value(args[0])?;
                let len = int_to_usize(self.eval_int(args[1])?)?;
                let ptr = self.eval_pointer_value(args[2])?;
                Node::Int(
                    i64::try_from(self.write_bfile(ptr, src, len)?)
                        .map_err(|_| EvalError::Overflow)?,
                )
            }
            "lz77c" => {
                let src = self.eval_pointer_value(args[0])?;
                let len = int_to_usize(self.eval_int(args[1])?)?;
                let out_ptr = self.eval_pointer_value(args[2])?;
                let bytes = self.read_pointer_bytes(src, len)?;
                let compressed = lz77_compress(&bytes)?;
                let compressed_ptr = self.alloc_memory(compressed.len())?;
                self.write_pointer_bytes(compressed_ptr, &compressed)?;
                self.poke_signed(out_ptr, 8, compressed_ptr)?;
                Node::Int(i64::try_from(compressed.len()).map_err(|_| EvalError::Overflow)?)
            }
            "peekPtr" => {
                let ptr = self.eval_pointer_value(args[0])?;
                Node::Ptr(self.peek_signed(ptr, 8)?)
            }
            "pokePtr" => {
                let ptr = self.eval_pointer_value(args[0])?;
                let value = self.eval_pointer_value(args[1])?;
                self.poke_signed(ptr, 8, value)?;
                Node::prim("I")
            }
            "peekWord" => {
                let ptr = self.eval_pointer_value(args[0])?;
                let result = self.int(self.peek_unsigned(ptr, 8)? as i64);
                return Ok(Some((2, self.pair(result, args[1]))));
            }
            "pokeWord" => {
                let ptr = self.eval_pointer_value(args[0])?;
                let value = self.eval_int(args[1])?;
                self.poke_unsigned(ptr, 8, value as u64)?;
                return Ok(Some((3, self.unit_pair(args[2]))));
            }
            "peek_uint8" => {
                let ptr = self.eval_pointer_value(args[0])?;
                Node::Int(self.peek_unsigned(ptr, 1)? as i64)
            }
            "poke_uint8" => {
                let ptr = self.eval_pointer_value(args[0])?;
                let value = self.eval_int(args[1])?;
                self.poke_unsigned(ptr, 1, value as u64)?;
                Node::prim("I")
            }
            "peek_uint16" => {
                let ptr = self.eval_pointer_value(args[0])?;
                Node::Int(self.peek_unsigned(ptr, 2)? as i64)
            }
            "poke_uint16" => {
                let ptr = self.eval_pointer_value(args[0])?;
                let value = self.eval_int(args[1])?;
                self.poke_unsigned(ptr, 2, value as u64)?;
                Node::prim("I")
            }
            "peek_uint32" => {
                let ptr = self.eval_pointer_value(args[0])?;
                Node::Int(self.peek_unsigned(ptr, 4)? as i64)
            }
            "poke_uint32" => {
                let ptr = self.eval_pointer_value(args[0])?;
                let value = self.eval_int(args[1])?;
                self.poke_unsigned(ptr, 4, value as u64)?;
                Node::prim("I")
            }
            "peek_uint64" => {
                let ptr = self.eval_pointer_value(args[0])?;
                let result = self.push_node(Node::Int64(self.peek_unsigned(ptr, 8)? as i64));
                return Ok(Some((2, self.pair(result, args[1]))));
            }
            "poke_uint64" => {
                let ptr = self.eval_pointer_value(args[0])?;
                let value = self.eval_int64(args[1])?;
                self.poke_unsigned(ptr, 8, value as u64)?;
                return Ok(Some((3, self.unit_pair(args[2]))));
            }
            "peek_int8" => {
                let ptr = self.eval_pointer_value(args[0])?;
                Node::Int(self.peek_signed(ptr, 1)?)
            }
            "poke_int8" => {
                let ptr = self.eval_pointer_value(args[0])?;
                let value = self.eval_int(args[1])?;
                self.poke_signed(ptr, 1, value)?;
                Node::prim("I")
            }
            "peek_int16" => {
                let ptr = self.eval_pointer_value(args[0])?;
                Node::Int(self.peek_signed(ptr, 2)?)
            }
            "poke_int16" => {
                let ptr = self.eval_pointer_value(args[0])?;
                let value = self.eval_int(args[1])?;
                self.poke_signed(ptr, 2, value)?;
                Node::prim("I")
            }
            "peek_int32" => {
                let ptr = self.eval_pointer_value(args[0])?;
                Node::Int(self.peek_signed(ptr, 4)?)
            }
            "poke_int32" => {
                let ptr = self.eval_pointer_value(args[0])?;
                let value = self.eval_int(args[1])?;
                self.poke_signed(ptr, 4, value)?;
                Node::prim("I")
            }
            "peek_int64" => {
                let ptr = self.eval_pointer_value(args[0])?;
                Node::Int64(self.peek_signed(ptr, 8)?)
            }
            "poke_int64" => {
                let ptr = self.eval_pointer_value(args[0])?;
                let value = self.eval_int64(args[1])?;
                self.poke_signed(ptr, 8, value)?;
                Node::prim("I")
            }
            "peek_char" => {
                let ptr = self.eval_pointer_value(args[0])?;
                Node::Int(self.peek_c_char(ptr)?)
            }
            "poke_char" => {
                let ptr = self.eval_pointer_value(args[0])?;
                let value = self.eval_int(args[1])?;
                self.poke_c_char(ptr, value)?;
                Node::prim("I")
            }
            "peek_schar" => {
                let ptr = self.eval_pointer_value(args[0])?;
                Node::Int(self.peek_signed(ptr, size_of::<std::os::raw::c_schar>())?)
            }
            "poke_schar" => {
                let ptr = self.eval_pointer_value(args[0])?;
                let value = self.eval_int(args[1])?;
                self.poke_signed(ptr, size_of::<std::os::raw::c_schar>(), value)?;
                Node::prim("I")
            }
            "peek_uchar" => {
                let ptr = self.eval_pointer_value(args[0])?;
                Node::Int(self.peek_unsigned(ptr, size_of::<std::os::raw::c_uchar>())? as i64)
            }
            "poke_uchar" => {
                let ptr = self.eval_pointer_value(args[0])?;
                let value = self.eval_int(args[1])?;
                self.poke_unsigned(ptr, size_of::<std::os::raw::c_uchar>(), value as u64)?;
                Node::prim("I")
            }
            "peek_short" => {
                let ptr = self.eval_pointer_value(args[0])?;
                Node::Int(self.peek_signed(ptr, size_of::<std::os::raw::c_short>())?)
            }
            "poke_short" => {
                let ptr = self.eval_pointer_value(args[0])?;
                let value = self.eval_int(args[1])?;
                self.poke_signed(ptr, size_of::<std::os::raw::c_short>(), value)?;
                Node::prim("I")
            }
            "peek_ushort" => {
                let ptr = self.eval_pointer_value(args[0])?;
                Node::Int(self.peek_unsigned(ptr, size_of::<std::os::raw::c_ushort>())? as i64)
            }
            "poke_ushort" => {
                let ptr = self.eval_pointer_value(args[0])?;
                let value = self.eval_int(args[1])?;
                self.poke_unsigned(ptr, size_of::<std::os::raw::c_ushort>(), value as u64)?;
                Node::prim("I")
            }
            "peek_int" => {
                let ptr = self.eval_pointer_value(args[0])?;
                Node::Int(self.peek_signed(ptr, size_of::<std::os::raw::c_int>())?)
            }
            "poke_int" => {
                let ptr = self.eval_pointer_value(args[0])?;
                let value = self.eval_int(args[1])?;
                self.poke_signed(ptr, size_of::<std::os::raw::c_int>(), value)?;
                Node::prim("I")
            }
            "peek_uint" => {
                let ptr = self.eval_pointer_value(args[0])?;
                Node::Int(self.peek_unsigned(ptr, size_of::<std::os::raw::c_uint>())? as i64)
            }
            "poke_uint" => {
                let ptr = self.eval_pointer_value(args[0])?;
                let value = self.eval_int(args[1])?;
                self.poke_unsigned(ptr, size_of::<std::os::raw::c_uint>(), value as u64)?;
                Node::prim("I")
            }
            "peek_long" => {
                let ptr = self.eval_pointer_value(args[0])?;
                Node::Int(self.peek_signed(ptr, size_of::<std::os::raw::c_long>())?)
            }
            "poke_long" => {
                let ptr = self.eval_pointer_value(args[0])?;
                let value = self.eval_int(args[1])?;
                self.poke_signed(ptr, size_of::<std::os::raw::c_long>(), value)?;
                Node::prim("I")
            }
            "peek_ulong" => {
                let ptr = self.eval_pointer_value(args[0])?;
                Node::Int(self.peek_unsigned(ptr, size_of::<std::os::raw::c_ulong>())? as i64)
            }
            "poke_ulong" => {
                let ptr = self.eval_pointer_value(args[0])?;
                let value = self.eval_int(args[1])?;
                self.poke_unsigned(ptr, size_of::<std::os::raw::c_ulong>(), value as u64)?;
                Node::prim("I")
            }
            "peek_llong" => {
                let ptr = self.eval_pointer_value(args[0])?;
                Node::Int(self.peek_signed(ptr, size_of::<std::os::raw::c_longlong>())?)
            }
            "poke_llong" => {
                let ptr = self.eval_pointer_value(args[0])?;
                let value = self.eval_int(args[1])?;
                self.poke_signed(ptr, size_of::<std::os::raw::c_longlong>(), value)?;
                Node::prim("I")
            }
            "peek_ullong" => {
                let ptr = self.eval_pointer_value(args[0])?;
                Node::Int(self.peek_unsigned(ptr, size_of::<std::os::raw::c_ulonglong>())? as i64)
            }
            "poke_ullong" => {
                let ptr = self.eval_pointer_value(args[0])?;
                let value = self.eval_int(args[1])?;
                self.poke_unsigned(ptr, size_of::<std::os::raw::c_ulonglong>(), value as u64)?;
                Node::prim("I")
            }
            "peek_size_t" => {
                let ptr = self.eval_pointer_value(args[0])?;
                Node::Int(self.peek_unsigned(ptr, size_of::<usize>())? as i64)
            }
            "poke_size_t" => {
                let ptr = self.eval_pointer_value(args[0])?;
                let value = self.eval_int(args[1])?;
                self.poke_unsigned(ptr, size_of::<usize>(), value as u64)?;
                Node::prim("I")
            }
            "peek_flt32" => {
                let ptr = self.eval_pointer_value(args[0])?;
                Node::Float32(f32::from_ne_bytes(self.peek_array(ptr)?))
            }
            "poke_flt32" => {
                let ptr = self.eval_pointer_value(args[0])?;
                let value = self.eval_float32(args[1])?;
                self.write_pointer_bytes(ptr, &value.to_ne_bytes())?;
                Node::prim("I")
            }
            "peek_flt64" => {
                let ptr = self.eval_pointer_value(args[0])?;
                Node::Float64(f64::from_ne_bytes(self.peek_array(ptr)?))
            }
            "poke_flt64" => {
                let ptr = self.eval_pointer_value(args[0])?;
                let value = self.eval_float64(args[1])?;
                self.write_pointer_bytes(ptr, &value.to_ne_bytes())?;
                Node::prim("I")
            }
            "acos" => Node::Float64(self.eval_float64(args[0])?.acos()),
            "asin" => Node::Float64(self.eval_float64(args[0])?.asin()),
            "atan" => Node::Float64(self.eval_float64(args[0])?.atan()),
            "cos" => Node::Float64(self.eval_float64(args[0])?.cos()),
            "exp" => Node::Float64(self.eval_float64(args[0])?.exp()),
            "log" => Node::Float64(self.eval_float64(args[0])?.ln()),
            "sin" => Node::Float64(self.eval_float64(args[0])?.sin()),
            "sqrt" => Node::Float64(self.eval_float64(args[0])?.sqrt()),
            "tan" => Node::Float64(self.eval_float64(args[0])?.tan()),
            "atan2" => {
                let x = self.eval_float64(args[0])?;
                let y = self.eval_float64(args[1])?;
                Node::Float64(x.atan2(y))
            }
            "pow" => {
                let x = self.eval_float64(args[0])?;
                let y = self.eval_float64(args[1])?;
                Node::Float64(x.powf(y))
            }
            "scalbn" => {
                let x = self.eval_float64(args[0])?;
                let n = int_to_i32(self.eval_int(args[1])?)?;
                Node::Float64(x * 2.0f64.powi(n))
            }
            "acosf" => Node::Float32(self.eval_float32(args[0])?.acos()),
            "asinf" => Node::Float32(self.eval_float32(args[0])?.asin()),
            "atanf" => Node::Float32(self.eval_float32(args[0])?.atan()),
            "cosf" => Node::Float32(self.eval_float32(args[0])?.cos()),
            "expf" => Node::Float32(self.eval_float32(args[0])?.exp()),
            "logf" => Node::Float32(self.eval_float32(args[0])?.ln()),
            "sinf" => Node::Float32(self.eval_float32(args[0])?.sin()),
            "sqrtf" => Node::Float32(self.eval_float32(args[0])?.sqrt()),
            "tanf" => Node::Float32(self.eval_float32(args[0])?.tan()),
            "atan2f" => {
                let x = self.eval_float32(args[0])?;
                let y = self.eval_float32(args[1])?;
                Node::Float32(x.atan2(y))
            }
            "powf" => {
                let x = self.eval_float32(args[0])?;
                let y = self.eval_float32(args[1])?;
                Node::Float32(x.powf(y))
            }
            "scalbnf" => {
                let x = self.eval_float32(args[0])?;
                let n = int_to_i32(self.eval_int(args[1])?)?;
                Node::Float32(x * 2.0f32.powi(n))
            }
            _ => unreachable!("checked FFI symbol"),
        };
        let result = self.push_value_node(result);
        Ok(Some((arity + 1, self.pair(result, args[arity]))))
    }

    fn zero_arity_ffi_result(&mut self, name: &str) -> Result<Option<Node>, EvalError> {
        if let Some(value) = errno_constant(name) {
            return Ok(Some(Node::Int(value)));
        }
        if let Some(value) = host_constant(name) {
            return Ok(Some(Node::Int(value)));
        }
        let result = match name {
            "GETRAW" => Node::Int(-1),
            "GETTIMEMICRO" => Node::Int(current_time_micro()),
            "islinux" => Node::Int(i64::from(cfg!(target_os = "linux"))),
            "ismacos" => Node::Int(i64::from(cfg!(target_os = "macos"))),
            "iswindows" => Node::Int(i64::from(cfg!(target_os = "windows"))),
            "sizeof_char" => Node::Int(size_of_i64::<std::os::raw::c_char>()),
            "sizeof_short" => Node::Int(size_of_i64::<std::os::raw::c_short>()),
            "sizeof_int" => Node::Int(size_of_i64::<std::os::raw::c_int>()),
            "sizeof_long" => Node::Int(size_of_i64::<std::os::raw::c_long>()),
            "sizeof_llong" => Node::Int(size_of_i64::<std::os::raw::c_longlong>()),
            "sizeof_size_t" => Node::Int(size_of_i64::<usize>()),
            "want_gmp" => Node::Int(0),
            "want_imath" => Node::Int(1),
            "&closeb" => Node::fun_ptr("closeb"),
            "&free" => Node::fun_ptr("free"),
            "&errno" | "errno" => Node::Ptr(self.errno_ptr()?),
            _ => return Ok(None),
        };
        Ok(Some(result))
    }

    fn unary_math_ffi_result(
        &mut self,
        name: &str,
        arg: NodeId,
    ) -> Result<Option<Node>, EvalError> {
        let result = match name {
            "acos" => Node::Float64(self.eval_float64(arg)?.acos()),
            "asin" => Node::Float64(self.eval_float64(arg)?.asin()),
            "atan" => Node::Float64(self.eval_float64(arg)?.atan()),
            "cos" => Node::Float64(self.eval_float64(arg)?.cos()),
            "exp" => Node::Float64(self.eval_float64(arg)?.exp()),
            "log" => Node::Float64(self.eval_float64(arg)?.ln()),
            "sin" => Node::Float64(self.eval_float64(arg)?.sin()),
            "sqrt" => Node::Float64(self.eval_float64(arg)?.sqrt()),
            "tan" => Node::Float64(self.eval_float64(arg)?.tan()),
            "acosf" => Node::Float32(self.eval_float32(arg)?.acos()),
            "asinf" => Node::Float32(self.eval_float32(arg)?.asin()),
            "atanf" => Node::Float32(self.eval_float32(arg)?.atan()),
            "cosf" => Node::Float32(self.eval_float32(arg)?.cos()),
            "expf" => Node::Float32(self.eval_float32(arg)?.exp()),
            "logf" => Node::Float32(self.eval_float32(arg)?.ln()),
            "sinf" => Node::Float32(self.eval_float32(arg)?.sin()),
            "sqrtf" => Node::Float32(self.eval_float32(arg)?.sqrt()),
            "tanf" => Node::Float32(self.eval_float32(arg)?.tan()),
            _ => return Ok(None),
        };
        Ok(Some(result))
    }

    fn js_call(
        &mut self,
        tags: &str,
        body: &[u8],
        args: &[NodeId],
    ) -> Result<Option<(usize, NodeId)>, EvalError> {
        let tags = tags.as_bytes();
        validate_js_tags(tags)?;
        let arity = tags.len() - 1;
        if args.len() < arity + 1 {
            return Ok(None);
        }
        let mut js_args = Vec::with_capacity(arity);
        for (idx, tag) in tags[1..].iter().copied().enumerate() {
            let arg = match tag {
                b'D' => JsArg::Double(self.eval_float64(args[idx])?),
                b'F' => JsArg::Double(f64::from(self.eval_float32(args[idx])?)),
                b'B' => JsArg::Int(i32::from(self.eval_bool(args[idx])?)),
                b'P' => JsArg::UInt(self.eval_pointer_value(args[idx])? as u32),
                b'J' => JsArg::Object(self.eval_js_object_handle(args[idx])?),
                b'S' => JsArg::String(self.eval_bytes(args[idx])?),
                b'U' => JsArg::UInt(self.eval_int(args[idx])? as u32),
                b'I' => JsArg::Int(self.eval_int(args[idx])? as i32),
                _ => return Err(EvalError::InvalidByteString),
            };
            js_args.push(arg);
        }
        let result = match tags[0] {
            b'V' => {
                host_js_call_void(body, arity, &js_args)?;
                Node::prim("I")
            }
            b'D' => Node::Float64(host_js_call_double(body, arity, &js_args)?),
            b'F' => Node::Float32(host_js_call_double(body, arity, &js_args)? as f32),
            b'P' => Node::Ptr(i64::from(host_js_call_ptr(body, arity, &js_args)?)),
            b'B' => Node::prim(if host_js_call_bool(body, arity, &js_args)? {
                "A"
            } else {
                "K"
            }),
            b'S' => Node::bytes(host_js_call_string(body, arity, &js_args)?),
            b'I' => Node::Int(i64::from(host_js_call_int(body, arity, &js_args)?)),
            b'U' => Node::Int(i64::from(host_js_call_uint(body, arity, &js_args)?)),
            b'J' => self.js_object_node(host_js_call_object(body, arity, &js_args)?),
            _ => return Err(EvalError::InvalidByteString),
        };
        let result = self.push_node(result);
        Ok(Some((arity + 1, self.pair(result, args[arity]))))
    }

    fn js_wrap(
        &mut self,
        tags: &str,
        args: &[NodeId],
    ) -> Result<Option<(usize, NodeId)>, EvalError> {
        validate_js_tags(tags.as_bytes())?;
        if args.len() < 2 {
            return Ok(None);
        }
        let program_handle = self.js_program_handle.ok_or(EvalError::UnsupportedJsFfi)?;
        let wrapper_index = self.register_js_wrapper_tags(tags)?;
        let stable_ptr = self.new_stable_ptr_handle(args[0])?;
        let object = match host_js_make_wrapper(program_handle, stable_ptr, wrapper_index) {
            Ok(object) => object,
            Err(err) => {
                let _ = self.free_stable_ptr(usize::try_from(stable_ptr).unwrap_or(usize::MAX));
                return Err(err);
            }
        };
        let result_node = self.js_object_node(object);
        let result = self.push_node(result_node);
        Ok(Some((2, self.pair(result, args[1]))))
    }

    fn register_js_wrapper_tags(&mut self, tags: &str) -> Result<u32, EvalError> {
        let index = u32::try_from(self.js_wrapper_tags.len()).map_err(|_| EvalError::Overflow)?;
        self.js_wrapper_tags.push(tags.to_owned());
        Ok(index)
    }

    fn js_value_node(&mut self, tag: u8, value: &JsValue) -> Result<NodeId, EvalError> {
        let node = match (tag, value) {
            (b'I', JsValue::Int(value)) => Node::Int(i64::from(*value)),
            (b'U', JsValue::UInt(value)) => Node::Int(i64::from(*value)),
            (b'D', JsValue::Double(value)) => Node::Float64(*value),
            (b'F', JsValue::Float(value)) => Node::Float32(*value),
            (b'B', JsValue::Bool(value)) => return Ok(self.prim(if *value { "A" } else { "K" })),
            (b'P', JsValue::Pointer(value)) => Node::Ptr(i64::from(*value)),
            (b'J', JsValue::Object(value)) => self.js_object_node(*value),
            (b'S', JsValue::Bytes(value)) => Node::bytes(value.clone()),
            _ => return Err(EvalError::InvalidByteString),
        };
        Ok(self.push_value_node(node))
    }

    fn js_value_from_node(&mut self, tag: u8, id: NodeId) -> Result<JsValue, EvalError> {
        match tag {
            b'V' => {
                let _ = self.reduce_node_whnf(id, FORCE_REDUCTION_LIMIT)?;
                Ok(JsValue::Unit)
            }
            b'I' => Ok(JsValue::Int(int_to_i32(self.eval_int(id)?)?)),
            b'U' => Ok(JsValue::UInt(
                u32::try_from(self.eval_int(id)?).map_err(|_| EvalError::Overflow)?,
            )),
            b'D' => Ok(JsValue::Double(self.eval_float64(id)?)),
            b'F' => Ok(JsValue::Float(self.eval_float32(id)?)),
            b'B' => Ok(JsValue::Bool(self.eval_bool(id)?)),
            b'P' => Ok(JsValue::Pointer(
                u32::try_from(self.eval_pointer_value(id)?).map_err(|_| EvalError::Overflow)?,
            )),
            b'J' => Ok(JsValue::Object(self.eval_js_object_handle(id)?)),
            b'S' => Ok(JsValue::Bytes(self.eval_bytes(id)?)),
            _ => Err(EvalError::InvalidByteString),
        }
    }

    fn eval_ffi_name(&mut self, id: NodeId) -> Result<String, EvalError> {
        let bytes = self.eval_string_bytes(id)?;
        String::from_utf8(bytes).map_err(|_| EvalError::InvalidByteString)
    }

    fn eval_string_bytes(&mut self, id: NodeId) -> Result<Vec<u8>, EvalError> {
        let root = self.reduce_node_whnf(id, FORCE_REDUCTION_LIMIT)?;
        let root = self.resolve(root)?;
        let bytes = match self.cold_node(root) {
            Some(Node::Bytes(bytes)) => bytes.as_slice().to_vec(),
            Some(Node::BytesView(_)) => self.bytes(root)?.to_vec(),
            Some(Node::MutableBytes(bytes)) => bytes.visible().to_vec(),
            _ => self.eval_char_list(root)?,
        };
        Ok(bytes)
    }

    fn eval_char_list(&mut self, mut id: NodeId) -> Result<Vec<u8>, EvalError> {
        let mut out = Vec::new();
        loop {
            let root = self.reduce_node_whnf(id, FORCE_REDUCTION_LIMIT)?;
            let root = self.resolve(root)?;
            if matches!(self.cell(root).prim(), Some(Prim::Known(KnownPrim::K))) {
                return Ok(out);
            }
            match self.cell(root).app_fields() {
                Some((fun, tail)) => {
                    let fun = self.resolve(fun)?;
                    let Some((cons, head)) = self.cell(fun).app_fields() else {
                        return Err(EvalError::InvalidByteString);
                    };
                    let cons = self.resolve(cons)?;
                    if matches!(self.cell(cons).prim(), Some(Prim::Known(KnownPrim::O))) {
                        out.extend(modified_utf8(self.eval_int(head)?)?);
                        id = tail;
                    } else {
                        return Err(EvalError::InvalidByteString);
                    }
                }
                _ => return Err(EvalError::InvalidByteString),
            }
        }
    }

    #[inline]
    fn eval_whnf_value<T>(
        &mut self,
        id: NodeId,
        extract: impl Fn(&Self, NodeId) -> Option<T>,
        expected: impl Fn(NodeId) -> EvalError,
    ) -> Result<T, EvalError> {
        let root = self.resolve(id)?;
        if let Some(value) = extract(self, root) {
            return Ok(value);
        }
        let root = self.reduce_node_whnf(root, FORCE_REDUCTION_LIMIT)?;
        let root = self.resolve(root)?;
        extract(self, root).ok_or_else(|| expected(root))
    }

    fn eval_int(&mut self, id: NodeId) -> Result<i64, EvalError> {
        self.eval_whnf_value(
            id,
            |program, root| program.cell(root).int_value(),
            EvalError::ExpectedInt,
        )
    }

    fn eval_int64(&mut self, id: NodeId) -> Result<i64, EvalError> {
        self.eval_whnf_value(
            id,
            |program, root| program.cell(root).int64_value(),
            EvalError::ExpectedInt64,
        )
    }

    fn eval_float64(&mut self, id: NodeId) -> Result<f64, EvalError> {
        self.eval_whnf_value(
            id,
            |program, root| program.cell(root).float64_value(),
            EvalError::ExpectedFloat64,
        )
    }

    fn eval_float32(&mut self, id: NodeId) -> Result<f32, EvalError> {
        self.eval_whnf_value(
            id,
            |program, root| program.cell(root).float32_value(),
            EvalError::ExpectedFloat32,
        )
    }

    fn eval_bool(&mut self, id: NodeId) -> Result<bool, EvalError> {
        self.eval_whnf_value(
            id,
            |program, root| match program.cell(root).prim() {
                Some(Prim::Known(KnownPrim::A)) => Some(true),
                Some(Prim::Known(KnownPrim::K)) => Some(false),
                _ => None,
            },
            EvalError::ExpectedInt,
        )
    }

    fn eval_thread_id(&mut self, id: NodeId) -> Result<i64, EvalError> {
        self.eval_whnf_value(
            id,
            |program, root| program.cell(root).thread_id_value(),
            EvalError::ExpectedThreadId,
        )
    }

    fn eval_pointer_value(&mut self, id: NodeId) -> Result<i64, EvalError> {
        let root = self.resolve(id)?;
        if let Some(value) = self.pointer_value_from_whnf(root) {
            self.trace_suspicious_pointer_value(root, value);
            return Ok(value);
        }
        let root = self.reduce_node_whnf(root, FORCE_REDUCTION_LIMIT)?;
        let root = self.resolve(root)?;
        let value = self
            .pointer_value_from_whnf(root)
            .ok_or(EvalError::ExpectedPointer(root))?;
        self.trace_suspicious_pointer_value(root, value);
        Ok(value)
    }

    fn pointer_value_from_whnf(&self, root: NodeId) -> Option<i64> {
        let cell = self.cell(root);
        if let Some(value) = cell.int_value() {
            return Some(value);
        }
        if let Some(value) = cell.ptr_value() {
            return Some(value);
        }
        if let Some(value) = cell.raw_fun_ptr_value() {
            return Some(value);
        }
        if let Some(value) = cell.thread_id_value() {
            return Some(value);
        }
        match cell.prim() {
            Some(prim) => std_handle_ptr(prim.name()),
            _ => None,
        }
    }

    fn trace_suspicious_pointer_value(&self, root: NodeId, ptr: i64) {
        if std::env::var_os("MHS_TRACE_INVALID_BYTES").is_none() {
            return;
        }
        if ptr >= 0 || handle_from_ptr(ptr).is_some() {
            return;
        }
        let suspicious = if ptr < BFILE_PTR_BASE {
            true
        } else if ptr < DIR_PTR_BASE {
            self.decode_bfile_pointer(ptr)
                .ok()
                .and_then(|slot| self.bfiles.get(slot))
                .and_then(Option::as_ref)
                .is_none()
        } else if ptr < ALLOCATION_PTR_BASE {
            self.decode_dir_pointer(ptr)
                .ok()
                .and_then(|slot| self.dirs.get(slot))
                .and_then(Option::as_ref)
                .is_none()
        } else {
            self.decode_allocation_pointer(ptr).is_err()
        };
        if suspicious {
            eprintln!(
                "suspicious pointer value: reductions={} ptr={ptr} root={}",
                self.reductions,
                self.node_trace_summary(root)
            );
        }
    }

    fn expected_bytes_error(&self, id: NodeId) -> EvalError {
        if self.trace_expected_bytes {
            eprintln!("expected bytes: reductions={}", self.reductions);
            eprintln!(
                "  gc_collections={} gc_last_live_nodes={} gc_last_free_nodes={} gc_current_nodes={} gc_current_free_nodes={} gc_allocations_since_collect={}",
                self.gc_collections,
                self.gc_last_live_nodes,
                self.gc_last_free_nodes,
                self.nodes.len(),
                self.free_nodes,
                self.gc_allocations_since_collect
            );
            eprintln!("  node={}", self.node_trace_summary(id));
        }
        EvalError::ExpectedBytes(id)
    }

    fn eval_foreign_ptr_id(&mut self, id: NodeId) -> Result<NodeId, EvalError> {
        self.eval_whnf_value(
            id,
            |program, root| {
                if matches!(program.cold_node(root), Some(Node::ForeignPtr(_))) {
                    return Some(root);
                }
                match program.cell(root).prim() {
                    Some(prim) if std_handle(prim.name()).is_some() => Some(root),
                    _ => None,
                }
            },
            EvalError::ExpectedForeignPtr,
        )
    }

    fn eval_bytes(&mut self, id: NodeId) -> Result<Vec<u8>, EvalError> {
        let id = self.eval_bytes_id(id)?;
        Ok(self.bytes(id)?.to_vec())
    }

    fn eval_bytes_id(&mut self, id: NodeId) -> Result<NodeId, EvalError> {
        self.eval_whnf_value(
            id,
            |program, root| match program.cold_node(root) {
                Some(Node::Bytes(_) | Node::BytesView(_) | Node::MutableBytes(_)) => Some(root),
                _ => None,
            },
            EvalError::ExpectedBytes,
        )
    }

    fn eval_array_id(&mut self, id: NodeId) -> Result<NodeId, EvalError> {
        self.eval_whnf_value(
            id,
            |program, root| match program.cold_node(root) {
                Some(Node::Array(_)) => Some(root),
                _ => None,
            },
            EvalError::ExpectedArray,
        )
    }

    fn array(&self, id: NodeId) -> Result<&[NodeId], EvalError> {
        match self.cold_node(id) {
            Some(Node::Array(items)) => Ok(items.as_slice()),
            _ => Err(EvalError::ExpectedArray(id)),
        }
    }

    fn array_mut(&mut self, id: NodeId) -> Result<&mut Vec<NodeId>, EvalError> {
        match self.cold_node_mut(id) {
            Some(Node::Array(items)) => Ok(items.as_mut()),
            _ => Err(EvalError::ExpectedArray(id)),
        }
    }

    fn bytes(&self, id: NodeId) -> Result<&[u8], EvalError> {
        match self.cold_node(id) {
            Some(Node::Bytes(bytes)) => Ok(bytes.as_slice()),
            Some(Node::BytesView(view)) => {
                let base = self.bytes(view.base)?;
                let end = view
                    .offset
                    .checked_add(view.len)
                    .ok_or(EvalError::Overflow)?;
                base.get(view.offset..end)
                    .ok_or(EvalError::InvalidByteString)
            }
            Some(Node::MutableBytes(bytes)) => Ok(bytes.visible()),
            _ => Err(self.expected_bytes_error(id)),
        }
    }

    fn byte_slice_node(&self, id: NodeId, offset: usize, len: usize) -> Result<Node, EvalError> {
        let bytes = self.bytes(id)?;
        let end = offset.checked_add(len).ok_or(EvalError::Overflow)?;
        if end > bytes.len() {
            return Err(EvalError::InvalidByteString);
        }
        match self.cold_node(id) {
            Some(Node::Bytes(_)) => Ok(Node::bytes_view(id, offset, len)),
            Some(Node::BytesView(view)) => {
                let offset = view.offset.checked_add(offset).ok_or(EvalError::Overflow)?;
                Ok(Node::bytes_view(view.base, offset, len))
            }
            Some(Node::MutableBytes(_)) => Ok(Node::bytes(bytes[offset..end].to_vec())),
            _ => Err(self.expected_bytes_error(id)),
        }
    }

    fn materialize_bytes_view_for_write(&mut self, id: NodeId) -> Result<(), EvalError> {
        if matches!(self.cold_node(id), Some(Node::BytesView(_))) {
            let bytes = self.bytes(id)?.to_vec();
            self.set_node_at(id.index(), Node::bytes(bytes));
        }
        Ok(())
    }

    fn new_mutable_bytes(&mut self, size: usize, capacity: usize) -> Result<NodeId, EvalError> {
        if size > capacity {
            return Err(EvalError::InvalidByteString);
        }
        let bytes = vec![0; capacity];
        Ok(
            self.push_node(Node::MutableBytes(Box::new(MutableBytesNode {
                bytes,
                size,
                capacity,
            }))),
        )
    }

    fn freeze_bytes(&mut self, id: NodeId) -> Result<NodeId, EvalError> {
        let frozen = match self.cold_node(id) {
            Some(Node::Bytes(_)) => return Ok(id),
            Some(Node::BytesView(_)) => return Ok(id),
            Some(Node::MutableBytes(bytes)) => bytes.visible().to_vec(),
            _ => return Err(self.expected_bytes_error(id)),
        };
        self.set_node_at(id.index(), Node::bytes(frozen));
        Ok(id)
    }

    fn append_byte(&mut self, id: NodeId, byte: u8) -> Result<(), EvalError> {
        self.materialize_bytes_view_for_write(id)?;
        match self.cold_node_mut(id) {
            Some(Node::Bytes(bytes)) => {
                bytes.push(byte);
                Ok(())
            }
            Some(Node::MutableBytes(bytes)) => {
                if bytes.size >= bytes.capacity {
                    bytes.capacity = bytes
                        .capacity
                        .checked_add(bytes.capacity / 2)
                        .and_then(|capacity| capacity.checked_add(2))
                        .ok_or(EvalError::Overflow)?;
                    if bytes.capacity < bytes.size {
                        return Err(EvalError::Overflow);
                    }
                    if bytes.capacity > bytes.bytes.capacity() {
                        bytes.bytes.reserve(bytes.capacity - bytes.bytes.capacity());
                    }
                    bytes.bytes.resize(bytes.capacity, 0);
                }
                bytes.bytes[bytes.size] = byte;
                bytes.size += 1;
                Ok(())
            }
            _ => Err(self.expected_bytes_error(id)),
        }
    }

    fn append_bytes(&mut self, id: NodeId, bytes: &[u8]) -> Result<(), EvalError> {
        self.materialize_bytes_view_for_write(id)?;
        match self.cold_node_mut(id) {
            Some(Node::Bytes(dst)) => {
                dst.extend_from_slice(bytes);
                Ok(())
            }
            Some(Node::MutableBytes(_)) => {
                for &byte in bytes {
                    self.append_byte(id, byte)?;
                }
                Ok(())
            }
            _ => Err(self.expected_bytes_error(id)),
        }
    }

    fn read_byte_unchecked_prim(&self, id: NodeId, index: usize) -> Result<u8, EvalError> {
        match self.cold_node(id) {
            Some(Node::Bytes(bytes)) => bytes
                .get(index)
                .copied()
                .ok_or(EvalError::InvalidByteString),
            Some(Node::BytesView(_)) => self
                .bytes(id)?
                .get(index)
                .copied()
                .ok_or(EvalError::InvalidByteString),
            Some(Node::MutableBytes(bytes)) => bytes
                .bytes
                .get(index)
                .copied()
                .ok_or(EvalError::InvalidByteString),
            _ => Err(self.expected_bytes_error(id)),
        }
    }

    fn byte_prim_lengths(&self, id: NodeId) -> Result<(usize, usize), EvalError> {
        match self.cold_node(id) {
            Some(Node::Bytes(bytes)) => Ok((bytes.len(), bytes.len())),
            Some(Node::BytesView(view)) => Ok((view.len, view.len)),
            Some(Node::MutableBytes(bytes)) => Ok((bytes.size, bytes.bytes.len())),
            _ => Err(self.expected_bytes_error(id)),
        }
    }

    fn write_byte_unchecked_prim(
        &mut self,
        id: NodeId,
        index: usize,
        byte: u8,
    ) -> Result<(), EvalError> {
        self.materialize_bytes_view_for_write(id)?;
        match self.cold_node_mut(id) {
            Some(Node::Bytes(bytes)) => {
                let slot = bytes.get_mut(index).ok_or(EvalError::InvalidByteString)?;
                *slot = byte;
                Ok(())
            }
            Some(Node::MutableBytes(bytes)) => {
                let slot = bytes
                    .bytes
                    .get_mut(index)
                    .ok_or(EvalError::InvalidByteString)?;
                *slot = byte;
                Ok(())
            }
            _ => Err(self.expected_bytes_error(id)),
        }
    }

    fn alloc_memory(&mut self, size: usize) -> Result<i64, EvalError> {
        let bytes = vec![0; size];
        let slot = if let Some(slot) = self.allocations.iter().position(Option::is_none) {
            self.allocations[slot] = Some(bytes);
            slot
        } else {
            self.allocations.push(Some(bytes));
            self.allocations.len() - 1
        };
        self.pointer_for_allocation(slot, 0)
    }

    fn alloc_c_string_bytes(&mut self, bytes: &[u8]) -> Result<i64, EvalError> {
        let len = bytes.len().checked_add(1).ok_or(EvalError::Overflow)?;
        let ptr = self.alloc_memory(len)?;
        self.write_pointer_bytes(ptr, bytes)?;
        self.write_pointer_bytes(
            ptr.checked_add(i64::try_from(bytes.len()).map_err(|_| EvalError::Overflow)?)
                .ok_or(EvalError::Overflow)?,
            &[0],
        )?;
        Ok(ptr)
    }

    fn errno_ptr(&mut self) -> Result<i64, EvalError> {
        if let Some(ptr) = self.errno_ptr {
            return Ok(ptr);
        }
        let ptr = self.alloc_memory(size_of::<std::os::raw::c_int>())?;
        self.errno_ptr = Some(ptr);
        self.write_errno_cell(ptr)?;
        Ok(ptr)
    }

    fn set_errno_value(&mut self, value: i32) -> Result<(), EvalError> {
        self.errno_value = value;
        if let Some(ptr) = self.errno_ptr {
            self.write_errno_cell(ptr)?;
        }
        Ok(())
    }

    fn write_errno_cell(&mut self, ptr: i64) -> Result<(), EvalError> {
        let value = self.errno_value as std::os::raw::c_int;
        self.write_pointer_bytes(ptr, &value.to_ne_bytes())
    }

    fn host_int_node(&mut self, result: HostIntResult) -> Result<Node, EvalError> {
        if let Some(errno) = result.errno {
            self.set_errno_value(errno)?;
        }
        Ok(Node::Int(result.value))
    }

    #[cfg_attr(not(all(unix, not(target_arch = "wasm32"))), allow(dead_code))]
    fn syscall_result_node(&mut self, value: i64) -> Result<Node, EvalError> {
        if value < 0 {
            self.set_errno_value(last_errno())?;
        }
        Ok(Node::Int(value))
    }

    fn gettimeofday_node(
        &mut self,
        timeval_ptr: i64,
        timezone_ptr: i64,
    ) -> Result<Node, EvalError> {
        #[cfg(all(unix, not(target_arch = "wasm32")))]
        {
            let _ = timezone_ptr;
            let mut tv = std::mem::MaybeUninit::<libc::timeval>::uninit();
            // SAFETY: libc writes the timeval on success. The timezone argument is obsolete.
            let rc = unsafe { libc::gettimeofday(tv.as_mut_ptr(), std::ptr::null_mut()) };
            if rc < 0 {
                return self.syscall_result_node(i64::from(rc));
            }
            if timeval_ptr != 0 {
                // SAFETY: gettimeofday succeeded, so tv is initialized.
                let tv = unsafe { tv.assume_init() };
                // SAFETY: tv is a plain C struct; copying its bytes matches the C FFI layout.
                let bytes = unsafe {
                    std::slice::from_raw_parts(
                        (&tv as *const libc::timeval).cast::<u8>(),
                        size_of::<libc::timeval>(),
                    )
                };
                self.write_pointer_bytes(timeval_ptr, bytes)?;
            }
            Ok(Node::Int(i64::from(rc)))
        }
        #[cfg(not(all(unix, not(target_arch = "wasm32"))))]
        {
            let _ = (timeval_ptr, timezone_ptr);
            self.host_int_node(HostIntResult::err(errno_i32("ENOSYS")))
        }
    }

    fn accept_socket_node(
        &mut self,
        fd: i32,
        addr_ptr: i64,
        len_ptr: i64,
    ) -> Result<Node, EvalError> {
        #[cfg(all(unix, not(target_arch = "wasm32")))]
        {
            let mut len = 0 as libc::socklen_t;
            let mut buffer = Vec::new();
            let (addr_arg, len_arg) = if addr_ptr != 0 && len_ptr != 0 {
                let raw_len = self.peek_unsigned(len_ptr, size_of::<libc::socklen_t>())?;
                len = libc::socklen_t::try_from(raw_len).map_err(|_| EvalError::Overflow)?;
                buffer.resize(usize::try_from(len).map_err(|_| EvalError::Overflow)?, 0);
                (buffer.as_mut_ptr().cast(), &mut len as *mut _)
            } else {
                (std::ptr::null_mut(), std::ptr::null_mut())
            };
            // SAFETY: pointers either point to temporary buffers or are null, as accepted by libc.
            let rc = unsafe { libc::accept(fd, addr_arg, len_arg) };
            if rc >= 0 && addr_ptr != 0 && len_ptr != 0 {
                let written = usize::try_from(len)
                    .map_err(|_| EvalError::Overflow)?
                    .min(buffer.len());
                self.write_pointer_bytes(addr_ptr, &buffer[..written])?;
                self.poke_unsigned(
                    len_ptr,
                    size_of::<libc::socklen_t>(),
                    u64::try_from(len).map_err(|_| EvalError::Overflow)?,
                )?;
            }
            self.syscall_result_node(i64::from(rc))
        }
        #[cfg(not(all(unix, not(target_arch = "wasm32"))))]
        {
            let _ = (fd, addr_ptr, len_ptr);
            self.host_int_node(HostIntResult::err(errno_i32("ENOSYS")))
        }
    }

    fn getsockopt_node(
        &mut self,
        fd: i32,
        level: i32,
        optname: i32,
        optval_ptr: i64,
        optlen_ptr: i64,
    ) -> Result<Node, EvalError> {
        #[cfg(all(unix, not(target_arch = "wasm32")))]
        {
            if optlen_ptr == 0 {
                return self.host_int_node(HostIntResult::err(errno_i32("EINVAL")));
            }
            let raw_len = self.peek_unsigned(optlen_ptr, size_of::<libc::socklen_t>())?;
            let mut len = libc::socklen_t::try_from(raw_len).map_err(|_| EvalError::Overflow)?;
            let mut buffer = vec![0; usize::try_from(len).map_err(|_| EvalError::Overflow)?];
            let optval = if optval_ptr == 0 {
                std::ptr::null_mut()
            } else {
                buffer.as_mut_ptr().cast()
            };
            // SAFETY: optval points to a temporary output buffer or is null; len points to stack storage.
            let rc = unsafe { libc::getsockopt(fd, level, optname, optval, &mut len) };
            if rc >= 0 {
                if optval_ptr != 0 {
                    let written = usize::try_from(len)
                        .map_err(|_| EvalError::Overflow)?
                        .min(buffer.len());
                    self.write_pointer_bytes(optval_ptr, &buffer[..written])?;
                }
                self.poke_unsigned(
                    optlen_ptr,
                    size_of::<libc::socklen_t>(),
                    u64::try_from(len).map_err(|_| EvalError::Overflow)?,
                )?;
            }
            self.syscall_result_node(i64::from(rc))
        }
        #[cfg(not(all(unix, not(target_arch = "wasm32"))))]
        {
            let _ = (fd, level, optname, optval_ptr, optlen_ptr);
            self.host_int_node(HostIntResult::err(errno_i32("ENOSYS")))
        }
    }

    fn recv_socket_node(
        &mut self,
        fd: i32,
        buf_ptr: i64,
        len: usize,
        flags: i32,
    ) -> Result<Node, EvalError> {
        #[cfg(all(unix, not(target_arch = "wasm32")))]
        {
            let mut buffer = vec![0; len];
            let ptr = if len == 0 {
                std::ptr::null_mut()
            } else {
                buffer.as_mut_ptr().cast()
            };
            // SAFETY: ptr points to a temporary buffer large enough for len bytes, or is null for len 0.
            let rc = unsafe { libc::recv(fd, ptr, len, flags) };
            if rc >= 0 && len != 0 {
                let read = usize::try_from(rc).map_err(|_| EvalError::Overflow)?;
                self.write_pointer_bytes(buf_ptr, &buffer[..read])?;
            }
            self.syscall_result_node(rc as i64)
        }
        #[cfg(not(all(unix, not(target_arch = "wasm32"))))]
        {
            let _ = (fd, buf_ptr, len, flags);
            self.host_int_node(HostIntResult::err(errno_i32("ENOSYS")))
        }
    }

    fn send_socket_node(
        &mut self,
        fd: i32,
        buf_ptr: i64,
        len: usize,
        flags: i32,
    ) -> Result<Node, EvalError> {
        #[cfg(all(unix, not(target_arch = "wasm32")))]
        {
            let buffer = if len == 0 {
                Vec::new()
            } else {
                self.read_pointer_bytes(buf_ptr, len)?
            };
            let ptr = if len == 0 {
                std::ptr::null()
            } else {
                buffer.as_ptr().cast()
            };
            // SAFETY: ptr points to len bytes copied from guest memory, or is null for len 0.
            let rc = unsafe { libc::send(fd, ptr, len, flags) };
            self.syscall_result_node(rc as i64)
        }
        #[cfg(not(all(unix, not(target_arch = "wasm32"))))]
        {
            let _ = (fd, buf_ptr, len, flags);
            self.host_int_node(HostIntResult::err(errno_i32("ENOSYS")))
        }
    }

    fn new_mpz_node(&mut self) -> Result<Node, EvalError> {
        let bigint = self.push_node(Node::bigint(b"0".to_vec()));
        let ptr = self.pointer_for_node(bigint, 0)?;
        Ok(self.foreign_ptr_node(None, 0, ptr))
    }

    fn mpz_node_id(&self, ptr: i64) -> Result<NodeId, EvalError> {
        let (slot, offset) = self.decode_pointer(ptr)?;
        if offset != 0 {
            return Err(EvalError::ExpectedForeignPtr(NodeId::from_index(slot)));
        }
        let id = NodeId::from_index(slot);
        match self.cold_node(id) {
            Some(Node::BigInt(_)) => Ok(id),
            _ => Err(EvalError::ExpectedForeignPtr(id)),
        }
    }

    fn mpz_decimal_bytes_for_ptr(&self, ptr: i64) -> Option<&[u8]> {
        let (slot, offset) = self.decode_pointer(ptr).ok()?;
        if offset != 0 {
            return None;
        }
        let id = NodeId::from_index(slot);
        match self.cold_node(id)? {
            Node::BigInt(bytes) => Some(bytes.as_slice()),
            _ => None,
        }
    }

    fn mpz_value(&self, ptr: i64) -> Result<MpzValue, EvalError> {
        let id = self.mpz_node_id(ptr)?;
        let Some(Node::BigInt(bytes)) = self.cold_node(id) else {
            return Err(EvalError::ExpectedForeignPtr(id));
        };
        MpzValue::parse_decimal(bytes).map_err(|_| EvalError::InvalidByteString)
    }

    fn write_mpz_value(&mut self, ptr: i64, value: MpzValue) -> Result<(), EvalError> {
        let id = self.mpz_node_id(ptr)?;
        self.set_node_at(id.index(), Node::bigint(value.to_decimal_bytes()));
        Ok(())
    }

    fn write_strerror(&mut self, errno: i32, ptr: i64, size: usize) -> Result<i64, EvalError> {
        if size == 0 {
            let erange = errno_i32("ERANGE");
            self.set_errno_value(erange)?;
            return Ok(i64::from(erange));
        }
        let bytes = strerror_bytes(errno);
        let truncated = bytes.len() + 1 > size;
        let copy_len = if truncated { size - 1 } else { bytes.len() };
        let mut out = Vec::with_capacity(copy_len + 1);
        out.extend_from_slice(&bytes[..copy_len]);
        out.push(0);
        self.write_pointer_bytes(ptr, &out)?;
        if truncated {
            let erange = errno_i32("ERANGE");
            self.set_errno_value(erange)?;
            Ok(i64::from(erange))
        } else {
            Ok(0)
        }
    }

    fn calloc_memory(&mut self, count: usize, size: usize) -> Result<i64, EvalError> {
        let len = count.checked_mul(size).ok_or(EvalError::Overflow)?;
        self.alloc_memory(len)
    }

    fn realloc_memory(&mut self, ptr: i64, size: usize) -> Result<i64, EvalError> {
        if ptr == 0 {
            return self.alloc_memory(size);
        }
        let (slot, offset) = self.decode_allocation_pointer(ptr)?;
        if offset != 0 {
            return Err(EvalError::InvalidByteString);
        }
        let bytes = self
            .allocations
            .get_mut(slot)
            .and_then(Option::as_mut)
            .ok_or(EvalError::InvalidByteString)?;
        bytes.resize(size, 0);
        self.pointer_for_allocation(slot, 0)
    }

    fn free_memory(&mut self, ptr: i64) -> Result<(), EvalError> {
        if ptr == 0 {
            return Ok(());
        }
        let (slot, offset) = self.decode_allocation_pointer(ptr)?;
        if offset != 0 {
            return Err(EvalError::InvalidByteString);
        }
        let slot = self
            .allocations
            .get_mut(slot)
            .ok_or(EvalError::InvalidByteString)?;
        if slot.is_none() {
            return Err(EvalError::InvalidByteString);
        }
        *slot = None;
        Ok(())
    }

    fn alloc_bfile(&mut self, bfile: BFile) -> Result<i64, EvalError> {
        let slot = if let Some(slot) = self.bfiles.iter().position(Option::is_none) {
            self.bfiles[slot] = Some(bfile);
            slot
        } else {
            self.bfiles.push(Some(bfile));
            self.bfiles.len() - 1
        };
        self.pointer_for_bfile(slot)
    }

    fn alloc_dir(&mut self, entries: Vec<Vec<u8>>) -> Result<i64, EvalError> {
        let dir = DirHandle { entries, pos: 0 };
        let slot = if let Some(slot) = self.dirs.iter().position(Option::is_none) {
            self.dirs[slot] = Some(dir);
            slot
        } else {
            self.dirs.push(Some(dir));
            self.dirs.len() - 1
        };
        self.pointer_for_dir(slot)
    }

    fn alloc_environ(&mut self) -> Result<i64, EvalError> {
        let vars = environ_bytes();
        let mut pointers = Vec::with_capacity(
            (vars.len() + 1)
                .checked_mul(size_of::<i64>())
                .ok_or(EvalError::Overflow)?,
        );
        for mut var in vars {
            var.push(0);
            let ptr = self.alloc_memory(var.len())?;
            self.write_pointer_bytes(ptr, &var)?;
            pointers.extend_from_slice(&ptr.to_ne_bytes());
        }
        pointers.extend_from_slice(&0_i64.to_ne_bytes());
        let ptr = self.alloc_memory(pointers.len())?;
        self.write_pointer_bytes(ptr, &pointers)?;
        Ok(ptr)
    }

    fn bfile_permissions(&self, ptr: i64) -> Result<(bool, bool), EvalError> {
        if let Some(handle) = handle_from_ptr(ptr) {
            return Ok(match handle {
                StdHandle::Stdin => (true, false),
                StdHandle::Stdout | StdHandle::Stderr => (false, true),
            });
        }
        let bfile = self.bfile(ptr)?;
        Ok((bfile.readable, bfile.writable))
    }

    fn add_utf8_bfile(&mut self, ptr: i64) -> Result<i64, EvalError> {
        let (readable, writable) = self.bfile_permissions(ptr)?;
        self.alloc_bfile(BFile {
            kind: BFileKind::Utf8 {
                inner: ptr,
                unget: None,
                pending: Vec::new(),
                pending_pos: 0,
            },
            readable,
            writable,
        })
    }

    fn add_crlf_bfile(&mut self, ptr: i64) -> Result<i64, EvalError> {
        let (readable, writable) = self.bfile_permissions(ptr)?;
        self.alloc_bfile(BFile {
            kind: BFileKind::Crlf { inner: ptr },
            readable,
            writable,
        })
    }

    fn add_rle_bfile(&mut self, ptr: i64, read: bool) -> Result<i64, EvalError> {
        let (inner_readable, inner_writable) = self.bfile_permissions(ptr)?;
        self.alloc_bfile(BFile {
            kind: BFileKind::Rle {
                inner: ptr,
                read,
                count: 0,
                byte: -1,
                unget: None,
            },
            readable: read && inner_readable,
            writable: !read && inner_writable,
        })
    }

    fn add_base64_bfile(&mut self, ptr: i64, read: bool) -> Result<i64, EvalError> {
        let (inner_readable, inner_writable) = self.bfile_permissions(ptr)?;
        self.alloc_bfile(BFile {
            kind: BFileKind::Base64 {
                inner: ptr,
                read,
                encbuf: [0; 3],
                encpos: 0,
                linelen: if read { 0 } else { 76 },
                outcol: 0,
                unget: None,
                outbuf: [0; 3],
                outpos: 0,
                outlen: 0,
            },
            readable: read && inner_readable,
            writable: !read && inner_writable,
        })
    }

    fn add_lz77_bfile(&mut self, ptr: i64, read: bool) -> Result<i64, EvalError> {
        let (inner_readable, inner_writable) = self.bfile_permissions(ptr)?;
        if read {
            if !inner_readable {
                return Err(EvalError::InvalidHandle);
            }
            let magic = self.read_bfile_bytes(ptr, 3)?;
            if magic != b"LZ1" {
                return Err(EvalError::InvalidByteString);
            }
            let len = self.read_bfile_u32_le(ptr)?;
            let compressed = self.read_bfile_bytes(ptr, len)?;
            if compressed.len() != len {
                return Err(EvalError::InvalidByteString);
            }
            let buffer = lz77_decompress(&compressed)?;
            self.alloc_bfile(BFile {
                kind: BFileKind::Lz77 {
                    inner: ptr,
                    read: true,
                    buffer,
                    pos: 0,
                    numflush: 0,
                },
                readable: true,
                writable: false,
            })
        } else {
            if !inner_writable {
                return Err(EvalError::InvalidHandle);
            }
            self.alloc_bfile(BFile {
                kind: BFileKind::Lz77 {
                    inner: ptr,
                    read: false,
                    buffer: Vec::with_capacity(25_000),
                    pos: 0,
                    numflush: 0,
                },
                readable: false,
                writable: true,
            })
        }
    }

    fn read_bfile_u32_le(&mut self, ptr: i64) -> Result<usize, EvalError> {
        let bytes = self.read_bfile_bytes(ptr, 4)?;
        let bytes: [u8; 4] = bytes.try_into().map_err(|_| EvalError::InvalidByteString)?;
        usize::try_from(u32::from_le_bytes(bytes)).map_err(|_| EvalError::Overflow)
    }

    fn add_bwt_bfile(&mut self, ptr: i64, read: bool) -> Result<i64, EvalError> {
        let (inner_readable, inner_writable) = self.bfile_permissions(ptr)?;
        if read {
            if !inner_readable {
                return Err(EvalError::InvalidHandle);
            }
            let magic = self.read_bfile_bytes(ptr, 3)?;
            if magic != b"BW1" {
                return Err(EvalError::InvalidByteString);
            }
            let len = self.read_bfile_u32_le(ptr)?;
            let zero = self.read_bfile_u32_le(ptr)?;
            let last = self.read_bfile_bytes(ptr, len)?;
            if last.len() != len {
                return Err(EvalError::InvalidByteString);
            }
            let buffer = bwt_decode(&last, zero)?;
            self.alloc_bfile(BFile {
                kind: BFileKind::Bwt {
                    inner: ptr,
                    read: true,
                    buffer,
                    pos: 0,
                    numflush: 0,
                },
                readable: true,
                writable: false,
            })
        } else {
            if !inner_writable {
                return Err(EvalError::InvalidHandle);
            }
            self.alloc_bfile(BFile {
                kind: BFileKind::Bwt {
                    inner: ptr,
                    read: false,
                    buffer: Vec::with_capacity(25_000),
                    pos: 0,
                    numflush: 0,
                },
                readable: false,
                writable: true,
            })
        }
    }

    fn add_lzma_bfile(&mut self, ptr: i64, read: bool) -> Result<i64, EvalError> {
        let (inner_readable, inner_writable) = self.bfile_permissions(ptr)?;
        if read {
            if !inner_readable {
                return Err(EvalError::InvalidHandle);
            }
            let magic = self.read_bfile_bytes(ptr, 3)?;
            if magic != b"LZ2" {
                return Err(EvalError::InvalidByteString);
            }
            let len = self.read_bfile_u32_le(ptr)?;
            let compressed = self.read_bfile_bytes(ptr, len)?;
            if compressed.len() != len {
                return Err(EvalError::InvalidByteString);
            }
            let buffer = lzma_decompress_payload(&compressed)?;
            self.alloc_bfile(BFile {
                kind: BFileKind::Lzma {
                    inner: ptr,
                    read: true,
                    buffer,
                    pos: 0,
                    numflush: 0,
                },
                readable: true,
                writable: false,
            })
        } else {
            if !inner_writable {
                return Err(EvalError::InvalidHandle);
            }
            self.alloc_bfile(BFile {
                kind: BFileKind::Lzma {
                    inner: ptr,
                    read: false,
                    buffer: Vec::with_capacity(25_000),
                    pos: 0,
                    numflush: 0,
                },
                readable: false,
                writable: true,
            })
        }
    }

    fn add_buf_bfile(&mut self, ptr: i64, bufsize: i64) -> Result<i64, EvalError> {
        let (readable, writable) = self.bfile_permissions(ptr)?;
        let linebuf = bufsize < 0;
        let size = if linebuf {
            bufsize.checked_neg().ok_or(EvalError::Overflow)?
        } else {
            bufsize
        };
        let size = int_to_usize(size)?;
        self.alloc_bfile(BFile {
            kind: BFileKind::Buf {
                inner: ptr,
                unget: None,
                buffer: vec![0; size],
                cur: 0,
                pos: 0,
                linebuf,
                read: false,
            },
            readable,
            writable,
        })
    }

    fn pointer_for_node(&mut self, id: NodeId, offset: usize) -> Result<i64, EvalError> {
        let offset = i64::try_from(offset).map_err(|_| EvalError::Overflow)?;
        if offset >= NODE_PTR_STRIDE {
            return Err(EvalError::Overflow);
        }
        let slot = if let Some(slot) = self.node_pointer_slots.get(&id) {
            *slot
        } else {
            let slot = self.node_pointers.len();
            self.node_pointers.push(id);
            self.node_pointer_slots.insert(id, slot);
            slot
        };
        let slot_word = slot
            .checked_add(1)
            .and_then(|slot| i64::try_from(slot).ok())
            .ok_or(EvalError::Overflow)?;
        if slot_word >= (1_i64 << 31) {
            return Err(EvalError::Overflow);
        }
        slot_word
            .checked_mul(NODE_PTR_STRIDE)
            .and_then(|base| base.checked_add(offset))
            .ok_or(EvalError::Overflow)
    }

    fn pointer_for_allocation(&self, slot: usize, offset: usize) -> Result<i64, EvalError> {
        let slot = i64::try_from(slot).map_err(|_| EvalError::Overflow)?;
        let offset = i64::try_from(offset).map_err(|_| EvalError::Overflow)?;
        if offset >= ALLOCATION_PTR_STRIDE {
            return Err(EvalError::Overflow);
        }
        let ptr = ALLOCATION_PTR_BASE
            .checked_add(
                slot.checked_mul(ALLOCATION_PTR_STRIDE)
                    .and_then(|raw| raw.checked_add(offset))
                    .ok_or(EvalError::Overflow)?,
            )
            .ok_or(EvalError::Overflow)?;
        if ptr >= 0 {
            return Err(EvalError::Overflow);
        }
        Ok(ptr)
    }

    fn pointer_for_bfile(&self, slot: usize) -> Result<i64, EvalError> {
        let slot = i64::try_from(slot).map_err(|_| EvalError::Overflow)?;
        BFILE_PTR_BASE
            .checked_add(
                slot.checked_mul(BFILE_PTR_STRIDE)
                    .ok_or(EvalError::Overflow)?,
            )
            .filter(|ptr| *ptr < DIR_PTR_BASE)
            .ok_or(EvalError::Overflow)
    }

    fn pointer_for_dir(&self, slot: usize) -> Result<i64, EvalError> {
        let slot = i64::try_from(slot).map_err(|_| EvalError::Overflow)?;
        DIR_PTR_BASE
            .checked_add(
                slot.checked_mul(DIR_PTR_STRIDE)
                    .ok_or(EvalError::Overflow)?,
            )
            .filter(|ptr| *ptr < ALLOCATION_PTR_BASE)
            .ok_or(EvalError::Overflow)
    }

    fn decode_pointer(&self, ptr: i64) -> Result<(usize, usize), EvalError> {
        if ptr <= 0 {
            return Err(trace_invalid_bytes!(self, "decode_pointer ptr={ptr}"));
        }
        let slot_word = usize::try_from(ptr >> 32)
            .map_err(|_| trace_invalid_bytes!(self, "decode_pointer block ptr={ptr}"))?;
        if slot_word == 0 {
            return Err(trace_invalid_bytes!(
                self,
                "decode_pointer zero slot ptr={ptr}"
            ));
        }
        let offset = usize::try_from(ptr & 0xffff_ffff)
            .map_err(|_| trace_invalid_bytes!(self, "decode_pointer offset ptr={ptr}"))?;
        let slot = slot_word - 1;
        let block = self.node_pointers.get(slot).copied().ok_or_else(|| {
            trace_invalid_bytes!(
                self,
                "decode_pointer missing node slot ptr={ptr} slot={slot} slots={}",
                self.node_pointers.len()
            )
        })?;
        Ok((block.index(), offset))
    }

    fn decode_allocation_pointer(&self, ptr: i64) -> Result<(usize, usize), EvalError> {
        if ptr < ALLOCATION_PTR_BASE || ptr >= 0 {
            return Err(trace_invalid_bytes!(
                self,
                "decode_allocation_pointer ptr={ptr}"
            ));
        }
        let raw = ptr
            .checked_sub(ALLOCATION_PTR_BASE)
            .ok_or(EvalError::Overflow)?;
        let slot = usize::try_from(raw / ALLOCATION_PTR_STRIDE)
            .map_err(|_| trace_invalid_bytes!(self, "allocation slot ptr={ptr} raw={raw}"))?;
        let offset = usize::try_from(raw % ALLOCATION_PTR_STRIDE)
            .map_err(|_| trace_invalid_bytes!(self, "allocation offset ptr={ptr} raw={raw}"))?;
        let bytes = self
            .allocations
            .get(slot)
            .and_then(Option::as_ref)
            .ok_or_else(|| {
                trace_invalid_bytes!(self, "allocation missing ptr={ptr} slot={slot}")
            })?;
        if offset > bytes.len() {
            return Err(trace_invalid_bytes!(
                self,
                "allocation offset out of range ptr={ptr} slot={slot} offset={offset} len={}",
                bytes.len()
            ));
        }
        Ok((slot, offset))
    }

    fn decode_bfile_pointer(&self, ptr: i64) -> Result<usize, EvalError> {
        if !(BFILE_PTR_BASE..DIR_PTR_BASE).contains(&ptr) {
            return Err(EvalError::InvalidHandle);
        }
        let raw = ptr.checked_sub(BFILE_PTR_BASE).ok_or(EvalError::Overflow)?;
        if raw % BFILE_PTR_STRIDE != 0 {
            return Err(EvalError::InvalidHandle);
        }
        usize::try_from(raw / BFILE_PTR_STRIDE).map_err(|_| EvalError::InvalidHandle)
    }

    fn decode_dir_pointer(&self, ptr: i64) -> Result<usize, EvalError> {
        if !(DIR_PTR_BASE..ALLOCATION_PTR_BASE).contains(&ptr) {
            return Err(EvalError::InvalidHandle);
        }
        let raw = ptr.checked_sub(DIR_PTR_BASE).ok_or(EvalError::Overflow)?;
        if raw % DIR_PTR_STRIDE != 0 {
            return Err(EvalError::InvalidHandle);
        }
        usize::try_from(raw / DIR_PTR_STRIDE).map_err(|_| EvalError::InvalidHandle)
    }

    fn allocation_bytes(&self, ptr: i64) -> Result<Option<&[u8]>, EvalError> {
        if ptr < ALLOCATION_PTR_BASE || ptr >= 0 {
            return Ok(None);
        }
        let (slot, offset) = self.decode_allocation_pointer(ptr)?;
        let bytes = self
            .allocations
            .get(slot)
            .and_then(Option::as_ref)
            .ok_or_else(|| {
                trace_invalid_bytes!(self, "allocation read missing ptr={ptr} slot={slot}")
            })?;
        Ok(Some(&bytes[offset..]))
    }

    fn pointer_bytes(&self, ptr: i64) -> Result<&[u8], EvalError> {
        if let Some(bytes) = self.allocation_bytes(ptr)? {
            return Ok(bytes);
        }
        let (base, offset) = self.decode_pointer(ptr)?;
        if base >= self.nodes.len() {
            return Err(trace_invalid_bytes!(
                self,
                "pointer base missing ptr={ptr} base={base} offset={offset} nodes={}",
                self.nodes.len()
            ));
        };
        let node_id = NodeId::from_index(base);
        let bytes = match self.cold_node(node_id) {
            Some(Node::Bytes(bytes)) => bytes.as_slice(),
            Some(Node::BytesView(_)) => self.bytes(node_id)?,
            Some(Node::MutableBytes(bytes)) => bytes.bytes.as_slice(),
            Some(Node::ForeignPtr(foreign_ptr)) if foreign_ptr.bytes.is_some() => {
                let bytes = foreign_ptr.bytes.as_ref().expect("checked above");
                let offset = foreign_ptr
                    .offset
                    .checked_add(offset)
                    .ok_or(EvalError::Overflow)?;
                return bytes.get(offset..).ok_or_else(|| {
                    trace_invalid_bytes!(
                        self,
                        "foreign pointer offset out of range ptr={ptr} base={base} offset={offset} len={}",
                        bytes.len()
                    )
                });
            }
            _ => {
                return Err(trace_invalid_bytes!(
                    self,
                    "pointer base not bytes ptr={ptr} base={base} offset={offset} kind={}",
                    self.profile_head_key(NodeId::from_index(base))
                ));
            }
        };
        if offset > bytes.len() {
            return Err(trace_invalid_bytes!(
                self,
                "pointer offset out of range ptr={ptr} base={base} offset={offset} len={}",
                bytes.len()
            ));
        }
        Ok(&bytes[offset..])
    }

    fn write_pointer_bytes(&mut self, ptr: i64, bytes: &[u8]) -> Result<(), EvalError> {
        if ptr >= ALLOCATION_PTR_BASE && ptr < 0 {
            let (slot, offset) = self.decode_allocation_pointer(ptr)?;
            let write_len = bytes.len();
            let available = self.allocations[slot]
                .as_ref()
                .expect("checked allocation slot")
                .len()
                .saturating_sub(offset);
            if available < write_len {
                return Err(trace_invalid_bytes!(
                    self,
                    "allocation write too short ptr={ptr} slot={slot} offset={offset} write_len={write_len} available={available}"
                ));
            }
            let dst = self.allocations[slot]
                .as_mut()
                .expect("checked allocation slot");
            dst[offset..offset + write_len].copy_from_slice(bytes);
            return Ok(());
        }
        let (base, offset) = self.decode_pointer(ptr)?;
        if base >= self.nodes.len() {
            return Err(trace_invalid_bytes!(
                self,
                "write pointer base missing ptr={ptr} base={base} offset={offset} nodes={}",
                self.nodes.len()
            ));
        }
        let write_len = bytes.len();
        let node_id = NodeId::from_index(base);
        if matches!(self.cold_node(node_id), Some(Node::BytesView(_))) {
            let current = self.bytes(node_id)?;
            let available = current.len().saturating_sub(offset);
            if offset > current.len() || available < write_len {
                return Err(trace_invalid_bytes!(
                    self,
                    "write bytes view out of range ptr={ptr} base={base} offset={offset} write_len={write_len} len={}",
                    current.len()
                ));
            }
            let mut owned = current.to_vec();
            owned[offset..offset + write_len].copy_from_slice(bytes);
            self.set_node_at(node_id.index(), Node::bytes(owned));
            return Ok(());
        }
        let kind = self.profile_head_key(NodeId::from_index(base));
        let is_mutable_bytes = match self.cold_node(node_id) {
            Some(Node::Bytes(dst)) => {
                let available = dst.len().saturating_sub(offset);
                if offset > dst.len() || available < write_len {
                    return Err(trace_invalid_bytes!(
                        self,
                        "write bytes out of range ptr={ptr} base={base} offset={offset} write_len={write_len} len={}",
                        dst.len()
                    ));
                }
                false
            }
            Some(Node::MutableBytes(dst)) => {
                let storage_len = dst.bytes.len();
                let available = storage_len.saturating_sub(offset);
                if offset > storage_len || available < write_len {
                    return Err(trace_invalid_bytes!(
                        self,
                        "write mutable bytes out of range ptr={ptr} base={base} offset={offset} write_len={write_len} storage_len={storage_len}"
                    ));
                }
                true
            }
            _ => {
                return Err(trace_invalid_bytes!(
                    self,
                    "write pointer base not bytes ptr={ptr} base={base} offset={offset} kind={}",
                    kind
                ));
            }
        };
        let dst = match self.cold_node_mut(node_id) {
            Some(Node::Bytes(dst)) if !is_mutable_bytes => &mut dst[offset..offset + write_len],
            Some(Node::MutableBytes(dst)) if is_mutable_bytes => {
                &mut dst.bytes[offset..offset + write_len]
            }
            _ => unreachable!("validated pointer target changed during write"),
        };
        dst.copy_from_slice(bytes);
        Ok(())
    }

    fn peek_array<const N: usize>(&self, ptr: i64) -> Result<[u8; N], EvalError> {
        self.read_pointer_bytes(ptr, N)?
            .try_into()
            .map_err(|_| EvalError::InvalidByteString)
    }

    fn peek_unsigned(&self, ptr: i64, size: usize) -> Result<u64, EvalError> {
        if size == 0 || size > 8 {
            return Err(EvalError::Overflow);
        }
        let bytes = self.read_pointer_bytes(ptr, size)?;
        let mut wide = [0; 8];
        if cfg!(target_endian = "little") {
            wide[..size].copy_from_slice(&bytes);
        } else {
            wide[8 - size..].copy_from_slice(&bytes);
        }
        Ok(u64::from_ne_bytes(wide))
    }

    fn poke_unsigned(&mut self, ptr: i64, size: usize, value: u64) -> Result<(), EvalError> {
        if size == 0 || size > 8 {
            return Err(EvalError::Overflow);
        }
        let wide = value.to_ne_bytes();
        let bytes = if cfg!(target_endian = "little") {
            &wide[..size]
        } else {
            &wide[8 - size..]
        };
        self.write_pointer_bytes(ptr, bytes)
    }

    fn peek_signed(&self, ptr: i64, size: usize) -> Result<i64, EvalError> {
        let unsigned = self.peek_unsigned(ptr, size)?;
        let shift = (8 - size) * 8;
        Ok(((unsigned << shift) as i64) >> shift)
    }

    fn poke_signed(&mut self, ptr: i64, size: usize, value: i64) -> Result<(), EvalError> {
        if size == 0 || size > 8 {
            return Err(EvalError::Overflow);
        }
        let wide = value.to_ne_bytes();
        let bytes = if cfg!(target_endian = "little") {
            &wide[..size]
        } else {
            &wide[8 - size..]
        };
        self.write_pointer_bytes(ptr, bytes)
    }

    fn peek_c_char(&self, ptr: i64) -> Result<i64, EvalError> {
        if std::os::raw::c_char::MIN < 0 {
            self.peek_signed(ptr, size_of::<std::os::raw::c_char>())
        } else {
            Ok(self.peek_unsigned(ptr, size_of::<std::os::raw::c_char>())? as i64)
        }
    }

    fn poke_c_char(&mut self, ptr: i64, value: i64) -> Result<(), EvalError> {
        if std::os::raw::c_char::MIN < 0 {
            self.poke_signed(ptr, size_of::<std::os::raw::c_char>(), value)
        } else {
            self.poke_unsigned(ptr, size_of::<std::os::raw::c_char>(), value as u64)
        }
    }

    fn read_pointer_bytes(&self, ptr: i64, len: usize) -> Result<Vec<u8>, EvalError> {
        let bytes = self.pointer_bytes(ptr)?;
        let bytes = bytes.get(..len).ok_or_else(|| {
            trace_invalid_bytes!(
                self,
                "read pointer too short ptr={ptr} len={len} available={}",
                bytes.len()
            )
        })?;
        Ok(bytes.to_vec())
    }

    fn read_c_string(&self, ptr: i64) -> Result<Vec<u8>, EvalError> {
        let bytes = self.pointer_bytes(ptr)?;
        let len = c_string_len(bytes);
        Ok(bytes[..len].to_vec())
    }

    fn c_string_len(&self, ptr: i64) -> Result<usize, EvalError> {
        Ok(c_string_len(self.pointer_bytes(ptr)?))
    }

    fn bfile(&self, ptr: i64) -> Result<&BFile, EvalError> {
        let slot = self.decode_bfile_pointer(ptr)?;
        self.bfiles
            .get(slot)
            .and_then(Option::as_ref)
            .ok_or(EvalError::InvalidHandle)
    }

    fn bfile_mut(&mut self, ptr: i64) -> Result<&mut BFile, EvalError> {
        let slot = self.decode_bfile_pointer(ptr)?;
        self.bfiles
            .get_mut(slot)
            .and_then(Option::as_mut)
            .ok_or(EvalError::InvalidHandle)
    }

    fn close_bfile(&mut self, ptr: i64) -> Result<(), EvalError> {
        if let Some(handle) = handle_from_ptr(ptr) {
            return self.flush_io_handle(handle);
        }
        let slot = self.decode_bfile_pointer(ptr)?;
        let flush_self = {
            let bfile = self
                .bfiles
                .get(slot)
                .and_then(Option::as_ref)
                .ok_or(EvalError::InvalidHandle)?;
            matches!(
                &bfile.kind,
                BFileKind::Buf { read: false, .. }
                    | BFileKind::Rle { read: false, .. }
                    | BFileKind::Base64 { read: false, .. }
                    | BFileKind::Lz77 { read: false, .. }
                    | BFileKind::Bwt { read: false, .. }
                    | BFileKind::Lzma { read: false, .. }
            )
        };
        if flush_self {
            self.flush_bfile(ptr)?;
        }
        let close_extra = {
            let bfile = self
                .bfiles
                .get_mut(slot)
                .and_then(Option::as_mut)
                .ok_or(EvalError::InvalidHandle)?;
            match &mut bfile.kind {
                BFileKind::Base64 {
                    inner,
                    read,
                    linelen,
                    outcol,
                    ..
                } if !*read && *linelen != 0 && *outcol != 0 => {
                    *outcol = 0;
                    Some((*inner, vec![b'\n']))
                }
                _ => None,
            }
        };
        if let Some((inner, bytes)) = close_extra {
            let written = self.write_bfile_bytes(inner, &bytes)?;
            if written != bytes.len() {
                return Err(EvalError::InvalidHandle);
            }
        }
        let close_inner = {
            let bfile = self
                .bfiles
                .get(slot)
                .and_then(Option::as_ref)
                .ok_or(EvalError::InvalidHandle)?;
            match &bfile.kind {
                BFileKind::Utf8 { inner, .. }
                | BFileKind::Crlf { inner }
                | BFileKind::Rle { inner, .. }
                | BFileKind::Base64 { inner, .. }
                | BFileKind::Lz77 { inner, .. }
                | BFileKind::Bwt { inner, .. }
                | BFileKind::Lzma { inner, .. }
                | BFileKind::Buf { inner, .. } => Some(*inner),
                _ => None,
            }
        };
        if let Some(inner) = close_inner {
            self.close_bfile(inner)?;
        }
        let slot = self.bfiles.get_mut(slot).ok_or(EvalError::InvalidHandle)?;
        let _bfile = slot.as_ref().ok_or(EvalError::InvalidHandle)?;
        #[cfg(not(target_arch = "wasm32"))]
        if let BFileKind::NativeFile { file, .. } = &_bfile.kind {
            use std::io::Write as _;

            if _bfile.writable {
                file.borrow_mut()
                    .flush()
                    .map_err(|_| EvalError::InvalidHandle)?;
            }
        }
        *slot = None;
        Ok(())
    }

    fn read_dir_entry(&mut self, ptr: i64) -> Result<i64, EvalError> {
        let slot = self.decode_dir_pointer(ptr)?;
        let name = {
            let dir = self
                .dirs
                .get_mut(slot)
                .and_then(Option::as_mut)
                .ok_or(EvalError::InvalidHandle)?;
            let Some(name) = dir.entries.get(dir.pos) else {
                return Ok(0);
            };
            dir.pos += 1;
            name.clone()
        };
        let mut bytes = name;
        bytes.push(0);
        let ptr = self.alloc_memory(bytes.len())?;
        self.write_pointer_bytes(ptr, &bytes)?;
        Ok(ptr)
    }

    fn close_dir(&mut self, ptr: i64) -> Result<(), EvalError> {
        let slot = self.decode_dir_pointer(ptr)?;
        let slot = self.dirs.get_mut(slot).ok_or(EvalError::InvalidHandle)?;
        if slot.is_none() {
            return Err(EvalError::InvalidHandle);
        }
        *slot = None;
        Ok(())
    }

    fn flush_bfile(&mut self, ptr: i64) -> Result<(), EvalError> {
        if let Some(handle) = handle_from_ptr(ptr) {
            return self.flush_io_handle(handle);
        }
        let flush_inner = {
            let bfile = self.bfile_mut(ptr)?;
            match &mut bfile.kind {
                BFileKind::Utf8 { inner, .. } => Some((*inner, Vec::new())),
                BFileKind::Crlf { inner } => Some((*inner, Vec::new())),
                BFileKind::Rle {
                    inner,
                    read,
                    count,
                    byte,
                    ..
                } => {
                    if *read {
                        None
                    } else {
                        let bytes = rle_pending_bytes(*count, *byte)?;
                        *count = 0;
                        Some((*inner, bytes))
                    }
                }
                BFileKind::Base64 {
                    inner,
                    read,
                    encbuf,
                    encpos,
                    linelen,
                    outcol,
                    ..
                } => {
                    if *read {
                        None
                    } else {
                        let bytes = base64_pending_bytes(encbuf, encpos, *linelen, outcol)?;
                        *encpos = 0;
                        Some((*inner, bytes))
                    }
                }
                BFileKind::Lz77 {
                    inner,
                    read,
                    buffer,
                    pos,
                    numflush,
                } => {
                    if *read {
                        None
                    } else if *numflush > 0 && *pos == 0 {
                        *numflush = numflush.checked_add(1).ok_or(EvalError::Overflow)?;
                        None
                    } else {
                        *numflush = numflush.checked_add(1).ok_or(EvalError::Overflow)?;
                        let compressed = lz77_compress(&buffer[..*pos])?;
                        let mut bytes = Vec::with_capacity(7 + compressed.len());
                        bytes.extend_from_slice(b"LZ1");
                        bytes.extend_from_slice(&u32_le_bytes(compressed.len())?);
                        bytes.extend_from_slice(&compressed);
                        buffer.clear();
                        *pos = 0;
                        Some((*inner, bytes))
                    }
                }
                BFileKind::Bwt {
                    inner,
                    read,
                    buffer,
                    pos,
                    numflush,
                } => {
                    if *read {
                        None
                    } else if *numflush > 0 && *pos == 0 {
                        *numflush = numflush.checked_add(1).ok_or(EvalError::Overflow)?;
                        None
                    } else {
                        *numflush = numflush.checked_add(1).ok_or(EvalError::Overflow)?;
                        let (zero, last) = bwt_encode(&buffer[..*pos])?;
                        let mut bytes = Vec::with_capacity(11 + last.len());
                        bytes.extend_from_slice(b"BW1");
                        bytes.extend_from_slice(&u32_le_bytes(*pos)?);
                        bytes.extend_from_slice(&u32_le_bytes(zero)?);
                        bytes.extend_from_slice(&last);
                        buffer.clear();
                        *pos = 0;
                        Some((*inner, bytes))
                    }
                }
                BFileKind::Lzma {
                    inner,
                    read,
                    buffer,
                    pos,
                    numflush,
                } => {
                    if *read {
                        None
                    } else if *numflush > 0 && *pos == 0 {
                        *numflush = numflush.checked_add(1).ok_or(EvalError::Overflow)?;
                        None
                    } else {
                        *numflush = numflush.checked_add(1).ok_or(EvalError::Overflow)?;
                        let compressed = lzma_compress_payload(&buffer[..*pos])?;
                        let mut bytes = Vec::with_capacity(7 + compressed.len());
                        bytes.extend_from_slice(b"LZ2");
                        bytes.extend_from_slice(&u32_le_bytes(compressed.len())?);
                        bytes.extend_from_slice(&compressed);
                        buffer.clear();
                        *pos = 0;
                        Some((*inner, bytes))
                    }
                }
                BFileKind::Buf {
                    inner,
                    buffer,
                    pos,
                    read,
                    ..
                } => {
                    let bytes = if !*read && *pos > 0 {
                        let bytes = buffer[..*pos].to_vec();
                        *pos = 0;
                        bytes
                    } else {
                        Vec::new()
                    };
                    Some((*inner, bytes))
                }
                _ => None,
            }
        };
        if let Some((inner, bytes)) = flush_inner {
            if !bytes.is_empty() {
                let written = self.write_bfile_bytes(inner, &bytes)?;
                if written != bytes.len() {
                    return Err(EvalError::InvalidHandle);
                }
            }
            return self.flush_bfile(inner);
        }
        let _bfile = self.bfile(ptr)?;
        #[cfg(not(target_arch = "wasm32"))]
        if let BFileKind::NativeFile { file, .. } = &_bfile.kind {
            use std::io::Write as _;

            if _bfile.writable {
                file.borrow_mut()
                    .flush()
                    .map_err(|_| EvalError::InvalidHandle)?;
            }
        }
        Ok(())
    }

    fn get_bfile_byte(&mut self, ptr: i64) -> Result<i64, EvalError> {
        if handle_from_ptr(ptr) == Some(StdHandle::Stdin) {
            return self.read_stdin_byte();
        }
        enum SpecialBFileRead {
            Utf8(i64, i64),
            Crlf(i64),
            Rle,
            Base64,
            Buf,
        }
        let special = {
            let bfile = self.bfile_mut(ptr)?;
            if !bfile.readable {
                return Err(EvalError::InvalidHandle);
            }
            match &mut bfile.kind {
                BFileKind::Utf8 {
                    inner,
                    unget,
                    pending,
                    pending_pos,
                } => {
                    if let Some(byte) = unget.take() {
                        return Ok(byte);
                    }
                    if *pending_pos < pending.len() {
                        let byte = pending[*pending_pos];
                        *pending_pos += 1;
                        if *pending_pos == pending.len() {
                            pending.clear();
                            *pending_pos = 0;
                        }
                        return Ok(i64::from(byte));
                    }
                    pending.clear();
                    *pending_pos = 0;
                    Some(SpecialBFileRead::Utf8(ptr, *inner))
                }
                BFileKind::Crlf { inner } => Some(SpecialBFileRead::Crlf(*inner)),
                BFileKind::Rle { read, .. } if *read => Some(SpecialBFileRead::Rle),
                BFileKind::Base64 { read, .. } if *read => Some(SpecialBFileRead::Base64),
                BFileKind::Buf { .. } => Some(SpecialBFileRead::Buf),
                _ => None,
            }
        };
        match special {
            Some(SpecialBFileRead::Utf8(ptr, inner)) => {
                return self.get_utf8_bfile_byte(ptr, inner);
            }
            Some(SpecialBFileRead::Crlf(inner)) => return self.get_crlf_bfile_byte(inner),
            Some(SpecialBFileRead::Rle) => return self.get_rle_bfile_byte(ptr),
            Some(SpecialBFileRead::Base64) => return self.get_base64_bfile_byte(ptr),
            Some(SpecialBFileRead::Buf) => return self.get_buf_bfile_byte(ptr),
            None => {}
        }
        let bfile = self.bfile_mut(ptr)?;
        match &mut bfile.kind {
            BFileKind::Memory { bytes, pos } => {
                if *pos >= bytes.len() {
                    return Ok(-1);
                }
                let byte = bytes[*pos];
                *pos += 1;
                Ok(i64::from(byte))
            }
            #[cfg(not(target_arch = "wasm32"))]
            BFileKind::NativeFile { file, ungot } => {
                if let Some(byte) = ungot.pop() {
                    return Ok(i64::from(byte));
                }
                use std::io::Read as _;

                let mut byte = [0];
                match file.borrow_mut().read(&mut byte) {
                    Ok(0) => Ok(-1),
                    Ok(_) => Ok(i64::from(byte[0])),
                    Err(_) => Err(EvalError::InvalidHandle),
                }
            }
            BFileKind::Utf8 { .. } => unreachable!("handled above"),
            BFileKind::Crlf { .. } => unreachable!("handled above"),
            BFileKind::Rle { .. } => unreachable!("handled above"),
            BFileKind::Base64 { .. } => unreachable!("handled above"),
            BFileKind::Lz77 {
                read, buffer, pos, ..
            } => {
                if !*read {
                    return Err(EvalError::InvalidHandle);
                }
                if *pos >= buffer.len() {
                    return Ok(-1);
                }
                let byte = buffer[*pos];
                *pos += 1;
                Ok(i64::from(byte))
            }
            BFileKind::Bwt {
                read, buffer, pos, ..
            } => {
                if !*read {
                    return Err(EvalError::InvalidHandle);
                }
                if *pos >= buffer.len() {
                    return Ok(-1);
                }
                let byte = buffer[*pos];
                *pos += 1;
                Ok(i64::from(byte))
            }
            BFileKind::Lzma {
                read, buffer, pos, ..
            } => {
                if !*read {
                    return Err(EvalError::InvalidHandle);
                }
                if *pos >= buffer.len() {
                    return Ok(-1);
                }
                let byte = buffer[*pos];
                *pos += 1;
                Ok(i64::from(byte))
            }
            BFileKind::Buf { .. } => unreachable!("handled above"),
        }
    }

    fn get_crlf_bfile_byte(&mut self, inner: i64) -> Result<i64, EvalError> {
        let byte = self.get_bfile_byte(inner)?;
        if byte != i64::from(b'\r') {
            return Ok(byte);
        }
        let next = self.get_bfile_byte(inner)?;
        if next == i64::from(b'\n') {
            return Ok(next);
        }
        if next >= 0 {
            self.unget_bfile_byte(inner, next)?;
        }
        Ok(byte)
    }

    fn get_rle_bfile_byte(&mut self, ptr: i64) -> Result<i64, EvalError> {
        let inner = {
            let bfile = self.bfile_mut(ptr)?;
            if !bfile.readable {
                return Err(EvalError::InvalidHandle);
            }
            match &mut bfile.kind {
                BFileKind::Rle {
                    inner,
                    read,
                    count,
                    byte,
                    unget,
                } => {
                    if !*read {
                        return Err(EvalError::InvalidHandle);
                    }
                    if let Some(byte) = unget.take() {
                        return Ok(byte);
                    }
                    if *count > 0 {
                        *count -= 1;
                        return Ok(*byte);
                    }
                    *inner
                }
                _ => return Err(EvalError::InvalidHandle),
            }
        };

        let Some(rep) = self.get_rle_rep(inner)? else {
            return Ok(-1);
        };
        if rep == 1 {
            let byte = self.get_bfile_byte(inner)?;
            if byte < 0 {
                return Ok(-1);
            }
            return Ok(byte | 0x80);
        }

        let byte = self.get_bfile_byte(inner)?;
        if byte < 0 {
            return Ok(-1);
        }
        let bfile = self.bfile_mut(ptr)?;
        match &mut bfile.kind {
            BFileKind::Rle {
                count, byte: out, ..
            } => {
                *count = rep;
                *out = byte;
                Ok(byte)
            }
            _ => Err(EvalError::InvalidHandle),
        }
    }

    fn get_rle_rep(&mut self, inner: i64) -> Result<Option<usize>, EvalError> {
        let mut n = 0usize;
        loop {
            let byte = self.get_bfile_byte(inner)?;
            if byte < 0 {
                return Ok(None);
            }
            if byte < 128 {
                self.unget_bfile_byte(inner, byte)?;
                return Ok(Some(n));
            }
            let digit = usize::try_from(byte - 128).map_err(|_| EvalError::Overflow)?;
            n = n
                .checked_mul(128)
                .and_then(|n| n.checked_add(digit))
                .ok_or(EvalError::Overflow)?;
        }
    }

    fn get_base64_bfile_byte(&mut self, ptr: i64) -> Result<i64, EvalError> {
        let inner = {
            let bfile = self.bfile_mut(ptr)?;
            if !bfile.readable {
                return Err(EvalError::InvalidHandle);
            }
            match &mut bfile.kind {
                BFileKind::Base64 {
                    inner,
                    read,
                    unget,
                    outbuf,
                    outpos,
                    outlen,
                    ..
                } => {
                    if !*read {
                        return Err(EvalError::InvalidHandle);
                    }
                    if let Some(byte) = unget.take() {
                        return Ok(byte);
                    }
                    if *outpos < *outlen {
                        let byte = outbuf[*outpos];
                        *outpos += 1;
                        return Ok(i64::from(byte));
                    }
                    *inner
                }
                _ => return Err(EvalError::InvalidHandle),
            }
        };

        let Some(v) = self.get_base64_quartet(inner)? else {
            return Ok(-1);
        };
        if v[0] < 0 || v[1] < 0 {
            return Err(EvalError::InvalidByteString);
        }
        let mut outbuf = [0; 3];
        let outlen;
        let mut triple = ((v[0] as u32) << 18) | ((v[1] as u32) << 12);
        if v[2] == -3 {
            outbuf[0] = ((triple >> 16) & 0xff) as u8;
            outlen = 1;
        } else {
            if v[2] < 0 {
                return Err(EvalError::InvalidByteString);
            }
            triple |= (v[2] as u32) << 6;
            if v[3] == -3 {
                outbuf[0] = ((triple >> 16) & 0xff) as u8;
                outbuf[1] = ((triple >> 8) & 0xff) as u8;
                outlen = 2;
            } else {
                if v[3] < 0 {
                    return Err(EvalError::InvalidByteString);
                }
                triple |= v[3] as u32;
                outbuf[0] = ((triple >> 16) & 0xff) as u8;
                outbuf[1] = ((triple >> 8) & 0xff) as u8;
                outbuf[2] = (triple & 0xff) as u8;
                outlen = 3;
            }
        }

        let bfile = self.bfile_mut(ptr)?;
        match &mut bfile.kind {
            BFileKind::Base64 {
                outbuf: buffer,
                outpos,
                outlen: len,
                ..
            } => {
                *buffer = outbuf;
                *outpos = 1;
                *len = outlen;
                Ok(i64::from(outbuf[0]))
            }
            _ => Err(EvalError::InvalidHandle),
        }
    }

    fn get_base64_quartet(&mut self, inner: i64) -> Result<Option<[i32; 4]>, EvalError> {
        let mut v = [0; 4];
        let mut got = 0;
        loop {
            let byte = self.get_bfile_byte(inner)?;
            if byte < 0 {
                if got == 0 {
                    return Ok(None);
                }
                return Err(EvalError::InvalidByteString);
            }
            match base64_decode_value(byte as u8) {
                Base64Input::Whitespace => continue,
                Base64Input::Invalid => return Err(EvalError::InvalidByteString),
                Base64Input::Value(value) => {
                    v[got] = value;
                    got += 1;
                    if got == 4 {
                        return Ok(Some(v));
                    }
                }
            }
        }
    }

    fn get_buf_bfile_byte(&mut self, ptr: i64) -> Result<i64, EvalError> {
        loop {
            enum BufReadAction {
                Direct(i64),
                Refill { inner: i64, size: usize },
            }
            let action = {
                let bfile = self.bfile_mut(ptr)?;
                if !bfile.readable {
                    return Err(EvalError::InvalidHandle);
                }
                match &mut bfile.kind {
                    BFileKind::Buf {
                        inner,
                        unget,
                        buffer,
                        cur,
                        pos,
                        read,
                        ..
                    } => {
                        *read = true;
                        if let Some(byte) = unget.take() {
                            return Ok(byte);
                        }
                        if buffer.is_empty() {
                            BufReadAction::Direct(*inner)
                        } else if *pos >= *cur {
                            BufReadAction::Refill {
                                inner: *inner,
                                size: buffer.len(),
                            }
                        } else {
                            let byte = buffer[*pos];
                            *pos += 1;
                            return Ok(i64::from(byte));
                        }
                    }
                    _ => return Err(EvalError::InvalidHandle),
                }
            };
            match action {
                BufReadAction::Direct(inner) => return self.get_bfile_byte(inner),
                BufReadAction::Refill { inner, size } => {
                    let bytes = self.read_bfile_bytes(inner, size)?;
                    if bytes.is_empty() {
                        return Ok(-1);
                    }
                    let bfile = self.bfile_mut(ptr)?;
                    match &mut bfile.kind {
                        BFileKind::Buf {
                            buffer, cur, pos, ..
                        } => {
                            buffer[..bytes.len()].copy_from_slice(&bytes);
                            *cur = bytes.len();
                            *pos = 0;
                        }
                        _ => return Err(EvalError::InvalidHandle),
                    }
                }
            }
        }
    }

    fn unget_bfile_byte(&mut self, ptr: i64, byte: i64) -> Result<(), EvalError> {
        let crlf_inner = {
            let bfile = self.bfile_mut(ptr)?;
            if !bfile.readable {
                return Err(EvalError::InvalidHandle);
            }
            match &bfile.kind {
                BFileKind::Crlf { inner } => Some(*inner),
                _ => None,
            }
        };
        if let Some(inner) = crlf_inner {
            return self.unget_bfile_byte(inner, byte);
        }
        let bfile = self.bfile_mut(ptr)?;
        match &mut bfile.kind {
            BFileKind::Memory { bytes, pos } => {
                if *pos == 0 {
                    return Err(EvalError::InvalidHandle);
                }
                let byte = byte as u8;
                if bytes[*pos - 1] != byte {
                    return Err(EvalError::InvalidHandle);
                }
                *pos -= 1;
                Ok(())
            }
            #[cfg(not(target_arch = "wasm32"))]
            BFileKind::NativeFile { ungot, .. } => {
                ungot.push(byte as u8);
                Ok(())
            }
            BFileKind::Utf8 { unget, .. } => {
                if unget.is_some() {
                    return Err(EvalError::InvalidHandle);
                }
                *unget = Some(byte);
                Ok(())
            }
            BFileKind::Crlf { .. } => unreachable!("handled above"),
            BFileKind::Rle { unget, .. } => {
                if unget.is_some() {
                    return Err(EvalError::InvalidHandle);
                }
                *unget = Some(byte);
                Ok(())
            }
            BFileKind::Base64 { unget, .. } => {
                if unget.is_some() {
                    return Err(EvalError::InvalidHandle);
                }
                *unget = Some(byte);
                Ok(())
            }
            BFileKind::Lz77 { read, pos, .. } => {
                if !*read || *pos == 0 {
                    return Err(EvalError::InvalidHandle);
                }
                *pos -= 1;
                Ok(())
            }
            BFileKind::Bwt { read, pos, .. } => {
                if !*read || *pos == 0 {
                    return Err(EvalError::InvalidHandle);
                }
                *pos -= 1;
                Ok(())
            }
            BFileKind::Lzma { read, pos, .. } => {
                if !*read || *pos == 0 {
                    return Err(EvalError::InvalidHandle);
                }
                *pos -= 1;
                Ok(())
            }
            BFileKind::Buf { unget, .. } => {
                if unget.is_some() {
                    return Err(EvalError::InvalidHandle);
                }
                *unget = Some(byte);
                Ok(())
            }
        }
    }

    fn put_bfile_byte(&mut self, ptr: i64, byte: i64) -> Result<(), EvalError> {
        if let Some(handle) = handle_from_ptr(ptr) {
            return self.write_io_handle_bytes(handle, &[byte as u8]);
        }
        enum SpecialBFileWrite {
            Utf8(i64),
            Crlf(i64),
            Rle,
            Base64,
            Buf,
        }
        let special = {
            let bfile = self.bfile_mut(ptr)?;
            if !bfile.writable {
                return Err(EvalError::InvalidHandle);
            }
            match &bfile.kind {
                BFileKind::Utf8 { inner, .. } => Some(SpecialBFileWrite::Utf8(*inner)),
                BFileKind::Crlf { inner } => Some(SpecialBFileWrite::Crlf(*inner)),
                BFileKind::Rle { read, .. } if !*read => Some(SpecialBFileWrite::Rle),
                BFileKind::Base64 { read, .. } if !*read => Some(SpecialBFileWrite::Base64),
                BFileKind::Buf { .. } => Some(SpecialBFileWrite::Buf),
                _ => None,
            }
        };
        match special {
            Some(SpecialBFileWrite::Utf8(inner)) => return self.put_utf8_bfile_byte(inner, byte),
            Some(SpecialBFileWrite::Crlf(inner)) => return self.put_crlf_bfile_byte(inner, byte),
            Some(SpecialBFileWrite::Rle) => return self.put_rle_bfile_byte(ptr, byte),
            Some(SpecialBFileWrite::Base64) => return self.put_base64_bfile_byte(ptr, byte),
            Some(SpecialBFileWrite::Buf) => return self.put_buf_bfile_byte(ptr, byte),
            None => {}
        }
        let bfile = self.bfile_mut(ptr)?;
        match &mut bfile.kind {
            BFileKind::Memory { bytes, pos } => {
                if *pos == bytes.len() {
                    bytes.push(byte as u8);
                } else if *pos < bytes.len() {
                    bytes[*pos] = byte as u8;
                } else {
                    return Err(EvalError::InvalidHandle);
                }
                *pos += 1;
                Ok(())
            }
            #[cfg(not(target_arch = "wasm32"))]
            BFileKind::NativeFile { file, .. } => {
                use std::io::Write as _;

                file.borrow_mut()
                    .write_all(&[byte as u8])
                    .map_err(|_| EvalError::InvalidHandle)
            }
            BFileKind::Utf8 { .. } => unreachable!("handled above"),
            BFileKind::Crlf { .. } => unreachable!("handled above"),
            BFileKind::Rle { .. } => unreachable!("handled above"),
            BFileKind::Base64 { .. } => unreachable!("handled above"),
            BFileKind::Lz77 {
                read, buffer, pos, ..
            } => {
                if *read {
                    return Err(EvalError::InvalidHandle);
                }
                if *pos == buffer.len() {
                    buffer.push(byte as u8);
                } else if *pos < buffer.len() {
                    buffer[*pos] = byte as u8;
                } else {
                    return Err(EvalError::InvalidHandle);
                }
                *pos += 1;
                Ok(())
            }
            BFileKind::Bwt {
                read, buffer, pos, ..
            } => {
                if *read {
                    return Err(EvalError::InvalidHandle);
                }
                if *pos == buffer.len() {
                    buffer.push(byte as u8);
                } else if *pos < buffer.len() {
                    buffer[*pos] = byte as u8;
                } else {
                    return Err(EvalError::InvalidHandle);
                }
                *pos += 1;
                Ok(())
            }
            BFileKind::Lzma {
                read, buffer, pos, ..
            } => {
                if *read {
                    return Err(EvalError::InvalidHandle);
                }
                if *pos == buffer.len() {
                    buffer.push(byte as u8);
                } else if *pos < buffer.len() {
                    buffer[*pos] = byte as u8;
                } else {
                    return Err(EvalError::InvalidHandle);
                }
                *pos += 1;
                Ok(())
            }
            BFileKind::Buf { .. } => unreachable!("handled above"),
        }
    }

    fn put_crlf_bfile_byte(&mut self, inner: i64, byte: i64) -> Result<(), EvalError> {
        if byte == i64::from(b'\n') {
            self.put_bfile_byte(inner, i64::from(b'\r'))?;
        }
        self.put_bfile_byte(inner, byte)
    }

    fn put_rle_bfile_byte(&mut self, ptr: i64, byte: i64) -> Result<(), EvalError> {
        if byte < 0 {
            return Err(EvalError::InvalidByteString);
        }
        let (inner, pending) = {
            let bfile = self.bfile_mut(ptr)?;
            if !bfile.writable {
                return Err(EvalError::InvalidHandle);
            }
            match &mut bfile.kind {
                BFileKind::Rle {
                    inner,
                    read,
                    count,
                    byte: current,
                    ..
                } => {
                    if *read {
                        return Err(EvalError::InvalidHandle);
                    }
                    if (byte & 0x80) != 0 {
                        let pending = rle_pending_bytes(*count, *current)?;
                        *count = 0;
                        *current = -1;
                        (*inner, Some((pending, vec![0x81, (byte as u8) & 0x7f])))
                    } else if byte == *current {
                        *count = count.checked_add(1).ok_or(EvalError::Overflow)?;
                        (*inner, None)
                    } else {
                        let pending = rle_pending_bytes(*count, *current)?;
                        *count = 1;
                        *current = byte;
                        (*inner, Some((pending, Vec::new())))
                    }
                }
                _ => return Err(EvalError::InvalidHandle),
            }
        };
        if let Some((pending, suffix)) = pending {
            let pending_written = self.write_bfile_bytes(inner, &pending)?;
            if pending_written != pending.len() {
                return Err(EvalError::InvalidHandle);
            }
            let suffix_written = self.write_bfile_bytes(inner, &suffix)?;
            if suffix_written != suffix.len() {
                return Err(EvalError::InvalidHandle);
            }
        }
        Ok(())
    }

    fn put_base64_bfile_byte(&mut self, ptr: i64, byte: i64) -> Result<(), EvalError> {
        if byte < 0 {
            return Err(EvalError::InvalidByteString);
        }
        let action = {
            let bfile = self.bfile_mut(ptr)?;
            if !bfile.writable {
                return Err(EvalError::InvalidHandle);
            }
            match &mut bfile.kind {
                BFileKind::Base64 {
                    inner,
                    read,
                    encbuf,
                    encpos,
                    linelen,
                    outcol,
                    ..
                } => {
                    if *read {
                        return Err(EvalError::InvalidHandle);
                    }
                    encbuf[*encpos] = byte as u8;
                    *encpos += 1;
                    if *encpos == 3 {
                        let bytes = base64_full_quad_bytes(encbuf, *linelen, outcol);
                        *encpos = 0;
                        Some((*inner, bytes))
                    } else {
                        None
                    }
                }
                _ => return Err(EvalError::InvalidHandle),
            }
        };
        if let Some((inner, bytes)) = action {
            let written = self.write_bfile_bytes(inner, &bytes)?;
            if written != bytes.len() {
                return Err(EvalError::InvalidHandle);
            }
        }
        Ok(())
    }

    fn put_buf_bfile_byte(&mut self, ptr: i64, byte: i64) -> Result<(), EvalError> {
        if byte < 0 {
            return Err(EvalError::InvalidByteString);
        }
        let byte = byte as u8;
        loop {
            enum BufWriteAction {
                Direct(i64),
                Flush { inner: i64, bytes: Vec<u8> },
                Done,
            }
            let action = {
                let bfile = self.bfile_mut(ptr)?;
                if !bfile.writable {
                    return Err(EvalError::InvalidHandle);
                }
                match &mut bfile.kind {
                    BFileKind::Buf {
                        inner,
                        buffer,
                        pos,
                        linebuf,
                        ..
                    } => {
                        if buffer.is_empty() {
                            BufWriteAction::Direct(*inner)
                        } else if *pos >= buffer.len() {
                            let bytes = buffer.clone();
                            *pos = 0;
                            BufWriteAction::Flush {
                                inner: *inner,
                                bytes,
                            }
                        } else {
                            buffer[*pos] = byte;
                            *pos += 1;
                            if *linebuf && byte == b'\n' {
                                let bytes = buffer[..*pos].to_vec();
                                *pos = 0;
                                BufWriteAction::Flush {
                                    inner: *inner,
                                    bytes,
                                }
                            } else {
                                BufWriteAction::Done
                            }
                        }
                    }
                    _ => return Err(EvalError::InvalidHandle),
                }
            };
            match action {
                BufWriteAction::Direct(inner) => {
                    return self.put_bfile_byte(inner, i64::from(byte));
                }
                BufWriteAction::Flush { inner, bytes } => {
                    let written = self.write_bfile_bytes(inner, &bytes)?;
                    if written != bytes.len() {
                        return Err(EvalError::InvalidHandle);
                    }
                }
                BufWriteAction::Done => return Ok(()),
            }
        }
    }

    fn read_bfile(&mut self, ptr: i64, dst: i64, len: usize) -> Result<usize, EvalError> {
        let bytes = self.read_bfile_bytes(ptr, len)?;
        self.write_pointer_bytes(dst, &bytes)?;
        Ok(bytes.len())
    }

    fn read_bfile_bytes(&mut self, ptr: i64, len: usize) -> Result<Vec<u8>, EvalError> {
        if let Some(handle) = handle_from_ptr(ptr) {
            if handle != StdHandle::Stdin {
                return Err(EvalError::InvalidHandle);
            }
            #[cfg(not(target_arch = "wasm32"))]
            {
                use std::io::Read as _;

                let mut bytes = vec![0; len];
                let read = std::io::stdin()
                    .lock()
                    .read(&mut bytes)
                    .map_err(|_| EvalError::InvalidHandle)?;
                bytes.truncate(read);
                return Ok(bytes);
            }
            #[cfg(target_arch = "wasm32")]
            {
                return Ok(Vec::new());
            }
        }
        let uses_getb_fallback = {
            let bfile = self.bfile(ptr)?;
            if !bfile.readable {
                return Err(EvalError::InvalidHandle);
            }
            matches!(
                &bfile.kind,
                BFileKind::Utf8 { .. }
                    | BFileKind::Crlf { .. }
                    | BFileKind::Rle { .. }
                    | BFileKind::Base64 { .. }
                    | BFileKind::Buf { .. }
            )
        };
        if uses_getb_fallback {
            let mut bytes = Vec::with_capacity(len);
            for _ in 0..len {
                let byte = self.get_bfile_byte(ptr)?;
                if byte < 0 {
                    break;
                }
                bytes.push(byte as u8);
            }
            return Ok(bytes);
        }
        let bfile = self.bfile_mut(ptr)?;
        match &mut bfile.kind {
            BFileKind::Memory { bytes, pos } => {
                let end = pos
                    .checked_add(len)
                    .map(|end| end.min(bytes.len()))
                    .ok_or(EvalError::Overflow)?;
                let out = bytes[*pos..end].to_vec();
                *pos = end;
                Ok(out)
            }
            #[cfg(not(target_arch = "wasm32"))]
            BFileKind::NativeFile { file, ungot } => {
                use std::io::Read as _;

                let mut bytes = vec![0; len];
                let mut read = 0;
                while read < len {
                    let Some(byte) = ungot.pop() else {
                        break;
                    };
                    bytes[read] = byte;
                    read += 1;
                }
                if read < len {
                    read += file
                        .borrow_mut()
                        .read(&mut bytes[read..])
                        .map_err(|_| EvalError::InvalidHandle)?;
                }
                bytes.truncate(read);
                Ok(bytes)
            }
            BFileKind::Utf8 { .. }
            | BFileKind::Crlf { .. }
            | BFileKind::Rle { .. }
            | BFileKind::Base64 { .. }
            | BFileKind::Lz77 { read: false, .. }
            | BFileKind::Bwt { read: false, .. }
            | BFileKind::Lzma { read: false, .. }
            | BFileKind::Buf { .. } => unreachable!("handled above"),
            BFileKind::Lz77 {
                read, buffer, pos, ..
            } => {
                if !*read {
                    return Err(EvalError::InvalidHandle);
                }
                let end = pos
                    .checked_add(len)
                    .map(|end| end.min(buffer.len()))
                    .ok_or(EvalError::Overflow)?;
                let out = buffer[*pos..end].to_vec();
                *pos = end;
                Ok(out)
            }
            BFileKind::Bwt {
                read, buffer, pos, ..
            } => {
                if !*read {
                    return Err(EvalError::InvalidHandle);
                }
                let end = pos
                    .checked_add(len)
                    .map(|end| end.min(buffer.len()))
                    .ok_or(EvalError::Overflow)?;
                let out = buffer[*pos..end].to_vec();
                *pos = end;
                Ok(out)
            }
            BFileKind::Lzma {
                read, buffer, pos, ..
            } => {
                if !*read {
                    return Err(EvalError::InvalidHandle);
                }
                let end = pos
                    .checked_add(len)
                    .map(|end| end.min(buffer.len()))
                    .ok_or(EvalError::Overflow)?;
                let out = buffer[*pos..end].to_vec();
                *pos = end;
                Ok(out)
            }
        }
    }

    fn write_bfile(&mut self, ptr: i64, src: i64, len: usize) -> Result<usize, EvalError> {
        let bytes = self.read_pointer_bytes(src, len)?;
        self.write_bfile_bytes(ptr, &bytes)
    }

    fn write_bfile_bytes(&mut self, ptr: i64, bytes: &[u8]) -> Result<usize, EvalError> {
        if let Some(handle) = handle_from_ptr(ptr) {
            self.write_io_handle_bytes(handle, bytes)?;
            return Ok(bytes.len());
        }
        let uses_putb_fallback = {
            let bfile = self.bfile(ptr)?;
            if !bfile.writable {
                return Err(EvalError::InvalidHandle);
            }
            matches!(
                &bfile.kind,
                BFileKind::Utf8 { .. }
                    | BFileKind::Crlf { .. }
                    | BFileKind::Rle { .. }
                    | BFileKind::Base64 { .. }
                    | BFileKind::Buf { .. }
            )
        };
        if uses_putb_fallback {
            for byte in bytes {
                self.put_bfile_byte(ptr, i64::from(*byte))?;
            }
            return Ok(bytes.len());
        }
        let bfile = self.bfile_mut(ptr)?;
        match &mut bfile.kind {
            BFileKind::Memory { bytes: buffer, pos } => {
                let end = pos.checked_add(bytes.len()).ok_or(EvalError::Overflow)?;
                if end > buffer.len() {
                    buffer.resize(end, 0);
                }
                buffer[*pos..end].copy_from_slice(&bytes);
                *pos = end;
            }
            #[cfg(not(target_arch = "wasm32"))]
            BFileKind::NativeFile { file, .. } => {
                use std::io::Write as _;

                file.borrow_mut()
                    .write_all(bytes)
                    .map_err(|_| EvalError::InvalidHandle)?;
            }
            BFileKind::Utf8 { .. }
            | BFileKind::Crlf { .. }
            | BFileKind::Rle { .. }
            | BFileKind::Base64 { .. }
            | BFileKind::Lz77 { read: true, .. }
            | BFileKind::Bwt { read: true, .. }
            | BFileKind::Lzma { read: true, .. }
            | BFileKind::Buf { .. } => unreachable!("handled above"),
            BFileKind::Lz77 {
                read, buffer, pos, ..
            } => {
                if *read {
                    return Err(EvalError::InvalidHandle);
                }
                let end = pos.checked_add(bytes.len()).ok_or(EvalError::Overflow)?;
                if end > buffer.len() {
                    buffer.resize(end, 0);
                }
                buffer[*pos..end].copy_from_slice(bytes);
                *pos = end;
            }
            BFileKind::Bwt {
                read, buffer, pos, ..
            } => {
                if *read {
                    return Err(EvalError::InvalidHandle);
                }
                let end = pos.checked_add(bytes.len()).ok_or(EvalError::Overflow)?;
                if end > buffer.len() {
                    buffer.resize(end, 0);
                }
                buffer[*pos..end].copy_from_slice(bytes);
                *pos = end;
            }
            BFileKind::Lzma {
                read, buffer, pos, ..
            } => {
                if *read {
                    return Err(EvalError::InvalidHandle);
                }
                let end = pos.checked_add(bytes.len()).ok_or(EvalError::Overflow)?;
                if end > buffer.len() {
                    buffer.resize(end, 0);
                }
                buffer[*pos..end].copy_from_slice(bytes);
                *pos = end;
            }
        }
        Ok(bytes.len())
    }

    fn bfile_output_bytes(&self, ptr: i64) -> Result<Vec<u8>, EvalError> {
        let bfile = self.bfile(ptr)?;
        if !bfile.writable {
            return Err(EvalError::InvalidHandle);
        }
        match &bfile.kind {
            BFileKind::Memory { bytes, pos } => Ok(bytes[..*pos].to_vec()),
            #[cfg(not(target_arch = "wasm32"))]
            BFileKind::NativeFile { .. } => Err(EvalError::InvalidHandle),
            BFileKind::Utf8 { .. } => Err(EvalError::InvalidHandle),
            BFileKind::Crlf { .. } => Err(EvalError::InvalidHandle),
            BFileKind::Rle { .. } => Err(EvalError::InvalidHandle),
            BFileKind::Base64 { .. } => Err(EvalError::InvalidHandle),
            BFileKind::Lz77 { .. } => Err(EvalError::InvalidHandle),
            BFileKind::Bwt { .. } => Err(EvalError::InvalidHandle),
            BFileKind::Lzma { .. } => Err(EvalError::InvalidHandle),
            BFileKind::Buf { .. } => Err(EvalError::InvalidHandle),
        }
    }

    #[cold]
    #[inline(never)]
    fn deserialize_bfile(&mut self, ptr: i64) -> Result<NodeId, EvalError> {
        let mut input = Vec::new();
        let mut last_error = None;
        loop {
            let byte = self.read_bfile_bytes(ptr, 1)?;
            if byte.is_empty() {
                return Err(last_error
                    .map(Self::deserialize_parse_error)
                    .unwrap_or(EvalError::InvalidByteString));
            }
            input.push(byte[0]);
            match crate::parse::parse_program(&input) {
                Ok(parsed) => return self.append_parsed_program(parsed),
                Err(err) => last_error = Some(err),
            }
        }
    }

    fn deserialize_parse_error(error: crate::parse::ParseError) -> EvalError {
        match error {
            crate::parse::ParseError::UnknownPrim(name) => EvalError::UnknownPrim(name),
            _ => EvalError::InvalidByteString,
        }
    }

    #[cold]
    #[inline(never)]
    fn append_parsed_program(&mut self, parsed: Program) -> Result<NodeId, EvalError> {
        let root = parsed.root();
        let nodes = parsed.nodes();
        let mut remap = Vec::with_capacity(nodes.len());
        for node in &nodes {
            let id = match node {
                Node::App(_, _)
                | Node::Indir(_)
                | Node::BytesView(_)
                | Node::MVar(Some(_))
                | Node::Weak(_)
                | Node::Array(_) => self.push_node(Node::Indir(None)),
                Node::Free(_) => return Err(EvalError::InvalidByteString),
                node => self.push_value_node(node.clone()),
            };
            remap.push(id);
        }

        for (index, node) in nodes.into_iter().enumerate() {
            let target = remap[index];
            let node = match node {
                Node::App(fun, arg) => Node::App(
                    Self::remap_parsed_id(&remap, fun)?,
                    Self::remap_parsed_id(&remap, arg)?,
                ),
                Node::Indir(target) => Node::Indir(
                    target
                        .map(|id| Self::remap_parsed_id(&remap, id))
                        .transpose()?,
                ),
                Node::BytesView(view) => Node::bytes_view(
                    Self::remap_parsed_id(&remap, view.base)?,
                    view.offset,
                    view.len,
                ),
                Node::MVar(Some(value)) => Node::MVar(Some(Self::remap_parsed_id(&remap, value)?)),
                Node::Weak(weak) => {
                    let weak = *weak;
                    Node::Weak(Box::new(WeakNode {
                        key: weak
                            .key
                            .map(|id| Self::remap_parsed_id(&remap, id))
                            .transpose()?,
                        value: weak
                            .value
                            .map(|id| Self::remap_parsed_id(&remap, id))
                            .transpose()?,
                        finalizer: weak
                            .finalizer
                            .map(|id| Self::remap_parsed_id(&remap, id))
                            .transpose()?,
                    }))
                }
                Node::Array(items) => Node::array(
                    items
                        .iter()
                        .map(|id| Self::remap_parsed_id(&remap, *id))
                        .collect::<Result<Vec<_>, _>>()?,
                ),
                Node::Free(_) => return Err(EvalError::InvalidByteString),
                _ => continue,
            };
            let is_weak = matches!(&node, Node::Weak(_));
            self.set_node_at(target.index(), node);
            if is_weak {
                self.weak_nodes.push(target);
            }
        }

        Self::remap_parsed_id(&remap, root)
    }

    fn remap_parsed_id(remap: &[NodeId], id: NodeId) -> Result<NodeId, EvalError> {
        remap
            .get(id.index())
            .copied()
            .ok_or(EvalError::InvalidByteString)
    }

    fn get_utf8_bfile_byte(&mut self, ptr: i64, inner: i64) -> Result<i64, EvalError> {
        let c1 = self.get_bfile_byte(inner)?;
        if c1 < 0 {
            return Ok(-1);
        }
        if (c1 & 0x80) == 0 {
            self.refill_utf8_ascii(ptr, inner)?;
            return Ok(c1);
        }
        let c2 = self.get_bfile_byte(inner)?;
        if c2 < 0 {
            return Ok(-1);
        }
        if (c1 & 0xe0) == 0xc0 {
            let c = ((c1 & 0x1f) << 6) | (c2 & 0x3f);
            if 0 < c && c < 0x80 {
                return Err(EvalError::InvalidByteString);
            }
            return Ok(c);
        }
        let c3 = self.get_bfile_byte(inner)?;
        if c3 < 0 {
            return Ok(-1);
        }
        if (c1 & 0xf0) == 0xe0 {
            let c = ((c1 & 0x0f) << 12) | ((c2 & 0x3f) << 6) | (c3 & 0x3f);
            if c < 0x800 {
                return Err(EvalError::InvalidByteString);
            }
            return Ok(c);
        }
        let c4 = self.get_bfile_byte(inner)?;
        if c4 < 0 {
            return Ok(-1);
        }
        if (c1 & 0xf8) == 0xf0 {
            let c = ((c1 & 0x07) << 18) | ((c2 & 0x3f) << 12) | ((c3 & 0x3f) << 6) | (c4 & 0x3f);
            if c < 0x10000 {
                return Err(EvalError::InvalidByteString);
            }
            return Ok(c);
        }
        Err(EvalError::InvalidByteString)
    }

    fn refill_utf8_ascii(&mut self, ptr: i64, inner: i64) -> Result<(), EvalError> {
        if !self.can_refill_utf8_ascii(ptr, inner)? {
            return Ok(());
        }
        let bytes = self.read_bfile_bytes(inner, UTF8_ASCII_REFILL)?;
        if bytes.is_empty() {
            return Ok(());
        }
        let ascii_len = bytes
            .iter()
            .position(|byte| (byte & 0x80) != 0)
            .unwrap_or(bytes.len());
        for byte in bytes[ascii_len..].iter().rev() {
            self.unget_bfile_byte(inner, i64::from(*byte))?;
        }
        if ascii_len == 0 {
            return Ok(());
        }
        let bfile = self.bfile_mut(ptr)?;
        match &mut bfile.kind {
            BFileKind::Utf8 {
                pending,
                pending_pos,
                ..
            } => {
                pending.clear();
                pending.extend_from_slice(&bytes[..ascii_len]);
                *pending_pos = 0;
                Ok(())
            }
            _ => Err(EvalError::InvalidHandle),
        }
    }

    fn can_refill_utf8_ascii(&self, ptr: i64, inner: i64) -> Result<bool, EvalError> {
        let outer = self.bfile(ptr)?;
        let BFileKind::Utf8 {
            unget,
            pending,
            pending_pos,
            ..
        } = &outer.kind
        else {
            return Err(EvalError::InvalidHandle);
        };
        if unget.is_some() || *pending_pos < pending.len() {
            return Ok(false);
        }
        if handle_from_ptr(inner).is_some() {
            return Ok(false);
        }

        let inner = self.bfile(inner)?;
        match &inner.kind {
            BFileKind::Memory { .. } => Ok(true),
            #[cfg(not(target_arch = "wasm32"))]
            BFileKind::NativeFile { .. } => Ok(true),
            _ => Ok(false),
        }
    }

    fn put_utf8_bfile_byte(&mut self, inner: i64, byte: i64) -> Result<(), EvalError> {
        if byte < 0 {
            return Err(EvalError::InvalidByteString);
        }
        if 0 < byte && byte < 0x80 {
            self.put_bfile_byte(inner, byte)?;
        } else if byte < 0x800 {
            self.put_bfile_byte(inner, (byte >> 6) | 0xc0)?;
            self.put_bfile_byte(inner, (byte & 0x3f) | 0x80)?;
        } else if byte < 0x10000 {
            self.put_bfile_byte(inner, (byte >> 12) | 0xe0)?;
            self.put_bfile_byte(inner, ((byte >> 6) & 0x3f) | 0x80)?;
            self.put_bfile_byte(inner, (byte & 0x3f) | 0x80)?;
        } else if byte < 0x110000 {
            self.put_bfile_byte(inner, (byte >> 18) | 0xf0)?;
            self.put_bfile_byte(inner, ((byte >> 12) & 0x3f) | 0x80)?;
            self.put_bfile_byte(inner, ((byte >> 6) & 0x3f) | 0x80)?;
            self.put_bfile_byte(inner, (byte & 0x3f) | 0x80)?;
        } else {
            return Err(EvalError::InvalidByteString);
        }
        Ok(())
    }

    fn write_io_handle_bytes(&self, handle: StdHandle, bytes: &[u8]) -> Result<(), EvalError> {
        if handle == StdHandle::Stdin {
            return Err(EvalError::InvalidHandle);
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            use std::io::Write as _;

            match handle {
                StdHandle::Stdout => {
                    let mut stdout = std::io::stdout().lock();
                    stdout
                        .write_all(bytes)
                        .map_err(|_| EvalError::InvalidHandle)?;
                }
                StdHandle::Stderr => {
                    let mut stderr = std::io::stderr().lock();
                    stderr
                        .write_all(bytes)
                        .map_err(|_| EvalError::InvalidHandle)?;
                }
                StdHandle::Stdin => unreachable!("checked above"),
            }
        }
        #[cfg(target_arch = "wasm32")]
        {
            let _ = bytes;
        }
        Ok(())
    }

    fn flush_io_handle(&self, handle: StdHandle) -> Result<(), EvalError> {
        if handle == StdHandle::Stdin {
            return Ok(());
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            use std::io::Write as _;

            match handle {
                StdHandle::Stdout => std::io::stdout()
                    .lock()
                    .flush()
                    .map_err(|_| EvalError::InvalidHandle)?,
                StdHandle::Stderr => std::io::stderr()
                    .lock()
                    .flush()
                    .map_err(|_| EvalError::InvalidHandle)?,
                StdHandle::Stdin => unreachable!("checked above"),
            }
        }
        Ok(())
    }

    fn read_stdin_byte(&self) -> Result<i64, EvalError> {
        #[cfg(not(target_arch = "wasm32"))]
        {
            use std::io::Read as _;

            let mut byte = [0];
            match std::io::stdin().lock().read(&mut byte) {
                Ok(0) => Ok(-1),
                Ok(_) => Ok(i64::from(byte[0])),
                Err(_) => Err(EvalError::InvalidHandle),
            }
        }
        #[cfg(target_arch = "wasm32")]
        {
            Ok(-1)
        }
    }

    #[cold]
    #[inline(never)]
    pub fn serialize_program(&self, root: NodeId) -> Result<Vec<u8>, EvalError> {
        let mut labels = self.find_serialization_labels(root)?;
        let mut out = b"v8.4\n".to_vec();
        push_display(&mut out, labels.shared.len());
        out.push(b'\n');
        self.serialize_comb_into(root, &mut labels, &mut out)?;
        out.extend_from_slice(b"}\n");
        Ok(out)
    }

    #[cold]
    #[inline(never)]
    fn print_program(&self, root: NodeId) -> Result<Vec<u8>, EvalError> {
        let mut labels = self.find_serialization_labels(root)?;
        let mut out = Vec::new();
        self.print_comb_into(root, &mut labels, &mut out)?;
        out.push(b'\n');
        Ok(out)
    }

    fn find_serialization_labels(&self, root: NodeId) -> Result<SerializationLabels, EvalError> {
        let mut labels = SerializationLabels::default();
        let mut marked = HashSet::new();
        let mut work = vec![root];
        while let Some(id) = work.pop() {
            let id = self.resolve(id)?;
            let node = self.node_for_debug(id);
            if !serialization_shareable_node(&node) {
                continue;
            }
            if !marked.insert(id) {
                labels.shared.insert(id);
                continue;
            }
            match node {
                Node::App(fun, arg) => {
                    work.push(arg);
                    work.push(fun);
                }
                Node::Array(items) => {
                    work.extend(items.iter().copied());
                }
                _ => {}
            }
        }
        Ok(labels)
    }

    #[cold]
    #[inline(never)]
    fn print_comb_into(
        &self,
        root: NodeId,
        labels: &mut SerializationLabels,
        out: &mut Vec<u8>,
    ) -> Result<(), EvalError> {
        enum PrintTask {
            Node(NodeId),
            Byte(u8),
        }

        let mut work = vec![PrintTask::Node(root)];
        while let Some(task) = work.pop() {
            match task {
                PrintTask::Byte(byte) => out.push(byte),
                PrintTask::Node(id) => {
                    let id = self.resolve(id)?;
                    if labels.shared.contains(&id) {
                        if !labels.printed.insert(id) {
                            out.push(b'_');
                            push_display(out, id.index());
                            continue;
                        }
                        out.push(b':');
                        push_display(out, id.index());
                        out.push(b' ');
                    }

                    match self.node_for_debug(id) {
                        Node::App(fun, arg) => {
                            out.push(b'(');
                            work.push(PrintTask::Byte(b')'));
                            work.push(PrintTask::Node(arg));
                            work.push(PrintTask::Byte(b' '));
                            work.push(PrintTask::Node(fun));
                        }
                        Node::Indir(_) | Node::Free(_) => {
                            return Err(EvalError::DanglingIndirection(id));
                        }
                        Node::Prim(name) => {
                            out.extend_from_slice(name.name().as_bytes());
                        }
                        Node::Int(n) => {
                            out.push(b'#');
                            push_display(out, n);
                        }
                        Node::Int64(n) => {
                            out.extend_from_slice(b"##");
                            push_display(out, n);
                        }
                        Node::Float64(n) => {
                            out.push(b'&');
                            out.extend_from_slice(format_float(n).as_bytes());
                        }
                        Node::Float32(n) => {
                            out.extend_from_slice(b"&&");
                            out.extend_from_slice(format_float(f64::from(n)).as_bytes());
                        }
                        Node::ThreadId(n) => {
                            out.extend_from_slice(b"ThreadId#");
                            push_display(out, n);
                        }
                        Node::Ptr(ptr) => {
                            if ptr == 0 {
                                out.extend_from_slice(b"(toPtr #0)");
                            } else if let Some(handle) = handle_name_from_ptr(ptr) {
                                out.extend_from_slice(handle.as_bytes());
                            } else {
                                out.extend_from_slice(b"Ptr#");
                                push_display(out, ptr);
                            }
                        }
                        Node::RawFunPtr(ptr) => {
                            out.push(b';');
                            push_display(out, ptr);
                        }
                        Node::ForeignPtr(foreign_ptr) => {
                            if let Some(mpz) = self.mpz_decimal_bytes_for_ptr(foreign_ptr.ptr) {
                                out.push(b'%');
                                out.extend_from_slice(mpz);
                                out.push(b'"');
                            } else if let Some(bytes) = &foreign_ptr.bytes {
                                serialize_bytes_comb(bytes, out);
                            } else if let Some(handle) = handle_name_from_ptr(foreign_ptr.ptr) {
                                out.extend_from_slice(handle.as_bytes());
                            } else {
                                out.extend_from_slice(b"ForeignPtr#");
                                push_display(out, foreign_ptr.ptr);
                            }
                        }
                        Node::Weak(_) | Node::MVar(_) => {
                            return Err(EvalError::UnsupportedSerialization(id));
                        }
                        Node::BigInt(bytes) => {
                            serialize_bigint_decimal(&bytes, out);
                        }
                        Node::Bytes(bytes) => {
                            serialize_bytes_comb(&bytes, out);
                        }
                        Node::BytesView(_) => {
                            serialize_bytes_comb(self.bytes(id)?, out);
                        }
                        Node::MutableBytes(bytes) => {
                            serialize_bytes_comb(bytes.visible(), out);
                        }
                        Node::Array(items) => {
                            out.push(b'[');
                            push_display(out, items.len());
                            out.push(b']');
                            for item in items.iter().rev() {
                                work.push(PrintTask::Node(*item));
                                work.push(PrintTask::Byte(b' '));
                            }
                        }
                        Node::Ffi(name) => {
                            out.push(b'^');
                            out.extend_from_slice(name.as_bytes());
                        }
                        Node::JsCall(call) => {
                            out.push(b'~');
                            out.extend_from_slice(call.tags.as_bytes());
                            out.push(b' ');
                            serialize_bytes_quoted(&call.body, out);
                        }
                        Node::JsWrap { tags } => {
                            out.push(b'`');
                            out.extend_from_slice(tags.as_bytes());
                        }
                        Node::FunPtr(name) => {
                            out.push(b';');
                            out.extend_from_slice(name.as_bytes());
                        }
                        Node::Tick(name) => {
                            out.push(b'!');
                            serialize_bytes_quoted(&name, out);
                        }
                    }
                }
            }
        }
        Ok(())
    }

    #[cold]
    #[inline(never)]
    fn serialize_comb_into(
        &self,
        root: NodeId,
        labels: &mut SerializationLabels,
        out: &mut Vec<u8>,
    ) -> Result<(), EvalError> {
        enum SerializeTask {
            Node(NodeId),
            App,
            Array(usize),
            Label(NodeId),
        }

        let mut work = vec![SerializeTask::Node(root)];
        while let Some(task) = work.pop() {
            match task {
                SerializeTask::Node(id) => {
                    let id = self.resolve(id)?;
                    let share = if labels.shared.contains(&id) {
                        if !labels.printed.insert(id) {
                            out.push(b'_');
                            push_display(out, id.index());
                            out.push(b' ');
                            continue;
                        }
                        true
                    } else {
                        false
                    };

                    match self.node_for_debug(id) {
                        Node::App(fun, arg) => {
                            if share {
                                work.push(SerializeTask::Label(id));
                            }
                            work.push(SerializeTask::App);
                            work.push(SerializeTask::Node(arg));
                            work.push(SerializeTask::Node(fun));
                        }
                        Node::Indir(_) | Node::Free(_) => {
                            return Err(EvalError::DanglingIndirection(id));
                        }
                        Node::Prim(name) => {
                            out.extend_from_slice(name.name().as_bytes());
                            out.push(b' ');
                            if share {
                                work.push(SerializeTask::Label(id));
                            }
                        }
                        Node::Int(n) => {
                            out.push(b'#');
                            push_display(out, n);
                            out.push(b' ');
                            if share {
                                work.push(SerializeTask::Label(id));
                            }
                        }
                        Node::Int64(n) => {
                            out.extend_from_slice(b"##");
                            push_display(out, n);
                            out.push(b' ');
                            if share {
                                work.push(SerializeTask::Label(id));
                            }
                        }
                        Node::Float64(n) => {
                            out.push(b'&');
                            out.extend_from_slice(format_float(n).as_bytes());
                            out.push(b' ');
                            if share {
                                work.push(SerializeTask::Label(id));
                            }
                        }
                        Node::Float32(n) => {
                            out.extend_from_slice(b"&&");
                            out.extend_from_slice(format_float(f64::from(n)).as_bytes());
                            out.push(b' ');
                            if share {
                                work.push(SerializeTask::Label(id));
                            }
                        }
                        Node::ThreadId(_) | Node::Weak(_) | Node::MVar(_) => {
                            return Err(EvalError::UnsupportedSerialization(id));
                        }
                        Node::Ptr(ptr) => {
                            serialize_ptr(ptr, out);
                            out.push(b' ');
                            if share {
                                work.push(SerializeTask::Label(id));
                            }
                        }
                        Node::RawFunPtr(ptr) => {
                            out.extend_from_slice(b"toFunPtr #");
                            push_display(out, ptr);
                            out.extend_from_slice(b" @ ");
                            if share {
                                work.push(SerializeTask::Label(id));
                            }
                        }
                        Node::ForeignPtr(foreign_ptr) => {
                            if let Some(mpz) = self.mpz_decimal_bytes_for_ptr(foreign_ptr.ptr) {
                                serialize_bigint_decimal(mpz, out);
                            } else if let Some(bytes) = &foreign_ptr.bytes {
                                if foreign_ptr.offset == 0 {
                                    out.extend_from_slice(b"bs2fp ");
                                    serialize_bytes_comb(bytes, out);
                                    out.extend_from_slice(b" @");
                                } else {
                                    out.extend_from_slice(b"fp+ bs2fp ");
                                    serialize_bytes_comb(bytes, out);
                                    out.extend_from_slice(b" @ #");
                                    push_display(out, foreign_ptr.offset);
                                    out.extend_from_slice(b" @");
                                }
                            } else {
                                out.extend_from_slice(b"fpnew ");
                                serialize_ptr(foreign_ptr.ptr, out);
                                out.extend_from_slice(b" @");
                            }
                            out.push(b' ');
                            if share {
                                work.push(SerializeTask::Label(id));
                            }
                        }
                        Node::BigInt(bytes) => {
                            serialize_bigint_decimal(&bytes, out);
                            out.push(b' ');
                            if share {
                                work.push(SerializeTask::Label(id));
                            }
                        }
                        Node::Bytes(bytes) => {
                            serialize_bytes_comb(&bytes, out);
                            out.push(b' ');
                            if share {
                                work.push(SerializeTask::Label(id));
                            }
                        }
                        Node::BytesView(_) => {
                            serialize_bytes_comb(self.bytes(id)?, out);
                            out.push(b' ');
                            if share {
                                work.push(SerializeTask::Label(id));
                            }
                        }
                        Node::MutableBytes(bytes) => {
                            serialize_bytes_comb(bytes.visible(), out);
                            out.push(b' ');
                            if share {
                                work.push(SerializeTask::Label(id));
                            }
                        }
                        Node::Array(items) => {
                            if share {
                                work.push(SerializeTask::Label(id));
                            }
                            work.push(SerializeTask::Array(items.len()));
                            for item in items.iter().rev() {
                                work.push(SerializeTask::Node(*item));
                            }
                        }
                        Node::Ffi(name) => {
                            out.push(b'^');
                            out.extend_from_slice(name.as_bytes());
                            out.push(b' ');
                            if share {
                                work.push(SerializeTask::Label(id));
                            }
                        }
                        Node::JsCall(call) => {
                            out.push(b'~');
                            out.extend_from_slice(call.tags.as_bytes());
                            out.push(b' ');
                            serialize_bytes_quoted(&call.body, out);
                            out.push(b' ');
                            if share {
                                work.push(SerializeTask::Label(id));
                            }
                        }
                        Node::JsWrap { tags } => {
                            out.push(b'`');
                            out.extend_from_slice(tags.as_bytes());
                            out.push(b' ');
                            if share {
                                work.push(SerializeTask::Label(id));
                            }
                        }
                        Node::FunPtr(name) => {
                            out.push(b';');
                            out.extend_from_slice(name.as_bytes());
                            out.push(b' ');
                            if share {
                                work.push(SerializeTask::Label(id));
                            }
                        }
                        Node::Tick(name) => {
                            out.push(b'!');
                            serialize_bytes_quoted(&name, out);
                            out.push(b' ');
                            if share {
                                work.push(SerializeTask::Label(id));
                            }
                        }
                    }
                }
                SerializeTask::App => out.push(b'@'),
                SerializeTask::Array(len) => {
                    out.push(b'[');
                    push_display(out, len);
                    out.push(b']');
                    out.push(b' ');
                }
                SerializeTask::Label(id) => {
                    out.push(b':');
                    push_display(out, id.index());
                    out.push(b' ');
                }
            }
        }
        Ok(())
    }

    fn new_stable_ptr_handle(&mut self, value: NodeId) -> Result<i64, EvalError> {
        let slot = self
            .stable_ptrs
            .iter()
            .enumerate()
            .skip(1)
            .find_map(|(slot, value)| value.is_none().then_some(slot));
        let slot = match slot {
            Some(slot) => {
                self.stable_ptrs[slot] = Some(value);
                slot
            }
            None => {
                self.stable_ptrs.push(Some(value));
                self.stable_ptrs.len() - 1
            }
        };
        i64::try_from(slot).map_err(|_| EvalError::Overflow)
    }

    fn new_stable_ptr(&mut self, value: NodeId) -> Result<NodeId, EvalError> {
        let handle = self.new_stable_ptr_handle(value)?;
        Ok(self.int(handle))
    }

    fn stable_ptr_handle(&mut self, id: NodeId) -> Result<usize, EvalError> {
        usize::try_from(self.eval_int(id)?).map_err(|_| EvalError::InvalidStablePtr)
    }

    fn deref_stable_ptr(&self, handle: usize) -> Result<NodeId, EvalError> {
        self.stable_ptrs
            .get(handle)
            .and_then(|value| *value)
            .ok_or(EvalError::InvalidStablePtr)
    }

    fn free_stable_ptr(&mut self, handle: usize) -> Result<(), EvalError> {
        let slot = self
            .stable_ptrs
            .get_mut(handle)
            .ok_or(EvalError::InvalidStablePtr)?;
        if slot.is_none() {
            return Err(EvalError::InvalidStablePtr);
        }
        *slot = None;
        Ok(())
    }

    fn new_foreign_finalizer(&mut self, arg: i64) -> usize {
        let state = ForeignFinalizerState {
            arg,
            finalizer: None,
        };
        if let Some(index) = self.foreign_finalizer_free.pop() {
            self.foreign_finalizers[index] = Some(state);
            index
        } else {
            let index = self.foreign_finalizers.len();
            self.foreign_finalizers.push(Some(state));
            index
        }
    }

    fn foreign_ptr_node(&mut self, bytes: Option<Vec<u8>>, offset: usize, ptr: i64) -> Node {
        let finalizer = Some(self.new_foreign_finalizer(ptr));
        Node::ForeignPtr(Box::new(ForeignPtrNode {
            bytes,
            offset,
            ptr,
            finalizer,
        }))
    }

    fn offset_foreign_ptr(&mut self, id: NodeId, by: usize) -> Result<NodeId, EvalError> {
        let (bytes, offset, ptr, finalizer) = match self.cold_node(id) {
            Some(Node::ForeignPtr(foreign_ptr)) => (
                foreign_ptr.bytes.clone(),
                foreign_ptr.offset,
                foreign_ptr.ptr,
                foreign_ptr.finalizer,
            ),
            _ => return Err(EvalError::ExpectedForeignPtr(id)),
        };
        let offset = offset.checked_add(by).ok_or(EvalError::Overflow)?;
        let ptr = ptr
            .checked_add(i64::try_from(by).map_err(|_| EvalError::Overflow)?)
            .ok_or(EvalError::Overflow)?;
        Ok(self.push_node(Node::ForeignPtr(Box::new(ForeignPtrNode {
            bytes,
            offset,
            ptr,
            finalizer,
        }))))
    }

    fn foreign_ptr_to_bytes(&mut self, id: NodeId, len: usize) -> Result<NodeId, EvalError> {
        let foreign_ptr = match self.cold_node(id) {
            Some(Node::ForeignPtr(foreign_ptr)) => foreign_ptr.clone(),
            _ => return Err(EvalError::ExpectedForeignPtr(id)),
        };
        let bytes = match self.read_pointer_bytes(foreign_ptr.ptr, len) {
            Ok(bytes) => bytes,
            Err(_) if foreign_ptr.bytes.is_some() => {
                let bytes = foreign_ptr.bytes.as_ref().expect("checked above");
                let end = foreign_ptr
                    .offset
                    .checked_add(len)
                    .filter(|end| *end <= bytes.len())
                    .ok_or(EvalError::InvalidByteString)?;
                bytes[foreign_ptr.offset..end].to_vec()
            }
            Err(err) => return Err(err),
        };
        Ok(self.push_node(Node::bytes(bytes)))
    }

    fn foreign_ptr_value(&self, id: NodeId) -> Result<i64, EvalError> {
        match self.cold_node(id) {
            Some(Node::ForeignPtr(foreign_ptr)) => Ok(foreign_ptr.ptr),
            _ => match self.cell(id).prim() {
                Some(prim) => std_handle_ptr(prim.name()).ok_or(EvalError::ExpectedForeignPtr(id)),
                _ => Err(EvalError::ExpectedForeignPtr(id)),
            },
        }
    }

    fn eval_js_object_handle(&mut self, id: NodeId) -> Result<u32, EvalError> {
        let foreign_ptr = self.eval_foreign_ptr_id(id)?;
        u32::try_from(self.foreign_ptr_value(foreign_ptr)?).map_err(|_| EvalError::Overflow)
    }

    fn js_object_node(&mut self, handle: u32) -> Node {
        self.foreign_ptr_node(None, 0, i64::from(handle))
    }

    fn set_foreign_ptr_finalizer(
        &mut self,
        id: NodeId,
        finalizer: NodeId,
    ) -> Result<(), EvalError> {
        let finalizer = self.eval_foreign_finalizer(finalizer)?;
        let (group, ptr) = match self.cold_node(id) {
            Some(Node::ForeignPtr(foreign_ptr)) => (foreign_ptr.finalizer, foreign_ptr.ptr),
            _ => return Err(EvalError::ExpectedForeignPtr(id)),
        };
        let group = match group {
            Some(group) => group,
            None => {
                let group = self.new_foreign_finalizer(ptr);
                match self.cold_node_mut(id) {
                    Some(Node::ForeignPtr(foreign_ptr)) => {
                        foreign_ptr.finalizer = Some(group);
                    }
                    _ => return Err(EvalError::ExpectedForeignPtr(id)),
                }
                group
            }
        };
        let state = self
            .foreign_finalizers
            .get_mut(group)
            .and_then(Option::as_mut)
            .ok_or(EvalError::ExpectedForeignPtr(id))?;
        state.finalizer = Some(finalizer);
        Ok(())
    }

    fn eval_foreign_finalizer(&mut self, id: NodeId) -> Result<ForeignFinalizer, EvalError> {
        let root = self.reduce_node_whnf(id, FORCE_REDUCTION_LIMIT)?;
        let root = self.resolve(root)?;
        if let Some(value) = self.cell(root).raw_fun_ptr_value() {
            return if value == 0 {
                Ok(ForeignFinalizer::RawZero)
            } else {
                Err(EvalError::UnsupportedForeignFinalizer(format!(
                    "FunPtr#{value}"
                )))
            };
        }
        match self.cold_node(root) {
            Some(Node::FunPtr(name)) => match name.as_str() {
                "0" => Ok(ForeignFinalizer::RawZero),
                "free" => Ok(ForeignFinalizer::Free),
                "closeb" => Ok(ForeignFinalizer::CloseB),
                _ => Err(EvalError::UnsupportedForeignFinalizer(name.to_string())),
            },
            Some(Node::Ffi(name)) => match name.as_str() {
                "&free" => Ok(ForeignFinalizer::Free),
                "&closeb" => Ok(ForeignFinalizer::CloseB),
                _ => Err(EvalError::UnsupportedForeignFinalizer(name.to_string())),
            },
            _ => Err(EvalError::UnsupportedForeignFinalizer(
                self.node_trace_summary(root),
            )),
        }
    }

    fn new_weak_ptr(&mut self, key: NodeId, value: NodeId, finalizer: Option<NodeId>) -> NodeId {
        let finalizer = finalizer.map(|finalizer| {
            let world = self.world();
            self.app(finalizer, world)
        });
        let weak = self.push_node(Node::Weak(Box::new(WeakNode {
            key: Some(key),
            value: Some(value),
            finalizer,
        })));
        self.weak_nodes.push(weak);
        weak
    }

    fn eval_weak_id(&mut self, id: NodeId) -> Result<NodeId, EvalError> {
        self.eval_whnf_value(
            id,
            |program, root| match program.cold_node(root) {
                Some(Node::Weak(_)) => Some(root),
                _ => None,
            },
            EvalError::ExpectedWeak,
        )
    }

    fn deref_weak_ptr(&mut self, id: NodeId) -> Result<NodeId, EvalError> {
        let id = self.eval_weak_id(id)?;
        let value = match self.cold_node(id) {
            Some(Node::Weak(weak)) => weak.value,
            _ => unreachable!(),
        };
        Ok(match value {
            Some(value) => self.just(value),
            None => self.nothing(),
        })
    }

    fn finalize_weak_ptr(&mut self, id: NodeId) -> Result<(), EvalError> {
        let id = self.eval_weak_id(id)?;
        let finalizer = match self.cold_node_mut(id) {
            Some(Node::Weak(weak)) => weak.finalizer.take(),
            _ => unreachable!(),
        };
        if let Some(finalizer) = finalizer {
            self.reduce_node_whnf(finalizer, FORCE_REDUCTION_LIMIT)?;
        }
        Ok(())
    }

    fn eval_mvar_id(&mut self, id: NodeId) -> Result<NodeId, EvalError> {
        self.eval_whnf_value(
            id,
            |program, root| match program.cold_node(root) {
                Some(Node::MVar(_)) => Some(root),
                _ => None,
            },
            EvalError::ExpectedMVar,
        )
    }

    fn read_mvar(&self, id: NodeId) -> Result<Option<NodeId>, EvalError> {
        match self.cold_node(id) {
            Some(Node::MVar(value)) => Ok(*value),
            _ => Err(EvalError::ExpectedMVar(id)),
        }
    }

    fn take_mvar(&mut self, id: NodeId) -> Result<Option<NodeId>, EvalError> {
        match self.cold_node_mut(id) {
            Some(Node::MVar(value)) => Ok(value.take()),
            _ => Err(EvalError::ExpectedMVar(id)),
        }
    }

    fn put_mvar(&mut self, id: NodeId, value: NodeId) -> Result<(), EvalError> {
        if !self.try_put_mvar(id, value)? {
            return Err(EvalError::InvalidMVar);
        }
        Ok(())
    }

    fn try_put_mvar(&mut self, id: NodeId, new_value: NodeId) -> Result<bool, EvalError> {
        match self.cold_node_mut(id) {
            Some(Node::MVar(value)) if value.is_none() => {
                *value = Some(new_value);
                Ok(true)
            }
            Some(Node::MVar(_)) => Ok(false),
            _ => Err(EvalError::ExpectedMVar(id)),
        }
    }

    fn int_list(&mut self, values: impl IntoIterator<Item = i64>) -> NodeId {
        let values: Vec<_> = values.into_iter().collect();
        let mut list = self.prim("K");
        for value in values.into_iter().rev() {
            let cons = self.prim("O");
            let value = self.int(value);
            let head = self.app(cons, value);
            list = self.app(head, list);
        }
        list
    }

    fn arg_ref_array(&mut self) -> NodeId {
        if let Some(array) = self.arg_ref_array {
            return array;
        }
        let mut list = self.prim("K");
        for arg in self.program_args.clone().into_iter().rev() {
            let string = self.int_list(arg.into_iter().map(i64::from));
            let cons = self.prim("O");
            let cons_string = self.app(cons, string);
            list = self.app(cons_string, list);
        }
        let array = self.push_node(Node::array(vec![list]));
        self.arg_ref_array = Some(array);
        array
    }

    fn reduce_node_whnf(&mut self, root: NodeId, limit: usize) -> Result<NodeId, EvalError> {
        self.reduce_whnf_from(root, limit, false, false)
            .map(|(root, _)| root)
    }

    fn rnf(&mut self, noerr: bool, root: NodeId) -> Result<(), EvalError> {
        let mut seen = HashSet::new();
        let mut stack = vec![root];
        while let Some(root) = stack.pop() {
            let root = self.resolve(root)?;
            if !seen.insert(root) {
                continue;
            }
            let root = match self.reduce_node_whnf(root, FORCE_REDUCTION_LIMIT) {
                Ok(root) => self.resolve(root)?,
                Err(EvalError::Raised(_)) if noerr => continue,
                Err(err) => return Err(err),
            };
            if let Some((fun, arg)) = self.cell(root).app_fields() {
                stack.push(arg);
                stack.push(fun);
            }
        }
        Ok(())
    }

    pub fn render(&self, root: NodeId) -> String {
        let mut out = String::new();
        self.render_into(root, 0, &mut out);
        out
    }

    fn render_into(&self, id: NodeId, depth: usize, out: &mut String) {
        if depth > 80 {
            out.push_str("...");
            return;
        }
        let Ok(id) = self.resolve(id) else {
            out.push_str("<dangling>");
            return;
        };
        let node = self.node_for_debug(id);
        match node {
            Node::App(fun, arg) => {
                out.push('(');
                self.render_into(fun, depth + 1, out);
                out.push(' ');
                self.render_into(arg, depth + 1, out);
                out.push(')');
            }
            Node::Indir(_) => out.push_str("<indir>"),
            Node::Free(_) => out.push_str("<free>"),
            Node::Prim(name) => out.push_str(name.name()),
            Node::Int(n) => out.push_str(&n.to_string()),
            Node::Int64(n) => {
                out.push_str(&n.to_string());
                out.push_str("i64");
            }
            Node::Float64(n) => out.push_str(&format_float(n)),
            Node::Float32(n) => {
                out.push_str(&format_float(f64::from(n)));
                out.push('f');
            }
            Node::ThreadId(n) => {
                out.push_str("ThreadId#");
                out.push_str(&n.to_string());
            }
            Node::Ptr(n) => {
                out.push_str("Ptr#");
                out.push_str(&n.to_string());
            }
            Node::RawFunPtr(n) => {
                out.push_str("FunPtr#");
                out.push_str(&n.to_string());
            }
            Node::ForeignPtr(foreign_ptr) => {
                if let Some(mpz) = self.mpz_decimal_bytes_for_ptr(foreign_ptr.ptr) {
                    out.push('%');
                    render_bytes(mpz, out);
                } else {
                    out.push_str("ForeignPtr#");
                    out.push_str(&foreign_ptr.ptr.to_string());
                }
            }
            Node::Weak(_) => {
                out.push_str("Weak#");
                out.push_str(&id.0.to_string());
            }
            Node::MVar(_) => {
                out.push_str("MVar#");
                out.push_str(&id.0.to_string());
            }
            Node::BigInt(bytes) => {
                out.push('%');
                render_bytes(&bytes, out);
            }
            Node::Bytes(bytes) => render_bytes(&bytes, out),
            Node::BytesView(_) => {
                if let Ok(bytes) = self.bytes(id) {
                    render_bytes(bytes, out);
                } else {
                    out.push_str("<bytes-view>");
                }
            }
            Node::MutableBytes(bytes) => render_bytes(bytes.visible(), out),
            Node::Array(items) => {
                out.push('[');
                for (idx, item) in items.iter().enumerate() {
                    if idx != 0 {
                        out.push_str(", ");
                    }
                    self.render_into(*item, depth + 1, out);
                }
                out.push(']');
            }
            Node::Ffi(name) => {
                out.push('^');
                out.push_str(&name);
            }
            Node::JsCall(call) => {
                out.push('~');
                out.push_str(&call.tags);
                out.push(' ');
                render_bytes(&call.body, out);
            }
            Node::JsWrap { tags } => {
                out.push('`');
                out.push_str(&tags);
            }
            Node::FunPtr(name) => {
                out.push(';');
                out.push_str(&name);
            }
            Node::Tick(name) => {
                out.push('!');
                render_bytes(&name, out);
            }
        }
    }
}

enum IntResult {
    Int(i64),
    Bool(bool),
    Ordering(Ordering),
}

#[derive(Clone, Copy, Debug)]
enum IntBinOp {
    Add,
    Sub,
    Mul,
    Quot,
    Rem,
    SubR,
    UAdd,
    USub,
    UMul,
    UQuot,
    URem,
    USubR,
    And,
    Or,
    Xor,
    Shl,
    Shr,
    Ashr,
    Eq,
    Ne,
    Lt,
    Le,
    Gt,
    Ge,
    Ult,
    Ule,
    Ugt,
    Uge,
    ICmp,
    UCmp,
}

impl IntBinOp {
    fn from_prim(name: &str) -> Option<Self> {
        Some(match name {
            "+" => Self::Add,
            "-" => Self::Sub,
            "*" => Self::Mul,
            "quot" => Self::Quot,
            "rem" => Self::Rem,
            "subtract" => Self::SubR,
            "u+" => Self::UAdd,
            "u-" => Self::USub,
            "u*" => Self::UMul,
            "uquot" => Self::UQuot,
            "urem" => Self::URem,
            "usubtract" => Self::USubR,
            "and" => Self::And,
            "or" => Self::Or,
            "xor" => Self::Xor,
            "shl" => Self::Shl,
            "shr" => Self::Shr,
            "ashr" => Self::Ashr,
            "==" => Self::Eq,
            "/=" => Self::Ne,
            "<" => Self::Lt,
            "<=" => Self::Le,
            ">" => Self::Gt,
            ">=" => Self::Ge,
            "u<" => Self::Ult,
            "u<=" => Self::Ule,
            "u>" => Self::Ugt,
            "u>=" => Self::Uge,
            "icmp" => Self::ICmp,
            "ucmp" => Self::UCmp,
            _ => return None,
        })
    }

    fn apply(self, x: i64, y: i64) -> Result<IntResult, EvalError> {
        let xu = x as u64;
        let yu = y as u64;
        let n = match self {
            Self::Add => x.checked_add(y).ok_or(EvalError::Overflow)?,
            Self::Sub => x.checked_sub(y).ok_or(EvalError::Overflow)?,
            Self::Mul => x.checked_mul(y).ok_or(EvalError::Overflow)?,
            Self::Quot => {
                if y == 0 {
                    return Err(EvalError::DivideByZero);
                }
                x.checked_div(y).ok_or(EvalError::Overflow)?
            }
            Self::Rem => {
                if y == 0 {
                    return Err(EvalError::DivideByZero);
                }
                x.checked_rem(y).ok_or(EvalError::Overflow)?
            }
            Self::SubR => y.checked_sub(x).ok_or(EvalError::Overflow)?,
            Self::UAdd => xu.wrapping_add(yu) as i64,
            Self::USub => xu.wrapping_sub(yu) as i64,
            Self::UMul => xu.wrapping_mul(yu) as i64,
            Self::UQuot => {
                if yu == 0 {
                    return Err(EvalError::DivideByZero);
                }
                (xu / yu) as i64
            }
            Self::URem => {
                if yu == 0 {
                    return Err(EvalError::DivideByZero);
                }
                (xu % yu) as i64
            }
            Self::USubR => yu.wrapping_sub(xu) as i64,
            Self::And => (xu & yu) as i64,
            Self::Or => (xu | yu) as i64,
            Self::Xor => (xu ^ yu) as i64,
            Self::Shl => (xu.wrapping_shl(shift(y)?)) as i64,
            Self::Shr => (xu.wrapping_shr(shift(y)?)) as i64,
            Self::Ashr => x.wrapping_shr(shift(y)?),
            Self::Eq => return Ok(IntResult::Bool(xu == yu)),
            Self::Ne => return Ok(IntResult::Bool(xu != yu)),
            Self::Lt => return Ok(IntResult::Bool(x < y)),
            Self::Le => return Ok(IntResult::Bool(x <= y)),
            Self::Gt => return Ok(IntResult::Bool(x > y)),
            Self::Ge => return Ok(IntResult::Bool(x >= y)),
            Self::Ult => return Ok(IntResult::Bool(xu < yu)),
            Self::Ule => return Ok(IntResult::Bool(xu <= yu)),
            Self::Ugt => return Ok(IntResult::Bool(xu > yu)),
            Self::Uge => return Ok(IntResult::Bool(xu >= yu)),
            Self::ICmp => return Ok(IntResult::Ordering(x.cmp(&y))),
            Self::UCmp => return Ok(IntResult::Ordering(xu.cmp(&yu))),
        };
        Ok(IntResult::Int(n))
    }
}

enum Int64Result {
    Int64(i64),
    Bool(bool),
    Ordering(Ordering),
}

#[derive(Clone, Copy, Debug)]
enum Int64BinOp {
    Add,
    Sub,
    Mul,
    Quot,
    Rem,
    SubR,
    UAdd,
    USub,
    UMul,
    UQuot,
    URem,
    USubR,
    And,
    Or,
    Xor,
    Shl,
    Shr,
    Ashr,
    Eq,
    Ne,
    Lt,
    Le,
    Gt,
    Ge,
    Ult,
    Ule,
    Ugt,
    Uge,
    ICmp,
    UCmp,
}

impl Int64BinOp {
    fn from_prim(name: &str) -> Option<Self> {
        Some(match name {
            "I+" => Self::Add,
            "I-" => Self::Sub,
            "I*" => Self::Mul,
            "Iquot" => Self::Quot,
            "Irem" => Self::Rem,
            "Isubtract" => Self::SubR,
            "Iu+" => Self::UAdd,
            "Iu-" => Self::USub,
            "Iu*" => Self::UMul,
            "Iuquot" => Self::UQuot,
            "Iurem" => Self::URem,
            "Iusubtract" => Self::USubR,
            "Iand" => Self::And,
            "Ior" => Self::Or,
            "Ixor" => Self::Xor,
            "Ishl" => Self::Shl,
            "Ishr" => Self::Shr,
            "Iashr" => Self::Ashr,
            "I==" => Self::Eq,
            "I/=" => Self::Ne,
            "I<" => Self::Lt,
            "I<=" => Self::Le,
            "I>" => Self::Gt,
            "I>=" => Self::Ge,
            "Iu<" => Self::Ult,
            "Iu<=" => Self::Ule,
            "Iu>" => Self::Ugt,
            "Iu>=" => Self::Uge,
            "Iicmp" => Self::ICmp,
            "Iucmp" => Self::UCmp,
            _ => return None,
        })
    }

    fn rhs_is_shift(self) -> bool {
        matches!(self, Self::Shl | Self::Shr | Self::Ashr)
    }

    fn driver_marker_safe(self) -> bool {
        !self.rhs_is_shift()
    }

    fn apply(self, x: i64, y: i64) -> Result<Int64Result, EvalError> {
        let xu = x as u64;
        let yu = y as u64;
        let n = match self {
            Self::Add => x.checked_add(y).ok_or(EvalError::Overflow)?,
            Self::Sub => x.checked_sub(y).ok_or(EvalError::Overflow)?,
            Self::Mul => x.checked_mul(y).ok_or(EvalError::Overflow)?,
            Self::Quot => {
                if y == 0 {
                    return Err(EvalError::DivideByZero);
                }
                x.checked_div(y).ok_or(EvalError::Overflow)?
            }
            Self::Rem => {
                if y == 0 {
                    return Err(EvalError::DivideByZero);
                }
                x.checked_rem(y).ok_or(EvalError::Overflow)?
            }
            Self::SubR => y.checked_sub(x).ok_or(EvalError::Overflow)?,
            Self::UAdd => xu.wrapping_add(yu) as i64,
            Self::USub => xu.wrapping_sub(yu) as i64,
            Self::UMul => xu.wrapping_mul(yu) as i64,
            Self::UQuot => {
                if yu == 0 {
                    return Err(EvalError::DivideByZero);
                }
                (xu / yu) as i64
            }
            Self::URem => {
                if yu == 0 {
                    return Err(EvalError::DivideByZero);
                }
                (xu % yu) as i64
            }
            Self::USubR => yu.wrapping_sub(xu) as i64,
            Self::And => (xu & yu) as i64,
            Self::Or => (xu | yu) as i64,
            Self::Xor => (xu ^ yu) as i64,
            Self::Shl => (xu.wrapping_shl(shift(y)?)) as i64,
            Self::Shr => (xu.wrapping_shr(shift(y)?)) as i64,
            Self::Ashr => x.wrapping_shr(shift(y)?),
            Self::Eq => return Ok(Int64Result::Bool(xu == yu)),
            Self::Ne => return Ok(Int64Result::Bool(xu != yu)),
            Self::Lt => return Ok(Int64Result::Bool(x < y)),
            Self::Le => return Ok(Int64Result::Bool(x <= y)),
            Self::Gt => return Ok(Int64Result::Bool(x > y)),
            Self::Ge => return Ok(Int64Result::Bool(x >= y)),
            Self::Ult => return Ok(Int64Result::Bool(xu < yu)),
            Self::Ule => return Ok(Int64Result::Bool(xu <= yu)),
            Self::Ugt => return Ok(Int64Result::Bool(xu > yu)),
            Self::Uge => return Ok(Int64Result::Bool(xu >= yu)),
            Self::ICmp => return Ok(Int64Result::Ordering(x.cmp(&y))),
            Self::UCmp => return Ok(Int64Result::Ordering(xu.cmp(&yu))),
        };
        Ok(Int64Result::Int64(n))
    }
}

enum Int64UnResult {
    Int64(i64),
    Int(i64),
}

#[derive(Clone, Copy, Debug)]
enum Int64UnOp {
    Neg,
    UNeg,
    Inv,
    PopCount,
    Clz,
    Ctz,
}

impl Int64UnOp {
    fn from_prim(name: &str) -> Option<Self> {
        Some(match name {
            "Ineg" => Self::Neg,
            "Iuneg" => Self::UNeg,
            "Iinv" => Self::Inv,
            "Ipopcount" => Self::PopCount,
            "Iclz" => Self::Clz,
            "Ictz" => Self::Ctz,
            _ => return None,
        })
    }

    fn apply(self, x: i64) -> Result<Int64UnResult, EvalError> {
        let xu = x as u64;
        Ok(match self {
            Self::Neg => Int64UnResult::Int64(x.checked_neg().ok_or(EvalError::Overflow)?),
            Self::UNeg => Int64UnResult::Int64((0u64.wrapping_sub(xu)) as i64),
            Self::Inv => Int64UnResult::Int64(!xu as i64),
            Self::PopCount => Int64UnResult::Int(xu.count_ones() as i64),
            Self::Clz => Int64UnResult::Int(xu.leading_zeros() as i64),
            Self::Ctz => Int64UnResult::Int(xu.trailing_zeros() as i64),
        })
    }
}

enum Float64Result {
    Float(f64),
    Bool(bool),
}

#[derive(Clone, Copy, Debug)]
enum Float64BinOp {
    Add,
    Sub,
    Mul,
    Div,
    Eq,
    Ne,
    Lt,
    Le,
    Gt,
    Ge,
}

impl Float64BinOp {
    fn from_prim(name: &str) -> Option<Self> {
        Some(match name {
            "d+" => Self::Add,
            "d-" => Self::Sub,
            "d*" => Self::Mul,
            "d/" => Self::Div,
            "d==" => Self::Eq,
            "d/=" => Self::Ne,
            "d<" => Self::Lt,
            "d<=" => Self::Le,
            "d>" => Self::Gt,
            "d>=" => Self::Ge,
            _ => return None,
        })
    }

    fn apply(self, x: f64, y: f64) -> Float64Result {
        match self {
            Self::Add => Float64Result::Float(x + y),
            Self::Sub => Float64Result::Float(x - y),
            Self::Mul => Float64Result::Float(x * y),
            Self::Div => Float64Result::Float(x / y),
            Self::Eq => Float64Result::Bool(x == y),
            Self::Ne => Float64Result::Bool(x != y),
            Self::Lt => Float64Result::Bool(x < y),
            Self::Le => Float64Result::Bool(x <= y),
            Self::Gt => Float64Result::Bool(x > y),
            Self::Ge => Float64Result::Bool(x >= y),
        }
    }
}

#[derive(Clone, Copy, Debug)]
enum Float64UnOp {
    Neg,
}

impl Float64UnOp {
    fn from_prim(name: &str) -> Option<Self> {
        Some(match name {
            "dneg" => Self::Neg,
            _ => return None,
        })
    }

    fn apply(self, x: f64) -> f64 {
        match self {
            Self::Neg => -x,
        }
    }
}

enum Float32Result {
    Float(f32),
    Bool(bool),
}

#[derive(Clone, Copy, Debug)]
enum Float32BinOp {
    Add,
    Sub,
    Mul,
    Div,
    Eq,
    Ne,
    Lt,
    Le,
    Gt,
    Ge,
}

impl Float32BinOp {
    fn from_prim(name: &str) -> Option<Self> {
        Some(match name {
            "f+" => Self::Add,
            "f-" => Self::Sub,
            "f*" => Self::Mul,
            "f/" => Self::Div,
            "f==" => Self::Eq,
            "f/=" => Self::Ne,
            "f<" => Self::Lt,
            "f<=" => Self::Le,
            "f>" => Self::Gt,
            "f>=" => Self::Ge,
            _ => return None,
        })
    }

    fn apply(self, x: f32, y: f32) -> Float32Result {
        match self {
            Self::Add => Float32Result::Float(x + y),
            Self::Sub => Float32Result::Float(x - y),
            Self::Mul => Float32Result::Float(x * y),
            Self::Div => Float32Result::Float(x / y),
            Self::Eq => Float32Result::Bool(x == y),
            Self::Ne => Float32Result::Bool(x != y),
            Self::Lt => Float32Result::Bool(x < y),
            Self::Le => Float32Result::Bool(x <= y),
            Self::Gt => Float32Result::Bool(x > y),
            Self::Ge => Float32Result::Bool(x >= y),
        }
    }
}

#[derive(Clone, Copy, Debug)]
enum Float32UnOp {
    Neg,
}

impl Float32UnOp {
    fn from_prim(name: &str) -> Option<Self> {
        Some(match name {
            "fneg" => Self::Neg,
            _ => return None,
        })
    }

    fn apply(self, x: f32) -> f32 {
        match self {
            Self::Neg => -x,
        }
    }
}

#[derive(Clone, Copy, Debug)]
enum BytesBinOp {
    Append,
    AppendDot,
    Eq,
    Ne,
    Lt,
    Le,
    Gt,
    Ge,
    Cmp,
}

impl BytesBinOp {
    fn from_prim(name: &str) -> Option<Self> {
        Some(match name {
            "bs++" => Self::Append,
            "bs++." => Self::AppendDot,
            "bs==" => Self::Eq,
            "bs/=" => Self::Ne,
            "bs<" => Self::Lt,
            "bs<=" => Self::Le,
            "bs>" => Self::Gt,
            "bs>=" => Self::Ge,
            "bscmp" => Self::Cmp,
            _ => return None,
        })
    }
}

#[derive(Clone, Copy, Debug)]
enum IntUnOp {
    Neg,
    UNeg,
    Inv,
    PopCount,
    Clz,
    Ctz,
}

impl IntUnOp {
    fn from_prim(name: &str) -> Option<Self> {
        Some(match name {
            "neg" => Self::Neg,
            "uneg" => Self::UNeg,
            "inv" => Self::Inv,
            "popcount" => Self::PopCount,
            "clz" => Self::Clz,
            "ctz" => Self::Ctz,
            _ => return None,
        })
    }

    fn apply(self, x: i64) -> Result<i64, EvalError> {
        Ok(match self {
            Self::Neg => x.checked_neg().ok_or(EvalError::Overflow)?,
            Self::UNeg => (0u64.wrapping_sub(x as u64)) as i64,
            Self::Inv => !(x as u64) as i64,
            Self::PopCount => (x as u64).count_ones() as i64,
            Self::Clz => (x as u64).leading_zeros() as i64,
            Self::Ctz => (x as u64).trailing_zeros() as i64,
        })
    }
}

fn shift(n: i64) -> Result<u32, EvalError> {
    if !(0..64).contains(&n) {
        return Err(EvalError::InvalidShift(n));
    }
    Ok(n as u32)
}

fn tag_index(name: &str) -> Option<usize> {
    let tag = name.strip_prefix("TAG")?.parse().ok()?;
    (tag <= 32).then_some(tag)
}

fn tuple_fields(name: &str) -> Option<usize> {
    let fields = name.strip_prefix('T')?.parse().ok()?;
    (3..=16).contains(&fields).then_some(fields)
}

fn rts_exception_message(code: i64) -> &'static [u8] {
    match code {
        0 => b"stack overflow",
        1 => b"heap overflow",
        2 => b"thread killed",
        3 => b"user interrupt",
        4 => b"DivideByZero",
        5 => b"blocked MVar",
        6 => b"blocked STM",
        7 => b"arithmetic overflow",
        _ => b"unknown",
    }
}

pub(crate) fn is_runtime_prim_name(name: &str) -> bool {
    Prim::from_name(name).is_some()
}

fn is_supported_runtime_prim_name(name: &str) -> bool {
    is_runtime_prim_name(name)
        && !matches!(
            name,
            "IO.fork" | "IO.throwto" | "IO.threaddelay" | "IO.waitrdfd" | "IO.waitwrfd"
        )
}

fn int_to_usize(n: i64) -> Result<usize, EvalError> {
    usize::try_from(n).map_err(|_| EvalError::InvalidByteString)
}

const MD5_S: [u32; 64] = [
    7, 12, 17, 22, 7, 12, 17, 22, 7, 12, 17, 22, 7, 12, 17, 22, 5, 9, 14, 20, 5, 9, 14, 20, 5, 9,
    14, 20, 5, 9, 14, 20, 4, 11, 16, 23, 4, 11, 16, 23, 4, 11, 16, 23, 4, 11, 16, 23, 6, 10, 15,
    21, 6, 10, 15, 21, 6, 10, 15, 21, 6, 10, 15, 21,
];

const MD5_K: [u32; 64] = [
    0xd76aa478, 0xe8c7b756, 0x242070db, 0xc1bdceee, 0xf57c0faf, 0x4787c62a, 0xa8304613, 0xfd469501,
    0x698098d8, 0x8b44f7af, 0xffff5bb1, 0x895cd7be, 0x6b901122, 0xfd987193, 0xa679438e, 0x49b40821,
    0xf61e2562, 0xc040b340, 0x265e5a51, 0xe9b6c7aa, 0xd62f105d, 0x02441453, 0xd8a1e681, 0xe7d3fbc8,
    0x21e1cde6, 0xc33707d6, 0xf4d50d87, 0x455a14ed, 0xa9e3e905, 0xfcefa3f8, 0x676f02d9, 0x8d2a4c8a,
    0xfffa3942, 0x8771f681, 0x6d9d6122, 0xfde5380c, 0xa4beea44, 0x4bdecfa9, 0xf6bb4b60, 0xbebfbc70,
    0x289b7ec6, 0xeaa127fa, 0xd4ef3085, 0x04881d05, 0xd9d4d039, 0xe6db99e5, 0x1fa27cf8, 0xc4ac5665,
    0xf4292244, 0x432aff97, 0xab9423a7, 0xfc93a039, 0x655b59c3, 0x8f0ccc92, 0xffeff47d, 0x85845dd1,
    0x6fa87e4f, 0xfe2ce6e0, 0xa3014314, 0x4e0811a1, 0xf7537e82, 0xbd3af235, 0x2ad7d2bb, 0xeb86d391,
];

struct Md5Context {
    size: u64,
    state: [u32; 4],
    input: [u8; 64],
}

impl Md5Context {
    fn new() -> Self {
        Self {
            size: 0,
            state: [0x67452301, 0xefcdab89, 0x98badcfe, 0x10325476],
            input: [0; 64],
        }
    }

    fn update(&mut self, bytes: &[u8]) {
        let mut offset = (self.size % 64) as usize;
        self.size = self.size.wrapping_add(bytes.len() as u64);
        for &byte in bytes {
            self.input[offset] = byte;
            offset += 1;
            if offset == 64 {
                md5_step(&mut self.state, &md5_block_words(&self.input));
                offset = 0;
            }
        }
    }

    fn finalize(mut self) -> [u8; 16] {
        let offset = (self.size % 64) as usize;
        let padding_len = if offset < 56 {
            56 - offset
        } else {
            120 - offset
        };
        let mut padding = [0; 64];
        padding[0] = 0x80;
        self.update(&padding[..padding_len]);
        self.size = self.size.wrapping_sub(padding_len as u64);

        let mut block = md5_block_words(&self.input);
        let bit_len = self.size.wrapping_mul(8);
        block[14] = bit_len as u32;
        block[15] = (bit_len >> 32) as u32;
        md5_step(&mut self.state, &block);

        let mut digest = [0; 16];
        for (idx, word) in self.state.iter().enumerate() {
            digest[idx * 4..idx * 4 + 4].copy_from_slice(&word.to_le_bytes());
        }
        digest
    }
}

fn md5_bytes(bytes: &[u8]) -> [u8; 16] {
    let mut ctx = Md5Context::new();
    ctx.update(bytes);
    ctx.finalize()
}

fn md5_block_words(input: &[u8; 64]) -> [u32; 16] {
    let mut out = [0; 16];
    for (idx, word) in out.iter_mut().enumerate() {
        let start = idx * 4;
        *word = u32::from_le_bytes([
            input[start],
            input[start + 1],
            input[start + 2],
            input[start + 3],
        ]);
    }
    out
}

fn md5_step(state: &mut [u32; 4], input: &[u32; 16]) {
    let mut a = state[0];
    let mut b = state[1];
    let mut c = state[2];
    let mut d = state[3];

    for idx in 0..64 {
        let (e, word_idx) = match idx / 16 {
            0 => ((b & c) | (!b & d), idx),
            1 => ((b & d) | (c & !d), (idx * 5 + 1) % 16),
            2 => (b ^ c ^ d, (idx * 3 + 5) % 16),
            _ => (c ^ (b | !d), (idx * 7) % 16),
        };
        let old_d = d;
        d = c;
        c = b;
        b = b.wrapping_add(
            a.wrapping_add(e)
                .wrapping_add(MD5_K[idx])
                .wrapping_add(input[word_idx])
                .rotate_left(MD5_S[idx]),
        );
        a = old_d;
    }

    state[0] = state[0].wrapping_add(a);
    state[1] = state[1].wrapping_add(b);
    state[2] = state[2].wrapping_add(c);
    state[3] = state[3].wrapping_add(d);
}

fn rle_pending_bytes(count: usize, byte: i64) -> Result<Vec<u8>, EvalError> {
    if count == 0 {
        return Ok(Vec::new());
    }
    if !(0..128).contains(&byte) {
        return Err(EvalError::InvalidHandle);
    }
    let byte = byte as u8;
    if count > 2 {
        let mut out = Vec::new();
        push_rle_rep(count - 1, &mut out)?;
        out.push(byte);
        Ok(out)
    } else {
        Ok(vec![byte; count])
    }
}

fn push_rle_rep(n: usize, out: &mut Vec<u8>) -> Result<(), EvalError> {
    if n > 127 {
        push_rle_rep(n / 128, out)?;
    }
    let digit = u8::try_from(n % 128).map_err(|_| EvalError::Overflow)?;
    out.push(digit | 0x80);
    Ok(())
}

enum Base64Input {
    Value(i32),
    Whitespace,
    Invalid,
}

const BASE64_ALPHABET: &[u8; 65] =
    b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/=";
const BASE64_PAD: usize = 64;

fn base64_decode_value(byte: u8) -> Base64Input {
    match byte {
        b'A'..=b'Z' => Base64Input::Value(i32::from(byte - b'A')),
        b'a'..=b'z' => Base64Input::Value(i32::from(byte - b'a') + 26),
        b'0'..=b'9' => Base64Input::Value(i32::from(byte - b'0') + 52),
        b'+' => Base64Input::Value(62),
        b'/' => Base64Input::Value(63),
        b'=' => Base64Input::Value(-3),
        b' ' | b'\t' | b'\n' | b'\r' => Base64Input::Whitespace,
        _ => Base64Input::Invalid,
    }
}

fn base64_full_quad_bytes(encbuf: &[u8; 3], linelen: usize, outcol: &mut usize) -> Vec<u8> {
    let x = ((u32::from(encbuf[0])) << 16) | ((u32::from(encbuf[1])) << 8) | u32::from(encbuf[2]);
    base64_quad_bytes(
        [
            ((x >> 18) & 0x3f) as usize,
            ((x >> 12) & 0x3f) as usize,
            ((x >> 6) & 0x3f) as usize,
            (x & 0x3f) as usize,
        ],
        linelen,
        outcol,
    )
}

fn base64_pending_bytes(
    encbuf: &[u8; 3],
    encpos: &usize,
    linelen: usize,
    outcol: &mut usize,
) -> Result<Vec<u8>, EvalError> {
    Ok(match *encpos {
        0 => Vec::new(),
        1 => {
            let x = (u32::from(encbuf[0])) << 16;
            base64_quad_bytes(
                [
                    ((x >> 18) & 0x3f) as usize,
                    ((x >> 12) & 0x3f) as usize,
                    BASE64_PAD,
                    BASE64_PAD,
                ],
                linelen,
                outcol,
            )
        }
        2 => {
            let x = ((u32::from(encbuf[0])) << 16) | ((u32::from(encbuf[1])) << 8);
            base64_quad_bytes(
                [
                    ((x >> 18) & 0x3f) as usize,
                    ((x >> 12) & 0x3f) as usize,
                    ((x >> 6) & 0x3f) as usize,
                    BASE64_PAD,
                ],
                linelen,
                outcol,
            )
        }
        _ => return Err(EvalError::InvalidHandle),
    })
}

fn base64_quad_bytes(indices: [usize; 4], linelen: usize, outcol: &mut usize) -> Vec<u8> {
    let mut out = Vec::with_capacity(5);
    if linelen != 0 && *outcol + 4 > linelen {
        out.push(b'\n');
        *outcol = 0;
    }
    for index in indices {
        out.push(BASE64_ALPHABET[index]);
    }
    *outcol += 4;
    out
}

const LZ77_MAXWIN: usize = 8_192;
const LZ77_MAXLEN: usize = 9 + 255;
const LZ77_MINMATCH: usize = 3;
const LZ77_MINOFFS: usize = 1;
const LZ77_MAXLIT: usize = 32;
const LZ77_HASHBITS: usize = 11;
const LZ77_HASHSIZE: usize = 1 << LZ77_HASHBITS;
const LZ77_NUMPREV: usize = 16;

fn lz77_decompress(src: &[u8]) -> Result<Vec<u8>, EvalError> {
    let mut out = Vec::with_capacity(100_000);
    let mut src_pos = 0;
    while src_pos < src.len() {
        let op = src[src_pos];
        src_pos += 1;
        let opx = usize::from(op & 0x1f);
        let op = op >> 5;
        if op == 0 {
            let len = opx.checked_add(1).ok_or(EvalError::Overflow)?;
            let end = src_pos.checked_add(len).ok_or(EvalError::Overflow)?;
            let bytes = src.get(src_pos..end).ok_or(EvalError::InvalidByteString)?;
            out.extend_from_slice(bytes);
            src_pos = end;
        } else {
            let lo = *src.get(src_pos).ok_or(EvalError::InvalidByteString)?;
            src_pos += 1;
            let offs = opx
                .checked_mul(256)
                .and_then(|offs| offs.checked_add(usize::from(lo)))
                .and_then(|offs| offs.checked_add(LZ77_MINOFFS))
                .ok_or(EvalError::Overflow)?;
            let len = if op == 7 {
                let extra = *src.get(src_pos).ok_or(EvalError::InvalidByteString)?;
                src_pos += 1;
                9usize
                    .checked_add(usize::from(extra))
                    .ok_or(EvalError::Overflow)?
            } else {
                2usize
                    .checked_add(usize::from(op))
                    .ok_or(EvalError::Overflow)?
            };
            if offs > out.len() {
                return Err(EvalError::InvalidByteString);
            }
            let start = out.len() - offs;
            for idx in 0..len {
                let byte = *out.get(start + idx).ok_or(EvalError::InvalidByteString)?;
                out.push(byte);
            }
        }
    }
    Ok(out)
}

fn lz77_compress(src: &[u8]) -> Result<Vec<u8>, EvalError> {
    let mut out = Vec::with_capacity(25_000);
    let mut hashes = [[usize::MAX; LZ77_NUMPREV]; LZ77_HASHSIZE];
    let mut cur = 0;
    while cur < src.len() {
        let mut match_offs = 0;
        let mut match_len = 0;
        let mut len = 0;
        while len < src.len() - cur {
            (match_len, match_offs) = lz77_find_longest_match(src, cur + len, &hashes);
            if match_len >= LZ77_MINMATCH {
                break;
            }
            lz77_update_hash(src, cur + len, &mut hashes);
            len += 1;
        }
        while len != 0 {
            let n = len.min(LZ77_MAXLIT);
            out.push(u8::try_from(n - 1).map_err(|_| EvalError::Overflow)?);
            out.extend_from_slice(&src[cur..cur + n]);
            cur += n;
            len -= n;
        }
        if match_len >= LZ77_MINMATCH {
            for _ in 0..match_len {
                lz77_update_hash(src, cur, &mut hashes);
                cur += 1;
            }
            let match_offs = match_offs
                .checked_sub(LZ77_MINOFFS)
                .ok_or(EvalError::Overflow)?;
            let match_len = match_len.checked_sub(2).ok_or(EvalError::Overflow)?;
            let hi = match_offs >> 8;
            let lo = match_offs & 0xff;
            if match_len < 7 {
                out.push(u8::try_from((match_len << 5) + hi).map_err(|_| EvalError::Overflow)?);
                out.push(u8::try_from(lo).map_err(|_| EvalError::Overflow)?);
            } else {
                out.push(u8::try_from((7 << 5) + hi).map_err(|_| EvalError::Overflow)?);
                out.push(u8::try_from(lo).map_err(|_| EvalError::Overflow)?);
                out.push(u8::try_from(match_len - 7).map_err(|_| EvalError::Overflow)?);
            }
        }
    }
    Ok(out)
}

fn lz77_hash(bytes: &[u8]) -> usize {
    let mut hash = 5381usize;
    for byte in bytes.iter().take(4) {
        hash = hash.wrapping_mul(33).wrapping_add(usize::from(*byte));
    }
    hash & (LZ77_HASHSIZE - 1)
}

fn lz77_find_longest_match(
    src: &[u8],
    cur: usize,
    hashes: &[[usize; LZ77_NUMPREV]; LZ77_HASHSIZE],
) -> (usize, usize) {
    let win_end = cur + 1;
    let win_len = win_end.min(LZ77_MAXWIN);
    let offsets = &hashes[lz77_hash(&src[cur..])];
    let mut match_len = 0;
    let mut match_offs = 0;
    for offset in offsets {
        if *offset == usize::MAX {
            break;
        }
        if *offset > cur {
            break;
        }
        let offs = cur - *offset;
        if !(LZ77_MINOFFS..win_len).contains(&offs) {
            break;
        }
        let len = lz77_match_len(src, cur, cur - offs);
        if len > match_len {
            match_len = len;
            match_offs = offs;
        }
    }
    (match_len, match_offs)
}

fn lz77_match_len(src: &[u8], cur: usize, win: usize) -> usize {
    let mut len = 0;
    while cur + len < src.len() && len < LZ77_MAXLEN && src[cur + len] == src[win + len] {
        len += 1;
    }
    len
}

fn lz77_update_hash(src: &[u8], cur: usize, hashes: &mut [[usize; LZ77_NUMPREV]; LZ77_HASHSIZE]) {
    let slots = &mut hashes[lz77_hash(&src[cur..])];
    for idx in (1..LZ77_NUMPREV).rev() {
        slots[idx] = slots[idx - 1];
    }
    slots[0] = cur;
}

fn u32_le_bytes(n: usize) -> Result<[u8; 4], EvalError> {
    let n = u32::try_from(n).map_err(|_| EvalError::Overflow)?;
    Ok(n.to_le_bytes())
}

fn u64_le_bytes(n: usize) -> Result<[u8; 8], EvalError> {
    let n = u64::try_from(n).map_err(|_| EvalError::Overflow)?;
    Ok(n.to_le_bytes())
}

fn lzma_compress_payload(input: &[u8]) -> Result<Vec<u8>, EvalError> {
    let props = lzma_sdk_rs::LzmaProps::for_level(5, u32::MAX);
    let raw = lzma_sdk_rs::encode(input, &props);
    let mut out = Vec::with_capacity(13 + raw.len());
    out.extend_from_slice(&lzma_sdk_rs::decoder_props(&props));
    out.extend_from_slice(&u64_le_bytes(input.len())?);
    out.extend_from_slice(&raw);
    Ok(out)
}

fn lzma_decompress_payload(input: &[u8]) -> Result<Vec<u8>, EvalError> {
    if input.len() < 13 {
        return Err(EvalError::InvalidByteString);
    }
    let props: [u8; 5] = input[..5]
        .try_into()
        .map_err(|_| EvalError::InvalidByteString)?;
    let out_len = u64::from_le_bytes(
        input[5..13]
            .try_into()
            .map_err(|_| EvalError::InvalidByteString)?,
    );
    let out_len = usize::try_from(out_len).map_err(|_| EvalError::Overflow)?;
    std::panic::catch_unwind(|| lzma_sdk_rs::decode_raw(&input[13..], &props, out_len))
        .map_err(|_| EvalError::InvalidByteString)
}

fn bwt_encode(data: &[u8]) -> Result<(usize, Vec<u8>), EvalError> {
    if data.is_empty() {
        return Ok((0, Vec::new()));
    }
    let mut rotations: Vec<usize> = (0..data.len()).collect();
    rotations.sort_by(|a, b| bwt_compare_rotation(data, *a, *b));
    let mut zero = 0;
    let mut last = Vec::with_capacity(data.len());
    for (idx, offset) in rotations.into_iter().enumerate() {
        last.push(data[(offset + data.len() - 1) % data.len()]);
        if offset == 0 {
            zero = idx;
        }
    }
    Ok((zero, last))
}

fn bwt_compare_rotation(data: &[u8], a: usize, b: usize) -> Ordering {
    if a == b {
        return Ordering::Equal;
    }
    for offset in 0..data.len() {
        let left = data[(a + offset) % data.len()];
        let right = data[(b + offset) % data.len()];
        match left.cmp(&right) {
            Ordering::Equal => {}
            ordering => return ordering,
        }
    }
    Ordering::Equal
}

fn bwt_decode(data: &[u8], zero: usize) -> Result<Vec<u8>, EvalError> {
    if data.is_empty() {
        if zero == 0 {
            return Ok(Vec::new());
        }
        return Err(EvalError::InvalidByteString);
    }
    if zero >= data.len() {
        return Err(EvalError::InvalidByteString);
    }
    let mut count = [0usize; 256];
    let mut pred = Vec::with_capacity(data.len());
    for byte in data {
        let index = usize::from(*byte);
        pred.push(count[index]);
        count[index] = count[index].checked_add(1).ok_or(EvalError::Overflow)?;
    }
    let mut sum = 0usize;
    for item in &mut count {
        let previous = *item;
        *item = sum;
        sum = sum.checked_add(previous).ok_or(EvalError::Overflow)?;
    }
    let mut out = vec![0; data.len()];
    let mut index = zero;
    for pos in (0..data.len()).rev() {
        let byte = data[index];
        out[pos] = byte;
        index = pred[index]
            .checked_add(count[usize::from(byte)])
            .ok_or(EvalError::Overflow)?;
        if pos != 0 && index >= data.len() {
            return Err(EvalError::InvalidByteString);
        }
    }
    Ok(out)
}

fn int_to_i32(n: i64) -> Result<i32, EvalError> {
    i32::try_from(n).map_err(|_| EvalError::Overflow)
}

fn validate_js_tags(tags: &[u8]) -> Result<(), EvalError> {
    if tags.is_empty() {
        return Err(EvalError::InvalidByteString);
    }
    for (idx, tag) in tags.iter().copied().enumerate() {
        let ok = matches!(tag, b'I' | b'U' | b'D' | b'F' | b'P' | b'B' | b'J' | b'S')
            || (idx == 0 && tag == b'V');
        if !ok {
            return Err(EvalError::InvalidByteString);
        }
    }
    Ok(())
}

fn host_js_debug(bytes: &[u8]) -> Result<(), EvalError> {
    #[cfg(target_arch = "wasm32")]
    {
        let bytes = nul_terminated(bytes)?;
        unsafe {
            mhs_js_debug(bytes.as_ptr());
        }
        Ok(())
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = bytes;
        Err(EvalError::UnsupportedJsFfi)
    }
}

fn host_js_eval_run(bytes: &[u8]) -> Result<(), EvalError> {
    #[cfg(target_arch = "wasm32")]
    {
        let bytes = nul_terminated(bytes)?;
        unsafe {
            mhs_js_eval_run(bytes.as_ptr());
        }
        Ok(())
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = bytes;
        Err(EvalError::UnsupportedJsFfi)
    }
}

fn host_js_eval_call(bytes: &[u8]) -> Result<Vec<u8>, EvalError> {
    #[cfg(target_arch = "wasm32")]
    {
        let bytes = nul_terminated(bytes)?;
        unsafe {
            let ptr = mhs_js_eval_call(bytes.as_ptr());
            copy_host_c_string(ptr)
        }
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = bytes;
        Err(EvalError::UnsupportedJsFfi)
    }
}

fn host_js_set_haskell_callback(callback: i32) -> Result<(), EvalError> {
    #[cfg(target_arch = "wasm32")]
    {
        unsafe {
            mhs_js_set_haskellCallback(callback);
        }
        Ok(())
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = callback;
        Err(EvalError::UnsupportedJsFfi)
    }
}

fn host_js_call_void(body: &[u8], arity: usize, args: &[JsArg]) -> Result<(), EvalError> {
    #[cfg(target_arch = "wasm32")]
    {
        let idx = host_js_prepare_call(body, arity, args)?;
        unsafe {
            mhs_js_call_void(idx);
        }
        host_js_check_error()
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = (body, arity, args);
        Err(EvalError::UnsupportedJsFfi)
    }
}

fn host_js_call_int(body: &[u8], arity: usize, args: &[JsArg]) -> Result<i32, EvalError> {
    #[cfg(target_arch = "wasm32")]
    {
        let idx = host_js_prepare_call(body, arity, args)?;
        let result = unsafe { mhs_js_call_int(idx) };
        host_js_check_error()?;
        Ok(result)
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = (body, arity, args);
        Err(EvalError::UnsupportedJsFfi)
    }
}

fn host_js_call_uint(body: &[u8], arity: usize, args: &[JsArg]) -> Result<u32, EvalError> {
    #[cfg(target_arch = "wasm32")]
    {
        let idx = host_js_prepare_call(body, arity, args)?;
        let result = unsafe { mhs_js_call_uint(idx) };
        host_js_check_error()?;
        Ok(result)
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = (body, arity, args);
        Err(EvalError::UnsupportedJsFfi)
    }
}

fn host_js_call_double(body: &[u8], arity: usize, args: &[JsArg]) -> Result<f64, EvalError> {
    #[cfg(target_arch = "wasm32")]
    {
        let idx = host_js_prepare_call(body, arity, args)?;
        let result = unsafe { mhs_js_call_dbl(idx) };
        host_js_check_error()?;
        Ok(result)
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = (body, arity, args);
        Err(EvalError::UnsupportedJsFfi)
    }
}

fn host_js_call_ptr(body: &[u8], arity: usize, args: &[JsArg]) -> Result<u32, EvalError> {
    #[cfg(target_arch = "wasm32")]
    {
        let idx = host_js_prepare_call(body, arity, args)?;
        let result = unsafe { mhs_js_call_ptr(idx) };
        host_js_check_error()?;
        Ok(result)
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = (body, arity, args);
        Err(EvalError::UnsupportedJsFfi)
    }
}

fn host_js_call_object(body: &[u8], arity: usize, args: &[JsArg]) -> Result<u32, EvalError> {
    #[cfg(target_arch = "wasm32")]
    {
        let idx = host_js_prepare_call(body, arity, args)?;
        let result = unsafe { mhs_js_call_obj(idx) };
        host_js_check_error()?;
        Ok(result)
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = (body, arity, args);
        Err(EvalError::UnsupportedJsFfi)
    }
}

fn host_js_call_bool(body: &[u8], arity: usize, args: &[JsArg]) -> Result<bool, EvalError> {
    #[cfg(target_arch = "wasm32")]
    {
        let idx = host_js_prepare_call(body, arity, args)?;
        let result = unsafe { mhs_js_call_bool(idx) != 0 };
        host_js_check_error()?;
        Ok(result)
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = (body, arity, args);
        Err(EvalError::UnsupportedJsFfi)
    }
}

fn host_js_call_string(body: &[u8], arity: usize, args: &[JsArg]) -> Result<Vec<u8>, EvalError> {
    #[cfg(target_arch = "wasm32")]
    {
        let idx = host_js_prepare_call(body, arity, args)?;
        unsafe {
            let ptr = mhs_js_call_str(idx);
            let len = usize::try_from(mhs_js_slen()).map_err(|_| EvalError::Overflow)?;
            let result = copy_host_bytes(ptr, len)?;
            host_js_check_error()?;
            Ok(result)
        }
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = (body, arity, args);
        Err(EvalError::UnsupportedJsFfi)
    }
}

fn host_js_make_wrapper(
    program_handle: u32,
    stable_ptr: i64,
    wrapper_index: u32,
) -> Result<u32, EvalError> {
    #[cfg(target_arch = "wasm32")]
    {
        let stable_ptr = u32::try_from(stable_ptr).map_err(|_| EvalError::Overflow)?;
        let result = unsafe { mhs_js_make_wrapper(program_handle, stable_ptr, wrapper_index) };
        host_js_check_error()?;
        Ok(result)
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = (program_handle, stable_ptr, wrapper_index);
        Err(EvalError::UnsupportedJsFfi)
    }
}

#[cfg(target_arch = "wasm32")]
fn host_js_prepare_call(body: &[u8], arity: usize, args: &[JsArg]) -> Result<i32, EvalError> {
    let body = nul_terminated(body)?;
    let arity = i32::try_from(arity).map_err(|_| EvalError::Overflow)?;
    unsafe {
        mhs_js_setup();
        let idx = mhs_js_register(body.as_ptr(), arity);
        mhs_js_argreset();
        for arg in args {
            match arg {
                JsArg::Int(value) => mhs_js_push_int(*value),
                JsArg::UInt(value) => mhs_js_push_uint(*value),
                JsArg::Double(value) => mhs_js_push_dbl(*value),
                JsArg::Object(value) => mhs_js_push_obj(*value),
                JsArg::String(bytes) => {
                    let len = i32::try_from(bytes.len()).map_err(|_| EvalError::Overflow)?;
                    mhs_js_push_str(bytes.as_ptr(), len);
                }
            }
        }
        Ok(idx)
    }
}

#[cfg(target_arch = "wasm32")]
fn host_js_check_error() -> Result<(), EvalError> {
    unsafe {
        if mhs_js_haserr() != 0 {
            mhs_js_logerr();
            return Err(EvalError::UnsupportedJsFfi);
        }
    }
    Ok(())
}

#[cfg(target_arch = "wasm32")]
fn nul_terminated(bytes: &[u8]) -> Result<Vec<u8>, EvalError> {
    if bytes.contains(&0) {
        return Err(EvalError::InvalidByteString);
    }
    let mut out = Vec::with_capacity(bytes.len() + 1);
    out.extend_from_slice(bytes);
    out.push(0);
    Ok(out)
}

#[cfg(target_arch = "wasm32")]
unsafe fn copy_host_c_string(ptr: *const std::os::raw::c_char) -> Result<Vec<u8>, EvalError> {
    if ptr.is_null() {
        return Ok(Vec::new());
    }
    Ok(unsafe { std::ffi::CStr::from_ptr(ptr) }.to_bytes().to_vec())
}

#[cfg(target_arch = "wasm32")]
unsafe fn copy_host_bytes(
    ptr: *const std::os::raw::c_char,
    len: usize,
) -> Result<Vec<u8>, EvalError> {
    if ptr.is_null() {
        return if len == 0 {
            Ok(Vec::new())
        } else {
            Err(EvalError::InvalidByteString)
        };
    }
    Ok(unsafe { std::slice::from_raw_parts(ptr.cast::<u8>(), len) }.to_vec())
}

#[cfg(target_arch = "wasm32")]
unsafe extern "C" {
    fn mhs_js_debug(ptr: *const u8);
    fn mhs_js_eval_run(ptr: *const u8);
    fn mhs_js_eval_call(ptr: *const u8) -> *const std::os::raw::c_char;
    fn mhs_js_set_haskellCallback(callback: i32);
    fn mhs_js_setup();
    fn mhs_js_register(body: *const u8, arity: i32) -> i32;
    fn mhs_js_argreset();
    fn mhs_js_push_int(value: i32);
    fn mhs_js_push_uint(value: u32);
    fn mhs_js_push_dbl(value: f64);
    fn mhs_js_push_obj(handle: u32);
    fn mhs_js_push_str(ptr: *const u8, len: i32);
    fn mhs_js_call_int(idx: i32) -> i32;
    fn mhs_js_call_uint(idx: i32) -> u32;
    fn mhs_js_call_dbl(idx: i32) -> f64;
    fn mhs_js_call_ptr(idx: i32) -> u32;
    fn mhs_js_call_obj(idx: i32) -> u32;
    fn mhs_js_call_bool(idx: i32) -> i32;
    fn mhs_js_call_str(idx: i32) -> *const std::os::raw::c_char;
    fn mhs_js_call_void(idx: i32);
    fn mhs_js_make_wrapper(program_handle: u32, stable_ptr: u32, wrapper_index: u32) -> u32;
    fn mhs_js_slen() -> i32;
    fn mhs_js_haserr() -> i32;
    fn mhs_js_logerr();
}

const MPZ_BASE: u32 = 1_000_000_000;

#[derive(Clone, Debug, Eq, PartialEq)]
struct MpzValue {
    negative: bool,
    digits: Vec<u32>,
}

impl MpzValue {
    fn zero() -> Self {
        Self {
            negative: false,
            digits: Vec::new(),
        }
    }

    fn one() -> Self {
        Self {
            negative: false,
            digits: vec![1],
        }
    }

    fn from_u64(mut value: u64) -> Self {
        let mut digits = Vec::new();
        let base = u64::from(MPZ_BASE);
        while value != 0 {
            digits.push((value % base) as u32);
            value /= base;
        }
        Self {
            negative: false,
            digits,
        }
    }

    fn from_i64(value: i64) -> Self {
        let mut out = Self::from_u64(value.unsigned_abs());
        out.negative = value < 0 && !out.is_zero();
        out
    }

    fn parse_decimal(bytes: &[u8]) -> Result<Self, ()> {
        let (negative, digits) = match bytes {
            [b'-', rest @ ..] => (true, rest),
            [b'+', rest @ ..] => (false, rest),
            rest => (false, rest),
        };
        if digits.is_empty() {
            return Err(());
        }
        let mut value = Self::zero();
        for &byte in digits {
            if !byte.is_ascii_digit() {
                return Err(());
            }
            value.mul_small_mut(10);
            value.add_small_mut(u32::from(byte - b'0'));
        }
        value.negative = negative && !value.is_zero();
        Ok(value)
    }

    fn to_decimal_bytes(&self) -> Vec<u8> {
        if self.is_zero() {
            return b"0".to_vec();
        }
        let mut out = Vec::new();
        if self.negative {
            out.push(b'-');
        }
        let mut digits = self.digits.iter().rev();
        if let Some(first) = digits.next() {
            out.extend(first.to_string().into_bytes());
        }
        for digit in digits {
            out.extend(format!("{digit:09}").into_bytes());
        }
        out
    }

    fn normalize(&mut self) {
        while self.digits.last() == Some(&0) {
            self.digits.pop();
        }
        if self.digits.is_empty() {
            self.negative = false;
        }
    }

    fn normalized(mut self) -> Self {
        self.normalize();
        self
    }

    fn is_zero(&self) -> bool {
        self.digits.is_empty()
    }

    fn abs(&self) -> Self {
        let mut out = self.clone();
        out.negative = false;
        out
    }

    fn cmp_abs(&self, other: &Self) -> Ordering {
        match self.digits.len().cmp(&other.digits.len()) {
            Ordering::Equal => {
                for (left, right) in self.digits.iter().rev().zip(other.digits.iter().rev()) {
                    match left.cmp(right) {
                        Ordering::Equal => {}
                        ordering => return ordering,
                    }
                }
                Ordering::Equal
            }
            ordering => ordering,
        }
    }

    fn cmp(&self, other: &Self) -> Ordering {
        match (self.negative, other.negative) {
            (true, false) => Ordering::Less,
            (false, true) => Ordering::Greater,
            (false, false) => self.cmp_abs(other),
            (true, true) => other.cmp_abs(self),
        }
    }

    fn abs_add(&self, other: &Self) -> Self {
        let mut out = Vec::with_capacity(self.digits.len().max(other.digits.len()) + 1);
        let mut carry = 0_u64;
        let base = u64::from(MPZ_BASE);
        let len = self.digits.len().max(other.digits.len());
        for idx in 0..len {
            let left = u64::from(*self.digits.get(idx).unwrap_or(&0));
            let right = u64::from(*other.digits.get(idx).unwrap_or(&0));
            let sum = left + right + carry;
            out.push((sum % base) as u32);
            carry = sum / base;
        }
        if carry != 0 {
            out.push(carry as u32);
        }
        Self {
            negative: false,
            digits: out,
        }
        .normalized()
    }

    fn abs_sub(&self, other: &Self) -> Self {
        debug_assert!(self.cmp_abs(other) != Ordering::Less);
        let mut out = Vec::with_capacity(self.digits.len());
        let mut borrow = 0_i64;
        let base = i64::from(MPZ_BASE);
        for idx in 0..self.digits.len() {
            let left = i64::from(self.digits[idx]) - borrow;
            let right = i64::from(*other.digits.get(idx).unwrap_or(&0));
            if left < right {
                out.push((left + base - right) as u32);
                borrow = 1;
            } else {
                out.push((left - right) as u32);
                borrow = 0;
            }
        }
        Self {
            negative: false,
            digits: out,
        }
        .normalized()
    }

    fn add(&self, other: &Self) -> Self {
        if self.negative == other.negative {
            let mut out = self.abs_add(other);
            out.negative = self.negative && !out.is_zero();
            return out;
        }
        match self.cmp_abs(other) {
            Ordering::Greater => {
                let mut out = self.abs_sub(other);
                out.negative = self.negative && !out.is_zero();
                out
            }
            Ordering::Less => {
                let mut out = other.abs_sub(self);
                out.negative = other.negative && !out.is_zero();
                out
            }
            Ordering::Equal => Self::zero(),
        }
    }

    fn sub(&self, other: &Self) -> Self {
        let mut neg_other = other.clone();
        if !neg_other.is_zero() {
            neg_other.negative = !neg_other.negative;
        }
        self.add(&neg_other)
    }

    fn mul(&self, other: &Self) -> Self {
        if self.is_zero() || other.is_zero() {
            return Self::zero();
        }
        let base = u64::from(MPZ_BASE);
        let mut out = vec![0_u64; self.digits.len() + other.digits.len()];
        for (i, &left) in self.digits.iter().enumerate() {
            let mut carry = 0_u64;
            for (j, &right) in other.digits.iter().enumerate() {
                let idx = i + j;
                let raw = out[idx] + u64::from(left) * u64::from(right) + carry;
                out[idx] = raw % base;
                carry = raw / base;
            }
            if carry != 0 {
                out[i + other.digits.len()] += carry;
            }
        }
        let mut digits = Vec::with_capacity(out.len());
        let mut carry = 0_u64;
        for raw in out {
            let raw = raw + carry;
            digits.push((raw % base) as u32);
            carry = raw / base;
        }
        while carry != 0 {
            digits.push((carry % base) as u32);
            carry /= base;
        }
        Self {
            negative: self.negative != other.negative,
            digits,
        }
        .normalized()
    }

    fn mul_small_mut(&mut self, value: u32) {
        if self.is_zero() || value == 1 {
            return;
        }
        if value == 0 {
            self.digits.clear();
            self.negative = false;
            return;
        }
        let base = u64::from(MPZ_BASE);
        let mut carry = 0_u64;
        for digit in &mut self.digits {
            let raw = u64::from(*digit) * u64::from(value) + carry;
            *digit = (raw % base) as u32;
            carry = raw / base;
        }
        while carry != 0 {
            self.digits.push((carry % base) as u32);
            carry /= base;
        }
    }

    fn add_small_mut(&mut self, value: u32) {
        if value == 0 {
            return;
        }
        let base = u64::from(MPZ_BASE);
        let mut carry = u64::from(value);
        let mut idx = 0;
        while carry != 0 {
            if idx == self.digits.len() {
                self.digits.push(0);
            }
            let raw = u64::from(self.digits[idx]) + carry;
            self.digits[idx] = (raw % base) as u32;
            carry = raw / base;
            idx += 1;
        }
    }

    fn div2_mut(&mut self) -> bool {
        let mut rem = 0_u64;
        let base = u64::from(MPZ_BASE);
        for digit in self.digits.iter_mut().rev() {
            let raw = rem * base + u64::from(*digit);
            *digit = (raw / 2) as u32;
            rem = raw % 2;
        }
        self.normalize();
        rem != 0
    }

    fn shl1_mut(&mut self) {
        self.mul_small_mut(2);
    }

    fn shl_bits(mut self, bits: usize) -> Self {
        for _ in 0..bits {
            self.shl1_mut();
        }
        self
    }

    fn shr_abs_bits(&self, bits: usize) -> (Self, bool) {
        let mut out = self.abs();
        let mut dropped = false;
        for _ in 0..bits {
            dropped |= out.div2_mut();
        }
        (out, dropped)
    }

    fn fdiv_q_2exp(&self, bits: usize) -> Self {
        let (mut quot, dropped) = self.shr_abs_bits(bits);
        if self.negative {
            if dropped {
                quot.add_small_mut(1);
            }
            if !quot.is_zero() {
                quot.negative = true;
            }
        }
        quot
    }

    fn to_bits_abs(&self) -> Vec<bool> {
        let mut tmp = self.abs();
        let mut bits = Vec::new();
        while !tmp.is_zero() {
            bits.push(tmp.div2_mut());
        }
        bits
    }

    fn from_bits_abs(bits: &[bool]) -> Self {
        let mut out = Self::zero();
        for bit in bits.iter().rev() {
            out.shl1_mut();
            if *bit {
                out.add_small_mut(1);
            }
        }
        out
    }

    fn one_shl(bits: usize) -> Self {
        Self::one().shl_bits(bits)
    }

    fn div_rem_abs(&self, divisor: &Self) -> Result<(Self, Self), EvalError> {
        if divisor.is_zero() {
            return Err(EvalError::InvalidByteString);
        }
        if self.cmp_abs(divisor) == Ordering::Less {
            return Ok((Self::zero(), self.abs()));
        }
        let bits = self.to_bits_abs();
        let mut quot = Self::zero();
        let mut rem = Self::zero();
        for bit in bits.iter().rev() {
            rem.shl1_mut();
            if *bit {
                rem.add_small_mut(1);
            }
            quot.shl1_mut();
            if rem.cmp_abs(divisor) != Ordering::Less {
                rem = rem.abs_sub(divisor);
                quot.add_small_mut(1);
            }
        }
        Ok((quot, rem))
    }

    fn tdiv_qr(&self, divisor: &Self) -> Result<(Self, Self), EvalError> {
        let (mut quot, mut rem) = self.abs().div_rem_abs(&divisor.abs())?;
        quot.negative = self.negative != divisor.negative && !quot.is_zero();
        rem.negative = self.negative && !rem.is_zero();
        Ok((quot, rem))
    }

    fn bit_len(&self) -> usize {
        self.to_bits_abs().len()
    }

    fn to_twos_bits(&self, width: usize) -> Vec<bool> {
        let mut bits = if self.negative {
            Self::one_shl(width).sub(&self.abs()).to_bits_abs()
        } else {
            self.to_bits_abs()
        };
        bits.resize(width, false);
        bits
    }

    fn from_twos_bits(bits: &[bool]) -> Self {
        if bits.last() != Some(&true) {
            return Self::from_bits_abs(bits);
        }
        let unsigned = Self::from_bits_abs(bits);
        let mut out = Self::one_shl(bits.len()).sub(&unsigned);
        if !out.is_zero() {
            out.negative = true;
        }
        out
    }

    fn bitwise(&self, other: &Self, op: fn(bool, bool) -> bool) -> Self {
        let width = self.bit_len().max(other.bit_len()) + 1;
        let left = self.to_twos_bits(width);
        let right = other.to_twos_bits(width);
        let bits: Vec<bool> = left
            .into_iter()
            .zip(right)
            .map(|(left, right)| op(left, right))
            .collect();
        Self::from_twos_bits(&bits)
    }

    fn bitand(&self, other: &Self) -> Self {
        self.bitwise(other, |left, right| left & right)
    }

    fn bitor(&self, other: &Self) -> Self {
        self.bitwise(other, |left, right| left | right)
    }

    fn bitxor(&self, other: &Self) -> Self {
        self.bitwise(other, |left, right| left ^ right)
    }

    fn test_bit_abs(&self, bit: usize) -> bool {
        self.to_bits_abs().get(bit).copied().unwrap_or(false)
    }

    fn test_bit_signed(&self, bit: usize) -> bool {
        if !self.negative {
            return self.test_bit_abs(bit);
        }
        let shifted = self.fdiv_q_2exp(bit);
        shifted.abs().test_bit_abs(0)
    }

    fn signed_popcount(&self) -> Result<i64, EvalError> {
        let count = i64::try_from(self.to_bits_abs().into_iter().filter(|bit| *bit).count())
            .map_err(|_| EvalError::Overflow)?;
        Ok(if self.negative { -count } else { count })
    }

    fn log2(&self) -> Result<i64, EvalError> {
        i64::try_from(self.bit_len().saturating_sub(1)).map_err(|_| EvalError::Overflow)
    }

    fn to_u64_low(&self) -> u64 {
        let mut out = 0_u64;
        for (idx, bit) in self.to_bits_abs().into_iter().take(64).enumerate() {
            if bit {
                out |= 1_u64 << idx;
            }
        }
        out
    }

    fn to_i64_wrapping(&self) -> i64 {
        let low = self.to_u64_low();
        if self.negative {
            0_u64.wrapping_sub(low) as i64
        } else {
            low as i64
        }
    }

    #[cold]
    #[inline(never)]
    fn to_f64(&self) -> f64 {
        let decimal = self.to_decimal_bytes();
        mpz_decimal_to_f64(&decimal)
    }
}

#[cold]
#[inline(never)]
fn mpz_decimal_to_f64(decimal: &[u8]) -> f64 {
    let text = std::str::from_utf8(decimal).expect("mpz decimal bytes are ASCII");
    text.parse::<f64>()
        .expect("mpz decimal bytes should parse as f64")
}

fn size_of_i64<T>() -> i64 {
    std::mem::size_of::<T>() as i64
}

fn format_float(value: f64) -> String {
    let mut out = value.to_string();
    if out == "NaN" {
        out = "nan".to_owned();
    }
    if out != "nan"
        && out != "-nan"
        && out != "inf"
        && out != "-inf"
        && !out.contains('.')
        && !out.contains('e')
        && !out.contains('E')
    {
        out.push_str(".0");
    }
    out
}

fn current_time_micro() -> i64 {
    #[cfg(target_arch = "wasm32")]
    {
        0
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        use std::time::{SystemTime, UNIX_EPOCH};

        let Ok(duration) = SystemTime::now().duration_since(UNIX_EPOCH) else {
            return 0;
        };
        i64::try_from(duration.as_micros()).unwrap_or(i64::MAX)
    }
}

#[cfg(not(any(unix, target_arch = "wasm32")))]
fn current_time_nanos() -> u64 {
    use std::time::{SystemTime, UNIX_EPOCH};

    let Ok(duration) = SystemTime::now().duration_since(UNIX_EPOCH) else {
        return 0;
    };
    u64::try_from(duration.as_nanos()).unwrap_or(u64::MAX)
}

#[cfg(all(
    any(target_os = "linux", target_os = "android"),
    not(target_arch = "wasm32")
))]
fn cpu_time() -> (u64, u64) {
    let mut ts = std::mem::MaybeUninit::<libc::timespec>::uninit();
    // SAFETY: clock_gettime writes the timespec on success. The pointer is valid for one call.
    let rc = unsafe { libc::clock_gettime(libc::CLOCK_PROCESS_CPUTIME_ID, ts.as_mut_ptr()) };
    if rc != 0 {
        return (0, 0);
    }
    // SAFETY: the call above succeeded, so the timespec has been initialized.
    let ts = unsafe { ts.assume_init() };
    (
        u64::try_from(ts.tv_sec).unwrap_or(0),
        u64::try_from(ts.tv_nsec).unwrap_or(0),
    )
}

#[cfg(any(
    target_arch = "wasm32",
    not(any(target_os = "linux", target_os = "android"))
))]
fn cpu_time() -> (u64, u64) {
    (0, 0)
}

fn errno_i32(name: &str) -> i32 {
    errno_constant(name).unwrap_or(-1) as i32
}

fn host_constant(name: &str) -> Option<i64> {
    #[cfg(all(unix, not(target_arch = "wasm32")))]
    {
        return Some(i64::from(match name {
            "F_SETFL" => libc::F_SETFL,
            "O_NONBLOCK" => libc::O_NONBLOCK,
            "SOL_SOCKET" => libc::SOL_SOCKET,
            "SO_DEBUG" => libc::SO_DEBUG,
            "SO_ERROR" => libc::SO_ERROR,
            "SO_REUSEADDR" => libc::SO_REUSEADDR,
            "SO_TYPE" => libc::SO_TYPE,
            _ => return None,
        }));
    }
    #[cfg(not(all(unix, not(target_arch = "wasm32"))))]
    {
        const HOST_CONSTANTS: &[&str] = &[
            "F_SETFL",
            "O_NONBLOCK",
            "SOL_SOCKET",
            "SO_DEBUG",
            "SO_ERROR",
            "SO_REUSEADDR",
            "SO_TYPE",
        ];
        HOST_CONSTANTS.contains(&name).then_some(-1)
    }
}

#[cfg_attr(target_arch = "wasm32", allow(dead_code))]
fn last_errno() -> i32 {
    std::io::Error::last_os_error()
        .raw_os_error()
        .unwrap_or_else(|| errno_i32("ENOENT"))
}

#[cfg_attr(target_arch = "wasm32", allow(dead_code))]
fn io_error_errno(error: &std::io::Error) -> Option<i32> {
    error.raw_os_error()
}

fn strerror_bytes(errno: i32) -> Vec<u8> {
    std::io::Error::from_raw_os_error(errno)
        .to_string()
        .into_bytes()
}

#[cfg(all(
    any(target_os = "linux", target_os = "android"),
    not(target_arch = "wasm32")
))]
fn errno_constant(name: &str) -> Option<i64> {
    Some(i64::from(match name {
        "EOK" => 0,
        "E2BIG" => libc::E2BIG,
        "EACCES" => libc::EACCES,
        "EADDRINUSE" => libc::EADDRINUSE,
        "EADDRNOTAVAIL" => libc::EADDRNOTAVAIL,
        "EADV" => libc::EADV,
        "EAFNOSUPPORT" => libc::EAFNOSUPPORT,
        "EAGAIN" => libc::EAGAIN,
        "EALREADY" => libc::EALREADY,
        "EBADF" => libc::EBADF,
        "EBADMSG" => libc::EBADMSG,
        "EBADRPC" => -1,
        "EBUSY" => libc::EBUSY,
        "ECHILD" => libc::ECHILD,
        "ECOMM" => libc::ECOMM,
        "ECONNABORTED" => libc::ECONNABORTED,
        "ECONNREFUSED" => libc::ECONNREFUSED,
        "ECONNRESET" => libc::ECONNRESET,
        "EDEADLK" => libc::EDEADLK,
        "EDESTADDRREQ" => libc::EDESTADDRREQ,
        "EDIRTY" => -1,
        "EDOM" => libc::EDOM,
        "EDQUOT" => libc::EDQUOT,
        "EEXIST" => libc::EEXIST,
        "EFAULT" => libc::EFAULT,
        "EFBIG" => libc::EFBIG,
        "EFTYPE" => -1,
        "EHOSTDOWN" => libc::EHOSTDOWN,
        "EHOSTUNREACH" => libc::EHOSTUNREACH,
        "EIDRM" => libc::EIDRM,
        "EILSEQ" => libc::EILSEQ,
        "EINPROGRESS" => libc::EINPROGRESS,
        "EINTR" => libc::EINTR,
        "EINVAL" => libc::EINVAL,
        "EIO" => libc::EIO,
        "EISCONN" => libc::EISCONN,
        "EISDIR" => libc::EISDIR,
        "ELOOP" => libc::ELOOP,
        "EMFILE" => libc::EMFILE,
        "EMLINK" => libc::EMLINK,
        "EMSGSIZE" => libc::EMSGSIZE,
        "EMULTIHOP" => libc::EMULTIHOP,
        "ENAMETOOLONG" => libc::ENAMETOOLONG,
        "ENETDOWN" => libc::ENETDOWN,
        "ENETRESET" => libc::ENETRESET,
        "ENETUNREACH" => libc::ENETUNREACH,
        "ENFILE" => libc::ENFILE,
        "ENOBUFS" => libc::ENOBUFS,
        "ENODATA" => libc::ENODATA,
        "ENODEV" => libc::ENODEV,
        "ENOENT" => libc::ENOENT,
        "ENOEXEC" => libc::ENOEXEC,
        "ENOLCK" => libc::ENOLCK,
        "ENOLINK" => libc::ENOLINK,
        "ENOMEM" => libc::ENOMEM,
        "ENOMSG" => libc::ENOMSG,
        "ENONET" => libc::ENONET,
        "ENOPROTOOPT" => libc::ENOPROTOOPT,
        "ENOSPC" => libc::ENOSPC,
        "ENOSR" => libc::ENOSR,
        "ENOSTR" => libc::ENOSTR,
        "ENOSYS" => libc::ENOSYS,
        "ENOTBLK" => libc::ENOTBLK,
        "ENOTCONN" => libc::ENOTCONN,
        "ENOTDIR" => libc::ENOTDIR,
        "ENOTEMPTY" => libc::ENOTEMPTY,
        "ENOTSOCK" => libc::ENOTSOCK,
        "ENOTSUP" => libc::ENOTSUP,
        "ENOTTY" => libc::ENOTTY,
        "ENXIO" => libc::ENXIO,
        "EOPNOTSUPP" => libc::EOPNOTSUPP,
        "EPERM" => libc::EPERM,
        "EPFNOSUPPORT" => libc::EPFNOSUPPORT,
        "EPIPE" => libc::EPIPE,
        "EPROCLIM" => -1,
        "EPROCUNAVAIL" => -1,
        "EPROGMISMATCH" => -1,
        "EPROGUNAVAIL" => -1,
        "EPROTO" => libc::EPROTO,
        "EPROTONOSUPPORT" => libc::EPROTONOSUPPORT,
        "EPROTOTYPE" => libc::EPROTOTYPE,
        "ERANGE" => libc::ERANGE,
        "EREMCHG" => libc::EREMCHG,
        "EREMOTE" => libc::EREMOTE,
        "EROFS" => libc::EROFS,
        "ERPCMISMATCH" => -1,
        "ERREMOTE" => -1,
        "ESHUTDOWN" => libc::ESHUTDOWN,
        "ESOCKTNOSUPPORT" => libc::ESOCKTNOSUPPORT,
        "ESPIPE" => libc::ESPIPE,
        "ESRCH" => libc::ESRCH,
        "ESRMNT" => libc::ESRMNT,
        "ESTALE" => libc::ESTALE,
        "ETIME" => libc::ETIME,
        "ETIMEDOUT" => libc::ETIMEDOUT,
        "ETOOMANYREFS" => libc::ETOOMANYREFS,
        "ETXTBSY" => libc::ETXTBSY,
        "EUSERS" => libc::EUSERS,
        "EWOULDBLOCK" => libc::EWOULDBLOCK,
        "EXDEV" => libc::EXDEV,
        _ => return None,
    }))
}

#[cfg(any(
    target_arch = "wasm32",
    not(any(target_os = "linux", target_os = "android"))
))]
fn errno_constant(name: &str) -> Option<i64> {
    if name == "EOK" {
        return Some(0);
    }
    const ERRNO_NAMES: &[&str] = &[
        "E2BIG",
        "EACCES",
        "EADDRINUSE",
        "EADDRNOTAVAIL",
        "EADV",
        "EAFNOSUPPORT",
        "EAGAIN",
        "EALREADY",
        "EBADF",
        "EBADMSG",
        "EBADRPC",
        "EBUSY",
        "ECHILD",
        "ECOMM",
        "ECONNABORTED",
        "ECONNREFUSED",
        "ECONNRESET",
        "EDEADLK",
        "EDESTADDRREQ",
        "EDIRTY",
        "EDOM",
        "EDQUOT",
        "EEXIST",
        "EFAULT",
        "EFBIG",
        "EFTYPE",
        "EHOSTDOWN",
        "EHOSTUNREACH",
        "EIDRM",
        "EILSEQ",
        "EINPROGRESS",
        "EINTR",
        "EINVAL",
        "EIO",
        "EISCONN",
        "EISDIR",
        "ELOOP",
        "EMFILE",
        "EMLINK",
        "EMSGSIZE",
        "EMULTIHOP",
        "ENAMETOOLONG",
        "ENETDOWN",
        "ENETRESET",
        "ENETUNREACH",
        "ENFILE",
        "ENOBUFS",
        "ENODATA",
        "ENODEV",
        "ENOENT",
        "ENOEXEC",
        "ENOLCK",
        "ENOLINK",
        "ENOMEM",
        "ENOMSG",
        "ENONET",
        "ENOPROTOOPT",
        "ENOSPC",
        "ENOSR",
        "ENOSTR",
        "ENOSYS",
        "ENOTBLK",
        "ENOTCONN",
        "ENOTDIR",
        "ENOTEMPTY",
        "ENOTSOCK",
        "ENOTSUP",
        "ENOTTY",
        "ENXIO",
        "EOPNOTSUPP",
        "EPERM",
        "EPFNOSUPPORT",
        "EPIPE",
        "EPROCLIM",
        "EPROCUNAVAIL",
        "EPROGMISMATCH",
        "EPROGUNAVAIL",
        "EPROTO",
        "EPROTONOSUPPORT",
        "EPROTOTYPE",
        "ERANGE",
        "EREMCHG",
        "EREMOTE",
        "EROFS",
        "ERPCMISMATCH",
        "ERREMOTE",
        "ESHUTDOWN",
        "ESOCKTNOSUPPORT",
        "ESPIPE",
        "ESRCH",
        "ESRMNT",
        "ESTALE",
        "ETIME",
        "ETIMEDOUT",
        "ETOOMANYREFS",
        "ETXTBSY",
        "EUSERS",
        "EWOULDBLOCK",
        "EXDEV",
    ];
    ERRNO_NAMES.contains(&name).then_some(-1)
}

#[cfg(all(unix, not(target_arch = "wasm32")))]
fn getenv_bytes(name: &[u8]) -> Option<Vec<u8>> {
    use std::ffi::OsStr;
    use std::os::unix::ffi::{OsStrExt, OsStringExt};

    std::env::var_os(OsStr::from_bytes(name)).map(|value| value.into_vec())
}

#[cfg(target_arch = "wasm32")]
fn getenv_bytes(name: &[u8]) -> Option<Vec<u8>> {
    let _ = name;
    None
}

#[cfg(not(any(unix, target_arch = "wasm32")))]
fn getenv_bytes(name: &[u8]) -> Option<Vec<u8>> {
    let name = std::str::from_utf8(name).ok()?;
    std::env::var_os(name).map(|value| value.to_string_lossy().into_owned().into_bytes())
}

#[cfg(all(unix, not(target_arch = "wasm32")))]
fn setenv_bytes(name: &[u8], value: &[u8], overwrite: i64) -> HostIntResult {
    use std::ffi::OsStr;
    use std::os::unix::ffi::OsStrExt;

    if name.is_empty() || name.contains(&b'=') {
        return HostIntResult::err(errno_i32("EINVAL"));
    }
    let name = OsStr::from_bytes(name);
    if overwrite == 0 && std::env::var_os(name).is_some() {
        return HostIntResult::ok(0);
    }
    // SAFETY: MicroHs executes user code on one runtime thread today; this mirrors C's process-global env.
    unsafe {
        std::env::set_var(name, OsStr::from_bytes(value));
    }
    HostIntResult::ok(0)
}

#[cfg(target_arch = "wasm32")]
fn setenv_bytes(name: &[u8], value: &[u8], overwrite: i64) -> HostIntResult {
    let _ = (name, value, overwrite);
    HostIntResult::err(errno_i32("ENOSYS"))
}

#[cfg(not(any(unix, target_arch = "wasm32")))]
fn setenv_bytes(name: &[u8], value: &[u8], overwrite: i64) -> HostIntResult {
    if name.is_empty() || name.contains(&b'=') {
        return HostIntResult::err(errno_i32("EINVAL"));
    }
    let Ok(name) = std::str::from_utf8(name) else {
        return HostIntResult::err(errno_i32("EINVAL"));
    };
    if overwrite == 0 && std::env::var_os(name).is_some() {
        return HostIntResult::ok(0);
    }
    let value = String::from_utf8_lossy(value);
    // SAFETY: MicroHs executes user code on one runtime thread today; this mirrors C's process-global env.
    unsafe {
        std::env::set_var(name, value.as_ref());
    }
    HostIntResult::ok(0)
}

#[cfg(all(unix, not(target_arch = "wasm32")))]
fn unsetenv_bytes(name: &[u8]) -> HostIntResult {
    use std::ffi::OsStr;
    use std::os::unix::ffi::OsStrExt;

    if name.is_empty() || name.contains(&b'=') {
        return HostIntResult::err(errno_i32("EINVAL"));
    }
    // SAFETY: MicroHs executes user code on one runtime thread today; this mirrors C's process-global env.
    unsafe {
        std::env::remove_var(OsStr::from_bytes(name));
    }
    HostIntResult::ok(0)
}

#[cfg(target_arch = "wasm32")]
fn unsetenv_bytes(name: &[u8]) -> HostIntResult {
    let _ = name;
    HostIntResult::err(errno_i32("ENOSYS"))
}

#[cfg(not(any(unix, target_arch = "wasm32")))]
fn unsetenv_bytes(name: &[u8]) -> HostIntResult {
    if name.is_empty() || name.contains(&b'=') {
        return HostIntResult::err(errno_i32("EINVAL"));
    }
    let Ok(name) = std::str::from_utf8(name) else {
        return HostIntResult::err(errno_i32("EINVAL"));
    };
    // SAFETY: MicroHs executes user code on one runtime thread today; this mirrors C's process-global env.
    unsafe {
        std::env::remove_var(name);
    }
    HostIntResult::ok(0)
}

#[cfg(all(unix, not(target_arch = "wasm32")))]
fn environ_bytes() -> Vec<Vec<u8>> {
    use std::os::unix::ffi::OsStringExt;

    std::env::vars_os()
        .map(|(name, value)| {
            let mut bytes = name.into_vec();
            bytes.push(b'=');
            bytes.extend(value.into_vec());
            bytes
        })
        .collect()
}

#[cfg(target_arch = "wasm32")]
fn environ_bytes() -> Vec<Vec<u8>> {
    Vec::new()
}

#[cfg(not(any(unix, target_arch = "wasm32")))]
fn environ_bytes() -> Vec<Vec<u8>> {
    std::env::vars_os()
        .map(|(name, value)| {
            let mut bytes = name.to_string_lossy().into_owned().into_bytes();
            bytes.push(b'=');
            bytes.extend(value.to_string_lossy().into_owned().into_bytes());
            bytes
        })
        .collect()
}

#[cfg(all(unix, not(target_arch = "wasm32")))]
fn remove_path_bytes(path: &[u8]) -> HostIntResult {
    use std::ffi::OsStr;
    use std::os::unix::ffi::OsStrExt;

    let path = std::path::Path::new(OsStr::from_bytes(path));
    match std::fs::remove_file(path) {
        Ok(()) => HostIntResult::ok(0),
        Err(file_err) => match std::fs::remove_dir(path) {
            Ok(()) => HostIntResult::ok(0),
            Err(dir_err) => HostIntResult::os_err(
                io_error_errno(&dir_err).or_else(|| io_error_errno(&file_err)),
            ),
        },
    }
}

#[cfg(target_arch = "wasm32")]
fn remove_path_bytes(path: &[u8]) -> HostIntResult {
    let _ = path;
    HostIntResult::err(errno_i32("ENOSYS"))
}

#[cfg(not(any(unix, target_arch = "wasm32")))]
fn remove_path_bytes(path: &[u8]) -> HostIntResult {
    let Ok(path) = std::str::from_utf8(path) else {
        return HostIntResult::err(errno_i32("EINVAL"));
    };
    match std::fs::remove_file(path) {
        Ok(()) => HostIntResult::ok(0),
        Err(file_err) => match std::fs::remove_dir(path) {
            Ok(()) => HostIntResult::ok(0),
            Err(dir_err) => HostIntResult::os_err(
                io_error_errno(&dir_err).or_else(|| io_error_errno(&file_err)),
            ),
        },
    }
}

#[cfg(all(unix, not(target_arch = "wasm32")))]
fn system_command_bytes(command: Option<&[u8]>) -> i64 {
    use std::ffi::OsStr;
    use std::os::unix::ffi::OsStrExt;
    use std::os::unix::process::ExitStatusExt;

    let Some(command) = command else {
        return 1;
    };
    match std::process::Command::new("/bin/sh")
        .arg("-c")
        .arg(OsStr::from_bytes(command))
        .status()
    {
        Ok(status) => i64::from(status.into_raw()),
        Err(_) => -1,
    }
}

#[cfg(target_arch = "wasm32")]
fn system_command_bytes(command: Option<&[u8]>) -> i64 {
    let _ = command;
    -1
}

#[cfg(not(any(unix, target_arch = "wasm32")))]
fn system_command_bytes(command: Option<&[u8]>) -> i64 {
    let Some(command) = command else {
        return 1;
    };
    let Ok(command) = std::str::from_utf8(command) else {
        return -1;
    };
    match std::process::Command::new("cmd")
        .arg("/C")
        .arg(command)
        .status()
    {
        Ok(status) => status.code().map_or(-1, i64::from),
        Err(_) => -1,
    }
}

#[cfg(all(unix, not(target_arch = "wasm32")))]
fn chdir_path_bytes(path: &[u8]) -> HostIntResult {
    use std::ffi::OsStr;
    use std::os::unix::ffi::OsStrExt;

    let path = std::path::Path::new(OsStr::from_bytes(path));
    match std::env::set_current_dir(path) {
        Ok(()) => HostIntResult::ok(0),
        Err(err) => HostIntResult::os_err(io_error_errno(&err)),
    }
}

#[cfg(target_arch = "wasm32")]
fn chdir_path_bytes(path: &[u8]) -> HostIntResult {
    let _ = path;
    HostIntResult::err(errno_i32("ENOSYS"))
}

#[cfg(not(any(unix, target_arch = "wasm32")))]
fn chdir_path_bytes(path: &[u8]) -> HostIntResult {
    let Ok(path) = std::str::from_utf8(path) else {
        return HostIntResult::err(errno_i32("EINVAL"));
    };
    match std::env::set_current_dir(path) {
        Ok(()) => HostIntResult::ok(0),
        Err(err) => HostIntResult::os_err(io_error_errno(&err)),
    }
}

#[cfg(all(unix, not(target_arch = "wasm32")))]
fn mkdir_path_bytes(path: &[u8], mode: i64) -> HostIntResult {
    use std::ffi::OsStr;
    use std::os::unix::ffi::OsStrExt;
    use std::os::unix::fs::DirBuilderExt;

    let Ok(mode) = u32::try_from(mode) else {
        return HostIntResult::err(errno_i32("EINVAL"));
    };
    let path = std::path::Path::new(OsStr::from_bytes(path));
    match std::fs::DirBuilder::new().mode(mode).create(path) {
        Ok(()) => HostIntResult::ok(0),
        Err(err) => HostIntResult::os_err(io_error_errno(&err)),
    }
}

#[cfg(target_arch = "wasm32")]
fn mkdir_path_bytes(path: &[u8], mode: i64) -> HostIntResult {
    let _ = (path, mode);
    HostIntResult::err(errno_i32("ENOSYS"))
}

#[cfg(not(any(unix, target_arch = "wasm32")))]
fn mkdir_path_bytes(path: &[u8], mode: i64) -> HostIntResult {
    let _ = mode;
    let Ok(path) = std::str::from_utf8(path) else {
        return HostIntResult::err(errno_i32("EINVAL"));
    };
    match std::fs::create_dir(path) {
        Ok(()) => HostIntResult::ok(0),
        Err(err) => HostIntResult::os_err(io_error_errno(&err)),
    }
}

#[cfg(all(unix, not(target_arch = "wasm32")))]
fn current_dir_bytes() -> Result<Vec<u8>, i32> {
    use std::os::unix::ffi::OsStringExt;

    std::env::current_dir()
        .map(|path| path.into_os_string().into_vec())
        .map_err(|err| io_error_errno(&err).unwrap_or_else(|| errno_i32("ENOENT")))
}

#[cfg(target_arch = "wasm32")]
fn current_dir_bytes() -> Result<Vec<u8>, i32> {
    Err(errno_i32("ENOSYS"))
}

#[cfg(not(any(unix, target_arch = "wasm32")))]
fn current_dir_bytes() -> Result<Vec<u8>, i32> {
    std::env::current_dir()
        .map(|path| path.to_string_lossy().into_owned().into_bytes())
        .map_err(|err| io_error_errno(&err).unwrap_or_else(|| errno_i32("ENOENT")))
}

#[cfg(all(unix, not(target_arch = "wasm32")))]
fn executable_path_bytes() -> Result<Vec<u8>, i32> {
    use std::os::unix::ffi::OsStringExt;

    std::env::current_exe()
        .map(|path| path.into_os_string().into_vec())
        .map_err(|err| io_error_errno(&err).unwrap_or_else(|| errno_i32("ENOENT")))
}

#[cfg(target_arch = "wasm32")]
fn executable_path_bytes() -> Result<Vec<u8>, i32> {
    Err(errno_i32("ENOSYS"))
}

#[cfg(not(any(unix, target_arch = "wasm32")))]
fn executable_path_bytes() -> Result<Vec<u8>, i32> {
    std::env::current_exe()
        .map(|path| path.to_string_lossy().into_owned().into_bytes())
        .map_err(|err| io_error_errno(&err).unwrap_or_else(|| errno_i32("ENOENT")))
}

#[cfg(all(unix, not(target_arch = "wasm32")))]
fn tmpname_bytes(pre: &[u8], suf: &[u8]) -> Result<Vec<u8>, i32> {
    use std::ffi::{CString, OsString};
    use std::os::unix::ffi::OsStringExt;
    use std::os::unix::io::RawFd;

    let tmpdir = std::env::var_os("TMPDIR")
        .unwrap_or_else(|| OsString::from("/tmp"))
        .into_vec();
    let mut template = Vec::with_capacity(tmpdir.len() + pre.len() + suf.len() + 8);
    template.extend_from_slice(&tmpdir);
    template.push(b'/');
    template.extend_from_slice(pre);
    template.extend_from_slice(b"XXXXXX");
    template.extend_from_slice(suf);
    template.push(0);
    let suffix_len = std::os::raw::c_int::try_from(suf.len()).map_err(|_| errno_i32("EINVAL"))?;
    let path = CString::from_vec_with_nul(template).map_err(|_| errno_i32("EINVAL"))?;
    let mut bytes = path.into_bytes_with_nul();
    // SAFETY: mkstemps mutates the NUL-terminated template in place and returns a file descriptor.
    let fd: RawFd = unsafe { libc::mkstemps(bytes.as_mut_ptr().cast(), suffix_len) };
    if fd < 0 {
        return Err(last_errno());
    }
    // SAFETY: fd came from mkstemps and is not used after this close.
    unsafe {
        libc::close(fd);
    }
    bytes.pop();
    Ok(bytes)
}

#[cfg(target_arch = "wasm32")]
fn tmpname_bytes(pre: &[u8], suf: &[u8]) -> Result<Vec<u8>, i32> {
    let _ = (pre, suf);
    Err(errno_i32("ENOSYS"))
}

#[cfg(not(any(unix, target_arch = "wasm32")))]
fn tmpname_bytes(pre: &[u8], suf: &[u8]) -> Result<Vec<u8>, i32> {
    let pre = std::str::from_utf8(pre).map_err(|_| errno_i32("EINVAL"))?;
    let suf = std::str::from_utf8(suf).map_err(|_| errno_i32("EINVAL"))?;
    let tmpdir = std::env::temp_dir();
    let seed = current_time_nanos() ^ u64::from(std::process::id());
    for attempt in 0..1024 {
        let mut name = String::with_capacity(pre.len() + 6 + suf.len());
        name.push_str(pre);
        name.push_str(
            std::str::from_utf8(&tmp_six(seed.wrapping_add(attempt))).unwrap_or("XXXXXX"),
        );
        name.push_str(suf);
        let path = tmpdir.join(name);
        match std::fs::OpenOptions::new()
            .read(true)
            .write(true)
            .create_new(true)
            .open(&path)
        {
            Ok(_) => return Ok(path.to_string_lossy().into_owned().into_bytes()),
            Err(err) if err.kind() == std::io::ErrorKind::AlreadyExists => continue,
            Err(err) => return Err(io_error_errno(&err).unwrap_or_else(|| errno_i32("ENOENT"))),
        }
    }
    Err(errno_i32("EEXIST"))
}

#[cfg(not(any(unix, target_arch = "wasm32")))]
fn tmp_six(mut value: u64) -> [u8; 6] {
    const ALPHABET: &[u8; 36] = b"0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZ";
    let mut out = [b'0'; 6];
    for byte in &mut out {
        *byte = ALPHABET[(value % 36) as usize];
        value /= 36;
    }
    out
}

#[cfg(all(unix, not(target_arch = "wasm32")))]
fn get_permissions_path_bytes(path: &[u8]) -> HostIntResult {
    use std::ffi::OsStr;
    use std::os::unix::ffi::OsStrExt;
    use std::os::unix::fs::PermissionsExt;

    let path = std::path::Path::new(OsStr::from_bytes(path));
    let metadata = match std::fs::metadata(path) {
        Ok(metadata) => metadata,
        Err(err) => return HostIntResult::os_err(io_error_errno(&err)),
    };
    let mode = metadata.permissions().mode();
    let mut permissions = 0;
    if mode & 0o400 != 0 {
        permissions |= 4;
    }
    if mode & 0o200 != 0 {
        permissions |= 2;
    }
    if mode & 0o100 != 0 {
        permissions |= if metadata.is_dir() { 8 } else { 1 };
    }
    HostIntResult::ok(permissions)
}

#[cfg(target_arch = "wasm32")]
fn get_permissions_path_bytes(path: &[u8]) -> HostIntResult {
    let _ = path;
    HostIntResult::err(errno_i32("ENOSYS"))
}

#[cfg(not(any(unix, target_arch = "wasm32")))]
fn get_permissions_path_bytes(path: &[u8]) -> HostIntResult {
    let Ok(path) = std::str::from_utf8(path) else {
        return HostIntResult::err(errno_i32("EINVAL"));
    };
    let metadata = match std::fs::metadata(path) {
        Ok(metadata) => metadata,
        Err(err) => return HostIntResult::os_err(io_error_errno(&err)),
    };
    let mut permissions = 4;
    if !metadata.permissions().readonly() {
        permissions |= 2;
    }
    if metadata.is_dir() {
        permissions |= 8;
    }
    HostIntResult::ok(permissions)
}

#[cfg(all(unix, not(target_arch = "wasm32")))]
fn set_permissions_path_bytes(path: &[u8], permissions: i64) -> HostIntResult {
    use std::ffi::OsStr;
    use std::os::unix::ffi::OsStrExt;
    use std::os::unix::fs::PermissionsExt;

    unsafe extern "C" {
        fn umask(mask: u32) -> u32;
    }

    let Ok(permissions) = u32::try_from(permissions) else {
        return HostIntResult::err(errno_i32("EINVAL"));
    };
    let path = std::path::Path::new(OsStr::from_bytes(path));
    let file = match std::fs::File::open(path) {
        Ok(file) => file,
        Err(err) => return HostIntResult::os_err(io_error_errno(&err)),
    };
    let metadata = match file.metadata() {
        Ok(metadata) => metadata,
        Err(err) => return HostIntResult::os_err(io_error_errno(&err)),
    };
    let mut user_mode = 0;
    if permissions & 4 != 0 {
        user_mode |= 0o400;
    }
    if permissions & 2 != 0 {
        user_mode |= 0o200;
    }
    if permissions & 1 != 0 || permissions & 8 != 0 {
        user_mode |= 0o100;
    }
    let mut mode = user_mode | (user_mode >> 3) | (user_mode >> 6);
    // SAFETY: umask is process-global like in the C runtime. We restore it immediately.
    let mask = unsafe { umask(0) };
    // SAFETY: restores the mask value just read above.
    unsafe {
        umask(mask);
    }
    mode &= !mask;
    mode |= metadata.permissions().mode() & !0o777;
    match file.set_permissions(std::fs::Permissions::from_mode(mode)) {
        Ok(()) => HostIntResult::ok(0),
        Err(err) => HostIntResult::os_err(io_error_errno(&err)),
    }
}

#[cfg(target_arch = "wasm32")]
fn set_permissions_path_bytes(path: &[u8], permissions: i64) -> HostIntResult {
    let _ = (path, permissions);
    HostIntResult::err(errno_i32("ENOSYS"))
}

#[cfg(not(any(unix, target_arch = "wasm32")))]
fn set_permissions_path_bytes(path: &[u8], permissions: i64) -> HostIntResult {
    let _ = permissions;
    let Ok(path) = std::str::from_utf8(path) else {
        return HostIntResult::err(errno_i32("EINVAL"));
    };
    let metadata = match std::fs::metadata(path) {
        Ok(metadata) => metadata,
        Err(err) => return HostIntResult::os_err(io_error_errno(&err)),
    };
    let mut permissions = metadata.permissions();
    permissions.set_readonly(false);
    match std::fs::set_permissions(path, permissions) {
        Ok(()) => HostIntResult::ok(0),
        Err(err) => HostIntResult::os_err(io_error_errno(&err)),
    }
}

#[cfg(all(unix, not(target_arch = "wasm32")))]
fn dir_entries_path_bytes(path: &[u8]) -> Result<Vec<Vec<u8>>, i32> {
    use std::ffi::OsStr;
    use std::os::unix::ffi::{OsStrExt, OsStringExt};

    let path = std::path::Path::new(OsStr::from_bytes(path));
    let mut entries = vec![b".".to_vec(), b"..".to_vec()];
    for entry in std::fs::read_dir(path)
        .map_err(|err| io_error_errno(&err).unwrap_or_else(|| errno_i32("ENOENT")))?
    {
        let entry =
            entry.map_err(|err| io_error_errno(&err).unwrap_or_else(|| errno_i32("ENOENT")))?;
        entries.push(entry.file_name().into_vec());
    }
    Ok(entries)
}

#[cfg(target_arch = "wasm32")]
fn dir_entries_path_bytes(path: &[u8]) -> Result<Vec<Vec<u8>>, i32> {
    let _ = path;
    Err(errno_i32("ENOSYS"))
}

#[cfg(not(any(unix, target_arch = "wasm32")))]
fn dir_entries_path_bytes(path: &[u8]) -> Result<Vec<Vec<u8>>, i32> {
    let path = std::str::from_utf8(path).map_err(|_| errno_i32("EINVAL"))?;
    let mut entries = vec![b".".to_vec(), b"..".to_vec()];
    for entry in std::fs::read_dir(path)
        .map_err(|err| io_error_errno(&err).unwrap_or_else(|| errno_i32("ENOENT")))?
    {
        let entry =
            entry.map_err(|err| io_error_errno(&err).unwrap_or_else(|| errno_i32("ENOENT")))?;
        entries.push(
            entry
                .file_name()
                .to_string_lossy()
                .into_owned()
                .into_bytes(),
        );
    }
    Ok(entries)
}

#[cfg(all(unix, not(target_arch = "wasm32")))]
fn native_fopen_bfile(path: &[u8], mode: &[u8]) -> Result<BFile, i32> {
    use std::ffi::OsStr;
    use std::os::unix::ffi::OsStrExt;

    let mode = parse_native_file_mode(mode).ok_or_else(|| errno_i32("EINVAL"))?;
    let file = open_native_file(std::path::Path::new(OsStr::from_bytes(path)), mode)?;
    Ok(BFile {
        kind: BFileKind::NativeFile {
            file: std::rc::Rc::new(std::cell::RefCell::new(file)),
            ungot: Vec::new(),
        },
        readable: mode.readable,
        writable: mode.writable,
    })
}

#[cfg(target_arch = "wasm32")]
fn native_fopen_bfile(path: &[u8], mode: &[u8]) -> Result<BFile, i32> {
    let _ = (path, mode);
    Err(errno_i32("ENOSYS"))
}

#[cfg(not(any(unix, target_arch = "wasm32")))]
fn native_fopen_bfile(path: &[u8], mode: &[u8]) -> Result<BFile, i32> {
    let path = std::str::from_utf8(path).map_err(|_| errno_i32("EINVAL"))?;
    let mode = parse_native_file_mode(mode).ok_or_else(|| errno_i32("EINVAL"))?;
    let file = open_native_file(std::path::Path::new(path), mode)?;
    Ok(BFile {
        kind: BFileKind::NativeFile {
            file: std::rc::Rc::new(std::cell::RefCell::new(file)),
            ungot: Vec::new(),
        },
        readable: mode.readable,
        writable: mode.writable,
    })
}

#[cfg(all(unix, not(target_arch = "wasm32")))]
fn native_fd_bfile(fd: i32) -> Result<BFile, i32> {
    use std::os::unix::io::FromRawFd;

    if fd < 0 {
        return Err(errno_i32("EBADF"));
    }
    // SAFETY: add_fd transfers fd ownership to the BFILE, matching the C runtime closeb_fd path.
    let file = unsafe { std::fs::File::from_raw_fd(fd) };
    Ok(BFile {
        kind: BFileKind::NativeFile {
            file: std::rc::Rc::new(std::cell::RefCell::new(file)),
            ungot: Vec::new(),
        },
        readable: true,
        writable: true,
    })
}

#[cfg(not(all(unix, not(target_arch = "wasm32"))))]
fn native_fd_bfile(fd: i32) -> Result<BFile, i32> {
    let _ = fd;
    Err(errno_i32("ENOSYS"))
}

#[cfg(all(unix, not(target_arch = "wasm32")))]
fn open_fd_path_bytes(path: &[u8], flags: i32, mode: i64) -> HostIntResult {
    let path = match std::ffi::CString::new(path) {
        Ok(path) => path,
        Err(_) => return HostIntResult::err(errno_i32("EINVAL")),
    };
    let mode = match libc::mode_t::try_from(mode) {
        Ok(mode) => mode,
        Err(_) => return HostIntResult::err(errno_i32("EINVAL")),
    };
    // SAFETY: path is NUL-terminated and flags/mode are plain C values.
    let fd = unsafe { libc::open(path.as_ptr(), flags, mode) };
    if fd < 0 {
        HostIntResult::err(last_errno())
    } else {
        HostIntResult::ok(i64::from(fd))
    }
}

#[cfg(not(all(unix, not(target_arch = "wasm32"))))]
fn open_fd_path_bytes(path: &[u8], flags: i32, mode: i64) -> HostIntResult {
    let _ = (path, flags, mode);
    HostIntResult::err(errno_i32("ENOSYS"))
}

#[cfg(all(unix, not(target_arch = "wasm32")))]
fn close_fd(fd: i32) -> HostIntResult {
    // SAFETY: close only consumes the integer file descriptor.
    let rc = unsafe { libc::close(fd) };
    if rc < 0 {
        HostIntResult::err(last_errno())
    } else {
        HostIntResult::ok(i64::from(rc))
    }
}

#[cfg(not(all(unix, not(target_arch = "wasm32"))))]
fn close_fd(fd: i32) -> HostIntResult {
    let _ = fd;
    HostIntResult::err(errno_i32("ENOSYS"))
}

#[cfg(all(unix, not(target_arch = "wasm32")))]
fn fcntl_fd(fd: i32, cmd: i32, arg: i32) -> HostIntResult {
    // SAFETY: this mirrors the C runtime's three-int fcntl wrapper.
    let rc = unsafe { libc::fcntl(fd, cmd, arg) };
    if rc < 0 {
        HostIntResult::err(last_errno())
    } else {
        HostIntResult::ok(i64::from(rc))
    }
}

#[cfg(not(all(unix, not(target_arch = "wasm32"))))]
fn fcntl_fd(fd: i32, cmd: i32, arg: i32) -> HostIntResult {
    let _ = (fd, cmd, arg);
    HostIntResult::err(errno_i32("ENOSYS"))
}

#[cfg(all(unix, not(target_arch = "wasm32")))]
fn socket_fd(domain: i32, typ: i32, protocol: i32) -> HostIntResult {
    // SAFETY: socket takes plain integer arguments.
    let fd = unsafe { libc::socket(domain, typ, protocol) };
    if fd < 0 {
        HostIntResult::err(last_errno())
    } else {
        HostIntResult::ok(i64::from(fd))
    }
}

#[cfg(not(all(unix, not(target_arch = "wasm32"))))]
fn socket_fd(domain: i32, typ: i32, protocol: i32) -> HostIntResult {
    let _ = (domain, typ, protocol);
    HostIntResult::err(errno_i32("ENOSYS"))
}

#[cfg(all(unix, not(target_arch = "wasm32")))]
fn bind_socket(fd: i32, addr: &[u8]) -> HostIntResult {
    let len = match libc::socklen_t::try_from(addr.len()) {
        Ok(len) => len,
        Err(_) => return HostIntResult::err(errno_i32("EINVAL")),
    };
    // SAFETY: addr points to len bytes copied from guest memory.
    let rc = unsafe { libc::bind(fd, addr.as_ptr().cast(), len) };
    if rc < 0 {
        HostIntResult::err(last_errno())
    } else {
        HostIntResult::ok(i64::from(rc))
    }
}

#[cfg(not(all(unix, not(target_arch = "wasm32"))))]
fn bind_socket(fd: i32, addr: &[u8]) -> HostIntResult {
    let _ = (fd, addr);
    HostIntResult::err(errno_i32("ENOSYS"))
}

#[cfg(all(unix, not(target_arch = "wasm32")))]
fn connect_socket(fd: i32, addr: &[u8]) -> HostIntResult {
    let len = match libc::socklen_t::try_from(addr.len()) {
        Ok(len) => len,
        Err(_) => return HostIntResult::err(errno_i32("EINVAL")),
    };
    // SAFETY: addr points to len bytes copied from guest memory.
    let rc = unsafe { libc::connect(fd, addr.as_ptr().cast(), len) };
    if rc < 0 {
        HostIntResult::err(last_errno())
    } else {
        HostIntResult::ok(i64::from(rc))
    }
}

#[cfg(not(all(unix, not(target_arch = "wasm32"))))]
fn connect_socket(fd: i32, addr: &[u8]) -> HostIntResult {
    let _ = (fd, addr);
    HostIntResult::err(errno_i32("ENOSYS"))
}

#[cfg(all(unix, not(target_arch = "wasm32")))]
fn listen_socket(fd: i32, backlog: i32) -> HostIntResult {
    // SAFETY: listen takes plain integer arguments.
    let rc = unsafe { libc::listen(fd, backlog) };
    if rc < 0 {
        HostIntResult::err(last_errno())
    } else {
        HostIntResult::ok(i64::from(rc))
    }
}

#[cfg(not(all(unix, not(target_arch = "wasm32"))))]
fn listen_socket(fd: i32, backlog: i32) -> HostIntResult {
    let _ = (fd, backlog);
    HostIntResult::err(errno_i32("ENOSYS"))
}

#[cfg(all(unix, not(target_arch = "wasm32")))]
fn setsockopt_socket(fd: i32, level: i32, optname: i32, optval: &[u8]) -> HostIntResult {
    let len = match libc::socklen_t::try_from(optval.len()) {
        Ok(len) => len,
        Err(_) => return HostIntResult::err(errno_i32("EINVAL")),
    };
    // SAFETY: optval points to len bytes copied from guest memory.
    let rc = unsafe { libc::setsockopt(fd, level, optname, optval.as_ptr().cast(), len) };
    if rc < 0 {
        HostIntResult::err(last_errno())
    } else {
        HostIntResult::ok(i64::from(rc))
    }
}

#[cfg(not(all(unix, not(target_arch = "wasm32"))))]
fn setsockopt_socket(fd: i32, level: i32, optname: i32, optval: &[u8]) -> HostIntResult {
    let _ = (fd, level, optname, optval);
    HostIntResult::err(errno_i32("ENOSYS"))
}

#[cfg(not(target_arch = "wasm32"))]
fn parse_native_file_mode(mode: &[u8]) -> Option<NativeFileMode> {
    let mut normalized = Vec::with_capacity(mode.len());
    for byte in mode {
        if *byte != b'b' {
            normalized.push(*byte);
        }
    }
    let mode = match normalized.as_slice() {
        b"r" => NativeFileMode {
            readable: true,
            writable: false,
            append: false,
            truncate: false,
            create: false,
        },
        b"w" => NativeFileMode {
            readable: false,
            writable: true,
            append: false,
            truncate: true,
            create: true,
        },
        b"a" => NativeFileMode {
            readable: false,
            writable: true,
            append: true,
            truncate: false,
            create: true,
        },
        b"r+" => NativeFileMode {
            readable: true,
            writable: true,
            append: false,
            truncate: false,
            create: false,
        },
        b"w+" => NativeFileMode {
            readable: true,
            writable: true,
            append: false,
            truncate: true,
            create: true,
        },
        b"a+" => NativeFileMode {
            readable: true,
            writable: true,
            append: true,
            truncate: false,
            create: true,
        },
        _ => return None,
    };
    Some(mode)
}

#[cfg(not(target_arch = "wasm32"))]
fn open_native_file(path: &std::path::Path, mode: NativeFileMode) -> Result<std::fs::File, i32> {
    std::fs::OpenOptions::new()
        .read(mode.readable)
        .write(mode.writable && !mode.append)
        .append(mode.append)
        .truncate(mode.truncate)
        .create(mode.create)
        .open(path)
        .map_err(|err| io_error_errno(&err).unwrap_or_else(|| errno_i32("ENOENT")))
}

fn ffi_arity(name: &str) -> Option<usize> {
    if errno_constant(name).is_some() {
        return Some(0);
    }
    if host_constant(name).is_some() {
        return Some(0);
    }
    Some(match name {
        "GETRAW"
        | "GETTIMEMICRO"
        | "islinux"
        | "ismacos"
        | "iswindows"
        | "sizeof_char"
        | "sizeof_short"
        | "sizeof_int"
        | "sizeof_long"
        | "sizeof_llong"
        | "sizeof_size_t"
        | "want_gmp"
        | "want_imath"
        | "&closeb"
        | "&free"
        | "&errno"
        | "errno"
        | "environ"
        | "get_executable_path"
        | "new_mpz"
        | "openb_wr_mem" => 0,
        "malloc"
        | "free"
        | "strlen"
        | "getenv"
        | "unsetenv"
        | "putchar"
        | "remove"
        | "system"
        | "chdir"
        | "get_permissions"
        | "add_fd"
        | "opendir"
        | "readdir"
        | "closedir"
        | "c_d_name"
        | "close"
        | "js_debug"
        | "js_eval_run"
        | "js_eval_call"
        | "js_set_haskellCallback"
        | "add_FILE"
        | "add_utf8"
        | "add_crlf"
        | "add_rle_compressor"
        | "add_rle_decompressor"
        | "add_base64_encoder"
        | "add_base64_decoder"
        | "add_lz77_compressor"
        | "add_lz77_decompressor"
        | "add_bwt_compressor"
        | "add_bwt_decompressor"
        | "add_lzma_compressor"
        | "add_lzma_decompressor"
        | "closeb"
        | "flushb"
        | "getb"
        | "peekPtr"
        | "peekWord"
        | "peek_uint8"
        | "peek_uint16"
        | "peek_uint32"
        | "peek_uint64"
        | "peek_int8"
        | "peek_int16"
        | "peek_int32"
        | "peek_int64"
        | "peek_char"
        | "peek_schar"
        | "peek_uchar"
        | "peek_short"
        | "peek_ushort"
        | "peek_int"
        | "peek_uint"
        | "peek_long"
        | "peek_ulong"
        | "peek_llong"
        | "peek_ullong"
        | "peek_size_t"
        | "peek_flt32"
        | "peek_flt64"
        | "mpz_get_d"
        | "mpz_get_f"
        | "mpz_get_si"
        | "mpz_get_si64"
        | "mpz_log2"
        | "mpz_popcount"
        | "acos"
        | "asin"
        | "atan"
        | "cos"
        | "exp"
        | "log"
        | "sin"
        | "sqrt"
        | "tan"
        | "acosf"
        | "asinf"
        | "atanf"
        | "cosf"
        | "expf"
        | "logf"
        | "sinf"
        | "sqrtf"
        | "tanf" => 1,
        "calloc" | "realloc" | "strcpy" | "fopen" | "tmpname" | "add_buf" | "mkdir" | "getcwd"
        | "set_permissions" | "md5BFILE" | "md5String" | "pokePtr" | "pokeWord" | "poke_uint8"
        | "poke_uint16" | "poke_uint32" | "poke_uint64" | "poke_int8" | "poke_int16"
        | "poke_int32" | "poke_int64" | "poke_char" | "poke_schar" | "poke_uchar"
        | "poke_short" | "poke_ushort" | "poke_int" | "poke_uint" | "poke_long" | "poke_ulong"
        | "poke_llong" | "poke_ullong" | "poke_size_t" | "poke_flt32" | "poke_flt64"
        | "openb_rd_mem" | "getcpu" | "gettimeofday" | "listen" | "mpz_abs" | "mpz_cmp"
        | "mpz_init_set_si" | "mpz_init_set_si64" | "mpz_init_set_ui" | "mpz_init_set_ui64"
        | "mpz_neg" | "mpz_tstbit" | "putb" | "ungetb" | "atan2" | "pow" | "scalbn" | "atan2f"
        | "powf" | "scalbnf" => 2,
        "memcpy" | "memmove" | "setenv" | "md5Array" | "get_mem" | "readb" | "writeb" | "open"
        | "accept" | "bind" | "connect" | "fcntl" | "lz77c" | "mpz_add" | "mpz_and"
        | "mpz_fdiv_q_2exp" | "mpz_ior" | "mpz_mul" | "mpz_mul_2exp" | "mpz_sub" | "mpz_xor"
        | "socket" => 3,
        "recv" | "send" => 4,
        "mpz_tdiv_qr" => 4,
        "getsockopt" | "setsockopt" => 5,
        "strerror_r" => 3,
        _ => return None,
    })
}

fn is_unary_math_ffi_candidate(name: &str) -> bool {
    let bytes = name.as_bytes();
    match bytes.first() {
        Some(b'a') => {
            bytes.starts_with(b"ac") || bytes.starts_with(b"as") || bytes.starts_with(b"at")
        }
        Some(b'c') => bytes.starts_with(b"co"),
        Some(b'e') => bytes.starts_with(b"ex"),
        Some(b'l') => bytes.starts_with(b"lo"),
        Some(b's') => bytes.starts_with(b"si") || bytes.starts_with(b"sq"),
        Some(b't') => bytes.starts_with(b"ta"),
        _ => false,
    }
}

fn c_string_len(bytes: &[u8]) -> usize {
    bytes
        .iter()
        .position(|byte| *byte == 0)
        .unwrap_or(bytes.len())
}

fn std_handle(name: &str) -> Option<StdHandle> {
    Some(match name {
        "IO.stdin" => StdHandle::Stdin,
        "IO.stdout" => StdHandle::Stdout,
        "IO.stderr" => StdHandle::Stderr,
        _ => return None,
    })
}

fn std_handle_ptr(name: &str) -> Option<i64> {
    Some(match std_handle(name)? {
        StdHandle::Stdin => -1,
        StdHandle::Stdout => -2,
        StdHandle::Stderr => -3,
    })
}

fn handle_from_ptr(ptr: i64) -> Option<StdHandle> {
    Some(match ptr {
        -1 => StdHandle::Stdin,
        -2 => StdHandle::Stdout,
        -3 => StdHandle::Stderr,
        _ => return None,
    })
}

fn handle_name_from_ptr(ptr: i64) -> Option<&'static str> {
    Some(match handle_from_ptr(ptr)? {
        StdHandle::Stdin => "IO.stdin",
        StdHandle::Stdout => "IO.stdout",
        StdHandle::Stderr => "IO.stderr",
    })
}

fn push_display<T: fmt::Display>(out: &mut Vec<u8>, value: T) {
    out.extend_from_slice(value.to_string().as_bytes());
}

fn serialize_ptr(ptr: i64, out: &mut Vec<u8>) {
    match ptr {
        -1 => out.extend_from_slice(b"fp2p IO.stdin @"),
        -2 => out.extend_from_slice(b"fp2p IO.stdout @"),
        -3 => out.extend_from_slice(b"fp2p IO.stderr @"),
        _ => {
            out.extend_from_slice(b"toPtr #");
            push_display(out, ptr);
            out.extend_from_slice(b" @");
        }
    }
}

fn serialize_bytes_comb(bytes: &[u8], out: &mut Vec<u8>) {
    if bytes.len() > 100 {
        out.push(b'$');
        push_display(out, bytes.len());
        out.push(b' ');
        out.extend_from_slice(bytes);
    } else {
        serialize_bytes_quoted(bytes, out);
    }
}

fn serialize_bigint_decimal(bytes: &[u8], out: &mut Vec<u8>) {
    out.push(b'%');
    out.extend_from_slice(bytes);
    out.push(b'"');
}

fn serialize_bytes_quoted(bytes: &[u8], out: &mut Vec<u8>) {
    out.push(b'"');
    for &byte in bytes {
        match byte {
            b'"' | b'\\' | b'^' | b'|' => {
                out.push(b'\\');
                out.push(byte);
            }
            0xff => out.extend_from_slice(b"\\_"),
            0x20..=0x7e => out.push(byte),
            0x00..=0x1f => {
                out.push(b'^');
                out.push(byte | 0x20);
            }
            0x7f => out.extend_from_slice(b"\\?"),
            0x80..=0x9f => {
                out.push(b'^');
                out.push(byte & 0x1f | 0x40);
            }
            0xa0..=0xfe => {
                out.push(b'|');
                out.push(byte & 0x7f);
            }
        }
    }
    out.push(b'"');
}

fn head_utf8(bytes: &[u8]) -> Result<(u32, usize), EvalError> {
    let c1 = *bytes.first().ok_or(EvalError::InvalidByteString)?;
    if c1 & 0x80 == 0 {
        return Ok((c1 as u32, 1));
    }

    let c2 = *bytes.get(1).ok_or(EvalError::InvalidByteString)?;
    if c1 & 0xe0 == 0xc0 {
        return Ok(((((c1 & 0x1f) as u32) << 6) | ((c2 & 0x3f) as u32), 2));
    }

    let c3 = *bytes.get(2).ok_or(EvalError::InvalidByteString)?;
    if c1 & 0xf0 == 0xe0 {
        return Ok((
            (((c1 & 0x0f) as u32) << 12) | (((c2 & 0x3f) as u32) << 6) | ((c3 & 0x3f) as u32),
            3,
        ));
    }

    let c4 = *bytes.get(3).ok_or(EvalError::InvalidByteString)?;
    if c1 & 0xf8 == 0xf0 {
        return Ok((
            (((c1 & 0x07) as u32) << 18)
                | (((c2 & 0x3f) as u32) << 12)
                | (((c3 & 0x3f) as u32) << 6)
                | ((c4 & 0x3f) as u32),
            4,
        ));
    }

    Err(EvalError::InvalidByteString)
}

fn head_utf8_string(bytes: &[u8]) -> Result<Option<(u32, usize)>, EvalError> {
    let Some(&c1) = bytes.first() else {
        return Ok(None);
    };
    if c1 & 0x80 == 0 {
        return Ok(Some((c1 as u32, 1)));
    }

    let Some(&c2) = bytes.get(1) else {
        return Ok(None);
    };
    if c1 & 0xe0 == 0xc0 {
        let c = (((c1 & 0x1f) as u32) << 6) | ((c2 & 0x3f) as u32);
        if 0 < c && c < 0x80 {
            return Err(EvalError::InvalidByteString);
        }
        return Ok(Some((c, 2)));
    }

    let Some(&c3) = bytes.get(2) else {
        return Ok(None);
    };
    if c1 & 0xf0 == 0xe0 {
        let c = (((c1 & 0x0f) as u32) << 12) | (((c2 & 0x3f) as u32) << 6) | ((c3 & 0x3f) as u32);
        if c < 0x800 {
            return Err(EvalError::InvalidByteString);
        }
        return Ok(Some((c, 3)));
    }

    let Some(&c4) = bytes.get(3) else {
        return Ok(None);
    };
    if c1 & 0xf8 == 0xf0 {
        let c = (((c1 & 0x07) as u32) << 18)
            | (((c2 & 0x3f) as u32) << 12)
            | (((c3 & 0x3f) as u32) << 6)
            | ((c4 & 0x3f) as u32);
        if c < 0x10000 {
            return Err(EvalError::InvalidByteString);
        }
        return Ok(Some((c, 4)));
    }

    Err(EvalError::InvalidByteString)
}

fn decode_utf8_string_bytes(mut bytes: &[u8]) -> Result<Vec<u32>, EvalError> {
    let mut values = Vec::new();
    while let Some((value, offset)) = head_utf8_string(bytes)? {
        values.push(value);
        bytes = &bytes[offset..];
    }
    Ok(values)
}

fn modified_utf8(n: i64) -> Result<Vec<u8>, EvalError> {
    let mut c = u32::try_from(n).map_err(|_| EvalError::InvalidByteString)?;
    if c & 0x1ff800 == 0xd800 {
        c = 0xfffd;
    }
    if c > 0 && c < 0x80 {
        Ok(vec![c as u8])
    } else if c < 0x800 {
        Ok(vec![0xc0 | (c >> 6) as u8, 0x80 | (c & 0x3f) as u8])
    } else if c < 0x10000 {
        Ok(vec![
            0xe0 | (c >> 12) as u8,
            0x80 | ((c >> 6) & 0x3f) as u8,
            0x80 | (c & 0x3f) as u8,
        ])
    } else if c < 0x110000 {
        Ok(vec![
            0xf0 | (c >> 18) as u8,
            0x80 | ((c >> 12) & 0x3f) as u8,
            0x80 | ((c >> 6) & 0x3f) as u8,
            0x80 | (c & 0x3f) as u8,
        ])
    } else {
        Err(EvalError::InvalidByteString)
    }
}

fn render_bytes(bytes: &[u8], out: &mut String) {
    out.push('"');
    for &byte in bytes {
        match byte {
            b'\\' => out.push_str("\\\\"),
            b'"' => out.push_str("\\\""),
            b'\n' => out.push_str("\\n"),
            b'\r' => out.push_str("\\r"),
            b'\t' => out.push_str("\\t"),
            0x20..=0x7e => out.push(byte as char),
            _ => {
                out.push_str("\\x");
                out.push(nibble(byte >> 4));
                out.push(nibble(byte & 0x0f));
            }
        }
    }
    out.push('"');
}

fn nibble(n: u8) -> char {
    match n {
        0..=9 => (b'0' + n) as char,
        10..=15 => (b'a' + n - 10) as char,
        _ => unreachable!(),
    }
}

#[cfg(test)]
mod tests {
    use super::{
        BFile, BFileKind, EvalFrameStack, EvalSpine, FORCE_REDUCTION_LIMIT,
        IGNORED_IO_SHORTCUT_RECURSION_LIMIT, MpzValue, PersistentSpine, bwt_decode, bwt_encode,
        lz77_compress, lz77_decompress, lzma_compress_payload, lzma_decompress_payload,
        serialize_bytes_quoted,
    };
    use crate::{EvalError, KnownPrim, Node, NodeId, ParseError, Prim, Program, parse_program};

    fn whnf(input: &[u8]) -> String {
        let mut program = parse_program(input).unwrap();
        let (root, _) = program.reduce_whnf(100).unwrap();
        program.render(root)
    }

    fn collect_for_test(program: &mut Program, root: NodeId) {
        program
            .collect_garbage_between_steps(
                root,
                &EvalFrameStack::default(),
                &EvalSpine::default(),
                &PersistentSpine::default(),
                &[],
                &[],
                None,
            )
            .unwrap();
    }

    fn deserialize_memory(program: &mut Program, bytes: Vec<u8>) -> (NodeId, i64) {
        let ptr = program
            .alloc_bfile(BFile {
                kind: BFileKind::Memory { bytes, pos: 0 },
                readable: true,
                writable: false,
            })
            .unwrap();
        let ptr_node = program.push_node(Node::Ptr(ptr));
        let world = program.push_node(Node::Int(99_999));
        let deserialize = program.prim("IO.deserialize");
        let deserialize_ptr = program.app(deserialize, ptr_node);
        let action = program.app(deserialize_ptr, world);
        let pair = program
            .reduce_node_whnf(action, FORCE_REDUCTION_LIMIT)
            .unwrap();
        let (left, returned_world) = program.cell(pair).app_fields().unwrap();
        assert_eq!(returned_world, world);
        let (head, value) = program.cell(left).app_fields().unwrap();
        assert!(matches!(
            program.cell(head).prim(),
            Some(Prim::Known(KnownPrim::P))
        ));
        (value, ptr)
    }

    #[test]
    fn reduces_identity() {
        assert_eq!(whnf(b"v8.4\n0\nI #42 @ }"), "42");
    }

    #[test]
    fn reduces_skk_identity() {
        assert_eq!(whnf(b"v8.4\n0\nS K @ K @ #7 @ }"), "7");
    }

    #[test]
    fn reduces_optimizer_combinators() {
        assert_eq!(whnf(b"v8.4\n0\nS' K @ K @ K @ #5 @ #0 @ }"), "5");
        assert_eq!(whnf(b"v8.4\n0\nB' K @ #5 @ K @ #0 @ }"), "5");
        assert_eq!(whnf(b"v8.4\n0\nZ K @ #5 @ #0 @ #1 @ }"), "5");
        assert_eq!(whnf(b"v8.4\n0\nJ #5 @ #0 @ I @ }"), "5");
        assert_eq!(whnf(b"v8.4\n0\nL #5 @ I @ #0 @ }"), "5");
        assert_eq!(whnf(b"v8.4\n0\nKK #0 @ #5 @ #1 @ }"), "5");
        assert_eq!(whnf(b"v8.4\n0\nKA #0 @ #1 @ #5 @ }"), "5");
        assert_eq!(whnf(b"v8.4\n0\nC' A @ K @ #5 @ #0 @ }"), "5");
        assert_eq!(whnf(b"v8.4\n0\nR #0 @ K @ #5 @ }"), "5");
        assert_eq!(whnf(b"v8.4\n0\nO #5 @ #0 @ #1 @ K @ }"), "5");
        assert_eq!(whnf(b"v8.4\n0\nC'B K @ K @ #5 @ #0 @ }"), "5");
    }

    #[test]
    fn reduces_partial_arity_specializations() {
        assert_eq!(whnf(b"v8.4\n0\nB' I @ I @ #9 @ }"), "((B (I I)) 9)");
        assert_eq!(whnf(b"v8.4\n0\nZ K @ #5 @ }"), "(K (K 5))");
        assert_eq!(whnf(b"v8.4\n0\nR #1 @ #2 @ }"), "((C 2) 1)");
        assert_eq!(whnf(b"v8.4\n0\nK2 #1 @ #2 @ }"), "(K 1)");
        assert_eq!(whnf(b"v8.4\n0\nK3 #1 @ #2 @ #3 @ }"), "(K 1)");
        assert_eq!(whnf(b"v8.4\n0\nC'B K @ I @ #9 @ }"), "((B (K 9)) I)");
    }

    #[test]
    fn reduces_constructor_tags_and_tuples() {
        assert_eq!(whnf(b"v8.4\n0\nTAG3 #99 @ K @ }"), "3");
        assert_eq!(whnf(b"v8.4\n0\nTAG10 #99 @ A @ }"), "99");
        assert_eq!(whnf(b"v8.4\n0\nT3 #1 @ #2 @ #3 @ K3 @ #0 @ }"), "1");
        assert_eq!(whnf(b"v8.4\n0\nT4 #1 @ #2 @ #3 @ #4 @ K4 @ #0 @ }"), "1");
    }

    #[test]
    fn resolves_shared_labels() {
        assert_eq!(whnf(b"v8.4\n1\nA #42 :0 @ _0 @ }"), "42");
    }

    #[test]
    fn serializes_quoted_bytestring_escapes_like_c() {
        let mut special = Vec::new();
        serialize_bytes_quoted(b"?^|\\\"", &mut special);
        assert_eq!(special, b"\"?\\^\\|\\\\\\\"\"");

        for byte in 0u8..=255 {
            let mut input = b"v8.4\n0\n".to_vec();
            serialize_bytes_quoted(&[byte], &mut input);
            input.extend_from_slice(b" }\n");

            let program = parse_program(&input).unwrap();
            match &program.nodes()[program.root().index()] {
                Node::Bytes(bytes) => assert_eq!(
                    bytes.as_slice(),
                    &[byte],
                    "byte {byte:#04x} encoded as {}",
                    String::from_utf8_lossy(&input)
                ),
                other => panic!(
                    "byte {byte:#04x} parsed as {other:?} from {}",
                    String::from_utf8_lossy(&input)
                ),
            }
        }
    }

    #[test]
    fn serializes_bigints_with_c_wire_format() {
        let program = parse_program(b"v8.4\n0\n%-123456789\" }\n").unwrap();
        assert_eq!(
            program.serialize_program(program.root()).unwrap(),
            b"v8.4\n0\n%-123456789\" }\n"
        );

        let legacy = parse_program(b"v8.4\n0\n%\"123456789\" }\n").unwrap();
        match &legacy.nodes()[legacy.root().index()] {
            Node::BigInt(bytes) => assert_eq!(bytes.as_slice(), b"123456789"),
            other => panic!("legacy bigint parsed as {other:?}"),
        }
    }

    #[test]
    fn mpz_get_d_uses_decimal_rounding_like_c() {
        let decimal = b"-299228957055072645483636";
        let value = MpzValue::parse_decimal(decimal).unwrap();
        let expected = std::str::from_utf8(decimal)
            .unwrap()
            .parse::<f64>()
            .unwrap();
        assert_eq!(value.to_f64().to_bits(), expected.to_bits());
    }

    #[test]
    fn serializes_shared_apps_with_labels() {
        let program = parse_program(b"v8.4\n1\nK #1 @ :0 _0 @ }\n").unwrap();
        let serialized = program.serialize_program(program.root()).unwrap();
        assert!(serialized.starts_with(b"v8.4\n1\n"), "{serialized:?}");
        assert!(
            serialized
                .windows(b":2 ".len())
                .any(|window| window == b":2 "),
            "{}",
            String::from_utf8_lossy(&serialized)
        );
        assert!(
            serialized
                .windows(b"_2 ".len())
                .any(|window| window == b"_2 "),
            "{}",
            String::from_utf8_lossy(&serialized)
        );

        let round_trip = parse_program(&serialized).unwrap();
        assert_eq!(
            round_trip.serialize_program(round_trip.root()).unwrap(),
            serialized
        );
    }

    #[test]
    fn serializes_cyclic_apps_with_labels() {
        let program = parse_program(b"v8.4\n1\nK _0 @ :0 }\n").unwrap();
        let serialized = program.serialize_program(program.root()).unwrap();
        assert!(serialized.starts_with(b"v8.4\n1\n"), "{serialized:?}");
        assert!(
            serialized
                .windows(b":2 ".len())
                .any(|window| window == b":2 "),
            "{}",
            String::from_utf8_lossy(&serialized)
        );
        assert!(
            serialized
                .windows(b"_2 ".len())
                .any(|window| window == b"_2 "),
            "{}",
            String::from_utf8_lossy(&serialized)
        );
        parse_program(&serialized).unwrap();
    }

    #[test]
    fn io_deserialize_reads_one_comb_from_bfile() {
        let mut program = parse_program(b"v8.4\n0\nI }\n").unwrap();
        let (value, ptr) = deserialize_memory(&mut program, b"v8.4\n0\n#42 }tail".to_vec());
        assert_eq!(program.render(value), "42");
        assert_eq!(program.read_bfile_bytes(ptr, 4).unwrap(), b"tail");
    }

    #[test]
    fn io_deserialize_preserves_shared_cycles() {
        let mut program = parse_program(b"v8.4\n0\nI }\n").unwrap();
        let (value, _) = deserialize_memory(&mut program, b"v8.4\n1\nK _0 @ :0 }\n".to_vec());
        let serialized = program.serialize_program(value).unwrap();
        assert!(serialized.starts_with(b"v8.4\n1\n"), "{serialized:?}");
        assert!(
            serialized.contains(&b'_'),
            "{}",
            String::from_utf8_lossy(&serialized)
        );
        assert!(
            serialized.contains(&b':'),
            "{}",
            String::from_utf8_lossy(&serialized)
        );
        parse_program(&serialized).unwrap();
    }

    #[test]
    fn reduces_integer_arithmetic() {
        assert_eq!(whnf(b"v8.4\n0\n+ #40 @ #2 @ }"), "42");
        assert_eq!(whnf(b"v8.4\n0\nsubtract #10 @ #3 @ }"), "-7");
        assert_eq!(whnf(b"v8.4\n0\nquot #22 @ #5 @ }"), "4");
        assert_eq!(whnf(b"v8.4\n0\nrem #22 @ #5 @ }"), "2");
    }

    #[test]
    fn reduces_integer_bit_ops() {
        assert_eq!(whnf(b"v8.4\n0\nand #6 @ #3 @ }"), "2");
        assert_eq!(whnf(b"v8.4\n0\nor #4 @ #1 @ }"), "5");
        assert_eq!(whnf(b"v8.4\n0\nshl #3 @ #2 @ }"), "12");
        assert_eq!(whnf(b"v8.4\n0\npopcount #7 @ }"), "3");
    }

    #[test]
    fn reduces_integer_comparisons_to_microhs_bools() {
        assert_eq!(whnf(b"v8.4\n0\n== #2 @ #2 @ }"), "A");
        assert_eq!(whnf(b"v8.4\n0\n< #2 @ #1 @ }"), "K");
        assert_eq!(whnf(b"v8.4\n0\nu> #-1 @ #1 @ }"), "A");
        assert_eq!(whnf(b"v8.4\n0\nicmp #1 @ #2 @ }"), "K2");
        assert_eq!(whnf(b"v8.4\n0\nucmp #-1 @ #1 @ }"), "KA");
    }

    #[test]
    fn reduces_int64_primitives() {
        assert_eq!(whnf(b"v8.4\n0\nI+ ##40 @ ##2 @ }"), "42i64");
        assert_eq!(whnf(b"v8.4\n0\nIsubtract ##10 @ ##3 @ }"), "-7i64");
        assert_eq!(whnf(b"v8.4\n0\nIquot ##22 @ ##5 @ }"), "4i64");
        assert_eq!(whnf(b"v8.4\n0\nIand ##6 @ ##3 @ }"), "2i64");
        assert_eq!(whnf(b"v8.4\n0\nIshl ##3 @ #2 @ }"), "12i64");
        assert_eq!(whnf(b"v8.4\n0\nIpopcount ##7 @ }"), "3");
        assert_eq!(whnf(b"v8.4\n0\nI== ##2 @ ##2 @ }"), "A");
        assert_eq!(whnf(b"v8.4\n0\nIu> ##-1 @ ##1 @ }"), "A");
        assert_eq!(whnf(b"v8.4\n0\nIicmp ##1 @ ##2 @ }"), "K2");
        assert_eq!(whnf(b"v8.4\n0\nIucmp ##-1 @ ##1 @ }"), "KA");
        assert_eq!(whnf(b"v8.4\n0\nitoI #7 @ }"), "7i64");
        assert_eq!(whnf(b"v8.4\n0\nItoi ##7 @ }"), "7");
    }

    #[test]
    fn reduces_float64_primitives() {
        assert_eq!(whnf(b"v8.4\n0\nd+ &1.5 @ &2.25 @ }"), "3.75");
        assert_eq!(whnf(b"v8.4\n0\nd* &3 @ &2.5 @ }"), "7.5");
        assert_eq!(whnf(b"v8.4\n0\ndneg &1.5 @ }"), "-1.5");
        assert_eq!(whnf(b"v8.4\n0\nd< &1.5 @ &2.25 @ }"), "A");
        assert_eq!(whnf(b"v8.4\n0\nd== &1.5 @ &2.25 @ }"), "K");
        assert_eq!(whnf(b"v8.4\n0\nitod #7 @ }"), "7.0");
        assert_eq!(whnf(b"v8.4\n0\nItod ##7 @ }"), "7.0");
        assert_eq!(whnf(b"v8.4\n0\ndtoi &7.75 @ }"), "7");
        assert_eq!(whnf(b"v8.4\n0\nd> utod #-1 @ @ &1000 @ }"), "A");
        assert_eq!(whnf(b"v8.4\n0\ntoDbl fromDbl &1.5 @ @ }"), "1.5");
    }

    #[test]
    fn reduces_float32_primitives() {
        assert_eq!(whnf(b"v8.4\n0\nf+ &&1.5 @ &&2.25 @ }"), "3.75f");
        assert_eq!(whnf(b"v8.4\n0\nf* &&3 @ &&2.5 @ }"), "7.5f");
        assert_eq!(whnf(b"v8.4\n0\nfneg &&1.5 @ }"), "-1.5f");
        assert_eq!(whnf(b"v8.4\n0\nf< &&1.5 @ &&2.25 @ }"), "A");
        assert_eq!(whnf(b"v8.4\n0\nf== &&1.5 @ &&2.25 @ }"), "K");
        assert_eq!(whnf(b"v8.4\n0\nitof #7 @ }"), "7.0f");
        assert_eq!(whnf(b"v8.4\n0\nItof ##7 @ }"), "7.0f");
        assert_eq!(whnf(b"v8.4\n0\nftoi &&7.75 @ }"), "7");
        assert_eq!(whnf(b"v8.4\n0\nf> utof #-1 @ @ &&1000 @ }"), "A");
        assert_eq!(whnf(b"v8.4\n0\nftod dtof &1.5 @ @ }"), "1.5");
        assert_eq!(whnf(b"v8.4\n0\ntoFlt fromFlt &&1.5 @ @ }"), "1.5f");
    }

    #[test]
    fn reduces_bytestring_primitives() {
        assert_eq!(whnf(b"v8.4\n0\nbs++ \"foo\" @ \"bar\" @ }"), "\"foobar\"");
        assert_eq!(whnf(b"v8.4\n0\nbs++. \"foo\" @ \"bar\" @ }"), "\"foo.bar\"");
        assert_eq!(whnf(b"v8.4\n0\nbs== \"x\" @ \"x\" @ }"), "A");
        assert_eq!(whnf(b"v8.4\n0\nbs< \"abc\" @ \"abd\" @ }"), "A");
        assert_eq!(whnf(b"v8.4\n0\nbscmp \"abd\" @ \"abc\" @ }"), "KA");
        assert_eq!(whnf(b"v8.4\n0\nbslength \"hello\" @ }"), "5");
        assert_eq!(whnf(b"v8.4\n0\nbsreplicate #3 @ #65 @ }"), "\"AAA\"");
        assert_eq!(whnf(b"v8.4\n0\nbsindex \"ABC\" @ #1 @ }"), "66");
        assert_eq!(
            whnf(b"v8.4\n0\nbssubstr \"abcdef\" @ #2 @ #3 @ }"),
            "\"cde\""
        );
        assert_eq!(whnf(b"v8.4\n0\nheadUTF8 $2 \xc3\xa5 @ }"), "229");
        assert_eq!(whnf(b"v8.4\n0\ntailUTF8 $3 \xc3\xa5x @ }"), "\"x\"");
        assert_eq!(whnf(b"v8.4\n0\nbsunpack \"AB\" @ #0 @ K @ }"), "65");
        assert_eq!(
            whnf(b"v8.4\n0\nbsunpack \"AB\" @ #0 @ A @ #0 @ K @ }"),
            "66"
        );
        assert_eq!(whnf(b"v8.4\n0\nfromUTF8 $3 \xc3\xa5x @ #0 @ K @ }"), "229");
        assert_eq!(
            whnf(b"v8.4\n0\nfromUTF8 $3 \xc3\xa5x @ #0 @ A @ #0 @ K @ }"),
            "120"
        );
        assert_eq!(whnf(b"v8.4\n0\nfromUTF8 $2 \xc0\x80 @ #0 @ K @ }"), "0");
        assert_eq!(whnf(b"v8.4\n0\nfromUTF8 $1 \xc3 @ }"), "K");

        let mut program = parse_program(b"v8.4\n0\nfromUTF8 $2 \xc1\x81 @ }").unwrap();
        assert!(matches!(
            program.reduce_whnf(100),
            Err(EvalError::InvalidByteString)
        ));
    }

    #[test]
    fn reduces_mutable_bytestring_primitives() {
        assert_eq!(whnf(b"v8.4\n0\nbsnew #2 @ #4 @ }"), "\"\\x00\\x00\"");
        assert_eq!(
            whnf(b"v8.4\n1\nseq bswrite bsnew #2 @ #4 @ :0 @ #1 @ #65 @ @ bsread _0 @ #1 @ @ }"),
            "65"
        );
        assert_eq!(
            whnf(b"v8.4\n1\nseq bsappbyte bsnew #0 @ #0 @ :0 @ #65 @ @ bsfreeze _0 @ @ }"),
            "\"A\""
        );
        assert_eq!(
            whnf(b"v8.4\n1\nseq bsappchar bsnew #0 @ #0 @ :0 @ #229 @ @ bsfreeze _0 @ @ }"),
            "\"\\xc3\\xa5\""
        );
        assert_eq!(
            whnf(b"v8.4\n1\nseq bswrite \"abc\" :0 @ #1 @ #88 @ @ bsread _0 @ #1 @ @ }"),
            "88"
        );
    }

    #[test]
    fn reduces_strict_alias_and_probe_primitives() {
        assert_eq!(whnf(b"v8.4\n0\nord #65 @ }"), "65");
        assert_eq!(whnf(b"v8.4\n0\nchr #65 @ }"), "65");
        assert_eq!(whnf(b"v8.4\n0\nseq + #1 @ #2 @ @ #9 @ }"), "9");
        assert_eq!(whnf(b"v8.4\n0\nisint #7 @ }"), "7");
        assert_eq!(whnf(b"v8.4\n0\nisint \"x\" @ }"), "-1");
    }

    #[test]
    fn rejects_unknown_primitives() {
        assert!(matches!(
            parse_program(b"v8.4\n0\nnot-a-prim }"),
            Err(ParseError::UnknownPrim(name)) if name == "not-a-prim"
        ));

        let mut unsupported = parse_program(b"v8.4\n0\nIO.fork #1 @ }").unwrap();
        assert!(matches!(
            unsupported.reduce_whnf(10),
            Err(EvalError::UnknownPrim(name)) if name == "IO.fork"
        ));
    }

    #[test]
    fn reduces_rnf_and_exception_primitives() {
        assert_eq!(whnf(b"v8.4\n0\nrnf #0 @ O #1 @ #2 @ @ }"), "I");
        assert_eq!(whnf(b"v8.4\n0\nrnf #1 @ raise #7 @ @ }"), "I");

        let mut program = parse_program(b"v8.4\n0\nraise #7 @ }").unwrap();
        assert!(matches!(
            program.reduce_whnf(100),
            Err(EvalError::Raised(_))
        ));

        assert_eq!(
            whnf(b"v8.4\n0\nIO.performIO catch IO.return #5 @ @ K IO.return #42 @ @ @ @ }"),
            "5"
        );
        assert_eq!(
            whnf(b"v8.4\n0\nIO.performIO catch raise #7 @ @ K IO.return #42 @ @ @ @ }"),
            "42"
        );
        assert_eq!(
            whnf(b"v8.4\n0\nIO.performIO catch IO.strict IO.return @ quot #1 @ #0 @ @ @ IO.return @ @ }"),
            "4"
        );
        assert_eq!(
            whnf(b"v8.4\n0\nIO.performIO catch IO.strict IO.return @ + #9223372036854775807 @ #1 @ @ @ IO.return @ @ }"),
            "7"
        );
        assert_eq!(
            whnf(b"v8.4\n0\nIO.performIO catch IO.strict IO.return @ Iuquot ##1 @ ##0 @ @ @ IO.return @ @ }"),
            "4"
        );
        assert_eq!(
            whnf(b"v8.4\n0\nIO.performIO catch IO.strict IO.return @ Ineg ##-9223372036854775808 @ @ @ IO.return @ @ }"),
            "7"
        );

        let mut program = parse_program(b"v8.4\n0\nshl #1 @ #64 @ }").unwrap();
        assert!(matches!(
            program.reduce_whnf(100),
            Err(EvalError::InvalidShift(64))
        ));
    }

    #[test]
    fn formats_uncaught_rts_exceptions_like_c() {
        let mut program = parse_program(b"v8.4\n0\nraise #4 @ }").unwrap();
        let Err(EvalError::Raised(exn)) = program.reduce_whnf(100) else {
            panic!("raise did not produce an exception");
        };
        assert_eq!(
            program.uncaught_exception_message_bytes(exn).unwrap(),
            b"DivideByZero"
        );
    }

    #[test]
    fn reduces_stable_pointer_primitives() {
        assert_eq!(whnf(b"v8.4\n0\nSPnew #42 @ }"), "1");
        assert_eq!(whnf(b"v8.4\n0\nSPderef SPnew #42 @ @ }"), "42");
        assert_eq!(
            whnf(b"v8.4\n2\nseq SPfree SPnew #1 @ :0 @ @ SPnew #2 @ :1 @ }"),
            "1"
        );
        assert_eq!(whnf(b"v8.4\n0\nIO.performIO SPnew #42 @ @ }"), "1");
        assert_eq!(
            whnf(b"v8.4\n0\nIO.performIO SPderef SPnew #42 @ @ @ }"),
            "42"
        );
    }

    #[test]
    fn reduces_pointer_conversion_primitives() {
        assert_eq!(whnf(b"v8.4\n0\ntoPtr #42 @ }"), "Ptr#42");
        assert_eq!(whnf(b"v8.4\n0\ntoInt toPtr #42 @ @ }"), "42");
        assert_eq!(whnf(b"v8.4\n0\ntoFunPtr #7 @ }"), "FunPtr#7");
        assert_eq!(whnf(b"v8.4\n0\ntoInt toFunPtr #7 @ @ }"), "7");
        assert_eq!(whnf(b"v8.4\n0\ntoInt toPtr toFunPtr #9 @ @ @ }"), "9");
    }

    #[test]
    fn reduces_foreign_pointer_primitives() {
        assert_eq!(
            whnf(b"v8.4\n0\nfp2bs bs2fp \"abcdef\" @ @ #3 @ }"),
            "\"abc\""
        );
        assert_eq!(
            whnf(b"v8.4\n0\nfp2bs fp+ bs2fp \"abcdef\" @ @ #2 @ @ #3 @ }"),
            "\"cde\""
        );
        assert_eq!(
            whnf(b"v8.4\n1\n== toInt fp2p bs2fp \"abc\" :0 @ @ @ @ toInt fp2p bs2fp _0 @ @ @ @ }"),
            "A"
        );
        assert_eq!(
            whnf(b"v8.4\n0\nIO.performIO fpnew toPtr #42 @ @ @ }"),
            "ForeignPtr#42"
        );
        assert_eq!(
            whnf(b"v8.4\n0\nseq fpfin toFunPtr #0 @ @ bs2fp \"abc\" @ @ @ #7 @ }"),
            "7"
        );
    }

    #[test]
    fn reduces_weak_pointer_primitives() {
        assert_eq!(
            whnf(b"v8.4\n0\nIO.performIO IO.lazyBind Wknew #0 @ #42 @ @ Wkderef @ @ #0 @ I @ }"),
            "42"
        );
        assert_eq!(
            whnf(b"v8.4\n1\nseq Wkfinal Wknew #0 @ #42 @ :0 @ @ IO.performIO Wkderef _0 @ @ #0 @ I @ @ }"),
            "42"
        );
    }

    #[test]
    fn weak_gc_clears_value_when_key_unreachable() {
        let mut program = parse_program(b"v8.4\n0\nI }\n").unwrap();
        let key = program.push_node(Node::Int(123_456));
        let value = program.push_node(Node::Int(42));
        let weak = program.new_weak_ptr(key, value, None);

        collect_for_test(&mut program, weak);

        match program.cold_node(weak) {
            Some(Node::Weak(weak)) => {
                assert_eq!(weak.key, None);
                assert_eq!(weak.value, None);
            }
            other => panic!("expected weak node, got {other:?}"),
        }
        let deref = program.deref_weak_ptr(weak).unwrap();
        assert_eq!(program.render(deref), "K");
    }

    #[test]
    fn weak_gc_keeps_value_when_key_reachable() {
        let mut program = parse_program(b"v8.4\n0\nI }\n").unwrap();
        let key = program.push_node(Node::Int(123_456));
        let value = program.push_node(Node::Int(42));
        let weak = program.new_weak_ptr(key, value, None);
        let root = program.app(weak, key);

        collect_for_test(&mut program, root);

        match program.cold_node(weak) {
            Some(Node::Weak(weak)) => {
                assert_eq!(weak.key, Some(key));
                assert_eq!(weak.value, Some(value));
            }
            other => panic!("expected weak node, got {other:?}"),
        }
        let deref = program.deref_weak_ptr(weak).unwrap();
        assert_eq!(
            program.cell(deref).app_fields().map(|(_, arg)| arg),
            Some(value)
        );
    }

    #[test]
    fn reduces_mvar_primitives() {
        assert_eq!(
            whnf(b"v8.4\n0\nIO.performIO IO.lazyBind IO.newmvar @ IO.trytakemvar @ @ #0 @ I @ }"),
            "0"
        );
        assert_eq!(
            whnf(b"v8.4\n1\nseq IO.performIO IO.newmvar @ :0 @ IO.performIO IO.>> IO.putmvar _0 @ #42 @ @ IO.takemvar _0 @ @ @ @ }"),
            "42"
        );
        assert_eq!(
            whnf(b"v8.4\n1\nseq IO.performIO IO.newmvar @ :0 @ IO.performIO IO.>> IO.putmvar _0 @ #42 @ @ IO.readmvar _0 @ @ @ @ }"),
            "42"
        );
        assert_eq!(
            whnf(b"v8.4\n1\nseq IO.performIO IO.newmvar @ :0 @ IO.performIO IO.tryputmvar _0 @ #42 @ @ @ }"),
            "A"
        );
    }

    #[test]
    fn reduces_array_primitives() {
        assert_eq!(whnf(b"v8.4\n0\nA.alloc #3 @ #7 @ }"), "[7, 7, 7]");
        assert_eq!(whnf(b"v8.4\n0\nA.size A.alloc #3 @ #7 @ @ }"), "3");
        assert_eq!(whnf(b"v8.4\n0\nA.read A.alloc #3 @ #7 @ @ #1 @ }"), "7");
        assert_eq!(whnf(b"v8.4\n1\nA.== #0 [1] :0 @ _0 @ }"), "A");
        assert_eq!(whnf(b"v8.4\n1\nA.== #0 [1] :0 @ A.copy _0 @ @ }"), "K");
        assert_eq!(
            whnf(b"v8.4\n1\nseq A.write #0 #0 #0 [3] :0 @ #1 @ #42 @ @ A.read _0 @ #1 @ @ }"),
            "42"
        );
        assert_eq!(
            whnf(b"v8.4\n1\nseq A.trunc #0 #0 #0 [3] :0 @ #1 @ @ A.size _0 @ @ }"),
            "1"
        );
        assert_eq!(whnf(b"v8.4\n1\nA.== A.alloc #1 @ #0 @ :0 @ _0 @ }"), "A");
        assert_eq!(
            whnf(b"v8.4\n1\nseq A.write A.alloc #3 @ #0 @ :0 @ #1 @ #42 @ @ A.read _0 @ #1 @ @ }"),
            "42"
        );
    }

    #[test]
    fn reduces_io_control_primitives() {
        assert_eq!(whnf(b"v8.4\n0\nIO.performIO IO.return #5 @ @ }"), "5");
        assert_eq!(
            whnf(b"v8.4\n0\nIO.performIO IO.>> IO.return #1 @ @ IO.return #7 @ @ @ }"),
            "7"
        );
        assert_eq!(
            whnf(b"v8.4\n0\nIO.performIO IO.>>= IO.return #1 @ @ K IO.return #7 @ @ @ @ }"),
            "7"
        );
        assert_eq!(
            whnf(b"v8.4\n0\nIO.performIO IO.lazyBind IO.return #3 @ @ IO.return @ @ }"),
            "3"
        );
        assert_eq!(
            whnf(b"v8.4\n0\nIO.performIO IO.strict IO.return #4 @ @ @ }"),
            "4"
        );
        assert_eq!(
            whnf(b"v8.4\n0\nIO.performIO IO.atomic IO.return #6 @ @ @ }"),
            "6"
        );
        assert_eq!(whnf(b"v8.4\n0\nIO.performIO IO.gc #0 @ @ }"), "I");
        assert_eq!(whnf(b"v8.4\n0\nIO.performIO IO.yield @ }"), "I");
        assert_eq!(whnf(b"v8.4\n0\nIO.performIO IO.getmaskingstate @ }"), "0");
        assert_eq!(
            whnf(b"v8.4\n0\nIO.performIO IO.>> IO.setmaskingstate #2 @ @ IO.getmaskingstate @ @ }"),
            "2"
        );
        assert_eq!(whnf(b"v8.4\n0\nthnum IO.performIO IO.thid @ @ }"), "1");
        assert_eq!(
            whnf(b"v8.4\n0\nIO.performIO IO.threadstatus IO.performIO IO.thid @ @ @ }"),
            "0"
        );
    }

    #[test]
    fn ignored_io_shortcut_has_depth_cap() {
        fn ignored_chain(program: &mut Program, len: usize) -> NodeId {
            let unit = program.prim("I");
            let ret = program.prim("IO.return");
            let unit_action = program.app(ret, unit);
            let result = program.push_node(Node::Int(7));
            let ret = program.prim("IO.return");
            let mut action = program.app(ret, result);
            for _ in 0..len {
                let then = program.prim("IO.>>");
                let left = program.app(then, unit_action);
                action = program.app(left, action);
            }
            action
        }

        let mut program = parse_program(b"v8.4\n0\nI }\n").unwrap();
        let bounded = ignored_chain(&mut program, IGNORED_IO_SHORTCUT_RECURSION_LIMIT / 2);
        assert!(
            program
                .ignored_io_action_reductions(bounded, usize::MAX)
                .unwrap()
                .is_some()
        );

        let mut program = parse_program(b"v8.4\n0\nI }\n").unwrap();
        let over_cap = ignored_chain(&mut program, IGNORED_IO_SHORTCUT_RECURSION_LIMIT + 1);
        assert_eq!(
            program
                .ignored_io_action_reductions(over_cap, usize::MAX)
                .unwrap(),
            None
        );

        let perform_io = program.prim("IO.performIO");
        program.root = program.app(perform_io, over_cap);
        let (root, _) = program.reduce_whnf(20_000).unwrap();
        assert_eq!(program.render(root), "7");
    }

    #[test]
    fn reduces_builtin_ffi_calls() {
        let is_linux = if cfg!(target_os = "linux") { "1" } else { "0" };
        assert_eq!(whnf(b"v8.4\n0\nIO.performIO ^islinux @ }"), is_linux);
        assert_eq!(whnf(b"v8.4\n0\nIO.performIO ^GETRAW @ }"), "-1");
        assert_eq!(whnf(b"v8.4\n0\nIO.performIO ^putchar #10 @ @ }"), "I");
        assert_eq!(whnf(b"v8.4\n0\nIO.performIO ^sizeof_char @ }"), "1");
        assert_eq!(
            whnf(b"v8.4\n0\nIO.performIO ^sizeof_int @ }"),
            std::mem::size_of::<std::os::raw::c_int>().to_string()
        );
        assert_eq!(whnf(b"v8.4\n0\nIO.performIO ^want_gmp @ }"), "0");
        assert_eq!(whnf(b"v8.4\n0\nIO.performIO ^want_imath @ }"), "1");
        assert_eq!(
            whnf(b"v8.4\n0\nIO.performIO dynsym \"islinux\" @ @ }"),
            is_linux
        );
        assert_eq!(
            whnf(b"v8.4\n0\nIO.performIO dynsym O #105 @ O #115 @ O #108 @ O #105 @ O #110 @ O #117 @ O #120 @ K @ @ @ @ @ @ @ @ @ }"),
            is_linux
        );

        let mut program = parse_program(b"v8.4\n0\nIO.performIO ^does_not_exist @ }").unwrap();
        assert!(matches!(
            program.reduce_whnf(100),
            Err(EvalError::UnknownFfi(name)) if name == "does_not_exist"
        ));

        let mut program = parse_program(b"v8.4\n0\nIO.performIO ^GETTIMEMICRO @ }").unwrap();
        let (root, _) = program.reduce_whnf(100).unwrap();
        let root = program.resolve(root).unwrap();
        match program.nodes()[root.index()] {
            Node::Int(n) => assert!(n >= 0),
            _ => panic!("GETTIMEMICRO did not return an Int"),
        }
    }

    #[test]
    fn lz77c_ffi_compresses_to_guest_buffer() {
        let mut program = parse_program(b"v8.4\n0\nI }\n").unwrap();
        let input = b"AAAAAAAAAAAAAAAAzzzzzzzzzzzzzzzz";
        let src = program.alloc_memory(input.len()).unwrap();
        program.write_pointer_bytes(src, input).unwrap();
        let out_ptr = program.alloc_memory(8).unwrap();

        let src_node = program.push_node(Node::Ptr(src));
        let len_node = program.push_node(Node::Int(input.len() as i64));
        let out_ptr_node = program.push_node(Node::Ptr(out_ptr));
        let world = program.prim("I");
        let Some((used, pair)) = program
            .ffi_call("lz77c", &[src_node, len_node, out_ptr_node, world])
            .unwrap()
        else {
            panic!("lz77c did not reduce");
        };
        assert_eq!(used, 4);

        let Some((compressed_len, returned_world)) = program.pair_fields(pair).unwrap() else {
            panic!("lz77c did not return a pair");
        };
        assert_eq!(returned_world, world);
        let compressed_len = match program.nodes()[compressed_len.index()] {
            Node::Int(n) => usize::try_from(n).unwrap(),
            ref other => panic!("lz77c length returned {other:?}"),
        };
        let compressed_ptr = program.peek_signed(out_ptr, 8).unwrap();
        let compressed = program
            .read_pointer_bytes(compressed_ptr, compressed_len)
            .unwrap();
        assert_eq!(lz77_decompress(&compressed).unwrap(), input);
    }

    #[test]
    fn decodes_c_runtime_compression_fixtures() {
        const INPUT: &[u8] = b"AAAAAAAAAAAAAAAABABABABABABA\xff\0end";
        const C_LZ77: &[u8] = &[
            76, 90, 49, 16, 0, 0, 0, 0, 65, 224, 0, 6, 0, 66, 224, 1, 2, 4, 255, 0, 101, 110, 100,
        ];
        const C_BWT: &[u8] = &[
            66, 87, 49, 33, 0, 0, 0, 1, 0, 0, 0, 255, 100, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65,
            65, 65, 65, 65, 65, 66, 66, 66, 66, 66, 66, 65, 65, 65, 65, 65, 65, 110, 0, 101, 65,
        ];
        const C_LZMA: &[u8] = &[
            76, 90, 50, 28, 0, 0, 0, 93, 0, 0, 0, 1, 33, 0, 0, 0, 0, 0, 0, 0, 0, 32, 237, 68, 84,
            65, 127, 132, 12, 164, 143, 145, 248, 248, 0,
        ];

        assert_eq!(&C_LZ77[..3], b"LZ1");
        let lz77_len = u32::from_le_bytes(C_LZ77[3..7].try_into().unwrap()) as usize;
        assert_eq!(lz77_len, C_LZ77.len() - 7);
        assert_eq!(lz77_decompress(&C_LZ77[7..]).unwrap(), INPUT);
        assert_eq!(
            lz77_decompress(&lz77_compress(INPUT).unwrap()).unwrap(),
            INPUT
        );

        assert_eq!(&C_BWT[..3], b"BW1");
        let bwt_len = u32::from_le_bytes(C_BWT[3..7].try_into().unwrap()) as usize;
        let bwt_zero = u32::from_le_bytes(C_BWT[7..11].try_into().unwrap()) as usize;
        assert_eq!(bwt_len, C_BWT.len() - 11);
        assert_eq!(bwt_decode(&C_BWT[11..], bwt_zero).unwrap(), INPUT);
        let (rust_bwt_zero, rust_bwt_last) = bwt_encode(INPUT).unwrap();
        assert_eq!(bwt_decode(&rust_bwt_last, rust_bwt_zero).unwrap(), INPUT);

        assert_eq!(&C_LZMA[..3], b"LZ2");
        let lzma_len = u32::from_le_bytes(C_LZMA[3..7].try_into().unwrap()) as usize;
        assert_eq!(lzma_len, C_LZMA.len() - 7);
        assert_eq!(lzma_decompress_payload(&C_LZMA[7..]).unwrap(), INPUT);
        assert_eq!(
            lzma_decompress_payload(&lzma_compress_payload(INPUT).unwrap()).unwrap(),
            INPUT
        );
    }

    #[test]
    fn reduces_math_ffi_calls() {
        assert_eq!(whnf(b"v8.4\n0\nIO.performIO ^sqrt &9 @ @ }"), "3.0");
        assert_eq!(whnf(b"v8.4\n0\nIO.performIO ^pow &2 @ &8 @ @ }"), "256.0");
        assert_eq!(
            whnf(b"v8.4\n0\nIO.performIO ^scalbn &1.5 @ #2 @ @ }"),
            "6.0"
        );
        assert_eq!(whnf(b"v8.4\n0\nIO.performIO ^sqrtf &&9 @ @ }"), "3.0f");
        assert_eq!(
            whnf(b"v8.4\n0\nIO.performIO ^powf &&2 @ &&8 @ @ }"),
            "256.0f"
        );
        assert_eq!(
            whnf(b"v8.4\n0\nIO.performIO ^scalbnf &&1.5 @ #2 @ @ }"),
            "6.0f"
        );
    }

    #[test]
    fn reduces_array_primitives_as_io_actions() {
        assert_eq!(
            whnf(b"v8.4\n0\nIO.performIO A.alloc #3 @ #7 @ @ }"),
            "[7, 7, 7]"
        );
        assert_eq!(whnf(b"v8.4\n0\nIO.performIO A.size #0 #0 [2] @ @ }"), "2");
        assert_eq!(whnf(b"v8.4\n0\nIO.performIO A.copy #0 [1] @ @ }"), "[0]");
        assert_eq!(
            whnf(b"v8.4\n1\nIO.performIO IO.>> A.write #0 #0 #0 [3] :0 @ #1 @ #42 @ @ A.read _0 @ #1 @ @ @ }"),
            "42"
        );
    }

    #[test]
    fn reduces_mutable_bytestring_primitives_as_io_actions() {
        assert_eq!(
            whnf(b"v8.4\n0\nIO.performIO bsnew #2 @ #4 @ @ }"),
            "\"\\x00\\x00\""
        );
        assert_eq!(
            whnf(b"v8.4\n1\nIO.performIO IO.>> bswrite bsnew #2 @ #4 @ :0 @ #1 @ #65 @ @ bsread _0 @ #1 @ @ @ }"),
            "65"
        );
        assert_eq!(
            whnf(b"v8.4\n1\nIO.performIO IO.>> bsappbyte bsnew #0 @ #0 @ :0 @ #65 @ @ bsfreeze _0 @ @ @ }"),
            "\"A\""
        );
    }
}
