use crate::atom_physics::{element::Element, id::IdMap};

use self::ast::Ast;

use super::{diagnostics::Diagnostics, FileId};

pub mod ast;

pub fn parse_file(
    code: &str,
    file: FileId,
    diagnostics: &mut Diagnostics,
    elements: &mut IdMap<Element>,
) {
    let ast = Ast::parse(code, file, diagnostics);
    // TODO
}
