mod receiver;
use std::fs::{self, File, OpenOptions};
use std::io::{self, Read, Write, Seek, SeekFrom};
use std::net::TcpStream;
use std::path::Path;
use rand::Rng;
use std::time::Instant; // <-- ADDED

fn main() -> std::io::Result<()>{
    let mut input_path_input = String::new();
    let mut target_ip_input = String::new();
    //let mut encryption_level = String::new();

    println!("Enter file path:");
    io::stdin().read_line(&mut input_path_input).expect("Failed to read line");
    let input_path = input_path_input.trim();
    let input_path = clean_path_input(&input_path_input);
    println!("Enter target address:");
    io::stdin().read_line(&mut target_ip_input).expect("Failed to read line");
    let target_ip = target_ip_input.trim();

    //println!("Choose encryption level : unsafe-quickest (estimate 300ms) mid-safety-quick; safe-slow :");
    //io::stdin().read_line(&mut encryption_level).expect("Failed to read line");
    // add over port question

    let mut stream = TcpStream::connect(target_ip)?;
    println!("Handshake successful - TCP connection established.");

    let mut file = File::open(input_path)?;
    let metadata = file.metadata()?;
    let file_size = metadata.len();
    let path = Path::new(input_path);
    let filename = path.file_name().and_then(|name| name.to_str()).unwrap();

    // Send filename length
    let name_bytes = filename.as_bytes();
    let name_len = name_bytes.len() as u16;
    stream.write_all(&name_len.to_be_bytes())?;

    // Send filename
    stream.write_all(name_bytes)?;

    stream.write_all(&file_size.to_be_bytes())?;

    // Start timer
    let start_time = Instant::now(); // <-- ADDED

    let mut buffer = [0u8; 4096];
    loop {
        let read_bytes = file.read(&mut buffer)?;
        if read_bytes == 0 {
            break;
        }
        stream.write_all(&buffer[..read_bytes])?;
    }

    // End timer and compute stats
    let elapsed = start_time.elapsed().as_secs_f64();
    let size_gb = file_size as f64 / 1_073_741_824.0;
    println!("Sent.");
    println!("Transfer complete: {:.2}GB in {:.2}s", size_gb, elapsed);

    println!("Shred? (Y/N):");
    let mut shred_choice = String::new();
    io::stdin().read_line(&mut shred_choice)?;

    let choice = shred_choice.trim().to_lowercase();

    if choice == "yes" || choice == "y" || choice == "Yes" || choice == "Y" {
        match shred_file(input_path) {
            Ok(_) => println!("Success."),
            Err(e) => eprintln!("Failed to shred file: {}", e),
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

    // Overwrite with random data
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

fn clean_path_input(input: &str) -> &str {
    let trimmed = input.trim();
    if (trimmed.starts_with('"') && trimmed.ends_with('"')) ||
        (trimmed.starts_with('\'') && trimmed.ends_with('\'')) {
        &trimmed[1..trimmed.len() - 1]
    } else {
        trimmed
    }
}
