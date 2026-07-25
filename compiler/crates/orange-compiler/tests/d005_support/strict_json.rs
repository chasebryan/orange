use std::collections::{BTreeMap, BTreeSet};

use super::domain::BUDGETS;

const MAX_IJSON_INTEGER: i64 = 9_007_199_254_740_991;

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum JsonValue {
    Null,
    Bool(bool),
    Integer(i64),
    String(String),
    Array(Vec<JsonValue>),
    Object(BTreeMap<String, JsonValue>),
}

impl JsonValue {
    pub(crate) fn as_object(&self) -> Option<&BTreeMap<String, JsonValue>> {
        match self {
            Self::Object(value) => Some(value),
            _ => None,
        }
    }

    pub(crate) fn as_array(&self) -> Option<&[JsonValue]> {
        match self {
            Self::Array(value) => Some(value),
            _ => None,
        }
    }

    pub(crate) fn as_str(&self) -> Option<&str> {
        match self {
            Self::String(value) => Some(value),
            _ => None,
        }
    }

    pub(crate) fn as_integer(&self) -> Option<i64> {
        match self {
            Self::Integer(value) => Some(*value),
            _ => None,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum JsonErrorKind {
    InputTooLarge,
    InvalidUtf8,
    UnexpectedEnd,
    UnexpectedToken,
    TrailingData,
    DuplicateKey,
    FloatingPoint,
    InvalidNumber,
    IntegerRange,
    InvalidEscape,
    InvalidUnicode,
    DepthLimit,
    NodeLimit,
    StringLimit,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct JsonError {
    pub(crate) kind: JsonErrorKind,
    pub(crate) offset: usize,
}

impl JsonError {
    const fn new(kind: JsonErrorKind, offset: usize) -> Self {
        Self { kind, offset }
    }
}

pub(crate) fn parse(input: &[u8]) -> Result<JsonValue, JsonError> {
    parse_with_max_input(input, BUDGETS.max_packet_bytes)
}

pub(crate) fn parse_with_max_input(
    input: &[u8],
    max_input_bytes: usize,
) -> Result<JsonValue, JsonError> {
    if input.len() > max_input_bytes {
        return Err(JsonError::new(JsonErrorKind::InputTooLarge, 0));
    }
    let source = std::str::from_utf8(input)
        .map_err(|error| JsonError::new(JsonErrorKind::InvalidUtf8, error.valid_up_to()))?;
    let mut parser = Parser {
        source,
        position: 0,
        nodes: 0,
    };
    parser.skip_whitespace();
    let value = parser.parse_value(0)?;
    parser.skip_whitespace();
    if parser.position != source.len() {
        return Err(JsonError::new(JsonErrorKind::TrailingData, parser.position));
    }
    Ok(value)
}

struct Parser<'a> {
    source: &'a str,
    position: usize,
    nodes: usize,
}

impl Parser<'_> {
    fn parse_value(&mut self, depth: usize) -> Result<JsonValue, JsonError> {
        self.nodes = self
            .nodes
            .checked_add(1)
            .ok_or_else(|| JsonError::new(JsonErrorKind::NodeLimit, self.position))?;
        if self.nodes > BUDGETS.max_json_nodes {
            return Err(JsonError::new(JsonErrorKind::NodeLimit, self.position));
        }
        match self.peek() {
            Some(b'n') => {
                self.consume_literal(b"null")?;
                Ok(JsonValue::Null)
            }
            Some(b't') => {
                self.consume_literal(b"true")?;
                Ok(JsonValue::Bool(true))
            }
            Some(b'f') => {
                self.consume_literal(b"false")?;
                Ok(JsonValue::Bool(false))
            }
            Some(b'"') => self.parse_string().map(JsonValue::String),
            Some(b'[') => self.parse_array(depth),
            Some(b'{') => self.parse_object(depth),
            Some(b'-' | b'0'..=b'9') => self.parse_integer().map(JsonValue::Integer),
            Some(_) => Err(JsonError::new(
                JsonErrorKind::UnexpectedToken,
                self.position,
            )),
            None => Err(JsonError::new(JsonErrorKind::UnexpectedEnd, self.position)),
        }
    }

    fn parse_array(&mut self, depth: usize) -> Result<JsonValue, JsonError> {
        self.require_container_depth(depth)?;
        self.position += 1;
        self.skip_whitespace();
        let mut values = Vec::new();
        if self.take(b']') {
            return Ok(JsonValue::Array(values));
        }
        loop {
            values.push(self.parse_value(depth + 1)?);
            self.skip_whitespace();
            if self.take(b']') {
                break;
            }
            if !self.take(b',') {
                return Err(JsonError::new(
                    JsonErrorKind::UnexpectedToken,
                    self.position,
                ));
            }
            self.skip_whitespace();
        }
        Ok(JsonValue::Array(values))
    }

    fn parse_object(&mut self, depth: usize) -> Result<JsonValue, JsonError> {
        self.require_container_depth(depth)?;
        self.position += 1;
        self.skip_whitespace();
        let mut values = BTreeMap::new();
        let mut keys = BTreeSet::new();
        if self.take(b'}') {
            return Ok(JsonValue::Object(values));
        }
        loop {
            if self.peek() != Some(b'"') {
                return Err(JsonError::new(
                    JsonErrorKind::UnexpectedToken,
                    self.position,
                ));
            }
            let key_offset = self.position;
            let key = self.parse_string()?;
            if !keys.insert(key.clone()) {
                return Err(JsonError::new(JsonErrorKind::DuplicateKey, key_offset));
            }
            self.skip_whitespace();
            if !self.take(b':') {
                return Err(JsonError::new(
                    JsonErrorKind::UnexpectedToken,
                    self.position,
                ));
            }
            self.skip_whitespace();
            let value = self.parse_value(depth + 1)?;
            values.insert(key, value);
            self.skip_whitespace();
            if self.take(b'}') {
                break;
            }
            if !self.take(b',') {
                return Err(JsonError::new(
                    JsonErrorKind::UnexpectedToken,
                    self.position,
                ));
            }
            self.skip_whitespace();
        }
        Ok(JsonValue::Object(values))
    }

    fn parse_string(&mut self) -> Result<String, JsonError> {
        let start = self.position;
        if !self.take(b'"') {
            return Err(JsonError::new(
                JsonErrorKind::UnexpectedToken,
                self.position,
            ));
        }
        let mut value = String::new();
        loop {
            let byte = self
                .peek()
                .ok_or_else(|| JsonError::new(JsonErrorKind::UnexpectedEnd, self.position))?;
            match byte {
                b'"' => {
                    self.position += 1;
                    return Ok(value);
                }
                b'\\' => {
                    self.position += 1;
                    self.parse_escape(&mut value)?;
                }
                0x00..=0x1f => {
                    return Err(JsonError::new(
                        JsonErrorKind::UnexpectedToken,
                        self.position,
                    ));
                }
                0x20..=0x7f => {
                    value.push(char::from(byte));
                    self.position += 1;
                }
                _ => {
                    let character =
                        self.source[self.position..].chars().next().ok_or_else(|| {
                            JsonError::new(JsonErrorKind::InvalidUnicode, self.position)
                        })?;
                    value.push(character);
                    self.position += character.len_utf8();
                }
            }
            if value.len() > BUDGETS.max_string_bytes {
                return Err(JsonError::new(JsonErrorKind::StringLimit, start));
            }
        }
    }

    fn parse_escape(&mut self, output: &mut String) -> Result<(), JsonError> {
        let escape_offset = self.position;
        let escaped = self
            .peek()
            .ok_or_else(|| JsonError::new(JsonErrorKind::UnexpectedEnd, self.position))?;
        self.position += 1;
        match escaped {
            b'"' => output.push('"'),
            b'\\' => output.push('\\'),
            b'/' => output.push('/'),
            b'b' => output.push('\u{0008}'),
            b'f' => output.push('\u{000c}'),
            b'n' => output.push('\n'),
            b'r' => output.push('\r'),
            b't' => output.push('\t'),
            b'u' => {
                let first = self.parse_hex_quad()?;
                let scalar = if (0xd800..=0xdbff).contains(&first) {
                    if !self.take(b'\\') || !self.take(b'u') {
                        return Err(JsonError::new(JsonErrorKind::InvalidUnicode, escape_offset));
                    }
                    let second = self.parse_hex_quad()?;
                    if !(0xdc00..=0xdfff).contains(&second) {
                        return Err(JsonError::new(JsonErrorKind::InvalidUnicode, escape_offset));
                    }
                    0x1_0000 + ((u32::from(first) - 0xd800) << 10) + (u32::from(second) - 0xdc00)
                } else if (0xdc00..=0xdfff).contains(&first) {
                    return Err(JsonError::new(JsonErrorKind::InvalidUnicode, escape_offset));
                } else {
                    u32::from(first)
                };
                let character = char::from_u32(scalar)
                    .ok_or_else(|| JsonError::new(JsonErrorKind::InvalidUnicode, escape_offset))?;
                output.push(character);
            }
            _ => return Err(JsonError::new(JsonErrorKind::InvalidEscape, escape_offset)),
        }
        Ok(())
    }

    fn parse_hex_quad(&mut self) -> Result<u16, JsonError> {
        let start = self.position;
        let end = start
            .checked_add(4)
            .ok_or_else(|| JsonError::new(JsonErrorKind::UnexpectedEnd, start))?;
        let bytes = self.source.as_bytes();
        if end > bytes.len() {
            return Err(JsonError::new(JsonErrorKind::UnexpectedEnd, start));
        }
        let mut value = 0_u16;
        for byte in &bytes[start..end] {
            let digit = match byte {
                b'0'..=b'9' => u16::from(byte - b'0'),
                b'a'..=b'f' => u16::from(byte - b'a') + 10,
                b'A'..=b'F' => u16::from(byte - b'A') + 10,
                _ => return Err(JsonError::new(JsonErrorKind::InvalidUnicode, self.position)),
            };
            value = value * 16 + digit;
        }
        self.position = end;
        Ok(value)
    }

    fn parse_integer(&mut self) -> Result<i64, JsonError> {
        let start = self.position;
        self.take(b'-');
        match self.peek() {
            Some(b'0') => {
                self.position += 1;
                if matches!(self.peek(), Some(b'0'..=b'9')) {
                    return Err(JsonError::new(JsonErrorKind::InvalidNumber, start));
                }
            }
            Some(b'1'..=b'9') => {
                self.position += 1;
                while matches!(self.peek(), Some(b'0'..=b'9')) {
                    self.position += 1;
                }
            }
            _ => return Err(JsonError::new(JsonErrorKind::InvalidNumber, start)),
        }
        if matches!(self.peek(), Some(b'.' | b'e' | b'E')) {
            return Err(JsonError::new(JsonErrorKind::FloatingPoint, self.position));
        }
        let token = &self.source[start..self.position];
        let value = token
            .parse::<i64>()
            .map_err(|_| JsonError::new(JsonErrorKind::IntegerRange, start))?;
        if !(-MAX_IJSON_INTEGER..=MAX_IJSON_INTEGER).contains(&value) {
            return Err(JsonError::new(JsonErrorKind::IntegerRange, start));
        }
        Ok(value)
    }

    fn consume_literal(&mut self, literal: &[u8]) -> Result<(), JsonError> {
        let end = self
            .position
            .checked_add(literal.len())
            .ok_or_else(|| JsonError::new(JsonErrorKind::UnexpectedEnd, self.position))?;
        if self.source.as_bytes().get(self.position..end) != Some(literal) {
            return Err(JsonError::new(
                JsonErrorKind::UnexpectedToken,
                self.position,
            ));
        }
        self.position = end;
        Ok(())
    }

    fn require_container_depth(&self, depth: usize) -> Result<(), JsonError> {
        if depth >= BUDGETS.max_json_depth {
            Err(JsonError::new(JsonErrorKind::DepthLimit, self.position))
        } else {
            Ok(())
        }
    }

    fn skip_whitespace(&mut self) {
        while matches!(self.peek(), Some(b' ' | b'\t' | b'\n' | b'\r')) {
            self.position += 1;
        }
    }

    fn peek(&self) -> Option<u8> {
        self.source.as_bytes().get(self.position).copied()
    }

    fn take(&mut self, expected: u8) -> bool {
        if self.peek() == Some(expected) {
            self.position += 1;
            true
        } else {
            false
        }
    }
}

pub(crate) fn canonical_bytes(value: &JsonValue) -> Vec<u8> {
    let mut output = String::new();
    write_canonical(value, &mut output);
    output.into_bytes()
}

fn write_canonical(value: &JsonValue, output: &mut String) {
    match value {
        JsonValue::Null => output.push_str("null"),
        JsonValue::Bool(true) => output.push_str("true"),
        JsonValue::Bool(false) => output.push_str("false"),
        JsonValue::Integer(integer) => output.push_str(&integer.to_string()),
        JsonValue::String(string) => write_string(string, output),
        JsonValue::Array(values) => {
            output.push('[');
            for (index, item) in values.iter().enumerate() {
                if index != 0 {
                    output.push(',');
                }
                write_canonical(item, output);
            }
            output.push(']');
        }
        JsonValue::Object(values) => {
            output.push('{');
            let mut entries = values.iter().collect::<Vec<_>>();
            entries.sort_by(|(left, _), (right, _)| {
                left.encode_utf16()
                    .collect::<Vec<_>>()
                    .cmp(&right.encode_utf16().collect::<Vec<_>>())
            });
            for (index, (key, item)) in entries.into_iter().enumerate() {
                if index != 0 {
                    output.push(',');
                }
                write_string(key, output);
                output.push(':');
                write_canonical(item, output);
            }
            output.push('}');
        }
    }
}

fn write_string(value: &str, output: &mut String) {
    output.push('"');
    for character in value.chars() {
        match character {
            '"' => output.push_str("\\\""),
            '\\' => output.push_str("\\\\"),
            '\u{0008}' => output.push_str("\\b"),
            '\u{000c}' => output.push_str("\\f"),
            '\n' => output.push_str("\\n"),
            '\r' => output.push_str("\\r"),
            '\t' => output.push_str("\\t"),
            '\u{0000}'..='\u{001f}' => {
                use std::fmt::Write as _;
                let _ = write!(output, "\\u{:04x}", u32::from(character));
            }
            _ => output.push(character),
        }
    }
    output.push('"');
}

pub(crate) fn object(entries: impl IntoIterator<Item = (String, JsonValue)>) -> JsonValue {
    JsonValue::Object(entries.into_iter().collect())
}

pub(crate) fn strings(values: impl IntoIterator<Item = &'static str>) -> JsonValue {
    JsonValue::Array(
        values
            .into_iter()
            .map(|value| JsonValue::String(value.to_owned()))
            .collect(),
    )
}
