//! Runtime primitive head classification and strict-action lookup.
use super::*;

pub(in crate::runtime) enum EvalHead {
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
pub(in crate::runtime) enum StrictPrimitiveAction {
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

pub(in crate::runtime) const INT_BIN_RUNTIME_START: u16 = 0;
pub(in crate::runtime) const INT_BIN_RUNTIME_OPS: [IntBinOp; 30] = [
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

pub(in crate::runtime) const INT_UN_RUNTIME_START: u16 = 30;
pub(in crate::runtime) const INT_UN_RUNTIME_OPS: [IntUnOp; 6] = [
    IntUnOp::Neg,
    IntUnOp::UNeg,
    IntUnOp::Inv,
    IntUnOp::PopCount,
    IntUnOp::Clz,
    IntUnOp::Ctz,
];

pub(in crate::runtime) const INT64_BIN_RUNTIME_START: u16 = 36;
pub(in crate::runtime) const INT64_BIN_RUNTIME_OPS: [Int64BinOp; 30] = [
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

pub(in crate::runtime) const INT64_UN_RUNTIME_START: u16 = 66;
pub(in crate::runtime) const INT64_UN_RUNTIME_OPS: [Int64UnOp; 6] = [
    Int64UnOp::Neg,
    Int64UnOp::UNeg,
    Int64UnOp::Inv,
    Int64UnOp::PopCount,
    Int64UnOp::Clz,
    Int64UnOp::Ctz,
];

pub(in crate::runtime) const FLOAT64_BIN_RUNTIME_START: u16 = 72;
pub(in crate::runtime) const FLOAT64_BIN_RUNTIME_OPS: [Float64BinOp; 10] = [
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

pub(in crate::runtime) const FLOAT64_UN_RUNTIME_START: u16 = 82;
pub(in crate::runtime) const FLOAT64_UN_RUNTIME_OPS: [Float64UnOp; 1] = [Float64UnOp::Neg];

pub(in crate::runtime) const FLOAT32_BIN_RUNTIME_START: u16 = 83;
pub(in crate::runtime) const FLOAT32_BIN_RUNTIME_OPS: [Float32BinOp; 10] = [
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

pub(in crate::runtime) const FLOAT32_UN_RUNTIME_START: u16 = 93;
pub(in crate::runtime) const FLOAT32_UN_RUNTIME_OPS: [Float32UnOp; 1] = [Float32UnOp::Neg];

pub(in crate::runtime) const CONVERSION_RUNTIME_START: u16 = 94;
pub(in crate::runtime) const CONVERSION_RUNTIME_OPS: [ConversionFrameKind; 18] = [
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

pub(in crate::runtime) const BYTES_BIN_RUNTIME_START: u16 = 145;
pub(in crate::runtime) const BYTES_BIN_RUNTIME_OPS: [BytesBinOp; 9] = [
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
pub(in crate::runtime) fn runtime_prim_op<T: Copy, const N: usize>(
    index: u16,
    start: u16,
    ops: &[T; N],
) -> Option<T> {
    let offset = index.checked_sub(start)? as usize;
    ops.get(offset).copied()
}

impl RuntimePrim {
    #[inline(always)]
    pub(in crate::runtime) fn strict_action(self, args_len: usize) -> StrictPrimitiveAction {
        let index = self.0;

        macro_rules! try_action {
            ($arity:literal, $start:expr, $ops:expr, $variant:path) => {
                if args_len >= $arity {
                    if let Some(op) = runtime_prim_op(index, $start, $ops) {
                        return $variant(op);
                    }
                }
            };
        }

        try_action!(
            2,
            INT_BIN_RUNTIME_START,
            &INT_BIN_RUNTIME_OPS,
            StrictPrimitiveAction::IntBin
        );
        try_action!(
            1,
            INT_UN_RUNTIME_START,
            &INT_UN_RUNTIME_OPS,
            StrictPrimitiveAction::IntUn
        );
        try_action!(
            2,
            INT64_BIN_RUNTIME_START,
            &INT64_BIN_RUNTIME_OPS,
            StrictPrimitiveAction::Int64Bin
        );
        try_action!(
            1,
            INT64_UN_RUNTIME_START,
            &INT64_UN_RUNTIME_OPS,
            StrictPrimitiveAction::Int64Un
        );
        try_action!(
            2,
            FLOAT64_BIN_RUNTIME_START,
            &FLOAT64_BIN_RUNTIME_OPS,
            StrictPrimitiveAction::Float64Bin
        );
        try_action!(
            1,
            FLOAT64_UN_RUNTIME_START,
            &FLOAT64_UN_RUNTIME_OPS,
            StrictPrimitiveAction::Float64Un
        );
        try_action!(
            2,
            FLOAT32_BIN_RUNTIME_START,
            &FLOAT32_BIN_RUNTIME_OPS,
            StrictPrimitiveAction::Float32Bin
        );
        try_action!(
            1,
            FLOAT32_UN_RUNTIME_START,
            &FLOAT32_UN_RUNTIME_OPS,
            StrictPrimitiveAction::Float32Un
        );
        try_action!(
            2,
            BYTES_BIN_RUNTIME_START,
            &BYTES_BIN_RUNTIME_OPS,
            StrictPrimitiveAction::BytesBin
        );
        try_action!(
            1,
            CONVERSION_RUNTIME_START,
            &CONVERSION_RUNTIME_OPS,
            StrictPrimitiveAction::Conversion
        );

        StrictPrimitiveAction::None
    }
}
