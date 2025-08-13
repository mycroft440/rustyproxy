use std::env;
use std::io::Error;
use std::sync::Arc;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::Mutex;
use tokio::time::{timeout, Duration};

mod protocol_detector;
use protocol_detector::{detect_protocol, Protocol};

#[tokio::main]
async fn main() -> Result<(), Error> {
    let port = get_port();
    let listener = TcpListener::bind(format!("[::]:{}", port)).await?;
    println!("Iniciando serviço na porta: {}", port);
    
    loop {
        match listener.accept().await {
            Ok((client_stream, addr)) => {
                tokio::spawn(async move {
                    if let Err(e) = handle_client(client_stream).await {
                        eprintln!("Erro ao processar cliente {}: {}", addr, e);
                    }
                });
            }
            Err(e) => {
                eprintln!("Erro ao aceitar conexão: {}", e);
            }
        }
    }
}

async fn handle_client(mut client_stream: TcpStream) -> Result<(), Error> {
    let mut buffer = vec![0; 8192];
    let bytes_peeked = client_stream.peek(&mut buffer).await?;
    let initial_data = &buffer[..bytes_peeked];

    let detected_protocol = detect_protocol(initial_data);

    match detected_protocol {
        Protocol::HTTP => {
            println!("Protocolo detectado: HTTP");
            return handle_http_proxy(client_stream, initial_data).await;
        },
        Protocol::SSH => {
            println!("Protocolo detectado: SSH");
            return handle_tunnel_proxy(client_stream, initial_data, "127.0.0.1:22").await;
        },
        Protocol::OpenVPN => {
            println!("Protocolo detectado: OpenVPN");
            return handle_tunnel_proxy(client_stream, initial_data, "127.0.0.1:1194").await;
        },
        Protocol::V2Ray => {
            println!("Protocolo detectado: V2Ray");
            return handle_tunnel_proxy(client_stream, initial_data, "127.0.0.1:8080").await;
        },
        _ => {
            println!("Protocolo detectado: Desconhecido (Fallback para SSH)");
            return handle_tunnel_proxy(client_stream, initial_data, "127.0.0.1:22").await;
        }
    }
}

async fn handle_http_proxy(mut client_stream: TcpStream, initial_data: &[u8]) -> Result<(), Error> {
    let request_str = String::from_utf8_lossy(initial_data);
    let mut lines = request_str.lines();
    let request_line = lines.next().unwrap_or("");

    let parts: Vec<&str> = request_line.split_whitespace().collect();
    if parts.len() < 2 {
        eprintln!("Requisição HTTP inválida: {}", request_line);
        return Ok(());
    }

    let method = parts[0];
    let uri = parts[1];

    let target_addr = if method == "CONNECT" {
        // For CONNECT method (HTTPS tunneling)
        println!("HTTP CONNECT para: {}", uri);
        client_stream.write_all(b"HTTP/1.1 200 Connection Established\r\n\r\n").await?;
        uri.to_string()
    } else {
        // For regular HTTP methods (GET, POST, etc.)
        let mut host = "127.0.0.1:80".to_string(); // Default fallback
        if let Some(host_line) = lines.find(|line| line.starts_with("Host:")) {
            let host_val = host_line.trim_start_matches("Host:").trim();
            if host_val.contains(":") {
                host = host_val.to_string();
            } else {
                host = format!("{}:80", host_val);
            }
        }
        println!("HTTP {} para: {}", method, host);

        // Send HTTP 101 Switching Protocols response like the original
        client_stream
            .write_all(format!("HTTP/1.1 101 {}\r\n\r\n", get_status()).as_bytes())
            .await?;

        // Read more data from client (this was in the original, might be needed for some HTTP injectors)
        let mut temp_buffer = vec![0; 1024];
        client_stream.read(&mut temp_buffer).await?;
        
        // Send HTTP 200 OK response like the original
        client_stream
            .write_all(format!("HTTP/1.1 200 {}\r\n\r\n", get_status()).as_bytes())
            .await?;

        host
    };

    let server_stream = match TcpStream::connect(&target_addr).await {
        Ok(s) => s,
        Err(e) => {
            eprintln!("Erro ao conectar ao servidor de destino {}: {}", target_addr, e);
            return Ok(());
        }
    };

    let (mut client_read, mut server_write) = client_stream.into_split();
    let (mut server_read, mut client_write) = server_stream.into_split();

    // Write the initial data to the server only for non-CONNECT requests
    if method != "CONNECT" {
        server_write.write_all(initial_data).await?;
    }

    let client_to_server = tokio::io::copy(&mut client_read, &mut server_write);
    let server_to_client = tokio::io::copy(&mut server_read, &mut client_write);

    tokio::try_join!(client_to_server, server_to_client)?;

    Ok(())
}

async fn handle_tunnel_proxy(mut client_stream: TcpStream, initial_data: &[u8], target_addr: &str) -> Result<(), Error> {
    let server_stream = match TcpStream::connect(target_addr).await {
        Ok(s) => s,
        Err(e) => {
            eprintln!("Erro ao conectar ao servidor de destino {}: {}", target_addr, e);
            return Ok(());
        }
    };

    let (mut client_read, mut server_write) = client_stream.into_split();
    let (mut server_read, mut client_write) = server_stream.into_split();

    server_write.write_all(initial_data).await?;

    let client_to_server = tokio::io::copy(&mut client_read, &mut server_write);
    let server_to_client = tokio::io::copy(&mut server_read, &mut client_write);

    tokio::try_join!(client_to_server, server_to_client)?;

    Ok(())
}

async fn peek_stream(stream: &TcpStream) -> Result<String, Error> {
    let mut peek_buffer = vec![0; 8192];
    let bytes_peeked = stream.peek(&mut peek_buffer).await?;
    let data = &peek_buffer[..bytes_peeked];
    let data_str = String::from_utf8_lossy(data);
    Ok(data_str.to_string())
}

fn get_port() -> u16 {
    let args: Vec<String> = env::args().collect();
    let mut port = 80;

    for i in 1..args.len() {
        if args[i] == "--port" {
            if i + 1 < args.len() {
                port = args[i + 1].parse().unwrap_or(80);
            }
        }
    }

    port
}

fn get_status() -> String {
    let args: Vec<String> = env::args().collect();
    let mut status = String::from("@RustyManager");

    for i in 1..args.len() {
        if args[i] == "--status" {
            if i + 1 < args.len() {
                status = args[i + 1].clone();
            }
        }
    }

    status
}

