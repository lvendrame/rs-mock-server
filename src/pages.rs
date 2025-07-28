use crate::link::Link;

pub struct Pages {
    links: Vec<Link>,
    index_template: &'static str,
    pub home_template: &'static str,
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

    pub fn push_link(&mut self, method: String, url: String){
        self.links.push(Link::new(method, url));
    }

    pub fn render_index(&self) -> String {

        let links = self.links.iter().map(|link|  {
            link.to_string()
        }).collect::<Vec<String>>().join("\n");

        self.index_template.replace("{{links}}", &links)
    }
}

