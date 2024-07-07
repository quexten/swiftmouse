#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
#![allow(rustdoc::missing_crate_level_docs)]
use std::{borrow::Cow, io::{Read, Write}};

use ashpd::{desktop::{device::Device, print, remote_desktop::{self, DeviceType, RemoteDesktop}, screenshot}, WindowIdentifier};
use eframe::egui::{self, InputState, ViewportCommand};
use image::{GenericImage, GenericImageView, Pixel};
use imageproc::definitions::Position;
use tokio::fs::read;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut screenshot_uri = String::new();
    match screenshot::ScreenshotRequest::default()
        .interactive(false)
        .modal(false)
        .send()
        .await
        .and_then(|r| r.response())
    {
        Ok(response) => {
            let screenshot = response.uri().clone();
            println!("Screenshot taken: {}", screenshot);
            screenshot_uri = String::from(screenshot.to_string().replace("file://", ""));
        }
        Err(err) => {
            println!("Failed to take screenshot: {}", err);
        }
    }

    println!("reading");
    let mut screenshot = image::open(screenshot_uri).unwrap();
    screenshot = screenshot.grayscale();
    screenshot.save("./screenshot_grayscale.png").unwrap();
    // edge detection
    screenshot = screenshot.filter3x3(&[
        -1.0, -1.0, -1.0,
        -1.0, 8.0, -1.0,
        -1.0, -1.0, -1.0,
    ]);
    for x in 0..screenshot.width() {
        for y in 0..screenshot.height() {
            if screenshot.get_pixel(x, y) != image::Rgba([0, 0, 0, 0]) {
                screenshot.put_pixel(x, y, image::Rgba([255, 255, 255, 255]));
            }
        }
    }
    screenshot.save("./edges.png").unwrap();


    let mut tiny = screenshot.to_rgb8();
    let mut boundingBoxes: Vec<(u32, u32, u32, u32)> = Vec::new();
    let mut visited_bitmap = vec![vec![false; tiny.height() as usize]; tiny.width() as usize];

    for x in 0..tiny.width() {
        println!("x: {}", x);
        for y in 0..tiny.height() {
            let bounding_box = get_bounding_box_flood_fill(&mut tiny, x, y, &mut visited_bitmap);
            match bounding_box {
                Some(bb) => {
                    // println!("{:?}", bb);
                    boundingBoxes.push(bb);
                },
                None => {
                    //println!("Failed to get bounding box");
                }
            }
        }
    }

    // extend each bounding box by 3 pixels
    let mut extendedBoundingBoxes: Vec<(u32, u32, u32, u32)> = Vec::new();
    for (min_x, min_y, max_x, max_y) in boundingBoxes.clone() {
        let min_x = min_x.saturating_sub(3);
        let min_y = min_y.saturating_sub(3);
        let max_x = max_x.saturating_add(3);
        let max_y = max_y.saturating_add(3);
        extendedBoundingBoxes.push((min_x, min_y, max_x, max_y));
    }

    let mut debug_img = tiny.clone();
    for (min_x, min_y, max_x, max_y) in boundingBoxes.clone() {
        for x in min_x..max_x {
            for y in min_y..max_y {
                if x == min_x || x == max_x-1 || y == min_y || y == max_y-1 {
                    debug_img.put_pixel(x, y, image::Rgb([0, 255, 0]));
                }
            }
        }
    }
    debug_img.save("./screenshot_annotated.png").unwrap();

    let mergedBoundingBoxes = merge_overlapping_bounding_boxes(extendedBoundingBoxes);
    println!("Merged bounding boxes {:?}", mergedBoundingBoxes.len());

    for (min_x, min_y, max_x, max_y) in mergedBoundingBoxes.clone() {
        for x in min_x..max_x {
            for y in min_y..max_y {
                if x < 0 || y < 0 || x >= debug_img.width() || y >= debug_img.height() {
                    continue;
                }

                let color = image::Rgb([255, 0, 0]);
                if x == min_x || x == max_x-1 || y == min_y || y == max_y-1 {
                    debug_img.put_pixel(x, y, color);
                }
            }
        }
    }

    debug_img.save("./screenshot_annotated_merged.png").unwrap();
    show_gui(mergedBoundingBoxes);
    Ok(())
}

fn merge_overlapping_bounding_boxes(boundingBoxes: Vec<(u32, u32, u32, u32)>) -> Vec<(u32, u32, u32, u32)> {
    let mut mergedBoundingBoxes: Vec<(u32, u32, u32, u32)> = Vec::new();
    let mut visited_list: Vec<bool> = vec![false; boundingBoxes.len()];
    let mut did_merge = false;
    println!("Merging bounding boxes {:?}", boundingBoxes.len());

    for (i, boundingBox) in boundingBoxes.iter().enumerate(){
        if visited_list[i] {
            continue;
        }
        visited_list[i] = true;
        println!("visited {:?}", i);
        let mut current_bb = *boundingBox;

        let mut did_merge_local = false;
        for (j, boundingBox2) in boundingBoxes.iter().enumerate(){
            if visited_list[j] {
                continue;
            }
            if bounds_overlap(current_bb, *boundingBox2) {
                visited_list[j] = true;
                current_bb = merge_bounds(current_bb, *boundingBox2);
                did_merge = true;
                did_merge_local = true;
                println!("merging {:?} with {:?}", boundingBox, boundingBox2);
            }
        }
        println!("merged {:?}", did_merge_local);

        mergedBoundingBoxes.push(current_bb);
    }
    
    if did_merge {
        println!("one more round{:?}", mergedBoundingBoxes.len());
        return merge_overlapping_bounding_boxes(mergedBoundingBoxes);
    } else {
        println!("done {:?}", mergedBoundingBoxes.len());
        return mergedBoundingBoxes;
    }
}

fn bounds_overlap(a: (u32, u32, u32, u32), b: (u32, u32, u32, u32)) -> bool {
    let (min_x, min_y, max_x, max_y) = a;
    let (min_x2, min_y2, max_x2, max_y2) = b;
    if min_x2 >= min_x && min_y2 >= min_y && max_x2 <= max_x && max_y2 <= max_y {
        return true;
    }

    if contains(a, (min_x2, min_y2)) || contains(a, (max_x2, max_y2)) || contains(a, (min_x2, max_y2)) || contains(a, (max_x2, min_y2)) {
        return true;
    }

    false
}

fn contains(a: (u32, u32, u32, u32), b: (u32, u32)) -> bool {
    let (min_x, min_y, max_x, max_y) = a;
    let (x, y) = b;
    if x >= min_x && x <= max_x && y >= min_y && y <= max_y {
        return true;
    }
    false
}

fn merge_bounds(a: (u32, u32, u32, u32), b: (u32, u32, u32, u32)) -> (u32, u32, u32, u32) {
    let (min_x, min_y, max_x, max_y) = a;
    let (min_x2, min_y2, max_x2, max_y2) = b;
    let min_x = min_x.min(min_x2);
    let min_y = min_y.min(min_y2);
    let max_x = max_x.max(max_x2);
    let max_y = max_y.max(max_y2);
    (min_x, min_y, max_x, max_y)
}

fn get_bounding_box_flood_fill(image: &mut image::ImageBuffer<image::Rgb<u8>, Vec<u8>>, x: u32, y: u32, visited_bitmap: &mut Vec<Vec<bool>>) -> Option<(u32, u32, u32, u32)> {
    let mut min_x = x;
    let mut max_x = x;
    let mut min_y = y;
    let mut max_y = y;

    let starting_color = image.get_pixel(x, y).clone();
    
    let mut open: Vec<(u32, u32)> = Vec::new();
    open.push((x, y));
    let mut pixel_count = 0;
    let mut visited: Vec<(u32, u32)> = Vec::new();

    while let Some((x, y)) = open.pop() {
        if x >= image.width() || y >= image.height() || x == 0 || y == 0 {
            continue;
        }

        // skip non edges
        if *image.get_pixel(x, y) != image::Rgb([255, 255, 255]) {
            continue;
        }

        // check if pixel is already visited
        if visited_bitmap[x as usize][y as usize] {
            continue;
        }
        visited_bitmap[x as usize][y as usize] = true;

        if x < min_x {
            min_x = x;
        }
        if x > max_x {
            max_x = x;
        }
        if y < min_y {
            min_y = y;
        }
        if y > max_y {
            max_y = y;
        }

        open.push((x+1, y));
        open.push((x-1, y));
        open.push((x, y+1));
        open.push((x, y-1));

        pixel_count += 1;
        visited.push((x, y));

        if pixel_count > 500 {
            for (x, y) in visited {
                //image.put_pixel(x, y, image::Rgb([0, 0, 255]));
            }
            return None
        }
    }

    let width = max_x - min_x;
    let height = max_y - min_y;

    if pixel_count < 5 {
        return None
    }

    for (x, y) in visited {
        //image.put_pixel(x, y, image::Rgb([0, 255, 0]));
    }

    Some((min_x, min_y, max_x, max_y))
}

fn show_gui(mut positions: Vec<(u32, u32, u32, u32)>) {
    let mut options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([400.0, 800.0]),
        ..Default::default()
    };
    options.viewport.fullscreen = Some(true);
    // options.viewport.maximized = Some(true);
    eframe::run_native(
        "Image Viewer",
        options,
        Box::new(|cc| {
            // This gives us image support:
            egui_extras::install_image_loaders(&cc.egui_ctx);
            let mut app = Box::<MyApp>::default();
            app.positions = positions;
            app.first_letter_typed = -1;
            app.second_letter_typed = -1;
            Ok(app)
        }),
    );
}

#[derive(Default)]
struct MyApp {
    positions: Vec<(u32, u32, u32, u32)>,
    first_letter_typed: i32,
    second_letter_typed: i32,
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
        ctx.input(|i| {
            if i.key_pressed(egui::Key::Escape) {
                std::process::exit(0);
            }
            
            let key = get_key(i);
            match key {
                Some(key) => {
                    println!("Key pressed: {:?}", key);
                    if self.first_letter_typed == -1 {
                        println!("First letter typed: {:?}", key);
                        self.first_letter_typed = key;
                    } else {
                        println!("Second letter typed: {:?}", key);
                        self.second_letter_typed = key;
                    }
                }
                None => {}
            }
        });

        let frame = egui::Frame::default().fill(egui::Color32::from_rgb(0, 0, 0)).inner_margin(0.0);
        egui::CentralPanel::default().frame(frame).show(ctx, |ui| {
            if self.second_letter_typed != -1 {
                let index = get_index_for_letters(self.first_letter_typed as u8, self.second_letter_typed as u8);
                ctx.send_viewport_cmd(egui::ViewportCommand::Minimized(true));
                let (min_x, min_y, max_x, max_y) = self.positions[index as usize];
                println!("Clicking at: {:?} {:?}", min_x, min_y);
                tokio::spawn(async move {
                    autoclick_at(min_x as i32, min_y as i32, 3840, 2160).await.unwrap();
                    // exit
                    std::process::exit(0);
                });
    
            }


            
            ui.add(
                egui::Image::new("file://./screenshot_grayscale.png"),
            );
            // });
            // label with background box

            for ((min_x, min_y, max_x, max_y), index) in self.positions.iter().zip(0..) {
                let (letter1, letter2) = get_letters_for_index(index);
                let letter1char = std::char::from_u32(letter1 as u32 + 65).unwrap();
                let letter2char = std::char::from_u32(letter2 as u32 + 65).unwrap();

                if self.first_letter_typed == -1 || self.first_letter_typed == letter1 as i32 {
                    ui.allocate_ui_at_rect(egui::Rect::from_min_max(
                        egui::pos2(*min_x as f32, *min_y as f32),
                        egui::pos2(*max_x as f32, *max_y as f32),
                    ), |ui| {
                        ui.label(egui::RichText::new(format!("{}{}", letter1char, letter2char)).heading().color(egui::Color32::from_rgb(255, 255, 255)).background_color(egui::Color32::from_rgb(0,100,100)))
                    });
                }
            }
     });
    }
}



async fn autoclick_at(x: i32, y: i32, screen_width: i32, screen_height: i32) -> Result<(), Box<dyn std::error::Error>> {
    // autotype portal
    let proxy = RemoteDesktop::new().await?;
    let session = proxy.create_session().await?;
    let token = read_token();
    match token {
        Some(token) => {
            proxy.select_devices(&session, DeviceType::Pointer | DeviceType::Touchscreen, Some(token.as_str()), ashpd::desktop::PersistMode::ExplicitlyRevoked).await?;
        }
        None => {
            proxy.select_devices(&session, DeviceType::Pointer | DeviceType::Touchscreen, None, ashpd::desktop::PersistMode::ExplicitlyRevoked).await?;
        }
    }
    let response = proxy
        .start(&session, &WindowIdentifier::default())
        .await?
        .response()?;
    match response.restore_token() {
        Some(token) => {
            write_token(&token)?;
        }
        None => {
            println!("No token found");
        }
    }
    
    // sleep
    proxy.notify_pointer_motion(&session, 10000.0, 10000.0).await?;
    proxy.notify_pointer_motion(&session, (x - screen_width) as f64, (y - screen_height) as f64).await?;
    // sleep
    tokio::time::sleep(std::time::Duration::from_millis(100)).await;
    proxy.notify_pointer_button(&session, 272, remote_desktop::KeyState::Pressed).await?;
    tokio::time::sleep(std::time::Duration::from_millis(100)).await;
    proxy.notify_pointer_button(&session, 272, remote_desktop::KeyState::Released).await?;

    Ok(())
}

fn write_token(token: &str) -> Result<(), Box<dyn std::error::Error>> {
    let mut file = std::fs::File::create("token")?;
    file.write_all(token.as_bytes())?;
    Ok(())
}

fn read_token() -> Option<String> {
    let mut file = std::fs::File::open("token");
    match file {
        Err(_) => {
            return None;
        }
        Ok(mut file) => {
            let mut token = String::new();
            file.read_to_string(&mut token).expect("something went wrong reading the file");
            Some(token)
        }
    }
}