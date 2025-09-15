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
    extension == "txt" || extension == "md" || extension == "json" || extension == "jgd" || extension == "sql"
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
