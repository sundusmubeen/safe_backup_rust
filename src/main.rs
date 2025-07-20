use std::fs;
use std::io::{self, Read, Write};
use std::path::{Path, PathBuf};
use std::ffi::OsStr;
use std::time::{SystemTime, UNIX_EPOCH};
use std::process;
use chrono::Local;


const MAX_FILENAME_LENGTH: usize = 255;
const MAX_FILE_SIZE: u64 = 10 * 1024 * 1024; // 10MB
const VALID_CHAR: &str = "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789_-.";

fn isValidFilename(filename: &str) -> bool {
    if filename.is_empty() || filename.len() > MAX_FILENAME_LENGTH {
        return false;
    }
    
    if filename.contains("..") || filename.contains('/') || filename.contains('\\') {
        return false;
    }
    
    filename.chars().all(|c| VALID_CHAR.contains(c))
}

fn backupFile(filename: &str) -> io::Result<()> {
    if !isValidFilename(filename) {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "Invalid filename",
        ));
    }

    let path = Path::new(filename);
    if !path.exists() {
        return Err(io::Error::new(
            io::ErrorKind::NotFound,
            "File not found",
        ));
    }

    let metadata = fs::metadata(path)?;
    if metadata.len() > MAX_FILE_SIZE {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "File too large",
        ));
    }

    let backupFilename = format!("{}.bak", filename);
    let backupFilepath = Path::new(&backupFilename);

    // Check if backup already exists
    if backupFilepath.exists() {
        println!("WARNING: Backup file {} already exists. Overwrite? (yes/no): ", backupFilename);
        let mut confirm = String::new();
        io::stdin().read_line(&mut confirm)?;
        if confirm.trim().to_lowercase() != "yes" {
            println!("Backup cancelled.");
            return Ok(());
        }
    }

    let currPath = format!("{}.tmp", backupFilename);
    {
        let mut inputFile = fs::File::open(path)?;
        let mut outputFile = fs::File::create(&currPath)?;
        
        // Set permissions (read/write for owner only)
        let mut permissions = outputFile.metadata()?.permissions();
        permissions.set_readonly(false);
        fs::set_permissions(&currPath, permissions)?;

        let bytes_copied = io::copy(&mut inputFile, &mut outputFile)?;
        if bytes_copied != metadata.len() {
            fs::remove_file(&currPath)?;
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "Failed to copy entire file",
            ));
        }
    }

    fs::rename(&currPath, backupFilepath)?;
    println!("Backup created: {}", backupFilename);
    logAction(&format!("Performed backup on {}", filename))?;

    Ok(())
}

fn restoreFile(filename: &str) -> io::Result<()> {
    if !isValidFilename(filename) {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "Invalid filename",
        ));
    }

    let backupFileName = format!("{}.bak", filename);
    let backupFilePath = Path::new(&backupFileName);

    if !backupFilePath.exists() {
        return Err(io::Error::new(
            io::ErrorKind::NotFound,
            format!("Backup file '{}' not found", backupFileName),
        ));
    }

    let metadata = match fs::metadata(backupFilePath) {
        Ok(m) => m,
        Err(_) => {
            return Err(io::Error::new(
                io::ErrorKind::NotFound,
                format!("Cannot access backup file '{}'", backupFileName),
            ));
        }
    };

    if metadata.len() > MAX_FILE_SIZE {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "Backup file too large",
        ));
    }

    if Path::new(filename).exists() {
        println!("WARNING: Target file {} already exists. Overwrite? (yes/no): ", filename);
        let mut confirm = String::new();
        io::stdin().read_line(&mut confirm)?;
        if confirm.trim().to_lowercase() != "yes" {
            println!("Restore cancelled");
            return Ok(());
        }
    }

    let currPath = format!("{}.tmp", filename);
    {
        let mut inputFile = fs::File::open(backupFilePath)?;
        let mut outputFile = fs::File::create(&currPath)?;

        let mut permissions = outputFile.metadata()?.permissions();
        permissions.set_readonly(false);
        fs::set_permissions(&currPath, permissions)?;

        let byteCopied = io::copy(&mut inputFile, &mut outputFile)?;
        if byteCopied != metadata.len() {
            fs::remove_file(&currPath)?;
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "Failed to copy entire file",
            ));
        }
    }

    fs::rename(&currPath, filename)?;
    println!("File restored from: {}", backupFileName);
    logAction(&format!("Performed restore on {}", filename))?;

    Ok(())
}

fn deleteFile(filename: &str) -> io::Result<()> {
    if !isValidFilename(filename) {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "Invalid filename",
        ));
    }

    let path = Path::new(filename);
    if !path.exists() {
        return Err(io::Error::new(
            io::ErrorKind::NotFound,
            format!("File '{}' not found", filename),
        ));
    }

    println!("Are you sure you want to delete {}? (type 'DELETE' to confirm): ", filename);
    let mut confirm = String::new();
    io::stdin().read_line(&mut confirm)?;

    if confirm.trim() == "DELETE" {
        fs::remove_file(path)?;

        println!("File deleted");

        if let Err(e) = logAction(&format!("Performed delete on {}", filename)) {
            eprintln!("Warning: Could not log delete action: {}", e);
        }

        Ok(())
    } else {
        println!("Delete cancelled");
        Err(io::Error::new(
            io::ErrorKind::PermissionDenied,
            "Delete permission denied",
        ))
    }
}


fn logAction(action: &str) -> io::Result<()> {
    
    let sanitizeInput = action.replace("\n", " ").replace("\r", " ");

    
    let timestamp = Local::now().format("%Y-%m-%d %H:%M:%S").to_string();

    let mut log = fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open("logfile.txt")?;

    writeln!(log, "[{}] {}", timestamp, sanitizeInput)?;
    Ok(())
}


fn main() {
    println!("Safe Backup - Rust");

    println!("Please enter your file name: ");
    let mut filename_input = String::new();
    if let Err(e) = io::stdin().read_line(&mut filename_input) {
        eprintln!("Error reading filename: {}", e);
        process::exit(1);
    }
    let filename = filename_input.trim();

    if !isValidFilename(filename) {
        eprintln!("\n[REJECTED] Invalid filename: Potential path traversal or illegal characters.");
        println!("\nPress Enter to exit...");
        let _ = io::stdin().read_line(&mut String::new());
        process::exit(1);
    }

    println!("Enter your command (backup, restore, delete): ");
    let mut command = String::new();
    if let Err(e) = io::stdin().read_line(&mut command) {
        eprintln!("Error reading command: {}", e);
        process::exit(1);
    }
    let command = command.trim();

    let result = match command {
        "backup" => backupFile(filename),
        "restore" => restoreFile(filename),
        "delete" => deleteFile(filename),
        _ => {
            eprintln!("Invalid command");
            process::exit(1);
        }
    };

    if let Err(e) = result {
    eprintln!("Error: {}", e);
    process::exit(1);
    }


    println!("\nPress Enter to exit...");
    let _ = io::stdin().read_line(&mut String::new());
}
