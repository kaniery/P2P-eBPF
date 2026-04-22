use std::env;
use std::error::Error;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<String> = env::args().collect();
    
    if args.len() < 2 {
        eprintln!("Usage: {} <listen_port> [peer_address:port]", args[0]);
        return Ok(());
    }

    let listen_port = &args[1];
    let listen_addr = format!("0.0.0.0:{}", listen_port);

    // 1. サーバータスク（他のノードからの接続を受け付ける）
    let listener = TcpListener::bind(&listen_addr).await?;
    println!("Node listening on {}", listen_addr);

    tokio::spawn(async move {
        loop {
            match listener.accept().await {
                Ok((socket, addr)) => {
                    println!("[+] Incoming connection from: {}", addr);
                    tokio::spawn(handle_connection(socket));
                }
                Err(e) => eprintln!("[-] Failed to accept connection: {}", e),
            }
        }
    });

    // 2. クライアントタスク（指定されたピアに接続する）
    if args.len() == 3 {
        let peer_addr = &args[2];
        println!("Attempting to connect to peer: {}", peer_addr);
        
        match TcpStream::connect(peer_addr).await {
            Ok(socket) => {
                println!("[+] Successfully connected to peer: {}", peer_addr);
                tokio::spawn(handle_connection(socket));
            }
            Err(e) => eprintln!("[-] Failed to connect to peer {}: {}", peer_addr, e),
        }
    }

    // メインスレッドを維持するためのダミーループ
    // 実際の実装では、ここで標準入力からのメッセージ送信や、
    // eBPFのロード/アタッチ処理の待機などを行います。
    loop {
        tokio::time::sleep(tokio::time::Duration::from_secs(60)).await;
    }
}

// 接続されたピアとの通信を処理する関数
async fn handle_connection(mut socket: TcpStream) {
    let mut buf = vec![0; 1024];

    // 接続時に簡単な挨拶メッセージを送信
    if let Err(e) = socket.write_all(b"Hello from Rust P2P node!").await {
        eprintln!("[-] Failed to send message: {}", e);
        return;
    }

    loop {
        match socket.read(&mut buf).await {
            Ok(0) => {
                println!("[-] Connection closed by peer.");
                break;
            }
            Ok(n) => {
                let msg = String::from_utf8_lossy(&buf[..n]);
                println!("[Message received] {}", msg);
                
                // ここにP2Pのルーティングやゴシッププロトコルの処理を追加します
            }
            Err(e) => {
                eprintln!("[-] Socket read error: {}", e);
                break;
            }
        }
    }
}