use std::mem;

use serde::Serialize;
use windows_sys::Win32::System::ProcessStatus::GetPerformanceInfo;
use windows_sys::Win32::System::SystemInformation::{GlobalMemoryStatusEx, MEMORYSTATUSEX};

use crate::error::MpaError;
use crate::ntapi;

#[derive(Debug, Clone, Serialize)]
pub struct MemoryStats {
    // High-level
    pub memory_load_percent: u32,
    pub total_physical_mb: f64,
    pub available_physical_mb: f64,
    pub total_commit_mb: f64,
    pub available_commit_mb: f64,

    // Page list breakdown
    pub page_size_bytes: usize,
    pub zeroed_pages: usize,
    pub free_pages: usize,
    pub modified_pages: usize,
    pub modified_no_write_pages: usize,
    pub bad_pages: usize,
    pub total_standby_pages: usize,
    pub standby_pages_by_priority: [usize; 8],

    // Derived (in MB)
    pub zeroed_mb: f64,
    pub free_mb: f64,
    pub modified_mb: f64,
    pub standby_mb: f64,

    // Performance info
    pub commit_total_mb: f64,
    pub commit_limit_mb: f64,
    pub system_cache_mb: f64,
    pub kernel_paged_mb: f64,
    pub kernel_nonpaged_mb: f64,
    pub process_count: u32,
    pub thread_count: u32,
    pub handle_count: u32,
}

fn pages_to_mb(pages: usize, page_size: usize) -> f64 {
    (pages as f64 * page_size as f64) / (1024.0 * 1024.0)
}

fn bytes_to_mb(bytes: u64) -> f64 {
    bytes as f64 / (1024.0 * 1024.0)
}

pub fn collect_stats() -> Result<MemoryStats, MpaError> {
    // 1. GlobalMemoryStatusEx
    let ms = unsafe {
        let mut ms: MEMORYSTATUSEX = mem::zeroed();
        ms.dwLength = mem::size_of::<MEMORYSTATUSEX>() as u32;
        if GlobalMemoryStatusEx(&mut ms) == 0 {
            return Err(MpaError::winapi("GlobalMemoryStatusEx"));
        }
        ms
    };

    // 2. NtQuerySystemInformation for page list breakdown
    let mli = ntapi::query_memory_list_info()?;

    // 3. GetPerformanceInfo
    let pi = unsafe {
        let mut pi: windows_sys::Win32::System::ProcessStatus::PERFORMANCE_INFORMATION = mem::zeroed();
        pi.cb = mem::size_of::<windows_sys::Win32::System::ProcessStatus::PERFORMANCE_INFORMATION>() as u32;
        if GetPerformanceInfo(
            &mut pi,
            mem::size_of::<windows_sys::Win32::System::ProcessStatus::PERFORMANCE_INFORMATION>() as u32,
        ) == 0
        {
            return Err(MpaError::winapi("GetPerformanceInfo"));
        }
        pi
    };

    let page_size = pi.PageSize;

    Ok(MemoryStats {
        memory_load_percent: ms.dwMemoryLoad,
        total_physical_mb: bytes_to_mb(ms.ullTotalPhys),
        available_physical_mb: bytes_to_mb(ms.ullAvailPhys),
        total_commit_mb: bytes_to_mb(ms.ullTotalPageFile),
        available_commit_mb: bytes_to_mb(ms.ullAvailPageFile),

        page_size_bytes: page_size,
        zeroed_pages: mli.zeroed_page_count,
        free_pages: mli.free_page_count,
        modified_pages: mli.modified_page_count,
        modified_no_write_pages: mli.modified_no_write_page_count,
        bad_pages: mli.bad_page_count,
        total_standby_pages: mli.total_standby_pages(),
        standby_pages_by_priority: mli.page_count_by_priority,

        zeroed_mb: pages_to_mb(mli.zeroed_page_count, page_size),
        free_mb: pages_to_mb(mli.free_page_count, page_size),
        modified_mb: pages_to_mb(mli.modified_page_count, page_size),
        standby_mb: pages_to_mb(mli.total_standby_pages(), page_size),

        commit_total_mb: pages_to_mb(pi.CommitTotal, page_size),
        commit_limit_mb: pages_to_mb(pi.CommitLimit, page_size),
        system_cache_mb: pages_to_mb(pi.SystemCache, page_size),
        kernel_paged_mb: pages_to_mb(pi.KernelPaged, page_size),
        kernel_nonpaged_mb: pages_to_mb(pi.KernelNonpaged, page_size),
        process_count: pi.ProcessCount as u32,
        thread_count: pi.ThreadCount as u32,
        handle_count: pi.HandleCount as u32,
    })
}
