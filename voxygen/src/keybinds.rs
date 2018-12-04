use std::{
    fmt,
    fs::File,
    io::{self, Read, Write},
    path::Path,
};

use glutin::VirtualKeyCode;
use serde::{Deserializer, Serializer};
use serde_derive::{Deserialize, Serialize};
use toml;

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

struct VKeyCodeVisitor;

impl<'de> serde::de::Visitor<'de> for VKeyCodeVisitor {
    type Value = VirtualKeyCode;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result { formatter.write_str("a virtual key code") }

    fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        match str_to_vkcode(value) {
            Some(code) => Ok(code),
            None => Err(E::custom(format!("invalid key: {}", value))),
        }
    }
}

#[derive(Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct VKeyCode(#[serde(with = "VKeyCode")] VirtualKeyCode);

impl VKeyCode {
    pub fn code(&self) -> VirtualKeyCode { self.0 }

    fn serialize<S>(code: &VirtualKeyCode, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(vkcode_to_str(code))
    }

    fn deserialize<'de, D>(deserializer: D) -> Result<VirtualKeyCode, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_str(VKeyCodeVisitor)
    }
}

pub fn vkcode_to_str(code: &VirtualKeyCode) -> &'static str {
    match code {
        VirtualKeyCode::Key1 => "Key1",
        VirtualKeyCode::Key2 => "Key2",
        VirtualKeyCode::Key3 => "Key3",
        VirtualKeyCode::Key4 => "Key4",
        VirtualKeyCode::Key5 => "Key5",
        VirtualKeyCode::Key6 => "Key6",
        VirtualKeyCode::Key7 => "Key7",
        VirtualKeyCode::Key8 => "Key8",
        VirtualKeyCode::Key9 => "Key9",
        VirtualKeyCode::Key0 => "Key0",
        VirtualKeyCode::A => "A",
        VirtualKeyCode::B => "B",
        VirtualKeyCode::C => "C",
        VirtualKeyCode::D => "D",
        VirtualKeyCode::E => "E",
        VirtualKeyCode::F => "F",
        VirtualKeyCode::G => "G",
        VirtualKeyCode::H => "H",
        VirtualKeyCode::I => "I",
        VirtualKeyCode::J => "J",
        VirtualKeyCode::K => "K",
        VirtualKeyCode::L => "L",
        VirtualKeyCode::M => "M",
        VirtualKeyCode::N => "N",
        VirtualKeyCode::O => "O",
        VirtualKeyCode::P => "P",
        VirtualKeyCode::Q => "Q",
        VirtualKeyCode::R => "R",
        VirtualKeyCode::S => "S",
        VirtualKeyCode::T => "T",
        VirtualKeyCode::U => "U",
        VirtualKeyCode::V => "V",
        VirtualKeyCode::W => "W",
        VirtualKeyCode::X => "X",
        VirtualKeyCode::Y => "Y",
        VirtualKeyCode::Z => "Z",
        VirtualKeyCode::Escape => "Escape",
        VirtualKeyCode::F1 => "F1",
        VirtualKeyCode::F2 => "F2",
        VirtualKeyCode::F3 => "F3",
        VirtualKeyCode::F4 => "F4",
        VirtualKeyCode::F5 => "F5",
        VirtualKeyCode::F6 => "F6",
        VirtualKeyCode::F7 => "F7",
        VirtualKeyCode::F8 => "F8",
        VirtualKeyCode::F9 => "F9",
        VirtualKeyCode::F10 => "F10",
        VirtualKeyCode::F11 => "F11",
        VirtualKeyCode::F12 => "F12",
        VirtualKeyCode::Snapshot => "Snapshot",
        VirtualKeyCode::Pause => "Pause",
        VirtualKeyCode::Insert => "Insert",
        VirtualKeyCode::Home => "Home",
        VirtualKeyCode::Delete => "Delete",
        VirtualKeyCode::End => "End",
        VirtualKeyCode::PageDown => "PageDown",
        VirtualKeyCode::PageUp => "PageUp",
        VirtualKeyCode::Left => "Left",
        VirtualKeyCode::Up => "Up",
        VirtualKeyCode::Right => "Right",
        VirtualKeyCode::Down => "Down",
        VirtualKeyCode::Back => "Back",
        VirtualKeyCode::Return => "Return",
        VirtualKeyCode::Space => "Space",
        VirtualKeyCode::LControl => "LControl",
        VirtualKeyCode::LShift => "LShift",
        _ => "",
    }
}

pub fn str_to_vkcode(s: &str) -> Option<VirtualKeyCode> {
    match s {
        "Key1" => Some(VirtualKeyCode::Key1),
        "Key2" => Some(VirtualKeyCode::Key2),
        "Key3" => Some(VirtualKeyCode::Key3),
        "Key4" => Some(VirtualKeyCode::Key4),
        "Key5" => Some(VirtualKeyCode::Key5),
        "Key6" => Some(VirtualKeyCode::Key6),
        "Key7" => Some(VirtualKeyCode::Key7),
        "Key8" => Some(VirtualKeyCode::Key8),
        "Key9" => Some(VirtualKeyCode::Key0),
        "Key0" => Some(VirtualKeyCode::Key0),
        "A" => Some(VirtualKeyCode::A),
        "B" => Some(VirtualKeyCode::B),
        "C" => Some(VirtualKeyCode::C),
        "D" => Some(VirtualKeyCode::D),
        "E" => Some(VirtualKeyCode::E),
        "F" => Some(VirtualKeyCode::F),
        "G" => Some(VirtualKeyCode::G),
        "H" => Some(VirtualKeyCode::H),
        "I" => Some(VirtualKeyCode::I),
        "J" => Some(VirtualKeyCode::J),
        "K" => Some(VirtualKeyCode::K),
        "L" => Some(VirtualKeyCode::L),
        "M" => Some(VirtualKeyCode::M),
        "N" => Some(VirtualKeyCode::N),
        "O" => Some(VirtualKeyCode::O),
        "P" => Some(VirtualKeyCode::P),
        "Q" => Some(VirtualKeyCode::Q),
        "R" => Some(VirtualKeyCode::R),
        "S" => Some(VirtualKeyCode::S),
        "T" => Some(VirtualKeyCode::T),
        "U" => Some(VirtualKeyCode::U),
        "V" => Some(VirtualKeyCode::V),
        "W" => Some(VirtualKeyCode::W),
        "X" => Some(VirtualKeyCode::X),
        "Y" => Some(VirtualKeyCode::Y),
        "Z" => Some(VirtualKeyCode::Z),
        "Escape" => Some(VirtualKeyCode::Escape),
        "Return" => Some(VirtualKeyCode::Return),
        "Space" => Some(VirtualKeyCode::Space),
        "LControl" => Some(VirtualKeyCode::LControl),
        "LShift" => Some(VirtualKeyCode::LShift),
        _ => None,
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
    pub back: Option<VKeyCode>,
    pub forward: Option<VKeyCode>,
    pub left: Option<VKeyCode>,
    pub right: Option<VKeyCode>,
    pub dodge: Option<VKeyCode>,
    pub crouch: Option<VKeyCode>,
    pub jump: Option<VKeyCode>,

    // Actions
    pub attack_1: Option<VKeyCode>,
    pub attack_2: Option<VKeyCode>,
    pub interact: Option<VKeyCode>,
    pub mount: Option<VKeyCode>,
    pub skill_1: Option<VKeyCode>,
    pub skill_2: Option<VKeyCode>,
    pub skill_3: Option<VKeyCode>,
    pub use_item: Option<VKeyCode>,

    // Menus
    pub chat: Option<VKeyCode>,
    pub inventory: Option<VKeyCode>,
    pub pause: Option<VKeyCode>,
}

#[derive(Serialize, Deserialize, PartialEq)]
pub struct Mount {
    pub dismount: Option<VKeyCode>,
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
                    attack_1: None,
                    attack_2: None,
                    interact: Some(general.interact.unwrap_or(default_keys.general.interact.unwrap())),
                    skill_1: None,
                    skill_2: None,
                    skill_3: None,
                    use_item: None,
                    mount: Some(general.mount.unwrap_or(default_keys.general.mount.unwrap())),
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
                back: Some(VKeyCode(VirtualKeyCode::S)),
                forward: Some(VKeyCode(VirtualKeyCode::W)),
                left: Some(VKeyCode(VirtualKeyCode::A)),
                right: Some(VKeyCode(VirtualKeyCode::D)),
                dodge: Some(VKeyCode(VirtualKeyCode::LShift)),
                crouch: Some(VKeyCode(VirtualKeyCode::LControl)),
                jump: Some(VKeyCode(VirtualKeyCode::Space)),

                attack_1: None,
                attack_2: None,
                interact: None,
                mount: Some(VKeyCode(VirtualKeyCode::M)),
                skill_1: None,
                skill_2: None,
                skill_3: None,
                use_item: Some(VKeyCode(VirtualKeyCode::Q)),

                chat: Some(VKeyCode(VirtualKeyCode::Return)),
                inventory: Some(VKeyCode(VirtualKeyCode::I)),
                pause: Some(VKeyCode(VirtualKeyCode::Escape)),
            },

            mount: Mount {
                dismount: Some(VKeyCode(VirtualKeyCode::M)),
            },
        }
    }
}
