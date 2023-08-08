use std::time::Duration;
use eframe::{App, Frame, run_native, Storage, egui::CentralPanel, CreationContext};
use eframe::emath::Vec2;
use egui;
use egui::{Context, Visuals};
use screenshots::{Screen, Compression};
use std::{fs};

struct DragApp {
    button_text1: String,
    delay_timer: u32,
}

impl DragApp {
    pub fn new(cc: &CreationContext<'_>) -> Self {
        //Qua dobbiamo mettere il setup di eventuali font eccetera
        Self {
            button_text1: "Take a screenshot!".to_owned(),
            delay_timer: 0,
        }
    }


}
impl App for DragApp {

    //UPDATE è FONDAMENTALE. CI DEVE ESSERE SEMPRE
    fn update(&mut self, ctx: &Context, frame: &mut Frame) {
        CentralPanel::default().show(ctx, |ui| {
            ui.heading("Hello World!");
            ui.label("This is a test for egui and eframe");
            //Button
            if ui.button("Take a screenshot!").clicked() {
                //Qua ci sta tipo la routine che toglie il focus dalla finestra e fa lo screenshot alla premuta di un pulsante o anche solo premendo solo questo pulsante. Va legato alla libreria screenshots
                let screens = Screen::all().unwrap();

                for screen in screens {

                    let image = screen.capture_area(300, 300, 300, 300).unwrap();
                    let buffer = image.to_png(None).unwrap();
                    fs::write(format!("target/{}.png", screen.display_info.id), buffer).unwrap();
                }

                
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
            if ui.button("Quit").clicked() {
                //Routine per chiudere il programma
                std::process::exit(0);
            }

        });
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
