use std::{future::IntoFuture, io::{Read, Write}, sync::Arc};
use once_cell::sync::Lazy;
use tokio::sync::Mutex;
use ashpd::{desktop::{remote_desktop::{self, DeviceType, RemoteDesktop}, Session}, zbus::blocking::proxy, WindowIdentifier};

static REMOTEDESKTOP: Lazy<Mutex<Option<RemoteDesktopSession>>> = Lazy::new(|| Mutex::new(None));

struct RemoteDesktopSession<'a> {
    proxy: RemoteDesktop<'a>,
    session: Session<'a, RemoteDesktop<'a>>,
}

#[derive(Debug,PartialEq)]
pub enum ClickType {
    Left,
    Right,
    Middle,
    Double,
}

pub async fn start_autoclick_session() -> Result<(), Box<dyn std::error::Error>> {
    // autotype portal
    let proxy = RemoteDesktop::new().await?;
    let session = proxy.create_session().await?;
    let token = read_token();
    match token {
        Some(token) => {
            proxy.select_devices(&session, DeviceType::Pointer.into(), Some(token.as_str()), ashpd::desktop::PersistMode::ExplicitlyRevoked).await?;
        }
        None => {
            proxy.select_devices(&session, DeviceType::Pointer.into(), None, ashpd::desktop::PersistMode::ExplicitlyRevoked).await?;
        }
    }
    let response = proxy
        .start(&session, &WindowIdentifier::default())
        .await?
        .response()?;
    match response.restore_token() {
        Some(token) => {
            write_token(&token)?;
        }
        None => {
            println!("No token found");
        }
    }
    let session = RemoteDesktopSession {
        proxy,
        session,
    };
    REMOTEDESKTOP.lock().into_future().await.replace(session);
    Ok(())
}

pub async fn movemouse(x: i32, y: i32, screen_width: i32, screen_height: i32) -> Result<(), Box<dyn std::error::Error>> {
    let session = REMOTEDESKTOP.lock().into_future().await;
    let session = session.as_ref().unwrap();
    let proxy = &session.proxy;
    let session = &session.session;

    proxy.notify_pointer_motion(&session, 10000.0, 10000.0).await?;
    proxy.notify_pointer_motion(&session, (x - screen_width) as f64, (y - screen_height) as f64).await?;
    Ok(())
}

pub async fn click(click_type: ClickType) {
    let key = match click_type {
        ClickType::Left => {
            println!("Left click");
            272
        }
        ClickType::Right => {
            273
        }
        ClickType::Middle => {
            274
        }
        ClickType::Double => {
            272
        }
    };

    let session = REMOTEDESKTOP.lock().into_future().await;
    let session = session.as_ref().unwrap();
    let proxy = &session.proxy;
    let session = &session.session;

    proxy.notify_pointer_button(&session, key, remote_desktop::KeyState::Pressed).await.unwrap();
    tokio::time::sleep(std::time::Duration::from_millis(100)).await;
    proxy.notify_pointer_button(&session, key, remote_desktop::KeyState::Released).await.unwrap();

    if click_type == ClickType::Double {
        tokio::time::sleep(std::time::Duration::from_millis(100)).await;
        proxy.notify_pointer_button(&session, key, remote_desktop::KeyState::Pressed).await.unwrap();
        tokio::time::sleep(std::time::Duration::from_millis(100)).await;
        proxy.notify_pointer_button(&session, key, remote_desktop::KeyState::Released).await.unwrap();
    }
}

fn write_token(token: &str) -> Result<(), Box<dyn std::error::Error>> {
    let mut file = std::fs::File::create("token")?;
    file.write_all(token.as_bytes())?;
    Ok(())
}

fn read_token() -> Option<String> {
    let mut file = std::fs::File::open("token");
    match file {
        Err(_) => {
            return None;
        }
        Ok(mut file) => {
            let mut token = String::new();
            file.read_to_string(&mut token).expect("something went wrong reading the file");
            Some(token)
        }
    }
}