use std::fmt;

use super::{FileId, Keyword, Modifier, Operator, Position, PrettyPrint};

pub struct Tokenizer<'a> {
    code: &'a [u8],
    position: Position,
}

pub fn tokenize(code: &[u8], file: FileId) -> Tokenizer<'_> {
    Tokenizer {
        code,
        position: Position::top_of(file),
    }
}

impl<'a> Iterator for Tokenizer<'a> {
    type Item = (Token<'a>, Position);

    fn next(&mut self) -> Option<Self::Item> {
        let whitespace_len = self
            .code
            .iter()
            .take_while(|&&ch| ch.is_ascii_whitespace() && ch != b'\n')
            .count();
        (_, self.code) = self.code.split_at(whitespace_len);
        self.position.index += whitespace_len as u32;
        let pos = self.position;
        if let [first, code @ ..] = self.code {
            macro_rules! char_token {
                ($ret:expr) => {{
                    let ret = $ret;
                    self.code = code;
                    self.position.index += 1;
                    Some((ret, pos))
                }};
            }
            match first {
                b'\n' => char_token!(Token::Newline),
                b'{' => char_token!(Token::Bracket {
                    ty: BracketTy::Curvy,
                    open: true,
                }),
                b'}' => char_token!(Token::Bracket {
                    ty: BracketTy::Curvy,
                    open: false
                }),
                b'=' => char_token!(Token::Operator(Operator::Assign)),
                b'#' => char_token!(Token::Modifier(Modifier::HexColor)),
                _ => {
                    let literal_len = self
                        .code
                        .iter()
                        .take_while(|ch| !ch.is_ascii_whitespace())
                        .count();
                    let literal;
                    (literal, self.code) = self.code.split_at(literal_len);
                    self.position.index += literal.len() as u32;

                    let token = match literal {
                        b"element" => Token::Keyword(Keyword::Element),
                        _ => Token::Literal(literal),
                    };
                    Some((token, pos))
                }
            }
        } else {
            None
        }
    }
}

impl<'a> Tokenizer<'a> {
    pub fn skip_whitespace(&mut self) -> TokenizerSkipWhitespace<'a, '_> {
        TokenizerSkipWhitespace(self)
    }
}

pub struct TokenizerSkipWhitespace<'a, 'b>(&'b mut Tokenizer<'a>);

impl<'a, 'b> Iterator for TokenizerSkipWhitespace<'a, 'b> {
    type Item = (Token<'a>, Position);

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            match self.0.next() {
                Some((Token::Newline, _)) => continue,
                ret => break ret,
            }
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Token<'a> {
    Newline,
    Keyword(Keyword),
    Operator(Operator),
    Modifier(Modifier),
    Bracket { ty: BracketTy, open: bool },
    Literal(&'a [u8]),
}

impl<'a> fmt::Debug for Token<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Token::Newline => write!(f, "Newline"),
            Token::Keyword(kw) => write!(f, "Keyword({})", kw.variant_name()),
            Token::Operator(o) => write!(f, "Operator({})", o.variant_name()),
            Token::Modifier(m) => write!(f, "Modifier({})", m.variant_name()),
            Token::Bracket { ty, open } => {
                write!(f, "Bracket {{ ty: {}, open: {} }}", ty.variant_name(), open)
            }
            Token::Literal(l) => write!(f, "Literal({})", PrettyPrint(l)),
        }
    }
}

impl<'a> fmt::Display for Token<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Token::Newline => write!(f, "newline"),
            Token::Keyword(kw) => write!(f, "keyword \"{}\"", kw.variant_name()),
            Token::Operator(o) => write!(f, "operator \"{}\"", o.variant_name()),
            Token::Modifier(m) => write!(f, "modifier \"{}\"", m.variant_name()),
            Token::Bracket { ty, open } => match (ty, open) {
                (BracketTy::Curvy, true) => write!(f, "{{"),
                (BracketTy::Curvy, false) => write!(f, "}}"),
            },
            Token::Literal(l) => write!(f, "\"{}\"", PrettyPrint(l)),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BracketTy {
    Curvy,
}

impl BracketTy {
    pub const fn variant_name(self) -> &'static str {
        match self {
            BracketTy::Curvy => "Curvy",
        }
    }
}
