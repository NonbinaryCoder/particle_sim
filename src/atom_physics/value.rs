use crate::terrain::color::AtomColor;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ValueUntyped<'a> {
    Color(AtomColor),
    EnumVariant(&'a str),
    Unit,
}

impl<'a> ValueUntyped<'a> {
    pub fn variant_name(&self) -> &'static str {
        match self {
            ValueUntyped::Color(_) => "Color",
            ValueUntyped::EnumVariant(_) => "Enum",
            ValueUntyped::Unit => "()",
        }
    }
}
