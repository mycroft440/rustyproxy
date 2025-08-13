use std::env;
use std::io::Error;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
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
    // Peek the initial data to detect protocol without consuming it
    let mut buffer = vec![0; 8192];
    let bytes_peeked = client_stream.peek(&mut buffer).await?;
    let initial_data = &buffer[..bytes_peeked];

    let detected_protocol = detect_protocol(initial_data);

    let target_addr = match detected_protocol {
        Protocol::SSH => {
            println!("Protocolo detectado: SSH");
            "127.0.0.1:22"
        },
        Protocol::OpenVPN => {
            println!("Protocolo detectado: OpenVPN");
            "127.0.0.1:1194"
        },
        Protocol::V2Ray => {
            println!("Protocolo detectado: V2Ray");
            "127.0.0.1:8080" // Exemplo de porta para V2Ray
        },
        _ => {
            println!("Protocolo detectado: Desconhecido (Fallback para SSH)");
            "127.0.0.1:22" // Fallback para SSH ou outro padrão
        }
    };

    let server_stream = match TcpStream::connect(target_addr).await {
        Ok(s) => s,
        Err(e) => {
            eprintln!("Erro ao conectar ao servidor de destino {}: {}", target_addr, e);
            return Ok(());
        }
    };

    // Now read the initial data from the client stream
    let bytes_read = client_stream.read(&mut buffer).await?;
    let initial_data_actual = &buffer[..bytes_read];

    let (mut client_read, mut server_write) = client_stream.into_split();
    let (mut server_read, mut client_write) = server_stream.into_split();

    server_write.write_all(initial_data_actual).await?;

    let client_to_server = tokio::io::copy(&mut client_read, &mut server_write);
    let server_to_client = tokio::io::copy(&mut server_read, &mut client_write);

    tokio::try_join!(client_to_server, server_to_client)?;

    Ok(())
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
