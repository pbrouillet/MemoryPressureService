/** IPC bridge provided by wry WebView */
interface WryIpc {
  postMessage(message: string): void;
}

declare global {
  interface Window {
    ipc: WryIpc;
    // Stats window globals (called from Rust via evaluate_script)
    updateStats?: (data: MemoryStats) => void;
    showError?: (msg: string) => void;
    // Settings window globals (called from Rust via evaluate_script)
    loadSettings?: (settings: SettingsData) => void;
    showToast?: (msg: string) => void;
  }
}

export interface MemoryStats {
  memory_load_percent: number;
  total_physical_mb: number;
  available_physical_mb: number;
  total_commit_mb: number;
  available_commit_mb: number;
  page_size_bytes: number;
  zeroed_pages: number;
  free_pages: number;
  modified_pages: number;
  modified_no_write_pages: number;
  bad_pages: number;
  total_standby_pages: number;
  standby_pages_by_priority: number[];
  zeroed_mb: number;
  free_mb: number;
  modified_mb: number;
  standby_mb: number;
  commit_total_mb: number;
  commit_limit_mb: number;
  system_cache_mb: number;
  kernel_paged_mb: number;
  kernel_nonpaged_mb: number;
  process_count: number;
  thread_count: number;
  handle_count: number;
}

export type ThresholdAction = "none" | "notify" | "purge";

export interface ThresholdConfig {
  warning: number;
  warning_action: ThresholdAction;
  critical: number;
  critical_action: ThresholdAction;
}

export interface SettingsData {
  memory_load: ThresholdConfig;
  modified_list: ThresholdConfig;
  standby_list: ThresholdConfig;
  available_memory: ThresholdConfig;
}
