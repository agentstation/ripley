use std::path::PathBuf;

use directories::ProjectDirs;

#[derive(Debug, thiserror::Error)]
pub enum DirError {
    #[error("could not determine platform directories")]
    NoPlatformDirs,
    #[error("could not create directory {path}: {source}")]
    CreateDir {
        path: PathBuf,
        source: std::io::Error,
    },
}

fn project_dirs() -> Result<ProjectDirs, DirError> {
    ProjectDirs::from("com", "agentstation", "ripley").ok_or(DirError::NoPlatformDirs)
}

fn ensure(path: PathBuf) -> Result<PathBuf, DirError> {
    std::fs::create_dir_all(&path).map_err(|e| DirError::CreateDir {
        path: path.clone(),
        source: e,
    })?;
    Ok(path)
}

pub fn config_dir() -> Result<PathBuf, DirError> {
    if let Ok(p) = std::env::var("RIPLEY_CONFIG_DIR") {
        return ensure(PathBuf::from(p));
    }
    ensure(project_dirs()?.config_dir().to_path_buf())
}

pub fn data_dir() -> Result<PathBuf, DirError> {
    if let Ok(p) = std::env::var("RIPLEY_DATA_DIR") {
        return ensure(PathBuf::from(p));
    }
    ensure(project_dirs()?.data_dir().to_path_buf())
}

pub fn cache_dir() -> Result<PathBuf, DirError> {
    if let Ok(p) = std::env::var("RIPLEY_CACHE_DIR") {
        return ensure(PathBuf::from(p));
    }
    ensure(project_dirs()?.cache_dir().to_path_buf())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dir_functions_return_paths() {
        let config = config_dir().expect("config_dir should succeed");
        let data = data_dir().expect("data_dir should succeed");
        let cache = cache_dir().expect("cache_dir should succeed");

        assert!(config.is_absolute());
        assert!(data.is_absolute());
        assert!(cache.is_absolute());
        assert!(config.exists());
        assert!(data.exists());
        assert!(cache.exists());
    }
}
