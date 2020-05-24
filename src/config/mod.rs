use crate::auth::{Credentials, Platform};
use std::collections::HashMap;
use dirs_next::config_dir;
use std::path::PathBuf;
use std::fs::{read_to_string, read_dir, DirEntry, File, create_dir_all, remove_dir_all};
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
///
/// A Profile is only allowed to join the channel its named after.
#[derive(Debug, Serialize, Deserialize, Eq, PartialEq, Clone)]
pub struct Profile {
    #[serde(skip)]
    name: String,
    credentials: HashMap<Platform, Credentials>,
}

impl Profile {
    pub fn new(name: String) -> Self {
        Profile {
            name,
            credentials: HashMap::new()
        }
    }

    pub fn profile_dir(name: OsString) -> PathBuf {
        Profiles::profiles_dir().join(name)
    }

    pub fn from_dir(dir: &DirEntry) -> Result<Self, ProfileError> {
        let cfg_file = dir.path().join("config.json");
        let content = read_to_string(cfg_file).map_err(ProfileError::from)?;
        let mut profile: Profile = serde_json::from_str(&content).map_err(ProfileError::from)?;
        profile.name = dir.file_name().into_string().expect("failed to create string from profile dir name");
        Ok(profile)
    }

    /// Sets the given credentials for the platform. Overwrites existing credentials for the platform.
    pub fn set_credentials(&mut self, platform: Platform, creds: Credentials) {
        self.credentials.insert(platform, creds);
    }

    pub fn get_credentials(&self, platform: &Platform) -> Option<&Credentials> {
        self.credentials.get(platform)
    }

    pub fn save(&self) -> Result<(), ProfileError> {
        let path = Profiles::profiles_dir().join(&self.name);
        let json = serde_json::to_string_pretty(self).map_err(ProfileError::from)?;
        create_dir_all(&path).expect("failed to create profile directory");
        let mut file = File::create(path.join("config.json")).map_err(ProfileError::from)?;
        file.write(json.as_bytes()).map_err(ProfileError::from)?;
        Ok(())
    }

    pub fn delete(&self) -> Result<(), ProfileError> {
        remove_dir_all(Profiles::profiles_dir().join(&self.name))
            .map_err(|why| ProfileError::IO(why))
    }
}

impl Display for Profile {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        writeln!(f, "Location:\t{}", Self::profile_dir(OsString::from(&self.name)).display())?;
        writeln!(f, "Name:\t\t{}", self.name)?;
        if self.credentials.is_empty() {
            write!(f, "Credentials:\tNone")?;
        } else {
            writeln!(f, "Credentials:")?;
            for (platform, creds) in self.credentials.iter() {
                writeln!(f, "\t{:?}: {}", platform, creds)?;
            }
        }
        Ok(())
    }
}

#[derive(Debug)]
pub struct Profiles {
    profiles: HashMap<OsString, Profile>
}

impl Profiles {
    pub fn cfg_dir() -> PathBuf {
        config_dir().expect("missing config directory").join("botrs")
    }

    pub fn profiles_dir() -> PathBuf {
        Self::cfg_dir().join("profiles")
    }

    pub fn load() -> Self {
        // TODO: Create directories if not present currently
        let path = Self::profiles_dir();
        create_dir_all(&path).expect("failed to create profile config files");
        let paths = read_dir(&path).expect("failed to read config directory");
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

    pub fn add(&mut self, profile: Profile) -> Result<(), ProfileError> {
        let osstr = OsString::from(&profile.name);
        if self.profiles.contains_key(&osstr) {
            return Err(ProfileError::AlreadyExists(osstr));
        }
        self.profiles.insert(osstr, profile);
        Ok(())
    }

    pub fn delete<S: AsRef<str>>(&mut self, name: S) -> Result<(), ProfileError> {
        let osstr = OsString::from(name.as_ref());
        if let Some(profile) = self.profiles.remove(&osstr) {
            profile.delete()?;
        }
        Ok(())
    }

    pub fn get<S: AsRef<str>>(&self, profile: S) -> Option<&Profile> {
        let osstr = OsString::from(profile.as_ref());
        self.profiles.get(&osstr)
    }

    pub fn get_mut<S: AsRef<str>>(&mut self, profile: S) -> Option<&mut Profile> {
        let osstr = OsString::from(profile.as_ref());
        self.profiles.get_mut(&osstr)
    }

    pub fn save(&self) -> Result<(), ProfileError> {
        for (_, profile) in self.profiles.iter() {
            profile.save()?;
        }
        Ok(())
    }
}
