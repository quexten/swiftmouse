use std::{sync::{Arc, Mutex}, thread};

use image::{DynamicImage, GenericImageView};

pub fn find_bounding_boxes(image: &DynamicImage) -> Vec<(u32, u32, u32, u32)> {
    let bounding_boxes = find_initial_bounding_boxes(image);
    let start_time = std::time::Instant::now();
    let mut extendedBoundingBoxes: Vec<(u32, u32, u32, u32)> = Vec::new();
    for (min_x, min_y, max_x, max_y) in bounding_boxes.clone() {
        let min_x = min_x.saturating_sub(3);
        let min_y = min_y.saturating_sub(3);
        let max_x = max_x.saturating_add(3);
        let max_y = max_y.saturating_add(3);
        extendedBoundingBoxes.push((min_x, min_y, max_x, max_y));
    }
    let merged_bounding_boxes = merge_overlapping_bounding_boxes(bounding_boxes);

    let mut filteredBoundingBoxes: Vec<(u32, u32, u32, u32)> = Vec::new();
    for (min_x, min_y, max_x, max_y) in extendedBoundingBoxes.clone() {
        if (max_x - min_x) < 100 && (max_y - min_y) < 100 {
            filteredBoundingBoxes.push((min_x, min_y, max_x, max_y));
        }
    }


    let start_time = std::time::Instant::now();
    let mergedBoundingBoxes = merge_overlapping_bounding_boxes(extendedBoundingBoxes);
    println!("Merged bounding boxes {:?}", mergedBoundingBoxes.len());
    return mergedBoundingBoxes;
}
pub fn find_initial_bounding_boxes(image: &DynamicImage) -> Vec<(u32, u32, u32, u32)> {
    let bounding_boxes: Arc<Mutex<Vec<(u32, u32, u32, u32)>>> = Arc::new(Mutex::new(Vec::new()));
    let mut join_handles = Vec::new();

    let num_threads = 16;
    for i in 0..num_threads {
        let local_screenshot = image.clone();
        let start_x = local_screenshot.width() as u32 / num_threads * i;
        let end_x = local_screenshot.width() as u32 / num_threads * (i+1);
        let bb1 = bounding_boxes.clone();
        let join_handle = thread::spawn(move || {
            let mut visited_bitmap = vec![vec![false; local_screenshot.height() as usize]; local_screenshot.width() as usize];
            let local_bitmap_height = local_screenshot.height() as u32;
            for x in start_x..end_x {
                //println!("Thread {:?} x {:?}", i, x);
                for y in 0..local_bitmap_height {
                    //println!("Thread {:?} y {:?}", i, y);
                    let bounding_box = get_bounding_box_flood_fill(&local_screenshot, x, y as u32, &mut visited_bitmap);
                    match bounding_box {
                        Some(bb) => {
                            let mut bbs = bb1.lock().unwrap();
                            bbs.push(bb);
                        },
                        None => {
                        }
                    }
                }
            }
        });
        join_handles.push(join_handle);
    }

    for joinHandle in join_handles {
        joinHandle.join().unwrap();
    }
    let boundingBoxes = bounding_boxes.lock().unwrap();
    return boundingBoxes.clone();
}

pub fn merge_overlapping_bounding_boxes(boundingBoxes: Vec<(u32, u32, u32, u32)>) -> Vec<(u32, u32, u32, u32)> {
    let mut mergedBoundingBoxes: Vec<(u32, u32, u32, u32)> = Vec::new();
    let mut visited_list: Vec<bool> = vec![false; boundingBoxes.len()];
    let mut did_merge = false;
    println!("Merging bounding boxes {:?}", boundingBoxes.len());

    for (i, boundingBox) in boundingBoxes.iter().enumerate(){
        if visited_list[i] {
            continue;
        }
        visited_list[i] = true;
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
            }
        }

        // discard if width and height are less are more than 100
        if (current_bb.2 - current_bb.0) > 100 && (current_bb.3 - current_bb.1) > 100 {
            continue;
        }

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

fn get_pixel_grayscale(image: &DynamicImage, x: u32, y: u32) -> i32 {
    let [r,g,b,a] = image.get_pixel(x, y).0;
    (r as u32 + g as u32 + b as u32) as i32
}

pub(crate) fn get_bounding_box_flood_fill(image: &DynamicImage, x: u32, y: u32, visited_bitmap: &mut Vec<Vec<bool>>) -> Option<(u32, u32, u32, u32)> {
    let mut min_x = x;
    let mut max_x = x;
    let mut min_y = y;
    let mut max_y = y;

    let mut open: Vec<(u32, u32)> = Vec::new();
    open.push((x, y));
    let mut pixel_count = 0;
    let mut visited: Vec<(u32, u32)> = Vec::new();

    while let Some((x, y)) = open.pop() {
        if x >= image.width()-1 as u32 || y >= image.height()-1 as u32 || x <= 1 || y <= 1 {
            continue;
        }

        let edge_pixel = get_pixel_grayscale(image, x, y) * 4
            + -1 * get_pixel_grayscale(image, x, y+1)
            + -1 * get_pixel_grayscale(image, x-1, y)
            + -1 * get_pixel_grayscale(image, x+1, y)
            + -1 * get_pixel_grayscale(image, x, y-1);
        if edge_pixel < 100 { 
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
            return None
        }
    }

    if pixel_count < 5 {
        return None
    }

    Some((min_x, min_y, max_x, max_y))
}