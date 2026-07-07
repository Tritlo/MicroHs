//! One-pass BFILE parser for `IO.deserialize`.
use super::*;

use crate::parse::ParseError;

const COMB_VERSION: &[u8] = b"v8.4\n";
const PARSE_SMALL_INT_MIN: i64 = -10;
const PARSE_SMALL_INT_MAX: i64 = 255;
const PARSE_SMALL_INT_COUNT: usize = (PARSE_SMALL_INT_MAX - PARSE_SMALL_INT_MIN + 1) as usize;

impl Program {
    pub(in crate::runtime) fn parse_bfile_program_streaming(
        &mut self,
        ptr: i64,
    ) -> Result<Program, EvalError> {
        let mut parser = BFileParser::new(self, ptr);
        let mut result = parser.parse_top();
        if let Some(byte) = parser.ungot.take() {
            match parser.program.unget_bfile_byte(parser.ptr, i64::from(byte)) {
                Ok(()) => {}
                Err(err) if result.is_ok() => result = Err(err),
                Err(_) => {}
            }
        }
        result
    }

    #[cfg(test)]
    pub(in crate::runtime) fn parse_bfile_program_for_test(
        &mut self,
        ptr: i64,
    ) -> Result<Program, EvalError> {
        self.parse_bfile_program_streaming(ptr)
    }
}

struct BFileParser<'a> {
    program: &'a mut Program,
    ptr: i64,
    ungot: Option<u8>,
    nodes: Vec<Node>,
    stack: Vec<NodeId>,
    labels: HashMap<usize, NodeId>,
    prim_nodes: HashMap<String, NodeId>,
    small_int_nodes: [Option<NodeId>; PARSE_SMALL_INT_COUNT],
}

impl<'a> BFileParser<'a> {
    fn new(program: &'a mut Program, ptr: i64) -> Self {
        Self {
            program,
            ptr,
            ungot: None,
            nodes: Vec::new(),
            stack: Vec::new(),
            labels: HashMap::new(),
            prim_nodes: HashMap::new(),
            small_int_nodes: [None; PARSE_SMALL_INT_COUNT],
        }
    }

    fn parse_top(&mut self) -> Result<Program, EvalError> {
        for expected in COMB_VERSION {
            if self.read_optional()? != Some(*expected) {
                return Err(Self::parse_error(ParseError::Version));
            }
        }
        self.gobble(b'\r')?;
        let _num_labels = self.parse_usize()?;
        self.expect(b'\n')?;
        self.gobble(b'\r')?;
        let root = self.parse_expr()?;
        for (label, id) in &self.labels {
            if matches!(self.nodes[id.index()], Node::Indir(None)) {
                return Err(Self::parse_error(ParseError::DanglingLabel(*label)));
            }
        }
        Ok(Program::new(
            std::mem::take(&mut self.nodes),
            root,
            std::mem::take(&mut self.labels),
        ))
    }

    fn parse_expr(&mut self) -> Result<NodeId, EvalError> {
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
                        return Err(Self::parse_error(ParseError::StackJunk));
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
                    let text = self.token_after_prefix_string()?;
                    let node = if is32 {
                        Node::Float32(
                            text.parse()
                                .map_err(|_| Self::parse_error(ParseError::InvalidNumber))?,
                        )
                    } else {
                        Node::Float64(
                            text.parse()
                                .map_err(|_| Self::parse_error(ParseError::InvalidNumber))?,
                        )
                    };
                    let id = self.push(node);
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
                        return Err(Self::parse_error(ParseError::StackUnderflow));
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
                        .ok_or_else(|| Self::parse_error(ParseError::StackUnderflow))?;
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
                    let mut bytes = Vec::with_capacity(len);
                    for _ in 0..len {
                        bytes.push(self.get()?);
                    }
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
                    let id = self.push(Node::JsCall(Box::new(JsCallNode { tags, body })));
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
                    let name = self.token_string_from_first(c)?;
                    let id = self.prim(&name)?;
                    self.stack.push(id);
                }
            }
        }
    }

    fn next_non_space(&mut self) -> Result<u8, EvalError> {
        loop {
            let c = self.get()?;
            if !matches!(c, b' ' | b'\n' | b'\r') {
                return Ok(c);
            }
        }
    }

    fn parse_string(&mut self) -> Result<Vec<u8>, EvalError> {
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

    fn token_after_prefix_string(&mut self) -> Result<String, EvalError> {
        self.token_string(Vec::new())
    }

    fn token_string_from_first(&mut self, first: u8) -> Result<String, EvalError> {
        self.token_string(vec![first])
    }

    fn token_string(&mut self, mut token: Vec<u8>) -> Result<String, EvalError> {
        loop {
            let Some(c) = self.peek()? else {
                return String::from_utf8(token)
                    .map_err(|_| Self::parse_error(ParseError::InvalidUtf8));
            };
            if matches!(c, b' ' | b'\n' | b'\r') {
                let _ = self.get()?;
                return String::from_utf8(token)
                    .map_err(|_| Self::parse_error(ParseError::InvalidUtf8));
            }
            token.push(self.get()?);
        }
    }

    fn parse_i64(&mut self) -> Result<i64, EvalError> {
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
                    .ok_or_else(|| Self::parse_error(ParseError::InvalidNumber))?
            } else {
                value
                    .checked_mul(10)
                    .and_then(|v| v.checked_add(digit))
                    .ok_or_else(|| Self::parse_error(ParseError::InvalidNumber))?
            };
        }
        if !saw_digit {
            return Err(Self::parse_error(ParseError::InvalidNumber));
        }
        Ok(value)
    }

    fn parse_usize(&mut self) -> Result<usize, EvalError> {
        let value = self.parse_i64()?;
        usize::try_from(value).map_err(|_| Self::parse_error(ParseError::InvalidNumber))
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

    fn define_label(&mut self, label: usize, target: NodeId) -> Result<(), EvalError> {
        if let Some(id) = self.labels.get(&label).copied() {
            match &mut self.nodes[id.index()] {
                Node::Indir(slot @ None) => {
                    *slot = Some(target);
                    Ok(())
                }
                _ => Err(Self::parse_error(ParseError::DuplicateLabel(label))),
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

    fn prim(&mut self, name: &str) -> Result<NodeId, EvalError> {
        if let Some(id) = self.prim_nodes.get(name).copied() {
            return Ok(id);
        }
        if !is_runtime_prim_name(name) {
            return Err(Self::parse_error(ParseError::UnknownPrim(name.to_owned())));
        }
        let id = self.push(Node::prim(name));
        self.prim_nodes.insert(name.to_owned(), id);
        Ok(id)
    }

    fn pop(&mut self) -> Result<NodeId, EvalError> {
        self.stack
            .pop()
            .ok_or_else(|| Self::parse_error(ParseError::StackUnderflow))
    }

    fn expect(&mut self, expected: u8) -> Result<(), EvalError> {
        let got = self.read_optional()?;
        if got == Some(expected) {
            Ok(())
        } else {
            Err(Self::parse_error(ParseError::Expected { expected, got }))
        }
    }

    fn gobble(&mut self, expected: u8) -> Result<bool, EvalError> {
        if self.peek()? == Some(expected) {
            let _ = self.get()?;
            Ok(true)
        } else {
            Ok(false)
        }
    }

    fn get(&mut self) -> Result<u8, EvalError> {
        self.read_optional()?
            .ok_or_else(|| Self::parse_error(ParseError::Eof))
    }

    fn peek(&mut self) -> Result<Option<u8>, EvalError> {
        if self.ungot.is_none() {
            self.ungot = self.read_raw()?;
        }
        Ok(self.ungot)
    }

    fn read_optional(&mut self) -> Result<Option<u8>, EvalError> {
        if let Some(byte) = self.ungot.take() {
            Ok(Some(byte))
        } else {
            self.read_raw()
        }
    }

    fn read_raw(&mut self) -> Result<Option<u8>, EvalError> {
        let byte = self.program.get_bfile_byte(self.ptr)?;
        if byte < 0 {
            return Ok(None);
        }
        u8::try_from(byte)
            .map(Some)
            .map_err(|_| EvalError::InvalidByteString)
    }

    fn parse_error(error: ParseError) -> EvalError {
        Program::deserialize_parse_error(error)
    }
}

fn parse_small_int_index(value: i64) -> Option<usize> {
    if (PARSE_SMALL_INT_MIN..=PARSE_SMALL_INT_MAX).contains(&value) {
        Some((value - PARSE_SMALL_INT_MIN) as usize)
    } else {
        None
    }
}
