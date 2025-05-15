use std::process::exit;

use telnet_rs::telnet;

#[tokio::main]
async fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() != 3 {
        eprintln!("Usage: {} <IP> <PORT>", args[0]);
        exit(1);
    }

    let ip = args.get(1).expect("error: missing ip address");
    let port = args.get(2).expect("error: missing port");

    telnet::connect(ip, port).await
}
