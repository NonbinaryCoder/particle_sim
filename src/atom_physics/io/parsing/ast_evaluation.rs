use crate::{
    atom_physics::{
        io::diagnostics::{self, Diagnostic, Positioned},
        value::ValueUntyped,
    },
    terrain::color::AtomColor,
};

use super::Ast;

impl<'a> Ast<'a> {
    pub fn const_eval(&self) -> Result<ValueUntyped<'a>, Positioned<EvalError>> {
        match self {
            Ast::Block(b) => match b.as_slice() {
                [] => Ok(ValueUntyped::Unit),
                [ast] => ast.const_eval(),
                _ => Err(b.position.position(EvalError::NotConst)),
            },
            Ast::Ident(i) => Ok(ValueUntyped::EnumVariant(i)),
            Ast::HexColor(c) => {
                for (pos, digit) in c.char_indices() {
                    if !matches!(digit, '0'..='9' | 'a'..='f' | 'A'..='F') {
                        return Err(c
                            .position
                            .char_inline(pos)
                            .position(EvalError::InvalidHexDigit));
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
                match *c.as_bytes() {
                    [y] => Ok(ValueUntyped::Color(AtomColor::from_grey(map(y)))),
                    [y0, y1] => Ok(ValueUntyped::Color(AtomColor::from_grey(
                        map(y0) << 4 | map(y1),
                    ))),
                    [r, g, b] => Ok(ValueUntyped::Color(AtomColor::from_parts(
                        map(r) << 4,
                        map(g) << 4,
                        map(b) << 4,
                        0xFF,
                    ))),
                    [r, g, b, a] => Ok(ValueUntyped::Color(AtomColor::from_parts(
                        map(r) << 4,
                        map(g) << 4,
                        map(b) << 4,
                        map(a) << 4,
                    ))),
                    [r0, r1, g0, g1, b0, b1] => Ok(ValueUntyped::Color(AtomColor::from_parts(
                        map(r0) << 4 | map(r1),
                        map(g0) << 4 | map(g1),
                        map(b0) << 4 | map(b1),
                        0xFF,
                    ))),
                    [r0, r1, g0, g1, b0, b1, a0, a1] => {
                        Ok(ValueUntyped::Color(AtomColor::from_parts(
                            map(r0) << 4 | map(r1),
                            map(g0) << 4 | map(g1),
                            map(b0) << 4 | map(b1),
                            map(a0) << 4 | map(a1),
                        )))
                    }
                    _ => Err(c.position.position(EvalError::InvalidHexColorLen)),
                }
            }
            Ast::Element { .. } => Ok(ValueUntyped::Unit),
            Ast::VariableAssign { .. } => Ok(ValueUntyped::Unit),
        }
    }
}

#[derive(Debug, Clone)]
pub enum EvalError {
    NotConst,
    InvalidHexDigit,
    InvalidHexColorLen,
}

impl Diagnostic for EvalError {
    fn level(&self) -> diagnostics::Level {
        diagnostics::Level::Error
    }

    fn description(&self) -> String {
        match self {
            EvalError::NotConst => {
                "Cannot evaluate non-const expression in const context".to_owned()
            }
            EvalError::InvalidHexDigit => {
                "Invalid hex digit (valid digits are 0-9, a-f, and A-F)".to_owned()
            }
            EvalError::InvalidHexColorLen => {
                "Hex colors must be in the format of y, yy, rgb, rgba, rrggbb, or rrggbbaa"
                    .to_owned()
            }
        }
    }
}
