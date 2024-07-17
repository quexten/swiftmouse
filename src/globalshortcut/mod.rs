use tokio::sync::mpsc;
use zbus::{blocking::connection, interface};

struct ZbusListener {
    // tx channel
    tx: mpsc::Sender<()>,
}

#[interface(name = "com.quexten.swiftmouse")]
impl ZbusListener {
    async fn run(&mut self) -> String {
        println!("sending");
        self.tx.send(()).await.unwrap();
        "".to_string()
    }
}


// fn listen and have a return channel to send events
pub async fn listen() -> (mpsc::Receiver<()> , connection::Connection) {
    let (tx, rx) = mpsc::channel(1);
    let listener = ZbusListener {
        tx,
    };
    let _conn = connection::Builder::session().unwrap()
        .name("com.quexten.swiftmouse").unwrap()
        .serve_at("/com/quexten/swiftmouse", listener).unwrap()
        .build()
        .unwrap();
    println!("[Global Shortcuts] Listening on D-Bus");
    return (rx, _conn);
}