use std::ffi::c_void;
use std::mem;

use windows_sys::Win32::Foundation::{BOOL, HANDLE, LUID};
use windows_sys::Win32::Security::{
    AdjustTokenPrivileges, LookupPrivilegeValueW, SE_PRIVILEGE_ENABLED, TOKEN_ADJUST_PRIVILEGES,
    TOKEN_PRIVILEGES, TOKEN_QUERY,
};
use windows_sys::Win32::System::Threading::{GetCurrentProcess, OpenProcessToken};

use crate::error::MpaError;

/// Enable a named privilege on the current process token.
fn enable_privilege(name: &str) -> Result<(), MpaError> {
    unsafe {
        let mut token: HANDLE = std::ptr::null_mut();
        if OpenProcessToken(
            GetCurrentProcess(),
            TOKEN_ADJUST_PRIVILEGES | TOKEN_QUERY,
            &mut token,
        ) == 0
        {
            return Err(MpaError::privilege("Failed to open process token"));
        }

        let wide_name: Vec<u16> = name.encode_utf16().chain(std::iter::once(0)).collect();

        let mut luid: LUID = mem::zeroed();
        if LookupPrivilegeValueW(std::ptr::null(), wide_name.as_ptr(), &mut luid) == 0 {
            windows_sys::Win32::Foundation::CloseHandle(token);
            return Err(MpaError::privilege(&format!(
                "Failed to lookup privilege '{name}'"
            )));
        }

        let mut tp: TOKEN_PRIVILEGES = mem::zeroed();
        tp.PrivilegeCount = 1;
        tp.Privileges[0].Luid = luid;
        tp.Privileges[0].Attributes = SE_PRIVILEGE_ENABLED;

        let ok: BOOL = AdjustTokenPrivileges(
            token,
            0, // FALSE — don't disable all
            &tp,
            0,
            std::ptr::null_mut(),
            std::ptr::null_mut(),
        );

        windows_sys::Win32::Foundation::CloseHandle(token);

        if ok == 0 {
            return Err(MpaError::privilege("AdjustTokenPrivileges failed"));
        }

        // AdjustTokenPrivileges can "succeed" but not actually grant — check last error
        let err = windows_sys::Win32::Foundation::GetLastError();
        if err != 0 {
            return Err(MpaError::privilege(&format!(
                "Privilege not held (error {err}). Run as Administrator."
            )));
        }

        Ok(())
    }
}

/// Enable SeProfileSingleProcessPrivilege (required for NtSetSystemInformation memory commands).
pub fn elevate_for_purge() -> Result<(), MpaError> {
    enable_privilege("SeProfileSingleProcessPrivilege")?;
    enable_privilege("SeDebugPrivilege")?;
    Ok(())
}

/// Check if the current process is running elevated (admin).
pub fn is_elevated() -> bool {
    use windows_sys::Win32::Security::{GetTokenInformation, TokenElevation, TOKEN_ELEVATION, TOKEN_QUERY as TQ};
    unsafe {
        let mut token: HANDLE = std::ptr::null_mut();
        if OpenProcessToken(GetCurrentProcess(), TQ, &mut token) == 0 {
            return false;
        }

        let mut elevation: TOKEN_ELEVATION = mem::zeroed();
        let mut size: u32 = 0;
        let ok = GetTokenInformation(
            token,
            TokenElevation,
            &mut elevation as *mut _ as *mut c_void,
            mem::size_of::<TOKEN_ELEVATION>() as u32,
            &mut size,
        );

        windows_sys::Win32::Foundation::CloseHandle(token);
        ok != 0 && elevation.TokenIsElevated != 0
    }
}
