use std::ffi::OsStr;

pub struct UploadConfiguration {
    pub uploads_path: String,
    pub clean_uploads: bool,
}

impl UploadConfiguration {
    pub fn new(uploads_path: String, clean_uploads: bool) -> Self {
        Self {
            uploads_path,
            clean_uploads,
        }
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

                    if entry_path.is_file()
                        && !entry_path
                            .extension()
                            .and_then(OsStr::to_str)
                            .unwrap_or_default()
                            .eq_ignore_ascii_case("toml")
                    {
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
                eprintln!(
                    "⚠️ Failed to read uploads directory {}: {}",
                    self.uploads_path, e
                );
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn new_stores_configuration() {
        let config = UploadConfiguration::new("uploads".to_string(), true);
        assert_eq!(config.uploads_path, "uploads");
        assert!(config.clean_uploads);
    }

    #[test]
    fn clean_upload_folder_respects_flag_and_keeps_toml() {
        let temp_dir = TempDir::new().unwrap();
        let json_path = temp_dir.path().join("file.json");
        let toml_path = temp_dir.path().join("config.toml");
        std::fs::write(&json_path, "{}").unwrap();
        std::fs::write(&toml_path, "x = 1").unwrap();

        UploadConfiguration::new(temp_dir.path().to_string_lossy().to_string(), false)
            .clean_upload_folder();
        assert!(json_path.exists());

        UploadConfiguration::new(temp_dir.path().to_string_lossy().to_string(), true)
            .clean_upload_folder();
        assert!(!json_path.exists());
        assert!(toml_path.exists());
    }

    #[test]
    fn clean_upload_folder_ignores_missing_directory() {
        UploadConfiguration::new("missing-upload-dir".to_string(), true).clean_upload_folder();
    }
}
