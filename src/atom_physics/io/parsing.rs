use std::fmt::Display;

use super::diagnostics::Diagnostics;

mod tokenizer;

pub fn parse_file(code: &[u8], file: FileId, diagnostics: &mut Diagnostics) -> Result<(), ()> {
    println!("[");
    for (token, position) in tokenizer::tokenize(code, file) {
        println!("    {:?}", token);
        diagnostics.warn(format!("{:?}", token)).position(position);
    }
    println!("]");
    Ok(())
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

pub struct PrettyPrint<'a>(&'a [u8]);

impl<'a> Display for PrettyPrint<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match std::str::from_utf8(self.0) {
            Ok(s) => write!(f, "{s}"),
            Err(_) => write!(f, "{:?}", self.0),
        }
    }
}
