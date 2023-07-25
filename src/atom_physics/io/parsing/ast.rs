use core::fmt;

use crate::{atom_physics::io::diagnostics::Diagnostics, terrain::color::AtomColor};

use super::{
    tokenizer::{BracketTy, Token, Tokenizer},
    Keyword, Modifier, Operator, Position, PrettyPrint,
};

#[derive(Clone, PartialEq, Eq)]
pub enum Ast<'a> {
    Literal(&'a [u8]),
    Block(Vec<Ast<'a>>),
    Color(AtomColor),
    Element {
        name: &'a [u8],
        body: Vec<Ast<'a>>,
    },
    VariableAssign {
        variable: &'a [u8],
        value: Box<Ast<'a>>,
    },
}

impl<'a> fmt::Debug for Ast<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Ast::Literal(l) => f
                .debug_tuple("Ast::Literal")
                .field(&PrettyPrint(l))
                .finish(),
            Ast::Block(b) => f.debug_tuple("Ast::Block").field(b).finish(),
            Ast::Color(c) => f.debug_tuple("Ast::Color").field(c).finish(),
            Ast::Element { name, body } => f
                .debug_struct("Ast::Element")
                .field("name", &PrettyPrint(name))
                .field("body", body)
                .finish(),
            Ast::VariableAssign { variable, value } => f
                .debug_struct("Ast::VariableAssign")
                .field("variable", &PrettyPrint(variable))
                .field("value", value)
                .finish(),
        }
    }
}

pub fn parse_block<'a>(
    tokens: &mut Tokenizer<'a>,
    start_pos: Option<Position>,
    diagnostics: &mut Diagnostics,
) -> Vec<Ast<'a>> {
    let mut ast = Vec::new();
    while let Some((token, pos)) = tokens.next() {
        match token {
            Token::Keyword(Keyword::Element) => ast.push(parse_element(tokens, diagnostics)),
            Token::Operator(o) => {
                diagnostics
                    .error("Unexpected operator")
                    .quoted_context(o.variant_name())
                    .position(pos);
            }
            Token::Modifier(Modifier::HexColor) => {
                ast.push(Ast::Color(parse_hex_color(tokens, diagnostics)))
            }
            Token::Literal(l) => match tokens.next() {
                None => {
                    diagnostics.error("Expected \"=\", found EOF");
                }
                Some((Token::Operator(Operator::Assign), _)) => ast.push(Ast::VariableAssign {
                    variable: l,
                    value: Box::new(parse_value(tokens, diagnostics)),
                }),
                Some((token, pos)) => {
                    diagnostics
                        .error(format!("Expected \"=\", found {}", token))
                        .position(pos);
                }
            },
            Token::Bracket {
                ty: BracketTy::Curvy,
                open: true,
            } => ast.push(Ast::Block(parse_block(tokens, Some(pos), diagnostics))),
            Token::Bracket { open: false, .. } => {
                if start_pos.is_some() {
                    return ast;
                } else {
                    diagnostics.error("Unmatched closing bracket").position(pos);
                }
            }
            Token::Newline => {}
        }
    }
    ast
}

fn parse_value<'a>(tokens: &mut Tokenizer<'a>, diagnostics: &mut Diagnostics) -> Ast<'a> {
    match tokens.skip_whitespace().next() {
        Some((Token::Keyword(kw), pos)) => {
            diagnostics
                .error("Unexpected keyword")
                .quoted_context(kw.variant_name())
                .position(pos);
            Ast::Block(Vec::new())
        }
        Some((Token::Operator(o), pos)) => {
            diagnostics
                .error("Unexpected operator")
                .quoted_context(o.variant_name())
                .position(pos);
            Ast::Block(Vec::new())
        }
        Some((Token::Modifier(Modifier::HexColor), _)) => {
            Ast::Color(parse_hex_color(tokens, diagnostics))
        }
        Some((
            Token::Bracket {
                ty: BracketTy::Curvy,
                open: true,
            },
            pos,
        )) => return Ast::Block(parse_block(tokens, Some(pos), diagnostics)),
        Some((Token::Bracket { open: false, .. }, pos)) => {
            diagnostics
                .error("Unexpected closing bracket")
                .position(pos);
            Ast::Block(Vec::new())
        }
        Some((Token::Literal(l), _)) => Ast::Literal(l),
        Some((Token::Newline, _)) => unreachable!(),
        None => {
            diagnostics.error("Missing value");
            Ast::Block(Vec::new())
        }
    }
}

fn parse_hex_color(tokens: &mut Tokenizer, diagnostics: &mut Diagnostics) -> AtomColor {
    match tokens.skip_whitespace().next() {
        Some((Token::Literal(value), pos)) => {
            for item in value {
                match item {
                    b'0'..=b'9' | b'a'..=b'f' | b'A'..=b'F' => {}
                    _ => {
                        diagnostics
                            .error("Hex color values must be in the range 0 to F")
                            .position(pos);

                        return AtomColor::INVISIBLE;
                    }
                }
            }

            fn map(v: u8) -> u8 {
                match v {
                    b'0'..=b'9' => v - b'0',
                    b'a'..=b'f' => v - b'a' + 0xA,
                    b'A'..=b'F' => v - b'A' + 0xA,
                    _ => 0,
                }
            }

            match *value {
                [r, g, b] => AtomColor::from_parts(map(r) << 4, map(g) << 4, map(b) << 4, 0xFF),
                [r, g, b, a] => {
                    AtomColor::from_parts(map(r) << 4, map(g) << 4, map(b) << 4, map(a) << 4)
                }
                [r0, r1, g0, g1, b0, b1] => AtomColor::from_parts(
                    map(r0) << 4 | map(r1),
                    map(g0) << 4 | map(g1),
                    map(b0) << 4 | map(b1),
                    0xFF,
                ),
                [r0, r1, g0, g1, b0, b1, a0, a1] => AtomColor::from_parts(
                    map(r0) << 4 | map(r1),
                    map(g0) << 4 | map(g1),
                    map(b0) << 4 | map(b1),
                    map(a0) << 4 | map(a1),
                ),
                _ => {
                    diagnostics
                        .error("Hex color values must have 3, 4, 6, or 8 elements")
                        .position(pos);
                    AtomColor::INVISIBLE
                }
            }
        }
        Some((token, pos)) => {
            diagnostics
                .error(format!("Expected hex color literal, found {}", token))
                .position(pos);
            AtomColor::INVISIBLE
        }
        None => {
            diagnostics.error("Missing value");
            AtomColor::INVISIBLE
        }
    }
}

fn parse_element<'a>(tokens: &mut Tokenizer<'a>, diagnostics: &mut Diagnostics) -> Ast<'a> {
    let name = match tokens.skip_whitespace().next() {
        Some((Token::Literal(name), _)) => name,
        None => {
            diagnostics.error("Expected element name, found EOF");
            return Ast::Element {
                name: b"",
                body: Vec::new(),
            };
        }
        Some((token, pos)) => {
            diagnostics
                .error(format!("Expected element name, found {}", token))
                .position(pos);
            b""
        }
    };
    let body = match tokens.skip_whitespace().next() {
        Some((
            Token::Bracket {
                ty: BracketTy::Curvy,
                open: true,
            },
            pos,
        )) => parse_block(tokens, Some(pos), diagnostics),
        None => {
            diagnostics.error("Expected element body, found EOF");
            Vec::new()
        }
        Some((token, pos)) => {
            diagnostics
                .error(format!("Expected \"{{\", found {}", token))
                .position(pos);
            Vec::new()
        }
    };
    Ast::Element { name, body }
}

#[cfg(test)]
mod tests {
    use crate::atom_physics::io::parsing::tokenizer;

    use super::*;

    fn parsing_test(input: &[u8], output: &[Ast]) {
        let mut diagnostics = Diagnostics::init();
        let parsed_block = parse_block(&mut tokenizer::tokenize(input, 0), None, &mut diagnostics);
        if !diagnostics.is_empty() {
            diagnostics.print_to_console(&[("input".to_owned(), input.to_vec())]);
            panic!("No diagnostics should appear in a successful parse");
        }
        assert_eq!(parsed_block, output);
    }

    #[test]
    fn element() {
        parsing_test(
            b"\
element Bedrock {
    color = #686868
}",
            &[Ast::Element {
                name: b"Bedrock",
                body: vec![Ast::VariableAssign {
                    variable: b"color",
                    value: Box::new(Ast::Color(AtomColor::from_u32(0x686868FF))),
                }],
            }],
        );
    }

    #[test]
    fn variable_assign() {
        parsing_test(
            b"color = #FFFFFF",
            &[Ast::VariableAssign {
                variable: b"color",
                value: Box::new(Ast::Color(AtomColor::from_u32(0xFFFFFFFF))),
            }],
        );
    }

    #[test]
    fn empty_block() {
        parsing_test(b"{}", &[Ast::Block(Vec::new())]);
    }

    #[test]
    fn empty_block_in_block() {
        parsing_test(b"{{}}", &[Ast::Block(vec![Ast::Block(Vec::new())])]);
    }

    #[test]
    fn empty_blocks_many_newlines() {
        parsing_test(
            b"{\n\n{\n}\n}\n\n",
            &[Ast::Block(vec![Ast::Block(Vec::new())])],
        );
    }

    #[test]
    fn hex_colors() {
        fn va(name: &[u8], value: u32) -> Ast {
            Ast::VariableAssign {
                variable: name,
                value: Box::new(Ast::Color(AtomColor::from_u32(value))),
            }
        }
        parsing_test(
            b"\
a = #012
b = #abc
c = #ABC
d = #0123
e = #abcd
f = #ABCD
g = #012345
h = #abcdef
i = #ABCDEF
j = #01234567
k = #abcdefab
l = #ABCDEfab",
            &[
                va(b"a", 0x001020FF),
                va(b"b", 0xA0B0C0FF),
                va(b"c", 0xA0B0C0FF),
                va(b"d", 0x00102030),
                va(b"e", 0xA0B0C0D0),
                va(b"f", 0xA0B0C0D0),
                va(b"g", 0x012345FF),
                va(b"h", 0xABCDEFFF),
                va(b"i", 0xABCDEFFF),
                va(b"j", 0x01234567),
                va(b"k", 0xABCDEFAB),
                va(b"l", 0xABCDEFAB),
            ],
        );
    }

    #[test]
    fn value_in_block() {
        parsing_test(
            b"color = { #FFFFFF }",
            &[Ast::VariableAssign {
                variable: b"color",
                value: Box::new(Ast::Block(vec![Ast::Color(AtomColor::from_u32(
                    0xFFFFFFFF,
                ))])),
            }],
        );
    }
}
