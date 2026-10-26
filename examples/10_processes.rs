//! Example 10: Process Management
//! Demonstrates enumerating running processes, finding a process by executable name,
//! and process termination primitives.

use computeruse::ComputerUse;

fn main() -> computeruse::Result<()> {
    println!("=== 10: PROCESS MANAGEMENT ===");

    // 1. List all active processes
    let processes = ComputerUse::list_processes()?;
    println!("Total running processes: {}", processes.len());

    println!("\nSample of active processes (first 10):");
    for (i, p) in processes.iter().take(10).enumerate() {
        println!("  #{:-2} PID: {:6} | Exe: {}", i + 1, p.pid, p.exe_name);
    }

    // 2. Find a specific process
    println!("\nSearching for Windows Explorer ('explorer')...");
    match ComputerUse::find_process("explorer") {
        Ok(proc_info) => {
            println!("  Found Explorer: PID={} | Executable={}", proc_info.pid, proc_info.exe_name);
        }
        Err(e) => println!("  Explorer not found: {}", e),
    }

    // 3. Process kill capabilities (info)
    println!("\nProcess termination functions:");
    println!("  - ComputerUse::kill_process(pid)");
    println!("  - ComputerUse::kill_by_name(\"target.exe\")");

    Ok(())
}
