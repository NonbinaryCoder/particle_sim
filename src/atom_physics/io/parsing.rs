use std::fmt;

use self::ast::Ast;

use super::diagnostics::Diagnostics;

mod ast;
mod tokenizer;

#[must_use]
pub fn parse_file<'a>(code: &'a [u8], file: FileId, diagnostics: &mut Diagnostics) -> Vec<Ast<'a>> {
    let mut tokens = tokenizer::tokenize(code, file);
    ast::parse_block(&mut tokens, None, diagnostics)
}

pub type FileId = u16;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Position {
    pub index: u32,
    pub file: FileId,
}

impl Position {
    pub fn top_of(file: FileId) -> Position {
        Position { index: 0, file }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Keyword {
    /// "element"
    Element,
}

impl Keyword {
    pub fn variant_name(self) -> &'static str {
        match self {
            Keyword::Element => "element",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Operator {
    /// "="
    Assign,
}

impl Operator {
    pub fn variant_name(self) -> &'static str {
        match self {
            Operator::Assign => "=",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Modifier {
    /// "#"
    HexColor,
}

impl Modifier {
    pub fn variant_name(self) -> &'static str {
        match self {
            Modifier::HexColor => "#",
        }
    }
}

#[derive(Clone, Copy)]
pub struct PrettyPrint<'a>(&'a [u8]);

impl<'a> fmt::Display for PrettyPrint<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match std::str::from_utf8(self.0) {
            Ok(s) => write!(f, "{s}"),
            Err(_) => write!(f, "{:?}", self.0),
        }
    }
}

impl<'a> fmt::Debug for PrettyPrint<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match std::str::from_utf8(self.0) {
            Ok(s) => write!(f, "{s:?}"),
            Err(_) => f.debug_list().entries(self.0).finish(),
        }
    }
}
