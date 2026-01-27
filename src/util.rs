pub mod util {
    use std::{env, fs};
    use std::os::unix::fs::PermissionsExt;
    use std::path::PathBuf;
    
    pub const TILDE: &'static str = "~";

    pub fn get_cmd_path(cmd: &str) -> Option<PathBuf> {
        let paths = env::var_os("PATH")?;
        for dir in env::split_paths(&paths) {
            let full_path = dir.as_path().join(cmd);
            if is_executable(&full_path) {
                return Some(full_path);
            }
        }
        None
    }

    pub fn get_all_executables() -> Vec<String> {
        let paths;
        match env::var_os("PATH") {
            None => {return Vec::new()}
            Some(x) => {paths = x}
        }

        let mut res = Vec::new();

        for dir in env::split_paths(&paths) {
            if !dir.is_dir() {
                continue
            }
            let entries = fs::read_dir(dir).unwrap();
            for entry in entries {
                let entry = entry.unwrap().path();
                if entry.is_file() && is_executable(&entry) {
                    res.push(entry.file_name().unwrap().to_str().unwrap().to_string());
                }
            }
        }

        res
    }

    pub fn is_executable(path_buf: &PathBuf) -> bool {
        match path_buf.metadata() {
            Ok(metadata) => metadata.is_file() && metadata.permissions().mode() & 0o111 != 0,
            Err(_) => false,
        }
    }

    pub fn into_path_str(full_path: PathBuf) -> String {
        full_path.into_os_string().into_string().unwrap()
    }
}
