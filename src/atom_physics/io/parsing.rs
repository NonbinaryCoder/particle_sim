use crate::{
    atom_physics::{
        element::Element,
        id::{IdMap, InsertError},
        value::ValueUntyped,
    },
    terrain::JoinFace,
};

use super::{
    diagnostics::{self, Diagnostic, Diagnostics, Position, Positioned},
    FileId,
};

use smartstring::alias::String;

mod ast_evaluation;
mod ast_generation;

#[derive(Debug, Clone)]
pub enum Ast<'a> {
    Block(Positioned<Vec<Ast<'a>>>),
    Ident(Positioned<&'a str>),
    HexColor(Positioned<&'a str>),
    Element {
        name: Positioned<&'a str>,
        body: Positioned<Vec<Ast<'a>>>,
    },
    VariableAssign {
        variable: Positioned<&'a str>,
        value: Box<Ast<'a>>,
    },
}

impl<'a> PartialEq for Ast<'a> {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Ast::Ident(a), Ast::Ident(b)) => a.object == b.object,
            (Ast::Block(a), Ast::Block(b)) => a.object == b.object,
            (Ast::HexColor(a), Ast::HexColor(b)) => a.object == b.object,
            (
                Ast::Element {
                    name: a_name,
                    body: a_body,
                    ..
                },
                Ast::Element {
                    name: b_name,
                    body: b_body,
                    ..
                },
            ) => a_name.object == b_name.object && a_body.object == b_body.object,
            (
                Ast::VariableAssign {
                    variable: a_var,
                    value: a_val,
                    ..
                },
                Ast::VariableAssign {
                    variable: b_var,
                    value: b_val,
                    ..
                },
            ) => a_var.object == b_var.object && a_val == b_val,
            _ => false,
        }
    }
}

impl<'a> Eq for Ast<'a> {}

impl<'a> Ast<'a> {
    pub fn position(&self) -> Position {
        match self {
            Ast::Block(b) => b.position,
            Ast::Ident(i) => i.position,
            Ast::HexColor(c) => c.position.extend_back_same_line(1),
            Ast::Element { name, body } => name.position.extend_to(body.position),
            Ast::VariableAssign { variable, value } => {
                variable.position.extend_to(value.position())
            }
        }
    }
}

pub fn parse_file(
    code: &str,
    file: FileId,
    diagnostics: &mut Diagnostics,
    elements: &mut IdMap<Element>,
) {
    let asts = Ast::generate(code, file, diagnostics);
    for ast in asts {
        match ast {
            Ast::Element { name, ref body } => {
                let element = parse_element(body, diagnostics);
                match elements.insert(*name, element) {
                    Ok(_) => {}
                    Err(InsertError::DuplicateName) => diagnostics.add(
                        ast.position(),
                        ElementError::DoubleDefineElement(name.object.into()),
                    ),
                    Err(InsertError::NoMoreIds) => {
                        diagnostics.add(ast.position(), ElementError::ElementLimitReached)
                    }
                }
            }
            Ast::Block(b) => diagnostics.add(b.position, ParseError::UnexpectedBlock),
            Ast::Ident(i) | Ast::VariableAssign { variable: i, .. } => {
                diagnostics.add(i.position, ParseError::UnexpectedIdent)
            }
            Ast::HexColor(c) => diagnostics.add(c.position, ParseError::UnexpectedValue),
        }
    }
}

#[derive(Debug, Clone)]
#[allow(clippy::enum_variant_names)]
enum ParseError {
    UnexpectedBlock,
    UnexpectedIdent,
    UnexpectedValue,
}

impl Diagnostic for ParseError {
    fn level(&self) -> diagnostics::Level {
        diagnostics::Level::Error
    }

    fn description(&self) -> std::string::String {
        match self {
            ParseError::UnexpectedBlock => "Unexpected block".to_owned(),
            ParseError::UnexpectedIdent => "Unexpected identifier".to_owned(),
            ParseError::UnexpectedValue => "Unexpected value".to_owned(),
        }
    }
}

pub fn parse_element(body: &[Ast<'_>], diagnostics: &mut Diagnostics) -> Element {
    let mut element = Element::default();
    let mut color_set = false;
    let mut join_face_set = false;
    for ast in body {
        match ast {
            Ast::VariableAssign { variable, value } => match **variable {
                "color" => {
                    if color_set {
                        diagnostics.add(value.position(), ElementError::DoubleDefineVariable);
                    }
                    color_set = true;
                    match value.const_eval() {
                        Ok(ValueUntyped::Color(val)) => {
                            element.color = val;
                        }
                        Ok(val) => diagnostics.add(
                            value.position(),
                            ElementError::VariableType {
                                expected: "color".into(),
                                found: val.variant_name(),
                            },
                        ),
                        Err(e) => diagnostics.add_positioned(e),
                    }
                }
                "join_face" => {
                    if join_face_set {
                        diagnostics.add(value.position(), ElementError::DoubleDefineVariable);
                    }
                    join_face_set = true;
                    match value.const_eval() {
                        Ok(ValueUntyped::EnumVariant("Never")) => {
                            element.join_face = JoinFace::Never;
                        }
                        Ok(ValueUntyped::EnumVariant("SameAlpha")) => {
                            element.join_face = JoinFace::SameAlpha;
                        }
                        Ok(val) => diagnostics.add(
                            value.position(),
                            ElementError::VariableType {
                                expected: "{ Never | SameAlpha }".into(),
                                found: val.variant_name(),
                            },
                        ),
                        Err(e) => diagnostics.add_positioned(e),
                    }
                }
                _ => diagnostics.add(variable.position, ElementError::UnknownVariable),
            },
            _ => diagnostics.add(ast.position(), ElementError::UnexpectedAstKind),
        }
    }
    element
}

#[derive(Debug, Clone)]
enum ElementError {
    UnexpectedAstKind,
    VariableType { expected: String, found: String },
    DoubleDefineVariable,
    UnknownVariable,
    DoubleDefineElement(String),
    ElementLimitReached,
}

impl Diagnostic for ElementError {
    fn level(&self) -> diagnostics::Level {
        match self {
            ElementError::DoubleDefineVariable
            | ElementError::UnknownVariable
            | ElementError::DoubleDefineElement(_)
            | ElementError::ElementLimitReached => diagnostics::Level::Warn,
            ElementError::UnexpectedAstKind | ElementError::VariableType { .. } => {
                diagnostics::Level::Error
            }
        }
    }

    fn description(&self) -> std::string::String {
        match self {
            ElementError::UnexpectedAstKind => {
                "Element body should contain only properties, variables, and rules".to_owned()
            }
            ElementError::VariableType { expected, found } => {
                format!("Variable has type {expected}, but found value of type {found}")
            }
            ElementError::DoubleDefineVariable => "Variable defined twice".to_owned(),
            ElementError::UnknownVariable => "Unknown variable".to_owned(),
            ElementError::DoubleDefineElement(_) => {
                "Element defined twice; using second definition".to_owned()
            }
            ElementError::ElementLimitReached => format!(
                "Limit of {} elements exceeded",
                crate::atom_physics::element::ElementId::MAX
            ),
        }
    }
}
