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

    pub fn get_all_files(dir: &str) -> Vec<String> {
        let directory_to_search = PathBuf::from(dir);
        if !directory_to_search.is_dir() {
            return vec![]
        }
        let mut res = Vec::new();
        for entry in fs::read_dir(directory_to_search).unwrap() {
            let entry = entry.unwrap();
            res.push(entry.file_name().to_str().unwrap().to_string());
        }
        res
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
                let entry = entry.unwrap();
                if entry.path().is_file() && is_executable(&entry.path()) {
                    res.push(entry.file_name().to_str().unwrap().to_string());
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
