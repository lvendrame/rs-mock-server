use std::{ffi::OsString, path::Path};

pub fn get_file_extension(file_path: &OsString) -> String {
    Path::new(file_path)
        .extension()
        .and_then(std::ffi::OsStr::to_str)
        .unwrap_or_default()
        .to_string()
}

pub fn is_text_file(file_path: &OsString) -> bool {
    let extension = get_file_extension(file_path);
    extension == "txt"
        || extension == "md"
        || extension == "json"
        || extension == "jgd"
        || extension == "sql"
}

pub fn is_json(file_path: &OsString) -> bool {
    let extension = get_file_extension(file_path);
    extension == "json"
}

pub fn is_jgd(file_path: &OsString) -> bool {
    let extension = get_file_extension(file_path);
    extension == "jgd"
}

pub fn is_sql(file_path: &OsString) -> bool {
    let extension = get_file_extension(file_path);
    extension == "sql"
}

pub fn is_toml(file_path: &OsString) -> bool {
    let extension = get_file_extension(file_path);
    extension == "toml"
}

pub trait SleepThread {
    fn sleep_thread(self);
}

impl SleepThread for Option<u16> {
    fn sleep_thread(self) {
        if let Some(delay) = self {
            let millis = std::time::Duration::from_millis(delay.into());
            std::thread::sleep(millis);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::ffi::OsString;
    use std::time::Instant;

    #[test]
    fn file_type_helpers_detect_supported_extensions() {
        assert_eq!(get_file_extension(&OsString::from("data.json")), "json");
        assert_eq!(get_file_extension(&OsString::from("README")), "");
        assert!(is_text_file(&OsString::from("data.json")));
        assert!(is_text_file(&OsString::from("data.jgd")));
        assert!(is_text_file(&OsString::from("query.sql")));
        assert!(is_json(&OsString::from("data.json")));
        assert!(is_jgd(&OsString::from("data.jgd")));
        assert!(is_sql(&OsString::from("query.sql")));
        assert!(is_toml(&OsString::from("config.toml")));
        assert!(!is_text_file(&OsString::from("image.png")));
    }

    #[test]
    fn sleep_thread_handles_none_and_some() {
        let start = Instant::now();
        None::<u16>.sleep_thread();
        Some(1).sleep_thread();
        assert!(start.elapsed().as_millis() >= 1);
    }
}
