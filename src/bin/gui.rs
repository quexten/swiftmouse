use zbus::zvariant::Endian;
use zbus::zvariant::ReadBytes;

const SCREENSHOT_PATH: &str = "/tmp/screenshot.png";

fn read_boxes(stdin: &mut std::io::Stdin) -> Vec<(u32, u32, u32, u32)> {
    let boxes_len = stdin.read_u32(Endian::Little).unwrap();
    let mut boxes = Vec::new();
    for _ in 0..boxes_len {
        let x = stdin.read_u32(Endian::Little).unwrap();
        let y = stdin.read_u32(Endian::Little).unwrap();
        let width = stdin.read_u32(Endian::Little).unwrap();
        let height = stdin.read_u32(Endian::Little).unwrap();
        boxes.push((x, y, width, height));
    }
    boxes
}

#[tokio::main]
pub async fn main() {
    let mut stdin = std::io::stdin();
    let big_boxes = read_boxes(&mut stdin);
    let line_boxes = read_boxes(&mut stdin);
    let small_images = read_boxes(&mut stdin);
    let large_images = read_boxes(&mut stdin);
    
    // start autoclick session
    swiftmouse::autotype::start_autoclick_session().await.unwrap();
    // screen width and height
    swiftmouse::gui::show_gui( big_boxes, line_boxes, small_images, large_images, SCREENSHOT_PATH.to_string());

}