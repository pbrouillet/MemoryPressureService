use comfy_table::{presets::UTF8_FULL, Cell, Color, ContentArrangement, Table};

use crate::stats::MemoryStats;

pub fn render(stats: &MemoryStats, json: bool) {
    if json {
        render_json(stats);
    } else {
        render_table(stats);
    }
}

fn render_json(stats: &MemoryStats) {
    let json = serde_json::to_string_pretty(stats).expect("Failed to serialize stats");
    println!("{json}");
}

/// Build the stats tables as a single String (for use in GUI windows).
#[allow(dead_code)]
pub fn render_table_to_string(stats: &MemoryStats) -> String {
    let mut out = String::new();
    out.push_str(&build_overview_table(stats).to_string());
    out.push_str("\r\n\r\n");
    out.push_str(&build_pages_table(stats).to_string());
    out.push_str("\r\n\r\n");
    out.push_str(&build_standby_table(stats).to_string());
    out
}

fn render_table(stats: &MemoryStats) {
    println!("{}", build_overview_table(stats));
    println!();
    println!("{}", build_pages_table(stats));
    println!();
    println!("{}", build_standby_table(stats));
}

fn build_overview_table(stats: &MemoryStats) -> Table {
    let mut overview = Table::new();
    overview.load_preset(UTF8_FULL);
    overview.set_content_arrangement(ContentArrangement::Dynamic);
    overview.set_header(vec![
        Cell::new("System Overview").fg(Color::Cyan),
        Cell::new("Value").fg(Color::Cyan),
    ]);
    overview.add_row(vec![
        "Memory Load",
        &format!("{}%", stats.memory_load_percent),
    ]);
    overview.add_row(vec![
        "Total Physical",
        &format!("{:.1} MB", stats.total_physical_mb),
    ]);
    overview.add_row(vec![
        "Available Physical",
        &format!("{:.1} MB", stats.available_physical_mb),
    ]);
    overview.add_row(vec![
        "Commit (Used / Limit)",
        &format!(
            "{:.1} / {:.1} MB",
            stats.commit_total_mb, stats.commit_limit_mb
        ),
    ]);
    overview.add_row(vec![
        "System Cache",
        &format!("{:.1} MB", stats.system_cache_mb),
    ]);
    overview.add_row(vec![
        "Kernel Paged / Nonpaged",
        &format!(
            "{:.1} / {:.1} MB",
            stats.kernel_paged_mb, stats.kernel_nonpaged_mb
        ),
    ]);
    overview.add_row(vec![
        "Processes / Threads / Handles",
        &format!(
            "{} / {} / {}",
            stats.process_count, stats.thread_count, stats.handle_count
        ),
    ]);
    overview
}

fn build_pages_table(stats: &MemoryStats) -> Table {
    let mut pages = Table::new();
    pages.load_preset(UTF8_FULL);
    pages.set_content_arrangement(ContentArrangement::Dynamic);
    pages.set_header(vec![
        Cell::new("Page List").fg(Color::Green),
        Cell::new("Pages").fg(Color::Green),
        Cell::new("Size (MB)").fg(Color::Green),
    ]);
    pages.add_row(vec![
        "Zeroed".to_string(),
        format_pages(stats.zeroed_pages),
        format!("{:.1}", stats.zeroed_mb),
    ]);
    pages.add_row(vec![
        "Free".to_string(),
        format_pages(stats.free_pages),
        format!("{:.1}", stats.free_mb),
    ]);
    pages.add_row(vec![
        "Modified".to_string(),
        format_pages(stats.modified_pages),
        format!("{:.1}", stats.modified_mb),
    ]);
    pages.add_row(vec![
        "Standby (total)".to_string(),
        format_pages(stats.total_standby_pages),
        format!("{:.1}", stats.standby_mb),
    ]);
    if stats.bad_pages > 0 {
        pages.add_row(vec![
            "Bad".to_string(),
            format_pages(stats.bad_pages),
            format!(
                "{:.1}",
                stats.bad_pages as f64 * stats.page_size_bytes as f64 / 1_048_576.0
            ),
        ]);
    }
    pages
}

fn build_standby_table(stats: &MemoryStats) -> Table {
    let mut standby = Table::new();
    standby.load_preset(UTF8_FULL);
    standby.set_content_arrangement(ContentArrangement::Dynamic);
    standby.set_header(vec![
        Cell::new("Standby Priority").fg(Color::Yellow),
        Cell::new("Pages").fg(Color::Yellow),
        Cell::new("Size (MB)").fg(Color::Yellow),
    ]);
    for (i, &count) in stats.standby_pages_by_priority.iter().enumerate() {
        let label = match i {
            0 => "0 (Lowest)".to_string(),
            7 => "7 (Highest)".to_string(),
            _ => format!("{i}"),
        };
        standby.add_row(vec![
            label,
            format_pages(count),
            format!(
                "{:.1}",
                count as f64 * stats.page_size_bytes as f64 / 1_048_576.0
            ),
        ]);
    }
    standby
}

fn format_pages(count: usize) -> String {
    // Add thousand separators
    let s = count.to_string();
    let mut result = String::new();
    for (i, c) in s.chars().rev().enumerate() {
        if i > 0 && i % 3 == 0 {
            result.push(',');
        }
        result.push(c);
    }
    result.chars().rev().collect()
}

/// Print a compact diff line showing what changed between before and after stats.
pub fn print_diff(before: &MemoryStats, after: &MemoryStats) {
    let freed_mb = after.available_physical_mb - before.available_physical_mb;
    let standby_delta = after.standby_mb - before.standby_mb;
    let modified_delta = after.modified_mb - before.modified_mb;

    println!();
    println!("  \u{2500}\u{2500}\u{2500} Delta \u{2500}\u{2500}\u{2500}");
    println!(
        "  Available:  {:+.1} MB  ({:.1} \u{2192} {:.1})",
        freed_mb, before.available_physical_mb, after.available_physical_mb
    );
    println!(
        "  Standby:    {:+.1} MB  ({:.1} \u{2192} {:.1})",
        standby_delta, before.standby_mb, after.standby_mb
    );
    println!(
        "  Modified:   {:+.1} MB  ({:.1} \u{2192} {:.1})",
        modified_delta, before.modified_mb, after.modified_mb
    );
    println!();
}
