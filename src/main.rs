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
    // let screenshot_uri = "/home/quexten/screenshot_5.png";
    println!("[Main] Elapsed: {:?}", start_time.elapsed());

    println!("[Main] Opening screenshot");
    let start_time = std::time::Instant::now();
    let screenshot = image::open(screenshot_uri.clone()).unwrap();
    std::fs::copy(screenshot_uri.clone(), SCREENSHOT_PATH).unwrap();
    // std::fs::remove_file(screenshot_uri).unwrap();
    println!("[Main] Elapsed: {:?}", start_time.elapsed());

    println!("[Main] Finding bounding boxes");
    let start_time = std::time::Instant::now();
    let (text_boxes, big_boxes)/* bounding_boxes */ = bounding_box::find_bounding_boxes_v2(&screenshot);
    println!("[Main] Elapsed: {:?}", start_time.elapsed());
    println!("[Main] Found {:?} small boxes and {:?} big boxes", text_boxes.len(), big_boxes.len());


    println!("[Main] Showing GUI");
    autotype::start_autoclick_session().await.unwrap();
    gui::show_gui(text_boxes, big_boxes, screenshot.width(), screenshot.height(), SCREENSHOT_PATH.to_string());
}
