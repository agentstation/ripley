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

pub fn config_dir() -> Result<PathBuf, DirError> {
    let dirs = project_dirs()?;
    let path = dirs.config_dir().to_path_buf();
    std::fs::create_dir_all(&path).map_err(|e| DirError::CreateDir {
        path: path.clone(),
        source: e,
    })?;
    Ok(path)
}

pub fn data_dir() -> Result<PathBuf, DirError> {
    let dirs = project_dirs()?;
    let path = dirs.data_dir().to_path_buf();
    std::fs::create_dir_all(&path).map_err(|e| DirError::CreateDir {
        path: path.clone(),
        source: e,
    })?;
    Ok(path)
}

pub fn cache_dir() -> Result<PathBuf, DirError> {
    let dirs = project_dirs()?;
    let path = dirs.cache_dir().to_path_buf();
    std::fs::create_dir_all(&path).map_err(|e| DirError::CreateDir {
        path: path.clone(),
        source: e,
    })?;
    Ok(path)
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
