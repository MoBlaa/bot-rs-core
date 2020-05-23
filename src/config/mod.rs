use crate::auth::Credentials;
use std::collections::HashMap;
use dirs_next::config_dir;
use std::path::PathBuf;
use std::fs::{read_to_string, read_dir, DirEntry, File};
use std::{io, error};
use serde::export::fmt::Display;
use serde_json::Error as JsonError;
use serde::export::Formatter;
use core::fmt;
use std::ffi::OsString;
use std::io::Write;

#[derive(Debug)]
pub enum ProfileError {
    AlreadyExists(OsString),
    IO(io::Error),
    Json(JsonError)
}

impl From<io::Error> for ProfileError {
    fn from(content: io::Error) -> Self {
        ProfileError::IO(content)
    }
}

impl From<JsonError> for ProfileError {
    fn from(content: JsonError) -> Self {
        ProfileError::Json(content)
    }
}

impl error::Error for ProfileError {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        match self {
            ProfileError::IO(source) => Some(source),
            ProfileError::Json(source) => Some(source),
            ProfileError::AlreadyExists(_) => None
        }
    }
}

impl Display for ProfileError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            ProfileError::IO(why) => write!(f, "failed to read/write config file '{}'", why),
            ProfileError::Json(why) => write!(f, "invalid config file: {}", why),
            ProfileError::AlreadyExists(name) => write!(f, "profile named '{}' already exists", name.to_str().unwrap())
        }
    }
}

/// Profile configuration containing configurations for the Bot-RS Core functionality.
///
/// Profile configurations are located at `{{ENV_CONFIG_DIR}}/profiles/{profile-name}`.
#[derive(Debug, Serialize, Deserialize)]
pub struct Profile {
    credentials: HashMap<String, Credentials>,
}

impl Profile {
    pub fn new() -> Self {
        Profile {
            credentials: HashMap::new()
        }
    }

    pub fn from_dir(dir: &DirEntry) -> Result<Self, ProfileError> {
        let cfg_file = dir.path().join("config.json");
        let content = read_to_string(cfg_file).map_err(ProfileError::from)?;
        serde_json::from_str(&content).map_err(ProfileError::from)
    }

    pub fn save(&self, path: PathBuf) -> Result<(), ProfileError> {
        let json = serde_json::to_string_pretty(self).map_err(ProfileError::from)?;
        let mut file = File::create(path).map_err(ProfileError::from)?;
        file.write(json.as_bytes()).map_err(ProfileError::from)?;
        Ok(())
    }
}

#[derive(Debug)]
pub struct Profiles {
    profiles: HashMap<OsString, Profile>
}

impl Profiles {
    fn cfg_dir() -> PathBuf {
        config_dir().expect("missing config directory").join(".botrs")
    }

    fn profiles_dir() -> PathBuf {
        Self::cfg_dir().join("profiles")
    }

    pub fn load() -> Self {
        // TODO: Create directories if not present currently
        let paths = read_dir(Self::profiles_dir()).expect("failed to read config directory");
        let mut profiles = HashMap::new();

        for path in paths {
            let path = path.expect("failed to get path of configuration subdir");
            if path.file_type().expect("failed to get file-type").is_dir() {
                match Profile::from_dir(&path) {
                    Err(why) => warn!("failed to load profile config: {}", why),
                    Ok(profile) => {
                        profiles.insert(path.file_name(), profile);
                    },
                }
            }
        }

        Profiles {
            profiles
        }
    }

    pub fn create_profile(&mut self, name: &str) -> Result<(), ProfileError> {
        let osstr = OsString::from(name);
        if self.profiles.contains_key(&osstr) {
            return Err(ProfileError::AlreadyExists(osstr));
        }
        self.profiles.insert(osstr, Profile::new());
        Ok(())
    }

    pub fn save(&self) -> Result<(), ProfileError> {
        for (name, profile) in self.profiles.iter() {
            let path = Self::profiles_dir().join(name);
            profile.save(path)?;
        }
        Ok(())
    }
}
