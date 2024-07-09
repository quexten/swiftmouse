use std::fmt::Error;

use ashpd::desktop::screenshot;

pub async fn screenshot() -> Result<String, Box<dyn std::error::Error>> {
    match screenshot::ScreenshotRequest::default()
        .interactive(false)
        .modal(true)
        .send()
        .await
        .and_then(|r| r.response())
    {
        Ok(response) => {
            let screenshot = response.uri().clone();
            println!("[Screenshot] Screenshot taken: {}", screenshot);
            return Ok(String::from(screenshot.to_string().replace("file://", "")));
        }
        Err(err) => {
            println!("[Screenshot] Failed to take screenshot: {}", err);
            return Err(Box::new(Error));
        }
    }
}