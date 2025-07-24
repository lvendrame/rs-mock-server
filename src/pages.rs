pub struct Pages {
    pub links: Vec<String>,
    pub home_template: &'static str,
    pub index_template: &'static str,
}

impl Default for Pages {
    fn default() -> Self {
        let links = vec![];
        let index_template = include_str!("index.html");
        let home_template = include_str!("home.html");
        Pages { links, home_template, index_template }
    }
}

impl Pages {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn render_index(&self) -> String {

        let links = self.links.iter().map(|link|  {
            format!("<li><a href=\"{}\" target=\"api_mocks\">{}</a></li>", link, link)
        }).collect::<Vec<String>>().join("\n");


        self.index_template.replace("{{links}}", &links)
    }
}

