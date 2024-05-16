use crate::error::{BingoError, BingoResult};
use serde::{Deserialize, Serialize};
use std::{os::unix::fs::PermissionsExt, path::Path};

static CONFIG_DIR: &str = ".bingo";
static CONFIG_FILE: &str = "bingo.json";
static CONFIG_VERSION: &str = env!("CARGO_PKG_VERSION");

pub fn get_config_dir() -> String {
    let home = std::env::var("HOME").unwrap();
    format!("{}/{}", home, CONFIG_DIR)
}

pub fn get_config_file() -> String {
    let config_dir = get_config_dir();
    format!("{}/{}", config_dir, CONFIG_FILE)
}

pub fn get_bingo_bin_dir() -> String {
    let config_dir = get_config_dir();
    format!("{}/bin", config_dir)
}

fn create_config_dir(path: &str) {
    let path = std::path::Path::new(path);
    if path.exists() {
        return;
    } else {
        std::fs::create_dir_all(path).unwrap();
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy)]
pub enum ExecutableType {
    Binary,
    LinkBinary,
}

impl From<&str> for ExecutableType {
    fn from(s: &str) -> Self {
        match s {
            "b" => ExecutableType::Binary,
            "lb" => ExecutableType::LinkBinary,
            _ => ExecutableType::Binary,
        }
    }
}

impl Into<&'static str> for ExecutableType {
    fn into(self) -> &'static str {
        match self {
            ExecutableType::Binary => "b",
            ExecutableType::LinkBinary => "lb",
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Executable {
    pub name: String,
    pub path: String,
    pub executable_type: ExecutableType,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BingoConfig {
    pub version: String,
    pub executables: Vec<Executable>,
}

impl BingoConfig {
    pub fn new() -> BingoConfig {
        // init bingo dir
        create_config_dir(&get_bingo_bin_dir());
        BingoConfig {
            version: String::new(),
            executables: Vec::new(),
        }
    }

    pub fn from_json(json: &str) -> BingoResult<BingoConfig> {
        let mut config = BingoConfig::new();
        let data = serde_json::from_str(json);
        let data: serde_json::Value = match data {
            Ok(data) => data,
            Err(e) => {
                let e = BingoError::ConfigFileError(e.to_string());
                return Err(e);
            }
        };
        let version = data["version"].as_str().unwrap();
        config.version = version.to_string();
        let executables = data["executables"].as_array().unwrap();
        for executable in executables {
            let name = executable["name"].as_str().unwrap();
            let path = executable["path"].as_str().unwrap();
            let executable_type = match executable["executable_type"].as_str().unwrap() {
                "Binary" => ExecutableType::Binary,
                "LinkBinary" => ExecutableType::LinkBinary,
                _ => ExecutableType::Binary,
            };
            config.executables.push(Executable {
                name: name.to_string(),
                path: path.to_string(),
                executable_type,
            });
        }
        Ok(config)
    }

    pub fn to_json(&self) -> BingoResult<String> {
        let s = serde_json::to_string_pretty(self);
        match s {
            Ok(s) => Ok(s),
            Err(e) => {
                let e = BingoError::ConfigFileError(e.to_string());
                Err(e)
            }
        }
    }
}

impl Default for BingoConfig {
    fn default() -> Self {
        BingoConfig::new()
    }
}

fn link_file(src: &str, dest: &str) {
    let src = std::path::Path::new(src);
    let dest = std::path::Path::new(dest);
    if dest.exists() {
        std::fs::remove_file(dest).unwrap();
    }
    std::os::unix::fs::symlink(src, dest).unwrap();
}

fn copy_file(src: &str, dest: &str) {
    let src = std::path::Path::new(src);
    let dest = std::path::Path::new(dest);
    if dest.exists() {
        std::fs::remove_file(dest).unwrap();
    }
    std::fs::copy(src, dest).unwrap();
}

impl BingoConfig {
    pub fn set_version(&mut self) {
        self.version = CONFIG_VERSION.to_string();
    }

    pub fn install_executables(
        path: &str,
        name: &str,
        executable_type: ExecutableType,
    ) -> BingoResult<()> {
        let config_dir = get_bingo_bin_dir();
        let dest = format!("{}/{}", config_dir, name);
        match executable_type {
            ExecutableType::Binary => copy_file(path, &dest),
            ExecutableType::LinkBinary => link_file(path, &dest),
        }
        // make executable
        let dest = std::path::Path::new(&dest);
        if let Ok(perms) = match executable_type {
            ExecutableType::Binary => dest.metadata(),
            ExecutableType::LinkBinary => dest.symlink_metadata(),
        } {
            perms.permissions().set_mode(0o777);
            return Ok(());
        } else {
            let e = BingoError::PermissionDenied(dest.to_str().unwrap().to_string());
            return Err(e);
        }
    }

    pub fn uninstall_executables(name: &str) {
        let config_dir = get_bingo_bin_dir();
        let dest = format!("{}/{}", config_dir, name);
        if std::path::Path::new(&dest).exists() {
            std::fs::remove_file(dest).unwrap();
        }
    }

    pub fn add_executable(
        &mut self,
        path: &Path,
        name: &str,
        executable_type: ExecutableType,
    ) -> BingoResult<()> {
        // check if name already exists
        for executable in &self.executables {
            if executable.name == name {
                let e = BingoError::DuplicateExecutableName(name.to_string());
                return Err(e);
            }
        }
        // check if path exists
        if !path.exists() {
            let e = BingoError::FileNotFound(path.to_str().unwrap().to_string());
            return Err(e);
        }

        // check if path is a file
        if !path.is_file() {
            let e = BingoError::ExecutableNotFile(path.to_str().unwrap().to_string());
            return Err(e);
        }

        let mut path = path.to_path_buf();
        // convert path to absolute path
        if !path.is_absolute() {
            path = std::env::current_dir().unwrap().join(path);
        }

        // check if path is executable
        if path.is_symlink() {
            if !path.symlink_metadata().unwrap().permissions().mode() & 0o111 != 0 {
                let e = BingoError::ExecutableNotExecutable(path.to_str().unwrap().to_string());
                return Err(e);
            }
        } else {
            if !path.metadata().unwrap().permissions().mode() & 0o111 != 0 {
                let e = BingoError::ExecutableNotExecutable(path.to_str().unwrap().to_string());
                return Err(e);
            }
        }

        // check if executable already exists
        for executable in &mut self.executables {
            if executable.name == name {
                // uninstall old executable
                BingoConfig::uninstall_executables(&name);
                // install new executable
                BingoConfig::install_executables(path.to_str().unwrap(), name, executable_type)?;
                // upadte executable
                executable.path = path.to_str().unwrap().to_string();
                executable.executable_type = executable_type;
                executable.name = name.to_string();
                return Ok(());
            }
        }

        let executable = Executable {
            name: name.to_string(),
            path: path.to_str().unwrap().to_string(),
            executable_type,
        };
        BingoConfig::install_executables(path.to_str().unwrap(), name, executable_type)?;
        self.executables.push(executable);

        Ok(())
    }

    pub fn remove_executable(&mut self, name: &str) -> BingoResult<()> {
        let mut index = 0;
        let mut found = false;
        for (i, executable) in self.executables.iter().enumerate() {
            if executable.name == name {
                index = i;
                found = true;
                break;
            }
        }
        if !found {
            return Ok(());
        }
        let executable = &self.executables[index];
        BingoConfig::uninstall_executables(&executable.name);
        self.executables.remove(index);
        Ok(())
    }

    pub fn rename_executable(&mut self, old_name: &str, new_name: &str) -> BingoResult<()> {
        let mut found = false;
        for executable in &mut self.executables {
            if executable.name == old_name {
                found = true;
                executable.name = new_name.to_string();
                break;
            }
        }
        if !found {
            return Ok(());
        }
        let bin_dir_path = get_bingo_bin_dir();
        let old_path = format!("{}/{}", bin_dir_path, old_name);
        let new_path = format!("{}/{}", bin_dir_path, new_name);
        std::fs::rename(old_path, new_path).unwrap();
        Ok(())
    }
}

#[derive(Debug)]
pub struct BingoConfigFile {
    pub config_dir_path: String,
    pub config_file_path: String,
    pub config: BingoConfig,
}

impl BingoConfigFile {
    pub fn new() -> BingoConfigFile {
        let config_dir_path = get_config_dir();
        let config_file_path = get_config_file();
        let config = BingoConfig::new();
        BingoConfigFile {
            config_dir_path,
            config_file_path,
            config,
        }
    }

    pub fn init() -> BingoResult<()> {
        let mut config = BingoConfig::new();
        config.set_version();
        let config_file = get_config_file();
        let path = std::path::Path::new(&config_file);
        if path.exists() {
            return Ok(());
        }
        let config = config.to_json()?;
        match std::fs::write(config_file, config) {
            Ok(_) => Ok(()),
            Err(err) => {
                let e = BingoError::ConfigFileError(err.to_string());
                Err(e)
            }
        }
    }

    pub fn load(&mut self) -> BingoResult<()> {
        let config_file_path = self.config_file_path.clone();
        let config = std::fs::read_to_string(config_file_path);
        let config = match config {
            Err(err) => {
                let e = BingoError::ConfigFileNotFound(err.to_string());
                return Err(e);
            }
            Ok(s) => s,
        };
        let config = BingoConfig::from_json(&config)?;
        self.config = config;
        Ok(())
    }

    pub fn save(&self) -> BingoResult<()> {
        let config_file_path = self.config_file_path.clone();
        let config = self.config.to_json()?;
        match std::fs::write(config_file_path, config) {
            Ok(_) => Ok(()),
            Err(err) => {
                let e = BingoError::ConfigFileError(err.to_string());
                Err(e)
            }
        }
    }

    pub fn export_path(&self) -> String {
        let bin = get_bingo_bin_dir();
        format!("export PATH={}:$PATH", bin)
    }
}
