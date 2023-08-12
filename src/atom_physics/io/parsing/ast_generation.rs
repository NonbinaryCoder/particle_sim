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

use super::Ast;

impl<'a> Ast<'a> {
    pub fn generate(
        contents: &'a str,
        file: FileId,
        diagnostics: &mut Diagnostics,
    ) -> Vec<Ast<'a>> {
        let s = Span::new_extra(contents, file);

        match block(BlockTy::File)(trim_start(s)) {
            Ok((_, asts)) => asts.object,
            Err(e) => {
                let e = match e {
                    nom::Err::Incomplete(_) => unreachable!(),
                    nom::Err::Error(e) => e,
                    nom::Err::Failure(e) => e,
                };
                diagnostics.add(e.position, e.kind);
                Vec::new()
            }
        }
    }
}

type IResult<'a, O, E = GenerateError> = nom::IResult<Span<'a>, O, E>;

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
            BlockTy::File => GenerateErrorKind::ExpectedEof.at(s).error(),
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
        err.map(|err| GenerateError {
            kind: GenerateErrorKind::ExpectedIdentifier,
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
    .map(|(name, body)| Ast::Element { name, body })
    .parse(s)
    .map_err(|err| {
        err.map(|err| match err.kind {
            GenerateErrorKind::Nom(ErrorKind::Tag) => GenerateError {
                kind: GenerateErrorKind::ExpectedKeywordElement,
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
struct GenerateError {
    pub position: Position,
    pub kind: GenerateErrorKind,
}

impl GenerateError {
    pub fn error<T>(self) -> Result<T, nom::Err<Self>> {
        Err(nom::Err::Error(self))
    }
}

#[derive(Debug, Clone)]
enum GenerateErrorKind {
    Nom(ErrorKind),
    WrongChar { expected: char },
    ExpectedIdentifier,
    ExpectedKeywordElement,
    ExpectedEof,
}

impl GenerateErrorKind {
    pub fn at(self, pos: impl Into<Position>) -> GenerateError {
        GenerateError {
            position: pos.into(),
            kind: self,
        }
    }
}

impl<'a> nom::error::ParseError<Span<'a>> for GenerateError {
    fn from_error_kind(input: Span<'a>, kind: ErrorKind) -> Self {
        let kind = match kind {
            ErrorKind::AlphaNumeric => GenerateErrorKind::ExpectedIdentifier,
            kind => GenerateErrorKind::Nom(kind),
        };
        GenerateError {
            position: input.into(),
            kind,
        }
    }

    fn append(_: Span<'a>, _: ErrorKind, other: Self) -> Self {
        other
    }

    fn from_char(input: Span<'a>, ch: char) -> Self {
        GenerateErrorKind::WrongChar { expected: ch }.at(input)
    }
}

impl Diagnostic for GenerateErrorKind {
    fn level(&self) -> diagnostics::Level {
        diagnostics::Level::Error
    }

    fn description(&self) -> String {
        match self {
            GenerateErrorKind::Nom(e) => format!("Unexpected nom error: {e:?}"),
            GenerateErrorKind::WrongChar { expected } => {
                format!("Expected character '{expected}'")
            }
            GenerateErrorKind::ExpectedIdentifier => "Expected identifier".to_owned(),
            GenerateErrorKind::ExpectedKeywordElement => r#"Expected keyword "element""#.to_owned(),
            GenerateErrorKind::ExpectedEof => "Expected EOF".to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::atom_physics::{id::MappedToId, io::FileContents};

    use super::*;

    fn pos<T>(object: T) -> Positioned<T> {
        Positioned::test_position(object)
    }

    fn parsing_test(input: &str, output: &[Ast]) {
        let mut diagnostics = Diagnostics::init();
        let parsed_block = Ast::generate(input, 0, &mut diagnostics);
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
        parsing_test("Name", &[Ast::Ident(pos("Name"))]);
    }

    #[test]
    fn element() {
        parsing_test(
            "\
element Bedrock {
    color = #686868
}",
            &[Ast::Element {
                name: pos("Bedrock"),
                body: pos(vec![Ast::VariableAssign {
                    variable: pos("color"),
                    value: Box::new(Ast::HexColor(pos("686868"))),
                }]),
            }],
        );
    }

    #[test]
    fn variable_assign() {
        parsing_test(
            "color = #FFFFFF",
            &[Ast::VariableAssign {
                variable: pos("color"),
                value: Box::new(Ast::HexColor(pos("FFFFFF"))),
            }],
        );
    }

    #[test]
    fn empty_block() {
        parsing_test("{}", &[Ast::Block(pos(Vec::new()))]);
    }

    #[test]
    fn empty_block_in_block() {
        parsing_test(
            "{{}}",
            &[Ast::Block(pos(vec![Ast::Block(pos(Vec::new()))]))],
        );
    }

    #[test]
    fn empty_blocks_many_newlines() {
        parsing_test(
            "\n   {\n \n{  \n}\n}\n\n",
            &[Ast::Block(pos(vec![Ast::Block(pos(Vec::new()))]))],
        );
    }

    #[test]
    fn hex_colors() {
        fn va<'a>(name: &'a str, value: &'a str) -> Ast<'a> {
            Ast::VariableAssign {
                variable: pos(name),
                value: Box::new(Ast::HexColor(pos(value))),
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
                variable: pos(name),
                value: Box::new(Ast::Ident(pos(value))),
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
                variable: pos("color"),
                value: Box::new(Ast::Block(pos(vec![Ast::HexColor(pos("FFFFFF"))]))),
            }],
        );
    }
}
