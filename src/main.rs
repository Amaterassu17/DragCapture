use std::borrow::Cow;
use std::time::Duration;
use eframe::{App, Frame, run_native, Storage, egui::CentralPanel, CreationContext};

use egui::{Context, Image, Rect, Visuals, Window, TextureHandle, TextureOptions, InputState};
use eframe::egui::{self, Modifiers};
use egui::widgets::color_picker;
use egui_glium::egui_winit::egui::{Color32, Rounding};
use imageproc::point::Point;
use screenshots::{Screen, Compression};
use screenshots;
use std::{fs, cmp};
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
enum DrawingType{
    None, Arrow, Circle, Rectangle, Line
}
enum Corner{
    None, TopLeft, TopRight, BottomLeft, BottomRight
}

struct CropRect{
    x0:f32,
    y0:f32,
    x1:f32,
    y1:f32,
}
impl CropRect{
    fn new(x0: f32,y0: f32,x1: f32,y1: f32) ->Self{
        return CropRect { x0: x0, y0: y0, x1: x1, y1: y1 };
    }

}
impl Default for CropRect{
    fn default() -> Self {
        return CropRect { x0: -1.0, y0: -1.0, x1: -1.0, y1: -1.0 };
    }
}


enum Hotkeys {
    TakeScreenshot
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
    initial_pos: egui::Pos2,
    hotkeys_strings: Vec<String>,
    hotkey_ui_status: bool,
    changing_hotkey: Vec<bool>,
    crop: bool,
    crop_point:CropRect,
    current_crop_point: Corner,
    color: epaint::Color32,
    texting:bool,
    text_string: String,
    all_keys:Vec<Key>,
}

fn create_visuals() -> egui::style::Visuals {
    let mut visuals = egui::style::Visuals::default();

    visuals.widgets.noninteractive.bg_fill = egui::Color32::from_black_alpha(220);
    visuals.widgets.noninteractive.fg_stroke = egui::Stroke::new(1.0, egui::Color32::WHITE);
    visuals.widgets.inactive.rounding = epaint::Rounding{ nw: 3.0, ne: 3.0, sw: 3.0, se: 3.0 };

   
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
            crop:false,
            crop_point: CropRect::default(),
            current_crop_point: Corner::None,
            color: epaint::Color32::default(),
            hotkeys_strings : vec!["S + W".to_string(),"Q + Esc".to_string()],
            hotkey_ui_status: false,
            changing_hotkey: vec![false; 5],
            texting:false,
            text_string: "".to_string(),
            all_keys: vec![Key::A, Key::B, Key::C,Key::D,Key::E,Key::F,Key::G,Key::H,Key::I,Key::L,Key::M,Key::N,Key::O,Key::P,Key::Q,Key::R,Key::S,Key::T,Key::U,Key::V,Key::Z,Key::J,Key::K,Key::W,Key::X,Key::Y, Key::Num0 , Key::Num1 , Key::Num2 , Key::Num3, Key::Num4, Key::Num5, Key::Num6 , Key::Num7 , Key::Num8, Key::Num9, Key::Minus, Key::PlusEquals, Key::Space, Key::Backspace, Key::Enter],
        }
    }

    pub fn take_screenshot (&mut self) {

        let screens = Screen::all().unwrap();
        let mut selected_screen = screens[self.selected_monitor as usize].clone();
        let x= 0;
        let y = 0;
        let width=selected_screen.display_info.width;
        let height=selected_screen.display_info.height;
        std::thread::sleep(Duration::from_secs(self.delay_timer as u64));

        let image = selected_screen.capture_area(x, y, width, height).unwrap();

        let buffer = image.to_png(None).unwrap();
        let img=  image::load_from_memory_with_format(&buffer, image::ImageFormat::Png).unwrap();
        let img = img.resize(width/2, height/2, imageops::FilterType::Lanczos3);

        self.image = img.clone();
        self.image_back = self.image.clone();
        self.mode="taken".to_string();

    }

    pub fn copy_to_clipboard (&mut self) {
        let mut clipboard = Clipboard::new().unwrap();
        let r=self.image.resize(self.current_width as u32, self.current_height as u32, imageops::FilterType::Lanczos3).into_rgba8();
        let (w,h)=r.dimensions();
        let img = ImageData {
            width: usize::try_from(w).unwrap(),
            height: usize::try_from(h).unwrap(),
            bytes: Cow::from(r.as_bytes())
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

    pub fn draw_arrow(image: & DynamicImage, x0: f32, y0: f32, x1: f32, y1: f32, color: Rgba<u8>) -> DynamicImage {
        // Draw the main arrow line
        if((x0-x1).abs() <1.0 && (y0-y1).abs() < 1.0){
            return image.clone();
        }
        let mut img = image::DynamicImage::ImageRgba8(imageproc::drawing::draw_line_segment(image, (x0, y0), (x1, y1), color));

        // Calculate arrowhead points
        let arrow_length = 15.0;
        let arrow_angle: f64 = 20.0 ;
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
        return image::DynamicImage::ImageRgba8( imageproc::drawing::draw_polygon(&img, arrowhead_points, color));
    }

    pub fn draw_rect(image: & DynamicImage, x0: f32, y0: f32, x1: f32, y1: f32, color: Rgba<u8>) -> DynamicImage{
        let mut startx= cmp::min(x0 as i32,x1 as i32);
        let mut endx= cmp::max(x0 as i32,x1 as i32);
        let mut starty= cmp::min(y0 as i32,y1 as i32);
        let mut endy= cmp::max(y0 as i32,y1 as i32);

        startx= cmp::max(startx, 0);
        starty= cmp::max(starty, 0);
        endx= cmp::max(endx, 0);
        endy= cmp::max(endy, 0);

        if(endx as u32 - startx as u32 == 0 || endy as u32 - starty as u32 == 0){
            return image::DynamicImage::ImageRgba8(imageproc::drawing::draw_line_segment(image, (startx as f32, starty as f32), (endx as f32, endy as f32),color ));
        }
        return image::DynamicImage::ImageRgba8(imageproc::drawing::draw_hollow_rect(image,  imageproc::rect::Rect::at(startx, starty as i32).of_size(endx as u32 - startx as u32, endy as u32 - starty as u32), color));
    }
}

impl App for DragApp {



    //UPDATE è FONDAMENTALE. CI DEVE ESSERE SEMPRE
    fn update(&mut self, ctx: &Context, frame: &mut Frame) {

        let arial:Font<'static>=Font::try_from_bytes(include_bytes!("../fonts/arial.ttf")).unwrap();

        let screens = Screen::all().unwrap();

        match self.mode.as_str() {
            "initial" => {

                CentralPanel::default().show(ctx, |ui| {
                    ui.heading("Cross-platform screenshot utility");
                    ui.label("This is a cross-platform utility designed to help people take screenshots. The application is all coded and compiled in Rust");
                    //Button
                    if ui.button("Take a screenshot!").clicked() {
                        self.take_screenshot();
                    }
                    if ui.button("Customize Hotkeys").clicked() {
                        //ROUTINE PER CAMBIARE GLI HOTKEYS. deve essere tipo una sotto finestra da cui togli focus e non puoi ricliccare su quella originale finchè non chiudi la sottofinestra. Al massimo ci confrontiamo con alessio
                        self.mode = "hotkey".to_string();
                    }
                    if ui.button("Delay Timer = ".to_owned() + &self.delay_timer.to_string()).clicked() {
                        match self.delay_timer {
                            0 => self.delay_timer = 1,
                            1 => self.delay_timer = 3,
                            3 => self.delay_timer = 5,
                            5 => self.delay_timer = 0,
                            _ => {}
                        }
                    }

                    //Container Combo box for dropdown menu

                    ui.vertical(|ui| {
                        ui.label("Select monitor: ");
                        ui.separator();
                        ui.with_layout(egui::Layout::top_down_justified(egui::Align::Center), |ui| {
                            ui.vertical_centered(|ui| {

                                for (i, screen) in screens.iter().enumerate() {
                                    ui.horizontal(|ui| {

                                        //Radio button for selection
                                        ui.radio_value(&mut self.selected_monitor, i as u32, "Monitor ".to_string() + &i.to_string());
                                    });
                                }
                            });
                        });
                    });

                    ui.horizontal(|ui| {
                        ui.with_layout(egui::Layout::right_to_left(Align::Max), |ui| {
                            if ui.button("Quit").clicked() {
                                //Routine per chiudere il programma
                                std::process::exit(0);
                            }
                        });
                    });



                });


            },
             "taken" => {
                CentralPanel::default().show(ctx, |ui| {

                    egui::containers::scroll_area::ScrollArea::both().show(ui, |ui| {

                    ui.heading("Screenshot taken!");
                
                    egui::widgets::color_picker::color_picker_color32(ui, &mut self.color, eframe::egui::color_picker::Alpha::Opaque);
                    ui.add_space(10.0);
                    

                    ui.horizontal(|ui| {
                        if ui.button("Arrow").clicked() {
                            self.drawing=true;
                            self.crop=false;
                            self.drawing_type=DrawingType::Arrow;
                            self.image= self.image_back.clone();
                            self.initial_pos=egui::pos2(-1.0, -1.0);
                            self.crop_point= CropRect::default();
                            self.current_crop_point= Corner::None;
                            self.texting=false;
                            self.text_string="".to_string();
                        }
                        if ui.button("Circle").clicked() {
                            self.drawing=true;
                            self.crop=false;
                            self.drawing_type=DrawingType::Circle;
                            self.image= self.image_back.clone();
                            self.initial_pos=egui::pos2(-1.0, -1.0);
                            self.crop_point= CropRect::default();
                            self.current_crop_point= Corner::None;
                            self.texting=false;
                            self.text_string="".to_string();
                        }
                        if ui.button("Line").clicked() {
                            self.drawing=true;
                            self.crop=false;
                            self.drawing_type=DrawingType::Line;
                            self.image= self.image_back.clone();
                            self.initial_pos=egui::pos2(-1.0, -1.0);
                            self.crop_point= CropRect::default();
                            self.current_crop_point= Corner::None;
                            self.texting=false;
                            self.text_string="".to_string();
                        }
                        if ui.button("Rectangle").clicked() {
                            self.drawing=true;
                            self.crop=false;
                            self.drawing_type=DrawingType::Rectangle;
                            self.image= self.image_back.clone();
                            self.initial_pos=egui::pos2(-1.0, -1.0);
                            self.crop_point= CropRect::default();
                            self.current_crop_point= Corner::None;
                            self.texting=false;
                            self.text_string="".to_string();
                        }
                        if ui.button("Text").clicked(){
                            self.drawing=false;
                            self.crop=false;
                            self.drawing_type=DrawingType::None;
                            self.image= self.image_back.clone();
                            self.initial_pos=egui::pos2(-1.0, -1.0);
                            self.crop_point= CropRect::default();
                            self.current_crop_point= Corner::None;

                            self.texting=true;
                            self.text_string="".to_string();
                        }
                        if ui.button("Crop").clicked() {
                            self.drawing=false;
                            self.drawing_type=DrawingType::None;
                            self.crop=true;
                            self.initial_pos=egui::pos2(-1.0, -1.0);
                            self.image= self.image_back.clone();
                            self.crop_point= CropRect::new(0.0,0.0, self.image.width() as f32, self.image.height() as f32);
                            self.current_crop_point= Corner::None;
                            self.image= DragApp::draw_rect(&self.image_back, 0.0,0.0, self.image.width() as f32, self.image.height() as f32, image::Rgba(epaint::Color32::DARK_GRAY.to_array()));
                            self.image= DragApp::draw_rect(&self.image, 0.5,0.5, self.image.width() as f32 -0.5, self.image.height() as f32 -0.5, image::Rgba(epaint::Color32::DARK_GRAY.to_array()));
                            self.image= DragApp::draw_rect(&self.image, 1.0,1.0, self.image.width() as f32 -1.0, self.image.height() as f32 -1.0, image::Rgba(epaint::Color32::DARK_GRAY.to_array()));
                            self.image= DragApp::draw_rect(&self.image, 1.5,1.5, self.image.width() as f32 -1.5, self.image.height() as f32 -1.5, image::Rgba(epaint::Color32::DARK_GRAY.to_array()));
                            
                            self.texting=false;
                            self.text_string="".to_string();
                        }
                    });

                    //Image rendering in a single frame
                    let color_image = DragApp::load_image_from_memory(self.image.clone()).unwrap();
                    self.current_width= color_image.size[0] as i32;
                    self.current_height= color_image.size[1] as i32;
                    let texture = ui.ctx().load_texture("ScreenShot", color_image, TextureOptions::default());
                
                    let image_w = ui.image(&texture, texture.size_vec2());

                    ctx.input_mut(|i: &mut InputState|{
                        if self.drawing==true {
                            if self.initial_pos.x== -1.0 && self.initial_pos.y== -1.0 && i.pointer.button_clicked(egui::PointerButton::Primary){
                                match  i.pointer.interact_pos(){
                                    None => (),
                                    Some(m) =>{
                                        if m.x - image_w.rect.left_top().x >= 0.0 && m.x - image_w.rect.left_top().x <= image_w.rect.width() && m.y - image_w.rect.left_top().y >= 0.0 && m.y - image_w.rect.left_top().y <= image_w.rect.height(){
                                            self.initial_pos= egui::pos2(m.x - image_w.rect.left_top().x, m.y - image_w.rect.left_top().y);
                                        }
                                    }
                                }
                            }
                            else if self.initial_pos.x!= -1.0 && self.initial_pos.y!= -1.0 && i.pointer.button_clicked(egui::PointerButton::Primary){
                                match  i.pointer.interact_pos(){
                                    None => (),
                                    Some(mut m) => {
                                        if m.x - image_w.rect.left_top().x >= 0.0 && m.x - image_w.rect.left_top().x <= image_w.rect.width() && m.y - image_w.rect.left_top().y >= 0.0 && m.y - image_w.rect.left_top().y <= image_w.rect.height(){
                                            m = egui::pos2(m.x - image_w.rect.left_top().x, m.y - image_w.rect.left_top().y);
                                            match self.drawing_type{
                                                DrawingType::None=>(),
                                                DrawingType::Arrow=>self.image= DragApp::draw_arrow(&self.image_back, self.initial_pos.x, self.initial_pos.y, m.x, m.y, image::Rgba(self.color.to_array())),
                                                DrawingType::Circle=>self.image = image::DynamicImage::ImageRgba8(imageproc::drawing::draw_hollow_circle(& self.image_back, (self.initial_pos.x as i32, self.initial_pos.y as i32), m.distance(self.initial_pos) as i32, image::Rgba(self.color.to_array()))),
                                                DrawingType::Line=> self.image = image::DynamicImage::ImageRgba8(imageproc::drawing::draw_line_segment(& self.image_back, (self.initial_pos.x, self.initial_pos.y), (m.x, m.y),image::Rgba(self.color.to_array()))),
                                                DrawingType::Rectangle=> self.image = DragApp::draw_rect(&self.image_back, self.initial_pos.x, self.initial_pos.y, m.x, m.y, image::Rgba(self.color.to_array())),
                                            }
                                            self.image_back= self.image.clone();
                                            self.drawing=false;
                                            self.drawing_type=DrawingType::None;
                                            self.initial_pos=egui::pos2(-1.0, -1.0);
                                        }
                                    },
                                }
                            }
                            else if self.initial_pos.x!= -1.0 && self.initial_pos.y!= -1.0{
                                match  i.pointer.interact_pos(){
                                    None => (),
                                    Some(mut m) =>{
                                        m = egui::pos2(m.x - image_w.rect.left_top().x, m.y - image_w.rect.left_top().y);
                                        match self.drawing_type{
                                            DrawingType::None=>(),
                                            DrawingType::Arrow=>self.image= DragApp::draw_arrow(&self.image_back, self.initial_pos.x, self.initial_pos.y, m.x, m.y, image::Rgba(self.color.to_array())),
                                            DrawingType::Circle=>self.image = image::DynamicImage::ImageRgba8(imageproc::drawing::draw_hollow_circle(& self.image_back, (self.initial_pos.x as i32, self.initial_pos.y as i32), m.distance(self.initial_pos) as i32, image::Rgba(self.color.to_array()))),
                                            DrawingType::Line=> self.image = image::DynamicImage::ImageRgba8(imageproc::drawing::draw_line_segment(& self.image_back, (self.initial_pos.x, self.initial_pos.y), (m.x, m.y),image::Rgba(self.color.to_array()))),
                                            DrawingType::Rectangle=> self.image = DragApp::draw_rect(&self.image_back, self.initial_pos.x, self.initial_pos.y, m.x, m.y, image::Rgba(self.color.to_array())),
                                        }
                                    },
                                }
                            }
                        }
                        else if self.crop==true{
                            if  self.initial_pos.x== -1.0 && self.initial_pos.y== -1.0 && i.pointer.button_clicked(egui::PointerButton::Primary){
                                match  i.pointer.interact_pos(){
                                    None => (),
                                    Some(mut m) =>{
                                        if m.x - image_w.rect.left_top().x >= 0.0 && m.x - image_w.rect.left_top().x <= image_w.rect.width() && m.y - image_w.rect.left_top().y >= 0.0 && m.y - image_w.rect.left_top().y <= image_w.rect.height(){
                                            m = egui::pos2(m.x - image_w.rect.left_top().x, m.y - image_w.rect.left_top().y);
                                            
                                            if m.distance(egui::pos2(self.crop_point.x0, self.crop_point.y0)) <= 20.0{
                                                self.current_crop_point= Corner::TopLeft;
                                                self.initial_pos= egui::pos2(self.crop_point.x1, self.crop_point.y1);
                                            }
                                            else if m.distance(egui::pos2(self.crop_point.x1 , self.crop_point.y0 )) <= 20.0{
                                                self.current_crop_point= Corner::TopRight;
                                                self.initial_pos= egui::pos2(self.crop_point.x0 , self.crop_point.y1 );
                                            }
                                            else if m.distance(egui::pos2(self.crop_point.x0 , self.crop_point.y1 )) <= 20.0{
                                                self.current_crop_point= Corner::BottomLeft;
                                                self.initial_pos= egui::pos2(self.crop_point.x1 , self.crop_point.y0 );
                                            }
                                            else if m.distance(egui::pos2(self.crop_point.x1 , self.crop_point.y1 )) <= 20.0{
                                                self.current_crop_point= Corner::BottomRight;
                                                self.initial_pos= egui::pos2(self.crop_point.x0 , self.crop_point.y0 );
                                            }
                                        }
                                    }
                                }
                            }
                            else if  self.initial_pos.x!= -1.0 && self.initial_pos.y!= -1.0 && i.pointer.button_clicked(egui::PointerButton::Primary){
                                match  i.pointer.interact_pos(){
                                    None => (),
                                    Some(mut m) =>{
                                        if m.x - image_w.rect.left_top().x >= 0.0 && m.x - image_w.rect.left_top().x <= image_w.rect.width() && m.y - image_w.rect.left_top().y >= 0.0 && m.y - image_w.rect.left_top().y <= image_w.rect.height(){
                                            m = egui::pos2(m.x - image_w.rect.left_top().x, m.y - image_w.rect.left_top().y);
                                            let p1= self.crop_point.x1 - cmp::max((self.crop_point.x1-m.x)as i32, 50) as f32;
                                            let p2= self.crop_point.y1 - cmp::max((self.crop_point.y1-m.y)as i32, 50) as f32;
                                            let p3= self.crop_point.x0 + cmp::max((m.x-self.crop_point.x0)as i32, 50) as f32;
                                            let p4= self.crop_point.y0 + cmp::max((m.y-self.crop_point.y0)as i32, 50) as f32;
                                            match self.current_crop_point {
                                                Corner::TopLeft=> self.crop_point= CropRect::new( p1, p2, self.crop_point.x1, self.crop_point.y1),
                                                Corner::TopRight=> self.crop_point= CropRect::new(self.crop_point.x0, p2, p3, self.crop_point.y1),
                                                Corner::BottomLeft=> self.crop_point= CropRect::new(p1, self.crop_point.y0, self.crop_point.x1, p4),
                                                Corner::BottomRight=> self.crop_point= CropRect::new(self.crop_point.x0, self.crop_point.y0, p3, p4),
                                                _=>(),
                                            }
                                            self.initial_pos=egui::pos2(-1.0, -1.0);
                                        }
                                    }
                                }
                            }
                            else if self.initial_pos.x!= -1.0 && self.initial_pos.y!= -1.0{
                                match  i.pointer.interact_pos(){
                                    None => (),
                                    Some(mut m) =>{
                                        m = egui::pos2(m.x - image_w.rect.left_top().x, m.y - image_w.rect.left_top().y);
                                        let p1= self.crop_point.x1 - cmp::max((self.crop_point.x1-m.x)as i32, 50) as f32;
                                            let p2= self.crop_point.y1 - cmp::max((self.crop_point.y1-m.y)as i32, 50) as f32;
                                            let p3= self.crop_point.x0 + cmp::max((m.x-self.crop_point.x0)as i32, 50) as f32;
                                            let p4= self.crop_point.y0 + cmp::max((m.y-self.crop_point.y0)as i32, 50) as f32;
                                            match self.current_crop_point {
                                                Corner::TopLeft=> self.crop_point= CropRect::new( p1, p2, self.crop_point.x1, self.crop_point.y1),
                                                Corner::TopRight=> self.crop_point= CropRect::new(self.crop_point.x0, p2, p3, self.crop_point.y1),
                                                Corner::BottomLeft=> self.crop_point= CropRect::new(p1, self.crop_point.y0, self.crop_point.x1, p4),
                                                Corner::BottomRight=> self.crop_point= CropRect::new(self.crop_point.x0, self.crop_point.y0, p3, p4),
                                                _=>(),
                                            }
                                        self.image = DragApp::draw_rect(&self.image_back, self.crop_point.x0, self.crop_point.y0, self.crop_point.x1, self.crop_point.y1, image::Rgba(epaint::Color32::DARK_GRAY.to_array()));
                                        self.image= DragApp::draw_rect(&self.image, self.crop_point.x0 +0.5, self.crop_point.y0+0.5, self.crop_point.x1-0.5, self.crop_point.y1-0.5, image::Rgba(epaint::Color32::DARK_GRAY.to_array()));
                                        self.image= DragApp::draw_rect(&self.image, self.crop_point.x0+1.0, self.crop_point.y0+1.0, self.crop_point.x1-1.0, self.crop_point.y1-1.0, image::Rgba(epaint::Color32::DARK_GRAY.to_array()));
                                        self.image= DragApp::draw_rect(&self.image, self.crop_point.x0+1.5, self.crop_point.y0+1.5, self.crop_point.x1-1.5, self.crop_point.y1-1.5, image::Rgba(epaint::Color32::DARK_GRAY.to_array()));
                                    },
                                }
                            }
                            else if self.initial_pos.x== -1.0 && self.initial_pos.y== -1.0 && i.pointer.button_double_clicked(egui::PointerButton::Primary){
                                match  i.pointer.interact_pos(){
                                    None => (),
                                    Some(m) =>{
                                        if m.x - image_w.rect.left_top().x >= 0.0 && m.x - image_w.rect.left_top().x <= image_w.rect.width() && m.y - image_w.rect.left_top().y >= 0.0 && m.y - image_w.rect.left_top().y <= image_w.rect.height(){
                                            self.image= image::DynamicImage::ImageRgba8(image::imageops::crop(&mut self.image_back.clone(), self.crop_point.x0 as u32,self.crop_point.y0 as u32, (self.crop_point.x1-self.crop_point.x0) as u32, (self.crop_point.y1-self.crop_point.y0) as u32).to_image());
                                            self.image= DragApp::draw_rect(&self.image, self.crop_point.x0 +0.5, self.crop_point.y0+0.5, self.crop_point.x1-0.5, self.crop_point.y1-0.5, image::Rgba(epaint::Color32::DARK_GRAY.to_array()));
                                            self.image= DragApp::draw_rect(&self.image, self.crop_point.x0+1.0, self.crop_point.y0+1.0, self.crop_point.x1-1.0, self.crop_point.y1-1.0, image::Rgba(epaint::Color32::DARK_GRAY.to_array()));
                                            self.image= DragApp::draw_rect(&self.image, self.crop_point.x0+1.5, self.crop_point.y0+1.5, self.crop_point.x1-1.5, self.crop_point.y1-1.5, image::Rgba(epaint::Color32::DARK_GRAY.to_array()));
                                            self.image_back= self.image.clone();
                                            self.crop=false;
                                            self.crop_point=CropRect::default();
                                            self.current_crop_point=Corner::None;
                                            self.initial_pos=egui::pos2(-1.0, -1.0);
                                        }
                                    }
                                }
                                
                            }
                        }
                        else if self.texting==true{
                            if self.initial_pos.x== -1.0 && self.initial_pos.y== -1.0 && i.pointer.button_clicked(egui::PointerButton::Primary){
                                match  i.pointer.interact_pos(){
                                    None => (),
                                    Some(m) =>{
                                        if m.x - image_w.rect.left_top().x >= 0.0 && m.x - image_w.rect.left_top().x <= image_w.rect.width() && m.y - image_w.rect.left_top().y >= 0.0 && m.y - image_w.rect.left_top().y <= image_w.rect.height(){
                                            self.initial_pos= egui::pos2(m.x - image_w.rect.left_top().x, m.y - image_w.rect.left_top().y);
                                        }
                                    }
                                }
                            }
                            else if self.initial_pos.x!= -1.0 && self.initial_pos.y!= -1.0 {
                                    
                                    for key in &self.all_keys{
                                        if i.consume_key(Modifiers::NONE, *key){
                                            if *key==Key::Backspace {
                                                self.text_string.pop();
                                            }
                                            else if *key==Key::Space{
                                                self.text_string.push_str(" ");
                                            }
                                            else if *key==Key::Enter{
                                                self.image=image::DynamicImage::ImageRgba8(imageproc::drawing::draw_text(&self.image_back,image::Rgba(self.color.to_array()) , self.initial_pos.x as i32, self.initial_pos.y as i32,rusttype::Scale { x: 30.0, y: 30.0 }, &arial, &self.text_string));
                                                self.image_back= self.image.clone();
                                                self.texting=false;
                                                self.text_string="".to_string();
                                                self.initial_pos=egui::pos2(-1.0, -1.0);
                                            }
                                            else{
                                                self.text_string.push(key.symbol_or_name().chars().next().unwrap());
                                            }
                                            self.image=image::DynamicImage::ImageRgba8(imageproc::drawing::draw_text(&self.image_back,image::Rgba(self.color.to_array()) , self.initial_pos.x as i32, self.initial_pos.y as i32,rusttype::Scale { x: 30.0, y: 30.0 }, &arial, &self.text_string));
                                        }
                                    }

                            }

                        }
                
                    });




                    ui.horizontal(|ui| {
                        if ui.button("Copy to clipboard").clicked() {
                            self.copy_to_clipboard();
                        }

                        if ui.button("Take another screenshot").clicked() {
                            self.mode="initial".to_string();
                        }

                        if ui.button("Save").clicked() {
                            self.mode= "saving".to_string();
                        }

                        if ui.button("Quit").clicked() {
                            std::process::exit(0);
                        }
                    });


                });

            });
            },
            "saving"=> {

                 CentralPanel::default().show(ctx, |ui| {

                    ui.heading("Choose a path, a name and a format for your screenshot");

                    ui.vertical(|ui| {
                        ui.horizontal(|ui| {
                            ui.label("Path: ");
                            ui.text_edit_singleline(&mut self.current_path);

                            if self.save_errors.0 == true
                            {
                                ui.label("Please insert a path");
                            }
                            else if self.save_errors.1 ==true {
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

                    if ui.button("Save").clicked() {

                        if self.save_errors.2 {
                            ui.label("The chosen path is not a directory or it is already a file");
                        }

                        match self.current_path.as_str() {

                            "" => {
                                self.save_errors.0 = true;
                                ()
                            },

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
                                }
                                else {

                                    if current_path.is_dir() == false || current_path.is_file() == true {
                                        self.save_errors.2 = true;
                                        ()
                                    }
                                    else {

                                        let res = self.save_image_to_disk(self.current_format.clone().as_str(), self.current_path.clone().as_str(), self.current_name.clone().as_str());
                                        match res {
                                            Ok(_) => {
                                                self.mode="saved".to_string();
                                            }
                                            Err(_) => {
                                                self.mode="error".to_string();
                                            }
                                        }

                                    }


                                }

                            }
                        }
                        


                    }

                });

            },
            "hotkey" => {

                let hotkeys : Vec<String> = vec!["Take a Screenshot".to_string(), "Quit".to_string()];

                CentralPanel::default().show(ctx, |ui| {
                    ui.vertical_centered(|ui| {

                        ui.heading("Hotkey Selection Screen");
                        ui.label("Select the hotkey you want to bind.\
                        You will have 3 seconds to choose the buttons");

                        for (i, hotkey) in hotkeys.iter().enumerate() {
                            ui.horizontal_wrapped(|ui| {

                                ui.label(hotkey);

                                // ui.add_enabled(self.hotkey_ui_status, );
                                ui.add_enabled_ui(self.hotkey_ui_status == false, |ui|{
                                    let button_text : String = if self.changing_hotkey[i] == true {"  ---  ".to_string()} else {self.hotkeys_strings[i].clone().to_string()};
                                    if ui.button(button_text).on_hover_text("Change hotkey").clicked(){
                                        self.hotkey_ui_status= true;
                                        self.changing_hotkey[i] = true;
                                    };
                                });


                            });
                        }

                        ctx.input(|i| if i.key_pressed(Key::Enter) {

                            let mut keys_pressed = i.keys_down.clone();
                            keys_pressed.remove(&Key::Enter);
                            println!("{:?}", keys_pressed );
                            if keys_pressed.len() != 0 {

                                let mut buf: String = "".to_string();
                                for (i,str_key) in keys_pressed.iter().enumerate() {
                                    if i==0 {buf = str_key.symbol_or_name().to_string() }
                                    else {
                                        buf = buf.to_string() + " + " + str_key.symbol_or_name();
                                    }
                                }

                                self.hotkeys_strings[self.changing_hotkey.iter().position(|&x| x == true).unwrap()] = buf;
                            }
                            self.hotkey_ui_status= false;
                            for changing_hotkey in self.changing_hotkey.iter_mut() {
                                *changing_hotkey = false;
                            }

                        }
                        );

                        ui.with_layout(Layout::right_to_left(Align::Max), |ui| {

                            if ui.button("Back").clicked() {
                                self.mode="initial".to_string();
                            }
                            if ui.button("Quit").clicked() {
                                //Routine per chiudere il programma
                                std::process::exit(0);
                            }

                        });

                    })
                });

            },
            "saved" => {
                CentralPanel::default().show(ctx, |ui| {
                    ui.heading("Screenshot saved!");
                    ui.label("Screenshot saved to disk");
                    if ui.button("Take another screenshot").clicked() {
                        self.mode="initial".to_string();
                    }
                    if ui.button("Quit").clicked() {
                        //Routine per chiudere il programma
                        std::process::exit(0);
                    }
                });
            },
            "error" => {
                CentralPanel::default().show(ctx, |ui| {
                    ui.heading("Error");
                    ui.label("Something went wrong");
                    if ui.button("Take another screenshot").clicked() {
                        self.mode="initial".to_string();
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


fn main() -> Result<(), eframe::Error>{
    let mut screen_sizes: [u32; 2] = [1920, 1080];

    for screen in Screen::all().unwrap().iter(){
        if screen.display_info.is_primary {
            screen_sizes[0] = screen.display_info.width;
            screen_sizes[1] = screen.display_info.height;

        }
    }
    
    let native_options = eframe::NativeOptions {
        always_on_top:false,
        resizable: true,
        follow_system_theme: true,
        centered: true,

        ..Default::default()
    };
    run_native("DragCapture", native_options, Box::new(|cc| Box::new(DragApp::new(cc))))



}