use crate::terrain::color::AtomColor;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ValueUntyped<'a> {
    Color(AtomColor),
    EnumVariant(&'a str),
    Unit,
}

impl<'a> ValueUntyped<'a> {
    pub fn variant_name(&self) -> String {
        match self {
            ValueUntyped::Color(_) => "Color".to_owned(),
            ValueUntyped::EnumVariant(v) => format!("{{ {v} }}"),
            ValueUntyped::Unit => "()".to_owned(),
        }
    }
}
