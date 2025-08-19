use std::fmt::Display;

use serde::Serialize;

#[derive(Default, Serialize)]
pub struct Link {
    pub method: String,
    pub route: String,
    pub options: Vec<String>,
}

impl Link {
    pub fn new(method: String, route: String, options: &[String]) -> Link {
        Link {
            method,
            route,
            options: options.to_vec()
        }
    }
}

impl Display for Link {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "<li>{} <a href=\"{}\" target=\"api_mocks\">{}</a></li>", self.method.to_uppercase(), self.route, self.route)
    }
}
