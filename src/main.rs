use std::borrow::Cow;
use std::time::Duration;
use eframe::{App, Frame, run_native, Storage, egui::CentralPanel, CreationContext};

use egui::{Context, Image, Rect, Visuals, Window, TextureHandle, TextureOptions, InputState};
use eframe::egui::{self, Direction, Modifiers, Pos2, Vec2};
use egui::widgets::color_picker;
use imageproc::point::Point;
use screenshots::{Screen, Compression};
use screenshots;
use std::{fs, cmp, time};
use std::cell::RefCell;
use image::*;
use arboard::*;
use egui::Align::Center;
use epaint::ColorImage;
use image::ImageError::IoError;
use std::error::Error;
use std::path::Path;
use dirs;
use chrono;
use eframe::egui::{Align, Key, Layout};
use rusttype::*;

use global_hotkey::hotkey;
use global_hotkey::GlobalHotKeyEvent;
use global_hotkey::{
    hotkey::{Code, HotKey},
    GlobalHotKeyManager,
};
use std::collections::HashMap;
use std::fs::File;
use std::io::{Read, Write};
use std::rc::Rc;
use std::str::Split;
use timer::Timer;
use crate::HotkeyAction::TakeScreenshot;

static HOTKEY_FILE: &str = "./config/hotkeys.txt";

enum DrawingType {
    None,
    Arrow,
    Circle,
    Rectangle,
    Line,
}

#[derive(Clone)]
enum HotkeyAction {
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
}


struct EguiKeyWrap {
    key: egui::Key,
}


enum Corner {
    None,
    TopLeft,
    TopRight,
    BottomLeft,
    BottomRight,
}

struct CropRect {
    x0: f32,
    y0: f32,
    x1: f32,
    y1: f32,
}

impl CropRect {
    fn new(x0: f32, y0: f32, x1: f32, y1: f32) -> Self {
        return CropRect { x0: x0, y0: y0, x1: x1, y1: y1 };
    }
}

impl Default for CropRect {
    fn default() -> Self {
        return CropRect { x0: -1.0, y0: -1.0, x1: -1.0, y1: -1.0 };
    }
}


impl From<EguiKeyWrap> for global_hotkey::hotkey::Code {
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

struct StringCodeWrap {
    code: String,
}

impl StringCodeWrap {
    pub fn new(code: String) -> Self {
        Self {
            code
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
            "A" => Code::KeyA,
            "B" => Code::KeyB,
            "C" => Code::KeyC,
            "D" => Code::KeyD,
            "E" => Code::KeyE,
            "F" => Code::KeyF,
            "G" => Code::KeyG,
            "H" => Code::KeyH,
            "I" => Code::KeyI,
            "J" => Code::KeyJ,
            "K" => Code::KeyK,
            "L" => Code::KeyL,
            "M" => Code::KeyM,
            "N" => Code::KeyN,
            "O" => Code::KeyO,
            "P" => Code::KeyP,
            "Q" => Code::KeyQ,
            "R" => Code::KeyR,
            "S" => Code::KeyS,
            "T" => Code::KeyT,
            "U" => Code::KeyU,
            "V" => Code::KeyV,
            "W" => Code::KeyW,
            "X" => Code::KeyX,
            "Y" => Code::KeyY,
            "Z" => Code::KeyZ,
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



struct DragApp {
    button_text1: String,
    delay_timer: u32,
    selected_monitor: u32,
    mode: String,
    image: DynamicImage,
    image_back: DynamicImage,
    current_name: String,
    current_path: String,
    current_format: String,
    current_width: i32,
    current_height: i32,
    save_errors: (bool, bool, bool),
    drawing: bool,
    drawing_type: DrawingType,
    timer: Timer,
    initial_pos: egui::Pos2,
    hotkeys_strings: Vec<String>,
    hotkey_ui_status: bool,
    changing_hotkey: Vec<bool>,
    hotkey_map: HashMap<u32, ((Option<hotkey::Modifiers>, Code), HotkeyAction)>,
    hotkey_created: bool,
    hotkey_manager: GlobalHotKeyManager,
    crop: bool,
    crop_point: CropRect,
    current_crop_point: Corner,
    color: epaint::Color32,
    texting: bool,
    text_string: String,
    all_keys: Vec<Key>,
    remaining_time: u32,
}

fn create_visuals() -> egui::style::Visuals {
    let mut visuals = egui::style::Visuals::default();

    visuals.widgets.noninteractive.bg_fill = egui::Color32::from_black_alpha(220);
    visuals.widgets.noninteractive.fg_stroke = egui::Stroke::new(1.0, egui::Color32::WHITE);
    visuals.widgets.inactive.rounding = epaint::Rounding { nw: 3.0, ne: 3.0, sw: 3.0, se: 3.0 };


    visuals
}

impl DragApp {
    pub fn new(cc: &CreationContext<'_>) -> Self {
        //Qua dobbiamo mettere il setup di eventuali font eccetera

        let visuals = create_visuals();
        cc.egui_ctx.set_visuals(visuals);

        Self {
            button_text1: "Take a screenshot!".to_owned(),
            delay_timer: 0,
            selected_monitor: 0,
            mode: "initial".to_string(),
            image: DynamicImage::default(),
            image_back: DynamicImage::default(),
            current_width: 0,
            current_height: 0,
            current_name: chrono::Local::now().format("%Y_%m_%d_%H_%M_%S").to_string(),
            current_path: dirs::picture_dir().unwrap().to_str().unwrap().to_string(),
            current_format: ".png".to_string(),
            save_errors: (false, false, false),
            drawing: false,
            drawing_type: DrawingType::None,
            initial_pos: egui::pos2(-1.0, -1.0),
            timer: Timer::new(),
            remaining_time : 0,

            crop: false,
            crop_point: CropRect::default(),
            current_crop_point: Corner::None,
            color: epaint::Color32::default(),
            texting: false,
            text_string: "".to_string(),
            all_keys: vec![Key::A, Key::B, Key::C, Key::D, Key::E, Key::F, Key::G, Key::H, Key::I, Key::L, Key::M, Key::N, Key::O, Key::P, Key::Q, Key::R, Key::S, Key::T, Key::U, Key::V, Key::Z, Key::J, Key::K, Key::W, Key::X, Key::Y, Key::Num0, Key::Num1, Key::Num2, Key::Num3, Key::Num4, Key::Num5, Key::Num6, Key::Num7, Key::Num8, Key::Num9, Key::Minus, Key::PlusEquals, Key::Space, Key::Backspace, Key::Enter],

            hotkeys_strings: Vec::new(),
            hotkey_ui_status: false,
            changing_hotkey: vec![false; 7],

            hotkey_map: HashMap::new(),
            hotkey_created: false,
            hotkey_manager: GlobalHotKeyManager::new().unwrap(),
        }
    }

    fn load_hotkey_map(&mut self) -> (Vec<String>, HashMap<u32, ((Option<hotkey::Modifiers>, Code), HotkeyAction)>) {
        let mut hotkeys_strings: Vec<String> = Vec::new();
        let mut buf = String::new();
        File::open(HOTKEY_FILE).unwrap().read_to_string(&mut buf).unwrap();
        let mut index: i32 = 0;
        let mut hotkey_map: HashMap<u32, ((Option<hotkey::Modifiers>, Code), HotkeyAction)> = HashMap::new();

        buf.split("\n").for_each(|x| {
            hotkeys_strings.push(x.to_string());
            let mut split = x.split(" + ");
            //If there is only a key -> (None, Code), else -> (Some(Modifiers), Code)
            let value = match split.clone().count() {
                0 => panic!("Empty hotkey"),
                1 => (None, DragApp::string_to_code(split.next().unwrap().to_string())),
                2 => {
                    let mut strings: Vec<String> = Vec::new();
                    split.clone().for_each(|x| strings.push(x.to_string()));
                    (Some(DragApp::string_to_modifiers(strings[0].clone())), DragApp::string_to_code(strings[1].clone()))
                }
                _ => panic!("Too many keys")
            };

            let new_hotkey = HotKey::new(value.0, value.1);
            println!("Hotkey: {:?}", new_hotkey);
            hotkey_map.insert(new_hotkey.id(), (value, HotkeyAction::new_from_i32(index).unwrap()));
            self.hotkey_manager.register(new_hotkey).unwrap();
            index = index + 1;
        });
        self.hotkey_created = true;
        (hotkeys_strings, hotkey_map)
    }

    fn hotkey_press(&mut self) {
        if self.hotkey_ui_status == false {
            if let Ok(event) = GlobalHotKeyEvent::receiver().try_recv() {
                println!("Hotkey pressed: {:?}", event);
                let value = self.hotkey_map.get(&event.id).unwrap();
                match value.1 {
                    TakeScreenshot => {
                        if self.mode != "saving" { self.take_screenshot(); }
                    }
                    HotkeyAction::Quit => {
                        std::process::exit(0);
                    }
                    HotkeyAction::SwitchDelay => {
                        self.switch_delay_timer();
                    }
                    HotkeyAction::CopyClipboard => {
                        if self.mode == "taken" { self.copy_to_clipboard(); }
                    }
                    HotkeyAction::FastSave => {
                        if self.mode == "taken" { self.save_image_to_disk(self.current_format.clone().as_str(), self.current_path.clone().as_str(), self.current_name.clone().as_str()).unwrap(); }
                    }
                    HotkeyAction::Undo => {
                        if self.mode == "taken" { self.undo_action(); }
                    }
                    HotkeyAction::ResetImage => {
                        if self.mode == "taken" { self.reset_image(); }
                    }
                }
            }
        }
    }

    pub fn undo_action(&mut self) -> () {}

    pub fn reset_image(&mut self) -> () {}

    fn string_to_code(s: String) -> Code {
        let wrap = StringCodeWrap::new(s.trim().to_string());
        wrap.into()
    }

    fn string_to_modifiers(s: String) -> hotkey::Modifiers {
        match s.as_str() {
            "Alt" => hotkey::Modifiers::ALT,
            "Ctrl" => hotkey::Modifiers::CONTROL,
            "Shift" => hotkey::Modifiers::SHIFT,
            "AltGraph" => hotkey::Modifiers::ALT_GRAPH,
            "CapsLock" => hotkey::Modifiers::CAPS_LOCK,
            "Fn" => hotkey::Modifiers::FN,
            "Symbol" => hotkey::Modifiers::SYMBOL,
            "Hyper" => hotkey::Modifiers::HYPER,
            "Meta" => hotkey::Modifiers::META,
            "NumLock" => hotkey::Modifiers::NUM_LOCK,
            "ScrollLock" => hotkey::Modifiers::SCROLL_LOCK,
            "Super" => hotkey::Modifiers::SUPER,
            "SymbolLock" => hotkey::Modifiers::SYMBOL_LOCK,
            _ => { hotkey::Modifiers::default() }
        }
    }

    pub fn update_hotkey_map(&mut self, new_codes_string: Vec<String>, old_codes_string: Vec<String>) -> () {
        let mut codes: ((Option<hotkey::Modifiers>, Code), HotkeyAction);
        let old_hotkey: HotKey;
        let old_action: HotkeyAction;
        match old_codes_string.len() {
            0 => panic!("Empty hotkey"),
            1 => {
                old_hotkey = HotKey::new(None, DragApp::string_to_code(old_codes_string[0].to_string()));
                old_action = self.hotkey_map.get(&old_hotkey.id()).unwrap().1.clone();
                self.hotkey_manager.unregister(old_hotkey).unwrap();
            }
            2 => {
                old_hotkey = HotKey::new(Some(DragApp::string_to_modifiers(old_codes_string[0].to_string())), DragApp::string_to_code(old_codes_string[1].to_string()));
                old_action = self.hotkey_map.get(&old_hotkey.id()).unwrap().1.clone();
                self.hotkey_manager.unregister(old_hotkey).unwrap()
            }
            _ => panic!("Too Many Keys")
        }

        let mut f = fs::OpenOptions::new()
            .read(true)
            .write(true) // <--------- this
            .create(true)
            .open(HOTKEY_FILE)
            .unwrap();

        let mut new_hotkey: HotKey;

        match new_codes_string.len() {
            0 => panic!("Empty hotkey"),
            1 => {
                codes = ((None, DragApp::string_to_code(new_codes_string[0].to_string())), old_action);
                new_hotkey = HotKey::new(codes.0.0, codes.0.1);
                self.hotkey_map.insert(new_hotkey.id(), codes.clone());
                self.hotkey_manager.register(HotKey::new(codes.0.0, codes.0.1)).unwrap();
                for (key, value) in self.hotkey_map.iter() {
                    let mut string = String::new();
                    match value.0.0 {
                        Some(x) => string.push_str(&format!("{:?} + ", x)),
                        None => {}
                    }
                    string.push_str(&format!("{:?}", value.0.1));
                    f.write_all(string.as_bytes()).unwrap();
                    f.write_all("\n".as_bytes()).unwrap();
                }
            }
            2 => {
                let mut strings: Vec<String> = Vec::new();

                codes = ((Some(DragApp::string_to_modifiers(new_codes_string[0].clone())), DragApp::string_to_code(new_codes_string[1].clone())), old_action);
                new_hotkey = HotKey::new(codes.0.0, codes.0.1);

                self.hotkey_map.insert(new_hotkey.id(), codes.clone());
                self.hotkey_manager.register(HotKey::new(codes.0.0, codes.0.1)).unwrap();
                for (key, value) in self.hotkey_map.iter() {
                    let mut string = String::new();
                    match value.0.0 {
                        Some(x) => string.push_str(&format!("{:?} + ", x)),
                        None => {}
                    }
                    string.push_str(&format!("{:?}", value.0.1));
                    f.write_all(string.as_bytes()).unwrap();
                    f.write_all("\n".as_bytes()).unwrap();
                }
            }
            _ => panic!("Too many keys")
        }
    }
    pub fn take_screenshot(&mut self) {

        // if self.remaining_time == 0 {

            let screens = Screen::all().unwrap();
            let mut selected_screen = screens[self.selected_monitor as usize].clone();
            let x = 0;
            let y = 0;
            let width = selected_screen.display_info.width;
            let height = selected_screen.display_info.height;
            std::thread::sleep(Duration::from_secs(self.delay_timer as u64));

            let image = selected_screen.capture_area(x, y, width, height).unwrap();

            let buffer = image.to_png(None).unwrap();
            let img = image::load_from_memory_with_format(&buffer, image::ImageFormat::Png).unwrap();
            let img = img.resize((width as f32 / 1.5) as u32, (height as f32 / 1.5) as u32, imageops::FilterType::Lanczos3);

            self.image = img.clone();
            self.image_back = self.image.clone();
            self.mode = "taken".to_string();
            self.remaining_time = self.delay_timer;



        // }
        // else {
            // let self_ref = Rc::new(RefCell::new(self.clone()));
            //
            //
            // self.timer.schedule_with_delay(
            //     chrono::Duration::seconds(1)
            //     , || {
            //         let mut self_mut = self_ref.borrow_mut();
            //
            //         self_mut.remaining_time -= 1; self_mut.take_screenshot() });

        // }



    }


    pub fn copy_to_clipboard(&mut self) {
        let mut clipboard = Clipboard::new().unwrap();
        let r = self.image.resize(self.current_width as u32, self.current_height as u32, imageops::FilterType::Lanczos3).into_rgba8();
        let (w, h) = r.dimensions();
        let img = ImageData {
            width: usize::try_from(w).unwrap(),
            height: usize::try_from(h).unwrap(),
            bytes: Cow::from(r.as_bytes()),
        };

        clipboard.set_image(img).expect("Error in copying to clipboard");
    }

    pub fn load_image_from_memory(image: DynamicImage) -> Result<ColorImage, image::ImageError> {
        let size = [image.width() as _, image.height() as _];
        let image_buffer = image.to_rgba8();
        let pixels = image_buffer.as_flat_samples();
        Ok(ColorImage::from_rgba_unmultiplied(
            size,
            pixels.as_slice(),
        ))
    }
    pub fn save_image_to_disk(&mut self, format: &str, path: &str, filename: &str) -> Result<(), Box<dyn Error>> {

        //NOT MANAGED: OVERRIDE
        match format {
            ".png" => self.image.clone().save(format!("{}/{}.png", path, filename))?,
            ".gif" => self.image.clone().save(format!("{}/{}.gif", path, filename))?,
            ".jpg" => self.image.clone().save(format!("{}/{}.jpg", path, filename))?,
            _ => return Ok(()),
        }
        Ok(())
    }

    pub fn draw_arrow(image: &DynamicImage, x0: f32, y0: f32, x1: f32, y1: f32, color: Rgba<u8>) -> DynamicImage {
        // Draw the main arrow line
        if ((x0 - x1).abs() < 1.0 || (y0 - y1).abs() < 1.0) {
            return image.clone();
        }
        let mut img = image::DynamicImage::ImageRgba8(imageproc::drawing::draw_line_segment(image, (x0, y0), (x1, y1), color));

        // Calculate arrowhead points
        let arrow_length = 15.0;
        let arrow_angle: f64 = 20.0;
        let dx = f64::from(x1 - x0);
        let dy = f64::from(y1 - y0);
        let angle = (dy).atan2(dx);
        let arrowhead_size = (dx * dx + dy * dy).sqrt().min(arrow_length);

        // Calculate arrowhead vertices
        let angle1 = angle + arrow_angle.to_radians();
        let angle2 = angle - arrow_angle.to_radians();

        let x2 = (x1 as f64 - arrowhead_size * angle1.cos()) as f32;
        let y2 = (y1 as f64 - arrowhead_size * angle1.sin()) as f32;
        let x3 = (x1 as f64 - arrowhead_size * angle2.cos()) as f32;
        let y3 = (y1 as f64 - arrowhead_size * angle2.sin()) as f32;

        let arrowhead_points: &[Point<i32>] = &[Point::new(x1 as i32, y1 as i32), Point::new(x2 as i32, y2 as i32), Point::new(x3 as i32, y3 as i32)];

        // Draw arrowhead polygon
        return image::DynamicImage::ImageRgba8(imageproc::drawing::draw_polygon(&img, arrowhead_points, color));
    }

    pub fn draw_rect(image: &DynamicImage, x0: f32, y0: f32, x1: f32, y1: f32, color: Rgba<u8>) -> DynamicImage {
        let mut startx = cmp::min(x0 as i32, x1 as i32);
        let mut endx = cmp::max(x0 as i32, x1 as i32);
        let mut starty = cmp::min(y0 as i32, y1 as i32);
        let mut endy = cmp::max(y0 as i32, y1 as i32);

        startx = cmp::max(startx, 0);
        starty = cmp::max(starty, 0);
        endx = cmp::max(endx, 0);
        endy = cmp::max(endy, 0);

        if (endx as u32 - startx as u32 == 0 || endy as u32 - starty as u32 == 0) {
            return image::DynamicImage::ImageRgba8(imageproc::drawing::draw_line_segment(image, (startx as f32, starty as f32), (endx as f32, endy as f32), color));
        }
        return image::DynamicImage::ImageRgba8(imageproc::drawing::draw_hollow_rect(image, imageproc::rect::Rect::at(startx, starty as i32).of_size(endx as u32 - startx as u32, endy as u32 - starty as u32), color));
    }

    pub fn switch_delay_timer(&mut self) {
        match self.delay_timer {
            0 => self.delay_timer = 1,
            1 => self.delay_timer = 3,
            3 => self.delay_timer = 5,
            5 => self.delay_timer = 0,
            _ => {}
        }
    }
}

impl App for DragApp {
    //UPDATE è FONDAMENTALE. CI DEVE ESSERE SEMPRE
    fn update(&mut self, ctx: &Context, frame: &mut Frame) {
        if self.hotkey_created == false {
            let res = self.load_hotkey_map();
            self.hotkeys_strings = res.0;
            self.hotkey_map = res.1;
        }

        if self.hotkey_ui_status == false {
            self.hotkey_press();
        }

        let arial: Font<'static> = Font::try_from_bytes(include_bytes!("../fonts/arial.ttf")).unwrap();

        let screens = Screen::all().unwrap();

        match self.mode.as_str() {
            "initial" => {
                CentralPanel::default().show(ctx, |ui| {

                    ui.vertical_centered(|ui| {
                        ui.heading("Cross-platform screenshot utility");
                        ui.label("This is a cross-platform utility designed to help people take screenshots. The application is all coded and compiled in Rust");
                        ui.separator();
                        if ui.button("Take a screenshot!").clicked() {
                            self.take_screenshot();
                            frame.set_minimized(false);
                            frame.set_visible(true);
                            frame.focus();
                        }

                        if ui.button("Delay Timer = ".to_owned() + &self.delay_timer.to_string()).clicked() {
                            self.switch_delay_timer();
                        }

                        ui.vertical_centered_justified(|ui| {
                            ui.label("Select monitor:");
                            ui.with_layout(egui::Layout::top_down_justified(egui::Align::Center), |ui| {
                                ui.vertical_centered(|ui| {
                                    for (i, screen) in screens.iter().enumerate() {
                                        ui.horizontal_wrapped(|ui| {
                                            ui.with_layout(Layout::centered_and_justified(Direction::LeftToRight), |ui| {
                                                ui.radio_value(&mut self.selected_monitor, i as u32, "Monitor ".to_string() + &i.to_string());

                                            });
                                            //Radio button for selection
                                        });
                                    }
                                });
                            });
                        });


                        ui.with_layout(egui::Layout::right_to_left(Align::Max), |ui| {

                            if ui.button("Customize Hotkeys").clicked() {
                                //ROUTINE PER CAMBIARE GLI HOTKEYS. deve essere tipo una sotto finestra da cui togli focus e non puoi ricliccare su quella originale finchè non chiudi la sottofinestra. Al massimo ci confrontiamo con alessio
                                self.mode = "hotkey".to_string();
                            }

                            if ui.button("Quit").clicked() {
                                //Routine per chiudere il programma
                                std::process::exit(0);
                            }
                        });
                    });
                });
            }
            "taken" => {
                CentralPanel::default().show(ctx, |ui| {
                    egui::containers::scroll_area::ScrollArea::both().show(ui, |ui| {
                        ui.vertical(|ui| {
                            ui.heading("Screenshot taken!");
                            ui.label("You can now either modify it, save it or copy it to clipboard");
                            ui.horizontal(|ui| {
                                ui.with_layout(Layout::left_to_right(Align::Center),|ui| {
                                    egui::widgets::color_picker::color_picker_color32(ui, &mut self.color, eframe::egui::color_picker::Alpha::Opaque);

                                    if ui.button("Arrow").clicked() {
                                        self.drawing = true;
                                        self.crop = false;
                                        self.drawing_type = DrawingType::Arrow;
                                        self.image = self.image_back.clone();
                                        self.initial_pos = egui::pos2(-1.0, -1.0);
                                        self.crop_point = CropRect::default();
                                        self.current_crop_point = Corner::None;
                                        self.texting = false;
                                        self.text_string = "".to_string();
                                    }
                                    if ui.button("Circle").clicked() {
                                        self.drawing = true;
                                        self.crop = false;
                                        self.drawing_type = DrawingType::Circle;
                                        self.image = self.image_back.clone();
                                        self.initial_pos = egui::pos2(-1.0, -1.0);
                                        self.crop_point = CropRect::default();
                                        self.current_crop_point = Corner::None;
                                        self.texting = false;
                                        self.text_string = "".to_string();
                                    }
                                    if ui.button("Line").clicked() {
                                        self.drawing = true;
                                        self.crop = false;
                                        self.drawing_type = DrawingType::Line;
                                        self.image = self.image_back.clone();
                                        self.initial_pos = egui::pos2(-1.0, -1.0);
                                        self.crop_point = CropRect::default();
                                        self.current_crop_point = Corner::None;
                                        self.texting = false;
                                        self.text_string = "".to_string();
                                    }
                                    if ui.button("Rectangle").clicked() {
                                        self.drawing = true;
                                        self.crop = false;
                                        self.drawing_type = DrawingType::Rectangle;
                                        self.image = self.image_back.clone();
                                        self.initial_pos = egui::pos2(-1.0, -1.0);
                                        self.crop_point = CropRect::default();
                                        self.current_crop_point = Corner::None;
                                        self.texting = false;
                                        self.text_string = "".to_string();
                                    }
                                    if ui.button("Text").clicked() {
                                        self.drawing = false;
                                        self.crop = false;
                                        self.drawing_type = DrawingType::None;
                                        self.image = self.image_back.clone();
                                        self.initial_pos = egui::pos2(-1.0, -1.0);
                                        self.crop_point = CropRect::default();
                                        self.current_crop_point = Corner::None;

                                        self.texting = true;
                                        self.text_string = "".to_string();
                                    }
                                    if ui.button("Crop").clicked() {
                                        self.drawing = false;
                                        self.drawing_type = DrawingType::None;
                                        self.crop = true;
                                        self.initial_pos = egui::pos2(-1.0, -1.0);
                                        self.image = self.image_back.clone();
                                        self.crop_point = CropRect::new(0.0, 0.0, self.image.width() as f32, self.image.height() as f32);
                                        self.current_crop_point = Corner::None;
                                        self.image = DragApp::draw_rect(&self.image_back, 0.0, 0.0, self.image.width() as f32, self.image.height() as f32, image::Rgba(epaint::Color32::DARK_GRAY.to_array()));
                                        self.image = DragApp::draw_rect(&self.image, 0.5, 0.5, self.image.width() as f32 - 0.5, self.image.height() as f32 - 0.5, image::Rgba(epaint::Color32::DARK_GRAY.to_array()));
                                        self.image = DragApp::draw_rect(&self.image, 1.0, 1.0, self.image.width() as f32 - 1.0, self.image.height() as f32 - 1.0, image::Rgba(epaint::Color32::DARK_GRAY.to_array()));
                                        self.image = DragApp::draw_rect(&self.image, 1.5, 1.5, self.image.width() as f32 - 1.5, self.image.height() as f32 - 1.5, image::Rgba(epaint::Color32::DARK_GRAY.to_array()));

                                        self.texting = false;
                                        self.text_string = "".to_string();
                                    }
                                });

                            });

                            ui.add_space(10.0);
                            ui.separator();

                            let color_image = DragApp::load_image_from_memory(self.image.clone()).unwrap();
                            self.current_width = color_image.size[0] as i32;
                            self.current_height = color_image.size[1] as i32;
                            let texture = ui.ctx().load_texture("ScreenShot", color_image, TextureOptions::default());

                            let image_w = ui.image(&texture, texture.size_vec2());

                            ctx.input_mut(|i: &mut InputState| {
                                if self.drawing == true {
                                    if self.initial_pos.x == -1.0 && self.initial_pos.y == -1.0 && i.pointer.button_clicked(egui::PointerButton::Primary) {
                                        match i.pointer.interact_pos() {
                                            None => (),
                                            Some(m) => {
                                                if m.x - image_w.rect.left_top().x >= 0.0 && m.x - image_w.rect.left_top().x <= image_w.rect.width() && m.y - image_w.rect.left_top().y >= 0.0 && m.y - image_w.rect.left_top().y <= image_w.rect.height() {
                                                    self.initial_pos = egui::pos2(m.x - image_w.rect.left_top().x, m.y - image_w.rect.left_top().y);
                                                }
                                            }
                                        }
                                    } else if self.initial_pos.x != -1.0 && self.initial_pos.y != -1.0 && i.pointer.button_clicked(egui::PointerButton::Primary) {
                                        match i.pointer.interact_pos() {
                                            None => (),
                                            Some(mut m) => {
                                                if m.x - image_w.rect.left_top().x >= 0.0 && m.x - image_w.rect.left_top().x <= image_w.rect.width() && m.y - image_w.rect.left_top().y >= 0.0 && m.y - image_w.rect.left_top().y <= image_w.rect.height() {
                                                    m = egui::pos2(m.x - image_w.rect.left_top().x, m.y - image_w.rect.left_top().y);
                                                    match self.drawing_type {
                                                        DrawingType::None => (),
                                                        DrawingType::Arrow => self.image = DragApp::draw_arrow(&self.image_back, self.initial_pos.x, self.initial_pos.y, m.x, m.y, image::Rgba(self.color.to_array())),
                                                        DrawingType::Circle => self.image = image::DynamicImage::ImageRgba8(imageproc::drawing::draw_hollow_circle(&self.image_back, (self.initial_pos.x as i32, self.initial_pos.y as i32), m.distance(self.initial_pos) as i32, image::Rgba(self.color.to_array()))),
                                                        DrawingType::Line => self.image = image::DynamicImage::ImageRgba8(imageproc::drawing::draw_line_segment(&self.image_back, (self.initial_pos.x, self.initial_pos.y), (m.x, m.y), image::Rgba(self.color.to_array()))),
                                                        DrawingType::Rectangle => self.image = DragApp::draw_rect(&self.image_back, self.initial_pos.x, self.initial_pos.y, m.x, m.y, image::Rgba(self.color.to_array())),
                                                    }
                                                    self.image_back = self.image.clone();
                                                    self.drawing = false;
                                                    self.drawing_type = DrawingType::None;
                                                    self.initial_pos = egui::pos2(-1.0, -1.0);
                                                }
                                            }
                                        }
                                    } else if self.initial_pos.x != -1.0 && self.initial_pos.y != -1.0 {
                                        match i.pointer.interact_pos() {
                                            None => (),
                                            Some(mut m) => {
                                                m = egui::pos2(m.x - image_w.rect.left_top().x, m.y - image_w.rect.left_top().y);
                                                match self.drawing_type {
                                                    DrawingType::None => (),
                                                    DrawingType::Arrow => self.image = DragApp::draw_arrow(&self.image_back, self.initial_pos.x, self.initial_pos.y, m.x, m.y, image::Rgba(self.color.to_array())),
                                                    DrawingType::Circle => self.image = image::DynamicImage::ImageRgba8(imageproc::drawing::draw_hollow_circle(&self.image_back, (self.initial_pos.x as i32, self.initial_pos.y as i32), m.distance(self.initial_pos) as i32, image::Rgba(self.color.to_array()))),
                                                    DrawingType::Line => self.image = image::DynamicImage::ImageRgba8(imageproc::drawing::draw_line_segment(&self.image_back, (self.initial_pos.x, self.initial_pos.y), (m.x, m.y), image::Rgba(self.color.to_array()))),
                                                    DrawingType::Rectangle => self.image = DragApp::draw_rect(&self.image_back, self.initial_pos.x, self.initial_pos.y, m.x, m.y, image::Rgba(self.color.to_array())),
                                                }
                                            }
                                        }
                                    }
                                }
                                else if self.crop == true {
                                    if self.initial_pos.x == -1.0 && self.initial_pos.y == -1.0 && i.pointer.button_clicked(egui::PointerButton::Primary) {
                                        match i.pointer.interact_pos() {
                                            None => (),
                                            Some(mut m) => {
                                                if m.x - image_w.rect.left_top().x >= 0.0 && m.x - image_w.rect.left_top().x <= image_w.rect.width() && m.y - image_w.rect.left_top().y >= 0.0 && m.y - image_w.rect.left_top().y <= image_w.rect.height() {
                                                    m = egui::pos2(m.x - image_w.rect.left_top().x, m.y - image_w.rect.left_top().y);

                                                    if m.distance(egui::pos2(self.crop_point.x0, self.crop_point.y0)) <= 20.0 {
                                                        self.current_crop_point = Corner::TopLeft;
                                                        self.initial_pos = egui::pos2(self.crop_point.x1, self.crop_point.y1);
                                                    } else if m.distance(egui::pos2(self.crop_point.x1, self.crop_point.y0)) <= 20.0 {
                                                        self.current_crop_point = Corner::TopRight;
                                                        self.initial_pos = egui::pos2(self.crop_point.x0, self.crop_point.y1);
                                                    } else if m.distance(egui::pos2(self.crop_point.x0, self.crop_point.y1)) <= 20.0 {
                                                        self.current_crop_point = Corner::BottomLeft;
                                                        self.initial_pos = egui::pos2(self.crop_point.x1, self.crop_point.y0);
                                                    } else if m.distance(egui::pos2(self.crop_point.x1, self.crop_point.y1)) <= 20.0 {
                                                        self.current_crop_point = Corner::BottomRight;
                                                        self.initial_pos = egui::pos2(self.crop_point.x0, self.crop_point.y0);
                                                    }
                                                }
                                            }
                                        }
                                    } else if self.initial_pos.x != -1.0 && self.initial_pos.y != -1.0 && i.pointer.button_clicked(egui::PointerButton::Primary) {
                                        match i.pointer.interact_pos() {
                                            None => (),
                                            Some(mut m) => {
                                                if m.x - image_w.rect.left_top().x >= 0.0 && m.x - image_w.rect.left_top().x <= image_w.rect.width() && m.y - image_w.rect.left_top().y >= 0.0 && m.y - image_w.rect.left_top().y <= image_w.rect.height() {
                                                    m = egui::pos2(m.x - image_w.rect.left_top().x, m.y - image_w.rect.left_top().y);
                                                    let p1 = self.crop_point.x1 - cmp::max((self.crop_point.x1 - m.x) as i32, 50) as f32;
                                                    let p2 = self.crop_point.y1 - cmp::max((self.crop_point.y1 - m.y) as i32, 50) as f32;
                                                    let p3 = self.crop_point.x0 + cmp::max((m.x - self.crop_point.x0) as i32, 50) as f32;
                                                    let p4 = self.crop_point.y0 + cmp::max((m.y - self.crop_point.y0) as i32, 50) as f32;
                                                    match self.current_crop_point {
                                                        Corner::TopLeft => self.crop_point = CropRect::new(p1, p2, self.crop_point.x1, self.crop_point.y1),
                                                        Corner::TopRight => self.crop_point = CropRect::new(self.crop_point.x0, p2, p3, self.crop_point.y1),
                                                        Corner::BottomLeft => self.crop_point = CropRect::new(p1, self.crop_point.y0, self.crop_point.x1, p4),
                                                        Corner::BottomRight => self.crop_point = CropRect::new(self.crop_point.x0, self.crop_point.y0, p3, p4),
                                                        _ => (),
                                                    }
                                                    self.initial_pos = egui::pos2(-1.0, -1.0);
                                                }
                                            }
                                        }
                                    } else if self.initial_pos.x != -1.0 && self.initial_pos.y != -1.0 {
                                        match i.pointer.interact_pos() {
                                            None => (),
                                            Some(mut m) => {
                                                m = egui::pos2(m.x - image_w.rect.left_top().x, m.y - image_w.rect.left_top().y);
                                                let p1 = self.crop_point.x1 - cmp::max((self.crop_point.x1 - m.x) as i32, 50) as f32;
                                                let p2 = self.crop_point.y1 - cmp::max((self.crop_point.y1 - m.y) as i32, 50) as f32;
                                                let p3 = self.crop_point.x0 + cmp::max((m.x - self.crop_point.x0) as i32, 50) as f32;
                                                let p4 = self.crop_point.y0 + cmp::max((m.y - self.crop_point.y0) as i32, 50) as f32;
                                                match self.current_crop_point {
                                                    Corner::TopLeft => self.crop_point = CropRect::new(p1, p2, self.crop_point.x1, self.crop_point.y1),
                                                    Corner::TopRight => self.crop_point = CropRect::new(self.crop_point.x0, p2, p3, self.crop_point.y1),
                                                    Corner::BottomLeft => self.crop_point = CropRect::new(p1, self.crop_point.y0, self.crop_point.x1, p4),
                                                    Corner::BottomRight => self.crop_point = CropRect::new(self.crop_point.x0, self.crop_point.y0, p3, p4),
                                                    _ => (),
                                                }
                                                self.image = DragApp::draw_rect(&self.image_back, self.crop_point.x0, self.crop_point.y0, self.crop_point.x1, self.crop_point.y1, image::Rgba(epaint::Color32::DARK_GRAY.to_array()));
                                                self.image = DragApp::draw_rect(&self.image, self.crop_point.x0 + 0.5, self.crop_point.y0 + 0.5, self.crop_point.x1 - 0.5, self.crop_point.y1 - 0.5, image::Rgba(epaint::Color32::DARK_GRAY.to_array()));
                                                self.image = DragApp::draw_rect(&self.image, self.crop_point.x0 + 1.0, self.crop_point.y0 + 1.0, self.crop_point.x1 - 1.0, self.crop_point.y1 - 1.0, image::Rgba(epaint::Color32::DARK_GRAY.to_array()));
                                                self.image = DragApp::draw_rect(&self.image, self.crop_point.x0 + 1.5, self.crop_point.y0 + 1.5, self.crop_point.x1 - 1.5, self.crop_point.y1 - 1.5, image::Rgba(epaint::Color32::DARK_GRAY.to_array()));
                                            }
                                        }
                                    } else if self.initial_pos.x == -1.0 && self.initial_pos.y == -1.0 && i.pointer.button_double_clicked(egui::PointerButton::Primary) {
                                        match i.pointer.interact_pos() {
                                            None => (),
                                            Some(m) => {
                                                if m.x - image_w.rect.left_top().x >= 0.0 && m.x - image_w.rect.left_top().x <= image_w.rect.width() && m.y - image_w.rect.left_top().y >= 0.0 && m.y - image_w.rect.left_top().y <= image_w.rect.height() {
                                                    self.image = image::DynamicImage::ImageRgba8(image::imageops::crop(&mut self.image_back.clone(), self.crop_point.x0 as u32, self.crop_point.y0 as u32, (self.crop_point.x1 - self.crop_point.x0) as u32, (self.crop_point.y1 - self.crop_point.y0) as u32).to_image());
                                                    
                                                    self.image_back = self.image.clone();
                                                    self.crop = false;
                                                    self.crop_point = CropRect::default();
                                                    self.current_crop_point = Corner::None;
                                                    self.initial_pos = egui::pos2(-1.0, -1.0);
                                                }
                                            }
                                        }
                                    }
                                }
                                else if self.texting == true {
                                    if self.initial_pos.x == -1.0 && self.initial_pos.y == -1.0 && i.pointer.button_clicked(egui::PointerButton::Primary) {
                                        match i.pointer.interact_pos() {
                                            None => (),
                                            Some(m) => {
                                                if m.x - image_w.rect.left_top().x >= 0.0 && m.x - image_w.rect.left_top().x <= image_w.rect.width() && m.y - image_w.rect.left_top().y >= 0.0 && m.y - image_w.rect.left_top().y <= image_w.rect.height() {
                                                    self.initial_pos = egui::pos2(m.x - image_w.rect.left_top().x, m.y - image_w.rect.left_top().y);
                                                }
                                            }
                                        }
                                    } else if self.initial_pos.x != -1.0 && self.initial_pos.y != -1.0 {
                                        for key in &self.all_keys {
                                            if i.consume_key(Modifiers::NONE, *key) {
                                                if *key == Key::Backspace {
                                                    self.text_string.pop();
                                                } else if *key == Key::Space {
                                                    self.text_string.push_str(" ");
                                                } else if *key == Key::Enter {
                                                    self.image = image::DynamicImage::ImageRgba8(imageproc::drawing::draw_text(&self.image_back, image::Rgba(self.color.to_array()), self.initial_pos.x as i32, self.initial_pos.y as i32, rusttype::Scale { x: 30.0, y: 30.0 }, &arial, &self.text_string));
                                                    self.image_back = self.image.clone();
                                                    self.texting = false;
                                                    self.text_string = "".to_string();
                                                    self.initial_pos = egui::pos2(-1.0, -1.0);
                                                } else {
                                                    self.text_string.push(key.symbol_or_name().chars().next().unwrap());
                                                }
                                                self.image = image::DynamicImage::ImageRgba8(imageproc::drawing::draw_text(&self.image_back, image::Rgba(self.color.to_array()), self.initial_pos.x as i32, self.initial_pos.y as i32, rusttype::Scale { x: 30.0, y: 30.0 }, &arial, &self.text_string));
                                            }
                                        }
                                    }
                                }
                            });

                            ui.separator();
                            ui.horizontal(|ui| {
                                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                                    if ui.button("Copy to clipboard").clicked() {
                                        self.copy_to_clipboard();
                                    }

                                    if ui.button("Back").clicked() {
                                        self.mode = "initial".to_string();
                                    }

                                    if ui.button("Save").clicked() {
                                        self.mode = "saving".to_string();
                                    }

                                    if ui.button("Quit").clicked() {
                                        std::process::exit(0);
                                    }
                                });

                            });

                        });
                    });
                });
            }
            "saving" => {
                CentralPanel::default().show(ctx, |ui| {

                    ui.with_layout(Layout::left_to_right(Center), |ui| {
                    ui.vertical(|ui| {
                        ui.heading("Choose a path, a name and a format for your screenshot");
                        ui.add_space(5.0);
                        ui.separator();
                        ui.add_space(5.0);
                        ui.horizontal_wrapped(|ui| {





                            ui.label("Path: ");
                            ui.text_edit_singleline(&mut self.current_path);

                            if self.save_errors.0 == true
                            {
                                ui.label("Please insert a path");
                            } else if self.save_errors.1 == true {
                                ui.label("Please insert a path that already exists");
                            }

                            ui.horizontal(|ui| {
                                ui.label("Name: ");
                                ui.text_edit_singleline(&mut self.current_name);
                            });
                            ui.horizontal(|ui| {
                                ui.label("Format: ");
                                ui.radio_value(&mut self.current_format, ".png".to_string(), ".png".to_string());
                                ui.radio_value(&mut self.current_format, ".jpg".to_string(), ".jpg".to_string());
                                ui.radio_value(&mut self.current_format, ".gif".to_string(), ".gif".to_string());
                            });


                        });

                        ui.add_space(7.0);
                        ui.horizontal_wrapped(|ui| {
                            if ui.button("Save").clicked() {
                                if self.save_errors.2 {
                                    ui.label("The chosen path is not a directory or it is already a file");
                                }

                                match self.current_path.as_str() {
                                    "" => {
                                        self.save_errors.0 = true;
                                        ()
                                    }

                                    _ => {
                                        self.save_errors = (false, false, false);
                                        let current_path = self.current_path.clone();
                                        let trimmed_path = current_path.trim();
                                        if trimmed_path.ends_with("/") == false {
                                            self.current_path = trimmed_path.to_string() + "/";
                                        }

                                        let current_path = Path::new(trimmed_path);

                                        if current_path.exists() == false {
                                            self.save_errors.1 = true;
                                            ()
                                        } else {
                                            if current_path.is_dir() == false || current_path.is_file() == true {
                                                self.save_errors.2 = true;
                                                ()
                                            } else {
                                                let res = self.save_image_to_disk(self.current_format.clone().as_str(), self.current_path.clone().as_str(), self.current_name.clone().as_str());
                                                match res {
                                                    Ok(_) => {
                                                        self.mode = "saved".to_string();
                                                    }
                                                    Err(_) => {
                                                        self.mode = "error".to_string();
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                            if ui.button("Back").clicked() {
                                self.mode= "taken".to_string();
                            }
                            if ui.button("Quit").clicked() {
                                std::process::exit(0);
                            }
                        });


                    });
                });

                });
            }
            "hotkey" => {
                let hotkeys: Vec<String> = vec!["Take a Screenshot".to_string(), "Quit".to_string(), "Switch Delay(*)".to_string(), "Copy to Clipboard(*)".to_string(), "Quick Save(*)".to_string(), "Undo(*)".to_string(), "Reset image".to_string()];

                CentralPanel::default().show(ctx, |ui| {
                    ui.vertical_centered(|ui| {
                        ui.heading("Hotkey Selection Screen");
                        ui.label("Select the hotkey you want to bind.\
                        You will have 3 seconds to choose the buttons");
                        ui.label("(*) = Only usable in the screenshot window");

                        for (i, hotkey) in hotkeys.iter().enumerate() {


                                ui.label(hotkey);
                                // ui.add_enabled(self.hotkey_ui_status, );
                                ui.add_enabled_ui(self.hotkey_ui_status == false, |ui| {
                                    let button_text: String = if self.changing_hotkey[i] == true { "  ---  ".to_string() } else { self.hotkeys_strings[i].clone().to_string() };
                                    if ui.button(button_text).on_hover_text("Change hotkey").clicked() {
                                        self.hotkey_ui_status = true;
                                        self.changing_hotkey[i] = true;
                                    };
                                });
                                ui.separator();
                        }


                        ctx.input(|i| if i.key_pressed(Key::Enter) {
                            let mut keys_pressed = i.keys_down.clone();
                            keys_pressed.remove(&Key::Enter);

                            let changing_hotkey_index = self.changing_hotkey.iter().position(|&x| x == true).unwrap();
                            let old_hotkey_strings = self.hotkeys_strings[changing_hotkey_index].clone().split(" + ").map(|x| x.to_string()).collect::<Vec<String>>();
                                println!("{:?}", keys_pressed);
                            if keys_pressed.len() != 0 {
                                let mut buf: String = "".to_string();
                                for (i, str_key) in keys_pressed.iter().enumerate() {
                                    if i == 0 { buf = str_key.symbol_or_name().to_string() } else {
                                        buf = buf.to_string() + " + " + str_key.symbol_or_name();
                                    }
                                }

                                self.hotkeys_strings[changing_hotkey_index] = buf;
                            }
                            let new_hotkey_strings = self.hotkeys_strings[changing_hotkey_index].clone().split(" + ").map(|x| x.to_string()).collect::<Vec<String>>();

                            self.update_hotkey_map(new_hotkey_strings, old_hotkey_strings);
                            self.hotkey_ui_status = false;
                            for changing_hotkey in self.changing_hotkey.iter_mut() {
                                *changing_hotkey = false;
                            }
                        }
                        );

                        ui.with_layout(Layout::right_to_left(Align::Max), |ui| {
                            if ui.button("Back").clicked() {
                                self.mode = "initial".to_string();
                            }
                            if ui.button("Quit").clicked() {
                                //Routine per chiudere il programma
                                std::process::exit(0);
                            }
                        });
                    })
                });
            }
            "saved" => {
                CentralPanel::default().show(ctx, |ui| {
                    ui.vertical_centered(|ui| {

                            ui.heading("Screenshot saved!");
                            ui.label("Screenshot saved to disk");
                            if ui.button("Home").clicked() {
                                self.mode = "initial".to_string();
                            }
                            if ui.button("Quit").clicked() {
                                //Routine per chiudere il programma
                                std::process::exit(0);
                            }

                    });

                });
            }
            "error" => {
                CentralPanel::default().show(ctx, |ui| {
                    ui.heading("Error");
                    ui.label("Something went wrong");
                    if ui.button("Take another screenshot").clicked() {
                        self.mode = "initial".to_string();
                    }
                    if ui.button("Quit").clicked() {
                        //Routine per chiudere il programma
                        std::process::exit(0);
                    }
                });
            }
            _ => {}
        }
    }
}

fn main() -> Result<(), eframe::Error> {
    let mut screen_sizes: [u32; 2] = [1920, 1080];

    for screen in Screen::all().unwrap().iter() {
        if screen.display_info.is_primary {
            screen_sizes[0] = screen.display_info.width;
            screen_sizes[1] = screen.display_info.height;
        }
    }

    let native_options = eframe::NativeOptions {
        always_on_top: false,
        resizable: true,
        follow_system_theme: true,
        centered: true,
        initial_window_size: Some(Vec2::new((screen_sizes[0] as f32/ 1.4), (screen_sizes[1] as f32/ 1.4))),
        ..Default::default()
    };
    run_native("DragCapture", native_options, Box::new(|cc| Box::new(DragApp::new(cc))))
}