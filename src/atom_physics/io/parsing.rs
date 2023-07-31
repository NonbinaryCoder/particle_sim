use crate::atom_physics::{
    element::{Element, ElementId},
    id::{IdMap, InsertError},
    value::ValueUntyped,
    PrettyPrint,
};

use self::ast::Ast;

use super::diagnostics::Diagnostics;

mod ast;
mod tokenizer;

pub fn parse_file(
    code: &[u8],
    file: FileId,
    diagnostics: &mut Diagnostics,
    elements: &mut IdMap<Element>,
) {
    let mut tokens = tokenizer::tokenize(code, file);
    let ast = ast::parse_block(&mut tokens, None, diagnostics);
    for ast in ast {
        match ast {
            Ast::Element { name, body } => {
                let element = parse_element(body, diagnostics);
                match elements.insert(name, element) {
                    Ok(_) => {}
                    Err(InsertError::DuplicateName) => {
                        diagnostics.error(format!(
                            "Element names may not be duplicated, but name {} was",
                            PrettyPrint(name)
                        ));
                    }
                    Err(InsertError::NoMoreIds) => {
                        diagnostics.error(format!(
                            "Limit of {} elements exceeded",
                            ElementId::max_value()
                        ));
                    }
                }
            }
            ast => {
                diagnostics.error(format!("Expected element, found {}", ast.variant_name()));
            }
        }
    }
}

fn parse_element(body: Vec<Ast>, diagnostics: &mut Diagnostics) -> Element {
    let mut element = Element::default();
    for ast in body {
        match ast {
            Ast::VariableAssign { variable, value } => match variable {
                b"color" => match value.const_eval(diagnostics) {
                    ValueUntyped::Color(c) => element.color = c,
                    value => {
                        diagnostics.error(format!(
                            "Property `color` expects a value of type `Color`, but found `{}`",
                            value.variant_name()
                        ));
                    }
                },
                variable => {
                    diagnostics
                        .error("Unknown variable")
                        .quoted_context(PrettyPrint(variable));
                }
            },
            ast => {
                diagnostics.error(format!("Expected property, found {}", ast.variant_name()));
            }
        }
    }
    element
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
