use smol_str::SmolStr;

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct WorkspaceSpec {
    raw: SmolStr,
}

impl WorkspaceSpec {
    pub fn new(raw: impl Into<SmolStr>) -> Self {
        Self { raw: raw.into() }
    }

    pub fn as_str(&self) -> &str {
        &self.raw
    }
}
