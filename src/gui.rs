use std::sync::Arc;

use eframe::egui::{self, InputState};

use crate::autotype::{self, ClickType};

static COLOR_GRAY: egui::Color32 = egui::Color32::from_rgb(100, 100, 100);

pub fn show_gui(big_boxes: Vec<(u32, u32, u32, u32)>, line_boxes: Vec<(u32, u32, u32, u32)>, small_images: Vec<(u32, u32, u32, u32)>, large_images: Vec<(u32, u32, u32, u32)>, path: String) {
    let mut options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([400.0, 800.0]),
        ..Default::default()
    };
    options.viewport.fullscreen = Some(true);
    let _ = eframe::run_native(
        "Swiftmouse",
        options,
        Box::new(|cc| {
            egui_extras::install_image_loaders(&cc.egui_ctx);
            let mut app = Box::<MyApp>::default();
            app.line_boxes = line_boxes;
            app.big_boxes = big_boxes;
            app.small_images = small_images;
            app.large_images = large_images;
            app.path = path;
            app.letters_typed = vec![];
            Ok(app)
        }),
    );
}

#[derive(Default)]
struct MyApp {
    line_boxes: Vec<(u32, u32, u32, u32)>,
    big_boxes: Vec<(u32, u32, u32, u32)>,
    small_images: Vec<(u32, u32, u32, u32)>,
    large_images: Vec<(u32, u32, u32, u32)>,
    letters_typed: Vec<u32>,
    selected_box: Option<(u32, u32, u32, u32)>,
    path: String,
}

fn get_key(i: &InputState) -> Option<i32> {
    if i.key_released(egui::Key::A) {
        return Some(0);
    } else if i.key_released(egui::Key::B) {
        return Some(1);
    } else if i.key_released(egui::Key::C) {
        return Some(2);
    } else if i.key_released(egui::Key::D) {
        return Some(3);
    } else if i.key_released(egui::Key::E) {
        return Some(4);
    } else if i.key_released(egui::Key::F) {
        return Some(5);
    } else if i.key_released(egui::Key::G) {
        return Some(6);
    } else if i.key_released(egui::Key::H) {
        return Some(7);
    } else if i.key_released(egui::Key::I) {
        return Some(8);
    } else if i.key_released(egui::Key::J) {
        return Some(9);
    } else if i.key_released(egui::Key::K) {
        return Some(10);
    } else if i.key_released(egui::Key::L) {
        return Some(11);
    } else if i.key_released(egui::Key::M) {
        return Some(12);
    } else if i.key_released(egui::Key::N) {
        return Some(13);
    } else if i.key_released(egui::Key::O) {
        return Some(14);
    } else if i.key_released(egui::Key::P) {
        return Some(15);
    } else if i.key_released(egui::Key::Q) {
        return Some(16);
    } else if i.key_released(egui::Key::R) {
        return Some(17);
    } else if i.key_released(egui::Key::S) {
        return Some(18);
    } else if i.key_released(egui::Key::T) {
        return Some(19);
    } else if i.key_released(egui::Key::U) {
        return Some(20);
    } else if i.key_released(egui::Key::V) {
        return Some(21);
    } else if i.key_released(egui::Key::W) {
        return Some(22);
    } else if i.key_released(egui::Key::X) {
        return Some(23);
    } else if i.key_released(egui::Key::Y) {
        return Some(24);
    } else if i.key_released(egui::Key::Z) {
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

impl MyApp {
    fn draw_images(&mut self, ui: &mut egui::Ui) {
        // if first letter is i
        let is_selected = self.letters_typed.len() > 0 && self.letters_typed[0] as u8 == 8;
        let is_other_selected = self.letters_typed.len() > 0 && self.letters_typed[0] as u8 != 8;
        if is_other_selected {
            return
        }

        let image_color = egui::Color32::from_rgb(255, 160, 50);

        for (index, (start_x, start_y, end_x, end_y)) in self.small_images.iter().enumerate() {
            let (letter1, letter2) = get_letters_for_index(index as i32);
            if self.letters_typed.len() > 1 {
                if self.letters_typed[1] as u8 != letter1 {
                    continue
                }
            }
            if self.letters_typed.len() > 2 {
                if self.letters_typed[2] as u8 != letter2 {
                    continue
                } else {
                    self.selected_box = Some((*start_x, *start_y, *end_x, *end_y));
                    continue
                }
            } 
            if self.letters_typed.len() == 2 {
                self.selected_box = None
            }

            ui.painter().rect_stroke(
                egui::Rect::from_min_max(
                    egui::pos2(*start_x as f32, *start_y as f32),
                    egui::pos2(*end_x as f32, *end_y as f32),
                ),
                0.0,
                egui::Stroke::new(2.0, image_color),
            );
            // if is_selected {
                let label = format!("{}{}", std::char::from_u32(letter1 as u32 + 65).unwrap(), std::char::from_u32(letter2 as u32 + 65).unwrap());                
                ui.allocate_ui_at_rect(egui::Rect::from_min_max(
                    egui::pos2(*start_x as f32, *start_y as f32),
                    egui::pos2(*start_x as f32 + 100.0, *end_y as f32 + 100.0),
                ), |ui| {
                    ui.label(egui::RichText::new(label).heading().color(egui::Color32::from_rgb(255, 255, 255)).background_color(image_color));
                });
            // }
        }
    }

    fn draw_text_lines(&mut self, ui: &mut egui::Ui) {
        let image_color = egui::Color32::from_rgb(150, 200, 20);
        // let is_selected = self.letters_typed.len() > 0 && self.letters_typed[0] as u8 == 11;
        let is_other_selected = self.letters_typed.len() > 0 && self.letters_typed[0] as u8 != 11;
        if is_other_selected {
            return
        }

        for (index, (start_x, start_y, end_x, end_y)) in self.line_boxes.iter().enumerate() {   
            let (letter1, letter2) = get_letters_for_index(index as i32);
            if self.letters_typed.len() > 1 {
                if self.letters_typed[1] as u8 != letter1 {
                    continue
                }
            }
            if self.letters_typed.len() > 2 {
                if self.letters_typed[2] as u8 != letter2 {
                    continue
                } else {
                    self.selected_box = Some((*start_x, *start_y, *end_x, *end_y));
                    continue
                }
            }
            if self.letters_typed.len() == 2 {
                self.selected_box = None
            }

            ui.painter().rect_stroke(
                egui::Rect::from_min_max(
                    egui::pos2(*start_x as f32, *start_y as f32),
                    egui::pos2(*end_x as f32, *end_y as f32),
                ),
                0.0,
                egui::Stroke::new(2.0, image_color),
            );
            // if is_selected {
                let label = format!("{}{}", std::char::from_u32(letter1 as u32 + 65).unwrap(), std::char::from_u32(letter2 as u32 + 65).unwrap());
                ui.allocate_ui_at_rect(egui::Rect::from_min_max(
                    egui::pos2(*start_x as f32, *start_y as f32),
                    egui::pos2(*start_x as f32 + 100.0, *end_y as f32 + 100.0),
                ), |ui| {
                    ui.label(egui::RichText::new(label).heading().color(egui::Color32::from_rgb(255, 255, 255)).background_color(image_color));
                });
            // }
        }
    }

    fn draw_big_boxes(&mut self, ui: &mut egui::Ui) {
        let image_color = egui::Color32::from_rgb(150, 0, 150);
        let is_selected = self.letters_typed.len() > 0 && self.letters_typed[0] as u8 == 1;
        let is_other_selected = self.letters_typed.len() > 0 && self.letters_typed[0] as u8 != 1;
        if is_other_selected {
            return
        }

        for (index, (start_x, start_y, end_x, end_y)) in self.big_boxes.iter().enumerate() {
            let (letter1, letter2) = get_letters_for_index(index as i32);
            if self.letters_typed.len() > 1 {
                if self.letters_typed[1] as u8 != letter1 {
                    continue
                }
            }
            if self.letters_typed.len() > 2 {
                if self.letters_typed[2] as u8 != letter2 {
                    continue
                } else {
                    self.selected_box = Some((*start_x, *start_y, *end_x, *end_y));
                    continue
                }
            }
            if self.letters_typed.len() == 2 {
                self.selected_box = None
            }

            ui.painter().rect_stroke(
                egui::Rect::from_min_max(
                    egui::pos2(*start_x as f32, *start_y as f32),
                    egui::pos2(*end_x as f32, *end_y as f32),
                ),
                0.0,
                egui::Stroke::new(2.0, image_color),
            );
            // if is_selected {
                let label = format!("{}{}", std::char::from_u32(letter1 as u32 + 65).unwrap(), std::char::from_u32(letter2 as u32 + 65).unwrap());
                ui.allocate_ui_at_rect(egui::Rect::from_min_max(
                    egui::pos2(*start_x as f32, *start_y as f32),
                    egui::pos2(*start_x as f32 + 100.0, *end_y as f32 + 100.0),
                ), |ui| {
                    ui.label(egui::RichText::new(label).heading().color(egui::Color32::from_rgb(255, 255, 255)).background_color(image_color));
                });
            // }
        }
    }
}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let mut key_to_click:Option<ClickType> = None;
        let mut close = false;
        let mut width = 0;
        let mut height = 0;

        ctx.input(|i| {
            width = i.screen_rect().width() as i32;
            height = i.screen_rect().height() as i32;
            if i.key_released(egui::Key::Escape) {
                close = true;
            }

            if i.key_released(egui::Key::Backspace) {
                self.letters_typed.pop();
            }

            if i.key_released(egui::Key::Num1)
                || i.key_released(egui::Key::Enter)
                || i.key_released(egui::Key::Num2)
                || i.key_released(egui::Key::Num3)
                || i.key_released(egui::Key::Num4) {

                if i.key_released(egui::Key::Num1) || i.key_released(egui::Key::Enter) {
                    key_to_click = Some(ClickType::Left);
                }
                // else if i.key_pressed(egui::Key::Num2) {
                //     key_to_click = Some(ClickType::Right);
                // } else if i.key_pressed(egui::Key::Num3) {
                //     key_to_click = Some(ClickType::Middle);
                // } else {
                //     key_to_click = Some(ClickType::Double);
                // }
            }
            let key = get_key(i);
            match key {
                Some(key) => {
                    println!("Key pressed: {:?}", key);
                    // max len 3
                    if self.letters_typed.len() < 3 {
                        self.letters_typed.append(&mut vec![key as u32]);
                    }
                }
                None => {}
            }
        });

        let frame = egui::Frame::default().fill(egui::Color32::from_rgb(0, 0, 0)).inner_margin(0.0);
        egui::CentralPanel::default().frame(frame).show(ctx, |ui| {
            if key_to_click.is_some() {
                ctx.send_viewport_cmd(egui::ViewportCommand::Minimized(true));
                let click_type = key_to_click.unwrap();
                tokio::spawn(async move {
                    // sleep
                    tokio::time::sleep(std::time::Duration::from_millis(50)).await;
                    autotype::click(click_type).await;
                    std::process::exit(0);
                });
            }
            if close {
                ctx.send_viewport_cmd(egui::ViewportCommand::Close);
            }

        //     if self.letters_typed.len() == 2 {
        //         let index = get_index_for_letters(self.letters_typed[0] as u8, self.letters_typed[1] as u8);
        //         // ctx.send_viewport_cmd(egui::ViewportCommand::Minimized(true));
        //         let (min_x, min_y, max_x, max_y) = if index < self.line_boxes.len() as i32 {
        //             self.line_boxes[index as usize]
        //         } else {
        //             self.big_boxes[(index - self.line_boxes.len() as i32) as usize]
        //         };
        //         let width = self.width;
        //         let height = self.height;
        //         tokio::spawn(async move {
        //             // click to center of box
        //             let click_x = (min_x + max_x) / 2;
        //             let click_y = (min_y + max_y) / 2;

        //             autotype::movemouse(click_x as i32, click_y as i32, width as i32, height as i32).await.unwrap();
        //         });
    
        //     }

            ui.add(
                egui::Image::new("file://".to_owned() + &self.path)
            );

            self.draw_text_lines(ui);
            self.draw_big_boxes(ui);
            self.draw_images(ui);
            if self.selected_box.is_some() {
                let (min_x, min_y, max_x, max_y) = self.selected_box.unwrap();
                
                // draw half transparent box
                ui.painter().rect(
                    egui::Rect::from_min_max(
                        egui::pos2(min_x as f32, min_y as f32),
                        egui::pos2(max_x as f32, max_y as f32),
                    ),
                    0.0,
                    egui::Color32::from_rgba_premultiplied(255, 0, 150, 5),
                    egui::Stroke::default()
                );
                ui.painter().rect_stroke(
                    egui::Rect::from_min_max(
                        egui::pos2(min_x as f32, min_y as f32),
                        egui::pos2(max_x as f32, max_y as f32),
                    ),
                    0.0,
                    egui::Stroke::new(4.0, egui::Color32::from_rgb(255, 255, 255)),
                );

                tokio::spawn(async move {
                    // click to center of box
                    let click_x = (min_x + max_x) / 2;
                    let click_y = (min_y + max_y) / 2;

                    autotype::movemouse(click_x as i32, click_y as i32, width as i32, height as i32).await.unwrap();
                });
            }


        //     for ((min_x, min_y, max_x, max_y), index) in self.large_images.iter().zip(0..) {
        //         let box_index = index as u32 + self.line_boxes.len() as u32;
        //         let (letter1, letter2) = get_letters_for_index(box_index as i32);
        //         let letter1char = std::char::from_u32(letter1 as u32 + 65).unwrap();
        //         let letter2char = std::char::from_u32(letter2 as u32 + 65).unwrap();

        //         if self.letters_typed.len() == 0 || self.letters_typed.len() > 0 && self.letters_typed[0] as u8 == letter1 {
        //             // color magenta if letter2 matches
        //             let color = if self.letters_typed.len() == 2 && self.letters_typed[1] as u8 == letter2 {
        //                 // draw half transparent box
        //                 ui.painter().rect(
        //                     egui::Rect::from_min_max(
        //                         egui::pos2(*min_x as f32, *min_y as f32),
        //                         egui::pos2(*max_x as f32, *max_y as f32),
        //                     ),
        //                     0.0,
        //                     egui::Color32::from_rgba_premultiplied(200, 0, 100, 10),
        //                     egui::Stroke::default()
        //                 );
        //                 egui::Color32::from_rgb(200, 0, 100)
        //             } else {
        //                 if self.letters_typed.len() == 2 {
        //                     egui::Color32::from_rgb(100, 100, 100)
        //                 } else {
        //                     egui::Color32::from_rgb(130, 50, 250)
        //                 }
        //             };
        //            ui.painter().rect_stroke(
        //                 egui::Rect::from_min_max(
        //                     egui::pos2(*min_x as f32, *min_y as f32),
        //                     egui::pos2(*max_x as f32, *max_y as f32),
        //                 ),
        //                 0.0,
        //                 egui::Stroke::new(1.0, color),
        //             );
                    
        //             ui.allocate_ui_at_rect(egui::Rect::from_min_max(
        //                 egui::pos2(*min_x as f32, *min_y as f32),
        //                 egui::pos2((*min_x + 50) as f32, (*min_y + 50) as f32),
        //             ), |ui| {
        //                 ui.label(egui::RichText::new(format!("{}{}", letter1char, letter2char)).heading().color(egui::Color32::from_rgb(255, 255, 255)).background_color(color));
        //             });
        //         }
        //     }

        //     for ((min_x, min_y, max_x, max_y), index) in self.line_boxes.iter().zip(0..) {
        //         let box_index = index as u32 + self.line_boxes.len() as u32;
        //         let (letter1, letter2) = get_letters_for_index(box_index as i32);
        //         let letter1char = std::char::from_u32(letter1 as u32 + 65).unwrap();
        //         let letter2char = std::char::from_u32(letter2 as u32 + 65).unwrap();

        //         if self.letters_typed.len() == 0 || self.letters_typed.len() > 0 && self.letters_typed[0] as u8 == letter1 {
        //             // color magenta if letter2 matches
        //             let color = if self.letters_typed.len() == 2 && self.letters_typed[1] as u8 == letter2 {
        //                 // draw half transparent box
        //                 ui.painter().rect(
        //                     egui::Rect::from_min_max(
        //                         egui::pos2(*min_x as f32, *min_y as f32),
        //                         egui::pos2(*max_x as f32, *max_y as f32),
        //                     ),
        //                     0.0,
        //                     egui::Color32::from_rgba_premultiplied(200, 0, 100, 10),
        //                     egui::Stroke::default()
        //                 );
        //                 egui::Color32::from_rgb(200, 0, 100)
        //             } else {
        //                 if self.letters_typed.len() == 2 {
        //                     egui::Color32::from_rgb(100, 100, 100)
        //                 } else {
        //                     egui::Color32::from_rgb(100,250, 10)
        //                 }
        //             };
        //             ui.painter().rect_stroke(
        //                 egui::Rect::from_min_max(
        //                     egui::pos2(*min_x as f32, *min_y as f32),
        //                     egui::pos2(*max_x as f32, *max_y as f32),
        //                 ),
        //                 0.0,
        //                 egui::Stroke::new(1.0, color),
        //             );
                    
        //             ui.allocate_ui_at_rect(egui::Rect::from_min_max(
        //                 egui::pos2(*min_x as f32, *min_y as f32),
        //                 egui::pos2((*min_x + 50) as f32, (*min_y + 50) as f32),
        //             ), |ui| {
        //                 ui.label(egui::RichText::new(format!("{}{}", letter1char, letter2char)).heading().color(egui::Color32::from_rgb(255, 255, 255)).background_color(color));
        //             });
        //         }
        //     }

        //     for ((min_x, min_y, max_x, max_y), index) in self.big_boxes.iter().zip(0..) {
        //         let box_index = index as u32 + self.line_boxes.len() as u32;
        //         let (letter1, letter2) = get_letters_for_index(box_index as i32);
        //         let letter1char = std::char::from_u32(letter1 as u32 + 65).unwrap();
        //         let letter2char = std::char::from_u32(letter2 as u32 + 65).unwrap();

        //         if self.letters_typed.len() == 0 || self.letters_typed.len() > 0 && self.letters_typed[0] as u8 == letter1 {
        //             // color magenta if letter2 matches
        //             let color = if self.letters_typed.len() == 2 && self.letters_typed[1] as u8 == letter2 {
        //                 // draw half transparent box
        //                 ui.painter().rect(
        //                     egui::Rect::from_min_max(
        //                         egui::pos2(*min_x as f32, *min_y as f32),
        //                         egui::pos2(*max_x as f32, *max_y as f32),
        //                     ),
        //                     0.0,
        //                     egui::Color32::from_rgba_premultiplied(200, 0, 100, 10),
        //                     egui::Stroke::default()
        //                 );
        //                 egui::Color32::from_rgb(200, 0, 100)
        //             } else {
        //                 if self.letters_typed.len() == 2 {
        //                     egui::Color32::from_rgb(100, 100, 100)
        //                 } else {
        //                     egui::Color32::from_rgb(250, 80, 50)
        //                 }
        //             };
        //             ui.painter().rect_stroke(
        //                 egui::Rect::from_min_max(
        //                     egui::pos2(*min_x as f32, *min_y as f32),
        //                     egui::pos2(*max_x as f32, *max_y as f32),
        //                 ),
        //                 0.0,
        //                 egui::Stroke::new(1.0, color),
        //             );
                    
        //             ui.allocate_ui_at_rect(egui::Rect::from_min_max(
        //                 egui::pos2(*min_x as f32, *min_y as f32),
        //                 egui::pos2((*min_x + 50) as f32, (*min_y + 50) as f32),
        //             ), |ui| {
        //                 ui.label(egui::RichText::new(format!("{}{}", letter1char, letter2char)).heading().color(egui::Color32::from_rgb(255, 255, 255)).background_color(color));
        //             });
        //         }
        //     }

        //     for ((min_x, min_y, max_x, max_y), index) in self.small_images.iter().zip(0..) {
        //         let (letter1, letter2) = get_letters_for_index(index);
        //         let letter1char = std::char::from_u32(letter1 as u32 + 65).unwrap();
        //         let letter2char = std::char::from_u32(letter2 as u32 + 65).unwrap();

        //         if self.letters_typed.len() == 0 || self.letters_typed.len() > 0 && self.letters_typed[0] as u8 == letter1 {
        //             // color magenta if letter2 matches
        //             let color = if self.letters_typed.len() == 2 && self.letters_typed[1] as u8 == letter2 {
        //                 // draw half transparent box
        //                 ui.painter().rect(
        //                     egui::Rect::from_min_max(
        //                         egui::pos2(*min_x as f32, *min_y as f32),
        //                         egui::pos2(*max_x as f32, *max_y as f32),
        //                     ),
        //                     0.0,
        //                     egui::Color32::from_rgba_premultiplied(200, 0, 100, 10),
        //                     egui::Stroke::default()
        //                 );
        //                 egui::Color32::from_rgb(200, 0, 100)
        //             } else {
        //                 if self.letters_typed.len() == 2 {
        //                     egui::Color32::from_rgb(100, 100, 100)
        //                 } else {
        //                     egui::Color32::from_rgb(0, 150, 150)
        //                 }
        //             };
        //             ui.painter().rect_stroke(
        //                 egui::Rect::from_min_max(
        //                     egui::pos2(*min_x as f32, *min_y as f32),
        //                     egui::pos2(*max_x as f32, *max_y as f32),
        //                 ),
        //                 0.0,
        //                 egui::Stroke::new(1.0, color),
        //             );
                    
        //             ui.allocate_ui_at_rect(egui::Rect::from_min_max(
        //                 egui::pos2(*min_x as f32, *min_y as f32),
        //                 egui::pos2((*min_x + 50) as f32, (*min_y + 50) as f32),
        //             ), |ui| {
        //                 ui.label(egui::RichText::new(format!("{}{}", letter1char, letter2char)).heading().color(egui::Color32::from_rgb(255, 255, 255)).background_color(color));
        //             });
        //         }

                ui.allocate_ui_at_rect(egui::Rect::from_min_size(
                    egui::pos2(0.0, 0.0),
                    egui::vec2(200.0, 100.0),
                ), |ui| {
                    if self.letters_typed.len() == 0 {
                        return
                    }
                    // combine vec
                    let letters = self.letters_typed.iter().map(|x| std::char::from_u32(*x as u32 + 65).unwrap()).collect::<String>();
                    // color by letter
                    let color = if self.letters_typed[0] == 8 {
                        egui::Color32::from_rgb(255, 160, 50)
                    } else if self.letters_typed[0] == 11 {
                        egui::Color32::from_rgb(150, 200, 20)
                    } else {
                        egui::Color32::from_rgb(50, 100, 200)
                    };

                    ui.label(egui::RichText::new(letters).heading().color(egui::Color32::from_rgb(255, 255, 255)).background_color(color).size(40.0));
                });
        //     }
     });
    }
}
