use image::{DynamicImage, GenericImage};

pub fn draw_box(image: &mut DynamicImage, min_x: u32, min_y: u32, max_x: u32, max_y: u32, color: image::Rgba<u8>) {
    for x in min_x..max_x {
        for y in min_y..max_y {
            if x < 0 || y < 0 || x >= image.width() || y >= image.height() {
                continue;
            }

            if x == min_x || x == max_x-1 || y == min_y || y == max_y-1 {
                image.put_pixel(x, y, color);
            }
        }
    }
}