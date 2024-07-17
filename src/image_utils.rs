use image::{DynamicImage, GenericImage};

pub fn draw_box(image: &mut DynamicImage, min_x: usize, min_y: usize, max_x: usize, max_y: usize, color: image::Rgba<u8>) {
    for x in min_x..(max_x+1) {
        for y in min_y..(max_y+1) {
            if x >= image.width() as usize || y >= image.height() as usize {
                continue;
            }

            if x == min_x || x == max_x || y == min_y || y == max_y {
                image.put_pixel(x as u32, y as u32, color);
            }
        }
    }
}

pub fn draw_boxes(image: &mut DynamicImage, boxes: &[(usize, usize, usize, usize)], color: image::Rgba<u8>) {
    for (min_x, min_y, max_x, max_y) in boxes {
        draw_box(image, *min_x, *min_y, *max_x, *max_y, color);
    }
}