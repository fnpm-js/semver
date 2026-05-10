use smol_str::SmolStr;

use super::VersionSpec;

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct AliasSpec {
    package: SmolStr,
    req: Box<VersionSpec>,
}

impl AliasSpec {
    pub fn new(package: impl Into<SmolStr>, req: VersionSpec) -> Self {
        Self {
            package: package.into(),
            req: Box::new(req),
        }
    }
    pub fn package(&self) -> &str {
        self.package.as_str()
    }
    pub fn req(&self) -> &VersionSpec {
        &self.req
    }
}