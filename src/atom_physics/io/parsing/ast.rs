use nom::{
    branch::alt,
    bytes::complete::{tag, take_till, take_while1},
    character::complete::{alphanumeric1, char, multispace0},
    combinator::recognize,
    error::ErrorKind,
    sequence::{delimited, pair, preceded, separated_pair, terminated},
    Parser,
};

use crate::atom_physics::io::{
    diagnostics::{self, Diagnostic, Diagnostics, Position, Positioned, Span},
    FileId,
};

#[derive(Debug, Clone)]
pub enum Ast<'a> {
    Block(Positioned<Vec<Ast<'a>>>),
    Ident(Positioned<&'a str>),
    HexColor(Positioned<&'a str>),
    Element {
        name: Positioned<&'a str>,
        body: Vec<Ast<'a>>,
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
            ) => a_name.object == b_name.object && a_body == b_body,
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
    pub fn parse(contents: &'a str, file: FileId, diagnostics: &mut Diagnostics) -> Vec<Ast<'a>> {
        let s = Span::new_extra(contents, file);

        match block(BlockTy::File)(trim_start(s)) {
            Ok((_, asts)) => asts.object,
            Err(e) => {
                let e = match e {
                    nom::Err::Incomplete(_) => unreachable!(),
                    nom::Err::Error(e) => e,
                    nom::Err::Failure(e) => e,
                };
                diagnostics.add(e);
                Vec::new()
            }
        }
    }
}

type IResult<'a, O, E = ParseError> = nom::IResult<Span<'a>, O, E>;

fn ast(s: Span<'_>) -> IResult<'_, Ast<'_>> {
    alt((
        block(BlockTy::Bracket).map(Ast::Block),
        element,
        variable_assign,
        ident.map(Ast::Ident),
        hex_color,
    ))(s)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum BlockTy {
    Bracket,
    File,
}

impl BlockTy {
    pub fn open(self, s: Span<'_>) -> IResult<'_, ()> {
        match self {
            BlockTy::Bracket => char('{').and(multispace0).map(drop).parse(s),
            BlockTy::File => Ok((s, ())),
        }
    }

    pub fn close(self, s: Span<'_>) -> IResult<'_, ()> {
        let s = trim_start(s);
        match self {
            BlockTy::Bracket => char('}').and(multispace0).map(drop).parse(s),
            BlockTy::File if s.len() == 0 => Ok((s, ())),
            BlockTy::File => ParseErrorKind::ExpectedEof.at(s).error(),
        }
    }
}

fn block(ty: BlockTy) -> impl Fn(Span<'_>) -> IResult<'_, Positioned<Vec<Ast<'_>>>> {
    move |original| {
        let mut block = Vec::new();
        let (mut s, ()) = ty.open(original)?;
        loop {
            match ty.close(s) {
                Ok((rem, ())) => {
                    break Ok((rem, Position::from_start_end(original, rem).position(block)))
                }
                Err(e) if s.len() == 0 => break Err(e),
                Err(_) => {}
            };
            let next;
            (s, next) = ast(s)?;
            block.push(next);
        }
    }
}

fn ident(s: Span<'_>) -> IResult<'_, Positioned<&'_ str>> {
    terminated(
        recognize(pair(
            take_while1(|ch: char| {
                !ch.is_whitespace() && !ch.is_ascii_digit() && !ch.is_ascii_punctuation()
            }),
            take_till(char::is_whitespace),
        )),
        multispace0,
    )
    .map(Into::into)
    .parse(s)
    .map_err(|err| {
        err.map(|err| ParseError {
            kind: ParseErrorKind::ExpectedIdentifier,
            ..err
        })
    })
}

fn hex_color(s: Span<'_>) -> IResult<'_, Ast<'_>> {
    delimited(char('#'), alphanumeric1, multispace0)
        .map(|color: Span| Ast::HexColor(color.into()))
        .parse(s)
}

fn variable_assign(s: Span<'_>) -> IResult<'_, Ast<'_>> {
    separated_pair(ident, pair(char('='), multispace0), ast)
        .map(|(name, value)| Ast::VariableAssign {
            variable: name,
            value: Box::new(value),
        })
        .parse(s)
}

fn element(s: Span<'_>) -> IResult<'_, Ast<'_>> {
    preceded(
        tag("element").and(multispace0),
        separated_pair(ident, multispace0, block(BlockTy::Bracket)),
    )
    .map(|(name, body)| Ast::Element {
        name,
        body: body.object,
    })
    .parse(s)
    .map_err(|err| {
        err.map(|err| match err.kind {
            ParseErrorKind::Nom(ErrorKind::Tag) => ParseError {
                kind: ParseErrorKind::ExpectedKeywordElement,
                ..err
            },
            _ => err,
        })
    })
}

fn trim_start(s: Span<'_>) -> Span<'_> {
    multispace0::<_, ()>(s).unwrap().0
}

#[derive(Debug, Clone)]
pub struct ParseError {
    pub position: Position,
    pub kind: ParseErrorKind,
}

impl ParseError {
    pub fn error<T>(self) -> Result<T, nom::Err<Self>> {
        Err(nom::Err::Error(self))
    }
}

#[derive(Debug, Clone)]
pub enum ParseErrorKind {
    Nom(ErrorKind),
    WrongChar { expected: char },
    ExpectedIdentifier,
    ExpectedKeywordElement,
    ExpectedEof,
}

impl ParseErrorKind {
    pub fn at(self, pos: impl Into<Position>) -> ParseError {
        ParseError {
            position: pos.into(),
            kind: self,
        }
    }
}

impl<'a> nom::error::ParseError<Span<'a>> for ParseError {
    fn from_error_kind(input: Span<'a>, kind: ErrorKind) -> Self {
        let kind = match kind {
            ErrorKind::AlphaNumeric => ParseErrorKind::ExpectedIdentifier,
            kind => ParseErrorKind::Nom(kind),
        };
        ParseError {
            position: input.into(),
            kind,
        }
    }

    fn append(_: Span<'a>, _: ErrorKind, other: Self) -> Self {
        other
    }

    fn from_char(input: Span<'a>, ch: char) -> Self {
        ParseErrorKind::WrongChar { expected: ch }.at(input)
    }
}

impl Diagnostic for ParseError {
    fn level(&self) -> diagnostics::Level {
        diagnostics::Level::Error
    }
}

#[cfg(test)]
mod tests {
    use crate::atom_physics::{id::MappedToId, io::FileContents};

    use super::*;

    fn parsing_test(input: &str, output: &[Ast]) {
        let mut diagnostics = Diagnostics::init();
        let parsed_block = Ast::parse(input, 0, &mut diagnostics);
        if !diagnostics.is_empty() {
            let mut map = FileContents::create_map();
            map.insert("input", FileContents(input.to_owned())).unwrap();
            diagnostics.print_to_console(&map);
            panic!("No diagnostics should appear in a successful parse");
        }
        assert_eq!(parsed_block, output);
    }

    #[test]
    fn literal() {
        parsing_test("Name", &[Ast::Ident(Position::TEST.position("Name"))]);
    }

    #[test]
    fn element() {
        parsing_test(
            "\
element Bedrock {
    color = #686868
}",
            &[Ast::Element {
                name: Position::TEST.position("Bedrock"),
                body: vec![Ast::VariableAssign {
                    variable: Position::TEST.position("color"),
                    value: Box::new(Ast::HexColor(Position::TEST.position("686868"))),
                }],
            }],
        );
    }

    #[test]
    fn variable_assign() {
        parsing_test(
            "color = #FFFFFF",
            &[Ast::VariableAssign {
                variable: Position::TEST.position("color"),
                value: Box::new(Ast::HexColor(Position::TEST.position("FFFFFF"))),
            }],
        );
    }

    #[test]
    fn empty_block() {
        parsing_test("{}", &[Ast::Block(Position::TEST.position(Vec::new()))]);
    }

    #[test]
    fn empty_block_in_block() {
        parsing_test(
            "{{}}",
            &[Ast::Block(Position::TEST.position(vec![Ast::Block(
                Position::TEST.position(Vec::new()),
            )]))],
        );
    }

    #[test]
    fn empty_blocks_many_newlines() {
        parsing_test(
            "\n   {\n \n{  \n}\n}\n\n",
            &[Ast::Block(Position::TEST.position(vec![Ast::Block(
                Position::TEST.position(Vec::new()),
            )]))],
        );
    }

    #[test]
    fn hex_colors() {
        fn va<'a>(name: &'a str, value: &'a str) -> Ast<'a> {
            Ast::VariableAssign {
                variable: Position::TEST.position(name),
                value: Box::new(Ast::HexColor(Position::TEST.position(value))),
            }
        }
        parsing_test(
            "\
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
l = #ABCDEfa",
            &[
                va("a", "012"),
                va("b", "abc"),
                va("c", "ABC"),
                va("d", "0123"),
                va("e", "abcd"),
                va("f", "ABCD"),
                va("g", "012345"),
                va("h", "abcdef"),
                va("i", "ABCDEF"),
                va("j", "01234567"),
                va("k", "abcdefab"),
                va("l", "ABCDEfa"),
            ],
        );
    }

    #[test]
    fn enum_variants() {
        fn va<'a>(name: &'a str, value: &'a str) -> Ast<'a> {
            Ast::VariableAssign {
                variable: Position::TEST.position(name),
                value: Box::new(Ast::Ident(Position::TEST.position(value))),
            }
        }

        parsing_test(
            "\
a = SameAlpha
b = OtherThing",
            &[va("a", "SameAlpha"), va("b", "OtherThing")],
        )
    }

    #[test]
    fn value_in_block() {
        parsing_test(
            "color = { #FFFFFF }",
            &[Ast::VariableAssign {
                variable: Position::TEST.position("color"),
                value: Box::new(Ast::Block(
                    Position::TEST.position(vec![Ast::HexColor(Position::TEST.position("FFFFFF"))]),
                )),
            }],
        );
    }
}
