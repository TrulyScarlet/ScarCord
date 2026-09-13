#![cfg(windows)]

use std::collections::HashSet;
use std::env;
use std::thread;
use std::time::Duration;

use windows::core::{Interface, PCWSTR};
use windows::Win32::Foundation::{CloseHandle, HANDLE, INVALID_HANDLE_VALUE};
use windows::Win32::Media::Audio::{
    eMultimedia, eRender, IAudioSessionControl, IAudioSessionControl2,
    IAudioSessionManager2, IMMDeviceEnumerator, MMDeviceEnumerator,
};
use windows::Win32::System::Com::{
    CoCreateInstance, CoInitializeEx, CLSCTX_ALL, COINIT_MULTITHREADED,
};
use windows::Win32::System::Diagnostics::ToolHelp::{
    CreateToolhelp32Snapshot, Process32FirstW, Process32NextW, PROCESSENTRY32W, TH32CS_SNAPPROCESS,
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

fn to_wide(s: &str) -> Vec<u16> {
    s.encode_utf16().chain(std::iter::once(0)).collect()
}

fn rename_session_if_ours(session: &IAudioSessionControl, app_name: &str, icon_spec: &str) {
    unsafe {
        let Ok(control2) = session.cast::<IAudioSessionControl2>() else {
            return;
        };
        let Ok(session_pid) = control2.GetProcessId() else {
            return;
        };

        let my_pid = std::process::id();
        let our_children = get_descendant_pids(my_pid);

        if session_pid == my_pid || our_children.contains(&session_pid) {
            let name_wide = to_wide(app_name);
            let icon_wide = to_wide(icon_spec);

            let _ = session.SetDisplayName(PCWSTR(name_wide.as_ptr()), std::ptr::null());
            let _ = session.SetIconPath(PCWSTR(icon_wide.as_ptr()), std::ptr::null());
        }
    }
}

pub fn start_volume_mixer_fix(app_name: &'static str) {
    thread::spawn(move || unsafe {
        let _ = CoInitializeEx(None, COINIT_MULTITHREADED);

        let current_exe = env::current_exe().unwrap_or_default();
        let icon_spec = format!("{},0", current_exe.to_string_lossy());

        loop {
            if let Ok(device_enumerator) = CoCreateInstance::<_, IMMDeviceEnumerator>(&MMDeviceEnumerator, None, CLSCTX_ALL) {
                if let Ok(default_device) = device_enumerator.GetDefaultAudioEndpoint(eRender, eMultimedia) {
                    if let Ok(session_manager) = default_device.Activate::<IAudioSessionManager2>(CLSCTX_ALL, None) {
                        if let Ok(enumerator) = session_manager.GetSessionEnumerator() {
                            if let Ok(count) = enumerator.GetCount() {
                                for i in 0..count {
                                    if let Ok(session) = enumerator.GetSession(i) {
                                        rename_session_if_ours(&session, app_name, &icon_spec);
                                    }
                                }
                            }
                        }
                    }
                }
            }

            thread::sleep(Duration::from_millis(1500));
        }
    });
}
