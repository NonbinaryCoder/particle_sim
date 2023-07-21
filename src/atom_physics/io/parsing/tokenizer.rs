use std::fmt::Debug;

use super::{Keyword, Modifier, Operator, PrettyPrint};

pub struct Tokenizer<'a> {
    code: &'a [u8],
}

pub fn tokenize(code: &[u8]) -> Tokenizer<'_> {
    Tokenizer { code }
}

impl<'a> Iterator for Tokenizer<'a> {
    type Item = Token<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        let whitespace_len = self
            .code
            .iter()
            .take_while(|&&ch| ch.is_ascii_whitespace() && ch != b'\n')
            .count();
        (_, self.code) = self.code.split_at(whitespace_len);
        if let [first, code @ ..] = self.code {
            macro_rules! char_token {
                ($ret:expr) => {{
                    let ret = $ret;
                    self.code = code;
                    Some(ret)
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

                    Some(match literal {
                        b"element" => Token::Keyword(Keyword::Element),
                        _ => Token::Literal(literal),
                    })
                }
            }
        } else {
            None
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

impl<'a> Debug for Token<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
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
