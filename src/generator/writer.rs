//! Filesystem writes for generated plans.

use std::{fs, io, path::PathBuf};

use crate::generator::domain::{WriteOperation, WritePlan};

/// Write-plan validation error.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Collision {
    /// Existing path that would be overwritten.
    pub path: PathBuf,
}

/// Returns existing paths that would block the plan when force is disabled.
pub fn collisions(plan: &WritePlan) -> Vec<Collision> {
    plan.operations
        .iter()
        .filter(|operation| operation.path.exists())
        .map(|operation| Collision {
            path: operation.path.clone(),
        })
        .collect()
}

/// Applies a write plan.
pub fn apply_plan(plan: &WritePlan, force: bool) -> io::Result<()> {
    validate_plan(plan, force)?;
    for operation in &plan.operations {
        apply_operation(operation)?;
    }
    Ok(())
}

fn validate_plan(plan: &WritePlan, force: bool) -> io::Result<()> {
    let found_collisions = collisions(plan);
    if force || found_collisions.is_empty() {
        return Ok(());
    }
    Err(collision_error(&found_collisions[0]))
}

fn collision_error(collision: &Collision) -> io::Error {
    io::Error::new(
        io::ErrorKind::AlreadyExists,
        format!("target already exists: {}", collision.path.display()),
    )
}

fn apply_operation(operation: &WriteOperation) -> io::Result<()> {
    if let Some(content) = &operation.content {
        if let Some(parent) = operation.path.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(&operation.path, content)?;
    } else {
        fs::create_dir_all(&operation.path)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::generator::domain::WriteOperation;

    #[test]
    fn refuses_collision_without_force() {
        let temp_dir = tempfile::TempDir::new().unwrap();
        let file = temp_dir.path().join("get.json");
        fs::write(&file, "{}").unwrap();
        let plan = WritePlan {
            operations: vec![WriteOperation::file(&file, "{\"ok\":true}")],
        };

        let err = apply_plan(&plan, false).unwrap_err();
        assert_eq!(err.kind(), io::ErrorKind::AlreadyExists);
    }

    #[test]
    fn writes_files_and_directories() {
        let temp_dir = tempfile::TempDir::new().unwrap();
        let plan = WritePlan {
            operations: vec![
                WriteOperation::directory(temp_dir.path().join("api")),
                WriteOperation::file(temp_dir.path().join("api/get.json"), "{}"),
            ],
        };

        apply_plan(&plan, false).unwrap();

        assert!(temp_dir.path().join("api").is_dir());
        assert_eq!(
            fs::read_to_string(temp_dir.path().join("api/get.json")).unwrap(),
            "{}"
        );
    }

    #[test]
    fn reports_all_existing_collisions_in_plan_order() {
        let temp_dir = tempfile::TempDir::new().unwrap();
        let first = temp_dir.path().join("first.json");
        let second = temp_dir.path().join("second.json");
        fs::write(&first, "{}").unwrap();
        fs::write(&second, "{}").unwrap();
        let plan = WritePlan {
            operations: vec![
                WriteOperation::file(&first, "{\"first\":true}"),
                WriteOperation::file(&second, "{\"second\":true}"),
            ],
        };
        let found = collisions(&plan);
        assert_eq!(found[0].path, first);
        assert_eq!(found[1].path, second);
    }

    #[test]
    fn force_overwrites_existing_file() {
        let temp_dir = tempfile::TempDir::new().unwrap();
        let file = temp_dir.path().join("get.json");
        fs::write(&file, "{\"old\":true}").unwrap();
        let plan = WritePlan {
            operations: vec![WriteOperation::file(&file, "{\"new\":true}")],
        };
        apply_plan(&plan, true).unwrap();
        assert_eq!(fs::read_to_string(file).unwrap(), "{\"new\":true}");
    }

    #[test]
    fn nested_file_write_creates_missing_parent_directories() {
        let temp_dir = tempfile::TempDir::new().unwrap();
        let file = temp_dir.path().join("api/users/get.json");
        let plan = WritePlan {
            operations: vec![WriteOperation::file(&file, "{}")],
        };
        apply_plan(&plan, false).unwrap();
        assert!(file.exists());
    }

    #[test]
    fn directory_collision_is_refused_without_force() {
        let temp_dir = tempfile::TempDir::new().unwrap();
        let directory = temp_dir.path().join("api");
        fs::create_dir_all(&directory).unwrap();
        let plan = WritePlan {
            operations: vec![WriteOperation::directory(&directory)],
        };
        let err = apply_plan(&plan, false).unwrap_err();
        assert_eq!(err.kind(), io::ErrorKind::AlreadyExists);
    }

    #[test]
    fn force_accepts_existing_directory_collision() {
        let temp_dir = tempfile::TempDir::new().unwrap();
        let directory = temp_dir.path().join("api");
        fs::create_dir_all(&directory).unwrap();
        let plan = WritePlan {
            operations: vec![WriteOperation::directory(&directory)],
        };
        apply_plan(&plan, true).unwrap();
        assert!(directory.is_dir());
    }
}
