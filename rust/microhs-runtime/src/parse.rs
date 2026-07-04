use std::collections::HashMap;
use std::fmt;

use crate::runtime::{Node, NodeId, Program, is_runtime_prim_name};

const COMB_VERSION: &[u8] = b"v8.4\n";
const PARSE_SMALL_INT_MIN: i64 = -10;
const PARSE_SMALL_INT_MAX: i64 = 255;
const PARSE_SMALL_INT_COUNT: usize = (PARSE_SMALL_INT_MAX - PARSE_SMALL_INT_MIN + 1) as usize;

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
    Parser::new(input).parse_top()
}

struct Parser<'a> {
    input: &'a [u8],
    pos: usize,
    nodes: Vec<Node>,
    stack: Vec<NodeId>,
    labels: HashMap<usize, NodeId>,
    prim_nodes: HashMap<String, NodeId>,
    small_int_nodes: [Option<NodeId>; PARSE_SMALL_INT_COUNT],
}

impl<'a> Parser<'a> {
    fn new(input: &'a [u8]) -> Self {
        let (nodes, stack) = if input.len() >= 64 * 1024 {
            let node_capacity = input.len().saturating_div(3);
            let stack_capacity = input.len().saturating_div(32);
            (
                Vec::with_capacity(node_capacity),
                Vec::with_capacity(stack_capacity),
            )
        } else {
            (Vec::new(), Vec::new())
        };
        Self {
            input,
            pos: 0,
            nodes,
            stack,
            labels: HashMap::new(),
            prim_nodes: HashMap::new(),
            small_int_nodes: [None; PARSE_SMALL_INT_COUNT],
        }
    }

    fn parse_top(mut self) -> Result<Program, ParseError> {
        if !self.input.starts_with(COMB_VERSION) {
            return Err(ParseError::Version);
        }
        self.pos = COMB_VERSION.len();
        self.gobble(b'\r');
        let _num_labels = self.parse_usize()?;
        self.expect(b'\n')?;
        self.gobble(b'\r');
        let root = self.parse_expr()?;
        for (label, id) in &self.labels {
            if matches!(self.nodes[id.index()], Node::Indir(None)) {
                return Err(ParseError::DanglingLabel(*label));
            }
        }
        Ok(Program::new(self.nodes, root, self.labels))
    }

    fn parse_expr(&mut self) -> Result<NodeId, ParseError> {
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
                        return Err(ParseError::StackJunk);
                    }
                    return Ok(root);
                }
                b'%' => {
                    self.gobble(b'"');
                    let digits = self.parse_string()?;
                    let node = self.push(Node::bigint(digits));
                    self.stack.push(node);
                }
                b'&' => {
                    let is32 = self.gobble(b'&');
                    let text = self.token_after_prefix_str()?;
                    let node = if is32 {
                        Node::Float32(text.parse().map_err(|_| ParseError::InvalidNumber)?)
                    } else {
                        Node::Float64(text.parse().map_err(|_| ParseError::InvalidNumber)?)
                    };
                    let id = self.push(node);
                    self.stack.push(id);
                }
                b'#' => {
                    let id = if self.gobble(b'#') {
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
                        return Err(ParseError::StackUnderflow);
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
                    let top = *self.stack.last().ok_or(ParseError::StackUnderflow)?;
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
                    if self.input.len().saturating_sub(self.pos) < len {
                        return Err(ParseError::Eof);
                    }
                    let bytes = self.input[self.pos..self.pos + len].to_vec();
                    self.pos += len;
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
                    let start = self.pos - 1;
                    let name = self.token_str_from(start)?;
                    let id = self.prim(name)?;
                    self.stack.push(id);
                }
            }
        }
    }

    fn next_non_space(&mut self) -> Result<u8, ParseError> {
        loop {
            let c = self.get()?;
            if !matches!(c, b' ' | b'\n' | b'\r') {
                return Ok(c);
            }
        }
    }

    fn parse_string(&mut self) -> Result<Vec<u8>, ParseError> {
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

    fn token_after_prefix_string(&mut self) -> Result<String, ParseError> {
        Ok(self.token_after_prefix_str()?.to_owned())
    }

    fn token_after_prefix_str(&mut self) -> Result<&'a str, ParseError> {
        let token = self.token_after_prefix_slice();
        std::str::from_utf8(token).map_err(|_| ParseError::InvalidUtf8)
    }

    fn token_str_from(&mut self, start: usize) -> Result<&'a str, ParseError> {
        let token = self.token_slice_from(start);
        std::str::from_utf8(token).map_err(|_| ParseError::InvalidUtf8)
    }

    fn token_after_prefix_slice(&mut self) -> &'a [u8] {
        let start = self.pos;
        self.token_slice_from(start)
    }

    fn token_slice_from(&mut self, start: usize) -> &'a [u8] {
        while let Some(c) = self.peek() {
            if matches!(c, b' ' | b'\n') {
                let end = self.pos;
                self.pos += 1;
                return &self.input[start..end];
            }
            self.pos += 1;
        }
        &self.input[start..self.pos]
    }

    fn parse_i64(&mut self) -> Result<i64, ParseError> {
        let negative = self.gobble(b'-');
        let mut saw_digit = false;
        let mut value: i64 = 0;
        while let Some(c) = self.peek() {
            if !c.is_ascii_digit() {
                break;
            }
            saw_digit = true;
            self.pos += 1;
            let digit = (c - b'0') as i64;
            value = if negative {
                value
                    .checked_mul(10)
                    .and_then(|v| v.checked_sub(digit))
                    .ok_or(ParseError::InvalidNumber)?
            } else {
                value
                    .checked_mul(10)
                    .and_then(|v| v.checked_add(digit))
                    .ok_or(ParseError::InvalidNumber)?
            };
        }
        if !saw_digit {
            return Err(ParseError::InvalidNumber);
        }
        Ok(value)
    }

    fn parse_usize(&mut self) -> Result<usize, ParseError> {
        let value = self.parse_i64()?;
        usize::try_from(value).map_err(|_| ParseError::InvalidNumber)
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

    fn define_label(&mut self, label: usize, target: NodeId) -> Result<(), ParseError> {
        if let Some(id) = self.labels.get(&label).copied() {
            match &mut self.nodes[id.index()] {
                Node::Indir(slot @ None) => {
                    *slot = Some(target);
                    Ok(())
                }
                _ => Err(ParseError::DuplicateLabel(label)),
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

    fn prim(&mut self, name: &str) -> Result<NodeId, ParseError> {
        if let Some(id) = self.prim_nodes.get(name).copied() {
            return Ok(id);
        }
        if !is_runtime_prim_name(name) {
            return Err(ParseError::UnknownPrim(name.to_owned()));
        }
        let id = self.push(Node::prim(name));
        self.prim_nodes.insert(name.to_owned(), id);
        Ok(id)
    }

    fn pop(&mut self) -> Result<NodeId, ParseError> {
        self.stack.pop().ok_or(ParseError::StackUnderflow)
    }

    fn expect(&mut self, expected: u8) -> Result<(), ParseError> {
        let got = self.get().ok();
        if got == Some(expected) {
            Ok(())
        } else {
            Err(ParseError::Expected { expected, got })
        }
    }

    fn gobble(&mut self, expected: u8) -> bool {
        if self.peek() == Some(expected) {
            self.pos += 1;
            true
        } else {
            false
        }
    }

    fn get(&mut self) -> Result<u8, ParseError> {
        let c = self.peek().ok_or(ParseError::Eof)?;
        self.pos += 1;
        Ok(c)
    }

    fn peek(&self) -> Option<u8> {
        self.input.get(self.pos).copied()
    }
}

fn parse_small_int_index(value: i64) -> Option<usize> {
    if (PARSE_SMALL_INT_MIN..=PARSE_SMALL_INT_MAX).contains(&value) {
        Some((value - PARSE_SMALL_INT_MIN) as usize)
    } else {
        None
    }
}
