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
                    Err(InsertError::DuplicateName) => diagnostics.add(ElementError {
                        position: ast.position(),
                        kind: ElementErrorKind::DoubleDefineElement(name.object.into()),
                    }),
                    Err(InsertError::NoMoreIds) => diagnostics.add(ElementError {
                        position: ast.position(),
                        kind: ElementErrorKind::ElementLimitReached,
                    }),
                }
            }
            Ast::Block(b) => diagnostics.add(ParseError {
                position: b.position,
                kind: ParseErrorKind::UnexpectedBlock,
            }),
            Ast::Ident(i) | Ast::VariableAssign { variable: i, .. } => {
                diagnostics.add(ParseError {
                    position: i.position,
                    kind: ParseErrorKind::UnexpectedIdent,
                })
            }
            Ast::HexColor(c) => diagnostics.add(ParseError {
                position: c.position,
                kind: ParseErrorKind::UnexpectedValue,
            }),
        }
    }
}

#[derive(Debug, Clone)]
struct ParseError {
    position: Position,
    kind: ParseErrorKind,
}

#[derive(Debug, Clone)]
#[allow(clippy::enum_variant_names)]
enum ParseErrorKind {
    UnexpectedBlock,
    UnexpectedIdent,
    UnexpectedValue,
}

impl Diagnostic for ParseError {
    fn level(&self) -> diagnostics::Level {
        diagnostics::Level::Error
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
                        diagnostics.add(ElementError {
                            position: value.position(),
                            kind: ElementErrorKind::DoubleDefineVariable,
                        });
                    }
                    color_set = true;
                    match value.const_eval() {
                        Ok(ValueUntyped::Color(val)) => {
                            element.color = val;
                        }
                        Ok(val) => diagnostics.add(ElementError {
                            position: value.position(),
                            kind: ElementErrorKind::VariableType {
                                expected: "color".into(),
                                found: val.variant_name(),
                            },
                        }),
                        Err(e) => diagnostics.add(e),
                    }
                }
                "join_face" => {
                    if join_face_set {
                        diagnostics.add(ElementError {
                            position: value.position(),
                            kind: ElementErrorKind::DoubleDefineVariable,
                        });
                    }
                    join_face_set = true;
                    match value.const_eval() {
                        Ok(ValueUntyped::EnumVariant("Never")) => {
                            element.join_face = JoinFace::Never;
                        }
                        Ok(ValueUntyped::EnumVariant("SameAlpha")) => {
                            element.join_face = JoinFace::SameAlpha;
                        }
                        Ok(val) => diagnostics.add(ElementError {
                            position: value.position(),
                            kind: ElementErrorKind::VariableType {
                                expected: "{ Never | SameAlpha }".into(),
                                found: val.variant_name(),
                            },
                        }),
                        Err(e) => diagnostics.add(e),
                    }
                }
                _ => diagnostics.add(ElementError {
                    position: variable.position,
                    kind: ElementErrorKind::UnknownVariable,
                }),
            },
            _ => diagnostics.add(ElementError {
                position: ast.position(),
                kind: ElementErrorKind::UnexpectedAstKind,
            }),
        }
    }
    element
}

#[derive(Debug, Clone)]
struct ElementError {
    position: Position,
    kind: ElementErrorKind,
}

#[derive(Debug, Clone)]
enum ElementErrorKind {
    UnexpectedAstKind,
    VariableType { expected: String, found: String },
    DoubleDefineVariable,
    UnknownVariable,
    DoubleDefineElement(String),
    ElementLimitReached,
}

impl Diagnostic for ElementError {
    fn level(&self) -> diagnostics::Level {
        match self.kind {
            ElementErrorKind::DoubleDefineVariable
            | ElementErrorKind::UnknownVariable
            | ElementErrorKind::DoubleDefineElement(_)
            | ElementErrorKind::ElementLimitReached => diagnostics::Level::Warn,
            ElementErrorKind::UnexpectedAstKind | ElementErrorKind::VariableType { .. } => {
                diagnostics::Level::Error
            }
        }
    }
}
