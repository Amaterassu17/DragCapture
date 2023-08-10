use std::borrow::Cow;
use std::time::Duration;
use eframe::{App, Frame, run_native, Storage, egui::CentralPanel, CreationContext};

use egui::{Context, Image, Rect, Visuals, Window, TextureHandle, TextureOptions, InputState};
use eframe::egui;

use imageproc::point::Point;
use screenshots::{Screen, Compression};
use screenshots;
use std::{fs};
use image::*;
use arboard::*;
use egui::Align::Center;
use epaint::ColorImage;
use image::ImageError::IoError;
use std::error::Error;
use std::path::Path;
use dirs;
use chrono;

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
    arrow: bool,
    initial_pos: egui::Pos2,
}

impl DragApp {
    pub fn new(cc: &CreationContext<'_>) -> Self {
        //Qua dobbiamo mettere il setup di eventuali font eccetera
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
            arrow: false,
            initial_pos: egui::pos2(-1.0, -1.0),

        }
    }

    // pub fn initiate_drag (&mut self, _ctx: &Context, _frame: &mut Frame, _id: egui::Id, _response: &mut egui::Response, _response_pos: egui::Pos2, _modifiers: egui::Modifiers) {
    //     //Qua ci sta la routine che toglie il focus dalla finestra e fa lo screenshot alla premuta di un pulsante o anche solo premendo solo questo pulsante. Va legato alla libreria screenshots
    //     let input = InputState::default();
    //
    //     println!("Drag initiated");
    //     println!("Mouse pos: {:?}", input);
    //
    // }


    // pub fn initiate_drag_simple (&mut self, _ctx: &Context, _frame: &mut Frame) {
    //     //Qua ci sta la routine che toglie il focus dalla finestra e fa lo screenshot alla premuta di un pulsante o anche solo premendo solo questo pulsante. Va legato alla libreria screenshots
    //     _frame.set_minimized(true);
    //
    //     let mut inserted_commands = Vec::new();
    //
    //     loop {
    //
    //         if(_ctx.input((|i| i.key_pressed(Key::)))) {
    //             println!("Mouse down");
    //             break;
    //         }
    //
    //         if(_ctx.input(())) {
    //             println!("Mouse up");
    //             break;
    //         }
    //
    //         if(_ctx.input(())) {
    //             println!("Mouse pressed");
    //             break;
    //         }
    //
    //     }
    //
    //     println!("End of loop");
    //
    //     //std::thread::sleep(Duration::from_secs(self.delay_timer as u64));
    //
    //     let input = _ctx.input(|i| i.clone());
    //
    //     println!("Drag initiated");
    //     println!("Mouse pos: {:?}", input);
    //
    //     //I want to start an input state that is a drag
    //     //First off, we see if any input
    //
    //
    //
    // }

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

        //Non sappiamo come gestire il fatto della sovrascrizione. Nel caso non è gestito

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
        if((x0-x1).abs() <1.0 || (y0-y1).abs() < 1.0){ 
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
}

impl App for DragApp {

    //UPDATE è FONDAMENTALE. CI DEVE ESSERE SEMPRE
    fn update(&mut self, ctx: &Context, frame: &mut Frame) {

        let red   = image::Rgba([255u8, 0u8,   0u8, 255u8]);
        let green = image::Rgba([0u8,   255u8, 0u8, 255u8]);
        let blue  = image::Rgba([0u8,   0u8,   255u8, 255u8]);
        let white = image::Rgba([255u8, 255u8, 255u8, 255u8]);
        let black = image::Rgba([0u8, 0u8, 0u8, 255u8]);


        let screens = Screen::all().unwrap();
        

        match self.mode.as_str() {
            "initial" => {

                CentralPanel::default().show(ctx, |ui| {
                    ui.heading("Hello World!");
                    ui.label("This is a test for egui and eframe");
                    //Button
                    if ui.button("Take a screenshot!").clicked() {
                        //Qua ci sta tipo la routine che toglie il focus dalla finestra e fa lo screenshot alla premuta di un pulsante o anche solo premendo solo questo pulsante. Va legato alla libreria screenshots
                        //let screens = Screen::all().unwrap();



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
                        self.image_back= self.image.clone();
                        self.mode="taken".to_string();
                    }
                    if ui.button("Customize Hotkeys").clicked() {
                        //ROUTINE PER CAMBIARE GLI HOTKEYS. deve essere tipo una sotto finestra da cui togli focus e non puoi ricliccare su quella originale finchè non chiudi la sottofinestra. Al massimo ci confrontiamo con alessio
                        //  println!("Hotkey Info: {:?}", self.hotkeys);
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

                    ui.vertical_centered(|ui| {
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

                    if ui.button("Quit").clicked() {
                        //Routine per chiudere il programma
                        std::process::exit(0);
                    }

                });


            },
             "taken" => {
                CentralPanel::default().show(ctx, |ui| {

                egui::containers::scroll_area::ScrollArea::vertical().show(ui, |ui| {

                    ui.heading("Screenshot taken!");
                    ui.label("Screenshot taken and copied to clipboard");
                    if ui.button("Take another screenshot").clicked() {
                        self.mode="initial".to_string();
                    }
                    if ui.button("Quit").clicked() {
                        //Routine per chiudere il programma
                        std::process::exit(0);
                    }
                    if ui.button("Arrow").clicked() {
                        self.arrow=true;

                    }
                    if ui.button("Circle").clicked() {
                        self.image = image::DynamicImage::ImageRgba8(imageproc::drawing::draw_hollow_circle(&mut self.image, (100, 100), 10, red));
                    }
                    if ui.button("Line").clicked() {
                        self.image = image::DynamicImage::ImageRgba8(imageproc::drawing::draw_line_segment(&mut self.image, (100.0, 100.0), (110.0, 110.0),black ));

                    }if ui.button("Rectangle").clicked() {
                        self.image = image::DynamicImage::ImageRgba8(imageproc::drawing::draw_hollow_rect(&mut self.image,  imageproc::rect::Rect::at(1, 1).of_size(200, 200), white));

                }
                if ui.button("Crop").clicked() {
                    
                    self.image= image::DynamicImage::ImageRgba8(image::imageops::crop(&mut self.image.clone(), 0,0, 600, 20).to_image());
                    
                }
                // let image_buffer= ImageBuffer::from_raw(self.image.1, self.image.2, self.image.0.clone()).unwrap().save("target/screenshot.png").unwrap();

                // let texture : TextureHandle = ui.ctx().load_texture("Screenshot", self.image.0.clone(), TextureOptions::default());

                // ui.image(texture, texture.size_vec2());



                    let color_image = DragApp::load_image_from_memory(self.image.clone()).unwrap();
                    self.current_width= color_image.size[0] as i32;
                    self.current_height= color_image.size[1] as i32;
                    let texture = ui.ctx().load_texture("ScreenShot", color_image, TextureOptions::default());

                    let image_w = ui.image(&texture, texture.size_vec2());

                    ctx.input(|i|{ 
                        if self.initial_pos.x== -1.0 && self.initial_pos.y== -1.0 && self.arrow==true && i.pointer.button_clicked(egui::PointerButton::Primary){
                            match  i.pointer.interact_pos(){
                                None => (),
                                Some(m) => self.initial_pos= egui::pos2(m.x - image_w.rect.left_top().x, m.y - image_w.rect.left_top().y),
                            }
                        }
                        else if self.initial_pos.x!= -1.0 && self.initial_pos.y!= -1.0 && self.arrow==true && i.pointer.button_clicked(egui::PointerButton::Primary){
                            match  i.pointer.interact_pos(){
                                None => (),
                                Some(mut m) => {
                                    m = egui::pos2(m.x - image_w.rect.left_top().x, m.y - image_w.rect.left_top().y);
                                    self.image= DragApp::draw_arrow(&self.image_back, self.initial_pos.x, self.initial_pos.y, m.x, m.y, green); 
                                    self.image_back= self.image.clone();
                                    self.arrow=false;
                                    self.initial_pos=egui::pos2(-1.0, -1.0);
                                },
                            }
                        }
                        else if self.initial_pos.x!= -1.0 && self.initial_pos.y!= -1.0 && self.arrow==true{
                            match  i.pointer.interact_pos(){
                                None => (),
                                Some(mut m) =>{ 
                                    m = egui::pos2(m.x - image_w.rect.left_top().x, m.y - image_w.rect.left_top().y);
                                    self.image= DragApp::draw_arrow(&self.image_back, self.initial_pos.x, self.initial_pos.y, m.x, m.y, green)},
                            }
                        }
                        
                    });


                    if ui.button("Copy to clipboard").clicked() {
                        //Routine per copiare l'immagine negli appunti

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

                    if ui.button("Save").clicked() {
                        self.mode= "saving".to_string();
                    }

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
                                println!("{:?}", current_path);

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
            "hotkey" => {},
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



    // DA QUI IN POI SONO TUTTE OPZIONALI. NON DOVREBBE SERVIRE IMPLEMENTARLE A MENO DI COSE SPECIFICHE TIPO HOTKEY BOH LA SPARO A CASO
    // fn save(&mut self, _storage: &mut dyn Storage) {
    //     todo!()
    // }
    //
    // fn on_close_event(&mut self) -> bool {
    //     todo!()
    // }
    //
    // fn on_exit(&mut self, _gl: Option<&eframe::glow::Context>) {
    //     todo!()
    // }
    //
    // fn auto_save_interval(&self) -> Duration {
    //     todo!()
    // }
    //
    // fn max_size_points(&self) -> Vec2 {
    //     todo!()
    // }
    //
    // fn clear_color(&self, _visuals: &Visuals) -> [f32; 4] {
    //     todo!()
    // }
    //
    // fn persist_native_window(&self) -> bool {
    //     todo!()
    // }
    //
    // fn persist_egui_memory(&self) -> bool {
    //     todo!()
    // }
    //
    // fn warm_up_enabled(&self) -> bool {
    //     todo!()
    // }
    //
    // fn post_rendering(&mut self, _window_size_px: [u32; 2], _frame: &Frame) {
    //     todo!()
    // }
}

fn main() -> Result<(), eframe::Error>{
    //Test for egui and eframe

    let mut screen_sizes: [u32; 2] = [1920, 1080];

    for screen in Screen::all().unwrap().iter(){
        if screen.display_info.is_primary {
            screen_sizes[0] = screen.display_info.width;
            screen_sizes[1] = screen.display_info.height;

        }
    }

    let native_options = eframe::NativeOptions {
        initial_window_size: Some(egui::Vec2::new((screen_sizes[0] as f32/ 1.5), (screen_sizes[1] as f32/ 1.5))),
        always_on_top:false,
        resizable: true,
        follow_system_theme: true,
        centered: true
        ,
        ..Default::default()
    };
    run_native("DragCapture", native_options, Box::new(|cc| Box::new(DragApp::new(cc))))



}