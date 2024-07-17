#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
#![allow(rustdoc::missing_crate_level_docs)]

use core::num;
use std::cmp;
use std::process::Stdio;

use ashpd::desktop::print;
use image::GenericImage;
use image::GenericImageView;
use rayon::iter::IndexedParallelIterator;
use rayon::iter::IntoParallelRefIterator;
use rayon::iter::ParallelIterator;
use rayon::vec;
use swiftmouse::bounding_box;
use swiftmouse::image_utils;
use swiftmouse::screenshot;
use swiftmouse::globalshortcut;
use zbus::zvariant::Endian;
use zbus::zvariant::WriteBytes;

mod gui;

const SCREENSHOT_PATH: &str = "/tmp/screenshot.png";

fn get_pixel_gray(image: &image::ImageBuffer<image::Rgb<u8>, Vec<u8>>, mut x: i16, mut y: i16) -> u16 {
    if x < 0 {
        x = 0;
    }
    if y < 0 {
        y = 0;
    }
    if x >= image.width() as i16 {
        x = image.width() as i16 - 1;
    }
    if y >= image.height() as i16 {
        y = image.height() as i16 - 1;
    }

    let pixel = image.get_pixel(x as u32, y as u32);
    return pixel.0[0] as u16 + pixel.0[1] as u16 + pixel.0[2] as u16;
}

#[tokio::main]
async fn main() {
    let (mut rx, _conn) = globalshortcut::listen().await;
    println!("[Main] Waiting for events");
    let mut screenshot_tool = screenshot::get_screenshot_tool();
    let screenshot = screenshot_tool.take_screenshot().await.unwrap();

   
    while let Some(_) = rx.recv().await {
        let total_start = std::time::Instant::now();
        let screenshot: image::ImageBuffer<image::Rgb<u8>, Vec<u8>> = screenshot_tool.take_screenshot().await.unwrap();
        // let dynamic_image = image::DynamicImage::ImageRgb8(screenshot.clone());
        // write to /tmp/screenshot.png
        screenshot.save("/tmp/screenshot.png").unwrap();
        let start = std::time::Instant::now();
        
        //gray image
        // let gray_image = dynamic_image.grayscale();
        // println!("Gray Elapsed: {:?}", start.elapsed());
        // 3x3 edge conv
        let start = std::time::Instant::now();
        let downsampled_map = vec![vec![false; (screenshot.width() / 2) as usize]; (screenshot.width() / 2) as usize];
        let downsampled_map = downsampled_map.par_iter().enumerate().map(|(x, row)| {
            let mut line = vec![false; row.len()];
            for y in 0..row.len() {
                let mut field = vec![vec![0; 4]; 4];
                for i in -1..3 {
                    for j in -1..3 {
                        field[(i+1) as usize][(j+1) as usize] = get_pixel_gray(&screenshot, (x as i16)*2 + i, (y as i16)*2 + j);
                    }
                }

                let mut tl_edge: i16 = 0;
                for i in 0..3 {
                    for j in 0..3 {
                        if i == 1 && j == 1 {
                            tl_edge += field[i][j] as i16 * 8;
                        } else {
                            tl_edge -= field[i][j] as i16;
                        }
                    }
                }
                if tl_edge > 0 {
                    line[y] = true;
                    continue;
                }


                let mut tr_edge: i16 = 0;
                for i in 0..3 {
                    for j in 0..3 {
                        if i == 1 && j == 1 {
                            tr_edge += field[i+1][j] as i16 * 8;
                        } else {
                            tr_edge -= field[i+1][j] as i16;
                        }
                    }
                }
                if tr_edge > 0 {
                    line[y] = true;
                    continue;
                }

                let mut bl_edge: i16 = 0;
                for i in 0..3 {
                    for j in 0..3 {
                        if i == 1 && j == 1 {
                            bl_edge += field[i][j+1] as i16 * 8;
                        } else {
                            bl_edge -= field[i][j+1] as i16;
                        }
                    }
                }
                if  bl_edge > 0 {
                    line[y] = true;
                    continue;
                }

                let mut br_edge: i16 = 0;
                for i in 0..3 {
                    for j in 0..3 {
                        if i == 1 && j == 1 {
                            br_edge += field[i+1][j+1] as i16 * 8;
                        } else {
                            br_edge -= field[i+1][j+1] as i16;
                        }
                    }
                }
                if br_edge > 0 {
                    line[y] = true;
                    continue;
                }
            }
            line
        }).collect::<Vec<Vec<bool>>>();
        println!("Downsampled Elapsed: {:?}", start.elapsed());

        // to debug img
        // let mut downsampled_image = image::DynamicImage::new_rgb8(screenshot.width()/2, screenshot.height()/2);
        // for x in 0..downsampled_image.width() {
        //     for y in 0..downsampled_image.height() {
        //         if downsampled_map[x as usize][y as usize] {
        //             downsampled_image.put_pixel(x, y, image::Rgba([255, 255, 255, 255]));
        //         } else {
        //             downsampled_image.put_pixel(x, y, image::Rgba([0, 0, 0, 255]));
        //         }
        //     }
        // }
        // // save downsampled image
        // downsampled_image.save("/tmp/downsampled.png").unwrap();


        // let start = std::time::Instant::now();
        // let mut edge_image = gray_image.filter3x3(&[
        //     -1.0, -1.0, -1.0,
        //     -1.0, 8.0, -1.0,
        //     -1.0, -1.0, -1.0,
        // ]);
        // println!("Edge Elapsed: {:?}", start.elapsed());
        // save edge image
        // edge_image.save("/tmp/edge.png").unwrap();
        // if > 0 then 1
        // for x in 0..edge_image.width() {
        //     for y in 0..edge_image.height() {
        //         let pixel = edge_image.get_pixel(x, y);
        //         if pixel[0] > 0 {
        //             edge_image.put_pixel(x, y, image::Rgba([255, 255, 255, 255]));
        //         } else {
        //             edge_image.put_pixel(x, y, image::Rgba([0, 0, 0, 255]));
        //         }
        //     }
        // }
        // save edge image
        // edge_image.save("/tmp/edge2.png").unwrap();
        // downsample to half  size manually
        // let start = std::time::Instant::now();
        // let downsampled_image_map = downsampled_map;
        // let mut downsampled_image_map = vec![vec![false; (edge_image.width() / 2) as usize]; (edge_image.width() / 2) as usize];
        // let mut downsampled_image1 = image::DynamicImage::new_rgb8(edge_image.width()/2, edge_image.height()/2);
        // for x in 0..edge_image.width()/2 {
        //     for y in 0..edge_image.height()/2 {
        //         // if one pixel is > 0
        //         if (
        //             edge_image.get_pixel(x*2, y*2)[0] > 0 ||
        //             edge_image.get_pixel(x*2+1, y*2)[0] > 0 ||
        //             edge_image.get_pixel(x*2, y*2+1)[0] > 0 ||
        //             edge_image.get_pixel(x*2+1, y*2+1)[0] > 0
        //         ) {
        //             downsampled_image_map[x as usize][y as usize] = true;
        //             downsampled_image1.put_pixel(x, y, image::Rgba([255, 255, 255, 255]));
        //         } else {
        //             downsampled_image_map[x as usize][y as usize] = false;
        //             downsampled_image1.put_pixel(x, y, image::Rgba([0, 0, 0, 255]));
        //         }
        //     }
        // }
        // println!("Downsampled Elapsed: {:?}", start.elapsed());
        // // save downsampled image
        // downsampled_image1.save("/tmp/downsampled1.png").unwrap();
        // map to true fals
        // for x in 0..downsampled_image.width() {
        //     for y in 0..downsampled_image.height() {
        //         let pixel = downsampled_image.get_pixel(x, y);
        //         if pixel[0] > 0 {
        //             downsampled_image_map[x as usize][y as usize] = true;
        //         }
        //     }
        // }
        let downsampled_image_map = downsampled_map;

        let start = std::time::Instant::now();
        let num_threads = rayon::current_num_threads();
        // println!("Num threads: {:?}", num_threads);
        let width = downsampled_image_map.len();
        let num_chunks = num_threads;
        let chunk_size = width / num_chunks;
        let mut chunks: Vec<(usize, usize)> = Vec::new();
        loop {
            let start = chunks.len() * chunk_size;
            let end = start + chunk_size;
            if end >= width {
                chunks.push((start, width-1));
                break;
            }
            chunks.push((start, end));
        }
        println!("Chunks: {:?}", chunks);
        let boxes = chunks.par_iter().enumerate().map(|(i, (start, end))| {
            let width = downsampled_image_map.len();
            let height = downsampled_image_map[0].len();
            let mut visited = vec![vec![false; height]; width as usize];
            let mut boxes = Vec::new();
            for x in *start..*end {
                for y in 0..height {
                    if downsampled_image_map[x][y] && !visited[x][y] {
                        let mut start_x = x;
                        let mut end_x = x;
                        let mut start_y = y;
                        let mut end_y = y;

                        let mut queue = std::collections::VecDeque::new();
                        queue.push_back((x, y));
                        visited[x][y] = true;
                        while let Some((x, y)) = queue.pop_front() {
                            if x < start_x {
                                start_x = x;
                            }
                            if x > end_x {
                                end_x = x;
                            }
                            if y < start_y {
                                start_y = y;
                            }
                            if y > end_y {
                                end_y = y;
                            }

                            if x > 0 && downsampled_image_map[x - 1][y] && !visited[x - 1][y] {
                                queue.push_back((x-1, y));
                                visited[x - 1][y] = true;
                            }
                            if x < width - 1 && downsampled_image_map[x + 1][y] && !visited[x + 1][y] {
                                queue.push_back((x+1, y));
                                visited[x + 1][y] = true;
                            }
                            if y > 0 && downsampled_image_map[x][y - 1] && !visited[x][y - 1] {
                                queue.push_back((x, y-1));
                                visited[x][y - 1] = true;
                            }
                            if y < height - 1 && downsampled_image_map[x][y + 1] && !visited[x][y + 1] {
                                queue.push_back((x, y+1));
                                visited[x][y + 1] = true;
                            }
                        }

                        // println!("Chunk {:?} Box: {:?} {:?} {:?} {:?}", i, start_x, start_y, end_x, end_y);
                        boxes.push((cmp::max(start_x as i32 -1, 0) as usize,
                         cmp::max(start_y as i32 -1, 0) as usize,
                         cmp::min(end_x as i32 +1, width as i32 -1) as usize,
                         cmp::min(end_y as i32 +1, height as i32 -1) as usize));
                    }
                }
            }
            boxes
        }).collect::<Vec<Vec<(usize, usize, usize, usize)>>>().concat();
        println!("Boxes Elapsed: {:?}", start.elapsed());

        let start = std::time::Instant::now();
        let big_boxes = boxes.clone().into_iter().filter(|(min_x, min_y, max_x, max_y)| {
            (max_y - min_y) > 15
        }).collect::<Vec<(usize, usize, usize, usize)>>();
        println!("Big box Elapsed: {:?}", start.elapsed());

        let start = std::time::Instant::now();
        let large_images = boxes.clone().into_iter().filter(|(min_x, min_y, max_x, max_y)| {
            (max_y - min_y) > 300 && (max_x - min_x) > 300 && *min_x > 10 && *max_x < width - 10
        }).collect::<Vec<(usize, usize, usize, usize)>>();
        println!("large img Elapsed: {:?}", start.elapsed());

        let start = std::time::Instant::now();
        let text_boxes = boxes.clone().into_iter().filter(|(min_x, min_y, max_x, max_y)| {
            (max_y - min_y) <= 15
        }).filter(|(min_x, min_y, max_x, max_y)| {
            let mut in_video = false;
            for video in &large_images {
                if *min_x >= video.0 && *min_y >= video.1 && *max_x <= video.2 && *max_y <= video.3 {
                    in_video = true;
                    break;
                }
            }
            !in_video
        }).collect::<Vec<(usize, usize, usize, usize)>>();
        println!("Video filter Elapsed: {:?}", start.elapsed());

        let start = std::time::Instant::now();
        // create lines by merging text boxes that are close on x and aligned on 1
        let mut lines: Vec<Vec<(usize,usize,usize,usize)>> = Vec::new();
        let mut text_boxes: Vec<(usize, usize, usize, usize)> = text_boxes.clone();
        let mut handled: Vec<i32> = vec![-1; text_boxes.len()];
        for i in 0..text_boxes.len() {
           if handled[i] == -1 {
                let mut line = Vec::new();
                line.push(text_boxes[i]);
                lines.push(line);
                handled[i] = lines.len() as i32 - 1;
            }

            for j in 0..text_boxes.len() {
                if handled[j] != -1 {
                    continue;
                }
                let (min_x, min_y, max_x, max_y) = text_boxes[j];
                let (min_x1, min_y1, max_x1, max_y1) = text_boxes[i];
                if (min_y1 as i32 - min_y as i32).abs() <= 3 && ((max_x1 as i32 - min_x as i32).abs() <= 4 || (max_x as i32 - min_x1 as i32).abs() <= 4) {
                    lines[handled[i] as usize].push(text_boxes[j]);
                    handled[j] = handled[i];
                }
            }
        }

        // create bounding box per line
        let mut line_boxes = Vec::new();
        for line in lines {
            let mut min_x = line[0].0;
            let mut min_y = line[0].1;
            let mut max_x = line[0].2;
            let mut max_y = line[0].3;
            for (x, y, x1, y1) in line {
                if x < min_x {
                    min_x = x;
                }
                if y < min_y {
                    min_y = y;
                }
                if x1 > max_x {
                    max_x = x1;
                }
                if y1 > max_y {
                    max_y = y1;
                }
            }
            line_boxes.push((min_x, min_y, max_x, max_y));
        }

        let start = std::time::Instant::now();
        // small images are big boxes that are > 50% white
        let mut small_images = Vec::new();
        for (min_x, min_y, max_x, max_y) in big_boxes.clone() {
            let mut white = 0;
            let mut total = 0;
            for x in min_x..max_x {
                for y in min_y..max_y {
                    if x >= downsampled_image_map.len() as usize || y >= downsampled_image_map[0].len() as usize {
                        continue;
                    }
                    total += 1;
                    if downsampled_image_map[x][y] {
                        white += 1;
                    }
                }
            }
            if white as f32 / total as f32 > 0.5 {
                small_images.push((min_x, min_y, max_x, max_y));
            }
        }
        println!("Small img Elapsed: {:?}", start.elapsed());


        println!("Num boxes: {:?}", boxes.len());
        println!("Num small boxes: {:?}", text_boxes.len());
        println!("Num big boxes: {:?}", big_boxes.len());
        println!("Num ultra large boxes: {:?}", large_images.len());
        println!("Num lines: {:?}", line_boxes.len());
        println!("Num small images: {:?}", small_images.len());
        println!("Total Elapsed: {:?}", total_start.elapsed());


        // image_utils::draw_boxes(&mut downsampled_image, &text_boxes, image::Rgba([255, 0, 0, 255]));
        // image_utils::draw_boxes(&mut downsampled_image, &big_boxes, image::Rgba([0, 255, 0, 255]));
        // image_utils::draw_boxes(&mut downsampled_image, &large_images, image::Rgba([0, 255, 255, 255]));
        // image_utils::draw_boxes(&mut downsampled_image, &line_boxes, image::Rgba([255, 255, 0, 255]));
        // image_utils::draw_boxes(&mut downsampled_image, &small_images, image::Rgba([255, 0, 255, 255]));
        // save downsampled image
        // downsampled_image.save("/tmp/downsampled2.png").unwrap();

        let mut binpath = std::env::current_exe().unwrap();
        binpath.set_file_name("gui");
        let gui_binpath = binpath.to_str().unwrap();
        println!("[Main] GUI binpath: {:?}", gui_binpath);

        let mut child = std::process::Command::new(gui_binpath)
            .stdin(Stdio::piped())
            .spawn()
            .unwrap();
        match child.stdin.as_mut() {
            Some(stdin) => {
                write_boxes(stdin, &unmap_downsampled_boxes(&big_boxes));
                write_boxes(stdin, &unmap_downsampled_boxes(&line_boxes));
                write_boxes(stdin, &unmap_downsampled_boxes(&small_images));
                write_boxes(stdin, &unmap_downsampled_boxes(&large_images));
            }
            None => {
                println!("[Main] Failed to open stdin");
            }
        }
        // wait for proc exit
        child.wait().unwrap();
    }

    println!("[Main] Exiting");
}

fn unmap_downsampled_boxes(boxes: &Vec<(usize, usize, usize, usize)>) -> Vec<(usize, usize, usize, usize)> {
    let mut new_boxes = Vec::new();
    for (min_x, min_y, max_x, max_y) in boxes {
        new_boxes.push((
            *min_x * 2,
            *min_y * 2,
            *max_x * 2,
            *max_y * 2
        ));
    }
    new_boxes
}

fn write_boxes(stdin: &mut std::process::ChildStdin, boxes: &Vec<(usize, usize, usize, usize)>) {
    let boxes_len = boxes.len() as u32;
    stdin.write_u32(Endian::Little, boxes_len).unwrap();
    for box_ in boxes {
        stdin.write_u32(Endian::Little, box_.0 as u32).unwrap();
        stdin.write_u32(Endian::Little, box_.1 as u32).unwrap();
        stdin.write_u32(Endian::Little, box_.2 as u32).unwrap();
        stdin.write_u32(Endian::Little, box_.3 as u32).unwrap();
    }
}

async fn run(screenshot: &image::ImageBuffer<image::Rgb<u8>, Vec<u8>>) {
    println!("[Main] Opening screenshot");
    let start_time = std::time::Instant::now();
    screenshot.save(SCREENSHOT_PATH).unwrap();
    println!("[Main] Elapsed: {:?}", start_time.elapsed());

    println!("[Main] Finding bounding boxes");
    let start_time = std::time::Instant::now();
    let dynamic_image = image::DynamicImage::ImageRgb8(screenshot.clone());
    let (text_boxes, big_boxes)/* bounding_boxes */ = bounding_box::find_bounding_boxes_v2(&dynamic_image);
    println!("[Main] Elapsed: {:?}", start_time.elapsed());
    println!("[Main] Found {:?} small boxes and {:?} big boxes", text_boxes.len(), big_boxes.len());

    // std::process::exit(0);

    // binpath
    let mut binpath = std::env::current_exe().unwrap();
    binpath.set_file_name("gui");
    let gui_binpath = binpath.to_str().unwrap();
    println!("[Main] GUI binpath: {:?}", gui_binpath);

    let mut child = std::process::Command::new(gui_binpath)
        .stdin(Stdio::piped())
        .spawn()
        .unwrap();
    match child.stdin.as_mut() {
        Some(stdin) => {
            // write len of big boxes
            let big_boxes_len = big_boxes.len() as u32;
            stdin.write_u32(Endian::Little, big_boxes_len).unwrap();
            for big_box in big_boxes {
                // write big boxes
                stdin.write_u32(Endian::Little, big_box.0).unwrap();
                stdin.write_u32(Endian::Little, big_box.1).unwrap();
                stdin.write_u32(Endian::Little, big_box.2).unwrap();
                stdin.write_u32(Endian::Little, big_box.3).unwrap();
            }
        }
        None => {
            println!("[Main] Failed to open stdin");
        }
    }
    // wait for proc exit
    child.wait().unwrap();
    
    println!("[Main] GUI closed");
}