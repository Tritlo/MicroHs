//! Strict primitive continuation frame payloads.
use super::*;

pub(in crate::runtime) enum WhnfFrameKind {
    Seq { result: NodeId },
    IoStrict { action: NodeId, value: NodeId },
    IsInt,
}

pub(in crate::runtime) enum IntFrameKind {
    BinSecond { op: IntBinOp, x: NodeId },
    BinFirst { op: IntBinOp, y: i64 },
    Un { op: IntUnOp },
}

pub(in crate::runtime) enum Int64FrameKind {
    BinSecond { op: Int64BinOp, x: NodeId },
    BinFirst { op: Int64BinOp, y: i64 },
    ShiftFirst { op: Int64BinOp, y: i64 },
    Un { op: Int64UnOp },
}

pub(in crate::runtime) enum Float64FrameKind {
    BinSecond { op: Float64BinOp, x: NodeId },
    BinFirst { op: Float64BinOp, y: f64 },
    Un { op: Float64UnOp },
}

pub(in crate::runtime) enum Float32FrameKind {
    BinSecond { op: Float32BinOp, x: NodeId },
    BinFirst { op: Float32BinOp, y: f32 },
    Un { op: Float32UnOp },
}

pub(in crate::runtime) enum BytesFrameKind {
    BinSecond { op: BytesBinOp, x: NodeId },
    BinFirst { op: BytesBinOp, y: NodeId },
}

#[derive(Clone, Copy, Debug)]
pub(in crate::runtime) enum ConversionFrameKind {
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

pub(in crate::runtime) enum ConversionValue {
    Int(i64),
    Int64(i64),
    Float64(f64),
    Float32(f32),
}

pub(in crate::runtime) struct StackWhnfFrame {
    pub(in crate::runtime) prev_app_base: usize,
    pub(in crate::runtime) redex: NodeId,
    pub(in crate::runtime) used: usize,
    pub(in crate::runtime) profile_head: ProfileHead,
    pub(in crate::runtime) kind: WhnfFrameKind,
}

pub(in crate::runtime) struct StackIntFrame {
    pub(in crate::runtime) prev_app_base: usize,
    pub(in crate::runtime) redex: NodeId,
    pub(in crate::runtime) profile_head: ProfileHead,
    pub(in crate::runtime) kind: IntFrameKind,
}

pub(in crate::runtime) struct StackInt64Frame {
    pub(in crate::runtime) prev_app_base: usize,
    pub(in crate::runtime) app_end: usize,
    pub(in crate::runtime) used: usize,
    pub(in crate::runtime) profile_head: ProfileHead,
    pub(in crate::runtime) kind: Int64FrameKind,
}

pub(in crate::runtime) struct StackInt64ShiftFrame {
    pub(in crate::runtime) prev_app_base: usize,
    pub(in crate::runtime) app_end: usize,
    pub(in crate::runtime) used: usize,
    pub(in crate::runtime) profile_head: ProfileHead,
    pub(in crate::runtime) op: Int64BinOp,
    pub(in crate::runtime) x: NodeId,
}

pub(in crate::runtime) struct StackFloat64Frame {
    pub(in crate::runtime) prev_app_base: usize,
    pub(in crate::runtime) app_end: usize,
    pub(in crate::runtime) used: usize,
    pub(in crate::runtime) profile_head: ProfileHead,
    pub(in crate::runtime) kind: Float64FrameKind,
}

pub(in crate::runtime) struct StackFloat32Frame {
    pub(in crate::runtime) prev_app_base: usize,
    pub(in crate::runtime) app_end: usize,
    pub(in crate::runtime) used: usize,
    pub(in crate::runtime) profile_head: ProfileHead,
    pub(in crate::runtime) kind: Float32FrameKind,
}

pub(in crate::runtime) struct StackBytesFrame {
    pub(in crate::runtime) prev_app_base: usize,
    pub(in crate::runtime) app_end: usize,
    pub(in crate::runtime) used: usize,
    pub(in crate::runtime) profile_head: ProfileHead,
    pub(in crate::runtime) kind: BytesFrameKind,
}

pub(in crate::runtime) struct StackConversionFrame {
    pub(in crate::runtime) prev_app_base: usize,
    pub(in crate::runtime) app_end: usize,
    pub(in crate::runtime) used: usize,
    pub(in crate::runtime) profile_head: ProfileHead,
    pub(in crate::runtime) kind: ConversionFrameKind,
}

pub(in crate::runtime) enum StackFrame {
    Whnf(StackWhnfFrame),
    Int(StackIntFrame),
    Int64(StackInt64Frame),
    Int64Shift(StackInt64ShiftFrame),
    Float64(StackFloat64Frame),
    Float32(StackFloat32Frame),
    Bytes(StackBytesFrame),
    Conversion(StackConversionFrame),
}

impl ConversionFrameKind {
    pub(in crate::runtime) fn expected_error(self, current: NodeId) -> EvalError {
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
