use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tokio::fs::File;
use std::env;
use std::path::Path;

async fn handle_client(mut stream: TcpStream) {
    let mut buffer = [0; 1024]; // Buffer to hold incoming data

    // Read the command from the stream
    let bytes_read = stream.read(&mut buffer).await.expect("Failed to read from socket");
    let command = String::from_utf8_lossy(&buffer[..bytes_read]).trim().to_string();

    if command.starts_with("UPLOAD") {
        let filename = command[7..].trim(); // Extract filename from command
        let mut file = File::create(&filename).await.expect("Failed to create file");

        // Read the file data from the stream
        while let Ok(bytes_read) = stream.read(&mut buffer).await {
            if bytes_read == 0 {
                break; // Connection closed
            }
            file.write_all(&buffer[..bytes_read]).await.expect("Failed to write to file");
        }

        println!("File '{}' uploaded successfully!", filename);
    } else if command.starts_with("DOWNLOAD") {
        let filename = command[9..].trim(); // Extract filename from command
        if Path::new(filename).exists() {
            let mut file = File::open(filename).await.expect("Failed to open file");
            let mut buffer = vec![0; 1024];

            // Send the file data to the client
            while let Ok(bytes_read) = file.read(&mut buffer).await {
                if bytes_read == 0 {
                    break; // End of file
                }
                stream.write_all(&buffer[..bytes_read]).await.expect("Failed to write to stream");
            }

            println!("File '{}' downloaded successfully!", filename);
        } else {
            eprintln!("File '{}' not found!", filename);
        }
    } else {
        eprintln!("Invalid command: {}", command);
    }
}

#[tokio::main]
async fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: {} <server|client>", args[0]);
        return;
    }

    match args[1].as_str() {
        "server" => {
            let listener = TcpListener::bind("127.0.0.1:8080").await.unwrap();
            println!("Server listening on 127.0.0.1:8080");
            loop {
                let (stream, _) = listener.accept().await.unwrap();
                tokio::spawn(handle_client(stream));
            }
        }
        "client" => {
            // Simple command-line interface for the client
            println!("Welcome to the P2P File Sharing Client!");
            println!("Please choose an option:");
            println!("1. Upload files");
            println!("2. Download a file");
            println!("3. Exit");

            let mut choice = String::new();
            std::io::stdin().read_line(&mut choice).expect("Failed to read line");

            match choice.trim() {
                "1" => {
                    println!("Enter filenames to upload (separated by spaces):");
                    let mut filenames = String::new();
                    std::io::stdin().read_line(&mut filenames).expect("Failed to read line");

                    let mut stream = TcpStream::connect("127.0.0.1:8080").await.unwrap();
                    for filename in filenames.split_whitespace() {
                        if !Path::new(filename).exists() {
                            eprintln!("File '{}' does not exist. Skipping upload.", filename);
                            continue; // Skip to the next file
                        }
                        // Send the upload command with the filename
                        stream.write_all(format!("UPLOAD {}\n", filename).as_bytes()).await.unwrap();

                        // Open the file and send its contents
                        let mut file = File::open(filename).await.expect("Failed to open file");
                        let mut buffer = vec![0; 1024];
                        loop {
                            let bytes_read = file.read(&mut buffer).await.expect("Failed to read file");
                            if bytes_read == 0 {
                                break; // End of file
                            }
                            stream.write_all(&buffer[..bytes_read]).await.unwrap();
                        }
                        println!("File '{}' uploaded successfully!", filename);
                    }
                }
                "2" => {
                    println!("Enter the filename to download:");
                    let mut filename = String::new();
                    std::io::stdin().read_line(&mut filename).expect("Failed to read line");

                    let mut stream = TcpStream::connect("127.0.0.1:8080").await.unwrap();
                    stream.write_all(format!("DOWNLOAD {}\n", filename.trim()).as_bytes()).await.unwrap();

                    // Receive the file data from the server
                    let mut buffer = vec![0; 1024];
                    let mut file = File::create(filename.trim()).await.expect("Failed to create file");
                    while let Ok(bytes_read) = stream.read(&mut buffer).await {
                        if bytes_read == 0 {
                            break; // Connection closed
                        }
                        file.write_all(&buffer[..bytes_read]).await.expect("Failed to write to file");
                    }

                    println!("File '{}' downloaded successfully!", filename.trim());
                }
                "3" => {
                    println!("Exiting...");
                    return;
                }
                _ => {
                    println!("Invalid choice. Please try again.");
                }
            }
        }
        _ => {
            eprintln!("Invalid argument. Use 'server' or 'client'.");
        }
    }
}
