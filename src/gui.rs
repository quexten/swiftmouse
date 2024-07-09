use std::{os::unix::process, sync::Arc};

use eframe::egui::{self, InputState};

use crate::autotype::{self, ClickType};

pub(crate) fn show_gui(mut positions: Vec<(u32, u32, u32, u32)>, width: u32, height: u32, path: String) {
    let mut options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([400.0, 800.0]),
        ..Default::default()
    };
    options.viewport.fullscreen = Some(true);
    // options.viewport.maximized = Some(true);
    let heap_width = Arc::new(width);
    let heap_height = Arc::new(height);
    eframe::run_native(
        "Image Viewer",
        options,
        Box::new(|cc| {
            // This gives us image support:
            egui_extras::install_image_loaders(&cc.egui_ctx);
            let mut app = Box::<MyApp>::default();
            app.positions = positions;
            app.width = Arc::try_unwrap(heap_width).unwrap();
            app.height = Arc::try_unwrap(heap_height).unwrap();
            app.path = path;
            app.letters_typed = vec![];
            Ok(app)
        }),
    );
}

#[derive(Default)]
struct MyApp {
    positions: Vec<(u32, u32, u32, u32)>,
    letters_typed: Vec<u32>,
    width: u32,
    height: u32,
    path: String,
}

fn get_key(i: &InputState) -> Option<i32> {
    if i.key_pressed(egui::Key::A) {
        return Some(0);
    } else if i.key_pressed(egui::Key::B) {
        return Some(1);
    } else if i.key_pressed(egui::Key::C) {
        return Some(2);
    } else if i.key_pressed(egui::Key::D) {
        return Some(3);
    } else if i.key_pressed(egui::Key::E) {
        return Some(4);
    } else if i.key_pressed(egui::Key::F) {
        return Some(5);
    } else if i.key_pressed(egui::Key::G) {
        return Some(6);
    } else if i.key_pressed(egui::Key::H) {
        return Some(7);
    } else if i.key_pressed(egui::Key::I) {
        return Some(8);
    } else if i.key_pressed(egui::Key::J) {
        return Some(9);
    } else if i.key_pressed(egui::Key::K) {
        return Some(10);
    } else if i.key_pressed(egui::Key::L) {
        return Some(11);
    } else if i.key_pressed(egui::Key::M) {
        return Some(12);
    } else if i.key_pressed(egui::Key::N) {
        return Some(13);
    } else if i.key_pressed(egui::Key::O) {
        return Some(14);
    } else if i.key_pressed(egui::Key::P) {
        return Some(15);
    } else if i.key_pressed(egui::Key::Q) {
        return Some(16);
    } else if i.key_pressed(egui::Key::R) {
        return Some(17);
    } else if i.key_pressed(egui::Key::S) {
        return Some(18);
    } else if i.key_pressed(egui::Key::T) {
        return Some(19);
    } else if i.key_pressed(egui::Key::U) {
        return Some(20);
    } else if i.key_pressed(egui::Key::V) {
        return Some(21);
    } else if i.key_pressed(egui::Key::W) {
        return Some(22);
    } else if i.key_pressed(egui::Key::X) {
        return Some(23);
    } else if i.key_pressed(egui::Key::Y) {
        return Some(24);
    } else if i.key_pressed(egui::Key::Z) {
        return Some(25);
    }
    return None;
}

fn get_letters_for_index(index: i32) -> (u8, u8) {
    let letter1 = (index / 26) as u8;
    let letter2 = (index % 26) as u8;
    (letter1, letter2)
}

fn get_index_for_letters(letter1: u8, letter2: u8) -> i32 {
    let letter1 = letter1 as i32;
    let letter2 = letter2 as i32;
    letter1 * 26 + letter2
}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let mut key_to_click:Option<ClickType> = None;

        ctx.input(|i| {
            if i.key_pressed(egui::Key::Escape) {
                std::process::exit(0);
            }

            if i.key_pressed(egui::Key::Backspace) {
                self.letters_typed.pop();
            }

            if i.key_pressed(egui::Key::Num1)
                || i.key_pressed(egui::Key::Enter)
                || i.key_pressed(egui::Key::Num2)
                || i.key_pressed(egui::Key::Num3)
                || i.key_pressed(egui::Key::Num4) {

                if i.key_pressed(egui::Key::Num1) || i.key_pressed(egui::Key::Enter) {
                    key_to_click = Some(ClickType::Left);
                } else if i.key_pressed(egui::Key::Num2) {
                    key_to_click = Some(ClickType::Right);
                } else if i.key_pressed(egui::Key::Num3) {
                    key_to_click = Some(ClickType::Middle);
                } else {
                    key_to_click = Some(ClickType::Double);
                }
            }
            let key = get_key(i);
            match key {
                Some(key) => {
                    println!("Key pressed: {:?}", key);
                    self.letters_typed.append(&mut vec![key as u32]);
                }
                None => {}
            }
        });

        let frame = egui::Frame::default().fill(egui::Color32::from_rgb(0, 0, 0)).inner_margin(0.0);
        egui::CentralPanel::default().frame(frame).show(ctx, |ui| {
            if key_to_click.is_some() {
                ctx.send_viewport_cmd(egui::ViewportCommand::Focus);
                let click_type = key_to_click.unwrap();
                tokio::spawn(async move {
                    autotype::click(click_type).await;
                    std::process::exit(0);
                });
            }

            if self.letters_typed.len() == 2 {
                let index = get_index_for_letters(self.letters_typed[0] as u8, self.letters_typed[1] as u8);
                // ctx.send_viewport_cmd(egui::ViewportCommand::Minimized(true));
                let (min_x, min_y, max_x, max_y) = self.positions[index as usize];
                let width = self.width;
                let height = self.height;
                tokio::spawn(async move {
                    // click to center of box
                    let click_x = (min_x + max_x) / 2;
                    let click_y = (min_y + max_y) / 2;

                    autotype::movemouse(click_x as i32, click_y as i32, width as i32, height as i32).await.unwrap();
                });
    
            }

            ui.add(
                egui::Image::new("file://".to_owned() + &self.path)
            );

            for ((min_x, min_y, max_x, max_y), index) in self.positions.iter().zip(0..) {
                let (letter1, letter2) = get_letters_for_index(index);
                let letter1char = std::char::from_u32(letter1 as u32 + 65).unwrap();
                let letter2char = std::char::from_u32(letter2 as u32 + 65).unwrap();

                if self.letters_typed.len() == 0 || self.letters_typed.len() > 0 && self.letters_typed[0] as u8 == letter1 {
                    // color magenta if letter2 matches
                    let color = if self.letters_typed.len() == 2 && self.letters_typed[1] as u8 == letter2 {
                        egui::Color32::from_rgb(200, 0, 100)
                    } else {
                        if self.letters_typed.len() == 2 {
                            egui::Color32::from_rgb(100, 100, 100)
                        } else {
                            egui::Color32::from_rgb(0, 150, 150)
                        }
                    };
                    ui.painter().rect_stroke(
                        egui::Rect::from_min_max(
                            egui::pos2(*min_x as f32, *min_y as f32),
                            egui::pos2(*max_x as f32, *max_y as f32),
                        ),
                        0.0,
                        egui::Stroke::new(1.0, color),
                    );
                    
                    ui.allocate_ui_at_rect(egui::Rect::from_min_max(
                        egui::pos2(*min_x as f32, *min_y as f32),
                        egui::pos2((*min_x + 50) as f32, (*min_y + 50) as f32),
                    ), |ui| {
                        ui.label(egui::RichText::new(format!("{}{}", letter1char, letter2char)).heading().color(egui::Color32::from_rgb(255, 255, 255)).background_color(color));
                    });
                }

                ui.allocate_ui_at_rect(egui::Rect::from_min_size(
                    egui::pos2(0.0, 0.0),
                    egui::vec2(200.0, 100.0),
                ), |ui| {
                    // combine vec
                    let letters = self.letters_typed.iter().map(|x| std::char::from_u32(*x as u32 + 65).unwrap()).collect::<String>();

                    ui.label(egui::RichText::new(letters).heading().color(egui::Color32::from_rgb(255, 255, 255)).background_color(egui::Color32::from_rgb(50, 100, 200)).size(40.0));
                });
            }
     });
    }
}
