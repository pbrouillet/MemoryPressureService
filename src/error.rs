use std::fmt;

#[derive(Debug)]
pub enum MpaError {
    /// Windows API call failed
    WinApi { context: String, code: u32 },
    /// Privilege / elevation issue
    Privilege(String),
    /// General error
    General(String),
}

impl MpaError {
    pub fn winapi(context: &str) -> Self {
        let code = unsafe { windows_sys::Win32::Foundation::GetLastError() };
        Self::WinApi {
            context: context.to_string(),
            code,
        }
    }

    pub fn winapi_with_code(context: &str, code: u32) -> Self {
        Self::WinApi {
            context: context.to_string(),
            code,
        }
    }

    pub fn privilege(msg: &str) -> Self {
        Self::Privilege(msg.to_string())
    }

    pub fn general(msg: &str) -> Self {
        Self::General(msg.to_string())
    }
}

impl fmt::Display for MpaError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::WinApi { context, code } => {
                write!(f, "{context} (Win32 error 0x{code:08X})")
            }
            Self::Privilege(msg) => write!(f, "Privilege error: {msg}"),
            Self::General(msg) => write!(f, "{msg}"),
        }
    }
}

impl std::error::Error for MpaError {}
