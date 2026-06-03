//! Pure route selection to filesystem path mapping.

use std::path::{Path, PathBuf};

use crate::generator::{
    content,
    domain::{GeneratedFileType, IdType, RouteKind, RouteSelection, WriteOperation, WritePlan},
};

/// Builds a write plan for a route selection.
pub fn build_route_plan(root: impl AsRef<Path>, selection: &RouteSelection) -> WritePlan {
    let root = root.as_ref();
    let mut plan = WritePlan::default();
    append_route_operations(&mut plan, root, selection);
    plan
}

fn append_route_operations(plan: &mut WritePlan, root: &Path, selection: &RouteSelection) {
    match selection.kind {
        RouteKind::Basic => push_basic(plan, root, selection),
        RouteKind::Rest => push_rest(plan, root, selection),
        RouteKind::Auth => push_auth(plan, root, selection),
        RouteKind::Upload => push_upload(plan, root, selection),
        RouteKind::Public => push_public(plan, root, selection),
        RouteKind::Graphql => push_graphql(plan, root, selection),
        RouteKind::Sql => push_sql(plan, root, selection),
    }
}

fn push_basic(plan: &mut WritePlan, root: &Path, selection: &RouteSelection) {
    push_route_file(
        plan,
        root,
        selection,
        basic_file_name(selection),
        route_content(selection),
    );
}

fn push_rest(plan: &mut WritePlan, root: &Path, selection: &RouteSelection) {
    push_route_file(
        plan,
        root,
        selection,
        rest_file_name(selection),
        route_content(selection),
    );
}

fn push_auth(plan: &mut WritePlan, root: &Path, selection: &RouteSelection) {
    let path = root.join(route_dir(&selection.route)).join("{auth}.json");
    let content = content::render_auth_json(&selection.fields);
    plan.push(WriteOperation::file(path, content));
}

fn push_upload(plan: &mut WritePlan, root: &Path, selection: &RouteSelection) {
    let path = root
        .join(route_parent_dir(&selection.route))
        .join(upload_directory_name(selection));
    plan.push(WriteOperation::directory(path));
}

fn push_public(plan: &mut WritePlan, root: &Path, selection: &RouteSelection) {
    let path = root.join(public_directory_name(&selection.route));
    plan.push(WriteOperation::directory(path));
}

fn push_graphql(plan: &mut WritePlan, root: &Path, selection: &RouteSelection) {
    let collections_dir = push_graphql_dirs(plan, root, selection);
    let seed_file = collections_dir.join(graphql_collection_file(selection));
    plan.push(WriteOperation::file(seed_file, route_content(selection)));
}

fn push_sql(plan: &mut WritePlan, root: &Path, selection: &RouteSelection) {
    let path = root
        .join(route_parent_dir(&selection.route))
        .join(sql_file_name(selection));
    plan.push(WriteOperation::file(path, sql_content(selection)));
}

fn push_route_file(
    plan: &mut WritePlan,
    root: &Path,
    selection: &RouteSelection,
    file_name: String,
    content: String,
) {
    let path = root.join(route_dir(&selection.route)).join(file_name);
    plan.push(WriteOperation::file(path, content));
}

fn push_graphql_dirs(plan: &mut WritePlan, root: &Path, selection: &RouteSelection) -> PathBuf {
    let graphql_dir = root.join(graphql_folder(selection));
    plan.push(WriteOperation::directory(&graphql_dir));
    let collections_dir = graphql_dir.join("collections");
    plan.push(WriteOperation::directory(&collections_dir));
    collections_dir
}

fn route_content(selection: &RouteSelection) -> String {
    match selection.file_type {
        GeneratedFileType::Json => match selection.kind {
            RouteKind::Rest | RouteKind::Graphql => content::render_json_array(&selection.fields),
            _ => content::render_json_object(&selection.fields),
        },
        GeneratedFileType::Jgd => content::render_jgd(selection),
        GeneratedFileType::Text => content::render_text(&selection.route),
        GeneratedFileType::Sql => sql_content(selection),
        GeneratedFileType::Directory => String::new(),
    }
}

fn sql_content(selection: &RouteSelection) -> String {
    content::render_sql(&sql_collection(selection), has_id_param(selection))
}

fn graphql_folder(selection: &RouteSelection) -> &'static str {
    if selection.protected {
        "$graphql"
    } else {
        "graphql"
    }
}

fn graphql_collection_file(selection: &RouteSelection) -> String {
    let collection = route_leaf(&selection.route).unwrap_or("items".to_string());
    format!("{}.{}", collection, data_extension(selection.file_type))
}

fn sql_collection(selection: &RouteSelection) -> String {
    selection
        .collection_name
        .clone()
        .or_else(|| route_leaf(&selection.route).map(strip_descriptor))
        .unwrap_or_else(|| "items".to_string())
}

fn has_id_param(selection: &RouteSelection) -> bool {
    selection.route.contains("{id}")
}

fn basic_file_name(selection: &RouteSelection) -> String {
    let prefix = if selection.protected { "$" } else { "" };
    let method = selection.method.to_ascii_lowercase();
    let descriptor = route_param_descriptor(&selection.route).unwrap_or_default();
    let extension = method_extension(selection.file_type);
    format!("{}{}{}.{}", prefix, method, descriptor, extension)
}

fn rest_file_name(selection: &RouteSelection) -> String {
    let prefix = if selection.protected { "$" } else { "" };
    let extension = data_extension(selection.file_type);
    format!("{}rest{}.{}", prefix, rest_descriptor(selection), extension)
}

fn rest_descriptor(selection: &RouteSelection) -> String {
    match (selection.id_key.as_str(), selection.id_type) {
        ("id", IdType::Uuid) => String::new(),
        ("id", IdType::Int) => "{int}".to_string(),
        ("id", IdType::None) => "{none}".to_string(),
        (key, IdType::Uuid) => format!("{{{}}}", key),
        (key, IdType::Int) => format!("{{{}-int}}", key),
        (key, IdType::None) => format!("{{{}-none}}", key),
    }
}

fn method_extension(file_type: GeneratedFileType) -> &'static str {
    match file_type {
        GeneratedFileType::Jgd => "jgd",
        GeneratedFileType::Text => "txt",
        GeneratedFileType::Sql => "sql",
        _ => "json",
    }
}

fn data_extension(file_type: GeneratedFileType) -> &'static str {
    match file_type {
        GeneratedFileType::Jgd => "jgd",
        _ => "json",
    }
}

fn upload_directory_name(selection: &RouteSelection) -> String {
    let prefix = if selection.protected { "$" } else { "" };
    let temp = if selection.temporary_upload {
        "{temp}"
    } else {
        ""
    };
    let leaf = route_leaf(&selection.route).unwrap_or_else(|| "upload".to_string());
    if leaf == "upload" {
        format!("{}{{upload}}{}", prefix, temp)
    } else {
        format!("{}{{upload}}{}-{}", prefix, temp, leaf)
    }
}

fn sql_file_name(selection: &RouteSelection) -> String {
    let prefix = if selection.protected { "$" } else { "" };
    let leaf = route_leaf(&selection.route).unwrap_or_else(|| "query".to_string());
    let descriptor = route_param_descriptor(&selection.route).unwrap_or_default();
    format!("{}{}{}.sql", prefix, leaf, descriptor)
}

fn public_directory_name(route: &str) -> String {
    let leaf = route_leaf(route).unwrap_or_else(|| "public".to_string());
    if leaf == "public" {
        "public".to_string()
    } else {
        format!("public-{}", leaf)
    }
}

fn route_dir(route: &str) -> PathBuf {
    let without_param = route
        .split('/')
        .filter(|segment| {
            !(segment.is_empty() || segment.starts_with('{') && segment.ends_with('}'))
        })
        .collect::<Vec<_>>();

    let mut path = PathBuf::new();
    for segment in without_param {
        path.push(segment);
    }
    path
}

fn route_parent_dir(route: &str) -> PathBuf {
    let mut segments = route
        .split('/')
        .filter(|segment| {
            !(segment.is_empty() || segment.starts_with('{') && segment.ends_with('}'))
        })
        .collect::<Vec<_>>();
    segments.pop();

    let mut path = PathBuf::new();
    for segment in segments {
        path.push(segment);
    }
    path
}

fn route_leaf(route: &str) -> Option<String> {
    route
        .split('/')
        .filter(|segment| !segment.is_empty())
        .filter(|segment| !(segment.starts_with('{') && segment.ends_with('}')))
        .next_back()
        .map(ToString::to_string)
}

fn route_param_descriptor(route: &str) -> Option<String> {
    route
        .split('/')
        .find(|segment| segment.starts_with('{') && segment.ends_with('}'))
        .map(ToString::to_string)
}

fn strip_descriptor(leaf: String) -> String {
    leaf.split('{').next().unwrap_or(&leaf).to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::generator::domain::{GeneratedFileType, RouteKind};

    #[test]
    fn basic_dynamic_route_maps_to_method_descriptor_file() {
        let selection = RouteSelection {
            route: "/api/users/{id}".to_string(),
            method: "get".to_string(),
            ..Default::default()
        };
        let plan = build_route_plan("mocks", &selection);
        assert_eq!(
            plan.operations[0].path,
            PathBuf::from("mocks/api/users/get{id}.json")
        );
    }

    #[test]
    fn rest_int_route_uses_rest_descriptor() {
        let selection = RouteSelection {
            kind: RouteKind::Rest,
            route: "/api/products".to_string(),
            id_key: "_id".to_string(),
            id_type: IdType::Int,
            ..Default::default()
        };
        let plan = build_route_plan("mocks", &selection);
        assert_eq!(
            plan.operations[0].path,
            PathBuf::from("mocks/api/products/rest{_id-int}.json")
        );
    }

    #[test]
    fn protected_upload_uses_existing_directory_convention() {
        let selection = RouteSelection {
            kind: RouteKind::Upload,
            route: "/api/files".to_string(),
            protected: true,
            temporary_upload: true,
            ..Default::default()
        };
        let plan = build_route_plan("mocks", &selection);
        assert_eq!(
            plan.operations[0].path,
            PathBuf::from("mocks/api/${upload}{temp}-files")
        );
    }

    #[test]
    fn sql_route_uses_leaf_as_filename() {
        let selection = RouteSelection {
            kind: RouteKind::Sql,
            route: "/reports/companies/{id}".to_string(),
            protected: true,
            ..Default::default()
        };
        let plan = build_route_plan("mocks", &selection);
        assert_eq!(
            plan.operations[0].path,
            PathBuf::from("mocks/reports/$companies{id}.sql")
        );
    }

    #[test]
    fn graphql_plan_creates_folder_collection_and_seed() {
        let selection = RouteSelection {
            kind: RouteKind::Graphql,
            route: "/users".to_string(),
            file_type: GeneratedFileType::Jgd,
            protected: true,
            ..Default::default()
        };
        let plan = build_route_plan("mocks", &selection);
        assert_eq!(plan.operations[0].path, PathBuf::from("mocks/$graphql"));
        assert_eq!(
            plan.operations[2].path,
            PathBuf::from("mocks/$graphql/collections/users.jgd")
        );
    }

    #[test]
    fn basic_protected_post_text_route_uses_method_prefix_and_txt_extension() {
        let selection = RouteSelection {
            route: "/api/messages/{slug}".to_string(),
            method: "POST".to_string(),
            file_type: GeneratedFileType::Text,
            protected: true,
            ..Default::default()
        };
        let plan = build_route_plan("mocks", &selection);
        assert_eq!(
            plan.operations[0].path,
            PathBuf::from("mocks/api/messages/$post{slug}.txt")
        );
    }

    #[test]
    fn rest_descriptor_covers_default_int_none_and_custom_uuid_conflicts() {
        let cases = [
            (IdType::Uuid, "id", "rest.json"),
            (IdType::Int, "id", "rest{int}.json"),
            (IdType::None, "id", "rest{none}.json"),
            (IdType::Uuid, "user_id", "rest{user_id}.json"),
            (IdType::Int, "user_id", "rest{user_id-int}.json"),
            (IdType::None, "user_id", "rest{user_id-none}.json"),
        ];
        for (id_type, id_key, file_name) in cases {
            let selection = RouteSelection {
                kind: RouteKind::Rest,
                route: "/api/users".to_string(),
                id_key: id_key.to_string(),
                id_type,
                ..Default::default()
            };
            let plan = build_route_plan("mocks", &selection);
            assert_eq!(
                plan.operations[0].path,
                PathBuf::from(format!("mocks/api/users/{file_name}"))
            );
        }
    }

    #[test]
    fn upload_directory_names_cover_plain_custom_temp_and_protected_conflicts() {
        let cases = [
            ("/upload", false, false, "mocks/{upload}"),
            ("/api/files", false, false, "mocks/api/{upload}-files"),
            ("/api/files", false, true, "mocks/api/{upload}{temp}-files"),
            ("/api/files", true, true, "mocks/api/${upload}{temp}-files"),
        ];
        for (route, protected, temporary_upload, expected) in cases {
            let selection = RouteSelection {
                kind: RouteKind::Upload,
                route: route.to_string(),
                protected,
                temporary_upload,
                ..Default::default()
            };
            let plan = build_route_plan("mocks", &selection);
            assert_eq!(plan.operations[0].path, PathBuf::from(expected));
        }
    }

    #[test]
    fn public_directory_names_cover_root_and_custom_routes() {
        let cases = [
            ("/public", "mocks/public"),
            ("/assets", "mocks/public-assets"),
            ("/static/images", "mocks/public-images"),
        ];
        for (route, expected) in cases {
            let selection = RouteSelection {
                kind: RouteKind::Public,
                route: route.to_string(),
                ..Default::default()
            };
            let plan = build_route_plan("mocks", &selection);
            assert_eq!(plan.operations[0].path, PathBuf::from(expected));
        }
    }

    #[test]
    fn graphql_json_uses_array_content_and_default_folder() {
        let selection = RouteSelection {
            kind: RouteKind::Graphql,
            route: "/inventory/items/{id}".to_string(),
            file_type: GeneratedFileType::Json,
            protected: false,
            ..Default::default()
        };
        let plan = build_route_plan("mocks", &selection);
        assert_eq!(plan.operations[0].path, PathBuf::from("mocks/graphql"));
        assert_eq!(
            plan.operations[2].path,
            PathBuf::from("mocks/graphql/collections/items.json")
        );
        assert!(
            plan.operations[2]
                .content
                .as_ref()
                .unwrap()
                .starts_with('[')
        );
    }

    #[test]
    fn sql_route_uses_collection_override_and_id_binding() {
        let selection = RouteSelection {
            kind: RouteKind::Sql,
            route: "/reports/orders/{id}".to_string(),
            collection_name: Some("archived_orders".to_string()),
            ..Default::default()
        };
        let plan = build_route_plan("mocks", &selection);
        assert_eq!(
            plan.operations[0].path,
            PathBuf::from("mocks/reports/orders{id}.sql")
        );
        assert_eq!(
            plan.operations[0].content.as_deref(),
            Some("select * from archived_orders where id = ?;")
        );
    }

    #[test]
    fn sql_route_strips_filename_descriptor_from_default_collection() {
        let selection = RouteSelection {
            kind: RouteKind::Sql,
            route: "/reports/companies{id}".to_string(),
            ..RouteSelection::default()
        };

        let plan = build_route_plan("mocks", &selection);

        assert_eq!(
            plan.operations[0].content.as_deref(),
            Some("select * from companies where id = ?;")
        );
    }

    #[test]
    fn auth_route_ignores_dynamic_param_for_directory_and_uses_auth_file() {
        let selection = RouteSelection {
            kind: RouteKind::Auth,
            route: "/api/login/{id}".to_string(),
            ..Default::default()
        };
        let plan = build_route_plan("mocks", &selection);
        assert_eq!(
            plan.operations[0].path,
            PathBuf::from("mocks/api/login/{auth}.json")
        );
        assert!(
            plan.operations[0]
                .content
                .as_ref()
                .unwrap()
                .contains("\"username\"")
        );
    }

    #[test]
    fn directory_file_type_is_never_used_as_file_extension_for_basic_route() {
        let selection = RouteSelection {
            file_type: GeneratedFileType::Directory,
            ..Default::default()
        };
        let plan = build_route_plan("mocks", &selection);
        assert_eq!(
            plan.operations[0].path,
            PathBuf::from("mocks/api/example/get.json")
        );
        assert_eq!(plan.operations[0].content.as_deref(), Some(""));
    }
}
