//! Example 09: System Metrics, Drives & Special Folders
//! Demonstrates querying RAM statistics, CPU core count, system uptime, battery status,
//! logical disk drive space, and Windows special folder locations.

use computeruse::ComputerUse;

fn main() -> computeruse::Result<()> {
    println!("=== 09: SYSTEM INFO, DRIVES & SPECIAL FOLDERS ===");

    // 1. System health & hardware metrics
    let sys = ComputerUse::system_info()?;
    println!("System Architecture & Hardware:");
    println!("  OS:             {}", sys.os_name);
    println!("  CPU Cores:      {}", sys.cpu_cores);
    println!("  Total RAM:      {} MB ({:.2} GB)", sys.total_memory_mb, sys.total_memory_mb as f64 / 1024.0);
    println!("  Available RAM:  {} MB ({:.2} GB)", sys.available_memory_mb, sys.available_memory_mb as f64 / 1024.0);
    println!("  Memory Load:    {}%", sys.memory_load_percent);
    println!("  System Uptime:  {} seconds ({:.1} hours)", sys.uptime_seconds, sys.uptime_seconds as f64 / 3600.0);
    println!("  Battery:        {:?}", sys.battery_percent);
    println!("  AC / Charging:  {:?}", sys.is_charging);

    // 2. Logical disk drives and free space
    println!("\nLogical Disk Drives:");
    let drives = ComputerUse::system_drives()?;
    for d in &drives {
        let total_gb = d.total_bytes as f64 / (1024.0 * 1024.0 * 1024.0);
        let free_gb = d.free_bytes as f64 / (1024.0 * 1024.0 * 1024.0);
        let pct_free = (d.free_bytes as f64 / d.total_bytes as f64) * 100.0;
        println!(
            "  Drive {:-3} | {:.1} GB free of {:.1} GB ({:.1}% free)",
            d.name, free_gb, total_gb, pct_free
        );
    }

    // 3. Standard Windows special folders
    println!("\nWindows Special Folders:");
    let folder_keys = ["desktop", "downloads", "documents", "temp", "appdata", "localappdata", "windows"];
    for key in folder_keys {
        match ComputerUse::system_special_folder(key) {
            Ok(path) => println!("  {:-13} -> {}", key, path),
            Err(e) => println!("  {:-13} -> Error: {}", key, e),
        }
    }

    Ok(())
}
