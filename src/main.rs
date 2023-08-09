use std::borrow::Cow;
use std::time::Duration;
use eframe::{App, Frame, run_native, Storage, egui::CentralPanel, CreationContext};
use eframe::emath::Vec2;
use egui;
use egui::{Context, Image, Rect, Visuals, Window};
use egui_extras::RetainedImage;
use screenshots::{Screen, Compression};
use std::{fs};
use image::*;
use arboard::*;
struct DragApp {
    button_text1: String,
    delay_timer: u32,
    selected_monitor: u32,
    screenshot_taken: bool,
    image: egui::ImageData,
}

struct Captured_Image {
    texture : Option<egui::TextureHandle>
}

impl Captured_Image {
    fn ui (&mut self, ui: &mut eframe::egui::Ui) {
        if let Some(texture) = &self.texture {
            ui.image(texture, Vec2::new(300.0, 300.0));
        }
    }
}

impl DragApp {
    pub fn new(cc: &CreationContext<'_>) -> Self {
        //Qua dobbiamo mettere il setup di eventuali font eccetera
        Self {
            button_text1: "Take a screenshot!".to_owned(),
            delay_timer: 0,
            selected_monitor: 0,
            screenshot_taken: false,
            image:// Da definire
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

}
impl App for DragApp {

    //UPDATE è FONDAMENTALE. CI DEVE ESSERE SEMPRE
    fn update(&mut self, ctx: &Context, frame: &mut Frame) {

        let screens = Screen::all().unwrap();

        if self.screenshot_taken==false {
            CentralPanel::default().show(ctx, |ui| {
                ui.heading("Hello World!");
                ui.label("This is a test for egui and eframe");
                //Button
                if ui.button("Take a screenshot!").clicked() {
                    //Qua ci sta tipo la routine che toglie il focus dalla finestra e fa lo screenshot alla premuta di un pulsante o anche solo premendo solo questo pulsante. Va legato alla libreria screenshots
                    //let screens = Screen::all().unwrap();

                let x= 300;
                let y = 300;
                let width= 300;
                let height=300;

                let mut selected_screens = Vec::new();
                for (i, screen) in screens.iter().enumerate() {
                    if self.selected_monitors[i] {
                        selected_screens.push(screen);
                    }
                }
                
                for (i, screen) in selected_screens.iter().enumerate() {
                    let image = screen.capture_area(x, y, width, height).unwrap();

                    let buffer = image.to_png(Compression::Fast).unwrap();
                    let img=  image::load_from_memory_with_format(&buffer, image::ImageFormat::Png).unwrap();




                    img.save(format!("target/{}.png", selected_screen.display_info.id)).expect("Error");
                    // img.save(format!("target/{}.jpg", screen.display_info.id)).expect("Error");
                    // img.save(format!("target/{}.gif", screen.display_info.id)).expect("Error");
                    let mut clipboard = Clipboard::new().unwrap();
                    let r=img.resize(width, height, imageops::FilterType::Lanczos3).into_rgba8();
                    let (w,h)=r.dimensions();
                    let img = ImageData {
                        width: usize::try_from(w).unwrap(),
                        height: usize::try_from(h).unwrap(),
                        bytes: Cow::from(r.as_bytes())
                    };

                    clipboard.set_image(img);




                }
                if ui.button("Customize Hotkeys").clicked() {
                    //ROUTINE PER CAMBIARE GLI HOTKEYS. deve essere tipo una sotto finestra da cui togli focus e non puoi ricliccare su quella originale finchè non chiudi la sottofinestra. Al massimo ci confrontiamo con alessio
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
        }
        else {
            CentralPanel::default().show(ctx, |ui| {
                ui.heading("Screenshot taken!");
                ui.label("Screenshot taken and copied to clipboard");
                if ui.button("Take another screenshot").clicked() {
                    self.screenshot_taken=false;
                }
                if ui.button("Quit").clicked() {
                    //Routine per chiudere il programma
                    std::process::exit(0);
                }

                let texture = ui.ctx().load_texture("Screenshot", &self.image, Default::default());

                ui.image(texture, texture.size_vec2())



            });
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

    let native_options = eframe::NativeOptions {
        initial_window_size: Some(egui::Vec2::new(400.0, 400.0)),
        ..Default::default()
    };
    run_native("DragCapture", native_options, Box::new(|cc| Box::new(DragApp::new(cc))))



}