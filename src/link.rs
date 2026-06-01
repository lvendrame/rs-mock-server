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
            options: options.to_vec(),
        }
    }
}

impl Display for Link {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "<li>{} <a href=\"{}\" target=\"api_mocks\">{}</a></li>",
            self.method.to_uppercase(),
            self.route,
            self.route
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_copies_options_and_display_uses_uppercase_method() {
        let options = vec!["upload".to_string(), "download".to_string()];
        let link = Link::new("get".to_string(), "/api/users".to_string(), &options);

        assert_eq!(link.method, "get");
        assert_eq!(link.route, "/api/users");
        assert_eq!(link.options, options);
        assert_eq!(
            link.to_string(),
            r#"<li>GET <a href="/api/users" target="api_mocks">/api/users</a></li>"#
        );
    }
}
