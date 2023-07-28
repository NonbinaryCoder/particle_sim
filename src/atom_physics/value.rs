use crate::terrain::color::AtomColor;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ValueUntyped {
    Color(AtomColor),
    Unit,
}

impl ValueUntyped {
    pub fn variant_name(&self) -> &'static str {
        match self {
            ValueUntyped::Color(_) => "color",
            ValueUntyped::Unit => "()",
        }
    }
}
