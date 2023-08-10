use crate::terrain::color::AtomColor;

use smartstring::alias::String;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ValueUntyped<'a> {
    Color(AtomColor),
    EnumVariant(&'a str),
    Unit,
}

impl<'a> ValueUntyped<'a> {
    pub fn variant_name(&self) -> String {
        match self {
            ValueUntyped::Color(_) => "Color".into(),
            ValueUntyped::EnumVariant(v) => format!("{{ {v} }}").into(),
            ValueUntyped::Unit => "()".into(),
        }
    }
}
