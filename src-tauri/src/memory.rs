#![cfg(windows)]

use std::collections::HashSet;
use windows::Win32::Foundation::{CloseHandle, HANDLE, INVALID_HANDLE_VALUE};
use windows::Win32::System::Diagnostics::ToolHelp::{
    CreateToolhelp32Snapshot, Process32FirstW, Process32NextW, PROCESSENTRY32W, TH32CS_SNAPPROCESS,
};
use windows::Win32::System::ProcessStatus::K32EmptyWorkingSet;
use windows::Win32::System::Threading::{
    GetCurrentProcess, GetCurrentProcessId, OpenProcess, PROCESS_QUERY_INFORMATION, PROCESS_SET_QUOTA,
};

fn get_descendant_pids(root_pid: u32) -> HashSet<u32> {
    let mut descendants = HashSet::new();
    let mut parent_map: Vec<(u32, u32)> = Vec::new();

    unsafe {
        let snapshot: HANDLE = match CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0) {
            Ok(h) if h != INVALID_HANDLE_VALUE => h,
            _ => return descendants,
        };

        let mut entry = PROCESSENTRY32W {
            dwSize: std::mem::size_of::<PROCESSENTRY32W>() as u32,
            ..Default::default()
        };

        if Process32FirstW(snapshot, &mut entry).is_ok() {
            loop {
                parent_map.push((entry.th32ProcessID, entry.th32ParentProcessID));
                if Process32NextW(snapshot, &mut entry).is_err() {
                    break;
                }
            }
        }
        let _ = CloseHandle(snapshot);
    }

    let mut to_check = vec![root_pid];
    while let Some(current_pid) = to_check.pop() {
        for &(pid, parent_pid) in &parent_map {
            if parent_pid == current_pid && !descendants.contains(&pid) {
                descendants.insert(pid);
                to_check.push(pid);
            }
        }
    }

    descendants
}

pub fn trim_memory() {
    unsafe {
        // Trim host process (ScarCord.exe)
        let current_handle = GetCurrentProcess();
        let _ = K32EmptyWorkingSet(current_handle);

        // Trim all WebView2 child processes (msedgewebview2.exe)
        let current_pid = GetCurrentProcessId();
        let child_pids = get_descendant_pids(current_pid);

        for pid in child_pids {
            if let Ok(handle) = OpenProcess(PROCESS_SET_QUOTA | PROCESS_QUERY_INFORMATION, false, pid) {
                let _ = K32EmptyWorkingSet(handle);
                let _ = CloseHandle(handle);
            }
        }
    }
}
