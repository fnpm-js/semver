use smol_str::SmolStr;

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct GitSpec {
    raw: SmolStr,
}

impl GitSpec {
    pub fn new(raw: impl Into<SmolStr>) -> Self {
        Self { raw: raw.into() }
    }

    pub fn as_str(&self) -> &str {
        &self.raw
    }
}
