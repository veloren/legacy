use toml;

use std::{
    fmt,
    fs::File,
    io::{self, Read, Write},
    path::Path,
};

const KEYS_PATH: &str = "keybinds.toml";

#[derive(Debug)]
enum Error {
    Io(io::Error),
    TomlDe(toml::de::Error),
    TomlSer(toml::ser::Error),
}

impl From<io::Error> for Error {
    fn from(err: io::Error) -> Error { Error::Io(err) }
}

impl From<toml::de::Error> for Error {
    fn from(err: toml::de::Error) -> Error { Error::TomlDe(err) }
}

impl From<toml::ser::Error> for Error {
    fn from(err: toml::ser::Error) -> Error { Error::TomlSer(err) }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Error::Io(e) => write!(f, "{}", e),
            Error::TomlDe(e) => write!(f, "{}", e),
            Error::TomlSer(e) => write!(f, "{}", e),
        }
    }
}

#[derive(Serialize, Deserialize, PartialEq)]
pub struct Keybinds {
    pub general: General,
    pub mount: Mount,
}

#[derive(Serialize, Deserialize, PartialEq)]
pub struct General {
    // Movement
    pub back: Option<u32>,
    pub forward: Option<u32>,
    pub left: Option<u32>,
    pub right: Option<u32>,
    pub dodge: Option<u32>,
    pub crouch: Option<u32>,
    pub jump: Option<u32>,

    // Actions
    pub attack_1: Option<u32>,
    pub attack_2: Option<u32>,
    pub interact: Option<u32>,
    pub mount: Option<u32>,
    pub skill_1: Option<u32>,
    pub skill_2: Option<u32>,
    pub skill_3: Option<u32>,
    pub use_item: Option<u32>,

    // Menus
    pub chat: Option<u32>,
    pub inventory: Option<u32>,
    pub pause: Option<u32>,
}

#[derive(Serialize, Deserialize, PartialEq)]
pub struct Mount {
    pub dismount: Option<u32>,
}

impl Keybinds {
    pub fn new() -> Keybinds {
        let path = Path::new(KEYS_PATH);
        let keys = Keybinds::load_from(path).unwrap_or_else(|_| Keybinds::default());
        if let Err(e) = keys.save_to_file() {
            warn!("failed to save keybinds.toml: {} ", e);
        }
        keys
    }

    fn load_from(path: &Path) -> Result<Keybinds, Error> {
        // Load the config from the given file path.
        let mut file = File::open(path)?;
        let mut content = String::new();
        file.read_to_string(&mut content)?;

        let default_keys = Keybinds::default();
        let user_keys: Keybinds = toml::from_str(&mut content)?;
        if user_keys == default_keys {
            // If the user has the default binds, skip the integrity check.
            Ok(user_keys)
        } else {
            // Make sure the user has all bindings in their file. If not, it adds them to the
            // loaded struct.
            // This avoids a panic on missing keybind.

            // Helper variables to clean up code. Add any new input modes here.
            let general = user_keys.general;
            let mount = user_keys.mount;

            // The actual integrity check
            let keys = Keybinds {
                general: General {
                    back: Some(general.back.unwrap_or(default_keys.general.back.unwrap())),
                    forward: Some(general.forward.unwrap_or(default_keys.general.forward.unwrap())),
                    left: Some(general.left.unwrap_or(default_keys.general.left.unwrap())),
                    right: Some(general.right.unwrap_or(default_keys.general.right.unwrap())),
                    dodge: Some(general.dodge.unwrap_or(default_keys.general.dodge.unwrap())),
                    crouch: Some(general.crouch.unwrap_or(default_keys.general.crouch.unwrap())),
                    jump: Some(general.jump.unwrap_or(default_keys.general.jump.unwrap())),
                    attack_1: Some(general.attack_1.unwrap_or(default_keys.general.attack_1.unwrap())),
                    attack_2: Some(general.attack_2.unwrap_or(default_keys.general.attack_2.unwrap())),
                    interact: Some(general.interact.unwrap_or(default_keys.general.interact.unwrap())),
                    mount: Some(general.mount.unwrap_or(default_keys.general.mount.unwrap())),
                    skill_1: Some(general.skill_1.unwrap_or(default_keys.general.skill_1.unwrap())),
                    skill_2: Some(general.skill_2.unwrap_or(default_keys.general.skill_2.unwrap())),
                    skill_3: Some(general.skill_3.unwrap_or(default_keys.general.skill_3.unwrap())),
                    use_item: Some(general.use_item.unwrap_or(default_keys.general.use_item.unwrap())),
                    chat: Some(general.chat.unwrap_or(default_keys.general.chat.unwrap())),
                    inventory: Some(general.inventory.unwrap_or(default_keys.general.inventory.unwrap())),
                    pause: Some(general.pause.unwrap_or(default_keys.general.pause.unwrap())),
                },

                mount: Mount {
                    dismount: Some(mount.dismount.unwrap_or(default_keys.mount.dismount.unwrap())),
                },
            };

            Ok(keys)
        }
    }

    fn save_to_file(&self) -> Result<(), Error> {
        // Writes to file. Will create a new file if it exists, or overwrite any existing one.
        let mut file = File::create(KEYS_PATH)?;
        let toml = toml::to_string(self)?;
        file.write_all(&toml.as_bytes())?;
        Ok(())
    }

    fn default() -> Keybinds {
        // The default keybinds struct. All new defaults will be added here.
        Keybinds {
            general: General {
                back: Some(31),
                forward: Some(17),
                left: Some(30),
                right: Some(32),
                dodge: Some(29),
                crouch: Some(42),
                jump: Some(57),

                attack_1: Some(100),
                attack_2: Some(100),
                interact: Some(18),
                mount: Some(20),
                skill_1: Some(2),
                skill_2: Some(3),
                skill_3: Some(4),
                use_item: Some(16),

                chat: Some(28),
                inventory: Some(48),
                pause: Some(1),
            },

            mount: Mount { dismount: Some(20) },
        }
    }
}
