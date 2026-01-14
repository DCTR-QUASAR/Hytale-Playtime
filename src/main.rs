use chrono::NaiveDateTime;
use std::fs;
use std::io::{self, BufRead};

fn main() {
    // 1. Find the platform-specific Hytale log folder
    let log_path = if let Some(mut path) = dirs::data_dir() {
        // Windows: %APPDATA%/Hytale/UserData/logs
        // Linux: ~/.local/share/Hytale/UserData/logs
        // macOS: ~/Library/Application Support/Hytale/UserData/logs
        path.push("Hytale/UserData/logs");
        path
    } else {
        eprintln!("Could not find Hytale data directory.");
        return;
    };

    let mut total_seconds: i64 = 0;
    let mut files_processed = 0;

    // Scan the directory for log files
    if let Ok(entries) = fs::read_dir(&log_path) {
        for entry in entries.flatten() {
            let path = entry.path();

            // Only process files ending in .log
            if path.extension().and_then(|s| s.to_str()) == Some("log") {
                // Open the file or skip to the next one if it fails
                let Ok(file) = fs::File::open(&path) else {
                    continue;
                };

                let reader = io::BufReader::new(file);
                let lines: Vec<String> = reader.lines().map_while(Result::ok).collect();

                // Need at least a start and end line to calculate duration
                if lines.len() < 2 {
                    continue;
                }

                // Hytale format: 2026-01-13 14:03:42.9604|DEBUG|...
                let fmt = "%Y-%m-%d %H:%M:%S";

                let start_line = &lines[0];
                let end_line = &lines[lines.len() - 1];

                // Extract the first 19 characters (ignores milliseconds for simple math)
                if let (Some(s_str), Some(e_str)) = (start_line.get(..19), end_line.get(..19)) {
                    let start_dt = NaiveDateTime::parse_from_str(s_str, fmt);
                    let end_dt = NaiveDateTime::parse_from_str(e_str, fmt);

                    if let (Ok(s), Ok(e)) = (start_dt, end_dt) {
                        // Calculate difference between first and last timestamp in the file
                        let duration = e.signed_duration_since(s).num_seconds();

                        // Add to total if the duration is valid
                        if duration > 0 {
                            total_seconds += duration;
                            files_processed += 1;
                        }
                    }
                }
            }
        }
    }

    // Display the results
    println!("-------------------------------------------");
    println!("Found {} log files in {:?}", files_processed, log_path);
    println!(
        "TOTAL PLAYTIME: {} hours, {} minutes, {} seconds",
        total_seconds / 3600,
        (total_seconds % 3600) / 60,
        total_seconds % 60
    );
    println!("-------------------------------------------");

    // This keeps the window from closing automatically so you can read your time
    println!("\nPress Enter to exit...");
    let mut exit_buffer = String::new();
    let _ = std::io::stdin().read_line(&mut exit_buffer);
}
