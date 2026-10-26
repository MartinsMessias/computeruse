use crate::error::{CuError, Result};
use serde::{Deserialize, Serialize};
use windows::Win32::Foundation::CloseHandle;
use windows::Win32::System::Diagnostics::ToolHelp::{
    CreateToolhelp32Snapshot, Process32FirstW, Process32NextW, PROCESSENTRY32W, TH32CS_SNAPPROCESS,
};
use windows::Win32::System::Threading::{OpenProcess, TerminateProcess, PROCESS_TERMINATE};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessInfo {
    pub pid: u32,
    pub parent_pid: u32,
    pub exe_name: String,
    pub threads: u32,
}

pub fn list_processes() -> Result<Vec<ProcessInfo>> {
    unsafe {
        let snap = CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0)?;
        let mut out = Vec::new();
        let mut entry = PROCESSENTRY32W {
            dwSize: std::mem::size_of::<PROCESSENTRY32W>() as u32,
            ..Default::default()
        };
        if Process32FirstW(snap, &mut entry).is_ok() {
            loop {
                let name = String::from_utf16_lossy(&entry.szExeFile)
                    .trim_end_matches('\0')
                    .to_string();
                out.push(ProcessInfo {
                    pid: entry.th32ProcessID,
                    parent_pid: entry.th32ParentProcessID,
                    exe_name: name,
                    threads: entry.cntThreads,
                });
                if Process32NextW(snap, &mut entry).is_err() {
                    break;
                }
            }
        }
        let _ = CloseHandle(snap);
        Ok(out)
    }
}

pub fn find_process(name_contains: &str) -> Result<ProcessInfo> {
    let lower = name_contains.to_lowercase();
    let procs = list_processes()?;
    procs
        .into_iter()
        .find(|p| p.exe_name.to_lowercase().contains(&lower))
        .ok_or_else(|| CuError::NotFound(format!("process '{}'", name_contains)))
}

pub fn kill_process(pid: u32) -> Result<()> {
    unsafe {
        let handle = OpenProcess(PROCESS_TERMINATE, false, pid)?;
        TerminateProcess(handle, 1)?;
        let _ = CloseHandle(handle);
    }
    Ok(())
}

pub fn kill_by_name(name_contains: &str) -> Result<u32> {
    let p = find_process(name_contains)?;
    kill_process(p.pid)?;
    Ok(p.pid)
}
