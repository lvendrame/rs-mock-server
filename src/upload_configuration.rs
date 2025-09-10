use std::ffi::OsStr;

pub struct UploadConfiguration {
    pub uploads_path: String,
    pub clean_uploads: bool,
}

impl UploadConfiguration {

    pub fn new(uploads_path: String, clean_uploads: bool) -> Self {
        Self { uploads_path, clean_uploads }
    }

    pub fn clean_upload_folder(&self) {
        use std::fs;

        if !self.clean_uploads {
            return;
        }

        match fs::read_dir(&self.uploads_path) {
            Ok(entries) => {
                for entry in entries.flatten() {
                    let entry_path = entry.path();

                    if entry_path.is_file() &&
                        !entry_path.extension().and_then(OsStr::to_str).unwrap_or_default().eq_ignore_ascii_case("toml") {
                        if let Err(e) = fs::remove_file(&entry_path) {
                            eprintln!("⚠️ Failed to delete file {}: {}", entry_path.display(), e);
                        } else {
                            println!("🗑️ Deleted uploaded file: {}", entry_path.display());
                        }
                    }
                }
                println!("✔️ Cleaned uploads folder: {}", self.uploads_path);
            }
            Err(e) => {
                eprintln!("⚠️ Failed to read uploads directory {}: {}", self.uploads_path, e);
            }
        }
    }

}
