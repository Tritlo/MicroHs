use std::cmp::Ordering;
use std::collections::{HashMap, HashSet};
use std::fmt;
use std::mem::size_of;

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct NodeId(pub usize);

#[derive(Clone, Debug)]
pub enum Node {
    App(NodeId, NodeId),
    Indir(Option<NodeId>),
    Prim(String),
    Int(i64),
    Int64(i64),
    Float64(f64),
    Float32(f32),
    ThreadId(i64),
    Ptr(i64),
    RawFunPtr(i64),
    ForeignPtr {
        bytes: Option<Vec<u8>>,
        offset: usize,
        ptr: i64,
        finalizer: Option<NodeId>,
    },
    Weak {
        value: Option<NodeId>,
        finalizer: Option<NodeId>,
    },
    MVar(Option<NodeId>),
    BigInt(Vec<u8>),
    Bytes(Vec<u8>),
    MutableBytes {
        bytes: Vec<u8>,
        capacity: usize,
    },
    Array(Vec<NodeId>),
    Ffi(String),
    JsCall {
        tags: String,
        body: Vec<u8>,
    },
    JsWrap {
        tags: String,
    },
    FunPtr(String),
    Tick(Vec<u8>),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum StdHandle {
    Stdin,
    Stdout,
    Stderr,
}

const ALLOCATION_PTR_BASE: i64 = -(1_i64 << 62);
const ALLOCATION_PTR_STRIDE: i64 = 1_i64 << 32;
const BFILE_PTR_BASE: i64 = i64::MIN + (1_i64 << 32);
const BFILE_PTR_STRIDE: i64 = 1_i64 << 32;
const DIR_PTR_BASE: i64 = i64::MIN + (1_i64 << 61);
const DIR_PTR_STRIDE: i64 = 1_i64 << 32;

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
    UnknownFfi(String),
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
            Self::UnknownFfi(name) => write!(f, "unknown FFI symbol {name}"),
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

#[derive(Clone, Debug)]
pub struct Program {
    nodes: Vec<Node>,
    root: NodeId,
    labels: HashMap<usize, NodeId>,
    stable_ptrs: Vec<Option<NodeId>>,
    allocations: Vec<Option<Vec<u8>>>,
    bfiles: Vec<Option<BFile>>,
    dirs: Vec<Option<DirHandle>>,
    program_args: Vec<Vec<u8>>,
    errno_value: i32,
    errno_ptr: Option<i64>,
    masking_state: i64,
    reductions: usize,
}

struct Spine {
    head: NodeId,
    args: Vec<NodeId>,
    apps: Vec<NodeId>,
}

struct StepResult {
    node: NodeId,
    in_place: bool,
    reductions: usize,
}

impl Program {
    pub fn new(nodes: Vec<Node>, root: NodeId, labels: HashMap<usize, NodeId>) -> Self {
        Self {
            nodes,
            root,
            labels,
            stable_ptrs: vec![None],
            allocations: Vec::new(),
            bfiles: Vec::new(),
            dirs: Vec::new(),
            program_args: Vec::new(),
            errno_value: 0,
            errno_ptr: None,
            masking_state: 0,
            reductions: 0,
        }
    }

    pub fn root(&self) -> NodeId {
        self.root
    }

    pub fn nodes(&self) -> &[Node] {
        &self.nodes
    }

    pub fn set_program_args(&mut self, args: Vec<Vec<u8>>) {
        self.program_args = args;
    }

    pub fn label(&self, label: usize) -> Option<NodeId> {
        self.labels.get(&label).copied()
    }

    pub fn push_node(&mut self, node: Node) -> NodeId {
        let id = NodeId(self.nodes.len());
        self.nodes.push(node);
        id
    }

    pub fn resolve(&self, mut id: NodeId) -> Result<NodeId, EvalError> {
        loop {
            match self.nodes.get(id.0) {
                Some(Node::Indir(Some(next))) => id = *next,
                Some(Node::Indir(None)) => return Err(EvalError::DanglingIndirection(id)),
                Some(_) => return Ok(id),
                None => return Err(EvalError::DanglingIndirection(id)),
            }
        }
    }

    pub fn reduce_whnf(&mut self, limit: usize) -> Result<(NodeId, usize), EvalError> {
        let mut root = self.root;
        let mut steps = 0;
        while steps < limit {
            let current = self.resolve(root)?;
            let Some(step) = self.step(current, limit - steps)? else {
                self.root = current;
                return Ok((current, steps));
            };
            steps += step.reductions;
            self.reductions += step.reductions;
            if !step.in_place {
                self.nodes[current.0] = Node::Indir(Some(step.node));
            }
            root = step.node;
        }
        Err(EvalError::StepLimit { limit })
    }

    fn step(&mut self, root: NodeId, budget: usize) -> Result<Option<StepResult>, EvalError> {
        let Spine { head, args, apps } = self.spine(root)?;
        if let Node::Ffi(name) = self.nodes[head.0].clone() {
            let Some((used, mut node)) = self.ffi_call(&name, &args)? else {
                return Ok(None);
            };
            let in_place = self.apply_remaining_spine(&mut node, &args[used..], &apps[used..]);
            return Ok(Some(StepResult {
                node,
                in_place,
                reductions: 1,
            }));
        }
        if let Node::JsCall { tags, body } = self.nodes[head.0].clone() {
            let Some((used, mut node)) = self.js_call(&tags, &body, &args)? else {
                return Ok(None);
            };
            let in_place = self.apply_remaining_spine(&mut node, &args[used..], &apps[used..]);
            return Ok(Some(StepResult {
                node,
                in_place,
                reductions: 1,
            }));
        }
        if let Node::JsWrap { tags } = self.nodes[head.0].clone() {
            let Some((used, mut node)) = self.js_wrap(&tags, &args)? else {
                return Ok(None);
            };
            let in_place = self.apply_remaining_spine(&mut node, &args[used..], &apps[used..]);
            return Ok(Some(StepResult {
                node,
                in_place,
                reductions: 1,
            }));
        }

        let Node::Prim(name) = self.nodes[head.0].clone() else {
            return Ok(None);
        };

        if name == "IO.>>" && args.len() >= 3 && budget >= 2 {
            let k = self.prim("K");
            let then = self.app(k, args[1]);
            let action = self.app(args[0], args[2]);
            let mut node = self.app(action, then);
            let in_place = self.apply_remaining_spine(&mut node, &args[3..], &apps[3..]);
            return Ok(Some(StepResult {
                node,
                in_place,
                reductions: 2,
            }));
        }

        let rewrite = match name.as_str() {
            "I" | "ord" | "chr" if !args.is_empty() => Some((1, args[0])),
            "K" if args.len() >= 2 => Some((2, args[0])),
            "A" if args.len() >= 2 => Some((2, args[1])),
            "U" if args.len() >= 2 => {
                let n = self.app(args[1], args[0]);
                Some((2, n))
            }
            "IO.performIO" if !args.is_empty() => {
                let world = self.world();
                let k = self.prim("K");
                let action = self.app(args[0], world);
                let n = self.app(action, k);
                Some((1, n))
            }
            "IO.atomic" if args.len() >= 2 => {
                let k = self.prim("K");
                let action = self.app(args[0], args[1]);
                let result = self.app(action, k);
                let pair = self.prim("P");
                let result_pair = self.app(pair, result);
                let n = self.app(result_pair, args[1]);
                Some((2, n))
            }
            "IO.>>=" if args.len() >= 3 => {
                let action = self.app(args[0], args[2]);
                let n = self.app(action, args[1]);
                Some((3, n))
            }
            "IO.>>" if args.len() >= 2 => {
                let bind = self.prim("IO.>>=");
                let bind_action = self.app(bind, args[0]);
                let k = self.prim("K");
                let then = self.app(k, args[1]);
                let n = self.app(bind_action, then);
                Some((2, n))
            }
            "IO.return" if args.len() >= 3 => {
                let kx = self.app(args[2], args[0]);
                let n = self.app(kx, args[1]);
                Some((3, n))
            }
            "IO.lazyBind" if args.len() >= 3 => {
                let world_result = self.app(args[0], args[2]);
                let fst = self.fst();
                let snd = self.snd();
                let result = self.app(fst, world_result);
                let world = self.app(snd, world_result);
                let next = self.app(args[1], result);
                let n = self.app(next, world);
                Some((3, n))
            }
            "IO.strict" if args.len() >= 2 => {
                self.reduce_node_whnf(args[1], 10_000)?;
                let n = self.app(args[0], args[1]);
                Some((2, n))
            }
            "IO.gc" if args.len() >= 2 => {
                let unit = self.prim("I");
                Some((2, self.pair(unit, args[1])))
            }
            "IO.stats" if !args.is_empty() => {
                let alloc = self.push_node(Node::Int(
                    i64::try_from(self.nodes.len()).unwrap_or(i64::MAX),
                ));
                let reductions = self.push_node(Node::Int(
                    i64::try_from(self.reductions).unwrap_or(i64::MAX),
                ));
                let stats = self.pair(alloc, reductions);
                Some((1, self.pair(stats, args[0])))
            }
            "IO.pp" if args.len() >= 2 => {
                #[cfg(not(target_arch = "wasm32"))]
                {
                    let rendered = self.render(args[0]);
                    eprintln!("{rendered}");
                }
                let unit = self.prim("I");
                Some((2, self.pair(unit, args[1])))
            }
            "IO.print" if args.len() >= 3 => {
                let handle = self.eval_io_handle(args[0])?;
                let value = self.reduce_node_whnf(args[1], 10_000)?;
                let rendered = self.render(value);
                self.write_io_handle(handle, &format!("{rendered}\n"))?;
                let unit = self.prim("I");
                Some((3, self.pair(unit, args[2])))
            }
            "IO.serialize" if args.len() >= 3 => {
                let handle = self.eval_io_handle(args[0])?;
                let value = self.reduce_node_whnf(args[1], 10_000)?;
                let serialized = self.serialize_program(value)?;
                self.write_io_handle_bytes(handle, &serialized)?;
                let unit = self.prim("I");
                Some((3, self.pair(unit, args[2])))
            }
            "IO.getArgRef" if !args.is_empty() => {
                let arg_array = self.arg_ref_array();
                Some((1, self.pair(arg_array, args[0])))
            }
            "IO.thid" if !args.is_empty() => {
                let thread = self.push_node(Node::ThreadId(1));
                Some((1, self.pair(thread, args[0])))
            }
            "IO.yield" if !args.is_empty() => {
                let unit = self.prim("I");
                Some((1, self.pair(unit, args[0])))
            }
            "IO.getmaskingstate" if !args.is_empty() => {
                let state = self.push_node(Node::Int(self.masking_state));
                Some((1, self.pair(state, args[0])))
            }
            "IO.setmaskingstate" if args.len() >= 2 => {
                self.masking_state = self.eval_int(args[0])?;
                let unit = self.prim("I");
                Some((2, self.pair(unit, args[1])))
            }
            "dynsym" if !args.is_empty() => {
                let name = self.eval_ffi_name(args[0])?;
                Some((1, self.push_node(Node::Ffi(name))))
            }
            "IO.threadstatus" if args.len() >= 2 => {
                self.eval_thread_id(args[0])?;
                let status = self.push_node(Node::Int(0));
                Some((2, self.pair(status, args[1])))
            }
            "IO.newmvar" if !args.is_empty() => {
                let mvar = self.push_node(Node::MVar(None));
                Some((1, self.pair(mvar, args[0])))
            }
            "IO.takemvar" if args.len() >= 2 => {
                let mvar = self.eval_mvar_id(args[0])?;
                let value = self.take_mvar(mvar)?.ok_or(EvalError::InvalidMVar)?;
                Some((2, self.pair(value, args[1])))
            }
            "IO.readmvar" if args.len() >= 2 => {
                let mvar = self.eval_mvar_id(args[0])?;
                let value = self.read_mvar(mvar)?.ok_or(EvalError::InvalidMVar)?;
                Some((2, self.pair(value, args[1])))
            }
            "IO.putmvar" if args.len() >= 3 => {
                let mvar = self.eval_mvar_id(args[0])?;
                self.put_mvar(mvar, args[1])?;
                let unit = self.prim("I");
                Some((3, self.pair(unit, args[2])))
            }
            "IO.trytakemvar" if args.len() >= 2 => {
                let mvar = self.eval_mvar_id(args[0])?;
                let value = match self.take_mvar(mvar)? {
                    Some(value) => self.just(value),
                    None => self.nothing(),
                };
                Some((2, self.pair(value, args[1])))
            }
            "IO.tryreadmvar" if args.len() >= 2 => {
                let mvar = self.eval_mvar_id(args[0])?;
                let value = match self.read_mvar(mvar)? {
                    Some(value) => self.just(value),
                    None => self.nothing(),
                };
                Some((2, self.pair(value, args[1])))
            }
            "IO.tryputmvar" if args.len() >= 3 => {
                let mvar = self.eval_mvar_id(args[0])?;
                let value = if self.try_put_mvar(mvar, args[1])? {
                    self.prim("A")
                } else {
                    self.prim("K")
                };
                Some((3, self.pair(value, args[2])))
            }
            "catch" if args.len() >= 3 => {
                let action = self.app(args[0], args[2]);
                Some((3, self.catch_result(action, args[1], args[2])?))
            }
            "catchr" if args.len() >= 3 => Some((3, self.catch_result(args[0], args[1], args[2])?)),
            "raise" if !args.is_empty() => return Err(EvalError::Raised(args[0])),
            "rnf" if args.len() >= 2 => {
                let noerr = self.eval_int(args[0])? != 0;
                self.rnf(noerr, args[1])?;
                Some((2, self.prim("I")))
            }
            "seq" if args.len() >= 2 => {
                self.reduce_node_whnf(args[0], 10_000)?;
                Some((2, args[1]))
            }
            "isint" if !args.is_empty() => {
                let root = self.reduce_node_whnf(args[0], 10_000)?;
                let n = match self.nodes[self.resolve(root)?.0] {
                    Node::Int(n) => n,
                    _ => -1,
                };
                Some((1, self.push_node(Node::Int(n))))
            }
            "thnum" if !args.is_empty() => {
                let thread = self.eval_thread_id(args[0])?;
                Some((1, self.push_node(Node::Int(thread))))
            }
            "S" if args.len() >= 3 => {
                let x = args[2];
                let left = self.app(args[0], x);
                let right = self.app(args[1], x);
                let n = self.app(left, right);
                Some((3, n))
            }
            "S'" if args.len() >= 4 => {
                let yw = self.app(args[1], args[3]);
                let zw = self.app(args[2], args[3]);
                let left = self.app(args[0], yw);
                let n = self.app(left, zw);
                Some((4, n))
            }
            "B" if args.len() >= 3 => {
                let yz = self.app(args[1], args[2]);
                let n = self.app(args[0], yz);
                Some((3, n))
            }
            "B'" if args.len() >= 4 => {
                let zw = self.app(args[2], args[3]);
                let xy = self.app(args[0], args[1]);
                let n = self.app(xy, zw);
                Some((4, n))
            }
            "B'" if args.len() >= 2 => {
                let xy = self.app(args[0], args[1]);
                let b = self.prim("B");
                let n = self.app(b, xy);
                Some((2, n))
            }
            "Z" if args.len() >= 3 => {
                let n = self.app(args[0], args[1]);
                Some((3, n))
            }
            "Z" if args.len() >= 2 => {
                let xy = self.app(args[0], args[1]);
                let k = self.prim("K");
                let n = self.app(k, xy);
                Some((2, n))
            }
            "J" if args.len() >= 3 => {
                let n = self.app(args[2], args[0]);
                Some((3, n))
            }
            "L" if args.len() >= 3 => {
                let n = self.app(args[1], args[0]);
                Some((3, n))
            }
            "KK" if args.len() >= 3 => Some((3, args[1])),
            "KA" if args.len() >= 3 => Some((3, args[2])),
            "C" if args.len() >= 3 => {
                let xz = self.app(args[0], args[2]);
                let n = self.app(xz, args[1]);
                Some((3, n))
            }
            "C'" if args.len() >= 4 => {
                let yw = self.app(args[1], args[3]);
                let xyw = self.app(args[0], yw);
                let n = self.app(xyw, args[2]);
                Some((4, n))
            }
            "P" if args.len() >= 3 => {
                let zx = self.app(args[2], args[0]);
                let n = self.app(zx, args[1]);
                Some((3, n))
            }
            "R" if args.len() >= 3 => {
                let yz = self.app(args[1], args[2]);
                let n = self.app(yz, args[0]);
                Some((3, n))
            }
            "R" if args.len() >= 2 => {
                let c = self.prim("C");
                let cy = self.app(c, args[1]);
                let n = self.app(cy, args[0]);
                Some((2, n))
            }
            "O" if args.len() >= 4 => {
                let wx = self.app(args[3], args[0]);
                let n = self.app(wx, args[1]);
                Some((4, n))
            }
            "K2" if args.len() >= 3 => Some((3, args[0])),
            "K2" if args.len() >= 2 => {
                let k = self.prim("K");
                let n = self.app(k, args[0]);
                Some((2, n))
            }
            "K3" if args.len() >= 4 => Some((4, args[0])),
            "K3" if args.len() >= 2 => {
                let k2 = self.prim("K2");
                let n = self.app(k2, args[0]);
                Some((2, n))
            }
            "K4" if args.len() >= 5 => Some((5, args[0])),
            "K4" if args.len() >= 2 => {
                let k3 = self.prim("K3");
                let n = self.app(k3, args[0]);
                Some((2, n))
            }
            "C'B" if args.len() >= 4 => {
                let yw = self.app(args[1], args[3]);
                let xz = self.app(args[0], args[2]);
                let n = self.app(xz, yw);
                Some((4, n))
            }
            "C'B" if args.len() >= 3 => {
                let xz = self.app(args[0], args[2]);
                let b = self.prim("B");
                let bxz = self.app(b, xz);
                let n = self.app(bxz, args[1]);
                Some((3, n))
            }
            "Y" if !args.is_empty() => {
                let y = self.prim("Y");
                let yy = self.app(y, args[0]);
                let n = self.app(args[0], yy);
                Some((1, n))
            }
            name if args.len() >= 2 && tag_index(name).is_some() => {
                let tag = self.push_node(Node::Int(tag_index(name).expect("checked tag") as i64));
                let ytag = self.app(args[1], tag);
                let n = self.app(ytag, args[0]);
                Some((2, n))
            }
            name if tuple_fields(name).is_some_and(|fields| args.len() > fields) => {
                let fields = tuple_fields(name).expect("checked tuple constructor");
                let mut n = args[fields];
                for arg in &args[..fields] {
                    n = self.app(n, *arg);
                }
                Some((fields + 1, n))
            }
            name if args.len() >= 2 => self
                .array_op(name, &args)?
                .or(self.foreign_ptr_op(name, &args)?)
                .or(self.stable_ptr_op(name, &args)?)
                .or(self.weak_ptr_op(name, &args)?)
                .or(self.bytes_op(name, &args)?)
                .or(self.float64_binop(name, &args)?)
                .or(self.float32_binop(name, &args)?)
                .or(self.int64_binop(name, &args)?)
                .or(self.int_binop(name, &args)?)
                .or(self.array_unop(name, &args)?)
                .or(self.bytes_unop(name, &args)?)
                .or(self.float64_unop(name, &args)?)
                .or(self.float32_unop(name, &args)?)
                .or(self.pointer_conversion(name, &args)?)
                .or(self.float_conversion(name, &args)?)
                .or(self.int64_unop(name, &args)?)
                .or(self.int_conversion(name, &args)?)
                .or(self.int_unop(name, &args)?),
            name if !args.is_empty() => self
                .array_unop(name, &args)?
                .or(self.foreign_ptr_unop(name, &args)?)
                .or(self.stable_ptr_unop(name, &args)?)
                .or(self.weak_ptr_unop(name, &args)?)
                .or(self.bytes_unop(name, &args)?)
                .or(self.float64_unop(name, &args)?)
                .or(self.float32_unop(name, &args)?)
                .or(self.pointer_conversion(name, &args)?)
                .or(self.float_conversion(name, &args)?)
                .or(self.int64_unop(name, &args)?)
                .or(self.int_conversion(name, &args)?)
                .or(self.int_unop(name, &args)?),
            _ => None,
        };

        let Some((mut used, mut node)) = rewrite else {
            return Ok(None);
        };
        let mut reductions = 1;
        if is_identity_alias(&name) {
            while reductions < budget && used < args.len() && self.is_identity_alias_node(node)? {
                node = args[used];
                used += 1;
                reductions += 1;
            }
        }
        let in_place = self.apply_remaining_spine(&mut node, &args[used..], &apps[used..]);
        Ok(Some(StepResult {
            node,
            in_place,
            reductions,
        }))
    }

    fn spine(&self, root: NodeId) -> Result<Spine, EvalError> {
        let mut node = self.resolve(root)?;
        let mut args = Vec::new();
        let mut apps = Vec::new();
        while let Node::App(fun, arg) = self.nodes[node.0] {
            args.push(self.resolve(arg)?);
            apps.push(node);
            node = self.resolve(fun)?;
        }
        args.reverse();
        apps.reverse();
        Ok(Spine {
            head: node,
            args,
            apps,
        })
    }

    fn apply_remaining_spine(
        &mut self,
        node: &mut NodeId,
        args: &[NodeId],
        apps: &[NodeId],
    ) -> bool {
        let mut in_place = false;
        for (arg, app) in args.iter().zip(apps) {
            self.nodes[app.0] = Node::App(*node, *arg);
            *node = *app;
            in_place = true;
        }
        in_place
    }

    fn is_identity_alias_node(&self, id: NodeId) -> Result<bool, EvalError> {
        let id = self.resolve(id)?;
        Ok(matches!(&self.nodes[id.0], Node::Prim(name) if is_identity_alias(name)))
    }

    fn app(&mut self, fun: NodeId, arg: NodeId) -> NodeId {
        self.push_node(Node::App(fun, arg))
    }

    fn prim(&mut self, name: &str) -> NodeId {
        self.push_node(Node::Prim(name.to_owned()))
    }

    fn world(&mut self) -> NodeId {
        self.push_node(Node::Int(99_999))
    }

    fn fst(&mut self) -> NodeId {
        let u = self.prim("U");
        let k = self.prim("K");
        self.app(u, k)
    }

    fn snd(&mut self) -> NodeId {
        let u = self.prim("U");
        let a = self.prim("A");
        self.app(u, a)
    }

    fn pair(&mut self, result: NodeId, world: NodeId) -> NodeId {
        let pair = self.prim("P");
        let result_pair = self.app(pair, result);
        self.app(result_pair, world)
    }

    fn just(&mut self, value: NodeId) -> NodeId {
        let z = self.prim("Z");
        let u = self.prim("U");
        let just = self.app(z, u);
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
        match self.reduce_node_whnf(action, 10_000) {
            Ok(result) => Ok(result),
            Err(EvalError::Raised(exn)) => {
                let handled = self.app(handler, exn);
                Ok(self.app(handled, world))
            }
            Err(err) => Err(err),
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
        let node = match op.apply(x, y)? {
            IntResult::Int(n) => self.push_node(Node::Int(n)),
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
        let node = self.push_node(Node::Int(op.apply(x)?));
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
        let node = match op.apply(x, y)? {
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
        let node = match op.apply(x)? {
            Int64UnResult::Int64(n) => self.push_node(Node::Int64(n)),
            Int64UnResult::Int(n) => self.push_node(Node::Int(n)),
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
                self.push_node(Node::Int(n))
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
                self.push_node(Node::Int(n as i64))
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
                self.push_node(Node::Int(n as i64))
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
                self.push_node(Node::Int((n.to_bits() as i32) as i64))
            }
            _ => return Ok(None),
        };
        Ok(Some((1, node)))
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
        Ok(Some((1, self.push_node(value))))
    }

    fn foreign_ptr_op(
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
                let foreign_ptr = self.push_node(Node::ForeignPtr {
                    bytes: None,
                    offset: 0,
                    ptr,
                    finalizer: None,
                });
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
        let node = match name {
            "bs2fp" => {
                let bytes_id = self.eval_bytes_id(args[0])?;
                let bytes = self.bytes(bytes_id)?.to_vec();
                let ptr = self.pointer_for_node(bytes_id, 0)?;
                self.push_node(Node::ForeignPtr {
                    bytes: Some(bytes),
                    offset: 0,
                    ptr,
                    finalizer: None,
                })
            }
            "fp2p" => {
                let foreign_ptr = self.eval_foreign_ptr_id(args[0])?;
                let ptr = self.foreign_ptr_value(foreign_ptr)?;
                self.push_node(Node::Ptr(ptr))
            }
            "fpnew" => {
                let ptr = self.eval_pointer_value(args[0])?;
                self.push_node(Node::ForeignPtr {
                    bytes: None,
                    offset: 0,
                    ptr,
                    finalizer: None,
                })
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
                let array = self.push_node(Node::Array(vec![args[1]; len]));
                Some((3, self.pair(array, args[2])))
            }
            "A.alloc" => {
                let len = int_to_usize(self.eval_int(args[0])?)?;
                Some((2, self.push_node(Node::Array(vec![args[1]; len]))))
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
                let copy = self.push_node(Node::Array(items));
                self.pair(copy, args[1])
            }
            "A.copy" => {
                let array = self.eval_array_id(args[0])?;
                let items = self.array(array)?.to_vec();
                self.push_node(Node::Array(items))
            }
            "A.size" if args.len() >= 2 => {
                let array = self.eval_array_id(args[0])?;
                let len =
                    i64::try_from(self.array(array)?.len()).map_err(|_| EvalError::Overflow)?;
                let size = self.push_node(Node::Int(len));
                self.pair(size, args[1])
            }
            "A.size" => {
                let array = self.eval_array_id(args[0])?;
                let len =
                    i64::try_from(self.array(array)?.len()).map_err(|_| EvalError::Overflow)?;
                self.push_node(Node::Int(len))
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
                let weak = self.new_weak_ptr(args[1], Some(args[2]));
                Some((4, self.pair(weak, args[3])))
            }
            "Wknewfin" if args.len() >= 3 => {
                let weak = self.new_weak_ptr(args[1], Some(args[2]));
                Some((3, weak))
            }
            "Wknew" if args.len() >= 3 => {
                let weak = self.new_weak_ptr(args[1], None);
                Some((3, self.pair(weak, args[2])))
            }
            "Wknew" if args.len() >= 2 => {
                let weak = self.new_weak_ptr(args[1], None);
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
        let rewrite = match name {
            "packCString" if args.len() >= 2 => {
                let ptr = self.eval_pointer_value(args[0])?;
                let bytes = self.read_c_string(ptr)?;
                let bytes = self.push_node(Node::Bytes(bytes));
                Some((2, self.pair(bytes, args[1])))
            }
            "packCStringLen" if args.len() >= 3 => {
                let ptr = self.eval_pointer_value(args[0])?;
                let len = int_to_usize(self.eval_int(args[1])?)?;
                let bytes = self.read_pointer_bytes(ptr, len)?;
                let bytes = self.push_node(Node::Bytes(bytes));
                Some((3, self.pair(bytes, args[2])))
            }
            "packCStringLen" => {
                let ptr = self.eval_pointer_value(args[0])?;
                let len = int_to_usize(self.eval_int(args[1])?)?;
                let bytes = self.read_pointer_bytes(ptr, len)?;
                Some((2, self.push_node(Node::Bytes(bytes))))
            }
            "bsgrab" if args.len() >= 2 => {
                let ptr = self.eval_pointer_value(args[0])?;
                let bytes = self.read_c_string(ptr)?;
                let bytes = self.push_node(Node::Bytes(bytes));
                Some((2, self.pair(bytes, args[1])))
            }
            "bsgrablen" if args.len() >= 3 => {
                let ptr = self.eval_pointer_value(args[0])?;
                let len = int_to_usize(self.eval_int(args[1])?)?;
                let bytes = self.read_pointer_bytes(ptr, len)?;
                let bytes = self.push_node(Node::Bytes(bytes));
                Some((3, self.pair(bytes, args[2])))
            }
            "bsgrablen" => {
                let ptr = self.eval_pointer_value(args[0])?;
                let len = int_to_usize(self.eval_int(args[1])?)?;
                let bytes = self.read_pointer_bytes(ptr, len)?;
                Some((2, self.push_node(Node::Bytes(bytes))))
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
                let byte = self
                    .bytes(bytes)?
                    .get(index)
                    .copied()
                    .ok_or(EvalError::InvalidByteString)?;
                let byte = self.push_node(Node::Int(byte as i64));
                Some((3, self.pair(byte, args[2])))
            }
            "bsread" => {
                let bytes = self.eval_bytes_id(args[0])?;
                let index = int_to_usize(self.eval_int(args[1])?)?;
                let byte = self
                    .bytes(bytes)?
                    .get(index)
                    .copied()
                    .ok_or(EvalError::InvalidByteString)?;
                Some((2, self.push_node(Node::Int(byte as i64))))
            }
            "bswrite" if args.len() >= 4 => {
                let bytes = self.eval_bytes_id(args[0])?;
                let index = int_to_usize(self.eval_int(args[1])?)?;
                let byte = self.eval_int(args[2])? as u8;
                let slot = self
                    .bytes_mut(bytes)?
                    .get_mut(index)
                    .ok_or(EvalError::InvalidByteString)?;
                *slot = byte;
                let unit = self.prim("I");
                Some((4, self.pair(unit, args[3])))
            }
            "bswrite" if args.len() >= 3 => {
                let bytes = self.eval_bytes_id(args[0])?;
                let index = int_to_usize(self.eval_int(args[1])?)?;
                let byte = self.eval_int(args[2])? as u8;
                let slot = self
                    .bytes_mut(bytes)?
                    .get_mut(index)
                    .ok_or(EvalError::InvalidByteString)?;
                *slot = byte;
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
            "bs++" => {
                let mut bytes = self.eval_bytes(args[0])?;
                bytes.extend(self.eval_bytes(args[1])?);
                Some((2, self.push_node(Node::Bytes(bytes))))
            }
            "bs++." => {
                let mut bytes = self.eval_bytes(args[0])?;
                bytes.push(b'.');
                bytes.extend(self.eval_bytes(args[1])?);
                Some((2, self.push_node(Node::Bytes(bytes))))
            }
            "bs==" | "bs/=" | "bs<" | "bs<=" | "bs>" | "bs>=" | "bscmp" => {
                let cmp = self.eval_bytes(args[0])?.cmp(&self.eval_bytes(args[1])?);
                let node = match name {
                    "bs==" => self.prim(if cmp == Ordering::Equal { "A" } else { "K" }),
                    "bs/=" => self.prim(if cmp != Ordering::Equal { "A" } else { "K" }),
                    "bs<" => self.prim(if cmp == Ordering::Less { "A" } else { "K" }),
                    "bs<=" => self.prim(if cmp != Ordering::Greater { "A" } else { "K" }),
                    "bs>" => self.prim(if cmp == Ordering::Greater { "A" } else { "K" }),
                    "bs>=" => self.prim(if cmp != Ordering::Less { "A" } else { "K" }),
                    "bscmp" => self.ordering(cmp),
                    _ => unreachable!(),
                };
                Some((2, node))
            }
            "bsreplicate" => {
                let len = int_to_usize(self.eval_int(args[0])?)?;
                let byte = self.eval_int(args[1])? as u8;
                Some((2, self.push_node(Node::Bytes(vec![byte; len]))))
            }
            "bsindex" => {
                let bytes = self.eval_bytes(args[0])?;
                let index = int_to_usize(self.eval_int(args[1])?)?;
                let byte = bytes
                    .get(index)
                    .copied()
                    .ok_or(EvalError::InvalidByteString)?;
                Some((2, self.push_node(Node::Int(byte as i64))))
            }
            "bssubstr" if args.len() >= 3 => {
                let bytes = self.eval_bytes(args[0])?;
                let offset = int_to_usize(self.eval_int(args[1])?)?;
                let len = int_to_usize(self.eval_int(args[2])?)?;
                let end = offset
                    .checked_add(len)
                    .filter(|end| *end <= bytes.len())
                    .ok_or(EvalError::InvalidByteString)?;
                Some((3, self.push_node(Node::Bytes(bytes[offset..end].to_vec()))))
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
        let node = match name {
            "packCString" => {
                let ptr = self.eval_pointer_value(args[0])?;
                let bytes = self.read_c_string(ptr)?;
                self.push_node(Node::Bytes(bytes))
            }
            "bsgrab" => {
                let ptr = self.eval_pointer_value(args[0])?;
                let bytes = self.read_c_string(ptr)?;
                self.push_node(Node::Bytes(bytes))
            }
            "bslength" => {
                let len = i64::try_from(self.eval_bytes(args[0])?.len())
                    .map_err(|_| EvalError::Overflow)?;
                self.push_node(Node::Int(len))
            }
            "headUTF8" => {
                let (codepoint, _) = head_utf8(&self.eval_bytes(args[0])?)?;
                self.push_node(Node::Int(codepoint as i64))
            }
            "tailUTF8" => {
                let bytes = self.eval_bytes(args[0])?;
                let (_, offset) = head_utf8(&bytes)?;
                self.push_node(Node::Bytes(bytes[offset..].to_vec()))
            }
            "bsunpack" => {
                let bytes = self.eval_bytes(args[0])?;
                let values = bytes.into_iter().map(i64::from);
                self.int_list(values)
            }
            "fromUTF8" => {
                let bytes = self.eval_bytes(args[0])?;
                let values = decode_utf8_bytes(&bytes)?;
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
                Node::Prim("I".to_owned())
            }
            "js_eval_run" => {
                let ptr = self.eval_pointer_value(args[0])?;
                let bytes = self.read_c_string(ptr)?;
                host_js_eval_run(&bytes)?;
                Node::Prim("I".to_owned())
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
                Node::Prim("I".to_owned())
            }
            "new_mpz" => self.new_mpz_node()?,
            "mpz_init_set_si" => {
                let ptr = self.eval_pointer_value(args[0])?;
                let value = self.eval_int(args[1])?;
                self.write_mpz_value(ptr, MpzValue::from_i64(value))?;
                Node::Prim("I".to_owned())
            }
            "mpz_init_set_ui" => {
                let ptr = self.eval_pointer_value(args[0])?;
                let value = self.eval_int(args[1])? as u64;
                self.write_mpz_value(ptr, MpzValue::from_u64(value))?;
                Node::Prim("I".to_owned())
            }
            "mpz_init_set_si64" => {
                let ptr = self.eval_pointer_value(args[0])?;
                let value = self.eval_int64(args[1])?;
                self.write_mpz_value(ptr, MpzValue::from_i64(value))?;
                Node::Prim("I".to_owned())
            }
            "mpz_init_set_ui64" => {
                let ptr = self.eval_pointer_value(args[0])?;
                let value = self.eval_int64(args[1])? as u64;
                self.write_mpz_value(ptr, MpzValue::from_u64(value))?;
                Node::Prim("I".to_owned())
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
                Node::Prim("I".to_owned())
            }
            "mpz_neg" => {
                let dst = self.eval_pointer_value(args[0])?;
                let src = self.eval_pointer_value(args[1])?;
                let mut value = self.mpz_value(src)?;
                if !value.is_zero() {
                    value.negative = !value.negative;
                }
                self.write_mpz_value(dst, value)?;
                Node::Prim("I".to_owned())
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
                Node::Prim("I".to_owned())
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
                Node::Prim("I".to_owned())
            }
            "mpz_fdiv_q_2exp" => {
                let dst = self.eval_pointer_value(args[0])?;
                let src = self.eval_pointer_value(args[1])?;
                let shift = int_to_usize(self.eval_int(args[2])?)?;
                self.write_mpz_value(dst, self.mpz_value(src)?.fdiv_q_2exp(shift))?;
                Node::Prim("I".to_owned())
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
                Node::Prim("I".to_owned())
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
            "&closeb" => Node::FunPtr("closeb".to_owned()),
            "&free" => Node::FunPtr("free".to_owned()),
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
                Node::Prim("I".to_owned())
            }
            "memcpy" | "memmove" => {
                let dst = self.eval_pointer_value(args[0])?;
                let src = self.eval_pointer_value(args[1])?;
                let len = int_to_usize(self.eval_int(args[2])?)?;
                let bytes = self.read_pointer_bytes(src, len)?;
                self.write_pointer_bytes(dst, &bytes)?;
                Node::Prim("I".to_owned())
            }
            "strcpy" => {
                let dst = self.eval_pointer_value(args[0])?;
                let src = self.eval_pointer_value(args[1])?;
                let mut bytes = self.read_c_string(src)?;
                bytes.push(0);
                self.write_pointer_bytes(dst, &bytes)?;
                Node::Prim("I".to_owned())
            }
            "strlen" => {
                let ptr = self.eval_pointer_value(args[0])?;
                let len = self.read_c_string(ptr)?.len();
                Node::Int(i64::try_from(len).map_err(|_| EvalError::Overflow)?)
            }
            "md5String" => {
                let input = self.eval_pointer_value(args[0])?;
                let result = self.eval_pointer_value(args[1])?;
                let bytes = self.read_c_string(input)?;
                self.write_pointer_bytes(result, &md5_bytes(&bytes))?;
                Node::Prim("I".to_owned())
            }
            "md5Array" => {
                let input = self.eval_pointer_value(args[0])?;
                let result = self.eval_pointer_value(args[1])?;
                let len = int_to_usize(self.eval_int(args[2])?)?;
                let bytes = self.read_pointer_bytes(input, len)?;
                self.write_pointer_bytes(result, &md5_bytes(&bytes))?;
                Node::Prim("I".to_owned())
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
                Node::Prim("I".to_owned())
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
                let ptr = match executable_path_bytes() {
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
                Node::Prim("I".to_owned())
            }
            "getcpu" => {
                let sec_ptr = self.eval_pointer_value(args[0])?;
                let nsec_ptr = self.eval_pointer_value(args[1])?;
                let (sec, nsec) = cpu_time();
                self.poke_unsigned(sec_ptr, size_of::<std::os::raw::c_ulong>(), sec)?;
                self.poke_unsigned(nsec_ptr, size_of::<std::os::raw::c_ulong>(), nsec)?;
                Node::Prim("I".to_owned())
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
                Node::Prim("I".to_owned())
            }
            "flushb" => {
                let ptr = self.eval_pointer_value(args[0])?;
                self.flush_bfile(ptr)?;
                Node::Prim("I".to_owned())
            }
            "getb" => {
                let ptr = self.eval_pointer_value(args[0])?;
                Node::Int(self.get_bfile_byte(ptr)?)
            }
            "putb" => {
                let byte = self.eval_int(args[0])?;
                let ptr = self.eval_pointer_value(args[1])?;
                self.put_bfile_byte(ptr, byte)?;
                Node::Prim("I".to_owned())
            }
            "ungetb" => {
                let byte = self.eval_int(args[0])?;
                let ptr = self.eval_pointer_value(args[1])?;
                self.unget_bfile_byte(ptr, byte)?;
                Node::Prim("I".to_owned())
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
            "peekPtr" => {
                let ptr = self.eval_pointer_value(args[0])?;
                Node::Ptr(self.peek_signed(ptr, 8)?)
            }
            "pokePtr" => {
                let ptr = self.eval_pointer_value(args[0])?;
                let value = self.eval_pointer_value(args[1])?;
                self.poke_signed(ptr, 8, value)?;
                Node::Prim("I".to_owned())
            }
            "peekWord" => {
                let ptr = self.eval_pointer_value(args[0])?;
                Node::Int(self.peek_unsigned(ptr, 8)? as i64)
            }
            "pokeWord" => {
                let ptr = self.eval_pointer_value(args[0])?;
                let value = self.eval_int(args[1])?;
                self.poke_unsigned(ptr, 8, value as u64)?;
                Node::Prim("I".to_owned())
            }
            "peek_uint8" => {
                let ptr = self.eval_pointer_value(args[0])?;
                Node::Int(self.peek_unsigned(ptr, 1)? as i64)
            }
            "poke_uint8" => {
                let ptr = self.eval_pointer_value(args[0])?;
                let value = self.eval_int(args[1])?;
                self.poke_unsigned(ptr, 1, value as u64)?;
                Node::Prim("I".to_owned())
            }
            "peek_uint16" => {
                let ptr = self.eval_pointer_value(args[0])?;
                Node::Int(self.peek_unsigned(ptr, 2)? as i64)
            }
            "poke_uint16" => {
                let ptr = self.eval_pointer_value(args[0])?;
                let value = self.eval_int(args[1])?;
                self.poke_unsigned(ptr, 2, value as u64)?;
                Node::Prim("I".to_owned())
            }
            "peek_uint32" => {
                let ptr = self.eval_pointer_value(args[0])?;
                Node::Int(self.peek_unsigned(ptr, 4)? as i64)
            }
            "poke_uint32" => {
                let ptr = self.eval_pointer_value(args[0])?;
                let value = self.eval_int(args[1])?;
                self.poke_unsigned(ptr, 4, value as u64)?;
                Node::Prim("I".to_owned())
            }
            "peek_uint64" => {
                let ptr = self.eval_pointer_value(args[0])?;
                Node::Int64(self.peek_unsigned(ptr, 8)? as i64)
            }
            "poke_uint64" => {
                let ptr = self.eval_pointer_value(args[0])?;
                let value = self.eval_int64(args[1])?;
                self.poke_unsigned(ptr, 8, value as u64)?;
                Node::Prim("I".to_owned())
            }
            "peek_int8" => {
                let ptr = self.eval_pointer_value(args[0])?;
                Node::Int(self.peek_signed(ptr, 1)?)
            }
            "poke_int8" => {
                let ptr = self.eval_pointer_value(args[0])?;
                let value = self.eval_int(args[1])?;
                self.poke_signed(ptr, 1, value)?;
                Node::Prim("I".to_owned())
            }
            "peek_int16" => {
                let ptr = self.eval_pointer_value(args[0])?;
                Node::Int(self.peek_signed(ptr, 2)?)
            }
            "poke_int16" => {
                let ptr = self.eval_pointer_value(args[0])?;
                let value = self.eval_int(args[1])?;
                self.poke_signed(ptr, 2, value)?;
                Node::Prim("I".to_owned())
            }
            "peek_int32" => {
                let ptr = self.eval_pointer_value(args[0])?;
                Node::Int(self.peek_signed(ptr, 4)?)
            }
            "poke_int32" => {
                let ptr = self.eval_pointer_value(args[0])?;
                let value = self.eval_int(args[1])?;
                self.poke_signed(ptr, 4, value)?;
                Node::Prim("I".to_owned())
            }
            "peek_int64" => {
                let ptr = self.eval_pointer_value(args[0])?;
                Node::Int64(self.peek_signed(ptr, 8)?)
            }
            "poke_int64" => {
                let ptr = self.eval_pointer_value(args[0])?;
                let value = self.eval_int64(args[1])?;
                self.poke_signed(ptr, 8, value)?;
                Node::Prim("I".to_owned())
            }
            "peek_char" => {
                let ptr = self.eval_pointer_value(args[0])?;
                Node::Int(self.peek_c_char(ptr)?)
            }
            "poke_char" => {
                let ptr = self.eval_pointer_value(args[0])?;
                let value = self.eval_int(args[1])?;
                self.poke_c_char(ptr, value)?;
                Node::Prim("I".to_owned())
            }
            "peek_schar" => {
                let ptr = self.eval_pointer_value(args[0])?;
                Node::Int(self.peek_signed(ptr, size_of::<std::os::raw::c_schar>())?)
            }
            "poke_schar" => {
                let ptr = self.eval_pointer_value(args[0])?;
                let value = self.eval_int(args[1])?;
                self.poke_signed(ptr, size_of::<std::os::raw::c_schar>(), value)?;
                Node::Prim("I".to_owned())
            }
            "peek_uchar" => {
                let ptr = self.eval_pointer_value(args[0])?;
                Node::Int(self.peek_unsigned(ptr, size_of::<std::os::raw::c_uchar>())? as i64)
            }
            "poke_uchar" => {
                let ptr = self.eval_pointer_value(args[0])?;
                let value = self.eval_int(args[1])?;
                self.poke_unsigned(ptr, size_of::<std::os::raw::c_uchar>(), value as u64)?;
                Node::Prim("I".to_owned())
            }
            "peek_short" => {
                let ptr = self.eval_pointer_value(args[0])?;
                Node::Int(self.peek_signed(ptr, size_of::<std::os::raw::c_short>())?)
            }
            "poke_short" => {
                let ptr = self.eval_pointer_value(args[0])?;
                let value = self.eval_int(args[1])?;
                self.poke_signed(ptr, size_of::<std::os::raw::c_short>(), value)?;
                Node::Prim("I".to_owned())
            }
            "peek_ushort" => {
                let ptr = self.eval_pointer_value(args[0])?;
                Node::Int(self.peek_unsigned(ptr, size_of::<std::os::raw::c_ushort>())? as i64)
            }
            "poke_ushort" => {
                let ptr = self.eval_pointer_value(args[0])?;
                let value = self.eval_int(args[1])?;
                self.poke_unsigned(ptr, size_of::<std::os::raw::c_ushort>(), value as u64)?;
                Node::Prim("I".to_owned())
            }
            "peek_int" => {
                let ptr = self.eval_pointer_value(args[0])?;
                Node::Int(self.peek_signed(ptr, size_of::<std::os::raw::c_int>())?)
            }
            "poke_int" => {
                let ptr = self.eval_pointer_value(args[0])?;
                let value = self.eval_int(args[1])?;
                self.poke_signed(ptr, size_of::<std::os::raw::c_int>(), value)?;
                Node::Prim("I".to_owned())
            }
            "peek_uint" => {
                let ptr = self.eval_pointer_value(args[0])?;
                Node::Int(self.peek_unsigned(ptr, size_of::<std::os::raw::c_uint>())? as i64)
            }
            "poke_uint" => {
                let ptr = self.eval_pointer_value(args[0])?;
                let value = self.eval_int(args[1])?;
                self.poke_unsigned(ptr, size_of::<std::os::raw::c_uint>(), value as u64)?;
                Node::Prim("I".to_owned())
            }
            "peek_long" => {
                let ptr = self.eval_pointer_value(args[0])?;
                Node::Int(self.peek_signed(ptr, size_of::<std::os::raw::c_long>())?)
            }
            "poke_long" => {
                let ptr = self.eval_pointer_value(args[0])?;
                let value = self.eval_int(args[1])?;
                self.poke_signed(ptr, size_of::<std::os::raw::c_long>(), value)?;
                Node::Prim("I".to_owned())
            }
            "peek_ulong" => {
                let ptr = self.eval_pointer_value(args[0])?;
                Node::Int(self.peek_unsigned(ptr, size_of::<std::os::raw::c_ulong>())? as i64)
            }
            "poke_ulong" => {
                let ptr = self.eval_pointer_value(args[0])?;
                let value = self.eval_int(args[1])?;
                self.poke_unsigned(ptr, size_of::<std::os::raw::c_ulong>(), value as u64)?;
                Node::Prim("I".to_owned())
            }
            "peek_llong" => {
                let ptr = self.eval_pointer_value(args[0])?;
                Node::Int(self.peek_signed(ptr, size_of::<std::os::raw::c_longlong>())?)
            }
            "poke_llong" => {
                let ptr = self.eval_pointer_value(args[0])?;
                let value = self.eval_int(args[1])?;
                self.poke_signed(ptr, size_of::<std::os::raw::c_longlong>(), value)?;
                Node::Prim("I".to_owned())
            }
            "peek_ullong" => {
                let ptr = self.eval_pointer_value(args[0])?;
                Node::Int(self.peek_unsigned(ptr, size_of::<std::os::raw::c_ulonglong>())? as i64)
            }
            "poke_ullong" => {
                let ptr = self.eval_pointer_value(args[0])?;
                let value = self.eval_int(args[1])?;
                self.poke_unsigned(ptr, size_of::<std::os::raw::c_ulonglong>(), value as u64)?;
                Node::Prim("I".to_owned())
            }
            "peek_size_t" => {
                let ptr = self.eval_pointer_value(args[0])?;
                Node::Int(self.peek_unsigned(ptr, size_of::<usize>())? as i64)
            }
            "poke_size_t" => {
                let ptr = self.eval_pointer_value(args[0])?;
                let value = self.eval_int(args[1])?;
                self.poke_unsigned(ptr, size_of::<usize>(), value as u64)?;
                Node::Prim("I".to_owned())
            }
            "peek_flt32" => {
                let ptr = self.eval_pointer_value(args[0])?;
                Node::Float32(f32::from_ne_bytes(self.peek_array(ptr)?))
            }
            "poke_flt32" => {
                let ptr = self.eval_pointer_value(args[0])?;
                let value = self.eval_float32(args[1])?;
                self.write_pointer_bytes(ptr, &value.to_ne_bytes())?;
                Node::Prim("I".to_owned())
            }
            "peek_flt64" => {
                let ptr = self.eval_pointer_value(args[0])?;
                Node::Float64(f64::from_ne_bytes(self.peek_array(ptr)?))
            }
            "poke_flt64" => {
                let ptr = self.eval_pointer_value(args[0])?;
                let value = self.eval_float64(args[1])?;
                self.write_pointer_bytes(ptr, &value.to_ne_bytes())?;
                Node::Prim("I".to_owned())
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
        let result = self.push_node(result);
        Ok(Some((arity + 1, self.pair(result, args[arity]))))
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
                Node::Prim("I".to_owned())
            }
            b'D' => Node::Float64(host_js_call_double(body, arity, &js_args)?),
            b'F' => Node::Float32(host_js_call_double(body, arity, &js_args)? as f32),
            b'P' => Node::Ptr(i64::from(host_js_call_ptr(body, arity, &js_args)?)),
            b'B' => Node::Prim(
                if host_js_call_bool(body, arity, &js_args)? {
                    "A"
                } else {
                    "K"
                }
                .to_owned(),
            ),
            b'S' => Node::Bytes(host_js_call_string(body, arity, &js_args)?),
            b'I' | b'U' => Node::Int(i64::from(host_js_call_int(body, arity, &js_args)?)),
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
        let stable_ptr = self.new_stable_ptr_handle(args[0])?;
        let object = match host_js_make_wrapper(tags.as_bytes(), stable_ptr) {
            Ok(object) => object,
            Err(err) => {
                let _ = self.free_stable_ptr(usize::try_from(stable_ptr).unwrap_or(usize::MAX));
                return Err(err);
            }
        };
        let result = self.push_node(self.js_object_node(object));
        Ok(Some((2, self.pair(result, args[1]))))
    }

    fn eval_ffi_name(&mut self, id: NodeId) -> Result<String, EvalError> {
        let root = self.reduce_node_whnf(id, 10_000)?;
        let root = self.resolve(root)?;
        let bytes = match self.nodes[root.0].clone() {
            Node::Bytes(bytes) | Node::MutableBytes { bytes, .. } => bytes,
            _ => self.eval_char_list(root)?,
        };
        String::from_utf8(bytes).map_err(|_| EvalError::InvalidByteString)
    }

    fn eval_char_list(&mut self, mut id: NodeId) -> Result<Vec<u8>, EvalError> {
        let mut out = Vec::new();
        for _ in 0..10_000 {
            let root = self.reduce_node_whnf(id, 10_000)?;
            let root = self.resolve(root)?;
            match self.nodes[root.0].clone() {
                Node::Prim(name) if name == "K" => return Ok(out),
                Node::App(fun, tail) => {
                    let fun = self.resolve(fun)?;
                    let Node::App(cons, head) = self.nodes[fun.0].clone() else {
                        return Err(EvalError::InvalidByteString);
                    };
                    let cons = self.resolve(cons)?;
                    match &self.nodes[cons.0] {
                        Node::Prim(name) if name == "O" => {
                            out.extend(modified_utf8(self.eval_int(head)?)?);
                            id = tail;
                        }
                        _ => return Err(EvalError::InvalidByteString),
                    }
                }
                _ => return Err(EvalError::InvalidByteString),
            }
        }
        Err(EvalError::StepLimit { limit: 10_000 })
    }

    fn eval_int(&mut self, id: NodeId) -> Result<i64, EvalError> {
        let root = self.reduce_node_whnf(id, 10_000)?;
        match self.nodes[self.resolve(root)?.0] {
            Node::Int(n) => Ok(n),
            _ => Err(EvalError::ExpectedInt(root)),
        }
    }

    fn eval_int64(&mut self, id: NodeId) -> Result<i64, EvalError> {
        let root = self.reduce_node_whnf(id, 10_000)?;
        match self.nodes[self.resolve(root)?.0] {
            Node::Int64(n) => Ok(n),
            _ => Err(EvalError::ExpectedInt64(root)),
        }
    }

    fn eval_float64(&mut self, id: NodeId) -> Result<f64, EvalError> {
        let root = self.reduce_node_whnf(id, 10_000)?;
        match self.nodes[self.resolve(root)?.0] {
            Node::Float64(n) => Ok(n),
            _ => Err(EvalError::ExpectedFloat64(root)),
        }
    }

    fn eval_float32(&mut self, id: NodeId) -> Result<f32, EvalError> {
        let root = self.reduce_node_whnf(id, 10_000)?;
        match self.nodes[self.resolve(root)?.0] {
            Node::Float32(n) => Ok(n),
            _ => Err(EvalError::ExpectedFloat32(root)),
        }
    }

    fn eval_bool(&mut self, id: NodeId) -> Result<bool, EvalError> {
        let root = self.reduce_node_whnf(id, 10_000)?;
        match &self.nodes[self.resolve(root)?.0] {
            Node::Prim(name) if name == "A" => Ok(true),
            Node::Prim(name) if name == "K" => Ok(false),
            _ => Err(EvalError::ExpectedInt(root)),
        }
    }

    fn eval_thread_id(&mut self, id: NodeId) -> Result<i64, EvalError> {
        let root = self.reduce_node_whnf(id, 10_000)?;
        match self.nodes[self.resolve(root)?.0] {
            Node::ThreadId(n) => Ok(n),
            _ => Err(EvalError::ExpectedThreadId(root)),
        }
    }

    fn eval_pointer_value(&mut self, id: NodeId) -> Result<i64, EvalError> {
        let root = self.reduce_node_whnf(id, 10_000)?;
        match &self.nodes[self.resolve(root)?.0] {
            Node::Int(n) | Node::Ptr(n) | Node::RawFunPtr(n) | Node::ThreadId(n) => Ok(*n),
            Node::Prim(name) => std_handle_ptr(name).ok_or(EvalError::ExpectedPointer(root)),
            _ => Err(EvalError::ExpectedPointer(root)),
        }
    }

    fn eval_foreign_ptr_id(&mut self, id: NodeId) -> Result<NodeId, EvalError> {
        let root = self.reduce_node_whnf(id, 10_000)?;
        let id = self.resolve(root)?;
        match &self.nodes[id.0] {
            Node::ForeignPtr { .. } => Ok(id),
            Node::Prim(name) if std_handle(name).is_some() => Ok(id),
            _ => Err(EvalError::ExpectedForeignPtr(root)),
        }
    }

    fn eval_bytes(&mut self, id: NodeId) -> Result<Vec<u8>, EvalError> {
        let id = self.eval_bytes_id(id)?;
        Ok(self.bytes(id)?.to_vec())
    }

    fn eval_bytes_id(&mut self, id: NodeId) -> Result<NodeId, EvalError> {
        let root = self.reduce_node_whnf(id, 10_000)?;
        let id = self.resolve(root)?;
        match self.nodes[id.0] {
            Node::Bytes(_) | Node::MutableBytes { .. } => Ok(id),
            _ => Err(EvalError::ExpectedBytes(root)),
        }
    }

    fn eval_array_id(&mut self, id: NodeId) -> Result<NodeId, EvalError> {
        let root = self.reduce_node_whnf(id, 10_000)?;
        let id = self.resolve(root)?;
        match self.nodes[id.0] {
            Node::Array(_) => Ok(id),
            _ => Err(EvalError::ExpectedArray(root)),
        }
    }

    fn array(&self, id: NodeId) -> Result<&[NodeId], EvalError> {
        match &self.nodes[id.0] {
            Node::Array(items) => Ok(items),
            _ => Err(EvalError::ExpectedArray(id)),
        }
    }

    fn array_mut(&mut self, id: NodeId) -> Result<&mut Vec<NodeId>, EvalError> {
        match &mut self.nodes[id.0] {
            Node::Array(items) => Ok(items),
            _ => Err(EvalError::ExpectedArray(id)),
        }
    }

    fn bytes(&self, id: NodeId) -> Result<&[u8], EvalError> {
        match &self.nodes[id.0] {
            Node::Bytes(bytes) | Node::MutableBytes { bytes, .. } => Ok(bytes),
            _ => Err(EvalError::ExpectedBytes(id)),
        }
    }

    fn bytes_mut(&mut self, id: NodeId) -> Result<&mut Vec<u8>, EvalError> {
        match &mut self.nodes[id.0] {
            Node::Bytes(bytes) | Node::MutableBytes { bytes, .. } => Ok(bytes),
            _ => Err(EvalError::ExpectedBytes(id)),
        }
    }

    fn new_mutable_bytes(&mut self, size: usize, capacity: usize) -> Result<NodeId, EvalError> {
        if size > capacity {
            return Err(EvalError::InvalidByteString);
        }
        let mut bytes = Vec::with_capacity(capacity);
        bytes.resize(size, 0);
        Ok(self.push_node(Node::MutableBytes { bytes, capacity }))
    }

    fn freeze_bytes(&mut self, id: NodeId) -> Result<NodeId, EvalError> {
        let frozen = match &mut self.nodes[id.0] {
            Node::Bytes(_) => return Ok(id),
            Node::MutableBytes { bytes, .. } => std::mem::take(bytes),
            _ => return Err(EvalError::ExpectedBytes(id)),
        };
        self.nodes[id.0] = Node::Bytes(frozen);
        Ok(id)
    }

    fn append_byte(&mut self, id: NodeId, byte: u8) -> Result<(), EvalError> {
        match &mut self.nodes[id.0] {
            Node::Bytes(bytes) => {
                bytes.push(byte);
                Ok(())
            }
            Node::MutableBytes { bytes, capacity } => {
                if bytes.len() >= *capacity {
                    *capacity = (*capacity)
                        .checked_add(*capacity / 2)
                        .and_then(|capacity| capacity.checked_add(2))
                        .ok_or(EvalError::Overflow)?;
                    if *capacity < bytes.len() {
                        return Err(EvalError::Overflow);
                    }
                    if *capacity > bytes.capacity() {
                        bytes.reserve(*capacity - bytes.capacity());
                    }
                }
                bytes.push(byte);
                Ok(())
            }
            _ => Err(EvalError::ExpectedBytes(id)),
        }
    }

    fn append_bytes(&mut self, id: NodeId, bytes: &[u8]) -> Result<(), EvalError> {
        for &byte in bytes {
            self.append_byte(id, byte)?;
        }
        Ok(())
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
        let bigint = self.push_node(Node::BigInt(b"0".to_vec()));
        let ptr = self.pointer_for_node(bigint, 0)?;
        Ok(Node::ForeignPtr {
            bytes: None,
            offset: 0,
            ptr,
            finalizer: None,
        })
    }

    fn mpz_node_id(&self, ptr: i64) -> Result<NodeId, EvalError> {
        let (slot, offset) = self.decode_pointer(ptr)?;
        if offset != 0 {
            return Err(EvalError::ExpectedForeignPtr(NodeId(slot)));
        }
        let id = NodeId(slot);
        match self.nodes.get(id.0) {
            Some(Node::BigInt(_)) => Ok(id),
            _ => Err(EvalError::ExpectedForeignPtr(id)),
        }
    }

    fn mpz_decimal_bytes_for_ptr(&self, ptr: i64) -> Option<&[u8]> {
        let (slot, offset) = self.decode_pointer(ptr).ok()?;
        if offset != 0 {
            return None;
        }
        match self.nodes.get(slot)? {
            Node::BigInt(bytes) => Some(bytes),
            _ => None,
        }
    }

    fn mpz_value(&self, ptr: i64) -> Result<MpzValue, EvalError> {
        let id = self.mpz_node_id(ptr)?;
        let Node::BigInt(bytes) = &self.nodes[id.0] else {
            return Err(EvalError::ExpectedForeignPtr(id));
        };
        MpzValue::parse_decimal(bytes).map_err(|_| EvalError::InvalidByteString)
    }

    fn write_mpz_value(&mut self, ptr: i64, value: MpzValue) -> Result<(), EvalError> {
        let id = self.mpz_node_id(ptr)?;
        self.nodes[id.0] = Node::BigInt(value.to_decimal_bytes());
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

    fn pointer_for_node(&self, id: NodeId, offset: usize) -> Result<i64, EvalError> {
        let base = i64::try_from(id.0).map_err(|_| EvalError::Overflow)?;
        let offset = i64::try_from(offset).map_err(|_| EvalError::Overflow)?;
        if offset >= (1_i64 << 32) {
            return Err(EvalError::Overflow);
        }
        base.checked_shl(32)
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
            return Err(EvalError::InvalidByteString);
        }
        let block = usize::try_from(ptr >> 32).map_err(|_| EvalError::InvalidByteString)?;
        let offset =
            usize::try_from(ptr & 0xffff_ffff).map_err(|_| EvalError::InvalidByteString)?;
        Ok((block, offset))
    }

    fn decode_allocation_pointer(&self, ptr: i64) -> Result<(usize, usize), EvalError> {
        if ptr < ALLOCATION_PTR_BASE || ptr >= 0 {
            return Err(EvalError::InvalidByteString);
        }
        let raw = ptr
            .checked_sub(ALLOCATION_PTR_BASE)
            .ok_or(EvalError::Overflow)?;
        let slot = usize::try_from(raw / ALLOCATION_PTR_STRIDE)
            .map_err(|_| EvalError::InvalidByteString)?;
        let offset = usize::try_from(raw % ALLOCATION_PTR_STRIDE)
            .map_err(|_| EvalError::InvalidByteString)?;
        let bytes = self
            .allocations
            .get(slot)
            .and_then(Option::as_ref)
            .ok_or(EvalError::InvalidByteString)?;
        if offset > bytes.len() {
            return Err(EvalError::InvalidByteString);
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
            .ok_or(EvalError::InvalidByteString)?;
        Ok(Some(&bytes[offset..]))
    }

    fn allocation_bytes_mut(&mut self, ptr: i64) -> Result<Option<&mut [u8]>, EvalError> {
        if ptr < ALLOCATION_PTR_BASE || ptr >= 0 {
            return Ok(None);
        }
        let (slot, offset) = self.decode_allocation_pointer(ptr)?;
        let bytes = self
            .allocations
            .get_mut(slot)
            .and_then(Option::as_mut)
            .ok_or(EvalError::InvalidByteString)?;
        Ok(Some(&mut bytes[offset..]))
    }

    fn pointer_bytes(&self, ptr: i64) -> Result<&[u8], EvalError> {
        if let Some(bytes) = self.allocation_bytes(ptr)? {
            return Ok(bytes);
        }
        let (base, offset) = self.decode_pointer(ptr)?;
        let bytes = match self.nodes.get(base).ok_or(EvalError::InvalidByteString)? {
            Node::Bytes(bytes) | Node::MutableBytes { bytes, .. } => bytes,
            Node::ForeignPtr {
                bytes: Some(bytes),
                offset: foreign_offset,
                ..
            } => {
                let offset = foreign_offset
                    .checked_add(offset)
                    .ok_or(EvalError::Overflow)?;
                return bytes.get(offset..).ok_or(EvalError::InvalidByteString);
            }
            _ => return Err(EvalError::InvalidByteString),
        };
        if offset > bytes.len() {
            return Err(EvalError::InvalidByteString);
        }
        Ok(&bytes[offset..])
    }

    fn write_pointer_bytes(&mut self, ptr: i64, bytes: &[u8]) -> Result<(), EvalError> {
        if let Some(dst) = self.allocation_bytes_mut(ptr)? {
            let dst = dst
                .get_mut(..bytes.len())
                .ok_or(EvalError::InvalidByteString)?;
            dst.copy_from_slice(bytes);
            return Ok(());
        }
        Err(EvalError::InvalidByteString)
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
        let bytes = bytes.get(..len).ok_or(EvalError::InvalidByteString)?;
        Ok(bytes.to_vec())
    }

    fn read_c_string(&self, ptr: i64) -> Result<Vec<u8>, EvalError> {
        let bytes = self.pointer_bytes(ptr)?;
        let len = bytes
            .iter()
            .position(|byte| *byte == 0)
            .unwrap_or(bytes.len());
        Ok(bytes[..len].to_vec())
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
            Utf8(i64),
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
                BFileKind::Utf8 { inner, unget } => {
                    if let Some(byte) = unget.take() {
                        return Ok(byte);
                    }
                    Some(SpecialBFileRead::Utf8(*inner))
                }
                BFileKind::Crlf { inner } => Some(SpecialBFileRead::Crlf(*inner)),
                BFileKind::Rle { read, .. } if *read => Some(SpecialBFileRead::Rle),
                BFileKind::Base64 { read, .. } if *read => Some(SpecialBFileRead::Base64),
                BFileKind::Buf { .. } => Some(SpecialBFileRead::Buf),
                _ => None,
            }
        };
        match special {
            Some(SpecialBFileRead::Utf8(inner)) => return self.get_utf8_bfile_byte(inner),
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

    fn get_utf8_bfile_byte(&mut self, inner: i64) -> Result<i64, EvalError> {
        let c1 = self.get_bfile_byte(inner)?;
        if c1 < 0 {
            return Ok(-1);
        }
        if (c1 & 0x80) == 0 {
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

    fn eval_io_handle(&mut self, id: NodeId) -> Result<StdHandle, EvalError> {
        let ptr = self.eval_pointer_value(id)?;
        handle_from_ptr(ptr).ok_or(EvalError::InvalidHandle)
    }

    fn write_io_handle(&self, handle: StdHandle, text: &str) -> Result<(), EvalError> {
        self.write_io_handle_bytes(handle, text.as_bytes())
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
                    stdout.flush().map_err(|_| EvalError::InvalidHandle)?;
                }
                StdHandle::Stderr => {
                    let mut stderr = std::io::stderr().lock();
                    stderr
                        .write_all(bytes)
                        .map_err(|_| EvalError::InvalidHandle)?;
                    stderr.flush().map_err(|_| EvalError::InvalidHandle)?;
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

    pub fn serialize_program(&self, root: NodeId) -> Result<Vec<u8>, EvalError> {
        let mut out = b"v8.4\n0\n".to_vec();
        self.serialize_comb_into(root, 0, &mut out)?;
        out.extend_from_slice(b"}\n");
        Ok(out)
    }

    fn serialize_comb_into(
        &self,
        id: NodeId,
        depth: usize,
        out: &mut Vec<u8>,
    ) -> Result<(), EvalError> {
        if depth > 10_000 {
            return Err(EvalError::StepLimit { limit: depth });
        }
        let id = self.resolve(id)?;
        let is_app = matches!(&self.nodes[id.0], Node::App(_, _));
        match &self.nodes[id.0] {
            Node::App(fun, arg) => {
                self.serialize_comb_into(*fun, depth + 1, out)?;
                self.serialize_comb_into(*arg, depth + 1, out)?;
                out.push(b'@');
            }
            Node::Indir(_) => return Err(EvalError::DanglingIndirection(id)),
            Node::Prim(name) => out.extend_from_slice(name.as_bytes()),
            Node::Int(n) => {
                out.push(b'#');
                push_display(out, *n);
            }
            Node::Int64(n) => {
                out.extend_from_slice(b"##");
                push_display(out, *n);
            }
            Node::Float64(n) => {
                out.push(b'&');
                out.extend_from_slice(format_float(*n).as_bytes());
            }
            Node::Float32(n) => {
                out.extend_from_slice(b"&&");
                out.extend_from_slice(format_float(f64::from(*n)).as_bytes());
            }
            Node::ThreadId(_) | Node::Weak { .. } | Node::MVar(_) => {
                return Err(EvalError::UnsupportedSerialization(id));
            }
            Node::Ptr(ptr) => serialize_ptr(*ptr, out),
            Node::RawFunPtr(ptr) => {
                out.extend_from_slice(b"toFunPtr #");
                push_display(out, *ptr);
                out.extend_from_slice(b" @");
            }
            Node::ForeignPtr {
                bytes, offset, ptr, ..
            } => {
                if let Some(mpz) = self.mpz_decimal_bytes_for_ptr(*ptr) {
                    out.push(b'%');
                    serialize_bytes_quoted(mpz, out);
                } else if let Some(bytes) = bytes {
                    if *offset == 0 {
                        out.extend_from_slice(b"bs2fp ");
                        serialize_bytes_comb(bytes, out);
                        out.extend_from_slice(b" @");
                    } else {
                        out.extend_from_slice(b"fp+ bs2fp ");
                        serialize_bytes_comb(bytes, out);
                        out.extend_from_slice(b" @ #");
                        push_display(out, *offset);
                        out.extend_from_slice(b" @");
                    }
                } else {
                    out.extend_from_slice(b"fpnew ");
                    serialize_ptr(*ptr, out);
                    out.extend_from_slice(b" @");
                }
            }
            Node::BigInt(bytes) => {
                out.push(b'%');
                serialize_bytes_quoted(bytes, out);
            }
            Node::Bytes(bytes) | Node::MutableBytes { bytes, .. } => {
                serialize_bytes_comb(bytes, out);
            }
            Node::Array(items) => {
                for item in items {
                    self.serialize_comb_into(*item, depth + 1, out)?;
                }
                out.push(b'[');
                push_display(out, items.len());
                out.push(b']');
            }
            Node::Ffi(name) => {
                out.push(b'^');
                out.extend_from_slice(name.as_bytes());
            }
            Node::JsCall { tags, body } => {
                out.push(b'~');
                out.extend_from_slice(tags.as_bytes());
                out.push(b' ');
                serialize_bytes_quoted(body, out);
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
                serialize_bytes_quoted(name, out);
            }
        }
        if !is_app {
            out.push(b' ');
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
        Ok(self.push_node(Node::Int(handle)))
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

    fn offset_foreign_ptr(&mut self, id: NodeId, by: usize) -> Result<NodeId, EvalError> {
        let (bytes, offset, ptr, finalizer) = match &self.nodes[id.0] {
            Node::ForeignPtr {
                bytes,
                offset,
                ptr,
                finalizer,
            } => (bytes.clone(), *offset, *ptr, *finalizer),
            _ => return Err(EvalError::ExpectedForeignPtr(id)),
        };
        let offset = offset.checked_add(by).ok_or(EvalError::Overflow)?;
        let ptr = ptr
            .checked_add(i64::try_from(by).map_err(|_| EvalError::Overflow)?)
            .ok_or(EvalError::Overflow)?;
        Ok(self.push_node(Node::ForeignPtr {
            bytes,
            offset,
            ptr,
            finalizer,
        }))
    }

    fn foreign_ptr_to_bytes(&mut self, id: NodeId, len: usize) -> Result<NodeId, EvalError> {
        let bytes = match &self.nodes[id.0] {
            Node::ForeignPtr {
                bytes: Some(bytes),
                offset,
                ..
            } => {
                let end = offset
                    .checked_add(len)
                    .filter(|end| *end <= bytes.len())
                    .ok_or(EvalError::InvalidByteString)?;
                bytes[*offset..end].to_vec()
            }
            Node::ForeignPtr { ptr, .. } => self.read_pointer_bytes(*ptr, len)?,
            _ => return Err(EvalError::ExpectedForeignPtr(id)),
        };
        Ok(self.push_node(Node::Bytes(bytes)))
    }

    fn foreign_ptr_value(&self, id: NodeId) -> Result<i64, EvalError> {
        match &self.nodes[id.0] {
            Node::ForeignPtr { ptr, .. } => Ok(*ptr),
            Node::Prim(name) => std_handle_ptr(name).ok_or(EvalError::ExpectedForeignPtr(id)),
            _ => Err(EvalError::ExpectedForeignPtr(id)),
        }
    }

    fn eval_js_object_handle(&mut self, id: NodeId) -> Result<u32, EvalError> {
        let foreign_ptr = self.eval_foreign_ptr_id(id)?;
        u32::try_from(self.foreign_ptr_value(foreign_ptr)?).map_err(|_| EvalError::Overflow)
    }

    fn js_object_node(&self, handle: u32) -> Node {
        Node::ForeignPtr {
            bytes: None,
            offset: 0,
            ptr: i64::from(handle),
            finalizer: None,
        }
    }

    fn set_foreign_ptr_finalizer(
        &mut self,
        id: NodeId,
        finalizer: NodeId,
    ) -> Result<(), EvalError> {
        match &mut self.nodes[id.0] {
            Node::ForeignPtr {
                finalizer: slot, ..
            } => {
                *slot = Some(finalizer);
                Ok(())
            }
            _ => Err(EvalError::ExpectedForeignPtr(id)),
        }
    }

    fn new_weak_ptr(&mut self, value: NodeId, finalizer: Option<NodeId>) -> NodeId {
        self.push_node(Node::Weak {
            value: Some(value),
            finalizer,
        })
    }

    fn eval_weak_id(&mut self, id: NodeId) -> Result<NodeId, EvalError> {
        let root = self.reduce_node_whnf(id, 10_000)?;
        let id = self.resolve(root)?;
        match self.nodes[id.0] {
            Node::Weak { .. } => Ok(id),
            _ => Err(EvalError::ExpectedWeak(root)),
        }
    }

    fn deref_weak_ptr(&mut self, id: NodeId) -> Result<NodeId, EvalError> {
        let id = self.eval_weak_id(id)?;
        let value = match self.nodes[id.0] {
            Node::Weak { value, .. } => value,
            _ => unreachable!(),
        };
        Ok(match value {
            Some(value) => self.just(value),
            None => self.nothing(),
        })
    }

    fn finalize_weak_ptr(&mut self, id: NodeId) -> Result<(), EvalError> {
        let id = self.eval_weak_id(id)?;
        let finalizer = match &mut self.nodes[id.0] {
            Node::Weak { finalizer, .. } => finalizer.take(),
            _ => unreachable!(),
        };
        if let Some(finalizer) = finalizer {
            let world = self.world();
            let action = self.app(finalizer, world);
            self.reduce_node_whnf(action, 10_000)?;
        }
        Ok(())
    }

    fn eval_mvar_id(&mut self, id: NodeId) -> Result<NodeId, EvalError> {
        let root = self.reduce_node_whnf(id, 10_000)?;
        let id = self.resolve(root)?;
        match self.nodes[id.0] {
            Node::MVar(_) => Ok(id),
            _ => Err(EvalError::ExpectedMVar(root)),
        }
    }

    fn read_mvar(&self, id: NodeId) -> Result<Option<NodeId>, EvalError> {
        match self.nodes[id.0] {
            Node::MVar(value) => Ok(value),
            _ => Err(EvalError::ExpectedMVar(id)),
        }
    }

    fn take_mvar(&mut self, id: NodeId) -> Result<Option<NodeId>, EvalError> {
        match &mut self.nodes[id.0] {
            Node::MVar(value) => Ok(value.take()),
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
        match &mut self.nodes[id.0] {
            Node::MVar(value) if value.is_none() => {
                *value = Some(new_value);
                Ok(true)
            }
            Node::MVar(_) => Ok(false),
            _ => Err(EvalError::ExpectedMVar(id)),
        }
    }

    fn int_list(&mut self, values: impl IntoIterator<Item = i64>) -> NodeId {
        let values: Vec<_> = values.into_iter().collect();
        let mut list = self.prim("K");
        for value in values.into_iter().rev() {
            let cons = self.prim("O");
            let value = self.push_node(Node::Int(value));
            let head = self.app(cons, value);
            list = self.app(head, list);
        }
        list
    }

    fn arg_ref_array(&mut self) -> NodeId {
        let mut list = self.prim("K");
        for arg in self.program_args.clone().into_iter().rev() {
            let string = self.int_list(arg.into_iter().map(i64::from));
            let cons = self.prim("O");
            let cons_string = self.app(cons, string);
            list = self.app(cons_string, list);
        }
        self.push_node(Node::Array(vec![list]))
    }

    fn reduce_node_whnf(&mut self, mut root: NodeId, limit: usize) -> Result<NodeId, EvalError> {
        let mut steps = 0;
        while steps < limit {
            let current = self.resolve(root)?;
            let Some(step) = self.step(current, limit - steps)? else {
                return Ok(current);
            };
            steps += step.reductions;
            self.reductions += step.reductions;
            if !step.in_place {
                self.nodes[current.0] = Node::Indir(Some(step.node));
            }
            root = step.node;
        }
        Err(EvalError::StepLimit { limit })
    }

    fn rnf(&mut self, noerr: bool, root: NodeId) -> Result<(), EvalError> {
        let mut seen = HashSet::new();
        self.rnf_rec(noerr, root, &mut seen)
    }

    fn rnf_rec(
        &mut self,
        noerr: bool,
        root: NodeId,
        seen: &mut HashSet<NodeId>,
    ) -> Result<(), EvalError> {
        let root = self.resolve(root)?;
        if !seen.insert(root) {
            return Ok(());
        }
        let root = match self.reduce_node_whnf(root, 10_000) {
            Ok(root) => self.resolve(root)?,
            Err(EvalError::Raised(_)) if noerr => return Ok(()),
            Err(err) => return Err(err),
        };
        if let Node::App(fun, arg) = self.nodes[root.0] {
            self.rnf_rec(noerr, fun, seen)?;
            self.rnf_rec(noerr, arg, seen)?;
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
        match &self.nodes[id.0] {
            Node::App(fun, arg) => {
                out.push('(');
                self.render_into(*fun, depth + 1, out);
                out.push(' ');
                self.render_into(*arg, depth + 1, out);
                out.push(')');
            }
            Node::Indir(_) => out.push_str("<indir>"),
            Node::Prim(name) => out.push_str(name),
            Node::Int(n) => out.push_str(&n.to_string()),
            Node::Int64(n) => {
                out.push_str(&n.to_string());
                out.push_str("i64");
            }
            Node::Float64(n) => out.push_str(&format_float(*n)),
            Node::Float32(n) => {
                out.push_str(&format_float(f64::from(*n)));
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
            Node::ForeignPtr { ptr, .. } => {
                if let Some(mpz) = self.mpz_decimal_bytes_for_ptr(*ptr) {
                    out.push('%');
                    render_bytes(mpz, out);
                } else {
                    out.push_str("ForeignPtr#");
                    out.push_str(&ptr.to_string());
                }
            }
            Node::Weak { .. } => {
                out.push_str("Weak#");
                out.push_str(&id.0.to_string());
            }
            Node::MVar(_) => {
                out.push_str("MVar#");
                out.push_str(&id.0.to_string());
            }
            Node::BigInt(bytes) => {
                out.push('%');
                render_bytes(bytes, out);
            }
            Node::Bytes(bytes) | Node::MutableBytes { bytes, .. } => render_bytes(bytes, out),
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
                out.push_str(name);
            }
            Node::JsCall { tags, body } => {
                out.push('~');
                out.push_str(tags);
                out.push(' ');
                render_bytes(body, out);
            }
            Node::JsWrap { tags } => {
                out.push('`');
                out.push_str(tags);
            }
            Node::FunPtr(name) => {
                out.push(';');
                out.push_str(name);
            }
            Node::Tick(name) => {
                out.push('!');
                render_bytes(name, out);
            }
        }
    }
}

enum IntResult {
    Int(i64),
    Bool(bool),
    Ordering(Ordering),
}

#[derive(Clone, Copy)]
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

#[derive(Clone, Copy)]
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

#[derive(Clone, Copy)]
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

#[derive(Clone, Copy)]
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

#[derive(Clone, Copy)]
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

#[derive(Clone, Copy)]
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

#[derive(Clone, Copy)]
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

#[derive(Clone, Copy)]
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

fn is_identity_alias(name: &str) -> bool {
    matches!(name, "I" | "ord" | "chr")
}

fn tuple_fields(name: &str) -> Option<usize> {
    let fields = name.strip_prefix('T')?.parse().ok()?;
    (3..=16).contains(&fields).then_some(fields)
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

fn host_js_make_wrapper(tags: &[u8], stable_ptr: i64) -> Result<u32, EvalError> {
    #[cfg(target_arch = "wasm32")]
    {
        let tags = nul_terminated(tags)?;
        let stable_ptr = u32::try_from(stable_ptr).map_err(|_| EvalError::Overflow)?;
        let result = unsafe { mhs_js_make_wrapper(tags.as_ptr(), stable_ptr) };
        host_js_check_error()?;
        Ok(result)
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = (tags, stable_ptr);
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
    fn mhs_js_call_dbl(idx: i32) -> f64;
    fn mhs_js_call_ptr(idx: i32) -> u32;
    fn mhs_js_call_obj(idx: i32) -> u32;
    fn mhs_js_call_bool(idx: i32) -> i32;
    fn mhs_js_call_str(idx: i32) -> *const std::os::raw::c_char;
    fn mhs_js_call_void(idx: i32);
    fn mhs_js_make_wrapper(tags: *const u8, stable_ptr: u32) -> u32;
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

    fn to_f64(&self) -> f64 {
        let mut out = 0.0;
        for &digit in self.digits.iter().rev() {
            out = out * f64::from(MPZ_BASE) + f64::from(digit);
        }
        if self.negative { -out } else { out }
    }
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
        | "accept" | "bind" | "connect" | "fcntl" | "mpz_add" | "mpz_and" | "mpz_fdiv_q_2exp"
        | "mpz_ior" | "mpz_mul" | "mpz_mul_2exp" | "mpz_sub" | "mpz_xor" | "socket" => 3,
        "recv" | "send" => 4,
        "mpz_tdiv_qr" => 4,
        "getsockopt" | "setsockopt" => 5,
        "strerror_r" => 3,
        _ => return None,
    })
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

fn serialize_bytes_quoted(bytes: &[u8], out: &mut Vec<u8>) {
    out.push(b'"');
    for &byte in bytes {
        match byte {
            b'"' | b'\\' => {
                out.push(b'\\');
                out.push(byte);
            }
            b'?' => out.extend_from_slice(b"\\?"),
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

fn decode_utf8_bytes(mut bytes: &[u8]) -> Result<Vec<u32>, EvalError> {
    let mut values = Vec::new();
    while !bytes.is_empty() {
        let (value, offset) = head_utf8(bytes)?;
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
    use crate::{EvalError, Node, parse_program};

    fn whnf(input: &[u8]) -> String {
        let mut program = parse_program(input).unwrap();
        let (root, _) = program.reduce_whnf(100).unwrap();
        program.render(root)
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
    fn reduces_builtin_ffi_calls() {
        let is_linux = if cfg!(target_os = "linux") { "1" } else { "0" };
        assert_eq!(whnf(b"v8.4\n0\nIO.performIO ^islinux @ }"), is_linux);
        assert_eq!(whnf(b"v8.4\n0\nIO.performIO ^GETRAW @ }"), "-1");
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
        match program.nodes()[root.0] {
            Node::Int(n) => assert!(n >= 0),
            _ => panic!("GETTIMEMICRO did not return an Int"),
        }
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
