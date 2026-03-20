use std::mem;
use std::sync::OnceLock;

use windows_sys::Win32::System::LibraryLoader::{GetModuleHandleW, GetProcAddress};

use crate::error::MpaError;

// ---- NT types ----

type NtStatus = i32;

fn nt_success(status: NtStatus) -> bool {
    status >= 0
}

/// Physical memory page list information returned by NtQuerySystemInformation.
#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct SystemMemoryListInformation {
    pub zeroed_page_count: usize,
    pub free_page_count: usize,
    pub modified_page_count: usize,
    pub modified_no_write_page_count: usize,
    pub bad_page_count: usize,
    /// Standby page counts by priority (index 0 = lowest, 7 = highest)
    pub page_count_by_priority: [usize; 8],
    pub repurposed_pages_by_priority: [usize; 8],
    pub modified_page_count_page_file: usize,
}

impl SystemMemoryListInformation {
    pub fn total_standby_pages(&self) -> usize {
        self.page_count_by_priority.iter().sum()
    }
}

/// Commands for NtSetSystemInformation(SystemMemoryListInformation).
#[repr(u32)]
#[derive(Debug, Clone, Copy)]
#[allow(dead_code)]
pub enum MemoryListCommand {
    CaptureAccessedBits = 0,
    CaptureAndResetAccessedBits = 1,
    EmptyWorkingSets = 2,
    FlushModifiedList = 3,
    PurgeStandbyList = 4,
    PurgeLowPriorityStandbyList = 5,
}

/// SystemInformationClass value for memory list info.
const SYSTEM_MEMORY_LIST_INFORMATION: u32 = 80;

// ---- Function pointer types ----

type FnNtQuerySystemInformation =
    unsafe extern "system" fn(u32, *mut u8, u32, *mut u32) -> NtStatus;

type FnNtSetSystemInformation =
    unsafe extern "system" fn(u32, *const u8, u32) -> NtStatus;

struct NtApiFunctions {
    query: FnNtQuerySystemInformation,
    set: FnNtSetSystemInformation,
}

static NT_API: OnceLock<Result<NtApiFunctions, String>> = OnceLock::new();

fn load_ntapi() -> Result<&'static NtApiFunctions, MpaError> {
    let result = NT_API.get_or_init(|| {
        unsafe {
            let ntdll_name: Vec<u16> = "ntdll.dll\0".encode_utf16().collect();
            let h = GetModuleHandleW(ntdll_name.as_ptr());
            if h.is_null() {
                return Err("Failed to get ntdll.dll handle".to_string());
            }

            let query_name = b"NtQuerySystemInformation\0";
            let set_name = b"NtSetSystemInformation\0";

            let query_ptr = GetProcAddress(h, query_name.as_ptr());
            let set_ptr = GetProcAddress(h, set_name.as_ptr());

            match (query_ptr, set_ptr) {
                (Some(q), Some(s)) => Ok(NtApiFunctions {
                    query: mem::transmute(q),
                    set: mem::transmute(s),
                }),
                _ => Err("Failed to resolve NtQuerySystemInformation or NtSetSystemInformation".to_string()),
            }
        }
    });

    match result {
        Ok(f) => Ok(f),
        Err(e) => Err(MpaError::general(e)),
    }
}

/// Query the system memory list information (page list breakdown).
pub fn query_memory_list_info() -> Result<SystemMemoryListInformation, MpaError> {
    let api = load_ntapi()?;
    unsafe {
        let mut info: SystemMemoryListInformation = mem::zeroed();
        let mut return_length: u32 = 0;

        let status = (api.query)(
            SYSTEM_MEMORY_LIST_INFORMATION,
            &mut info as *mut _ as *mut u8,
            mem::size_of::<SystemMemoryListInformation>() as u32,
            &mut return_length,
        );

        if nt_success(status) {
            Ok(info)
        } else {
            Err(MpaError::winapi_with_code(
                "NtQuerySystemInformation(SystemMemoryListInformation)",
                status as u32,
            ))
        }
    }
}

/// Execute a memory list command (purge standby, flush modified, etc.)
pub fn execute_memory_command(command: MemoryListCommand) -> Result<(), MpaError> {
    let api = load_ntapi()?;
    unsafe {
        let cmd = command as u32;
        let status = (api.set)(
            SYSTEM_MEMORY_LIST_INFORMATION,
            &cmd as *const u32 as *const u8,
            mem::size_of::<u32>() as u32,
        );

        if nt_success(status) {
            Ok(())
        } else {
            Err(MpaError::winapi_with_code(
                &format!("NtSetSystemInformation({command:?})"),
                status as u32,
            ))
        }
    }
}
