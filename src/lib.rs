//! Library types and command planning for `pod5-tools`.
//!
//! The crate is intentionally small at this stage. It establishes documented
//! public contracts for the planned command outputs and a `clap` parser that
//! the binary can expose while POD5 reading support is developed in later
//! slices.

use std::fmt;
use std::path::PathBuf;

use clap::{Parser, Subcommand, ValueEnum};

/// Command-line parser for the `pod5-tools` binary.
#[derive(Debug, Parser)]
#[command(version, about, long_about = None)]
pub struct Cli {
    /// Subcommand to run.
    #[command(subcommand)]
    pub command: Command,
}

/// Supported `pod5-tools` subcommands.
#[derive(Debug, Subcommand)]
pub enum Command {
    /// Locate directories containing one or more POD5 files.
    Find {
        /// Root path to search.
        path: PathBuf,
        /// Output format.
        #[arg(long, value_enum, default_value_t = OutputFormat::Tsv)]
        format: OutputFormat,
    },
    /// Inspect one POD5 file.
    Fileinfo {
        /// POD5 file to inspect.
        path: PathBuf,
        /// Output format.
        #[arg(long, value_enum, default_value_t = OutputFormat::Tsv)]
        format: OutputFormat,
    },
    /// Summarise a folder or run tree containing POD5 files.
    Folderinfo {
        /// Folder to inspect.
        path: PathBuf,
        /// Output format.
        #[arg(long, value_enum, default_value_t = OutputFormat::Tsv)]
        format: OutputFormat,
    },
    /// Create or emit a sequencing-like playback plan from existing data.
    Playback {
        /// Source POD5 folder or manifest.
        #[arg(long)]
        input: PathBuf,
        /// Output directory for generated playback artefacts.
        #[arg(long)]
        out: PathBuf,
    },
    /// Write a stable manifest for a POD5 collection.
    Manifest {
        /// Source POD5 file, folder, or run tree.
        path: PathBuf,
        /// Optional output path. Standard output is used when omitted.
        #[arg(long)]
        output: Option<PathBuf>,
        /// Output format.
        #[arg(long, value_enum, default_value_t = OutputFormat::Json)]
        format: OutputFormat,
    },
    /// Plan temporal or structural POD5 subdivisions.
    Subdivide {
        /// Source POD5 file, folder, or manifest.
        path: PathBuf,
    },
    /// Compare two POD5 collections or manifests.
    Compare {
        /// Left-hand collection or manifest.
        left: PathBuf,
        /// Right-hand collection or manifest.
        right: PathBuf,
    },
}

/// Machine-readable command output formats.
#[derive(Clone, Copy, Debug, Eq, PartialEq, ValueEnum)]
pub enum OutputFormat {
    /// Tab-separated values for shell and workflow integration.
    Tsv,
    /// JSON for structured integrations.
    Json,
}

/// Metadata for one directory that contains POD5 files.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Pod5DirectoryRecord {
    /// Directory containing one or more `.pod5` files.
    pub path: PathBuf,
    /// Number of POD5 files found directly or recursively, depending on the command mode.
    pub pod5_file_count: u64,
    /// Total bytes across the discovered POD5 files.
    pub total_bytes: u64,
    /// Oldest observed file modification time as RFC 3339 text when available.
    pub oldest_modified_utc: Option<String>,
    /// Newest observed file modification time as RFC 3339 text when available.
    pub newest_modified_utc: Option<String>,
}

/// File-level POD5 metadata and integrity status.
#[derive(Clone, Debug, PartialEq)]
pub struct Pod5FileInfo {
    /// POD5 file path.
    pub path: PathBuf,
    /// File size in bytes.
    pub size_bytes: u64,
    /// Flow cell identifier reported by the file metadata when available.
    pub flow_cell_id: Option<String>,
    /// Sequencing kit reported by the file metadata when available.
    pub sequencing_kit: Option<String>,
    /// Number of reads observed in the file when available.
    pub read_count: Option<u64>,
    /// Acquisition start time as RFC 3339 text when available.
    pub acquisition_start_utc: Option<String>,
    /// Signal duration in seconds when available.
    pub duration_seconds: Option<f64>,
    /// POD5 schema or writer version when available.
    pub pod5_version: Option<String>,
    /// Integrity status determined by the active reader.
    pub integrity: IntegrityStatus,
}

/// Folder-level summary across multiple POD5 files.
#[derive(Clone, Debug, PartialEq)]
pub struct Pod5FolderInfo {
    /// Folder or run tree that was inspected.
    pub path: PathBuf,
    /// Number of POD5 files included in the summary.
    pub pod5_file_count: u64,
    /// Total bytes across all included POD5 files.
    pub total_bytes: u64,
    /// Total reads across files where read counts were available.
    pub total_reads: Option<u64>,
    /// Distinct flow cell identifiers observed in the folder.
    pub flow_cell_ids: Vec<String>,
    /// Distinct sequencing kits observed in the folder.
    pub sequencing_kits: Vec<String>,
    /// Earliest acquisition start time as RFC 3339 text when available.
    pub acquisition_start_utc: Option<String>,
    /// Latest acquisition end time as RFC 3339 text when available.
    pub acquisition_end_utc: Option<String>,
    /// Integrity status summarising all inspected files.
    pub integrity: IntegrityStatus,
}

/// Integrity state for a POD5 file or collection.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum IntegrityStatus {
    /// The file or collection was checked and no integrity problem was found.
    Passed,
    /// The file or collection was checked and at least one problem was found.
    Failed {
        /// Human-readable reason for the failure.
        reason: String,
    },
    /// Integrity has not yet been checked.
    NotChecked,
    /// Integrity could not be checked by the active reader.
    Unavailable {
        /// Human-readable reason why integrity could not be checked.
        reason: String,
    },
}

/// Error returned while dispatching a command.
#[derive(Debug, Eq, PartialEq)]
pub struct Pod5ToolsError {
    message: String,
}

impl Pod5ToolsError {
    /// Create a new command error from a message.
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}

impl fmt::Display for Pod5ToolsError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.message.fmt(formatter)
    }
}

impl std::error::Error for Pod5ToolsError {}

/// Dispatch a parsed command.
///
/// Current subcommands are stubs: they parse arguments and return a stable
/// "not yet implemented" message while command behavior is built in later
/// development slices.
pub fn run(cli: Cli) -> Result<String, Pod5ToolsError> {
    let command = match cli.command {
        Command::Find { .. } => "find",
        Command::Fileinfo { .. } => "fileinfo",
        Command::Folderinfo { .. } => "folderinfo",
        Command::Playback { .. } => "playback",
        Command::Manifest { .. } => "manifest",
        Command::Subdivide { .. } => "subdivide",
        Command::Compare { .. } => "compare",
    };
    Ok(format!(
        "pod5-tools {command} is not implemented yet; see todo.md for the development plan"
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::Parser;

    #[test]
    fn cli_parses_find_with_default_tsv_output() {
        let cli = Cli::try_parse_from(["pod5-tools", "find", "/data"]).unwrap();
        let Command::Find { path, format } = cli.command else {
            panic!("expected find command");
        };
        assert_eq!(path, PathBuf::from("/data"));
        assert_eq!(format, OutputFormat::Tsv);
    }

    #[test]
    fn cli_parses_fileinfo_with_json_output() {
        let cli = Cli::try_parse_from([
            "pod5-tools",
            "fileinfo",
            "/data/reads.pod5",
            "--format",
            "json",
        ])
        .unwrap();
        let Command::Fileinfo { path, format } = cli.command else {
            panic!("expected fileinfo command");
        };
        assert_eq!(path, PathBuf::from("/data/reads.pod5"));
        assert_eq!(format, OutputFormat::Json);
    }

    #[test]
    fn cli_parses_playback_io_paths() {
        let cli = Cli::try_parse_from([
            "pod5-tools",
            "playback",
            "--input",
            "/data/source",
            "--out",
            "/tmp/playback",
        ])
        .unwrap();
        let Command::Playback { input, out } = cli.command else {
            panic!("expected playback command");
        };
        assert_eq!(input, PathBuf::from("/data/source"));
        assert_eq!(out, PathBuf::from("/tmp/playback"));
    }

    #[test]
    fn file_info_struct_preserves_planned_metadata() {
        let info = Pod5FileInfo {
            path: PathBuf::from("/data/reads.pod5"),
            size_bytes: 42,
            flow_cell_id: Some("PBI00001".to_string()),
            sequencing_kit: Some("SQK-LSK114".to_string()),
            read_count: Some(10),
            acquisition_start_utc: Some("2026-06-13T09:00:00Z".to_string()),
            duration_seconds: Some(120.5),
            pod5_version: Some("3".to_string()),
            integrity: IntegrityStatus::NotChecked,
        };

        assert_eq!(info.flow_cell_id.as_deref(), Some("PBI00001"));
        assert_eq!(info.read_count, Some(10));
        assert_eq!(info.integrity, IntegrityStatus::NotChecked);
    }

    #[test]
    fn run_returns_stub_message_for_parsed_command() {
        let cli = Cli::try_parse_from(["pod5-tools", "folderinfo", "/data"]).unwrap();
        let message = run(cli).unwrap();
        assert!(message.contains("folderinfo"));
        assert!(message.contains("not implemented yet"));
    }
}
