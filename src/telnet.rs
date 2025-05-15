use std::{process::exit, time::Duration};

use tokio::{
    io::{self, AsyncBufReadExt, AsyncWriteExt, BufReader},
    net::TcpStream,
    sync::mpsc,
};

pub async fn connect(ip: &str, port: &str) {
    let address = format!("{ip}:{port}");
    let timeout = Duration::from_secs(5);

    let stream = match tokio::time::timeout(timeout, TcpStream::connect(&address)).await {
        Ok(Ok(stream)) => {
            println!("Connected to {}", address);
            stream
        }
        Ok(Err(e)) => {
            eprintln!("Connection error to {}: {}", address, e);
            std::process::exit(1);
        }
        Err(_) => {
            eprintln!("Timeout ({:?}) connecting to {}", timeout, address);
            std::process::exit(1);
        }
    };

    let (reader, mut writer) = stream.into_split();
    let mut server_reader = BufReader::new(reader);

    // Channel for notify error or close from server
    let (tx, mut rx) = mpsc::unbounded_channel::<()>();

    // Task for read response from server
    let tx_clone = tx.clone();
    tokio::spawn(async move {
        let mut buf = String::new();
        loop {
            buf.clear();
            match server_reader.read_line(&mut buf).await {
                Ok(0) => {
                    println!("Server closed the connection");
                    let _ = tx_clone.send(());
                    break;
                }
                Ok(_) => print!("Server response: {}", buf),
                Err(e) => {
                    eprintln!("Error reading from server: {}", e);
                    let _ = tx_clone.send(());
                    break;
                }
            }
        }
    });

    // Task for read input from user
    let stdin = io::stdin();
    let mut stdin_reader = BufReader::new(stdin);
    let mut line = String::new();
    println!("Enter commands ('exit' to quit)");
    loop {
        tokio::select! {
            // Server connection closed
            _ = rx.recv() => {
                exit(0);
            }
            // Read commanf from user
            result = stdin_reader.read_line(&mut line) => {
                if let Ok(0) | Err(_) = result {
                    println!("Input stream closed");
                    break;
                }
                let cmd = line.trim_end();
                if cmd.eq_ignore_ascii_case("exit") {
                    println!("Exiting on user command");
                    break;
                }
                if let Err(e) = writer.write_all(cmd.as_bytes()).await {
                    eprintln!("Error sending command: {}", e);
                    break;
                }
                if let Err(e) = writer.write_all(b"\n").await {
                    eprintln!("Error sending newline: {}", e);
                    break;
                }
                line.clear();
            }
        }
    }

    println!("Shutting down client...");
}
