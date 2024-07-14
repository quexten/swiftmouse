use image::{DynamicImage, GenericImage};

pub fn draw_box(image: &mut DynamicImage, min_x: u32, min_y: u32, max_x: u32, max_y: u32, color: image::Rgba<u8>) {
    for x in min_x..max_x {
        for y in min_y..max_y {
            if x >= image.width() || y >= image.height() {
                continue;
            }

            if x == min_x || x == max_x-1 || y == min_y || y == max_y-1 {
                image.put_pixel(x, y, color);
            }
        }
    }
}

pub fn draw_boxes(image: &mut DynamicImage, boxes: &[(u32, u32, u32, u32)]) {
    for (min_x, min_y, max_x, max_y) in boxes {
        draw_box(image, *min_x, *min_y, *max_x, *max_y, image::Rgba([255, 0, 0, 255]));
    }
}