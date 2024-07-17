use std::{borrow::{Borrow, BorrowMut}, fmt::Error, sync::Arc, thread, time::Duration};

use ashpd::desktop::{print, screenshot};
use scap::{
    capturer::{Area, Capturer, Options, Point, Size},
    frame::Frame,
};
use tokio::{net::unix::pipe::{self, pipe}, sync::Mutex, time::timeout};
use xcap::Monitor;

pub struct PipewireCapturer {
    needs_screenshot: Arc<Mutex<bool>>,
    image_rx: tokio::sync::mpsc::Receiver<image::ImageBuffer<image::Rgb<u8>, Vec<u8>>>,
}

impl PipewireCapturer {
    pub async fn take_screenshot(&mut self) -> Result<image::ImageBuffer<image::Rgb<u8>, Vec<u8>>, Box<dyn std::error::Error>> {
        let mut needs_capture = self.needs_screenshot.lock().await;
        *needs_capture = true;
        println!("Needs capture: {:?}", *needs_capture);
        drop(needs_capture);
        println!("waiting for screenshot");
        let screenshot = timeout(Duration::from_secs(5), self.image_rx.recv()).await;
        match screenshot {
            Ok(res) => {
                match res {
                    Some(screenshot) => {
                        return Ok(screenshot);
                    }
                    None => {
                        println!("[PipewireCapturer] Failed to take screenshot");
                        return Err(Box::new(Error));
                    }
                }
            }
            Err(_) => {
                println!("[PipewireCapturer] Timeout");
                return Err(Box::new(Error));
            }
        }
    }
}

pub struct ScreenshotTool {
    pipewire_capturer: Arc<Mutex<Option<PipewireCapturer>>>,
    heartbeat_rx: Arc<Mutex<Option<tokio::sync::mpsc::Receiver<()>>>>,
}

impl ScreenshotTool {

    pub fn start(&mut self) {
        let timeout_rx = self.heartbeat_rx.clone();
        let pipewire_capture = self.pipewire_capturer.clone();
        tokio::spawn(async move {
            loop {
                let mut timeout_rx_opt = timeout_rx.lock().await;
                let timeout_rx_opt1 = timeout_rx_opt.borrow_mut();
                let rx = timeout_rx_opt1.as_mut();
                let mut close = false;
                match rx {
                    Some(rx) => {
                        if rx.is_closed() {
                            close = true;
                        } else {
                            match timeout(Duration::from_secs(10), rx.recv()).await {
                                Ok(_) => {
                                }
                                Err(_) => {
                                    println!("[Watchdog] timeout");
                                    let mut pipewire_capturer = pipewire_capture.lock().await;
                                    *pipewire_capturer = None;
                                    let mut timeout_rx = timeout_rx.lock().await;
                                    *timeout_rx = None;
                                }
                            }
                        }
                    }
                    None => {
                        println!("[Watchdog] No capturer found");
                        tokio::time::sleep(Duration::from_secs(1)).await;
                    }
                }
                drop(timeout_rx_opt);

                if close {
                    println!("[Watchdog] Capturer closing");
                    let mut pipewire_capturer = pipewire_capture.lock().await;
                    *pipewire_capturer = None;
                    let mut timeout_rx = timeout_rx.lock().await;
                    *timeout_rx = None;
                    println!("Capturer closed");
                }
            }
        });
    }

    pub async fn take_screenshot(&mut self) -> Result<image::ImageBuffer<image::Rgb<u8>, Vec<u8>>, Box<dyn std::error::Error>> {
        match self.take_screenshot_pipewire().await {
            Ok(screenshot) => {
                println!("[Screenshot Tool] Screenshot taken using pipewire");
                return Ok(screenshot);
            }
            Err(err) => {
                println!("Failed to take screenshot using pipewire: {:?}", err);
                println!("[Screenshot Tool] Failed to take screenshot using pipewire, falling back to screenshot portal");
                match screenshot_portal().await {
                    Ok(screenshot) => {
                        let screenshot = image::open(screenshot)?.to_rgb8();
                        return Ok(screenshot);
                    }
                    Err(_) => {
                        println!("[Screenshot Tool] Failed to take screenshot using screenshot portal");
                        let screenshot = screenshot_xcap().await?;
                        let screenshot = image::open(screenshot)?.to_rgb8();
                        return Ok(screenshot);
                    }
                }
            }
        }
    }

    async fn start_capturer_if_needed(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let heartbeat_rx = self.heartbeat_rx.lock().await;
        let is_none = heartbeat_rx.is_none();
        drop(heartbeat_rx);

        println!("Is none: {:?}", is_none);
        
        if is_none {
            let (capturer, rx) = self.start_screenshare().await?;
            let mut pipewire_capturer = self.pipewire_capturer.lock().await;
            *pipewire_capturer = Some(capturer);
            let mut heartbeat_rx = self.heartbeat_rx.lock().await;
            *heartbeat_rx = Some(rx);
        }

        return Ok(());
    }

    async fn take_screenshot_pipewire(&mut self) -> Result<image::ImageBuffer<image::Rgb<u8>, Vec<u8>>, Box<dyn std::error::Error>> {
        println!("Taking screenshot using pipewire");
        self.start_capturer_if_needed().await?;
        println!("Capturer started");

        match self.pipewire_capturer.lock().await.as_mut() {
            Some(capturer) => {
                let screenshot = capturer.take_screenshot().await?;
                return Ok(screenshot);
            }
            None => {
                return Err(Box::new(Error));
            }
        }
    }

    pub async fn start_screenshare(&mut self) -> Result<(PipewireCapturer, tokio::sync::mpsc::Receiver<()>), Box<dyn std::error::Error>> {
        if !scap::is_supported() {
            println!("❌ Platform not supported");
            return Err(Box::new(Error));
        }
        if !scap::has_permission() {
            println!("❌ Permission not granted. Requesting permission...");
            if !scap::request_permission() {
                println!("❌ Permission denied");
                return Err(Box::new(Error));
            }
        }
        let options = Options {
            fps: 60,
            excluded_targets: None,
            excluded_windows: None,
            show_cursor: true,
            show_highlight: false,
            output_type: scap::frame::FrameType::RGB,
            output_resolution: scap::capturer::Resolution::_720p,
            source_rect: Some(Area {
                origin: Point { x: 0.0, y: 0.0 },
                size: Size {
                    width: 2000.0,
                    height: 1000.0,
                },
            }),
            ..Default::default()
        };

        let needs_capture = Arc::new(Mutex::new(false));
        let (tx, rx) = tokio::sync::mpsc::channel::<image::ImageBuffer<image::Rgb<u8>, Vec<u8>>>(1);
        let (heartbeat_tx, heartbeat_rx) = tokio::sync::mpsc::channel::<()>(1);

        // self.needs_screenshot = Some(needs_capture.clone());
        let pipewire_capturer = PipewireCapturer {
            needs_screenshot: needs_capture.clone(),
            image_rx: rx,
        };

        tokio::spawn(async move {
            let mut capturer: Capturer = Capturer::new(options);
            println!("[Screencapture Thread] Starting capture");
            capturer.start_capture();
            println!("Capture started");
            let mut last_screenshot_taken = std::time::Instant::now();
            let timeout_duration = Duration::from_secs(120);
            loop {
                if last_screenshot_taken.elapsed() > timeout_duration {
                    println!("Stopping capturer");
                    // close tx
                    capturer.stop_capture();
                    return;
                }

                if let Err(_) = timeout(Duration::from_secs(2), heartbeat_tx.send(())).await.unwrap() {
                    println!("Failed to send heartbeat");
                    capturer.stop_capture();
                    return;
                }
                let start = std::time::Instant::now();
                let frame = capturer.get_next_frame();
                let frame = match frame {
                    Ok(frame) => frame,
                    Err(_) => {
                        println!("Failed to get frame");
                        return
                    }
                };

                if start.elapsed().as_millis() > 1000 {
                    println!("Frame took too long to receive");
                }

                let should_read = needs_capture.lock().await.clone();
                if  !should_read {
                    continue;
                }
                last_screenshot_taken = std::time::Instant::now();
                println!("[Screencapture Thread] Reading frame");
                let mut should_read = needs_capture.as_ref().lock().await;
                println!("Should read: {:?}", *should_read);
                *should_read = false;
                println!("Should read: {:?}", *should_read);
                println!("[Screencapture Thread] Frame read");

                match frame {
                    Frame::BGRA(frame) => {
                        println!("BGRA Frame");
                    }
                    Frame::BGR0(frame) => {
                        println!("BGR0 Frame");
                    }
                    Frame::RGB(frame) => {
                        println!("RGB Frame");
                    }
                    Frame::RGBx(frame) => {
                        println!("RGBx Frame");
                    }
                    Frame::XBGR(frame) => {
                        println!("XBGR Frame");
                    }
                    Frame::BGRx(frame) => {
                        // empty frame black with same size
                        let mut image = image::ImageBuffer::from_fn(frame.width as u32, frame.height as u32, |x, y| {
                            image::Rgb([0, 0, 0])
                        });
                        let start = std::time::Instant::now();
                        frame.data.chunks_exact(4).enumerate().for_each(|(i, pixel)| {
                            let x = i % frame.width as usize;
                            let y = i / frame.width as usize;
                            let pixel = image::Rgb([pixel[2], pixel[1], pixel[0]]);
                            image.put_pixel(x as u32, y as u32, pixel);
                        });
                        println!("Image creation: {:?}", start.elapsed());
                        println!("Sending image");
                        tx.send(image.clone()).await.unwrap();
                    } 
                    Frame::YUVFrame(frame) => {
                        println!("YUV Frame");
                    }
                    _ => {
                        println!("Frame type not supported");
                    }
                }
            }
        });
        
        return Ok((pipewire_capturer, heartbeat_rx));
    }
}

pub fn get_screenshot_tool() -> ScreenshotTool {
    let mut screenshot_tool = ScreenshotTool {
        pipewire_capturer: Arc::new(Mutex::new(None)),
        heartbeat_rx: Arc::new(Mutex::new(None)),
    };
    screenshot_tool.start();
    return screenshot_tool;
}

pub async fn screenshot_portal() -> Result<String, Box<dyn std::error::Error>> {
    match screenshot::ScreenshotRequest::default()
        .interactive(false)
        .modal(true)
        .send()
        .await
        .and_then(|r| r.response())
    {
        Ok(response) => {
            let screenshot = response.uri().clone();
            println!("[Scoeeenshot] Screenshot taken: {}", screenshot);
            return Ok(String::from(screenshot.to_string().replace("file://", "")));
        }
        Err(err) => {
            println!("[Screenshot] Failed to take screenshot: {}", err);
            return Err(Box::new(Error));
        }
    }
}

pub async fn screenshot_xcap() -> Result<String, Box<dyn std::error::Error>> {
    let monitors = Monitor::all().unwrap();
    let monitor = monitors.get(0);
    match monitor {
        Some(monitor) => {
            let res = monitor.capture_image()?;
            res.save("/tmp/tmp.png")?;
            return Ok(String::from("/tmp/tmp.png"));
        }
        None => {
            return Err(Box::new(Error));
        }
    }
}