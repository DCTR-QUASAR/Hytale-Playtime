use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::io::{self, BufRead};

#[derive(Serialize, Deserialize, Default)]
struct PlaytimeCache {
    files: HashMap<String, i64>,
    total_permanent_seconds: i64,
}

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

    let cache_path = "playtime_cache.json";
    let mut cache: PlaytimeCache = fs::read_to_string(cache_path)
        .ok()
        .and_then(|content| serde_json::from_str(&content).ok())
        .unwrap_or_default();

    let mut total_sessions = 0; // count real sessions

    // Scan the directory for log files
    if let Ok(entries) = fs::read_dir(&log_path) {
        for entry in entries.flatten() {
            let path = entry.path();

            // Only process files ending in .log
            if path.extension().and_then(|s| s.to_str()) == Some("log") {
                let file_name = path.file_name().unwrap().to_string_lossy().into_owned();

                // Open the file or skip to the next one if it fails
                if let Ok(file) = fs::File::open(&path) {
                    let reader = io::BufReader::new(file);
                    let mut last_time: Option<NaiveDateTime> = None;
                    let mut file_seconds = 0;

                    // Hytale format: 2026-01-13 14:03:42.9604|DEBUG|...
                    let fmt = "%Y-%m-%d %H:%M:%S";

                    for line in reader.lines().map_while(Result::ok) {
                        // Extract the first 19 characters (ignores milliseconds for simple math)
                        if let Some(timestamp_str) = line.get(..19)
                            && let Ok(current_time) =
                                NaiveDateTime::parse_from_str(timestamp_str, fmt)
                        {
                            if let Some(previous) = last_time {
                                let delta =
                                    current_time.signed_duration_since(previous).num_seconds();

                                // Add to total if the duration is valid and within a 5-minute activity window
                                if (1..300).contains(&delta) {
                                    file_seconds += delta;
                                }
                            }
                            last_time = Some(current_time);
                        }
                    }

                    let saved_seconds = cache.files.get(&file_name).cloned().unwrap_or(0);
                    if file_seconds > saved_seconds {
                        let difference = file_seconds - saved_seconds;
                        cache.total_permanent_seconds += difference;
                        cache.files.insert(file_name, file_seconds);
                    }

                    if file_seconds > 0 {
                        total_sessions += 1; // count this as one session
                    }
                }
            }
        }
    }

    if let Ok(json) = serde_json::to_string_pretty(&cache) {
        let _ = fs::write(cache_path, json);
    }

    // calculate average session in seconds
    let avg_session = if total_sessions > 0 {
        cache.total_permanent_seconds / total_sessions
    } else {
        0
    };

    // Display the results
    println!("-------------------------------------------");
    println!(
        "TOTAL PLAYTIME: {} hours, {} minutes, {} seconds",
        cache.total_permanent_seconds / 3600,
        (cache.total_permanent_seconds % 3600) / 60,
        cache.total_permanent_seconds % 60
    );
    println!("TOTAL SESSIONS: {}", total_sessions); // how many times you played
    println!(
        "AVERAGE SESSION: {} hours, {} minutes, {} seconds",
        avg_session / 3600,
        (avg_session % 3600) / 60,
        avg_session % 60
    );
    println!("-------------------------------------------");

    // This keeps the window from closing automatically so you can read your time
    println!("\nPress Enter to exit...");
    let mut exit_buffer = String::new();
    let _ = std::io::stdin().read_line(&mut exit_buffer);
}
