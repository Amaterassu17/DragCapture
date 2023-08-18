use std::collections::{BTreeMap, HashMap};
use std::fs;
use std::fs::File;
use std::io::{Read, Write};
use eframe::egui;
use egui::Key;
use global_hotkey::{GlobalHotKeyManager, hotkey};
use global_hotkey::hotkey::{Code, HotKey};


pub(crate) static HOTKEY_FILE: &str = "./config/hotkeys";

static HOTKEY_INITIAL: &str = "0;ALT + KeyF
1;ALT + Escape
2;ALT + KeyT
3;ALT + KeyC
4;ALT + KeyS
5;ALT + KeyZ
6;ALT + KeyR";

pub(crate) struct HotkeySettings {
    pub hotkey_map: HashMap<u32, ((Option<hotkey::Modifiers>, Code), HotkeyAction)>,
    pub hotkey_created: bool,
    pub hotkey_manager: GlobalHotKeyManager,
}

impl HotkeySettings {
    pub fn new () -> Self {

        HotkeySettings {
            hotkey_map: HashMap::new(),
            hotkey_created: false,
            hotkey_manager: GlobalHotKeyManager::new().unwrap()
        }
    }


    pub fn load_hotkey_map(&mut self) -> (Vec<String>, HashMap<u32, ((Option<hotkey::Modifiers>, Code), HotkeyAction)>) {
        let mut hotkeys_strings: Vec<String> = Vec::new();
        let mut buf = String::new();

        let file = File::open(HOTKEY_FILE);

        match file {
            Ok(mut f) => {f.read_to_string(&mut buf).unwrap();}
            Err(_) => {
                match fs::create_dir("./config") {
                    Ok(_) => {
                        match File::options().create(true).write(true).open(HOTKEY_FILE){
                            Ok(mut f) => {buf = HOTKEY_INITIAL.to_string();
                                f.write_all(buf.as_bytes()).unwrap();}
                            Err(err) => {panic!("{:?}",err)}
                        };
                    }
                    Err(_) => {panic!("Couldn't create config folder")}
                }





                // let mut f = File::open(HOTKEY_FILE).unwrap();


            }
        }

        let mut hotkey_map: HashMap<u32, ((Option<hotkey::Modifiers>, Code), HotkeyAction)> = HashMap::new();

        buf.split("\n").for_each(|x| {
            let mut line = x.split(";");

            //split in 2 parts: action in u32 and hotkey
            let id = line.next().unwrap().parse::<i32>().unwrap();

            let action = HotkeyAction::new_from_i32(id).unwrap();
            let hotkey = line.next().unwrap();
            hotkeys_strings.push(hotkey.to_string());
            let mut split = hotkey.split(" + ");
            //If there is only a key -> (None, Code), else -> (Some(Modifiers), Code)
            let value = match split.clone().count() {
                0 => panic!("Empty hotkey"),
                1 => (None, StringCodeWrap::string_to_code(split.next().unwrap().to_string())),
                2 => {
                    let mut strings: Vec<String> = Vec::new();
                    split.clone().for_each(|x| strings.push(x.to_string()));
                    (Some(StringCodeWrap::string_to_modifiers(strings[0].clone())), StringCodeWrap::string_to_code(strings[1].clone()))
                }
                _ => panic!("Too many keys")
            };

            let new_hotkey = HotKey::new(value.0, value.1);
            hotkey_map.insert(new_hotkey.id(), (value, action));
            self.hotkey_manager.register(new_hotkey).unwrap();
        });
        self.hotkey_created = true;
        (hotkeys_strings, hotkey_map)
    }

    pub fn update_hotkey_map(&mut self, new_codes_string: Vec<String>, old_codes_string: Vec<String>) -> () {
        let codes: ((Option<hotkey::Modifiers>, Code), HotkeyAction);
        let old_hotkey: HotKey;
        let old_action: HotkeyAction;
        match old_codes_string.len() {
            0 => panic!("Empty hotkey"),
            1 => {
                old_hotkey = HotKey::new(None, StringCodeWrap::string_to_code(old_codes_string[0].to_string()));
                old_action = self.hotkey_map.remove(&old_hotkey.id()).unwrap().1.clone();
                self.hotkey_manager.unregister(old_hotkey).unwrap();
            }
            2 => {
                old_hotkey = HotKey::new(Some(StringCodeWrap::string_to_modifiers(old_codes_string[0].to_string())), StringCodeWrap::string_to_code(old_codes_string[1].to_string()));
                old_action = self.hotkey_map.remove(&old_hotkey.id()).unwrap().1.clone();
                self.hotkey_manager.unregister(old_hotkey).unwrap()
            }
            _ => panic!("Too Many Keys")
        }

        fs::remove_file(HOTKEY_FILE).unwrap();

        let mut f = fs::OpenOptions::new()
            .write(true) // <--------- this
            .create(true)
            .truncate(true)
            .open(HOTKEY_FILE)
            .unwrap();

        let new_hotkey: HotKey;
        let mut string = String::new();

        let mut temporary_map: BTreeMap<i32, String> = BTreeMap::new();

        match new_codes_string.len() {
            0 => panic!("Empty hotkey"),
            1 => {
                codes = ((None, StringCodeWrap::string_to_code(new_codes_string[0].to_string())), old_action);
                new_hotkey = HotKey::new(codes.0.0, codes.0.1);
                self.hotkey_map.insert(new_hotkey.id(), codes.clone());
                self.hotkey_manager.register(HotKey::new(codes.0.0, codes.0.1)).unwrap();
                for (_, value) in self.hotkey_map.iter() {
                    let mut temp = String::new();
                    match value.0.0 {
                        Some(x) => temp.push_str(&format!("{:?} + ", x)),
                        None => {}
                    }
                    // string.push_str(&format!("{}", HotkeyAction::i32_from_action(value.1.clone()).to_string() + ";"));

                    temp.push_str(&format!("{:?}", value.0.1));
                    temporary_map.insert(HotkeyAction::i32_from_action(value.1.clone()), temp);
                    // if index != self.hotkey_map.len() {
                    //     // string.push_str(&format!("\n"));
                    // }
                    // index += 1;
                }
            }
            2 => {
                codes = ((Some(StringCodeWrap::string_to_modifiers(new_codes_string[0].clone())), StringCodeWrap::string_to_code(new_codes_string[1].clone())), old_action);
                new_hotkey = HotKey::new(codes.0.0, codes.0.1);

                self.hotkey_map.insert(new_hotkey.id(), codes.clone());
                self.hotkey_manager.register(HotKey::new(codes.0.0, codes.0.1)).unwrap();
                for (_, value) in self.hotkey_map.iter() {
                    let mut temp = String::new();

                    // string.push_str(&format!("{}", HotkeyAction::i32_from_action(value.1.clone()).to_string() + ";"));
                    match value.0.0 {
                        Some(x) => temp.push_str(&format!("{:?} + ", x)),
                        None => {}
                    }
                    temp.push_str(&format!("{:?}", value.0.1));
                    temporary_map.insert(HotkeyAction::i32_from_action(value.1.clone()), temp);
                }
            }
            _ => panic!("Too many keys")
        }


        for (key, value) in temporary_map.iter() {
            //write key;value and then \n, except for the last one
            if *key == (temporary_map.len() - 1) as i32 {
                string.push_str(&format!("{};{}", key, value));
            } else {
                string.push_str(&format!("{};{}\n", key, value));
            }
        }

        f.write_all(string.as_bytes()).unwrap();
    }


}





#[derive(Clone)]
pub(crate) enum HotkeyAction {
    TakeScreenshot = 0,
    Quit = 1,
    SwitchDelay = 2,
    CopyClipboard = 3,
    FastSave = 4,
    Undo = 5,
    ResetImage = 6,
}



impl HotkeyAction {
    pub fn new_from_i32(value: i32) -> Option<HotkeyAction> {
        match value {
            0 => Some(HotkeyAction::TakeScreenshot),
            1 => Some(HotkeyAction::Quit),
            2 => Some(HotkeyAction::SwitchDelay),
            3 => Some(HotkeyAction::CopyClipboard),
            4 => Some(HotkeyAction::FastSave),
            5 => Some(HotkeyAction::Undo),
            6 => Some(HotkeyAction::ResetImage),
            _ => None
        }
    }

    pub fn i32_from_action(value: HotkeyAction) -> i32 {
        match value {
            HotkeyAction::TakeScreenshot => { 0 }
            HotkeyAction::Quit => { 1 }
            HotkeyAction::SwitchDelay => { 2 }
            HotkeyAction::CopyClipboard => { 3 }
            HotkeyAction::FastSave => { 4 }
            HotkeyAction::Undo => { 5 }
            HotkeyAction::ResetImage => { 6 }
        }
    }
}

pub(crate) struct EguiKeyWrap {
    key: Key,
}

impl EguiKeyWrap {
    pub fn new(key: Key) -> EguiKeyWrap {
        EguiKeyWrap { key }
    }
}

impl From<EguiKeyWrap> for Code {
    fn from(value: EguiKeyWrap) -> Self {
        match value.key {
            Key::ArrowDown => { Code::ArrowDown }
            Key::ArrowLeft => { Code::ArrowLeft }
            Key::ArrowRight => { Code::ArrowRight }
            Key::ArrowUp => { Code::ArrowUp }
            Key::Escape => { Code::Escape }
            Key::Tab => { Code::Tab }
            Key::Backspace => { Code::Backspace }
            Key::Enter => { Code::Enter }
            Key::Space => { Code::Space }
            Key::Insert => { Code::Insert }
            Key::Delete => { Code::Delete }
            Key::Home => { Code::Home }
            Key::End => { Code::End }
            Key::PageUp => { Code::PageUp }
            Key::PageDown => { Code::PageDown }
            Key::Minus => { Code::Minus }
            Key::PlusEquals => { Code::Equal }
            Key::Num0 => { Code::Digit0 }
            Key::Num1 => { Code::Digit1 }
            Key::Num2 => { Code::Digit2 }
            Key::Num3 => { Code::Digit3 }
            Key::Num4 => { Code::Digit4 }
            Key::Num5 => { Code::Digit5 }
            Key::Num6 => { Code::Digit6 }
            Key::Num7 => { Code::Digit7 }
            Key::Num8 => { Code::Digit8 }
            Key::Num9 => { Code::Digit9 }
            Key::A => { Code::KeyA }
            Key::B => { Code::KeyB }
            Key::C => { Code::KeyC }
            Key::D => { Code::KeyD }
            Key::E => { Code::KeyE }
            Key::F => { Code::KeyF }
            Key::G => { Code::KeyG }
            Key::H => { Code::KeyH }
            Key::I => { Code::KeyI }
            Key::J => { Code::KeyJ }
            Key::K => { Code::KeyK }
            Key::L => { Code::KeyL }
            Key::M => { Code::KeyM }
            Key::N => { Code::KeyN }
            Key::O => { Code::KeyO }
            Key::P => { Code::KeyP }
            Key::Q => { Code::KeyQ }
            Key::R => { Code::KeyR }
            Key::S => { Code::KeyS }
            Key::T => { Code::KeyT }
            Key::U => { Code::KeyU }
            Key::V => { Code::KeyV }
            Key::W => { Code::KeyW }
            Key::X => { Code::KeyX }
            Key::Y => { Code::KeyY }
            Key::Z => { Code::KeyZ }
            Key::F1 => { Code::F1 }
            Key::F2 => { Code::F2 }
            Key::F3 => { Code::F3 }
            Key::F4 => { Code::F4 }
            Key::F5 => { Code::F5 }
            Key::F6 => { Code::F6 }
            Key::F7 => { Code::F7 }
            Key::F8 => { Code::F8 }
            Key::F9 => { Code::F9 }
            Key::F10 => { Code::F10 }
            Key::F11 => { Code::F11 }
            Key::F12 => { Code::F12 }
            Key::F13 => { Code::F13 }
            Key::F14 => { Code::F14 }
            Key::F15 => { Code::F15 }
            Key::F16 => { Code::F16 }
            Key::F17 => { Code::F17 }
            Key::F18 => { Code::F18 }
            Key::F19 => { Code::F19 }
            Key::F20 => { Code::F20 }
        }
    }
}

impl Into<String> for EguiKeyWrap {
    fn into(self) -> String {
        match self.key {
            Key::ArrowDown => { "ArrowDown".to_string() }
            Key::ArrowLeft => { "ArrowLeft".to_string() }
            Key::ArrowRight => { "ArrowRight".to_string() }
            Key::ArrowUp => { "ArrowUp".to_string() }
            Key::Escape => { "Escape".to_string() }
            Key::Tab => { "Tab".to_string() }
            Key::Backspace => { "Backspace".to_string() }
            Key::Enter => { "Enter".to_string() }
            Key::Space => { "Space".to_string() }
            Key::Insert => { "Insert".to_string() }
            Key::Delete => { "Insert".to_string() }
            Key::Home => { "Home".to_string() }
            Key::End => { "End".to_string() }
            Key::PageUp => { "PageUp".to_string() }
            Key::PageDown => { "PageDown".to_string() }
            Key::Minus => { "Minus".to_string() }
            Key::PlusEquals => { "PlusEquals".to_string() }
            Key::Num0 => { "Num0".to_string() }
            Key::Num1 => { "Num1".to_string() }
            Key::Num2 => { "Num2".to_string() }
            Key::Num3 => { "Num3".to_string() }
            Key::Num4 => { "Num4".to_string() }
            Key::Num5 => { "Num5".to_string() }
            Key::Num6 => { "Num6".to_string() }
            Key::Num7 => { "Num7".to_string() }
            Key::Num8 => { "Num8".to_string() }
            Key::Num9 => { "Num9".to_string() }
            Key::A => { "KeyA".to_string() }
            Key::B => { "KeyB".to_string() }
            Key::C => { "KeyC".to_string() }
            Key::D => { "KeyD".to_string() }
            Key::E => { "KeyE".to_string() }
            Key::F => { "KeyF".to_string() }
            Key::G => { "KeyG".to_string() }
            Key::H => { "KeyH".to_string() }
            Key::I => { "KeyI".to_string() }
            Key::J => { "KeyJ".to_string() }
            Key::K => { "KeyK".to_string() }
            Key::L => { "KeyL".to_string() }
            Key::M => { "KeyM".to_string() }
            Key::N => { "KeyN".to_string() }
            Key::O => { "KeyO".to_string() }
            Key::P => { "KeyP".to_string() }
            Key::Q => { "KeyQ".to_string() }
            Key::R => { "KeyR".to_string() }
            Key::S => { "KeyS".to_string() }
            Key::T => { "KeyT".to_string() }
            Key::U => { "KeyU".to_string() }
            Key::V => { "KeyV".to_string() }
            Key::W => { "KeyW".to_string() }
            Key::X => { "KeyX".to_string() }
            Key::Y => { "KeyY".to_string() }
            Key::Z => { "KeyZ".to_string() }
            Key::F1 => { "F1".to_string() }
            Key::F2 => { "F2".to_string() }
            Key::F3 => { "F3".to_string() }
            Key::F4 => { "F4".to_string() }
            Key::F5 => { "F5".to_string() }
            Key::F6 => { "F6".to_string() }
            Key::F7 => { "F7".to_string() }
            Key::F8 => { "F8".to_string() }
            Key::F9 => { "F9".to_string() }
            Key::F10 => { "F10".to_string() }
            Key::F11 => { "F11".to_string() }
            Key::F12 => { "F12".to_string() }
            Key::F13 => { "F13".to_string() }
            Key::F14 => { "F14".to_string() }
            Key::F15 => { "F15".to_string() }
            Key::F16 => { "F16".to_string() }
            Key::F17 => { "F17".to_string() }
            Key::F18 => { "F18".to_string() }
            Key::F19 => { "F19".to_string() }
            Key::F20 => { "F20".to_string() }
        }
    }
}



pub(crate) struct StringCodeWrap {
    code: String,
}

impl StringCodeWrap {
    pub fn new(code: String) -> Self {
        Self {
            code
        }
    }

    pub fn string_to_code(s: String) -> Code {
        let wrap = StringCodeWrap::new(s.trim().to_string());
        wrap.into()
    }

    pub fn string_to_modifiers(s: String) -> hotkey::Modifiers {
        match s.as_str() {
            "ALT" => hotkey::Modifiers::ALT,
            "CONTROL" => hotkey::Modifiers::CONTROL,
            "SHIFT" => hotkey::Modifiers::SHIFT,
            "ALT_GRAPH" => hotkey::Modifiers::ALT_GRAPH,
            "CAPS_LOCK" => hotkey::Modifiers::CAPS_LOCK,
            "FN" => hotkey::Modifiers::FN,
            "SYMBOL" => hotkey::Modifiers::SYMBOL,
            "HYPER" => hotkey::Modifiers::HYPER,
            "META" => hotkey::Modifiers::META,
            "NUM_LOCK" => hotkey::Modifiers::NUM_LOCK,
            "SCROLL_LOCK" => hotkey::Modifiers::SCROLL_LOCK,
            "SUPER" => hotkey::Modifiers::SUPER,
            "SYMBOL_LOCK" => hotkey::Modifiers::SYMBOL_LOCK,
            _ => { hotkey::Modifiers::default() }
        }
    }

}

impl Into<Code> for StringCodeWrap {
    fn into(self) -> Code {
        //HERE WE MAP EVERY POSSIBLE CODE STRING INTO A CODE
        match self.code.as_str() {
            "ArrowDown" => Code::ArrowDown,
            "ArrowLeft" => Code::ArrowLeft,
            "ArrowRight" => Code::ArrowRight,
            "ArrowUp" => Code::ArrowUp,
            "Escape" => Code::Escape,
            "Tab" => Code::Tab,
            "Backquote" => Code::Backquote,
            "Backslash" => Code::Backslash,
            "AltLeft" => Code::AltLeft,
            "AltRight" => Code::AltRight,
            "CapsLock" => Code::CapsLock,
            "ControlLeft" => Code::ControlLeft,
            "ControlRight" => Code::ControlRight,
            "ShiftLeft" => Code::ShiftLeft,
            "ShiftRight" => Code::ShiftRight,
            "BracketLeft" => Code::BracketLeft,
            "BracketRight" => Code::BracketRight,
            "MetaLeft" => Code::MetaLeft,
            "MetaRight" => Code::MetaRight,
            "Semicolon" => Code::Semicolon,
            "Quote" => Code::Quote,
            "IntlBackslash" => Code::IntlBackslash,
            "IntlRo" => Code::IntlRo,
            "IntlYen" => Code::IntlYen,
            "ContextMenu" => Code::ContextMenu,
            "Comma" => Code::Comma,
            "Period" => Code::Period,
            "Slash" => Code::Slash,
            "Digit0" => Code::Digit0,
            "Digit1" => Code::Digit1,
            "Digit2" => Code::Digit2,
            "Digit3" => Code::Digit3,
            "Digit4" => Code::Digit4,
            "Digit5" => Code::Digit5,
            "Digit6" => Code::Digit6,
            "Digit7" => Code::Digit7,
            "Digit8" => Code::Digit8,
            "Digit9" => Code::Digit9,
            "Backspace" => Code::Backspace,
            "Enter" => Code::Enter,
            "Space" => Code::Space,
            "Insert" => Code::Insert,
            "Delete" => Code::Delete,
            "Home" => Code::Home,
            "End" => Code::End,
            "PageUp" => Code::PageUp,
            "PageDown" => Code::PageDown,
            "Minus" => Code::Minus,
            "PlusEquals" => Code::Equal,
            "Num0" => Code::Numpad0,
            "Num1" => Code::Numpad1,
            "Num2" => Code::Numpad2,
            "Num3" => Code::Numpad3,
            "Num4" => Code::Numpad4,
            "Num5" => Code::Numpad5,
            "Num6" => Code::Numpad6,
            "Num7" => Code::Numpad7,
            "Num8" => Code::Numpad8,
            "Num9" => Code::Numpad9,
            "KeyA" => Code::KeyA,
            "KeyB" => Code::KeyB,
            "KeyC" => Code::KeyC,
            "KeyD" => Code::KeyD,
            "KeyE" => Code::KeyE,
            "KeyF" => Code::KeyF,
            "KeyG" => Code::KeyG,
            "KeyH" => Code::KeyH,
            "KeyI" => Code::KeyI,
            "KeyJ" => Code::KeyJ,
            "KeyK" => Code::KeyK,
            "KeyL" => Code::KeyL,
            "KeyM" => Code::KeyM,
            "KeyN" => Code::KeyN,
            "KeyO" => Code::KeyO,
            "KeyP" => Code::KeyP,
            "KeyQ" => Code::KeyQ,
            "KeyR" => Code::KeyR,
            "KeyS" => Code::KeyS,
            "KeyT" => Code::KeyT,
            "KeyU" => Code::KeyU,
            "KeyV" => Code::KeyV,
            "KeyW" => Code::KeyW,
            "KeyX" => Code::KeyX,
            "KeyY" => Code::KeyY,
            "KeyZ" => Code::KeyZ,
            "F1" => Code::F1,
            "F2" => Code::F2,
            "F3" => Code::F3,
            "F4" => Code::F4,
            "F5" => Code::F5,
            "F6" => Code::F6,
            "F7" => Code::F7,
            "F8" => Code::F8,
            "F9" => Code::F9,
            "F10" => Code::F10,
            "F11" => Code::F11,
            "F12" => Code::F12,
            "F13" => Code::F13,
            "F14" => Code::F14,
            "F15" => Code::F15,
            "F16" => Code::F16,
            "F17" => Code::F17,
            "F18" => Code::F18,
            "F19" => Code::F19,
            "F20" => Code::F20,
            "Fn" => Code::Fn,
            _ => Code::Enter,
        }
    }
}

impl From<Code> for StringCodeWrap {
    fn from(value: Code) -> Self {
        match value {
            Code::ArrowDown => StringCodeWrap::new("ArrowDown".to_string()),
            Code::ArrowLeft => StringCodeWrap::new("ArrowLeft".to_string()),
            Code::ArrowRight => StringCodeWrap::new("ArrowRight".to_string()),
            Code::ArrowUp => StringCodeWrap::new("ArrowUp".to_string()),
            Code::Escape => StringCodeWrap::new("Escape".to_string()),
            Code::Tab => StringCodeWrap::new("Tab".to_string()),
            Code::Backquote => StringCodeWrap::new("Backquote".to_string()),
            Code::Backslash => StringCodeWrap::new("Backslash".to_string()),
            Code::AltLeft => StringCodeWrap::new("AltLeft".to_string()),
            Code::AltRight => StringCodeWrap::new("AltRight".to_string()),
            Code::CapsLock => StringCodeWrap::new("CapsLock".to_string()),
            Code::ControlLeft => StringCodeWrap::new("ControlLeft".to_string()),
            Code::ControlRight => StringCodeWrap::new("ControlRight".to_string()),
            Code::ShiftLeft => StringCodeWrap::new("ShiftLeft".to_string()),
            Code::ShiftRight => StringCodeWrap::new("ShiftRight".to_string()),
            Code::BracketLeft => StringCodeWrap::new("BracketLeft".to_string()),
            Code::BracketRight => StringCodeWrap::new("BracketRight".to_string()),
            Code::MetaLeft => StringCodeWrap::new("MetaLeft".to_string()),
            Code::MetaRight => StringCodeWrap::new("MetaRight".to_string()),
            Code::Semicolon => StringCodeWrap::new("Semicolon".to_string()),
            Code::Quote => StringCodeWrap::new("Quote".to_string()),
            Code::IntlBackslash => StringCodeWrap::new("IntlBackslash".to_string()),
            Code::IntlRo => StringCodeWrap::new("IntlRo".to_string()),
            Code::IntlYen => StringCodeWrap::new("IntlYen".to_string()),
            Code::ContextMenu => StringCodeWrap::new("ContextMenu".to_string()),
            Code::Comma => StringCodeWrap::new("Comma".to_string()),
            Code::Period => StringCodeWrap::new("Period".to_string()),
            Code::Slash => StringCodeWrap::new("Slash".to_string()),
            Code::Digit0 => StringCodeWrap::new("Digit0".to_string()),
            Code::Digit1 => StringCodeWrap::new("Digit1".to_string()),
            Code::Digit2 => StringCodeWrap::new("Digit2".to_string()),
            Code::Digit3 => StringCodeWrap::new("Digit3".to_string()),
            Code::Digit4 => StringCodeWrap::new("Digit4".to_string()),
            Code::Digit5 => StringCodeWrap::new("Digit5".to_string()),
            Code::Digit6 => StringCodeWrap::new("Digit6".to_string()),
            Code::Digit7 => StringCodeWrap::new("Digit7".to_string()),
            Code::Digit8 => StringCodeWrap::new("Digit8".to_string()),
            Code::Digit9 => StringCodeWrap::new("Digit9".to_string()),
            Code::Backspace => StringCodeWrap::new("Backspace".to_string()),
            Code::Enter => StringCodeWrap::new("Enter".to_string()),
            Code::Space => StringCodeWrap::new("Space".to_string()),
            Code::Insert => StringCodeWrap::new("Insert".to_string()),
            Code::Delete => StringCodeWrap::new("Delete".to_string()),
            Code::Home => StringCodeWrap::new("Home".to_string()),
            Code::End => StringCodeWrap::new("End".to_string()),
            Code::PageUp => StringCodeWrap::new("PageUp".to_string()),
            Code::PageDown => StringCodeWrap::new("PageDown".to_string()),
            Code::Minus => StringCodeWrap::new("Minus".to_string()),
            Code::Equal => StringCodeWrap::new("PlusEquals".to_string()),
            Code::Numpad0 => StringCodeWrap::new("Num0".to_string()),
            Code::Numpad1 => StringCodeWrap::new("Num1".to_string()),
            Code::Numpad2 => StringCodeWrap::new("Num2".to_string()),
            Code::Numpad3 => StringCodeWrap::new("Num3".to_string()),
            Code::Numpad4 => StringCodeWrap::new("Num4".to_string()),
            Code::Numpad5 => StringCodeWrap::new("Num5".to_string()),
            Code::Numpad6 => StringCodeWrap::new("Num6".to_string()),
            Code::Numpad7 => StringCodeWrap::new("Num7".to_string()),
            Code::Numpad8 => StringCodeWrap::new("Num8".to_string()),
            Code::Numpad9 => StringCodeWrap::new("Num9".to_string()),
            Code::KeyA => StringCodeWrap::new("A".to_string()),
            Code::KeyB => StringCodeWrap::new("B".to_string()),
            Code::KeyC => StringCodeWrap::new("C".to_string()),
            Code::KeyD => StringCodeWrap::new("D".to_string()),
            Code::KeyE => StringCodeWrap::new("E".to_string()),
            Code::KeyF => StringCodeWrap::new("F".to_string()),
            Code::KeyG => StringCodeWrap::new("G".to_string()),
            Code::KeyH => StringCodeWrap::new("H".to_string()),
            Code::KeyI => StringCodeWrap::new("I".to_string()),
            Code::KeyJ => StringCodeWrap::new("J".to_string()),
            Code::KeyK => StringCodeWrap::new("K".to_string()),
            Code::KeyL => StringCodeWrap::new("L".to_string()),
            Code::KeyM => StringCodeWrap::new("M".to_string()),
            Code::KeyN => StringCodeWrap::new("N".to_string()),
            Code::KeyO => StringCodeWrap::new("O".to_string()),
            Code::KeyP => StringCodeWrap::new("P".to_string()),
            Code::KeyQ => StringCodeWrap::new("Q".to_string()),
            Code::KeyR => StringCodeWrap::new("R".to_string()),
            Code::KeyS => StringCodeWrap::new("S".to_string()),
            Code::KeyT => StringCodeWrap::new("T".to_string()),
            Code::KeyU => StringCodeWrap::new("U".to_string()),
            Code::KeyV => StringCodeWrap::new("V".to_string()),
            Code::KeyW => StringCodeWrap::new("W".to_string()),
            Code::KeyX => StringCodeWrap::new("X".to_string()),
            Code::KeyY => StringCodeWrap::new("Y".to_string()),
            Code::KeyZ => StringCodeWrap::new("Z".to_string()),
            Code::F1 => StringCodeWrap::new("F1".to_string()),
            Code::F2 => StringCodeWrap::new("F2".to_string()),
            Code::F3 => StringCodeWrap::new("F3".to_string()),
            Code::F4 => StringCodeWrap::new("F4".to_string()),
            Code::F5 => StringCodeWrap::new("F5".to_string()),
            Code::F6 => StringCodeWrap::new("F6".to_string()),
            Code::F7 => StringCodeWrap::new("F7".to_string()),
            Code::F8 => StringCodeWrap::new("F8".to_string()),
            Code::F9 => StringCodeWrap::new("F9".to_string()),
            Code::F10 => StringCodeWrap::new("F10".to_string()),
            Code::F11 => StringCodeWrap::new("F11".to_string()),
            Code::F12 => StringCodeWrap::new("F12".to_string()),
            Code::F13 => StringCodeWrap::new("F13".to_string()),
            Code::F14 => StringCodeWrap::new("F14".to_string()),
            Code::F15 => StringCodeWrap::new("F15".to_string()),
            Code::F16 => StringCodeWrap::new("F16".to_string()),
            Code::F17 => StringCodeWrap::new("F17".to_string()),
            Code::F18 => StringCodeWrap::new("F18".to_string()),
            Code::F19 => StringCodeWrap::new("F19".to_string()),
            Code::F20 => StringCodeWrap::new("F20".to_string()),
            Code::Fn => StringCodeWrap::new("Fn".to_string()),
            _ => StringCodeWrap::new("Enter".to_string()),
        }
    }
}


