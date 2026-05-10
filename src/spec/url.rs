use url::Url;

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct UrlSpec {
    url: Url,
}

impl UrlSpec {
    pub fn parse(input: &str) -> Result<Self, url::ParseError> {
        Ok(Self {
            url: Url::parse(input)?,
        })
    }

    pub fn as_str(&self) -> &str {
        self.url.as_str()
    }

    pub fn url(&self) -> &Url {
        &self.url
    }
}
