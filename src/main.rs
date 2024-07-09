#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
#![allow(rustdoc::missing_crate_level_docs)]

use bounding_box::{find_bounding_boxes, find_initial_bounding_boxes};

mod bounding_box;
mod gui;
mod screenshot;
mod autotype;
mod image_utils;

const SCREENSHOT_PATH: &str = "/tmp/screenshot.png";

#[tokio::main]
async fn main() {
    println!("[Main] Taking screenshot");
    let start_time = std::time::Instant::now();
    let screenshot_uri = match screenshot::screenshot().await {
        Ok(uri) => uri,
        Err(err) => {
            println!("[Main] Failed to take screenshot: {:?}", err);
            return;
        }
    };
    println!("[Main] Elapsed: {:?}", start_time.elapsed());

    println!("[Main] Opening screenshot");
    let start_time = std::time::Instant::now();
    let screenshot = image::open(screenshot_uri.clone()).unwrap();
    std::fs::copy(screenshot_uri.clone(), SCREENSHOT_PATH).unwrap();
    std::fs::remove_file(screenshot_uri).unwrap();
    println!("[Main] Elapsed: {:?}", start_time.elapsed());

    println!("[Main] Finding bounding boxes");
    let start_time = std::time::Instant::now();
    let bounding_boxes = find_bounding_boxes(&screenshot);
    println!("[Main] Elapsed: {:?}", start_time.elapsed());
    println!("[Main] Found {:?} bounding boxes", bounding_boxes.len());


    println!("[Main] Showing GUI");
    autotype::start_autoclick_session().await.unwrap();
    gui::show_gui(bounding_boxes, screenshot.width(), screenshot.height(), SCREENSHOT_PATH.to_string());
}
