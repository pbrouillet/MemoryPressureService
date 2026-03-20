use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "mpa", about = "Windows Memory Pressure Agent", version)]
pub struct Cli {
    /// Output in JSON format instead of table
    #[arg(long, global = true)]
    pub json: bool,

    #[command(subcommand)]
    pub command: Command,
}

#[derive(Subcommand)]
pub enum Command {
    /// Display system memory statistics
    Stats,

    /// Purge memory regions to relieve pressure
    Purge {
        #[command(subcommand)]
        target: PurgeTarget,
    },

    /// Run as a resident system-tray monitor
    Monitor,
}

#[derive(Subcommand)]
pub enum PurgeTarget {
    /// Empty working sets of all processes
    #[command(name = "workingsets")]
    WorkingSets,

    /// Purge the standby page list
    Standby {
        /// Only purge low-priority standby pages (priorities 0-3)
        #[arg(long)]
        low_only: bool,
    },

    /// Flush the modified page list to disk
    Modified,

    /// Purge all: empty working sets, flush modified, purge standby
    All,
}
