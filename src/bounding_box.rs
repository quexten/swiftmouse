use std::{cmp, sync::{Arc, Mutex}, thread, vec};

use image::{DynamicImage, GenericImage, GenericImageView};
use rayon::iter::{IndexedParallelIterator, IntoParallelRefIterator, ParallelIterator};

use crate::image_utils::{self, draw_box};

#[derive(Debug, PartialEq)]
enum Direction {
    Horizontal,
    Vertical,
}

pub fn find_horizontal_lines(image: &DynamicImage) -> Vec<Vec<(u32, u32)>> {
    // with rayon
    let mapped_lines: Vec<Vec<(u32, u32)>> = (0..image.height()).collect::<Vec<_>>()
        .par_iter()
        .map(|index| {
            let mut lines = Vec::new();
            let mut current_color = image.get_pixel(0, *index as u32);
            let mut start_x = 0;
            for x in 1..image.width() {
                let color = image.get_pixel(x, *index as u32);
                if color != current_color || x == image.width()-1 {
                    // if length is greater than 10
                    if x - start_x > 40 {
                        lines.push((start_x, x));
                    }
                    
                    start_x = x;
                    current_color = color;
                }
            }
            lines
         })
         .collect();

    return mapped_lines;
}

pub fn find_vertical_lines(image: &DynamicImage) -> Vec<Vec<(u32, u32)>> {
    // let mut vertical_lines = vec![Vec::new(); image.width() as usize];
    // for x in 0..image.width() {
    //     let mut current_color = image.get_pixel(x, 0);
    //     let mut start_y = 0;
    //     for y in 1..image.height() {
    //         let color = image.get_pixel(x, y);
    //         if color != current_color || y == image.height()-1 {
    //             // if length is greater than 10
    //             if y - start_y > 20 {
    //                 vertical_lines[x as usize].push((start_y, y));
    //             }
    //             
    //             start_y = y;
    //             current_color = color;
    //         }
    //     }
    // }
    
    let mapped_lines = (0..image.width()).collect::<Vec<_>>()
        .par_iter()
        .map(|index| {
            let mut lines = Vec::new();
            let mut current_color = image.get_pixel(*index as u32, 0);
            let mut start_y = 0;
            for y in 1..image.height() {
                let color = image.get_pixel(*index as u32, y);
                if color != current_color || y == image.height()-1 {
                    // if length is greater than 10
                    if y - start_y > 20 {
                        lines.push((start_y, y));
                    }
                    
                    start_y = y;
                    current_color = color;
                }
            }
            lines
         })
         .collect();

    return mapped_lines;
}

fn draw_lines(image: &DynamicImage, lines: &Vec<Vec<(u32, u32)>>, direction: Direction, color: image::Rgba<u8>) -> DynamicImage {
    let mut debug_img = image.clone();
    let size = match direction {
        Direction::Horizontal => image.height(),
        Direction::Vertical => image.width(),
    };

    for i in 0..size {
        for (start_j, end_j) in lines[i as usize].clone() {
            for j in start_j..end_j {
                let (x, y) = match direction {
                    Direction::Horizontal => (j, i),
                    Direction::Vertical => (i, j),
                };
                debug_img.put_pixel(x, y, color);
                if j == start_j {
                    debug_img.put_pixel(x, y, image::Rgba([0, 255, 255, 255]));
                } else if j == end_j-1 {
                    debug_img.put_pixel(x, y, image::Rgba([255, 0, 255, 255]));
                }
            }
        }
    }
    return debug_img;
}

fn boxes_overlap(box1: &(u32, u32, u32, u32), box2: &(u32, u32, u32, u32)) -> bool {
    return box1.0 < box2.2 && box1.2 > box2.0 && box1.1 < box2.3 && box1.3 > box2.1;
}

fn merge_boxes(boxes: Vec<(u32, u32, u32, u32)>, x_padding: u32, y_padding: u32) -> Vec<(u32, u32, u32, u32)> {
    // println!("Merging boxes {:?}", boxes.len());
    let mut boxes = boxes.clone();
    let mut merged_boxes = Vec::new();
    // merge two boxes if they overlap, while there are overlapping boxes remaining
    loop {
        // println!("loop {:?}", boxes.len());
        let mut merged = vec![false; boxes.len()];
        for (idx, (start_x, start_y, end_x, end_y)) in boxes.iter().enumerate() {
            for (idx1, (start_x1, start_y1, end_x1, end_y1)) in boxes.iter().enumerate() {
                if idx <= idx1 || merged[idx] || merged[idx1] {
                    continue;
                }

                let mut start_x_cmp = start_x.clone();
                let mut end_x_cmp = end_x.clone();
                let mut start_x1_cmp = start_x1.clone();
                let mut end_x1_cmp = end_x1.clone();

                start_x_cmp = start_x - x_padding;
                end_x_cmp = end_x + x_padding;
                start_x1_cmp = start_x1 - y_padding;
                end_x1_cmp = end_x1 + y_padding;

                if boxes_overlap(&(start_x_cmp, *start_y, end_x_cmp, *end_y), &(start_x1_cmp, *start_y1, end_x1_cmp, *end_y1)) {
                    let new_box = (
                        cmp::min(*start_x, *start_x1),
                        cmp::min(*start_y, *start_y1),
                        cmp::max(*end_x, *end_x1),
                        cmp::max(*end_y, *end_y1),
                    );
                    merged_boxes.push(new_box);
                    merged[idx] = true;
                    merged[idx1] = true;
                    break;
                }
            }
        }
        // push all non merged 
        for (idx, merged) in merged.iter().enumerate() {
            if !*merged {
                merged_boxes.push(boxes[idx]);
            }
        }

        // println!("loop end {:?} {:?}", merged.iter().all(|x| *x), merged.iter().filter(|x| !*x).count());
        boxes = merged_boxes.clone();
        merged_boxes = Vec::new();
        if !merged.iter().any(|x| *x) {
            break;
        }
    }

    // println!("Merged boxes {:?}", boxes.len());
    return boxes;
}

fn remove_box_padding(img: &DynamicImage, boxes: Vec<(u32, u32, u32, u32)>) -> Vec<(u32, u32, u32, u32)> {
    boxes.iter()
        .map(|(start_x, start_y, end_x, end_y)| {
            // for each side, remove the rows / columns that are fully the same color
            let mut new_start_x = start_x.clone();
            let mut new_start_y = start_y.clone();
            let mut new_end_x = end_x.clone();
            let mut new_end_y = end_y.clone();

            for i in *start_x..*end_x {
                let mut same_color = true;
                for j in *start_y..*end_y {
                    if img.get_pixel(i, j) != img.get_pixel(new_start_x, new_start_y) {
                        same_color = false;
                        break;
                    }
                }
                if same_color {
                    new_start_x += 1;
                } else {
                    break;
                }
            }

            for i in (*start_x..*end_x).rev() {
                let mut same_color = true;
                for j in *start_y..*end_y {
                    if img.get_pixel(i, j) != img.get_pixel(new_end_x, new_end_y) {
                        same_color = false;
                        break;
                    }
                }
                if same_color {
                    new_end_x -= 1;
                } else {
                    break;
                }
            }

            for j in *start_y..*end_y {
                let mut same_color = true;
                for i in *start_x..*end_x {
                    if img.get_pixel(i, j) != img.get_pixel(new_start_x, new_start_y) {
                        same_color = false;
                        break;
                    }
                }
                if same_color {
                    new_start_y += 1;
                } else {
                    break;
                }
            }

            for j in (*start_y..*end_y).rev() {
                let mut same_color = true;
                for i in *start_x..*end_x {
                    if img.get_pixel(i, j) != img.get_pixel(new_end_x, new_end_y) {
                        same_color = false;
                        break;
                    }
                }
                if same_color {
                    new_end_y -= 1;
                } else {
                    break;
                }
            }

            return (new_start_x, new_start_y, new_end_x, new_end_y);
        }).collect()
}

// drop out parent boxes that contain other boxes
fn filter_parents(boxes: Vec<(u32, u32, u32, u32)>) -> Vec<(u32, u32, u32, u32)> {
    let mut filtered_boxes = Vec::new();
    for (start_x, start_y, end_x, end_y) in boxes.iter() {
        let mut is_parent = false;
        for (start_x1, start_y1, end_x1, end_y1) in boxes.iter() {
            if start_x == start_x1 && start_y == start_y1 && end_x == end_x1 && end_y == end_y1 {
                continue;
            }
            if start_x1 >= start_x && start_y1 >= start_y && end_x1 <= end_x && end_y1 <= end_y {
                is_parent = true;
                break;
            }
        }
        if !is_parent {
            filtered_boxes.push((start_x.clone(), start_y.clone(), end_x.clone(), end_y.clone()));
        }
    }

    return filtered_boxes;

}

fn filter_children(boxes: Vec<(u32, u32, u32, u32)>) -> Vec<(u32, u32, u32, u32)> {
    let mut filtered_boxes = Vec::new();
    for (start_x, start_y, end_x, end_y) in boxes.iter() {
        let mut is_child = false;
        for (start_x1, start_y1, end_x1, end_y1) in boxes.iter() {
            if start_x == start_x1 && start_y == start_y1 && end_x == end_x1 && end_y == end_y1 {
                continue;
            }
            if start_x1 <= start_x && start_y1 <= start_y && end_x1 >= end_x && end_y1 >= end_y {
                is_child = true;
                break;
            }
        }
        if !is_child {
            filtered_boxes.push((start_x.clone(), start_y.clone(), end_x.clone(), end_y.clone()));
        }
    }

    return filtered_boxes;
}

#[derive(Debug,PartialEq,Copy,Clone)]
enum EdgeType {
    Before, After, None, Both
}

fn detect_edge_types(image: &DynamicImage, lines: &Vec<Vec<(u32, u32)>>, direction: Direction) -> Vec<Vec<(u32, u32, EdgeType)>> {
    let lines_with_index = lines.iter().enumerate().collect::<Vec<_>>();
    let mapped_lines: Vec<Vec<(u32, u32, EdgeType)>> = lines_with_index
        .par_iter()
        .map(|(index, edge)| {
            let mut edges = Vec::new();

            for (start, end) in edge.iter() {
                let length = match direction {
                    Direction::Horizontal => image.height(),
                    Direction::Vertical => image.width(),
                };

                let mut before = false;
                let mut before_count = 0;
                let mut after = false;
                let mut after_count = 0;

                let min_pixels = if direction == Direction::Horizontal {
                    if (end - start) > 100 {
                        4
                    } else {
                        1
                    }
                } else {
                    1
                };
                
                if *index > 0 {
                    for i in (*start+1)..*end {
                        let (x, y, prev_x, prev_y) = match direction {
                            Direction::Horizontal => (i, (*index-1)as u32, i-1, (*index-1) as u32),
                            Direction::Vertical => ((*index-1) as u32, i, (*index-1) as u32, i-1),
                        };
                        if image.get_pixel(x, y) != image.get_pixel(prev_x, prev_y) {
                            before_count += 1;
                            if before_count > min_pixels {
                                after = true;
                                break;
                            }
                        }
                    }
                }

                if (*index as u32) < length-1 {
                    for i in (*start+1)..*end {
                        let (x, y, next_x, next_y) = match direction {
                            Direction::Horizontal => (i, (*index+1) as u32, i-1, (*index+1) as u32),
                            Direction::Vertical => ((*index+1) as u32, i, (*index+1) as u32, i-1),
                        };
                        if image.get_pixel(x, y) != image.get_pixel(next_x, next_y) {
                            after_count += 1;
                            if after_count > min_pixels {
                                before = true;
                                break;
                            }
                        }
                    }
                }

                let edge_type = match (before, after) {
                    (true, true) => EdgeType::Both,
                    (true, false) => EdgeType::Before,
                    (false, true) => EdgeType::After,
                    (false, false) => EdgeType::None,
                };
                edges.push((*start, *end, edge_type));
            }

            edges
         })
         .collect();
    
    println!("Mapped lines {:?}", mapped_lines.iter().map(|x| x.len()).sum::<usize>());
    println!("input lines outer {:?}", lines.len());
    println!("mapped lines outer {:?}", mapped_lines.len());
    return mapped_lines;
}

fn deduplicate_captured_edges(edges: &Vec<Vec<(u32, u32, EdgeType)>>) -> Vec<Vec<(u32, u32, EdgeType)>> {
    let mut deduplicated_edges = Vec::new();
    for (idx, edge_line) in edges.iter().enumerate() {
        if idx == 0 || idx == edges.len()-1 {
            deduplicated_edges.push(edge_line.clone());
            continue;
        }

        let mut deduplicated_edges_for_idx = Vec::new();
        for (start, end, edgetype) in edge_line.iter() {
            // check last / next n lines
            let delta = 8;
            if *edgetype == EdgeType::Before {
                // check check previous few lines
                let check_start = cmp::max(0, idx as i32 - delta) as usize;
                let mut included = false;
                for i in check_start..idx {
                    let previous_line = &edges[i];
                    for (start_prev, end_prev, edgetype_prev) in previous_line.iter() {
                        if *edgetype_prev == EdgeType::Before {
                            if (*start >= *start_prev && *end <= *end_prev) {
                                included = true;
                                break;
                            }
                        }
                    }
                    if included {
                        break;
                    }
                }
                if !included {
                    deduplicated_edges_for_idx.push((start.clone(), end.clone(), *edgetype));
                }
            } else {
                let delta = 8;
                let check_end = cmp::min(edges.len(), idx + delta);
                let mut included = false;
                for i in idx+1..check_end {
                    let next_line = &edges[i];
                    for (start_next, end_next, edgetype_next) in next_line.iter() {
                        if *edgetype_next == EdgeType::After {
                            if (*start >= *start_next && *end <= *end_next) {
                                included = true;
                                break;
                            }
                        }
                    }
                    if included {
                        break;
                    }
                }
                if !included {
                    deduplicated_edges_for_idx.push((start.clone(), end.clone(), *edgetype));
                }
            }
        }
        deduplicated_edges.push(deduplicated_edges_for_idx);
    }

    return deduplicated_edges;
}

pub fn find_bounding_boxes_v2(image: &DynamicImage) -> (Vec<(u32, u32, u32, u32)>, Vec<(u32, u32, u32, u32)>) {
    let write_debug_images = cfg!(debug_assertions) || false;

    println!("Finding horizontal lines");
    let start_time = std::time::Instant::now();
    let horizontal_lines = find_horizontal_lines(image);
    if write_debug_images {
        println!("Horizontal lines {:?}", horizontal_lines.iter().map(|x| x.len()).sum::<usize>());
        draw_lines(&image.clone(), &horizontal_lines, Direction::Horizontal, image::Rgba([255, 0, 0, 255])).save("/tmp/swiftmouse_0_horizontal_lines.png").unwrap();
    }
    println!("Elapsed: {:?}", start_time.elapsed());

    println!("Finding vertical lines");
    let start_time = std::time::Instant::now();
    let vertical_lines = find_vertical_lines(image);
    if write_debug_images {
        println!("Vertical lines {:?}", vertical_lines.iter().map(|x| x.len()).sum::<usize>());
        draw_lines(&image.clone(), &vertical_lines, Direction::Vertical, image::Rgba([255, 0, 0, 255])).save("/tmp/swiftmouse_0_vertical_lines.png").unwrap();
    }
    println!("Elapsed: {:?}", start_time.elapsed());
    
    println!("Detect horizontal edge types");
    let start_time = std::time::Instant::now();
    let horizontal_edges = detect_edge_types(image, &horizontal_lines, Direction::Horizontal);
    if write_debug_images {
        // filter to only before lines
        let before_lines = horizontal_edges.iter().map(|x| x.iter().filter(|(_, _, edge_type)| *edge_type == EdgeType::Before).map(|(start, end, _)| (start.clone(), end.clone())).collect::<Vec<_>>()).collect::<Vec<_>>();
        draw_lines(&image.clone(), &before_lines, Direction::Horizontal, image::Rgba([255, 0, 0, 255])).save("/tmp/swiftmouse_0_horizontal_lines_before.png").unwrap();
        let after_lines: Vec<Vec<(u32, u32)>> = horizontal_edges.iter().map(|x| x.iter().filter(|(_, _, edge_type)| *edge_type == EdgeType::After).map(|(start, end, _)| (start.clone(), end.clone())).collect::<Vec<_>>()).collect::<Vec<_>>();
        draw_lines(&image.clone(), &after_lines, Direction::Horizontal, image::Rgba([255, 0, 0, 255])).save("/tmp/swiftmouse_0_horizontal_lines_after.png").unwrap();
        let both_lines: Vec<Vec<(u32, u32)>> = horizontal_edges.iter().map(|x| x.iter().filter(|(_, _, edge_type)| *edge_type == EdgeType::Both).map(|(start, end, _)| (start.clone(), end.clone())).collect::<Vec<_>>()).collect::<Vec<_>>();
        draw_lines(&image.clone(), &both_lines, Direction::Horizontal, image::Rgba([255, 0, 0, 255])).save("/tmp/swiftmouse_0_horizontal_lines_both.png").unwrap();
        let none_lines: Vec<Vec<(u32, u32)>> = horizontal_edges.iter().map(|x| x.iter().filter(|(_, _, edge_type)| *edge_type == EdgeType::None).map(|(start, end, _)| (start.clone(), end.clone())).collect::<Vec<_>>()).collect::<Vec<_>>();
        draw_lines(&image.clone(), &none_lines, Direction::Horizontal, image::Rgba([255, 0, 0, 255])).save("/tmp/swiftmouse_0_horizontal_lines_none.png").unwrap();
    }
    println!("Elapsed: {:?}", start_time.elapsed());

    println!("Detect vertical edge types");
    let vertical_edges = detect_edge_types(image, &vertical_lines, Direction::Vertical);
    if write_debug_images {
        // filter to only before lines
        let before_lines = vertical_edges.iter().map(|x| x.iter().filter(|(_, _, edge_type)| *edge_type == EdgeType::Before).map(|(start, end, _)| (start.clone(), end.clone())).collect::<Vec<_>>()).collect::<Vec<_>>();
        draw_lines(&image.clone(), &before_lines, Direction::Vertical, image::Rgba([255, 0, 0, 255])).save("/tmp/swiftmouse_0_vertical_lines_before.png").unwrap();
        let after_lines: Vec<Vec<(u32, u32)>> = vertical_edges.iter().map(|x| x.iter().filter(|(_, _, edge_type)| *edge_type == EdgeType::After).map(|(start, end, _)| (start.clone(), end.clone())).collect::<Vec<_>>()).collect::<Vec<_>>();
        draw_lines(&image.clone(), &after_lines, Direction::Vertical, image::Rgba([255, 0, 0, 255])).save("/tmp/swiftmouse_0_vertical_lines_after.png").unwrap();
        let both_lines: Vec<Vec<(u32, u32)>> = vertical_edges.iter().map(|x| x.iter().filter(|(_, _, edge_type)| *edge_type == EdgeType::Both).map(|(start, end, _)| (start.clone(), end.clone())).collect::<Vec<_>>()).collect::<Vec<_>>();
        draw_lines(&image.clone(), &both_lines, Direction::Vertical, image::Rgba([255, 0, 0, 255])).save("/tmp/swiftmouse_0_vertical_lines_both.png").unwrap();
        let none_lines: Vec<Vec<(u32, u32)>> = vertical_edges.iter().map(|x| x.iter().filter(|(_, _, edge_type)| *edge_type == EdgeType::None).map(|(start, end, _)| (start.clone(), end.clone())).collect::<Vec<_>>()).collect::<Vec<_>>();
        draw_lines(&image.clone(), &none_lines, Direction::Vertical, image::Rgba([255, 0, 0, 255])).save("/tmp/swiftmouse_0_vertical_lines_none.png").unwrap();
    }
    println!("Elapsed: {:?}", start_time.elapsed());

    // keep only after and before
    println!("Filtering edges horizontally");
    let start_time = std::time::Instant::now();
    let filtered_horizontal_edges = horizontal_edges.iter().map(|x| x.iter().filter(|(_, _, edge_type)| *edge_type == EdgeType::Before || *edge_type == EdgeType::After).map(|(start, end, edgetype)| (start.clone(), end.clone(), edgetype.clone())).collect::<Vec<_>>()).collect::<Vec<_>>();
    println!("Elapsed: {:?}", start_time.elapsed());

    println!("Filtering edges vertically");
    let start_time = std::time::Instant::now();
    let filtered_vertical_edges = vertical_edges.iter().map(|x| x.iter().filter(|(_, _, edge_type)| *edge_type == EdgeType::Before || *edge_type == EdgeType::After || *edge_type == EdgeType::Both).map(|(start, end, edgetype)| (start.clone(), end.clone(), edgetype.clone())).collect::<Vec<_>>()).collect::<Vec<_>>();
    println!("Elapsed: {:?}", start_time.elapsed());

    if write_debug_images {
        println!("Filtered horizontal edges {:?}", filtered_horizontal_edges.iter().map(|x| x.len()).sum::<usize>());
        println!("Filtered vertical edges {:?}", filtered_vertical_edges.iter().map(|x| x.len()).sum::<usize>());
    }

    println!("Deduplicating horizontal edges");
    let start_time = std::time::Instant::now();
    let deduplicated_horizontal_edges = deduplicate_captured_edges(&filtered_horizontal_edges);
    println!("Elapsed: {:?}", start_time.elapsed());

    println!("Deduplicating vertical edges");
    let start_time = std::time::Instant::now();
    let deduplicated_vertical_edges = deduplicate_captured_edges(&filtered_vertical_edges);
    println!("Elapsed: {:?}", start_time.elapsed());

    if write_debug_images {
        let before_lines_deduplicated = deduplicated_horizontal_edges.iter().map(|x| x.iter().filter(|(_, _, edge_type)| *edge_type == EdgeType::Before).map(|(start, end, _)| (start.clone(), end.clone())).collect::<Vec<_>>()).collect::<Vec<_>>();
        draw_lines(&image.clone(), &before_lines_deduplicated, Direction::Horizontal, image::Rgba([255, 0, 0, 255])).save("/tmp/swiftmouse_0_horizontal_lines_before_deduplicated.png").unwrap();
        let after_lines_deduplicated: Vec<Vec<(u32, u32)>> = deduplicated_horizontal_edges.iter().map(|x| x.iter().filter(|(_, _, edge_type)| *edge_type == EdgeType::After).map(|(start, end, _)| (start.clone(), end.clone())).collect::<Vec<_>>()).collect::<Vec<_>>();
        draw_lines(&image.clone(), &after_lines_deduplicated, Direction::Horizontal, image::Rgba([255, 0, 0, 255])).save("/tmp/swiftmouse_0_horizontal_lines_after_deduplicated.png").unwrap();

        let before_lines_deduplicated = deduplicated_vertical_edges.iter().map(|x| x.iter().filter(|(_, _, edge_type)| *edge_type == EdgeType::Before).map(|(start, end, _)| (start.clone(), end.clone())).collect::<Vec<_>>()).collect::<Vec<_>>();
        draw_lines(&image.clone(), &before_lines_deduplicated, Direction::Vertical, image::Rgba([255, 0, 0, 255])).save("/tmp/swiftmouse_0_vertical_lines_before_deduplicated.png").unwrap();
        let after_lines_deduplicated: Vec<Vec<(u32, u32)>> = deduplicated_vertical_edges.iter().map(|x| x.iter().filter(|(_, _, edge_type)| *edge_type == EdgeType::After).map(|(start, end, _)| (start.clone(), end.clone())).collect::<Vec<_>>()).collect::<Vec<_>>();
        draw_lines(&image.clone(), &after_lines_deduplicated, Direction::Vertical, image::Rgba([255, 0, 0, 255])).save("/tmp/swiftmouse_0_vertical_lines_after_deduplicated.png").unwrap();

        println!("Deduplicated horizontal edges {:?}", deduplicated_horizontal_edges.iter().map(|x| x.len()).sum::<usize>());
        println!("Deduplicated vertical edges {:?}", deduplicated_vertical_edges.iter().map(|x| x.len()).sum::<usize>());
    }

    // give each line an id
    let mut new_horizontal_lines_id = 0;
    let mut horizontal_lines: Vec<_> = Vec::new();
    let mut horizontal_lines_total: Vec<(u32, u32, u32, EdgeType, u32)> = Vec::new();
    for (i, line) in deduplicated_horizontal_edges.iter().enumerate() {
        let mut horizontal_lines_local = Vec::new();
        for (start, end, edgetype) in line.iter() {
            horizontal_lines_local.push((i.clone(), start.clone(), end.clone(), *edgetype, new_horizontal_lines_id.clone()));
            horizontal_lines_total.push((i.clone() as u32, start.clone(), end.clone(), *edgetype, new_horizontal_lines_id.clone()));
            new_horizontal_lines_id += 1;
        }
        horizontal_lines.push(horizontal_lines_local);
    }

    let mut new_vertical_lines_id = 0;
    let mut vertical_lines: Vec<_> = Vec::new();
    let mut vertical_lines_total: Vec<(u32, u32, u32, EdgeType, u32)> = Vec::new();
    for (i, line) in deduplicated_vertical_edges.iter().enumerate() {
        let mut vertical_lines_local = Vec::new();
        for (start, end, edgetype) in line.iter() {
            vertical_lines_local.push((i.clone(), start.clone(), end.clone(), *edgetype, new_vertical_lines_id.clone()));
            vertical_lines_total.push((i.clone() as u32, start.clone(), end.clone(), *edgetype, new_vertical_lines_id.clone()));
            new_vertical_lines_id += 1;
        }
        vertical_lines.push(vertical_lines_local);
    }


    let start = std::time::Instant::now();
    println!("Finding intersections with {:?} horizontal lines and {:?} vertical lines", horizontal_lines.len(), vertical_lines.len());
    let mut horizontal_intersections = vec![Vec::new(); new_horizontal_lines_id as usize];
    let mut vertical_intersections = vec![Vec::new(); new_vertical_lines_id as usize];
    let mut intersections: Vec<(u32, u32, EdgeType, EdgeType)> = Vec::new();
    // for horizontal_line_y in 0..image.height() {
    //     for (_, horizontal_start_x, horizontal_end_x, edgetype_h, id_h) in horizontal_lines[horizontal_line_y as usize].iter() {
    //         for vertical_line_x in (*horizontal_start_x)..(*horizontal_end_x) {
    //             for (_, vertical_start_y, vertical_end_y, edgetype_v, id_v) in vertical_lines[vertical_line_x as usize].iter() {
    //                 if horizontal_line_y < *vertical_start_y {
    //                     continue
    //                 }
    //                 if horizontal_line_y > *vertical_end_y {
    //                     break
    //                 }
    //                 
    //                 vertical_intersections[*id_v as usize].push(*id_h);
    //                 horizontal_intersections[*id_h as usize].push(*id_v);
    //                 
    //                 intersections.push((vertical_line_x, horizontal_line_y));
    //             }
    //         }
    //     }
    // }
    for (horizontal_line_y, start_x, end_x, _, id_h) in horizontal_lines_total.iter() {
        for (vertical_line_x, start_y, end_y, _, id_v) in vertical_lines_total.iter() {
            if *horizontal_line_y > *start_y && *horizontal_line_y < *end_y && *vertical_line_x > *start_x && *vertical_line_x < *end_x {
                // if *vertical_line_x > 400 {
                //     continue
                // }
                vertical_intersections[*id_v as usize].push(*id_h);
                horizontal_intersections[*id_h as usize].push(*id_v);
                // get horizontal edgetype
                // get vertical edgetype
                let (_, _, _, et_h, _) = horizontal_lines_total[*id_h as usize];
                let (_, _, _, et_v, _) = vertical_lines_total[*id_v as usize];
                intersections.push((*vertical_line_x, *horizontal_line_y, et_h, et_v));
            }
        }
    }
    println!("Elapsed: {:?}", start.elapsed());


    if write_debug_images {
        println!("Horizontal intersections {:?}", horizontal_intersections.iter().map(|x| x.len()).sum::<usize>());
        println!("Vertical intersections {:?}", vertical_intersections.iter().map(|x| x.len()).sum::<usize>());
        
        let mut debug_img = image.clone();
        for (x, y, et_h, et_v) in intersections.iter() {
            let color = match (et_h, et_v) {
                (EdgeType::Before, EdgeType::Before) => image::Rgba([255, 0, 0, 255]),
                (EdgeType::Before, EdgeType::After) => image::Rgba([0, 255, 0, 255]),
                (EdgeType::After, EdgeType::Before) => image::Rgba([0, 0, 255, 255]),
                (EdgeType::After, EdgeType::After) => image::Rgba([255, 255, 0, 255]),
                _ => image::Rgba([255, 255, 255, 255]),
            };
            debug_img.put_pixel(*x, *y, color);
        }
        debug_img.save("/tmp/swiftmouse_0_intersections.png").unwrap();
    }

    println!("Finding boxes");
    let start_time = std::time::Instant::now();
    let boxes: Vec<(u32, u32, u32, u32)> = horizontal_intersections
        .par_iter()
        .enumerate()
        .filter(|(id, _)| horizontal_lines_total[*id].3 == EdgeType::Before)
        .map(|(horizontal_line_id, vertical_lines)| {
            let mut boxes = Vec::new();
            for vertical_line_id in vertical_lines.iter() {
                if vertical_lines_total[*vertical_line_id as usize].3 != EdgeType::Before && vertical_lines_total[*vertical_line_id as usize].3 != EdgeType::Both {
                    continue;
                }
    
                // println!("Checking before-vertical line {:?}", vertical_line_id);
    
                for vertical_line_id1 in vertical_lines.iter() {
                    if vertical_lines_total[*vertical_line_id1 as usize].3 != EdgeType::After && vertical_lines_total[*vertical_line_id1 as usize].3 != EdgeType::Both {
                        continue;
                    }
                    // println!("Checking after-vertical line {:?}", vertical_line_id1);
    
                    let lines_1 = &vertical_intersections[*vertical_line_id as usize];
                    let lines_2 = &vertical_intersections[*vertical_line_id1 as usize];
                    for line in lines_1.iter() {
                        // if after
                        if lines_2.contains(line) && horizontal_lines_total[*line as usize].3 == EdgeType::After {
                            let id_ht = &horizontal_line_id;
                            let id_vl = vertical_line_id;
                            let id_hb = line;
                            let id_vr = vertical_line_id1;
                            let (start_y, _, _, _, _) = horizontal_lines_total[*id_ht as usize];
                            let (end_y, _, _, _, _) = horizontal_lines_total[*id_hb as usize];
                            let (start_x, _, _, _, _) = vertical_lines_total[*id_vl as usize];
                            let (end_x, _, _, _, _) = vertical_lines_total[*id_vr as usize];
    
                            // println!("Found box {:?} {:?} {:?} {:?}", start_x, start_y, end_x, end_y);
    
                            if start_x < end_x && start_y < end_y {
                                let width = end_x - start_x;
                                let height = end_y - start_y;
                                // println!("Dimensions {:?} x {:?}", width, height);
                                if width < 5 || height < 12 {
                                    continue;
                                }
                                boxes.push((start_x, start_y, end_x, end_y));
                            }
                        }
                    }
                }
            }
            boxes
        })
        .flatten()
        .collect();


    println!("Elapsed: {:?}", start_time.elapsed());
    println!("Found {:?} boxes", boxes.len());

    if write_debug_images {
        let mut debug_img = image.clone();
        for (start_x, start_y, end_x, end_y) in boxes.iter() {
            draw_horizontal_line_colored(&mut debug_img, *start_y, *start_x, *end_x, image::Rgba([255, 0, 0, 255]));
            draw_horizontal_line_colored(&mut debug_img, *end_y, *start_x, *end_x, image::Rgba([255, 255, 0, 255]));
            draw_vertical_line_colored(&mut debug_img, *start_x, *start_y, *end_y, image::Rgba([255, 0, 255, 255]));
            draw_vertical_line_colored(&mut debug_img, *end_x, *start_y, *end_y, image::Rgba([0, 255, 255, 255]));
        }
        debug_img.save("/tmp/swiftmouse_0_boxes_initial.png").unwrap();
    }
    
    println!("Filtering boxes");
    let start = std::time::Instant::now();
    let filtered_boxes: Vec<(u32, u32, u32, u32)> = boxes.par_iter()
        .filter(|(start_x, start_y, end_x, end_y)| {
            // println!("Checking box {:?} {:?} {:?} {:?}", start_x, start_y, end_x, end_y);

            // check if middle horizontal line is same color
            let middle_y = (start_y + end_y) / 2;
            let middle_x = (start_x + end_x) / 2;
            
            // if there is a padding of 40 pixels same color on one middle side continue
            let padding = 40;
            let mut same_color = true;
            let start_color = image.get_pixel(middle_x, *start_y);
            for y in *start_y..cmp::min(*start_y+padding, *end_y) {
                if image.get_pixel(middle_x, y) != start_color {
                    same_color = false;
                    break;
                }
            }
            if same_color {
                // println!("Same color top {:?}", start_y);
                return false;
            }

            let mut same_color = true;
            let start_color = image.get_pixel(middle_x, *end_y);
            for y in cmp::max((*end_y as i32) - padding as i32, *start_y as i32)..(*end_y as i32) {
                let y = y as u32;
                if image.get_pixel(middle_x, y) != start_color {
                    same_color = false;
                    break;
                }
            }
            if same_color {
                // println!("Same color bottom {:?}", end_y);
                return false;
            }

            let mut same_color = true;
            let start_color = image.get_pixel(*start_x, middle_y);
            for x in *start_x..cmp::min(*start_x+padding, *end_x) {
                if image.get_pixel(x, middle_y) != start_color {
                    same_color = false;
                    break;
                }
            }
            if same_color {
                // println!("Same color left {:?}", start_x);
                return false;
            }

            let mut same_color = true;
            let start_color = image.get_pixel(*end_x, middle_y);
            for x in cmp::max((*end_x as i32) - padding as i32, *start_x as i32)..*end_x as i32 {
                let x = x as u32;
                if image.get_pixel(x, middle_y) != start_color {
                    same_color = false;
                    break;
                }
            }
            if same_color {
                // println!("Same color right {:?}", end_x);
                return false;
            }

            return true;
        })
        .map(|x| x.clone())
        .collect();
    println!("Filtered boxes {:?}", filtered_boxes.len());
    println!("Elapsed: {:?}", start.elapsed());

    // // debug
    // let mut debug_img = image.clone();
    // for (start_x, start_y, end_x, end_y) in filtered_boxes.iter() {
    //     draw_horizontal_line_colored(&mut debug_img, *start_y, *start_x, *end_x, image::Rgba([255, 0, 0, 255]));
    //     draw_horizontal_line_colored(&mut debug_img, *end_y, *start_x, *end_x, image::Rgba([255, 255, 0, 255]));
    //     draw_vertical_line_colored(&mut debug_img, *start_x, *start_y, *end_y, image::Rgba([255, 0, 255, 255]));
    //     draw_vertical_line_colored(&mut debug_img, *end_x, *start_y, *end_y, image::Rgba([0, 255, 255, 255]));
    // }
    // debug_img.save("/tmp/swiftmouse_0_filtered_boxes.png").unwrap();

    let mut sorted_boxes = vec![Vec::new(); image.height() as usize];
    let mut big_boxes = Vec::new();
    let mut small_boxes = Vec::new();
    for (start_x, start_y, end_x, end_y) in filtered_boxes.iter() {
        let max_textfragment_height = 35;
        let max_textfragment_height_2 = 35;
        if end_y - start_y > max_textfragment_height {
            big_boxes.push((start_x.clone(), start_y.clone(), end_x.clone(), end_y.clone()));
        } else if end_y - start_y > max_textfragment_height_2 {
             // if the color around the box is the same in the padding area
            let (padded_startx, padded_starty, padded_endx, padded_endy) = (cmp::max(0, *start_x as i32 - 4) as u32, cmp::max(0, *start_y as i32 - 4) as u32, end_x + 4, end_y + 4);
            let mut padding_dirty = false;
            for y in padded_starty..padded_endy {
                for x in padded_startx..padded_endx {
                    if x < 0 || y < 0 || x >= image.width() || y >= image.height() {
                        continue;
                    }
                    if x < *start_x || x > *end_x || y < *start_y || y > *end_y {
                        if image.get_pixel(x, y) != image.get_pixel(*start_x, *start_y) {
                            padding_dirty = true;
                            break;
                        }
                    }
                }
                if padding_dirty {
                    break;
                }
            }

            // if there is no collision it is a big box, else a small box
            if !padding_dirty {
                big_boxes.push((start_x.clone(), start_y.clone(), end_x.clone(), end_y.clone()));
            } else {
                small_boxes.push((start_x.clone(), start_y.clone(), end_x.clone(), end_y.clone()));
            }
        } else {
            small_boxes.push((start_x.clone(), start_y.clone(), end_x.clone(), end_y.clone()));
        }
    }

    let mut merged_texts = Vec::new();
    for (start_x, start_y, end_x, end_y) in small_boxes.iter() {
        sorted_boxes[*start_y as usize].push((start_x.clone(), start_y.clone(), end_x.clone(), end_y.clone()));
    }

    let start = std::time::Instant::now();
    println!("Resolving texts");
    for boxes in sorted_boxes.iter() {
        // group by end_y
        let mut sorted_boxes_end_y = vec![Vec::new(); image.height() as usize];
        for (start_x, start_y, end_x, end_y) in boxes.iter() {
            sorted_boxes_end_y[*end_y as usize].push((start_x.clone(), start_y.clone(), end_x.clone(), end_y.clone()));
        }

        for boxes in sorted_boxes_end_y.iter() {
            if boxes.len() > 1 {
                // let filtered_boxes = filter_Varents(boxes.clone());
                // println!("After parent filter {:?}", filtered_boxes.len());
                let merged_boxes = merge_boxes(boxes.clone(), 5, 0);
                // println!("After merge {:?}", merged_boxes.len());
                let merged_boxes = filter_children(merged_boxes);
                // println!("After child filter {:?}", merged_boxes.len());
                merged_texts.extend(merged_boxes);
            }
        }
    }
    println!("Elapsed: {:?}", start.elapsed());
    println!("Merged texts {:?}", merged_texts.len());

    if write_debug_images {
        let mut debug_img = image.clone();
        for (start_x, start_y, end_x, end_y) in big_boxes.iter() {
            draw_horizontal_line_colored(&mut debug_img, *start_y, *start_x, *end_x, image::Rgba([255, 0, 0, 255]));
            draw_horizontal_line_colored(&mut debug_img, *end_y, *start_x, *end_x, image::Rgba([255, 255, 0, 255]));
            draw_vertical_line_colored(&mut debug_img, *start_x, *start_y, *end_y, image::Rgba([255, 0, 255, 255]));
            draw_vertical_line_colored(&mut debug_img, *end_x, *start_y, *end_y, image::Rgba([0, 255, 255, 255]));
        }
        debug_img.save("/tmp/swiftmouse_0_big_boxes.png").unwrap();
        
        let mut debug_img = image.clone();
        for (start_x, start_y, end_x, end_y) in small_boxes.iter() {
            draw_horizontal_line_colored(&mut debug_img, *start_y, *start_x, *end_x, image::Rgba([255, 0, 0, 255]));
            draw_horizontal_line_colored(&mut debug_img, *end_y, *start_x, *end_x, image::Rgba([255, 255, 0, 255]));
            draw_vertical_line_colored(&mut debug_img, *start_x, *start_y, *end_y, image::Rgba([255, 0, 255, 255]));
            draw_vertical_line_colored(&mut debug_img, *end_x, *start_y, *end_y, image::Rgba([0, 255, 255, 255]));
        }
        debug_img.save("/tmp/swiftmouse_0_small_boxes.png").unwrap();
    }   
    
    // remove padding texts
    let start = std::time::Instant::now();
    let no_padding_boxes = remove_box_padding(image, merged_texts.clone());
    println!("No padding boxes {:?}", no_padding_boxes.len());
    println!("Elapsed: {:?}", start.elapsed());

    if write_debug_images {
        let mut debug_img = image.clone();
        for (start_x, start_y, end_x, end_y) in no_padding_boxes.iter() {
            draw_horizontal_line_colored(&mut debug_img, *start_y, *start_x, *end_x, image::Rgba([255, 0, 0, 255]));
            draw_horizontal_line_colored(&mut debug_img, *end_y, *start_x, *end_x, image::Rgba([255, 255, 0, 255]));
            draw_vertical_line_colored(&mut debug_img, *start_x, *start_y, *end_y, image::Rgba([255, 0, 255, 255]));
            draw_vertical_line_colored(&mut debug_img, *end_x, *start_y, *end_y, image::Rgba([0, 255, 255, 255]));
        }
        debug_img.save("/tmp/swiftmouse_0_no_padding_boxes.png").unwrap();
    }

    // for each text segment cut out long same colored sectinos
    let mut new_boxes = Vec::new();
    for (start_x, start_y, end_x, end_y) in no_padding_boxes.iter() {
        let mut sections = Vec::new();
        let mut sx = start_x.clone();
        let mut same_colored_count = 0;
        let same_colored_threshhold = 10;
        let mut is_segment_started = true;

        for x in *start_x..*end_x {
            let mut same_color = true;
            for y in *start_y..*end_y {
                if image.get_pixel(x, y) != image.get_pixel(x, *start_y) {
                    same_color = false;
                    break;
                }
            }
            if same_color {
                if is_segment_started {
                    same_colored_count += 1;
                    if same_colored_count > same_colored_threshhold {
                        is_segment_started = false;
                        sections.push((sx, x));
                    }
                }
            } else {
                same_colored_count = 0;
                if !is_segment_started {
                    sx = x;
                    is_segment_started = true;
                }
            }
        }

        if is_segment_started {
            sections.push((sx, end_x.clone()));
        }

        for (sx, ex) in sections.iter() {
            new_boxes.push((sx.clone(), start_y.clone(), ex.clone(), end_y.clone()));
        }
    }

    if write_debug_images {
        let mut debug_img = image.clone();
        for (start_x, start_y, end_x, end_y) in new_boxes.iter() {
            draw_horizontal_line_colored(&mut debug_img, *start_y, *start_x, *end_x, image::Rgba([255, 0, 0, 255]));
            draw_horizontal_line_colored(&mut debug_img, *end_y, *start_x, *end_x, image::Rgba([255, 255, 0, 255]));
            draw_vertical_line_colored(&mut debug_img, *start_x, *start_y, *end_y, image::Rgba([255, 0, 255, 255]));
            draw_vertical_line_colored(&mut debug_img, *end_x, *start_y, *end_y, image::Rgba([0, 255, 255, 255]));
        }
        debug_img.save("/tmp/swiftmouse_0_new_boxes.png").unwrap();
    }

    // remove padding once more
    let no_padding_boxes = remove_box_padding(image, new_boxes.clone());
    println!("No padding boxes {:?}", no_padding_boxes.len());

    
    if write_debug_images {
        let mut debug_img = image.clone();
        for (start_x, start_y, end_x, end_y) in no_padding_boxes.iter() {
            draw_horizontal_line_colored(&mut debug_img, *start_y, *start_x, *end_x, image::Rgba([255, 0, 0, 255]));
            draw_horizontal_line_colored(&mut debug_img, *end_y, *start_x, *end_x, image::Rgba([255, 255, 0, 255]));
            draw_vertical_line_colored(&mut debug_img, *start_x, *start_y, *end_y, image::Rgba([255, 0, 255, 255]));
            draw_vertical_line_colored(&mut debug_img, *end_x, *start_y, *end_y, image::Rgba([0, 255, 255, 255]));
        }
        debug_img.save("/tmp/swiftmouse_0_no_padding_boxes_2.png").unwrap();
    }

    let smallboxes = filter_children(no_padding_boxes.clone());
    println!("No children boxes {:?}", smallboxes.len());
    // let big_boxes_filtered = filter_parents(big_boxes.clone());
    // println!("No parents boxes {:?}", big_boxes_filtered.len());
    // //debug
    // let mut debug_img = image.clone();
    // for (start_x, start_y, end_x, end_y) in big_boxes_filtered.iter() {
    //     draw_horizontal_line_colored(&mut debug_img, *start_y, *start_x, *end_x, image::Rgba([255, 0, 0, 255]));
    //     draw_horizontal_line_colored(&mut debug_img, *end_y, *start_x, *end_x, image::Rgba([255, 255, 0, 255]));
    //     draw_vertical_line_colored(&mut debug_img, *start_x, *start_y, *end_y, image::Rgba([255, 0, 255, 255]));
    //     draw_vertical_line_colored(&mut debug_img, *end_x, *start_y, *end_y, image::Rgba([0, 255, 255, 255]));
    // }
    // debug_img.save("/tmp/swiftmouse_0_no_parent_big_boxes.png").unwrap();

    // // debug
    // let mut debug_img = image.clone();
    // for (start_x, start_y, end_x, end_y) in smallboxes.iter() {
    //     draw_horizontal_line_colored(&mut debug_img, *start_y, *start_x, *end_x, image::Rgba([255, 0, 0, 255]));
    //     draw_horizontal_line_colored(&mut debug_img, *end_y, *start_x, *end_x, image::Rgba([255, 255, 0, 255]));
    //     draw_vertical_line_colored(&mut debug_img, *start_x, *start_y, *end_y, image::Rgba([255, 0, 255, 255]));
    //     draw_vertical_line_colored(&mut debug_img, *end_x, *start_y, *end_y, image::Rgba([0, 255, 255, 255]));
    // }
    // debug_img.save("/tmp/swiftmouse_0_no_children_boxes.png").unwrap();

    // drop all big boxes with endx bigger than 70
    // let big_boxes = big_boxes.iter().filter(|(_, _, end_x, _)| *end_x < 70).map(|(start_x, start_y, end_x, end_y)| (start_x.clone(), start_y.clone(), end_x.clone(), end_y.clone())).collect::<Vec<_>>();

    if write_debug_images {
        let mut debug_img = image.clone();
        for (start_x, start_y, end_x, end_y) in big_boxes.iter() {
            draw_horizontal_line_colored(&mut debug_img, *start_y, *start_x, *end_x, image::Rgba([255, 0, 0, 255]));
            draw_horizontal_line_colored(&mut debug_img, *end_y, *start_x, *end_x, image::Rgba([255, 255, 0, 255]));
            draw_vertical_line_colored(&mut debug_img, *start_x, *start_y, *end_y, image::Rgba([255, 0, 255, 255]));
            draw_vertical_line_colored(&mut debug_img, *end_x, *start_y, *end_y, image::Rgba([0, 255, 255, 255]));
            // // draw greenline at offset
            // let offset = start_y + end_y % 1000;
            // draw_vertical_line_colored(&mut debug_img, *start_x + offset, *start_y, *end_y, image::Rgba([0, 255, 0, 255]));
        }
        debug_img.save("/tmp/swiftmouse_0_big_boxes_filtered.png").unwrap();
    }

    // filter boxes where horizontal sides are the same but vertical sizes overlap
    let start = std::time::Instant::now();
    println!("Filtering boxes 2");
    let mut filtered_boxes_2 = Vec::new();
    for (idx_1, (start_x_1, start_y_1, end_x_1, end_y_1)) in big_boxes.iter().enumerate() {
        let mut discard = false;
        for (idx_2, (start_x_2, start_y_2, end_x_2, end_y_2)) in big_boxes.iter().enumerate() {
            if idx_1 == idx_2 {
                continue;
            }
            
            if start_x_1 == start_x_2 && end_x_1 == end_x_2 {
                // if other box is inside this
                if start_y_1 <= start_y_2 && end_y_1 >= end_y_2 {
                    discard = true;
                    break;
                }
            }

            if start_y_1 == start_y_2 && end_y_1 == end_y_2 {
                // if other box is inside this
                if start_x_1 <= start_x_2 && end_x_1 >= end_x_2 {
                    discard = true;
                    break;
                }
            }
        }

        if !discard {
            filtered_boxes_2.push((start_x_1.clone(), start_y_1.clone(), end_x_1.clone(), end_y_1.clone()));
        }
    }
    println!("Filtered boxes 2 {:?}", filtered_boxes_2.len());
    println!("Elapsed: {:?}", start.elapsed());

    // debug
    let mut debug_img = image.clone();
    for (start_x, start_y, end_x, end_y) in filtered_boxes_2.iter() {
        draw_horizontal_line_colored(&mut debug_img, *start_y, *start_x, *end_x, image::Rgba([255, 0, 0, 255]));
        draw_horizontal_line_colored(&mut debug_img, *end_y, *start_x, *end_x, image::Rgba([255, 255, 0, 255]));
        draw_vertical_line_colored(&mut debug_img, *start_x, *start_y, *end_y, image::Rgba([255, 0, 255, 255]));
        draw_vertical_line_colored(&mut debug_img, *end_x, *start_y, *end_y, image::Rgba([0, 255, 255, 255]));
    }
    println!("Filtered boxes 2 {:?}", filtered_boxes_2.len());
    debug_img.save("/tmp/swiftmouse_0_filtered_boxes_2.png").unwrap();

    // merge big boxes w
    let start = std::time::Instant::now();
    println!("Merging boxes");
    // filter all parents
    let filtered_boxes_2 = filter_parents(filtered_boxes_2.clone());
    let merged_big_boxes = merge_boxes(filtered_boxes_2.clone(), 1, 0);

    // drop less than 10 wide
    let merged_big_boxes = merged_big_boxes.iter().filter(|(start_x, _, end_x, _)| end_x - start_x > 10).map(|(start_x, start_y, end_x, end_y)| (start_x.clone(), start_y.clone(), end_x.clone(), end_y.clone())).collect::<Vec<_>>();
    
    // discard boxes less than 10 wide
    // let merged_big_boxes = merged_big_boxes.iter().filter(|(start_x, _, end_x, _)| end_x - start_x > 10).map(|(start_x, start_y, end_x, end_y)| (start_x.clone(), start_y.clone(), end_x.clone(), end_y.clone())).collect::<Vec<_>>();
    println!("Merged boxes {:?}", merged_big_boxes.len());
    println!("Elapsed: {:?}", start.elapsed());

    // debug
    let mut debug_img = image.clone();
    for (start_x, start_y, end_x, end_y) in merged_big_boxes.iter() {
        draw_horizontal_line_colored(&mut debug_img, *start_y, *start_x, *end_x, image::Rgba([255, 0, 0, 255]));
        draw_horizontal_line_colored(&mut debug_img, *end_y, *start_x, *end_x, image::Rgba([255, 255, 0, 255]));
        draw_vertical_line_colored(&mut debug_img, *start_x, *start_y, *end_y, image::Rgba([255, 0, 255, 255]));
        draw_vertical_line_colored(&mut debug_img, *end_x, *start_y, *end_y, image::Rgba([0, 255, 255, 255]));
    }
    debug_img.save("/tmp/swiftmouse_0_merged_big_boxes.png").unwrap();

    // filter big boxes, if a big box intersects more than 2 small boxes discard
    let start = std::time::Instant::now();
    println!("Filtering big boxes");
    let mut filtered_big_boxes = Vec::new();
    for (start_x, start_y, end_x, end_y) in merged_big_boxes.iter() {
        let smallbox_threshold = 1;
        let mut smallbox_intersects_count = 0;
        let mut smallbox_contains_count = 0;
        for (small_start_x, small_start_y, small_end_x, small_end_y) in smallboxes.iter() {
            let intersects = small_end_x > start_x && end_x > small_start_x && small_end_y > start_y && end_y > small_start_y;
            if intersects {
                smallbox_intersects_count += 1;
            }
            let contains = small_start_x > start_x && small_end_x < end_x && small_start_y > start_y && small_end_y < end_y;
            if contains {
                // smallbox_contains_count += 1;
            }
        }
        let intersections_only = smallbox_intersects_count - smallbox_contains_count;

        if intersections_only <= smallbox_threshold {
            filtered_big_boxes.push((start_x.clone(), start_y.clone(), end_x.clone(), end_y.clone()));
        }
    }
    println!("Filtered big boxes {:?}", filtered_big_boxes.len());
    println!("Elapsed: {:?}", start.elapsed());

    // remove padding
    let mut filtered_big_boxes = remove_box_padding(image, filtered_big_boxes.clone());

    // filter out big boxes that are contained by small boxes
    let mut filtered_big_boxes_new = Vec::new();
    for (start_x, start_y, end_x, end_y) in filtered_big_boxes.iter() {
        let mut discard = false;
        for (small_start_x, small_start_y, small_end_x, small_end_y) in smallboxes.iter() {
            if small_start_x <= start_x && small_end_x >= end_x && small_start_y <= start_y && small_end_y >= end_y {
                discard = true;
                break;
            }
        }
        if !discard {
            filtered_big_boxes_new.push((start_x.clone(), start_y.clone(), end_x.clone(), end_y.clone()));
        }
    }
    filtered_big_boxes = filtered_big_boxes_new;

    return (smallboxes, filtered_big_boxes);
}

fn draw_horizontal_line_colored(image: &mut DynamicImage, y: u32, start_x: u32, end_x: u32, color: image::Rgba<u8>) {
    for x in start_x..end_x {
        if x >= image.width() {
            continue;
        }
        image.put_pixel(x, y, color);
    }
}

fn draw_vertical_line_colored(image: &mut DynamicImage, x: u32, start_y: u32, end_y: u32, color: image::Rgba<u8>) {
    for y in start_y..end_y {
        if y >= image.height() {
            continue;
        }
        image.put_pixel(x, y, color);
    }
}