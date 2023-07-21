use std::fmt::Display;

mod tokenizer;

pub fn parse(code: &[u8]) {
    println!("[");
    for token in tokenizer::tokenize(code) {
        println!("    {:?}", token);
    }
    println!("]");
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Position {
    pub file_id: u16,
    pub index: u32,
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
