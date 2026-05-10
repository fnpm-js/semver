use smol_str::SmolStr;

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct FileSpec {
    path: SmolStr,
}

impl FileSpec {
    pub fn new(path: impl Into<SmolStr>) -> Self {
        Self { path: path.into() }
    }

    pub fn path(&self) -> &str {
        &self.path
    }
}
