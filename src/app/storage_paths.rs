use std::{fs, io, path::PathBuf};

#[cfg(not(target_os = "macos"))]
use std::env;

const APP_DIR_NAME: &str = "ClipKeeper";

pub fn data_file_path(file_name: &str) -> io::Result<PathBuf> {
    Ok(app_data_dir()?.join(file_name))
}

pub fn app_data_dir() -> io::Result<PathBuf> {
    #[cfg(target_os = "macos")]
    {
        let base_dir = dirs::data_local_dir().ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::NotFound,
                "failed to resolve macOS application support directory",
            )
        })?;
        let app_dir = base_dir.join(APP_DIR_NAME);
        fs::create_dir_all(&app_dir)?;
        return Ok(app_dir);
    }

    #[cfg(not(target_os = "macos"))]
    {
        let exe_path = env::current_exe()?;
        let exe_dir = exe_path.parent().ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::NotFound,
                "failed to resolve executable directory",
            )
        })?;
        Ok(exe_dir.to_path_buf())
    }
}