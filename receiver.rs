use std::fs::File;
use std::io::{self, Read, Write};
use std::net::TcpListener;

fn main() -> std::io::Result<()> {
    let mut portI = String::new();

    println!("Enter listening port:");
    io::stdin().read_line(&mut portI).expect("Failed to read line");

    let address = format!("0.0.0.0:{}", portI.trim());
    let listener = TcpListener::bind(address)?;
    println!("Receiver listening on port {}...", portI);

    for stream in listener.incoming() {
        let mut stream = stream?;
        println!("Connection from {}", stream.peer_addr()?);

        // === Read filename length (2 bytes) ===
        let mut len_buf = [0u8; 2];
        stream.read_exact(&mut len_buf)?;
        let filename_len = u16::from_be_bytes(len_buf) as usize;

        // === Read filename ===
        let mut filename_buf = vec![0u8; filename_len];
        stream.read_exact(&mut filename_buf)?;
        let filename = String::from_utf8(filename_buf)
            .expect("Invalid UTF-8 in filename");

        println!("Receiving file: {}", filename);

        // === Read file size (8 bytes) ===
        let mut size_buf = [0u8; 8];
        stream.read_exact(&mut size_buf)?;
        let file_size = u64::from_be_bytes(size_buf);
        println!("File size: {} bytes", file_size);
        let mut q = String::new();
        println!("Accept? (Y/N)");
        io::stdin().read_line(&mut q).expect("Failed to read line");
        let q = q.trim();
        if q == "N" || q == "No" || q == "n" || q == "no"{
            println!("Not accepted");
            break
        }
        // === Create local file ===
        let mut file = File::create(&filename)?;

        // === Receive file content ===
        let mut remaining = file_size;
        let mut buffer = [0u8; 4096];
        while remaining > 0 {
            let read_bytes = stream.read(&mut buffer)?;
            if read_bytes == 0 {
                break;
            }
            file.write_all(&buffer[..read_bytes])?;
            remaining -= read_bytes as u64;
        }

        println!("File '{}' received successfully.\n", filename);
    }

    Ok(())
}

//received file from : ; accept?