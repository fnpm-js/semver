use smol_str::SmolStr;

use super::PackageVersionSpec;

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct AliasSpec {
    package: SmolStr,
    req: Box<PackageVersionSpec>,
}

impl AliasSpec {
    pub fn new(package: impl Into<SmolStr>, req: impl Into<PackageVersionSpec>) -> Self {
        Self {
            package: package.into(),
            req: Box::new(req.into()),
        }
    }
    pub fn package(&self) -> &str {
        self.package.as_str()
    }
    pub fn req(&self) -> &PackageVersionSpec {
        &self.req
    }
}
