#![allow(non_snake_case)]
mod hotkey_personal_lib;
mod imageprocessing_personal_lib;

use imageprocessing_personal_lib::{CropRect, DrawingType, Corner, draw_rect, draw_arrow, ImageProcSetting};

use hotkey_personal_lib::{HotkeyAction, HOTKEY_FILE, EguiKeyWrap, StringCodeWrap};

use std::borrow::Cow;
use std::time::Duration;
use std::{fs, cmp};
use std::error::Error;
use std::path::Path;
use std::collections::{BTreeMap, HashMap, VecDeque};
use std::fs::File;
use std::io::{Read, Write};

use eframe::egui::{self, Direction, Modifiers, Vec2, Align, Key, Layout};
use egui::{Context, TextureOptions, InputState};
use eframe::{App, Frame, run_native, egui::CentralPanel, CreationContext};

use screenshots::{self, Screen};

use image::*;
use epaint::ColorImage;

use arboard::*;
use dirs;
use chrono;
use rusttype::*;

use global_hotkey::hotkey;
use global_hotkey::GlobalHotKeyEvent;
use global_hotkey::{hotkey::{Code, HotKey}, GlobalHotKeyManager};
use crate::WindowMode::{ChangeHotkeys, ErrorMode, Initial, Saved, Saving, Taken};


#[derive(PartialEq, Eq)]
enum WindowMode {
    Initial,
    Taken,
    Saving,
    Saved,
    ErrorMode,
    ChangeHotkeys,
}


struct DragApp {
    delay_timer: u32,
    selected_monitor: u32,
    mode: WindowMode,

    image: DynamicImage,
    image_back: DynamicImage,
    image_history: VecDeque<DynamicImage>,

    current_name: String,
    current_path: String,
    current_format: String,
    current_width: i32,
    current_height: i32,
    save_errors: (bool, bool, bool),

    hotkeys_strings: Vec<String>,
    hotkey_ui_status: bool,
    changing_hotkey: Vec<bool>,
    hotkey_map: HashMap<u32, ((Option<hotkey::Modifiers>, Code), HotkeyAction)>,
    hotkey_created: bool,
    hotkey_manager: GlobalHotKeyManager,
    hotkeys_enabled: bool,

    color: epaint::Color32,
    image_setting: ImageProcSetting,

    all_keys: Vec<Key>,
}


impl DragApp {

    pub fn new(cc: &CreationContext<'_>) -> Self {

        let visuals = DragApp::create_visuals();
        cc.egui_ctx.set_visuals(visuals);

        Self {
            delay_timer: 0,
            selected_monitor: 0,
            mode: Initial,
            image: DynamicImage::default(),
            image_back: DynamicImage::default(),
            image_history: VecDeque::new(),
            current_width: 0,
            current_height: 0,
            current_name: chrono::Local::now().format("%Y_%m_%d_%H_%M_%S").to_string(),
            current_path: dirs::picture_dir().unwrap().to_str().unwrap().to_string(),
            current_format: ".png".to_string(),
            save_errors: (false, false, false),

            color: epaint::Color32::default(),
            image_setting:ImageProcSetting::default(),

            all_keys: vec![Key::A, Key::B, Key::C, Key::D, Key::E, Key::F, Key::G, Key::H, Key::I, Key::L, Key::M, Key::N, Key::O, Key::P, Key::Q, Key::R, Key::S, Key::T, Key::U, Key::V, Key::Z, Key::J, Key::K, Key::W, Key::X, Key::Y, Key::Num0, Key::Num1, Key::Num2, Key::Num3, Key::Num4, Key::Num5, Key::Num6, Key::Num7, Key::Num8, Key::Num9, Key::Minus, Key::PlusEquals, Key::Space, Key::Backspace, Key::Enter],
            hotkeys_strings: Vec::new(),
            hotkey_ui_status: false,
            changing_hotkey: vec![false; 7],
            hotkey_map: HashMap::new(),
            hotkey_created: false,
            hotkey_manager: GlobalHotKeyManager::new().unwrap(),
            hotkeys_enabled: true,
        }
    }

    fn create_visuals() -> egui::style::Visuals {
        let mut visuals = egui::style::Visuals::default();
    
        visuals.widgets.noninteractive.bg_fill = egui::Color32::from_black_alpha(220);
        visuals.widgets.noninteractive.fg_stroke = egui::Stroke::new(1.0, egui::Color32::WHITE);
        visuals.widgets.inactive.rounding = epaint::Rounding { nw: 3.0, ne: 3.0, sw: 3.0, se: 3.0 };
    
        visuals
    }

    fn load_hotkey_map(&mut self) -> (Vec<String>, HashMap<u32, ((Option<hotkey::Modifiers>, Code), HotkeyAction)>) {
        let mut hotkeys_strings: Vec<String> = Vec::new();
        let mut buf = String::new();
        File::open(HOTKEY_FILE).unwrap().read_to_string(&mut buf).unwrap();
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

    fn hotkey_press(&mut self) {
        if self.hotkeys_enabled == true {
            if let Ok(event) = GlobalHotKeyEvent::receiver().try_recv() {
                let value = self.hotkey_map.get(&event.id).unwrap();
                match value.1 {
                    HotkeyAction::TakeScreenshot => {
                        if self.mode != Saving {
                            self.reset_image_history();
                            self.take_screenshot();
                        }
                    }
                    HotkeyAction::Quit => {
                        std::process::exit(0);
                    }
                    HotkeyAction::SwitchDelay => {
                        self.switch_delay_timer();
                    }
                    HotkeyAction::CopyClipboard => {
                        if self.mode == Taken { self.copy_to_clipboard(); }
                    }
                    HotkeyAction::FastSave => {
                        if self.mode == Taken {
                            self.reset_image_history();
                            self.save_image_to_disk(self.current_format.clone().as_str(), self.current_path.clone().as_str(), self.current_name.clone().as_str()).unwrap();
                        }
                    }
                    HotkeyAction::Undo => {
                        if self.mode == Taken { self.undo_image_modify(); }
                    }
                    HotkeyAction::ResetImage => {
                        if self.mode == Taken { self.reset_image_history(); }
                    }
                }
            }
        }
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

    pub fn take_screenshot(&mut self) -> () {
        let screens = Screen::all().unwrap();
        let selected_screen = screens[self.selected_monitor as usize].clone();
        let x = 0;
        let y = 0;
        let width = selected_screen.display_info.width;
        let height = selected_screen.display_info.height;

        std::thread::sleep(Duration::from_secs(self.delay_timer as u64));

        let image = selected_screen.capture_area(x, y, width, height).unwrap();

        let buffer = image.to_png(None).unwrap();
        let img = load_from_memory_with_format(&buffer, ImageFormat::Png).unwrap();
        let img = img.resize((width as f32 / 1.5) as u32, (height as f32 / 1.5) as u32, imageops::FilterType::Lanczos3);

        self.image = img.clone();
        self.image_back = self.image.clone();
        self.save_image_history();
        self.mode = Taken;
        return;
    }

    pub fn undo_image_modify(&mut self) -> () {
        if self.mode == Taken {
            if self.image_history.len() > 1 {
                let image = self.image_history.pop_front();
                self.image = image.unwrap();
                self.image_back = self.image.clone();
            }
        }
    }

    pub fn save_image_history(&mut self) -> () { self.image_history.push_front(self.image_back.clone()); }

    pub fn reset_image_history(&mut self) -> () {
        let original_image = self.image_history.pop_back();
        self.image = original_image.unwrap();
        self.image_back = self.image.clone();
        self.image_history.clear();
        self.image_history.push_front(self.image.clone());
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

    pub fn load_image_from_memory(image: DynamicImage) -> Result<ColorImage, ImageError> {
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

    pub fn switch_delay_timer(&mut self) {
        match self.delay_timer {
            0 => { self.delay_timer = 1 }
            1 => { self.delay_timer = 3 }
            3 => { self.delay_timer = 5 }
            5 => { self.delay_timer = 0 }
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

        match self.mode {
            Initial => {
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
                            ui.with_layout(Layout::top_down_justified(Align::Center), |ui| {
                                ui.vertical_centered(|ui| {
                                    for (i, _) in screens.iter().enumerate() {
                                        ui.horizontal_wrapped(|ui| {
                                            ui.with_layout(Layout::centered_and_justified(Direction::LeftToRight), |ui| {
                                                ui.radio_value(&mut self.selected_monitor, i as u32, "Monitor ".to_string() + &i.to_string());
                                            });
                                        });
                                    }
                                });
                            });
                        });

                        ui.with_layout(Layout::right_to_left(Align::Max), |ui| {
                            if ui.button("Customize Hotkeys").clicked() {
                                //ROUTINE PER CAMBIARE GLI HOTKEYS. deve essere tipo una sotto finestra da cui togli focus e non puoi ricliccare su quella originale finchè non chiudi la sottofinestra. Al massimo ci confrontiamo con alessio
                                self.mode = ChangeHotkeys;
                            }

                            if ui.button("Quit").clicked() {
                                //Routine per chiudere il programma
                                std::process::exit(0);
                            }
                        });
                    });
                });
            }
            Taken => {
                CentralPanel::default().show(ctx, |ui| {
                    egui::containers::scroll_area::ScrollArea::both().show(ui, |ui| {
                        ui.vertical(|ui| {
                            ui.heading("Screenshot taken!");
                            ui.label("You can now either modify it, save it or copy it to clipboard");
                            ui.horizontal(|ui| {
                                ui.with_layout(Layout::left_to_right(Align::Center), |ui| {
                                    if egui::widgets::color_picker::color_picker_color32(ui, &mut self.color, egui::color_picker::Alpha::Opaque){
                                        self.image_setting=ImageProcSetting::default();
                                        self.image = self.image_back.clone();
                                    }
                                    
                                    if ui.button("Free Draw").clicked() {
                                        self.image_setting = ImageProcSetting::setup_free_draw();
                                        self.image = self.image_back.clone();
                                     
                                    }
                                    if ui.button("Arrow").clicked() {
                                        self.image_setting = ImageProcSetting::setup_arrow();
                                        self.image = self.image_back.clone();
                                     
                                    }
                                    if ui.button("Circle").clicked() {
                                        self.image_setting = ImageProcSetting::setup_circle();
                                        self.image = self.image_back.clone(); 
                                        
                                    }
                                    if ui.button("Line").clicked() {
                                        self.image_setting = ImageProcSetting::setup_line();
                                        self.image = self.image_back.clone();
                                        
                                    }
                                    if ui.button("Rectangle").clicked() {
                                        self.image_setting = ImageProcSetting::setup_rectangle();
                                        self.image = self.image_back.clone();
                                        
                                    }
                                    if ui.button("Text").clicked() {
                                        self.image_setting = ImageProcSetting::setup_text();
                                        self.image = self.image_back.clone();
                                        
                                    }
                                    if ui.button("Crop").clicked() {
                                        self.image_setting = ImageProcSetting::setup_crop(self.image.width() as f32, self.image.height() as f32);
                                    
                                        self.image = self.image_back.clone();
                                        
                                        self.image = draw_rect(&self.image_back, 0.0, 0.0, self.image.width() as f32, self.image.height() as f32, Rgba(epaint::Color32::DARK_GRAY.to_array()));
                                        self.image = draw_rect(&self.image, 0.5, 0.5, self.image.width() as f32 - 0.5, self.image.height() as f32 - 0.5, Rgba(epaint::Color32::DARK_GRAY.to_array()));
                                        self.image = draw_rect(&self.image, 1.0, 1.0, self.image.width() as f32 - 1.0, self.image.height() as f32 - 1.0, Rgba(epaint::Color32::DARK_GRAY.to_array()));
                                        self.image = draw_rect(&self.image, 1.5, 1.5, self.image.width() as f32 - 1.5, self.image.height() as f32 - 1.5, Rgba(epaint::Color32::DARK_GRAY.to_array()));
                                    }

                                    ui.add_enabled_ui(self.image_history.len() > 1 && !self.image_setting.drawing && !self.image_setting.crop && !self.image_setting.texting, |ui| {
                                        if ui.button("Undo").on_hover_text("Undo last drawing").clicked() {
                                            self.undo_image_modify();
                                        }
                                    });

                                    ui.add_enabled_ui(self.image_history.len() > 1 && !self.image_setting.drawing && !self.image_setting.crop && !self.image_setting.texting, |ui| {
                                        if ui.button("Reset").on_hover_text("Reset screenshot").clicked() {
                                            self.reset_image_history();
                                        }
                                    });
                                });
                            });

                            ui.add_space(10.0);
                            ui.separator();

                            let color_image = DragApp::load_image_from_memory(self.image.clone()).unwrap();
                            self.current_width = color_image.size[0] as i32;
                            self.current_height = color_image.size[1] as i32;
                            let texture = ui.ctx().load_texture("ScreenShot", color_image, TextureOptions::default());

                            let image_w = ui.image(&texture, texture.size_vec2());

                            ctx.input_mut(|i: &mut InputState| if self.image_setting.drawing == true {
                                if self.image_setting.initial_pos.x == -1.0 && self.image_setting.initial_pos.y == -1.0 && i.pointer.button_clicked(egui::PointerButton::Primary) {
                                    match i.pointer.interact_pos() {
                                        None => (),
                                        Some(m) => {
                                            if m.x - image_w.rect.left_top().x >= 0.0 && m.x - image_w.rect.left_top().x <= image_w.rect.width() && m.y - image_w.rect.left_top().y >= 0.0 && m.y - image_w.rect.left_top().y <= image_w.rect.height() {
                                                self.image_setting.initial_pos = egui::pos2(m.x - image_w.rect.left_top().x, m.y - image_w.rect.left_top().y);
                                            }
                                        }
                                    }
                                } else if self.image_setting.initial_pos.x != -1.0 && self.image_setting.initial_pos.y != -1.0 && i.pointer.button_clicked(egui::PointerButton::Primary) {
                                    match i.pointer.interact_pos() {
                                        None => (),
                                        Some(mut m) => {
                                            if m.x - image_w.rect.left_top().x >= 0.0 && m.x - image_w.rect.left_top().x <= image_w.rect.width() && m.y - image_w.rect.left_top().y >= 0.0 && m.y - image_w.rect.left_top().y <= image_w.rect.height() {
                                                m = egui::pos2(m.x - image_w.rect.left_top().x, m.y - image_w.rect.left_top().y);
                                                match self.image_setting.drawing_type {
                                                    DrawingType::None => (),
                                                    DrawingType::Arrow => self.image = draw_arrow(&self.image_back, self.image_setting.initial_pos.x, self.image_setting.initial_pos.y, m.x, m.y, Rgba(self.color.to_array())),
                                                    DrawingType::Circle => self.image = DynamicImage::ImageRgba8(imageproc::drawing::draw_hollow_circle(&self.image_back, (self.image_setting.initial_pos.x as i32, self.image_setting.initial_pos.y as i32), m.distance(self.image_setting.initial_pos) as i32, Rgba(self.color.to_array()))),
                                                    DrawingType::Line => self.image = DynamicImage::ImageRgba8(imageproc::drawing::draw_line_segment(&self.image_back, (self.image_setting.initial_pos.x, self.image_setting.initial_pos.y), (m.x, m.y), Rgba(self.color.to_array()))),
                                                    DrawingType::Rectangle => self.image = draw_rect(&self.image_back, self.image_setting.initial_pos.x, self.image_setting.initial_pos.y, m.x, m.y, Rgba(self.color.to_array())),
                                                }
                                                self.save_image_history();
                                                self.image_back = self.image.clone();
                                                self.image_setting.drawing = false;
                                                self.image_setting.drawing_type = DrawingType::None;
                                                self.image_setting.initial_pos = egui::pos2(-1.0, -1.0);
                                            }
                                        }
                                    }
                                } else if self.image_setting.initial_pos.x != -1.0 && self.image_setting.initial_pos.y != -1.0 {
                                    match i.pointer.interact_pos() {
                                        None => (),
                                        Some(mut m) => {
                                            m = egui::pos2(m.x - image_w.rect.left_top().x, m.y - image_w.rect.left_top().y);
                                            match self.image_setting.drawing_type {
                                                DrawingType::None => (),
                                                DrawingType::Arrow => self.image = draw_arrow(&self.image_back, self.image_setting.initial_pos.x, self.image_setting.initial_pos.y, m.x, m.y, Rgba(self.color.to_array())),
                                                DrawingType::Circle => self.image = DynamicImage::ImageRgba8(imageproc::drawing::draw_hollow_circle(&self.image_back, (self.image_setting.initial_pos.x as i32, self.image_setting.initial_pos.y as i32), m.distance(self.image_setting.initial_pos) as i32, Rgba(self.color.to_array()))),
                                                DrawingType::Line => self.image = DynamicImage::ImageRgba8(imageproc::drawing::draw_line_segment(&self.image_back, (self.image_setting.initial_pos.x, self.image_setting.initial_pos.y), (m.x, m.y), Rgba(self.color.to_array()))),
                                                DrawingType::Rectangle => self.image =draw_rect(&self.image_back, self.image_setting.initial_pos.x, self.image_setting.initial_pos.y, m.x, m.y, Rgba(self.color.to_array())),
                                            }
                                        }
                                    }
                                }
                            } else if self.image_setting.free_drawing == true{
                                if self.image_setting.initial_pos.x == -1.0 && self.image_setting.initial_pos.y == -1.0 && i.pointer.button_clicked(egui::PointerButton::Primary) {
                                    match i.pointer.interact_pos() {
                                        None => (),
                                        Some(m) => {
                                            if m.x - image_w.rect.left_top().x >= 0.0 && m.x - image_w.rect.left_top().x <= image_w.rect.width() && m.y - image_w.rect.left_top().y >= 0.0 && m.y - image_w.rect.left_top().y <= image_w.rect.height() {
                                                self.image_setting.initial_pos = egui::pos2(m.x - image_w.rect.left_top().x, m.y - image_w.rect.left_top().y);
                                                self.image_setting.free_drawing_points.push(self.image_setting.initial_pos);
                                            }
                                        }
                                    }
                                }else if self.image_setting.initial_pos.x != -1.0 && self.image_setting.initial_pos.y != -1.0 && i.pointer.button_clicked(egui::PointerButton::Primary){
                                    match i.pointer.interact_pos() {
                                        None => (),
                                        Some(mut m) => {
                                            if m.x - image_w.rect.left_top().x >= 0.0 && m.x - image_w.rect.left_top().x <= image_w.rect.width() && m.y - image_w.rect.left_top().y >= 0.0 && m.y - image_w.rect.left_top().y <= image_w.rect.height() {
                                                m = egui::pos2(m.x - image_w.rect.left_top().x, m.y - image_w.rect.left_top().y);
                                                self.image= DynamicImage::ImageRgba8(imageproc::drawing::draw_line_segment(&self.image, (self.image_setting.free_drawing_points.last().unwrap().x, self.image_setting.free_drawing_points.last().unwrap().y), (m.x, m.y), Rgba(self.color.to_array())));
                                                self.save_image_history();
                                                self.image_back = self.image.clone();
                                                
                                                
                                                self.image_setting.free_drawing = false;
                                                self.image_setting.free_drawing_points = Vec::new();
                                                self.image_setting.initial_pos = egui::pos2(-1.0, -1.0);
                                            }
                                        }
                                    }
                                }else if self.image_setting.initial_pos.x != -1.0 && self.image_setting.initial_pos.y != -1.0 {
                                    match i.pointer.interact_pos() {
                                        None => (),
                                        Some(mut m) => {
                                            m = egui::pos2(m.x - image_w.rect.left_top().x, m.y - image_w.rect.left_top().y);
                                            self.image= DynamicImage::ImageRgba8(imageproc::drawing::draw_line_segment(&self.image, (self.image_setting.free_drawing_points.last().unwrap().x, self.image_setting.free_drawing_points.last().unwrap().y), (m.x, m.y), Rgba(self.color.to_array())));
                                            self.image_setting.free_drawing_points.push(m);
                                        }
                                    }
                                }


                            }else if self.image_setting.crop == true {
                                if self.image_setting.initial_pos.x == -1.0 && self.image_setting.initial_pos.y == -1.0 && i.pointer.button_clicked(egui::PointerButton::Primary) {
                                    match i.pointer.interact_pos() {
                                        None => (),
                                        Some(mut m) => {
                                            if m.x - image_w.rect.left_top().x >= 0.0 && m.x - image_w.rect.left_top().x <= image_w.rect.width() && m.y - image_w.rect.left_top().y >= 0.0 && m.y - image_w.rect.left_top().y <= image_w.rect.height() {
                                                m = egui::pos2(m.x - image_w.rect.left_top().x, m.y - image_w.rect.left_top().y);

                                                if m.distance(egui::pos2(self.image_setting.crop_point.x0, self.image_setting.crop_point.y0)) <= 20.0 {
                                                    self.image_setting.current_crop_point = Corner::TopLeft;
                                                    self.image_setting.initial_pos = egui::pos2(self.image_setting.crop_point.x1, self.image_setting.crop_point.y1);
                                                } else if m.distance(egui::pos2(self.image_setting.crop_point.x1, self.image_setting.crop_point.y0)) <= 20.0 {
                                                    self.image_setting.current_crop_point = Corner::TopRight;
                                                    self.image_setting.initial_pos = egui::pos2(self.image_setting.crop_point.x0, self.image_setting.crop_point.y1);
                                                } else if m.distance(egui::pos2(self.image_setting.crop_point.x0, self.image_setting.crop_point.y1)) <= 20.0 {
                                                    self.image_setting.current_crop_point = Corner::BottomLeft;
                                                    self.image_setting.initial_pos = egui::pos2(self.image_setting.crop_point.x1, self.image_setting.crop_point.y0);
                                                } else if m.distance(egui::pos2(self.image_setting.crop_point.x1, self.image_setting.crop_point.y1)) <= 20.0 {
                                                    self.image_setting.current_crop_point = Corner::BottomRight;
                                                    self.image_setting.initial_pos = egui::pos2(self.image_setting.crop_point.x0, self.image_setting.crop_point.y0);
                                                }
                                            }
                                        }
                                    }
                                } else if self.image_setting.initial_pos.x != -1.0 && self.image_setting.initial_pos.y != -1.0 && i.pointer.button_clicked(egui::PointerButton::Primary) {
                                    match i.pointer.interact_pos() {
                                        None => (),
                                        Some(mut m) => {
                                            if m.x - image_w.rect.left_top().x >= 0.0 && m.x - image_w.rect.left_top().x <= image_w.rect.width() && m.y - image_w.rect.left_top().y >= 0.0 && m.y - image_w.rect.left_top().y <= image_w.rect.height() {
                                                m = egui::pos2(m.x - image_w.rect.left_top().x, m.y - image_w.rect.left_top().y);
                                                let p1 = self.image_setting.crop_point.x1 - cmp::max((self.image_setting.crop_point.x1 - m.x) as i32, 50) as f32;
                                                let p2 = self.image_setting.crop_point.y1 - cmp::max((self.image_setting.crop_point.y1 - m.y) as i32, 50) as f32;
                                                let p3 = self.image_setting.crop_point.x0 + cmp::max((m.x - self.image_setting.crop_point.x0) as i32, 50) as f32;
                                                let p4 = self.image_setting.crop_point.y0 + cmp::max((m.y - self.image_setting.crop_point.y0) as i32, 50) as f32;
                                                match self.image_setting.current_crop_point {
                                                    Corner::TopLeft => self.image_setting.crop_point = CropRect::new(p1, p2, self.image_setting.crop_point.x1, self.image_setting.crop_point.y1),
                                                    Corner::TopRight => self.image_setting.crop_point = CropRect::new(self.image_setting.crop_point.x0, p2, p3, self.image_setting.crop_point.y1),
                                                    Corner::BottomLeft => self.image_setting.crop_point = CropRect::new(p1, self.image_setting.crop_point.y0, self.image_setting.crop_point.x1, p4),
                                                    Corner::BottomRight => self.image_setting.crop_point = CropRect::new(self.image_setting.crop_point.x0, self.image_setting.crop_point.y0, p3, p4),
                                                    _ => (),
                                                }
                                                self.image_setting.initial_pos = egui::pos2(-1.0, -1.0);
                                            }
                                        }
                                    }
                                } else if self.image_setting.initial_pos.x != -1.0 && self.image_setting.initial_pos.y != -1.0 {
                                    match i.pointer.interact_pos() {
                                        None => (),
                                        Some(mut m) => {
                                            m = egui::pos2(m.x - image_w.rect.left_top().x, m.y - image_w.rect.left_top().y);
                                            let p1 = self.image_setting.crop_point.x1 - cmp::max((self.image_setting.crop_point.x1 - m.x) as i32, 50) as f32;
                                            let p2 = self.image_setting.crop_point.y1 - cmp::max((self.image_setting.crop_point.y1 - m.y) as i32, 50) as f32;
                                            let p3 = self.image_setting.crop_point.x0 + cmp::max((m.x - self.image_setting.crop_point.x0) as i32, 50) as f32;
                                            let p4 = self.image_setting.crop_point.y0 + cmp::max((m.y - self.image_setting.crop_point.y0) as i32, 50) as f32;
                                            match self.image_setting.current_crop_point {
                                                Corner::TopLeft => self.image_setting.crop_point = CropRect::new(p1, p2, self.image_setting.crop_point.x1, self.image_setting.crop_point.y1),
                                                Corner::TopRight => self.image_setting.crop_point = CropRect::new(self.image_setting.crop_point.x0, p2, p3, self.image_setting.crop_point.y1),
                                                Corner::BottomLeft => self.image_setting.crop_point = CropRect::new(p1, self.image_setting.crop_point.y0, self.image_setting.crop_point.x1, p4),
                                                Corner::BottomRight => self.image_setting.crop_point = CropRect::new(self.image_setting.crop_point.x0, self.image_setting.crop_point.y0, p3, p4),
                                                _ => (),
                                            }
                                            self.image = draw_rect(&self.image_back, self.image_setting.crop_point.x0, self.image_setting.crop_point.y0, self.image_setting.crop_point.x1, self.image_setting.crop_point.y1, Rgba(epaint::Color32::DARK_GRAY.to_array()));
                                            self.image = draw_rect(&self.image, self.image_setting.crop_point.x0 + 0.5, self.image_setting.crop_point.y0 + 0.5, self.image_setting.crop_point.x1 - 0.5, self.image_setting.crop_point.y1 - 0.5, Rgba(epaint::Color32::DARK_GRAY.to_array()));
                                            self.image = draw_rect(&self.image, self.image_setting.crop_point.x0 + 1.0, self.image_setting.crop_point.y0 + 1.0, self.image_setting.crop_point.x1 - 1.0, self.image_setting.crop_point.y1 - 1.0, Rgba(epaint::Color32::DARK_GRAY.to_array()));
                                            self.image = draw_rect(&self.image, self.image_setting.crop_point.x0 + 1.5, self.image_setting.crop_point.y0 + 1.5, self.image_setting.crop_point.x1 - 1.5, self.image_setting.crop_point.y1 - 1.5, Rgba(epaint::Color32::DARK_GRAY.to_array()));
                                        }
                                    }
                                } else if self.image_setting.initial_pos.x == -1.0 && self.image_setting.initial_pos.y == -1.0 && i.pointer.button_double_clicked(egui::PointerButton::Primary) {
                                    match i.pointer.interact_pos() {
                                        None => (),
                                        Some(m) => {
                                            if m.x - image_w.rect.left_top().x >= 0.0 && m.x - image_w.rect.left_top().x <= image_w.rect.width() && m.y - image_w.rect.left_top().y >= 0.0 && m.y - image_w.rect.left_top().y <= image_w.rect.height() {
                                                self.save_image_history();
                                                self.image = DynamicImage::ImageRgba8(imageops::crop(&mut self.image_back.clone(), self.image_setting.crop_point.x0 as u32, self.image_setting.crop_point.y0 as u32, (self.image_setting.crop_point.x1 - self.image_setting.crop_point.x0) as u32, (self.image_setting.crop_point.y1 - self.image_setting.crop_point.y0) as u32).to_image());

                                                self.image_back = self.image.clone();
                                                self.image_setting.crop = false;
                                                self.image_setting.crop_point = CropRect::default();
                                                self.image_setting.current_crop_point = Corner::None;
                                                self.image_setting.initial_pos = egui::pos2(-1.0, -1.0);
                                            }
                                        }
                                    }
                                }
                            } else if self.image_setting.texting == true {
                                if self.image_setting.initial_pos.x == -1.0 && self.image_setting.initial_pos.y == -1.0 && i.pointer.button_clicked(egui::PointerButton::Primary) {
                                    match i.pointer.interact_pos() {
                                        None => (),
                                        Some(m) => {
                                            if m.x - image_w.rect.left_top().x >= 0.0 && m.x - image_w.rect.left_top().x <= image_w.rect.width() && m.y - image_w.rect.left_top().y >= 0.0 && m.y - image_w.rect.left_top().y <= image_w.rect.height() {
                                                self.image_setting.initial_pos = egui::pos2(m.x - image_w.rect.left_top().x, m.y - image_w.rect.left_top().y);
                                            }
                                        }
                                    }
                                } else if self.image_setting.initial_pos.x != -1.0 && self.image_setting.initial_pos.y != -1.0 {
                                    for key in &self.all_keys {
                                        if i.consume_key(Modifiers::NONE, *key) {
                                            if *key == Key::Backspace {
                                                self.image_setting.text_string.pop();
                                            } else if *key == Key::Space {
                                                self.image_setting.text_string.push_str(" ");
                                            } else if *key == Key::Enter {
                                                if self.image_setting.text_string != "".to_string() {
                                                    self.image_history.push_front(self.image_back.clone());
                                                }

                                                self.image = DynamicImage::ImageRgba8(imageproc::drawing::draw_text(&self.image_back, Rgba(self.color.to_array()), self.image_setting.initial_pos.x as i32, self.image_setting.initial_pos.y as i32, Scale { x: 30.0, y: 30.0 }, &arial, &self.image_setting.text_string));
                                                self.image_back = self.image.clone();
                                                self.image_setting.texting = false;
                                                self.image_setting.text_string = "".to_string();
                                                self.image_setting.initial_pos = egui::pos2(-1.0, -1.0);
                                            } else {
                                                self.image_setting.text_string.push(key.symbol_or_name().chars().next().unwrap());
                                            }
                                            self.image = DynamicImage::ImageRgba8(imageproc::drawing::draw_text(&self.image_back, Rgba(self.color.to_array()), self.image_setting.initial_pos.x as i32, self.image_setting.initial_pos.y as i32, Scale { x: 30.0, y: 30.0 }, &arial, &self.image_setting.text_string));
                                        }
                                    }
                                }
                            });

                            ui.separator();
                            ui.horizontal(|ui| {
                                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                                    if ui.button("Copy to clipboard").clicked() {
                                        self.image_setting=ImageProcSetting::default();
                                        self.image = self.image_back.clone();
                                        
                                        self.copy_to_clipboard();
                                    }

                                    if ui.button("Back").clicked() {
                                        self.image_setting=ImageProcSetting::default();
                                        self.image = self.image_back.clone();
                                        
                                        self.mode = Initial;
                                    }

                                    if ui.button("Save").clicked() {
                                        self.image_setting=ImageProcSetting::default();
                                        self.image = self.image_back.clone();
                                        
                                        self.mode = Saving;
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
            Saving => {
                CentralPanel::default().show(ctx, |ui| {
                    ui.vertical_centered(|ui| {
                        ui.heading("Choose a path, a name and a format for your screenshot");
                        ui.add_space(5.0);
                        ui.separator();
                        ui.add_space(5.0);
                        ui.horizontal_wrapped(|ui| {
                            ui.horizontal_wrapped(|ui| {
                                ui.label("Path: ");
                                ui.text_edit_singleline(&mut self.current_path);
                                if self.save_errors.0 == true
                                {
                                    ui.label("Please insert a path");
                                } else if self.save_errors.1 == true {
                                    ui.label("Please insert a path that already exists");
                                }
                            });


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
                                                        self.mode = Saved;
                                                    }
                                                    Err(_) => {
                                                        self.mode = ErrorMode;
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                            if ui.button("Back").clicked() {
                                self.mode = Taken;
                            }
                            if ui.button("Quit").clicked() {
                                std::process::exit(0);
                            }
                        });
                    });
                });
            }
            ChangeHotkeys => {
                self.hotkeys_enabled = false;
                let hotkeys: Vec<String> = vec!["Take a Screenshot".to_string(), "Quit".to_string(), "Switch Delay(*)".to_string(), "Copy to Clipboard(*)".to_string(), "Quick Save(*)".to_string(), "Undo(*)".to_string(), "Reset image".to_string()];

                CentralPanel::default().show(ctx, |ui| {
                    ui.vertical_centered(|ui| {
                        ui.heading("Hotkey Selection Screen");
                        ui.label("Select the hotkey you want to bind.\
                        Press a key and a modifier OR just a key and then Enter to bind it. If you press more keys/modifiers at once, just one will be chosen");
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
                            let pressed_modifiers = i.modifiers;
                            let mut keys_pressed = i.keys_down.clone();
                            keys_pressed.remove(&Key::Enter);


                            let changing_hotkey_index = self.changing_hotkey.iter().position(|&x| x == true).unwrap();
                            let old_hotkey_strings = self.hotkeys_strings[changing_hotkey_index].clone().split(" + ").map(|x| x.to_string()).collect::<Vec<String>>();
                            if keys_pressed.len() != 0 {
                                let mut buf: String = "".to_string();
                                //Take the only element left in an hashset
                                let key = keys_pressed.iter().next().unwrap();
                                //we search for the true value in pressed_modifiers which is a structy that contains 5 bool fields

                                if pressed_modifiers.mac_cmd {
                                    buf = "COMMAND + ".to_string();
                                }
                                if pressed_modifiers.command {
                                    buf = "COMMAND + ".to_string();
                                }
                                if pressed_modifiers.ctrl {
                                    buf = "CONTROL + ".to_string();
                                }
                                if pressed_modifiers.shift {
                                    buf = "SHIFT + ".to_string();
                                }
                                if pressed_modifiers.alt {
                                    buf = "ALT + ".to_string();
                                }

                                let pressed_string: String = EguiKeyWrap::new(key.clone()).into();
                                buf = buf.to_string() + &*pressed_string;


                                self.hotkeys_strings[changing_hotkey_index] = buf;
                                let new_hotkey_strings = self.hotkeys_strings[changing_hotkey_index].clone().split(" + ").map(|x| x.to_string()).collect::<Vec<String>>();

                                self.update_hotkey_map(new_hotkey_strings, old_hotkey_strings);
                            } else {
                                self.hotkey_ui_status = false;
                                for changing_hotkey in self.changing_hotkey.iter_mut() {
                                    *changing_hotkey = false;
                                }
                            }

                            self.hotkey_ui_status = false;
                            for changing_hotkey in self.changing_hotkey.iter_mut() {
                                *changing_hotkey = false;
                            }
                        }
                        );

                        ui.with_layout(Layout::right_to_left(Align::Max), |ui| {
                            if ui.button("Back").clicked() {
                                self.mode = Initial;
                                self.hotkeys_enabled = true;
                            }
                            if ui.button("Quit").clicked() {
                                //Routine per chiudere il programma
                                std::process::exit(0);
                            }
                        });
                    })
                });
            }
            Saved => {
                CentralPanel::default().show(ctx, |ui| {
                    ui.vertical_centered(|ui| {
                        ui.heading("Screenshot saved!");
                        ui.label("Screenshot saved to disk");
                        if ui.button("Home").clicked() {
                            self.mode = Initial;
                        }
                        if ui.button("Quit").clicked() {
                            //Routine per chiudere il programma
                            std::process::exit(0);
                        }
                    });
                });
            }
            ErrorMode => {
                CentralPanel::default().show(ctx, |ui| {
                    ui.heading("Error");
                    ui.label("Something went wrong");
                    if ui.button("Take another screenshot").clicked() {
                        self.mode = Initial;
                    }
                    if ui.button("Quit").clicked() {
                        //Routine per chiudere il programma
                        std::process::exit(0);
                    }
                });
            }
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
        initial_window_size: Some(Vec2::new(screen_sizes[0] as f32 / 1.4, screen_sizes[1] as f32 / 1.4)),
        ..Default::default()
    };
    run_native("DragCapture", native_options, Box::new(|cc| Box::new(DragApp::new(cc))))
}