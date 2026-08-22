use std::collections::HashMap;
use std::fmt;

use crate::runtime::{Node, NodeId, Program, is_runtime_prim_name};

const COMB_VERSION: &[u8] = b"v8.4\n";
const PARSE_SMALL_INT_MIN: i64 = -10;
const PARSE_SMALL_INT_MAX: i64 = 255;
const PARSE_SMALL_INT_COUNT: usize = (PARSE_SMALL_INT_MAX - PARSE_SMALL_INT_MIN + 1) as usize;
const JS_EXPORTS_TRAILER: &[u8] = b"##### JS_EXPORTS";

#[derive(Debug)]
pub enum ParseError {
    Version,
    Eof,
    Expected { expected: u8, got: Option<u8> },
    InvalidNumber,
    InvalidUtf8,
    StackUnderflow,
    StackJunk,
    DuplicateLabel(usize),
    DanglingLabel(usize),
    UnknownPrim(String),
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Version => write!(f, "unsupported combinator file version"),
            Self::Eof => write!(f, "unexpected end of file"),
            Self::Expected { expected, got } => match got {
                Some(got) => write!(
                    f,
                    "expected {:?}, got {:?}",
                    *expected as char, *got as char
                ),
                None => write!(f, "expected {:?}, got EOF", *expected as char),
            },
            Self::InvalidNumber => write!(f, "invalid number"),
            Self::InvalidUtf8 => write!(f, "invalid utf-8 in token"),
            Self::StackUnderflow => write!(f, "combinator parse stack underflow"),
            Self::StackJunk => write!(f, "combinator parse stack left extra values"),
            Self::DuplicateLabel(label) => write!(f, "duplicate shared label {label}"),
            Self::DanglingLabel(label) => write!(f, "dangling shared label {label}"),
            Self::UnknownPrim(name) => write!(f, "unknown primitive {name}"),
        }
    }
}

impl std::error::Error for ParseError {}

pub fn parse_program(input: &[u8]) -> Result<Program, ParseError> {
    Parser::new(SliceSource::new(input)).parse_program_file()
}

pub(crate) trait ByteSource {
    type Error;

    fn parse_error(error: ParseError) -> Self::Error;

    fn size_hint(&self) -> Option<usize> {
        None
    }

    fn next_byte(&mut self) -> Result<Option<u8>, Self::Error>;

    fn peek_byte(&mut self) -> Result<Option<u8>, Self::Error> {
        let Some(byte) = self.next_byte()? else {
            return Ok(None);
        };
        self.unget_byte(byte)?;
        Ok(Some(byte))
    }

    fn unget_byte(&mut self, byte: u8) -> Result<(), Self::Error>;

    fn with_token<R, F>(
        &mut self,
        scratch: &mut Vec<u8>,
        first: Option<u8>,
        f: F,
    ) -> Result<R, Self::Error>
    where
        F: FnOnce(&[u8]) -> Result<R, ParseError>,
    {
        scratch.clear();
        if let Some(first) = first {
            scratch.push(first);
        }
        loop {
            let Some(byte) = self.peek_byte()? else {
                break;
            };
            if is_token_terminator(byte) {
                let _ = self.next_byte()?;
                break;
            }
            scratch.push(self.get_byte()?);
        }
        f(scratch).map_err(Self::parse_error)
    }

    fn with_bytes<R, F>(
        &mut self,
        scratch: &mut Vec<u8>,
        len: usize,
        f: F,
    ) -> Result<R, Self::Error>
    where
        F: FnOnce(&[u8]) -> R,
    {
        scratch.clear();
        scratch.reserve(len);
        for _ in 0..len {
            scratch.push(self.get_byte()?);
        }
        Ok(f(scratch))
    }

    fn get_byte(&mut self) -> Result<u8, Self::Error> {
        self.next_byte()?
            .ok_or_else(|| Self::parse_error(ParseError::Eof))
    }
}

pub(crate) trait Rewindable {
    type Mark;

    fn mark(&self) -> Self::Mark;

    fn rewind(&mut self, mark: Self::Mark);
}

pub(crate) struct SliceSource<'a> {
    input: &'a [u8],
    pos: usize,
}

impl<'a> SliceSource<'a> {
    fn new(input: &'a [u8]) -> Self {
        Self { input, pos: 0 }
    }
}

impl ByteSource for SliceSource<'_> {
    type Error = ParseError;

    fn parse_error(error: ParseError) -> Self::Error {
        error
    }

    fn size_hint(&self) -> Option<usize> {
        Some(self.input.len())
    }

    fn next_byte(&mut self) -> Result<Option<u8>, Self::Error> {
        let byte = self.input.get(self.pos).copied();
        if byte.is_some() {
            self.pos += 1;
        }
        Ok(byte)
    }

    fn peek_byte(&mut self) -> Result<Option<u8>, Self::Error> {
        Ok(self.input.get(self.pos).copied())
    }

    fn unget_byte(&mut self, byte: u8) -> Result<(), Self::Error> {
        if self.pos == 0 || self.input[self.pos - 1] != byte {
            return Err(ParseError::Expected {
                expected: byte,
                got: None,
            });
        }
        self.pos -= 1;
        Ok(())
    }

    fn with_token<R, F>(
        &mut self,
        _scratch: &mut Vec<u8>,
        first: Option<u8>,
        f: F,
    ) -> Result<R, Self::Error>
    where
        F: FnOnce(&[u8]) -> Result<R, ParseError>,
    {
        let start = self.pos - usize::from(first.is_some());
        loop {
            let Some(byte) = self.input.get(self.pos).copied() else {
                return f(&self.input[start..self.pos]);
            };
            if is_token_terminator(byte) {
                let end = self.pos;
                self.pos += 1;
                return f(&self.input[start..end]);
            }
            self.pos += 1;
        }
    }

    fn with_bytes<R, F>(
        &mut self,
        _scratch: &mut Vec<u8>,
        len: usize,
        f: F,
    ) -> Result<R, Self::Error>
    where
        F: FnOnce(&[u8]) -> R,
    {
        if self.input.len().saturating_sub(self.pos) < len {
            return Err(ParseError::Eof);
        }
        let start = self.pos;
        self.pos += len;
        Ok(f(&self.input[start..self.pos]))
    }
}

impl Rewindable for SliceSource<'_> {
    type Mark = usize;

    fn mark(&self) -> Self::Mark {
        self.pos
    }

    fn rewind(&mut self, mark: Self::Mark) {
        self.pos = mark;
    }
}

pub(crate) struct Parser<S: ByteSource> {
    source: S,
    nodes: Vec<Node>,
    stack: Vec<NodeId>,
    labels: HashMap<usize, NodeId>,
    prim_nodes: HashMap<String, NodeId>,
    small_int_nodes: [Option<NodeId>; PARSE_SMALL_INT_COUNT],
    scratch: Vec<u8>,
}

impl<S: ByteSource> Parser<S> {
    pub(crate) fn new(source: S) -> Self {
        let (nodes, stack) = if source.size_hint().is_some_and(|len| len >= 64 * 1024) {
            let len = source.size_hint().unwrap_or_default();
            let node_capacity = len.saturating_div(3);
            let stack_capacity = len.saturating_div(32);
            (
                Vec::with_capacity(node_capacity),
                Vec::with_capacity(stack_capacity),
            )
        } else {
            (Vec::new(), Vec::new())
        };
        Self {
            source,
            nodes,
            stack,
            labels: HashMap::new(),
            prim_nodes: HashMap::new(),
            small_int_nodes: [None; PARSE_SMALL_INT_COUNT],
            scratch: Vec::new(),
        }
    }

    pub(crate) fn parse_graph_only(mut self) -> Result<Program, S::Error> {
        self.parse_header()?;
        let root = self.parse_expr()?;
        self.finish_program(root)
    }

    fn parse_header(&mut self) -> Result<(), S::Error> {
        for expected in COMB_VERSION {
            if self.next_optional()? != Some(*expected) {
                return Err(S::parse_error(ParseError::Version));
            }
        }
        self.gobble(b'\r')?;
        let _num_labels = self.parse_usize()?;
        self.expect(b'\n')?;
        self.gobble(b'\r')?;
        Ok(())
    }

    fn finish_program(self, root: NodeId) -> Result<Program, S::Error> {
        for (label, id) in &self.labels {
            if matches!(self.nodes[id.index()], Node::Indir(None)) {
                return Err(S::parse_error(ParseError::DanglingLabel(*label)));
            }
        }
        Ok(Program::new(self.nodes, root, self.labels))
    }

    fn parse_expr(&mut self) -> Result<NodeId, S::Error> {
        loop {
            let c = self.next_non_space()?;
            match c {
                b'@' => {
                    let x = self.pop()?;
                    let y = self.pop()?;
                    let app = self.push(Node::App(y, x));
                    self.stack.push(app);
                }
                b'}' => {
                    let root = self.pop()?;
                    if !self.stack.is_empty() {
                        return Err(S::parse_error(ParseError::StackJunk));
                    }
                    return Ok(root);
                }
                b'%' => {
                    self.gobble(b'"')?;
                    let digits = self.parse_string()?;
                    let node = self.push(Node::bigint(digits));
                    self.stack.push(node);
                }
                b'&' => {
                    let is32 = self.gobble(b'&')?;
                    let id = if is32 {
                        let value = self.token_after_prefix_parse::<f32>()?;
                        self.push(Node::Float32(value))
                    } else {
                        let value = self.token_after_prefix_parse::<f64>()?;
                        self.push(Node::Float64(value))
                    };
                    self.stack.push(id);
                }
                b'#' => {
                    let id = if self.gobble(b'#')? {
                        let value = self.parse_i64()?;
                        self.push(Node::Int64(value))
                    } else {
                        let value = self.parse_i64()?;
                        self.push_int(value)
                    };
                    self.stack.push(id);
                }
                b'[' => {
                    let size = self.parse_usize()?;
                    self.expect(b']')?;
                    if self.stack.len() < size {
                        return Err(S::parse_error(ParseError::StackUnderflow));
                    }
                    let start = self.stack.len() - size;
                    let items = self.stack.drain(start..).collect();
                    let id = self.push(Node::array(items));
                    self.stack.push(id);
                }
                b'_' => {
                    let label = self.parse_usize()?;
                    let id = self.ref_label(label);
                    self.stack.push(id);
                }
                b':' => {
                    let label = self.parse_usize()?;
                    self.expect(b' ')?;
                    let top = *self
                        .stack
                        .last()
                        .ok_or_else(|| S::parse_error(ParseError::StackUnderflow))?;
                    self.define_label(label, top)?;
                }
                b'"' => {
                    let bytes = self.parse_string()?;
                    let id = self.push(Node::bytes(bytes));
                    self.stack.push(id);
                }
                b'$' => {
                    let len = self.parse_usize()?;
                    self.expect(b' ')?;
                    let bytes = self.raw_bytes(len)?;
                    let id = self.push(Node::bytes(bytes));
                    self.stack.push(id);
                }
                b'!' => {
                    self.expect(b'"')?;
                    let name = self.parse_string()?;
                    let id = self.push(Node::tick(name));
                    self.stack.push(id);
                }
                b'^' => {
                    let name = self.token_after_prefix_string()?;
                    let id = self.push(Node::ffi(name));
                    self.stack.push(id);
                }
                b'~' => {
                    let tags = self.token_after_prefix_string()?;
                    self.expect(b'"')?;
                    let body = self.parse_string()?;
                    let id = self.push(Node::JsCall(Box::new(crate::runtime::JsCallNode {
                        tags,
                        body,
                    })));
                    self.stack.push(id);
                }
                b'`' => {
                    let tags = self.token_after_prefix_string()?;
                    let id = self.push(Node::js_wrap(tags));
                    self.stack.push(id);
                }
                b';' => {
                    let name = self.token_after_prefix_string()?;
                    let id = self.push(Node::fun_ptr(name));
                    self.stack.push(id);
                }
                _ => {
                    let id = self.prim_from_first(c)?;
                    self.stack.push(id);
                }
            }
        }
    }

    fn next_non_space(&mut self) -> Result<u8, S::Error> {
        loop {
            let c = self.get()?;
            if !matches!(c, b' ' | b'\n' | b'\r') {
                return Ok(c);
            }
        }
    }

    fn parse_string(&mut self) -> Result<Vec<u8>, S::Error> {
        let mut bytes = Vec::new();
        loop {
            let mut c = self.get()?;
            if c == b'"' {
                return Ok(bytes);
            }
            match c {
                b'\\' => {
                    c = self.get()?;
                    c = match c {
                        b'?' => 0x7f,
                        b'_' => 0xff,
                        other => other,
                    };
                }
                b'^' => {
                    c = self.get()?;
                    c = if c < 0x40 {
                        c & 0x1f
                    } else {
                        (c & 0x1f) | 0x80
                    };
                }
                b'|' => {
                    c = self.get()? | 0x80;
                }
                _ => {}
            }
            bytes.push(c);
        }
    }

    fn token_after_prefix_string(&mut self) -> Result<String, S::Error> {
        let source = &mut self.source;
        let scratch = &mut self.scratch;
        source.with_token(scratch, None, token_to_string)
    }

    fn token_after_prefix_parse<T>(&mut self) -> Result<T, S::Error>
    where
        T: std::str::FromStr,
    {
        let source = &mut self.source;
        let scratch = &mut self.scratch;
        source.with_token(scratch, None, |token| {
            let text = std::str::from_utf8(token).map_err(|_| ParseError::InvalidUtf8)?;
            text.parse().map_err(|_| ParseError::InvalidNumber)
        })
    }

    fn prim_from_first(&mut self, first: u8) -> Result<NodeId, S::Error> {
        enum PrimLookup {
            Cached(NodeId),
            New(String),
        }

        let source = &mut self.source;
        let scratch = &mut self.scratch;
        let prim_nodes = &self.prim_nodes;
        let lookup = source.with_token(scratch, Some(first), |token| {
            let name = std::str::from_utf8(token).map_err(|_| ParseError::InvalidUtf8)?;
            if let Some(id) = prim_nodes.get(name).copied() {
                return Ok(PrimLookup::Cached(id));
            }
            if !is_runtime_prim_name(name) {
                return Err(ParseError::UnknownPrim(name.to_owned()));
            }
            Ok(PrimLookup::New(name.to_owned()))
        })?;

        match lookup {
            PrimLookup::Cached(id) => Ok(id),
            PrimLookup::New(name) => {
                let id = self.push(Node::prim(&name));
                self.prim_nodes.insert(name, id);
                Ok(id)
            }
        }
    }

    fn raw_bytes(&mut self, len: usize) -> Result<Vec<u8>, S::Error> {
        let source = &mut self.source;
        let scratch = &mut self.scratch;
        source.with_bytes(scratch, len, |bytes| bytes.to_vec())
    }

    fn parse_i64(&mut self) -> Result<i64, S::Error> {
        let negative = self.gobble(b'-')?;
        let mut saw_digit = false;
        let mut value: i64 = 0;
        while let Some(c) = self.peek()? {
            if !c.is_ascii_digit() {
                break;
            }
            saw_digit = true;
            let _ = self.get()?;
            let digit = (c - b'0') as i64;
            value = if negative {
                value
                    .checked_mul(10)
                    .and_then(|v| v.checked_sub(digit))
                    .ok_or_else(|| S::parse_error(ParseError::InvalidNumber))?
            } else {
                value
                    .checked_mul(10)
                    .and_then(|v| v.checked_add(digit))
                    .ok_or_else(|| S::parse_error(ParseError::InvalidNumber))?
            };
        }
        if !saw_digit {
            return Err(S::parse_error(ParseError::InvalidNumber));
        }
        Ok(value)
    }

    fn parse_usize(&mut self) -> Result<usize, S::Error> {
        let value = self.parse_i64()?;
        usize::try_from(value).map_err(|_| S::parse_error(ParseError::InvalidNumber))
    }

    fn ref_label(&mut self, label: usize) -> NodeId {
        if let Some(id) = self.labels.get(&label) {
            *id
        } else {
            let id = self.push(Node::Indir(None));
            self.labels.insert(label, id);
            id
        }
    }

    fn define_label(&mut self, label: usize, target: NodeId) -> Result<(), S::Error> {
        if let Some(id) = self.labels.get(&label).copied() {
            match &mut self.nodes[id.index()] {
                Node::Indir(slot @ None) => {
                    *slot = Some(target);
                    Ok(())
                }
                _ => Err(S::parse_error(ParseError::DuplicateLabel(label))),
            }
        } else {
            self.labels.insert(label, target);
            Ok(())
        }
    }

    fn push(&mut self, node: Node) -> NodeId {
        let id = NodeId::from_index(self.nodes.len());
        self.nodes.push(node);
        id
    }

    fn push_int(&mut self, value: i64) -> NodeId {
        let Some(index) = parse_small_int_index(value) else {
            return self.push(Node::Int(value));
        };
        if let Some(id) = self.small_int_nodes[index] {
            return id;
        }
        let id = self.push(Node::Int(value));
        self.small_int_nodes[index] = Some(id);
        id
    }

    fn pop(&mut self) -> Result<NodeId, S::Error> {
        self.stack
            .pop()
            .ok_or_else(|| S::parse_error(ParseError::StackUnderflow))
    }

    fn expect(&mut self, expected: u8) -> Result<(), S::Error> {
        let got = self.next_optional()?;
        if got == Some(expected) {
            Ok(())
        } else {
            Err(S::parse_error(ParseError::Expected { expected, got }))
        }
    }

    fn gobble(&mut self, expected: u8) -> Result<bool, S::Error> {
        if self.peek()? == Some(expected) {
            let _ = self.get()?;
            Ok(true)
        } else {
            Ok(false)
        }
    }

    fn get(&mut self) -> Result<u8, S::Error> {
        self.source.get_byte()
    }

    fn next_optional(&mut self) -> Result<Option<u8>, S::Error> {
        self.source.next_byte()
    }

    fn peek(&mut self) -> Result<Option<u8>, S::Error> {
        self.source.peek_byte()
    }
}

impl<S> Parser<S>
where
    S: ByteSource + Rewindable,
{
    fn parse_program_file(mut self) -> Result<Program, S::Error> {
        self.parse_header()?;
        let root = self.parse_expr()?;
        self.probe_js_exports_trailer()?;
        let mut program = self.finish_program(root)?;
        program.collect_garbage_after_parse();
        Ok(program)
    }

    fn probe_js_exports_trailer(&mut self) -> Result<(), S::Error> {
        let mark = self.source.mark();
        for expected in JS_EXPORTS_TRAILER {
            if self.next_optional()? != Some(*expected) {
                self.source.rewind(mark);
                return Ok(());
            }
        }
        Ok(())
    }
}

fn token_to_string(token: &[u8]) -> Result<String, ParseError> {
    String::from_utf8(token.to_vec()).map_err(|_| ParseError::InvalidUtf8)
}

fn is_token_terminator(byte: u8) -> bool {
    matches!(byte, b' ' | b'\n' | b'\r')
}

fn parse_small_int_index(value: i64) -> Option<usize> {
    if (PARSE_SMALL_INT_MIN..=PARSE_SMALL_INT_MAX).contains(&value) {
        Some((value - PARSE_SMALL_INT_MIN) as usize)
    } else {
        None
    }
}
