use crate::link::Link;

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
        Pages { links, index_template, scripts_template, styles_template }
    }
}

impl Pages {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn push_link(&mut self, method: String, route: String, options: &[String]){
        self.links.push(Link::new(method, route, options));
    }

    pub fn render_index(&self) -> String {
        let json = serde_json::to_string(&self.links);
        let mock_routes = format!("let mock_routes = {};", json.unwrap());

        let scripts = format!(r#"<script type="text/javascript">
    {}
    {}
        </script>"#, mock_routes, self.scripts_template);

        let styles = format!(r#"<style>
            {}
        </style>"#, self.styles_template);

        self.index_template
            .replace(r#"<script src="/mock-routes.js"></script>"#, "")
            .replace(r#"<script src="/scripts.js"></script>"#, &scripts)
            .replace(r#"<link rel="stylesheet" href="/styles.css" />"#, &styles)
    }

}

