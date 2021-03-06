use anyhow::Result;
use log::error;
use std::path::{Path, PathBuf};
use thiserror::Error;

pub fn resolve_imports(group_path: &Path, imports: &[String]) -> Result<Vec<String>> {
  let mut paths = Vec::new();

  // Get the containing directory
  let current_dir = if group_path.is_file() {
    if let Some(parent) = group_path.parent() {
      parent
    } else {
      return Err(
        ResolveImportError::Failed(format!(
          "unable to resolve imports for match group starting from current path: {:?}",
          group_path
        ))
        .into(),
      );
    }
  } else {
    group_path
  };

  for import in imports.iter() {
    let import_path = PathBuf::from(import);

    // Absolute or relative import
    let full_path = if import_path.is_relative() {
      current_dir.join(import_path)
    } else {
      import_path
    };

    match dunce::canonicalize(&full_path) {
      Ok(canonical_path) => {
        if canonical_path.exists() && canonical_path.is_file() {
          paths.push(canonical_path)
        } else {
          // Best effort imports
          error!("unable to resolve import at path: {:?}", canonical_path);
        }
      }
      Err(error) => {
        error!(
          "unable to canonicalize import path: {:?}, with error: {}",
          full_path, error
        );
      }
    }
  }

  let string_paths = paths
    .into_iter()
    .map(|path| path.to_string_lossy().to_string())
    .collect();

  Ok(string_paths)
}

#[derive(Error, Debug)]
pub enum ResolveImportError {
  #[error("resolve import failed: `{0}`")]
  Failed(String),
}

#[cfg(test)]
pub mod tests {
  use super::*;
  use crate::util::tests::use_test_directory;
  use std::fs::create_dir_all;

  #[test]
  fn resolve_imports_works_correctly() {
    use_test_directory(|_, match_dir, _| {
      let sub_dir = match_dir.join("sub");
      create_dir_all(&sub_dir).unwrap();

      let base_file = match_dir.join("base.yml");
      std::fs::write(&base_file, "test").unwrap();

      let another_file = match_dir.join("another.yml");
      std::fs::write(&another_file, "test").unwrap();

      let sub_file = sub_dir.join("sub.yml");
      std::fs::write(&sub_file, "test").unwrap();

      let absolute_file = sub_dir.join("absolute.yml");
      std::fs::write(&absolute_file, "test").unwrap();

      let imports = vec![
        "another.yml".to_string(),
        "sub/sub.yml".to_string(),
        absolute_file.to_string_lossy().to_string(),
        "sub/invalid.yml".to_string(), // Should be skipped
      ];

      assert_eq!(
        resolve_imports(&base_file, &imports).unwrap(),
        vec![
          another_file.to_string_lossy().to_string(),
          sub_file.to_string_lossy().to_string(),
          absolute_file.to_string_lossy().to_string(),
        ]
      );
    });
  }

  #[test]
  fn resolve_imports_parent_relative_path() {
    use_test_directory(|_, match_dir, _| {
      let sub_dir = match_dir.join("sub");
      create_dir_all(&sub_dir).unwrap();

      let base_file = match_dir.join("base.yml");
      std::fs::write(&base_file, "test").unwrap();

      let sub_file = sub_dir.join("sub.yml");
      std::fs::write(&sub_file, "test").unwrap();

      let imports = vec!["../base.yml".to_string()];

      assert_eq!(
        resolve_imports(&sub_file, &imports).unwrap(),
        vec![base_file.to_string_lossy().to_string(),]
      );
    });
  }
}
