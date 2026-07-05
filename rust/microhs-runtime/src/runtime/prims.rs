//! MicroHs primitive identifiers and name/code mappings.
use super::*;

/// Stable index into the packed-cell arena.
///
/// The runtime keeps node ids stable across GC. Host handles, temporary eval
/// state, and serialized graph references can therefore hold ids directly.
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
pub struct RuntimePrim(pub(in crate::runtime) u16);

impl RuntimePrim {
    pub(in crate::runtime) fn from_name(name: &str) -> Option<Self> {
        RUNTIME_PRIM_NAMES
            .iter()
            .position(|candidate| *candidate == name)
            .map(|index| Self(index as u16))
    }

    pub(in crate::runtime) fn name(self) -> &'static str {
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
    pub(in crate::runtime) fn from_name(name: &str) -> Option<Self> {
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

    pub(in crate::runtime) fn name(self) -> &'static str {
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

pub(in crate::runtime) fn encode_known_prim(known: KnownPrim) -> u16 {
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

#[inline(always)]
pub(in crate::runtime) fn decode_known_prim(code: u16) -> KnownPrim {
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

pub(in crate::runtime) const TAG_PRIM_NAMES: [&str; 33] = [
    "TAG0", "TAG1", "TAG2", "TAG3", "TAG4", "TAG5", "TAG6", "TAG7", "TAG8", "TAG9", "TAG10",
    "TAG11", "TAG12", "TAG13", "TAG14", "TAG15", "TAG16", "TAG17", "TAG18", "TAG19", "TAG20",
    "TAG21", "TAG22", "TAG23", "TAG24", "TAG25", "TAG26", "TAG27", "TAG28", "TAG29", "TAG30",
    "TAG31", "TAG32",
];

pub(in crate::runtime) const TUPLE_PRIM_NAMES: [&str; 17] = [
    "", "", "", "T3", "T4", "T5", "T6", "T7", "T8", "T9", "T10", "T11", "T12", "T13", "T14", "T15",
    "T16",
];

pub(in crate::runtime) const RUNTIME_PRIM_NAMES: &[&str] = &[
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
