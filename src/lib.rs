//! Library types and command planning for `pod5-tools`.
//!
//! The crate is intentionally small at this stage. It establishes documented
//! public contracts for the planned command outputs and a `clap` parser that
//! the binary can expose while POD5 reading support is developed in later
//! slices.

use std::fmt;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::SystemTime;

use chrono::{DateTime, Utc};
use clap::{Parser, Subcommand, ValueEnum};
use serde::Serialize;

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
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, ValueEnum)]
#[serde(rename_all = "lowercase")]
pub enum OutputFormat {
    /// Tab-separated values for shell and workflow integration.
    Tsv,
    /// JSON for structured integrations.
    Json,
}

/// Metadata for one directory that contains POD5 files.
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
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
#[derive(Clone, Debug, PartialEq, Serialize)]
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
#[derive(Clone, Debug, PartialEq, Serialize)]
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
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
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

/// Result alias for POD5 metadata reader operations.
pub type Pod5ReaderResult<T> = Result<T, Pod5ReaderError>;

/// Error returned by POD5 metadata reader adapters.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Pod5ReaderError {
    /// The path could not be opened, read, or statted.
    Path {
        /// Path associated with the failure.
        path: PathBuf,
        /// Human-readable reason from the backend or operating system.
        reason: String,
    },
    /// The input is not a valid POD5 file or collection for the requested operation.
    Format {
        /// Path associated with the failure.
        path: PathBuf,
        /// Human-readable reason from the backend.
        reason: String,
    },
    /// The POD5 schema or metadata shape is unsupported or internally inconsistent.
    Schema {
        /// Path associated with the failure.
        path: PathBuf,
        /// Human-readable reason from the backend.
        reason: String,
    },
    /// Integrity verification found corruption or incomplete data.
    Integrity {
        /// Path associated with the failure.
        path: PathBuf,
        /// Human-readable reason from the backend.
        reason: String,
    },
}

impl Pod5ReaderError {
    /// Return the path associated with this reader error.
    pub fn path(&self) -> &Path {
        match self {
            Self::Path { path, .. }
            | Self::Format { path, .. }
            | Self::Schema { path, .. }
            | Self::Integrity { path, .. } => path,
        }
    }

    /// Return the category name used in machine-readable diagnostics.
    pub fn category(&self) -> &'static str {
        match self {
            Self::Path { .. } => "path",
            Self::Format { .. } => "format",
            Self::Schema { .. } => "schema",
            Self::Integrity { .. } => "integrity",
        }
    }
}

impl fmt::Display for Pod5ReaderError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Path { path, reason } => {
                write!(formatter, "path error for {}: {reason}", path.display())
            }
            Self::Format { path, reason } => {
                write!(formatter, "format error for {}: {reason}", path.display())
            }
            Self::Schema { path, reason } => {
                write!(formatter, "schema error for {}: {reason}", path.display())
            }
            Self::Integrity { path, reason } => {
                write!(
                    formatter,
                    "integrity error for {}: {reason}",
                    path.display()
                )
            }
        }
    }
}

impl std::error::Error for Pod5ReaderError {}

/// Filesystem-only POD5 metadata reader.
///
/// This reader validates that an input is a `.pod5` file and reports file size.
/// POD5-internal fields are left unavailable until a concrete POD5 parser
/// backend is connected behind `Pod5MetadataReader`.
#[derive(Debug, Default)]
pub struct FilesystemPod5MetadataReader;

impl Pod5MetadataReader for FilesystemPod5MetadataReader {
    fn read_file_info(&self, path: &Path) -> Pod5ReaderResult<Pod5FileInfo> {
        let metadata = fs::metadata(path).map_err(|error| Pod5ReaderError::Path {
            path: path.to_path_buf(),
            reason: error.to_string(),
        })?;
        if !metadata.is_file() {
            return Err(Pod5ReaderError::Path {
                path: path.to_path_buf(),
                reason: "expected a file".to_string(),
            });
        }
        if !is_pod5_path(path) {
            return Err(Pod5ReaderError::Format {
                path: path.to_path_buf(),
                reason: "expected a .pod5 file".to_string(),
            });
        }

        Ok(Pod5FileInfo {
            path: path.to_path_buf(),
            size_bytes: metadata.len(),
            flow_cell_id: None,
            sequencing_kit: None,
            read_count: None,
            acquisition_start_utc: None,
            duration_seconds: None,
            pod5_version: None,
            integrity: IntegrityStatus::Unavailable {
                reason:
                    "POD5 parser backend not configured; only filesystem metadata was inspected"
                        .to_string(),
            },
        })
    }
}

/// Adapter boundary for POD5 metadata access.
///
/// Implementations may be backed by Rust-native Arrow readers, bindings to the
/// official POD5 implementation, or test doubles. The trait is read-only and
/// returns metadata contracts used by the command layer.
pub trait Pod5MetadataReader {
    /// Read metadata and integrity state for one POD5 file.
    fn read_file_info(&self, path: &Path) -> Pod5ReaderResult<Pod5FileInfo>;
}

/// Read file metadata through a POD5 metadata reader adapter.
pub fn read_pod5_file_info(
    reader: &impl Pod5MetadataReader,
    path: &Path,
) -> Pod5ReaderResult<Pod5FileInfo> {
    reader.read_file_info(path)
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

impl From<Pod5ReaderError> for Pod5ToolsError {
    fn from(error: Pod5ReaderError) -> Self {
        Self::new(error.to_string())
    }
}

/// Dispatch a parsed command.
///
/// Current subcommands are stubs: they parse arguments and return a stable
/// "not yet implemented" message while command behavior is built in later
/// development slices.
pub fn run(cli: Cli) -> Result<String, Pod5ToolsError> {
    let command = match cli.command {
        Command::Find { path, format } => return run_find(path, format),
        Command::Fileinfo { path, format } => {
            return run_fileinfo(&FilesystemPod5MetadataReader, &path, format);
        }
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

/// Recursively find directories that contain POD5 files.
pub fn find_pod5_directories(root: PathBuf) -> Result<Vec<Pod5DirectoryRecord>, Pod5ToolsError> {
    let metadata = fs::metadata(&root).map_err(|error| {
        Pod5ToolsError::new(format!("failed to inspect {}: {error}", root.display()))
    })?;
    if !metadata.is_dir() {
        return Err(Pod5ToolsError::new(format!(
            "find expects a directory: {}",
            root.display()
        )));
    }

    let mut records = Vec::<Pod5DirectoryRecord>::new();
    collect_pod5_directories(&root, &mut records)?;
    records.sort_by(|left, right| left.path.cmp(&right.path));
    Ok(records)
}

fn collect_pod5_directories(
    directory: &Path,
    records: &mut Vec<Pod5DirectoryRecord>,
) -> Result<(), Pod5ToolsError> {
    let mut pod5_file_count = 0_u64;
    let mut total_bytes = 0_u64;
    let mut oldest_modified = None::<SystemTime>;
    let mut newest_modified = None::<SystemTime>;
    let mut child_directories = Vec::<PathBuf>::new();

    let entries = fs::read_dir(directory).map_err(|error| {
        Pod5ToolsError::new(format!("failed to read {}: {error}", directory.display()))
    })?;
    for entry in entries {
        let entry = entry.map_err(|error| {
            Pod5ToolsError::new(format!(
                "failed to read entry in {}: {error}",
                directory.display()
            ))
        })?;
        let path = entry.path();
        let metadata = entry.metadata().map_err(|error| {
            Pod5ToolsError::new(format!("failed to inspect {}: {error}", path.display()))
        })?;
        if metadata.is_dir() {
            child_directories.push(path);
        } else if metadata.is_file() && is_pod5_path(&path) {
            pod5_file_count += 1;
            total_bytes += metadata.len();
            if let Ok(modified) = metadata.modified() {
                oldest_modified = Some(match oldest_modified {
                    Some(current) => current.min(modified),
                    None => modified,
                });
                newest_modified = Some(match newest_modified {
                    Some(current) => current.max(modified),
                    None => modified,
                });
            }
        }
    }

    if pod5_file_count > 0 {
        records.push(Pod5DirectoryRecord {
            path: directory.to_path_buf(),
            pod5_file_count,
            total_bytes,
            oldest_modified_utc: oldest_modified.map(system_time_to_rfc3339),
            newest_modified_utc: newest_modified.map(system_time_to_rfc3339),
        });
    }

    child_directories.sort();
    for child in child_directories {
        collect_pod5_directories(&child, records)?;
    }

    Ok(())
}

fn is_pod5_path(path: &Path) -> bool {
    path.extension()
        .and_then(|extension| extension.to_str())
        .is_some_and(|extension| extension.eq_ignore_ascii_case("pod5"))
}

fn system_time_to_rfc3339(time: SystemTime) -> String {
    DateTime::<Utc>::from(time).to_rfc3339()
}

fn run_find(path: PathBuf, format: OutputFormat) -> Result<String, Pod5ToolsError> {
    let records = find_pod5_directories(path)?;
    match format {
        OutputFormat::Tsv => Ok(format_directory_records_tsv(&records)),
        OutputFormat::Json => serde_json::to_string_pretty(&records)
            .map_err(|error| Pod5ToolsError::new(format!("failed to serialize JSON: {error}"))),
    }
}

fn run_fileinfo(
    reader: &impl Pod5MetadataReader,
    path: &Path,
    format: OutputFormat,
) -> Result<String, Pod5ToolsError> {
    let info = read_pod5_file_info(reader, path)?;
    match format {
        OutputFormat::Tsv => Ok(format_file_info_tsv(&info)),
        OutputFormat::Json => serde_json::to_string_pretty(&info)
            .map_err(|error| Pod5ToolsError::new(format!("failed to serialize JSON: {error}"))),
    }
}

/// Format POD5 directory records as tab-separated text.
pub fn format_directory_records_tsv(records: &[Pod5DirectoryRecord]) -> String {
    let mut output = String::from(
        "path\tpod5_file_count\ttotal_bytes\toldest_modified_utc\tnewest_modified_utc",
    );
    for record in records {
        output.push('\n');
        output.push_str(&format!(
            "{}\t{}\t{}\t{}\t{}",
            record.path.display(),
            record.pod5_file_count,
            record.total_bytes,
            record.oldest_modified_utc.as_deref().unwrap_or(""),
            record.newest_modified_utc.as_deref().unwrap_or("")
        ));
    }
    output
}

/// Format one POD5 file metadata record as tab-separated text.
pub fn format_file_info_tsv(info: &Pod5FileInfo) -> String {
    let (integrity_status, integrity_reason) = integrity_tsv_fields(&info.integrity);
    format!(
        "path\tsize_bytes\tflow_cell_id\tsequencing_kit\tread_count\tacquisition_start_utc\tduration_seconds\tpod5_version\tintegrity_status\tintegrity_reason\n{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}",
        info.path.display(),
        info.size_bytes,
        info.flow_cell_id.as_deref().unwrap_or(""),
        info.sequencing_kit.as_deref().unwrap_or(""),
        info.read_count
            .map(|value| value.to_string())
            .unwrap_or_default(),
        info.acquisition_start_utc.as_deref().unwrap_or(""),
        info.duration_seconds
            .map(|value| value.to_string())
            .unwrap_or_default(),
        info.pod5_version.as_deref().unwrap_or(""),
        integrity_status,
        integrity_reason,
    )
}

fn integrity_tsv_fields(integrity: &IntegrityStatus) -> (&'static str, &str) {
    match integrity {
        IntegrityStatus::Passed => ("passed", ""),
        IntegrityStatus::Failed { reason } => ("failed", reason),
        IntegrityStatus::NotChecked => ("not_checked", ""),
        IntegrityStatus::Unavailable { reason } => ("unavailable", reason),
    }
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

    #[test]
    fn find_groups_pod5_files_by_immediate_parent_directory() {
        let root = tempfile::tempdir().unwrap();
        let sample_a = root.path().join("sample-a");
        let sample_b = root.path().join("nested").join("sample-b");
        fs::create_dir(&sample_a).unwrap();
        fs::create_dir_all(&sample_b).unwrap();
        fs::write(sample_a.join("reads-1.pod5"), b"1234").unwrap();
        fs::write(sample_a.join("reads-2.POD5"), b"12").unwrap();
        fs::write(sample_a.join("notes.txt"), b"ignore").unwrap();
        fs::write(sample_b.join("reads-3.pod5"), b"123").unwrap();

        let records = find_pod5_directories(root.path().to_path_buf()).unwrap();

        let sample_a_record = records
            .iter()
            .find(|record| record.path == sample_a)
            .expect("sample-a should be reported");
        let sample_b_record = records
            .iter()
            .find(|record| record.path == sample_b)
            .expect("sample-b should be reported");

        assert_eq!(records.len(), 2);
        assert_eq!(sample_a_record.pod5_file_count, 2);
        assert_eq!(sample_a_record.total_bytes, 6);
        assert_eq!(sample_b_record.pod5_file_count, 1);
        assert_eq!(sample_b_record.total_bytes, 3);
        assert!(sample_a_record.oldest_modified_utc.is_some());
        assert!(sample_a_record.newest_modified_utc.is_some());
    }

    #[test]
    fn find_tsv_includes_header_and_records() {
        let record = Pod5DirectoryRecord {
            path: PathBuf::from("/data/sample-a"),
            pod5_file_count: 2,
            total_bytes: 6,
            oldest_modified_utc: Some("2026-06-13T10:00:00+00:00".to_string()),
            newest_modified_utc: Some("2026-06-13T10:05:00+00:00".to_string()),
        };

        let output = format_directory_records_tsv(&[record]);

        assert!(output.starts_with("path\tpod5_file_count\ttotal_bytes"));
        assert!(output.contains("/data/sample-a\t2\t6"));
    }

    #[test]
    fn run_find_emits_json_when_requested() {
        let root = tempfile::tempdir().unwrap();
        fs::write(root.path().join("reads.pod5"), b"1234").unwrap();

        let cli = Cli::try_parse_from([
            "pod5-tools",
            "find",
            root.path().to_str().unwrap(),
            "--format",
            "json",
        ])
        .unwrap();
        let output = run(cli).unwrap();

        assert!(output.contains("\"pod5_file_count\": 1"));
        assert!(output.contains("\"total_bytes\": 4"));
    }

    #[test]
    fn filesystem_fileinfo_reports_size_and_unavailable_integrity() {
        let root = tempfile::tempdir().unwrap();
        let pod5 = root.path().join("reads.pod5");
        fs::write(&pod5, b"pod5-bytes").unwrap();

        let reader = FilesystemPod5MetadataReader;
        let info = read_pod5_file_info(&reader, &pod5).unwrap();

        assert_eq!(info.path, pod5);
        assert_eq!(info.size_bytes, 10);
        assert_eq!(info.flow_cell_id, None);
        assert!(matches!(
            info.integrity,
            IntegrityStatus::Unavailable { .. }
        ));
    }

    #[test]
    fn filesystem_fileinfo_rejects_non_pod5_files() {
        let root = tempfile::tempdir().unwrap();
        let text_file = root.path().join("reads.txt");
        fs::write(&text_file, b"not pod5").unwrap();

        let reader = FilesystemPod5MetadataReader;
        let error = read_pod5_file_info(&reader, &text_file).unwrap_err();

        assert_eq!(error.category(), "format");
        assert!(error.to_string().contains("expected a .pod5 file"));
    }

    #[test]
    fn run_fileinfo_emits_tsv_by_default() {
        let root = tempfile::tempdir().unwrap();
        let pod5 = root.path().join("reads.pod5");
        fs::write(&pod5, b"pod5").unwrap();

        let cli = Cli::try_parse_from(["pod5-tools", "fileinfo", pod5.to_str().unwrap()]).unwrap();
        let output = run(cli).unwrap();

        assert!(output.starts_with("path\tsize_bytes\tflow_cell_id"));
        assert!(output.contains("\t4\t"));
        assert!(output.contains("\tunavailable\tPOD5 parser backend not configured"));
    }

    #[test]
    fn run_fileinfo_emits_json_when_requested() {
        let root = tempfile::tempdir().unwrap();
        let pod5 = root.path().join("reads.pod5");
        fs::write(&pod5, b"pod5").unwrap();

        let cli = Cli::try_parse_from([
            "pod5-tools",
            "fileinfo",
            pod5.to_str().unwrap(),
            "--format",
            "json",
        ])
        .unwrap();
        let output = run(cli).unwrap();

        assert!(output.contains("\"size_bytes\": 4"));
        assert!(output.contains("\"flow_cell_id\": null"));
        assert!(output.contains("\"Unavailable\""));
    }

    #[derive(Debug)]
    struct MockReader {
        result: Pod5ReaderResult<Pod5FileInfo>,
    }

    impl Pod5MetadataReader for MockReader {
        fn read_file_info(&self, _path: &Path) -> Pod5ReaderResult<Pod5FileInfo> {
            self.result.clone()
        }
    }

    #[test]
    fn metadata_reader_trait_supports_mocked_file_info() {
        let expected = Pod5FileInfo {
            path: PathBuf::from("/data/reads.pod5"),
            size_bytes: 128,
            flow_cell_id: Some("PBI00001".to_string()),
            sequencing_kit: Some("SQK-LSK114".to_string()),
            read_count: Some(25),
            acquisition_start_utc: Some("2026-06-13T10:00:00Z".to_string()),
            duration_seconds: Some(42.0),
            pod5_version: Some("3".to_string()),
            integrity: IntegrityStatus::Passed,
        };
        let reader = MockReader {
            result: Ok(expected.clone()),
        };

        let observed = read_pod5_file_info(&reader, Path::new("/data/reads.pod5")).unwrap();

        assert_eq!(observed, expected);
    }

    #[test]
    fn reader_errors_keep_distinct_categories() {
        let path = PathBuf::from("/data/broken.pod5");
        let errors = [
            Pod5ReaderError::Path {
                path: path.clone(),
                reason: "permission denied".to_string(),
            },
            Pod5ReaderError::Format {
                path: path.clone(),
                reason: "not a POD5 container".to_string(),
            },
            Pod5ReaderError::Schema {
                path: path.clone(),
                reason: "unsupported schema version".to_string(),
            },
            Pod5ReaderError::Integrity {
                path: path.clone(),
                reason: "checksum mismatch".to_string(),
            },
        ];

        assert_eq!(errors[0].category(), "path");
        assert_eq!(errors[1].category(), "format");
        assert_eq!(errors[2].category(), "schema");
        assert_eq!(errors[3].category(), "integrity");
        assert!(errors[3].to_string().contains("integrity error"));
    }
}
