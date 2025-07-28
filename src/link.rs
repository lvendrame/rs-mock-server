use std::fmt::Display;

#[derive(Default)]
pub struct Link {
    pub method: String,
    pub url: String,
}

impl Link {
    pub fn new(method: String, url: String) -> Link {
        Link {
            method,
            url,
        }
    }
}

impl Display for Link {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "<li>{} <a href=\"{}\" target=\"api_mocks\">{}</a></li>", self.method.to_uppercase(), self.url, self.url)
    }
}