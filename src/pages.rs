use crate::link::Link;

/// Renderer for the embedded mock-server home page.
pub struct Pages {
    links: Vec<Link>,
    index_template: &'static str,
    scripts_template: &'static str,
    styles_template: &'static str,
}

impl Default for Pages {
    fn default() -> Self {
        let links = vec![];
        let index_template = include_str!("home/index.html");
        let scripts_template = include_str!("home/scripts.js");
        let styles_template = include_str!("home/styles.css");
        Pages {
            links,
            index_template,
            scripts_template,
            styles_template,
        }
    }
}

impl Pages {
    /// Creates an empty home page renderer with embedded assets.
    pub fn new() -> Self {
        Self::default()
    }

    /// Adds a route entry to the home page.
    pub fn push_link(&mut self, method: String, route: String, options: &[String]) {
        self.links.push(Link::new(method, route, options));
    }

    /// Renders the full home page HTML with route data and assets inlined.
    pub fn render_index(&self) -> String {
        let json = serde_json::to_string(&self.links);
        let mock_routes = format!("let mock_routes = {};", json.unwrap());

        let scripts = format!(
            r#"<script type="text/javascript">
    {}
    {}
        </script>"#,
            mock_routes, self.scripts_template
        );

        let styles = format!(
            r#"<style>
            {}
        </style>"#,
            self.styles_template
        );

        self.index_template
            .replace(r#"<script src="/mock-routes.js"></script>"#, "")
            .replace(r#"<script src="/scripts.js"></script>"#, &scripts)
            .replace(r#"<link rel="stylesheet" href="/styles.css" />"#, &styles)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn render_index_inlines_assets_and_routes() {
        let mut pages = Pages::new();
        pages.push_link(
            "POST".to_string(),
            "/api/users".to_string(),
            &["upload".to_string()],
        );

        let html = pages.render_index();

        assert!(html.contains("let mock_routes ="));
        assert!(html.contains("/api/users"));
        assert!(html.contains("POST"));
        assert!(html.contains("<script type=\"text/javascript\">"));
        assert!(html.contains("<style>"));
        assert!(!html.contains(r#"<script src="/mock-routes.js"></script>"#));
    }
}
