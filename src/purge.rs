use std::mem;

use windows_sys::Win32::Foundation::CloseHandle;
use windows_sys::Win32::System::ProcessStatus::{EmptyWorkingSet, EnumProcesses};
use windows_sys::Win32::System::Threading::OpenProcess;

use crate::error::MpaError;
use crate::ntapi::{self, MemoryListCommand};

/// Result of a working set purge across all processes.
pub struct WorkingSetPurgeResult {
    pub processes_trimmed: u32,
    pub processes_skipped: u32,
}

/// Empty working sets of all running processes.
pub fn purge_working_sets() -> Result<WorkingSetPurgeResult, MpaError> {
    const PROCESS_QUERY_INFORMATION: u32 = 0x0400;
    const PROCESS_SET_QUOTA: u32 = 0x0100;

    let mut pids = vec![0u32; 4096];
    let mut needed: u32 = 0;

    unsafe {
        if EnumProcesses(
            pids.as_mut_ptr(),
            (pids.len() * mem::size_of::<u32>()) as u32,
            &mut needed,
        ) == 0
        {
            return Err(MpaError::winapi("EnumProcesses"));
        }
    }

    let count = needed as usize / mem::size_of::<u32>();
    let mut trimmed = 0u32;
    let mut skipped = 0u32;

    for &pid in &pids[..count] {
        if pid == 0 {
            continue;
        }

        unsafe {
            let h = OpenProcess(PROCESS_QUERY_INFORMATION | PROCESS_SET_QUOTA, 0, pid);
            if h.is_null() {
                skipped += 1;
                continue;
            }

            if EmptyWorkingSet(h) != 0 {
                trimmed += 1;
            } else {
                skipped += 1;
            }

            CloseHandle(h);
        }
    }

    Ok(WorkingSetPurgeResult {
        processes_trimmed: trimmed,
        processes_skipped: skipped,
    })
}

/// Purge the standby list.
pub fn purge_standby(low_only: bool) -> Result<(), MpaError> {
    let cmd = if low_only {
        MemoryListCommand::PurgeLowPriorityStandbyList
    } else {
        MemoryListCommand::PurgeStandbyList
    };
    ntapi::execute_memory_command(cmd)
}

/// Flush the modified page list to disk.
pub fn purge_modified() -> Result<(), MpaError> {
    ntapi::execute_memory_command(MemoryListCommand::FlushModifiedList)
}

/// Purge all: empty working sets, flush modified, then purge standby.
#[allow(dead_code)]
pub fn purge_all() -> Result<(WorkingSetPurgeResult, (), ()), MpaError> {
    let ws_result = purge_working_sets()?;
    let mod_result = purge_modified()?;
    let sb_result = purge_standby(false)?;
    Ok((ws_result, mod_result, sb_result))
}
