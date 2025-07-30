use clap::Parser;
use std::net::TcpStream;
use std::path::Path;
use std::fs::{self, File, OpenOptions};
use std::io::{self, Read, Write, Seek, SeekFrom};
use std::net::TcpListener;
use rand::Rng;

#[derive(Parser)]
#[command(author, version, about)]
struct Cli {

    #[arg(short, long)]
    file: String,


    #[arg(short, long)]
    target: String,


    #[arg(short, long, default_value_t = false)]
    shred: bool,

    #[arg(short,long,default_value_t = false)]
    receive: bool
}

fn main() -> std::io::Result<()> {

        let args = Cli::parse();

        let input_path = args.file;
        let target_ip = args.target;
        if args.receive == false {
        let mut stream = TcpStream::connect(&target_ip)?;
        println!("Handshake successful - TCP connection established.");

        let mut file = File::open(&input_path)?;
        let metadata = file.metadata()?;
        let file_size = metadata.len();
        let path = Path::new(&input_path);
        let filename = path.file_name().and_then(|name| name.to_str()).unwrap();

        // Send filename length
        let name_bytes = filename.as_bytes();
        let name_len = name_bytes.len() as u16;
        stream.write_all(&name_len.to_be_bytes())?;
        stream.write_all(name_bytes)?;
        stream.write_all(&file_size.to_be_bytes())?;

        let mut buffer = [0u8; 4096];
        loop {
            let read_bytes = file.read(&mut buffer)?;
            if read_bytes == 0 {
                break;
            }
            stream.write_all(&buffer[..read_bytes])?;
        }

        println!("Sent.");

        if args.shred {
            match shred_file(&input_path) {
                Ok(_) => println!("File shredded successfully."),
                Err(e) => eprintln!("Failed to shred file: {}", e),
            }
        }

    }
    else {
        let address = "0.0.0.0:8080";
        let listener = TcpListener::bind(address)?;
        println!("Receiver listening on port 8080...");

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
            if q == "N" || q == "No" || q == "n" || q == "no" {
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
    }
    Ok(())
}

fn shred_file<P: AsRef<std::path::Path>>(path: P) -> io::Result<()> {
    let metadata = fs::metadata(&path)?;
    let file_size = metadata.len();

    let mut file = OpenOptions::new()
        .write(true)
        .open(&path)?;


    let mut rng = rand::thread_rng();
    let mut buffer = vec![0u8; 4096];
    let mut written: u64 = 0;

    while written < file_size {
        let to_write = std::cmp::min(buffer.len() as u64, file_size - written) as usize;
        rng.fill(&mut buffer[..to_write]);
        file.write_all(&buffer[..to_write])?;
        written += to_write as u64;
    }

    file.flush()?;
    file.seek(SeekFrom::Start(0))?;
    drop(file);

    fs::remove_file(&path)?;
    Ok(())
}