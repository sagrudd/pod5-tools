//! Library types and command planning for `pod5-tools`.
//!
//! The crate is intentionally small at this stage. It establishes documented
//! public contracts for the planned command outputs and a `clap` parser that
//! the binary can expose while POD5 reading support is developed in later
//! slices.

use std::collections::{BTreeMap, BTreeSet};
use std::fmt;
use std::fs;
use std::io::{Read, Seek, SeekFrom};
use std::path::{Path, PathBuf};
use std::time::SystemTime;

use chrono::{DateTime, Utc};
use clap::{Parser, Subcommand, ValueEnum};
use serde::{Deserialize, Serialize};

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
    /// Verify that one file is specification-adherent POD5.
    Verify {
        /// POD5 file to verify.
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
        /// Playback action to run.
        #[command(subcommand)]
        command: PlaybackCommand,
    },
    /// Write a stable manifest for a POD5 collection.
    Manifest {
        /// Source POD5 file, folder, or run tree.
        path: PathBuf,
        /// Optional output path. Standard output is used when omitted.
        #[arg(long)]
        output: Option<PathBuf>,
        /// Output format.
        #[arg(long, value_enum, default_value_t = OutputFormat::Tsv)]
        format: OutputFormat,
    },
    /// Plan temporal or structural POD5 subdivisions.
    Subdivide {
        /// Subdivision action to run.
        #[command(subcommand)]
        command: SubdivideCommand,
    },
    /// Compare two POD5 collections or manifests.
    Compare {
        /// Left-hand collection or manifest.
        left: PathBuf,
        /// Right-hand collection or manifest.
        right: PathBuf,
        /// Output format.
        #[arg(long, value_enum, default_value_t = OutputFormat::Tsv)]
        format: OutputFormat,
    },
}

/// Supported playback actions.
#[derive(Debug, Subcommand)]
pub enum PlaybackCommand {
    /// Inspect playback manifests and report a merged schedule.
    Plan {
        /// Playback manifest JSON file. Repeat for multiple sample streams.
        #[arg(long, required = true)]
        manifest: Vec<PathBuf>,
        /// Sample label. Repeat once per manifest.
        #[arg(long, required = true)]
        sample: Vec<String>,
        /// Original input path. Repeat once per manifest, or omit to infer from the manifest path.
        #[arg(long)]
        input: Vec<PathBuf>,
        /// Optional output path. Standard output is used when omitted.
        #[arg(long)]
        output: Option<PathBuf>,
        /// Output format.
        #[arg(long, value_enum, default_value_t = OutputFormat::Tsv)]
        format: OutputFormat,
    },
    /// Report the batch emission order and wall-clock waits for a playback schedule.
    Emit {
        /// Playback manifest JSON file. Repeat for multiple sample streams.
        #[arg(long, required = true)]
        manifest: Vec<PathBuf>,
        /// Sample label. Repeat once per manifest.
        #[arg(long, required = true)]
        sample: Vec<String>,
        /// Original input path. Repeat once per manifest, or omit to infer from the manifest path.
        #[arg(long)]
        input: Vec<PathBuf>,
        /// Accelerate wall-clock playback while preserving source sequencing timestamps.
        #[arg(long, default_value = "1x")]
        speedup: String,
        /// Output format.
        #[arg(long, value_enum, default_value_t = OutputFormat::Tsv)]
        format: OutputFormat,
    },
}

/// Supported subdivision actions.
#[derive(Debug, Subcommand)]
pub enum SubdivideCommand {
    /// Create a read-only subdivision plan.
    Plan {
        /// Source POD5 file, folder, or manifest.
        path: PathBuf,
        /// Planning strategy.
        #[arg(long, value_enum, default_value_t = SubdivideStrategy::FileCount)]
        strategy: SubdivideStrategy,
        /// Maximum files per chunk for file-count planning.
        #[arg(long, default_value_t = 1)]
        files_per_chunk: u64,
        /// Target elapsed seconds per chunk for elapsed-time planning.
        #[arg(long)]
        seconds_per_chunk: Option<u64>,
        /// Target reads per chunk for read-count planning.
        #[arg(long)]
        reads_per_chunk: Option<u64>,
        /// Optional output path. Standard output is used when omitted.
        #[arg(long)]
        output: Option<PathBuf>,
        /// Output format.
        #[arg(long, value_enum, default_value_t = OutputFormat::Tsv)]
        format: OutputFormat,
    },
}

/// Machine-readable command output formats.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize, ValueEnum)]
#[serde(rename_all = "lowercase")]
pub enum OutputFormat {
    /// Tab-separated values for shell and workflow integration.
    Tsv,
    /// JSON for structured integrations.
    Json,
}

/// Read-only subdivision planning strategies.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize, ValueEnum)]
#[serde(rename_all = "kebab-case")]
pub enum SubdivideStrategy {
    /// Group manifest entries into chunks containing at most `files_per_chunk` files.
    FileCount,
    /// Group files by acquisition elapsed time when temporal metadata is available.
    ElapsedTime,
    /// Group files by read counts when POD5 read-count metadata is available.
    ReadCount,
    /// Group files by sample label inferred from the first manifest-relative path component.
    SampleLabel,
}

/// Metadata for one directory that contains POD5 files.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
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
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
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
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
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
    /// Number of files that could not be read by the active metadata reader.
    pub failed_file_count: u64,
    /// Number of files with at least one implemented verification failure.
    pub verification_failed_count: u64,
    /// Duplicate POD5 basenames observed in different paths.
    pub duplicate_file_names: Vec<String>,
    /// Warnings for checks that could not be completed with the active reader.
    pub warnings: Vec<String>,
}

/// Overall status from `pod5-tools verify`.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum VerifyStatus {
    /// All implemented checks passed, but some specification checks are not implemented yet.
    Incomplete,
    /// One or more implemented checks failed.
    Failed,
    /// Every required check passed.
    Passed,
}

/// Status for an individual verification check.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum VerifyCheckStatus {
    /// The check passed.
    Passed,
    /// The check failed.
    Failed,
    /// The check is part of the verification contract but is not implemented yet.
    NotChecked,
}

/// One verification check result.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct VerifyCheck {
    /// Stable check identifier for machine-readable output.
    pub name: String,
    /// Category used to group failures.
    pub category: String,
    /// Check result.
    pub status: VerifyCheckStatus,
    /// Human-readable check detail.
    pub detail: String,
}

/// Verification report for one candidate POD5 file.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct Pod5VerifyReport {
    /// Candidate file path.
    pub path: PathBuf,
    /// File size in bytes.
    pub size_bytes: u64,
    /// Overall verification status.
    pub status: VerifyStatus,
    /// Individual checks performed or reserved by the verification contract.
    pub checks: Vec<VerifyCheck>,
}

/// Current manifest schema version.
pub const MANIFEST_SCHEMA_VERSION: u32 = 1;

/// Versioned manifest for a POD5 collection.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct Pod5Manifest {
    /// Manifest schema version.
    pub schema_version: u32,
    /// Source path used to create the manifest.
    pub source: PathBuf,
    /// Manifest entries, sorted by relative path.
    pub entries: Vec<Pod5ManifestEntry>,
}

/// One POD5 file inventory entry.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct Pod5ManifestEntry {
    /// Path relative to the manifest source when possible.
    pub relative_path: PathBuf,
    /// Absolute or invocation-resolved file path.
    pub path: PathBuf,
    /// File size in bytes.
    pub size_bytes: u64,
    /// Fast verification status for the file.
    pub verification_status: VerifyStatus,
    /// Number of implemented verification checks that failed.
    pub verification_failed_checks: u64,
}

/// Overall comparison status for automation.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CompareStatus {
    /// Inputs have no manifest-level differences.
    Match,
    /// Inputs differ.
    Different,
}

/// Manifest or folder comparison report.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct Pod5CompareReport {
    /// Overall comparison status.
    pub status: CompareStatus,
    /// Entries present only in the left input.
    pub missing_from_right: Vec<PathBuf>,
    /// Entries present only in the right input.
    pub missing_from_left: Vec<PathBuf>,
    /// Entries present on both sides but with different inventory fields.
    pub changed: Vec<Pod5CompareChange>,
}

/// One changed POD5 manifest entry.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct Pod5CompareChange {
    /// Relative path shared by both compared inputs.
    pub relative_path: PathBuf,
    /// Left file size in bytes.
    pub left_size_bytes: u64,
    /// Right file size in bytes.
    pub right_size_bytes: u64,
    /// Left verification status.
    pub left_verification_status: VerifyStatus,
    /// Right verification status.
    pub right_verification_status: VerifyStatus,
}

/// Current subdivision plan schema version.
pub const SUBDIVIDE_PLAN_SCHEMA_VERSION: u32 = 1;

/// Read-only plan describing how a POD5 collection could be subdivided.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct Pod5SubdividePlan {
    /// Subdivision plan schema version.
    pub schema_version: u32,
    /// Source path used to create the plan.
    pub source: PathBuf,
    /// Planning strategy used to create chunks.
    pub strategy: SubdivideStrategy,
    /// Human-readable target used by the selected strategy.
    pub target: String,
    /// Planned chunks, sorted deterministically.
    pub chunks: Vec<Pod5SubdivideChunk>,
    /// Warnings describing metadata gaps or planning limitations.
    pub warnings: Vec<String>,
}

/// One planned subdivision chunk.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct Pod5SubdivideChunk {
    /// Stable 1-based chunk index.
    pub index: u64,
    /// Suggested label for sidecar manifests or future output folders.
    pub label: String,
    /// Manifest-relative POD5 paths assigned to the chunk.
    pub relative_paths: Vec<PathBuf>,
    /// Number of files assigned to the chunk.
    pub file_count: u64,
    /// Total bytes assigned to the chunk.
    pub total_bytes: u64,
    /// Read count assigned to the chunk when available.
    pub read_count: Option<u64>,
}

/// Manifest produced by a playback planner for one POD5 stream.
///
/// This mirrors the generic scheduling contract from the Mnematikon playback
/// implementation while omitting API session, upload, and flowcell concerns.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct PlaybackManifest {
    /// Number of reads included in playback batches.
    pub read_count: u64,
    /// Number of source POD5 files used to create the playback stream.
    pub source_pod5_count: u64,
    /// Source POD5 files used to create the playback stream.
    #[serde(default)]
    pub source_pod5_files: Vec<PathBuf>,
    /// Sequencing seconds represented by each planning bucket.
    pub tempo_seconds: f64,
    /// Optional elapsed-sequencing cutoff in seconds.
    pub cutoff_seconds: Option<f64>,
    /// Path to the planner master table when available.
    pub master_table: PathBuf,
    /// Playback batches sorted by source sequencing time.
    pub batches: Vec<PlaybackBatch>,
}

/// One playback batch scheduled for later emission.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct PlaybackBatch {
    /// POD5 batch file to emit.
    pub path: PathBuf,
    /// Stable 1-based batch index.
    pub batch_index: u64,
    /// Number of reads contained in the batch.
    pub read_count: u64,
    /// Number of source POD5 files contributing to this batch.
    pub source_pod5_count: u64,
    /// Source POD5 files contributing to this batch.
    #[serde(default)]
    pub source_pod5_files: Vec<PathBuf>,
    /// Bucket start in source sequencing elapsed seconds.
    pub bucket_start_seconds: f64,
    /// Minimum read elapsed time in seconds when available.
    pub min_elapsed_seconds: Option<f64>,
    /// Maximum read elapsed time in seconds when available.
    pub max_elapsed_seconds: Option<f64>,
}

/// Playback plan for one named sample stream.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct PlaybackSamplePlan {
    /// Sample or stream label.
    pub sample: String,
    /// Original input path.
    pub input: PathBuf,
    /// Path to the playback manifest.
    pub manifest_path: PathBuf,
    /// Playback manifest loaded from `manifest_path`.
    pub manifest: PlaybackManifest,
}

/// Report returned by `playback plan`.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct PlaybackPlanReport {
    /// Sample plans loaded from playback manifests.
    pub sample_plans: Vec<PlaybackSamplePlan>,
    /// Merged source sequencing seconds at which at least one batch is emitted.
    pub schedule_seconds: Vec<f64>,
}

/// Report returned by `playback emit`.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct PlaybackEmitReport {
    /// Numeric wall-clock speedup factor.
    pub speedup: f64,
    /// Human-readable speedup label.
    pub speedup_label: String,
    /// Ordered batch emission events.
    pub events: Vec<PlaybackEmitEvent>,
}

/// One dry-run playback emission event.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct PlaybackEmitEvent {
    /// 1-based emission round index from the merged schedule.
    pub round_index: u64,
    /// Sample or stream label.
    pub sample: String,
    /// Batch file to emit.
    pub batch_path: PathBuf,
    /// Stable 1-based batch index.
    pub batch_index: u64,
    /// Source sequencing second at which this batch is emitted.
    pub emit_seconds: f64,
    /// Source sequencing seconds since the previous emitted bucket.
    pub sequence_wait_seconds: f64,
    /// Wall-clock seconds to wait after applying speedup.
    pub wall_wait_seconds: f64,
    /// Number of reads contained in the batch.
    pub read_count: u64,
}

/// Playback cutoff for source sequencing time.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub enum PlaybackCutoff {
    /// Emit reads up to the provided elapsed-sequencing second.
    ElapsedSeconds(f64),
    /// Emit all source reads.
    All,
}

impl PlaybackCutoff {
    /// Return the cutoff in elapsed seconds, or `None` when all reads are requested.
    pub fn seconds(self) -> Option<f64> {
        match self {
            Self::ElapsedSeconds(seconds) => Some(seconds),
            Self::All => None,
        }
    }
}

/// Integrity state for a POD5 file or collection.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
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

/// Dispatch a parsed command and return rendered command output.
pub fn run(cli: Cli) -> Result<String, Pod5ToolsError> {
    match cli.command {
        Command::Find { path, format } => run_find(path, format),
        Command::Fileinfo { path, format } => {
            run_fileinfo(&FilesystemPod5MetadataReader, &path, format)
        }
        Command::Verify { path, format } => run_verify(&path, format),
        Command::Folderinfo { path, format } => run_folderinfo(&path, format),
        Command::Playback { command } => match command {
            PlaybackCommand::Plan {
                manifest,
                sample,
                input,
                output,
                format,
            } => run_playback_plan(&manifest, &sample, &input, output.as_deref(), format),
            PlaybackCommand::Emit {
                manifest,
                sample,
                input,
                speedup,
                format,
            } => run_playback_emit(&manifest, &sample, &input, &speedup, format),
        },
        Command::Manifest {
            path,
            output,
            format,
        } => run_manifest(&path, output.as_deref(), format),
        Command::Subdivide { command } => match command {
            SubdivideCommand::Plan {
                path,
                strategy,
                files_per_chunk,
                seconds_per_chunk,
                reads_per_chunk,
                output,
                format,
            } => run_subdivide_plan(
                &path,
                strategy,
                files_per_chunk,
                seconds_per_chunk,
                reads_per_chunk,
                output.as_deref(),
                format,
            ),
        },
        Command::Compare {
            left,
            right,
            format,
        } => run_compare(&left, &right, format),
    }
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

fn run_verify(path: &Path, format: OutputFormat) -> Result<String, Pod5ToolsError> {
    let report = verify_pod5_file(path)?;
    match format {
        OutputFormat::Tsv => Ok(format_verify_report_tsv(&report)),
        OutputFormat::Json => serde_json::to_string_pretty(&report)
            .map_err(|error| Pod5ToolsError::new(format!("failed to serialize JSON: {error}"))),
    }
}

fn run_folderinfo(path: &Path, format: OutputFormat) -> Result<String, Pod5ToolsError> {
    let info = folder_info(path, &FilesystemPod5MetadataReader)?;
    match format {
        OutputFormat::Tsv => Ok(format_folder_info_tsv(&info)),
        OutputFormat::Json => serde_json::to_string_pretty(&info)
            .map_err(|error| Pod5ToolsError::new(format!("failed to serialize JSON: {error}"))),
    }
}

fn run_manifest(
    path: &Path,
    output: Option<&Path>,
    format: OutputFormat,
) -> Result<String, Pod5ToolsError> {
    let manifest = manifest_from_path(path)?;
    let rendered = match format {
        OutputFormat::Tsv => format_manifest_tsv(&manifest),
        OutputFormat::Json => serde_json::to_string_pretty(&manifest)
            .map_err(|error| Pod5ToolsError::new(format!("failed to serialize JSON: {error}")))?,
    };
    if let Some(output) = output {
        fs::write(output, rendered).map_err(|error| {
            Pod5ToolsError::new(format!("failed to write {}: {error}", output.display()))
        })?;
        Ok(format!("wrote manifest to {}", output.display()))
    } else {
        Ok(rendered)
    }
}

fn run_compare(left: &Path, right: &Path, format: OutputFormat) -> Result<String, Pod5ToolsError> {
    let report = compare_inputs(left, right)?;
    match format {
        OutputFormat::Tsv => Ok(format_compare_report_tsv(&report)),
        OutputFormat::Json => serde_json::to_string_pretty(&report)
            .map_err(|error| Pod5ToolsError::new(format!("failed to serialize JSON: {error}"))),
    }
}

fn run_subdivide_plan(
    path: &Path,
    strategy: SubdivideStrategy,
    files_per_chunk: u64,
    seconds_per_chunk: Option<u64>,
    reads_per_chunk: Option<u64>,
    output: Option<&Path>,
    format: OutputFormat,
) -> Result<String, Pod5ToolsError> {
    let plan = subdivide_plan_from_path(
        path,
        strategy,
        files_per_chunk,
        seconds_per_chunk,
        reads_per_chunk,
    )?;
    let rendered = match format {
        OutputFormat::Tsv => format_subdivide_plan_tsv(&plan),
        OutputFormat::Json => serde_json::to_string_pretty(&plan)
            .map_err(|error| Pod5ToolsError::new(format!("failed to serialize JSON: {error}")))?,
    };
    if let Some(output) = output {
        fs::write(output, rendered).map_err(|error| {
            Pod5ToolsError::new(format!("failed to write {}: {error}", output.display()))
        })?;
        Ok(format!("wrote subdivide plan to {}", output.display()))
    } else {
        Ok(rendered)
    }
}

fn run_playback_plan(
    manifest_paths: &[PathBuf],
    samples: &[String],
    inputs: &[PathBuf],
    output: Option<&Path>,
    format: OutputFormat,
) -> Result<String, Pod5ToolsError> {
    let report = playback_plan_report(manifest_paths, samples, inputs)?;
    let rendered = match format {
        OutputFormat::Tsv => format_playback_plan_tsv(&report),
        OutputFormat::Json => serde_json::to_string_pretty(&report)
            .map_err(|error| Pod5ToolsError::new(format!("failed to serialize JSON: {error}")))?,
    };
    if let Some(output) = output {
        fs::write(output, rendered).map_err(|error| {
            Pod5ToolsError::new(format!("failed to write {}: {error}", output.display()))
        })?;
        Ok(format!("wrote playback plan to {}", output.display()))
    } else {
        Ok(rendered)
    }
}

fn run_playback_emit(
    manifest_paths: &[PathBuf],
    samples: &[String],
    inputs: &[PathBuf],
    speedup: &str,
    format: OutputFormat,
) -> Result<String, Pod5ToolsError> {
    let plans = playback_sample_plans(manifest_paths, samples, inputs)?;
    let report = playback_emit_report(&plans, parse_playback_speedup(speedup)?)?;
    match format {
        OutputFormat::Tsv => Ok(format_playback_emit_tsv(&report)),
        OutputFormat::Json => serde_json::to_string_pretty(&report)
            .map_err(|error| Pod5ToolsError::new(format!("failed to serialize JSON: {error}"))),
    }
}

const POD5_SIGNATURE: [u8; 8] = [0x8B, b'P', b'O', b'D', 0x0D, 0x0A, 0x1A, 0x0A];

/// Verify that a candidate file matches implemented POD5 format checks.
pub fn verify_pod5_file(path: &Path) -> Result<Pod5VerifyReport, Pod5ToolsError> {
    let metadata = fs::metadata(path).map_err(|error| {
        Pod5ToolsError::new(format!("failed to inspect {}: {error}", path.display()))
    })?;
    if !metadata.is_file() {
        return Err(Pod5ToolsError::new(format!(
            "verify expects a file: {}",
            path.display()
        )));
    }

    let mut checks = Vec::new();
    checks.push(verify_check(
        "extension",
        "extension",
        if is_pod5_path(path) {
            VerifyCheckStatus::Passed
        } else {
            VerifyCheckStatus::Failed
        },
        if is_pod5_path(path) {
            "file extension is .pod5"
        } else {
            "file extension is not .pod5"
        },
    ));

    if metadata.len() < (POD5_SIGNATURE.len() * 2) as u64 {
        checks.push(verify_check(
            "leading_signature",
            "signature",
            VerifyCheckStatus::Failed,
            "file is too small to contain a leading POD5 signature",
        ));
        checks.push(verify_check(
            "trailing_signature",
            "signature",
            VerifyCheckStatus::Failed,
            "file is too small to contain a trailing POD5 signature",
        ));
    } else {
        let mut file = fs::File::open(path).map_err(|error| {
            Pod5ToolsError::new(format!("failed to open {}: {error}", path.display()))
        })?;
        let mut leading = [0_u8; 8];
        file.read_exact(&mut leading).map_err(|error| {
            Pod5ToolsError::new(format!(
                "failed to read leading signature from {}: {error}",
                path.display()
            ))
        })?;
        checks.push(verify_check(
            "leading_signature",
            "signature",
            if leading == POD5_SIGNATURE {
                VerifyCheckStatus::Passed
            } else {
                VerifyCheckStatus::Failed
            },
            if leading == POD5_SIGNATURE {
                "leading signature matches ONT POD5 signature"
            } else {
                "leading signature does not match ONT POD5 signature"
            },
        ));

        file.seek(SeekFrom::End(-(POD5_SIGNATURE.len() as i64)))
            .map_err(|error| {
                Pod5ToolsError::new(format!(
                    "failed to seek trailing signature in {}: {error}",
                    path.display()
                ))
            })?;
        let mut trailing = [0_u8; 8];
        file.read_exact(&mut trailing).map_err(|error| {
            Pod5ToolsError::new(format!(
                "failed to read trailing signature from {}: {error}",
                path.display()
            ))
        })?;
        checks.push(verify_check(
            "trailing_signature",
            "signature",
            if trailing == POD5_SIGNATURE {
                VerifyCheckStatus::Passed
            } else {
                VerifyCheckStatus::Failed
            },
            if trailing == POD5_SIGNATURE {
                "trailing signature matches ONT POD5 signature"
            } else {
                "trailing signature does not match ONT POD5 signature"
            },
        ));
    }

    checks.extend(deferred_pod5_specification_checks());
    let status = verify_status(&checks);
    Ok(Pod5VerifyReport {
        path: path.to_path_buf(),
        size_bytes: metadata.len(),
        status,
        checks,
    })
}

fn verify_check(
    name: &str,
    category: &str,
    status: VerifyCheckStatus,
    detail: &str,
) -> VerifyCheck {
    VerifyCheck {
        name: name.to_string(),
        category: category.to_string(),
        status,
        detail: detail.to_string(),
    }
}

fn deferred_pod5_specification_checks() -> Vec<VerifyCheck> {
    vec![
        verify_check(
            "combined_file_layout",
            "layout",
            VerifyCheckStatus::NotChecked,
            "combined-file layout, section markers, footer magic, footer length, and padding checks require the POD5 parser backend",
        ),
        verify_check(
            "required_tables",
            "schema",
            VerifyCheckStatus::NotChecked,
            "Reads, Signal, and Run Info table presence checks require the POD5 parser backend",
        ),
        verify_check(
            "schema_metadata",
            "schema",
            VerifyCheckStatus::NotChecked,
            "POD5 version, writer software, and file identifier consistency checks require the POD5 parser backend",
        ),
    ]
}

fn verify_status(checks: &[VerifyCheck]) -> VerifyStatus {
    if checks
        .iter()
        .any(|check| check.status == VerifyCheckStatus::Failed)
    {
        VerifyStatus::Failed
    } else if checks
        .iter()
        .any(|check| check.status == VerifyCheckStatus::NotChecked)
    {
        VerifyStatus::Incomplete
    } else {
        VerifyStatus::Passed
    }
}

/// Aggregate file-level metadata and fast verification state for a folder.
pub fn folder_info(
    root: &Path,
    reader: &impl Pod5MetadataReader,
) -> Result<Pod5FolderInfo, Pod5ToolsError> {
    let metadata = fs::metadata(root).map_err(|error| {
        Pod5ToolsError::new(format!("failed to inspect {}: {error}", root.display()))
    })?;
    if !metadata.is_dir() {
        return Err(Pod5ToolsError::new(format!(
            "folderinfo expects a directory: {}",
            root.display()
        )));
    }

    let mut files = Vec::new();
    collect_pod5_files(root, &mut files)?;
    files.sort();

    let mut total_bytes = 0_u64;
    let mut total_reads = 0_u64;
    let mut saw_read_count = false;
    let mut flow_cell_ids = BTreeSet::<String>::new();
    let mut sequencing_kits = BTreeSet::<String>::new();
    let mut starts = Vec::<String>::new();
    let mut failed_file_count = 0_u64;
    let mut verification_failed_count = 0_u64;
    let mut names = BTreeMap::<String, u64>::new();

    for file in &files {
        if let Some(name) = file.file_name().and_then(|name| name.to_str()) {
            *names.entry(name.to_string()).or_default() += 1;
        }

        match read_pod5_file_info(reader, file) {
            Ok(info) => {
                total_bytes += info.size_bytes;
                if let Some(read_count) = info.read_count {
                    saw_read_count = true;
                    total_reads += read_count;
                }
                if let Some(flow_cell_id) = info.flow_cell_id {
                    flow_cell_ids.insert(flow_cell_id);
                }
                if let Some(sequencing_kit) = info.sequencing_kit {
                    sequencing_kits.insert(sequencing_kit);
                }
                if let Some(start) = info.acquisition_start_utc {
                    starts.push(start);
                }
            }
            Err(_) => failed_file_count += 1,
        }

        if verify_pod5_file(file).is_ok_and(|report| report.status == VerifyStatus::Failed) {
            verification_failed_count += 1;
        }
    }

    starts.sort();
    let duplicate_file_names = names
        .into_iter()
        .filter_map(|(name, count)| (count > 1).then_some(name))
        .collect::<Vec<_>>();
    let mut warnings = Vec::new();
    if flow_cell_ids.is_empty() {
        warnings
            .push("flow cell metadata unavailable with current POD5 reader backend".to_string());
    }
    if sequencing_kits.is_empty() {
        warnings.push(
            "sequencing kit metadata unavailable with current POD5 reader backend".to_string(),
        );
    }
    if starts.len() < files.len() {
        warnings.push("acquisition timestamps unavailable for one or more files; temporal gap checks are incomplete".to_string());
    }
    if !duplicate_file_names.is_empty() {
        warnings.push("duplicate POD5 file names detected".to_string());
    }
    if verification_failed_count > 0 {
        warnings.push("one or more files failed implemented verification checks".to_string());
    }
    if failed_file_count > 0 {
        warnings.push("one or more files could not be read by the metadata reader".to_string());
    }

    let integrity = if failed_file_count > 0 || verification_failed_count > 0 {
        IntegrityStatus::Failed {
            reason: "one or more files failed metadata or verification checks".to_string(),
        }
    } else if files.is_empty() {
        IntegrityStatus::NotChecked
    } else {
        IntegrityStatus::Unavailable {
            reason: "deep POD5 integrity requires the parser backend".to_string(),
        }
    };

    Ok(Pod5FolderInfo {
        path: root.to_path_buf(),
        pod5_file_count: files.len() as u64,
        total_bytes,
        total_reads: saw_read_count.then_some(total_reads),
        flow_cell_ids: flow_cell_ids.into_iter().collect(),
        sequencing_kits: sequencing_kits.into_iter().collect(),
        acquisition_start_utc: starts.first().cloned(),
        acquisition_end_utc: starts.last().cloned(),
        integrity,
        failed_file_count,
        verification_failed_count,
        duplicate_file_names,
        warnings,
    })
}

fn collect_pod5_files(directory: &Path, files: &mut Vec<PathBuf>) -> Result<(), Pod5ToolsError> {
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
            collect_pod5_files(&path, files)?;
        } else if metadata.is_file() && is_pod5_path(&path) {
            files.push(path);
        }
    }
    Ok(())
}

/// Build a versioned POD5 manifest from one file or a folder tree.
pub fn manifest_from_path(path: &Path) -> Result<Pod5Manifest, Pod5ToolsError> {
    let metadata = fs::metadata(path).map_err(|error| {
        Pod5ToolsError::new(format!("failed to inspect {}: {error}", path.display()))
    })?;
    let mut files = Vec::new();
    if metadata.is_dir() {
        collect_pod5_files(path, &mut files)?;
    } else if metadata.is_file() && is_pod5_path(path) {
        files.push(path.to_path_buf());
    } else if metadata.is_file() {
        return Err(Pod5ToolsError::new(format!(
            "manifest expects a .pod5 file or directory: {}",
            path.display()
        )));
    } else {
        return Err(Pod5ToolsError::new(format!(
            "manifest expects a file or directory: {}",
            path.display()
        )));
    }
    files.sort();

    let mut entries = Vec::new();
    for file in files {
        let report = verify_pod5_file(&file)?;
        let relative_path = manifest_relative_path(path, &file);
        let verification_failed_checks = report
            .checks
            .iter()
            .filter(|check| check.status == VerifyCheckStatus::Failed)
            .count() as u64;
        entries.push(Pod5ManifestEntry {
            relative_path,
            path: file,
            size_bytes: report.size_bytes,
            verification_status: report.status,
            verification_failed_checks,
        });
    }
    entries.sort_by(|left, right| left.relative_path.cmp(&right.relative_path));

    Ok(Pod5Manifest {
        schema_version: MANIFEST_SCHEMA_VERSION,
        source: path.to_path_buf(),
        entries,
    })
}

fn manifest_relative_path(root: &Path, file: &Path) -> PathBuf {
    if root.is_dir() {
        file.strip_prefix(root).unwrap_or(file).to_path_buf()
    } else {
        file.file_name()
            .map(PathBuf::from)
            .unwrap_or_else(|| file.to_path_buf())
    }
}

/// Load a manifest JSON file or build a manifest from a POD5 file/folder.
pub fn manifest_input(path: &Path) -> Result<Pod5Manifest, Pod5ToolsError> {
    if path.is_file()
        && path
            .extension()
            .and_then(|extension| extension.to_str())
            .is_some_and(|extension| extension.eq_ignore_ascii_case("json"))
    {
        let content = fs::read_to_string(path).map_err(|error| {
            Pod5ToolsError::new(format!("failed to read {}: {error}", path.display()))
        })?;
        let manifest: Pod5Manifest = serde_json::from_str(&content).map_err(|error| {
            Pod5ToolsError::new(format!(
                "failed to parse manifest {}: {error}",
                path.display()
            ))
        })?;
        if manifest.schema_version != MANIFEST_SCHEMA_VERSION {
            return Err(Pod5ToolsError::new(format!(
                "unsupported manifest schema version {} in {}",
                manifest.schema_version,
                path.display()
            )));
        }
        Ok(manifest)
    } else {
        manifest_from_path(path)
    }
}

/// Compare two POD5 manifests or inputs that can be converted to manifests.
pub fn compare_inputs(left: &Path, right: &Path) -> Result<Pod5CompareReport, Pod5ToolsError> {
    compare_manifests(&manifest_input(left)?, &manifest_input(right)?)
}

/// Compare two POD5 manifests.
pub fn compare_manifests(
    left: &Pod5Manifest,
    right: &Pod5Manifest,
) -> Result<Pod5CompareReport, Pod5ToolsError> {
    let left_entries = left
        .entries
        .iter()
        .map(|entry| (entry.relative_path.clone(), entry))
        .collect::<BTreeMap<_, _>>();
    let right_entries = right
        .entries
        .iter()
        .map(|entry| (entry.relative_path.clone(), entry))
        .collect::<BTreeMap<_, _>>();

    let missing_from_right = left_entries
        .keys()
        .filter(|path| !right_entries.contains_key(*path))
        .cloned()
        .collect::<Vec<_>>();
    let missing_from_left = right_entries
        .keys()
        .filter(|path| !left_entries.contains_key(*path))
        .cloned()
        .collect::<Vec<_>>();
    let mut changed = Vec::new();
    for (relative_path, left_entry) in &left_entries {
        if let Some(right_entry) = right_entries.get(relative_path)
            && (left_entry.size_bytes != right_entry.size_bytes
                || left_entry.verification_status != right_entry.verification_status)
        {
            changed.push(Pod5CompareChange {
                relative_path: relative_path.clone(),
                left_size_bytes: left_entry.size_bytes,
                right_size_bytes: right_entry.size_bytes,
                left_verification_status: left_entry.verification_status.clone(),
                right_verification_status: right_entry.verification_status.clone(),
            });
        }
    }

    let status =
        if missing_from_right.is_empty() && missing_from_left.is_empty() && changed.is_empty() {
            CompareStatus::Match
        } else {
            CompareStatus::Different
        };
    Ok(Pod5CompareReport {
        status,
        missing_from_right,
        missing_from_left,
        changed,
    })
}

/// Create a read-only subdivision plan from a POD5 file, folder, or manifest.
pub fn subdivide_plan_from_path(
    path: &Path,
    strategy: SubdivideStrategy,
    files_per_chunk: u64,
    seconds_per_chunk: Option<u64>,
    reads_per_chunk: Option<u64>,
) -> Result<Pod5SubdividePlan, Pod5ToolsError> {
    let manifest = manifest_input(path)?;
    subdivide_plan_from_manifest(
        &manifest,
        strategy,
        files_per_chunk,
        seconds_per_chunk,
        reads_per_chunk,
    )
}

/// Create a read-only subdivision plan from an already loaded manifest.
pub fn subdivide_plan_from_manifest(
    manifest: &Pod5Manifest,
    strategy: SubdivideStrategy,
    files_per_chunk: u64,
    seconds_per_chunk: Option<u64>,
    reads_per_chunk: Option<u64>,
) -> Result<Pod5SubdividePlan, Pod5ToolsError> {
    let mut warnings = Vec::new();
    let (target, chunks) = match strategy {
        SubdivideStrategy::FileCount => {
            if files_per_chunk == 0 {
                return Err(Pod5ToolsError::new(
                    "files-per-chunk must be greater than zero",
                ));
            }
            (
                format!("{files_per_chunk} file(s) per chunk"),
                file_count_subdivide_chunks(&manifest.entries, files_per_chunk),
            )
        }
        SubdivideStrategy::SampleLabel => (
            "first relative-path component as sample label".to_string(),
            sample_label_subdivide_chunks(&manifest.entries),
        ),
        SubdivideStrategy::ElapsedTime => {
            let seconds = seconds_per_chunk.unwrap_or(3600);
            warnings.push(
                "elapsed-time planning requires acquisition timestamps from the POD5 reader backend; emitted one placeholder chunk"
                    .to_string(),
            );
            (
                format!("{seconds} second(s) per chunk"),
                placeholder_subdivide_chunk(&manifest.entries, "elapsed-time-unavailable"),
            )
        }
        SubdivideStrategy::ReadCount => {
            let reads = reads_per_chunk.unwrap_or(100_000);
            warnings.push(
                "read-count planning requires read counts from the POD5 reader backend; emitted one placeholder chunk"
                    .to_string(),
            );
            (
                format!("{reads} read(s) per chunk"),
                placeholder_subdivide_chunk(&manifest.entries, "read-count-unavailable"),
            )
        }
    };

    Ok(Pod5SubdividePlan {
        schema_version: SUBDIVIDE_PLAN_SCHEMA_VERSION,
        source: manifest.source.clone(),
        strategy,
        target,
        chunks,
        warnings,
    })
}

fn file_count_subdivide_chunks(
    entries: &[Pod5ManifestEntry],
    files_per_chunk: u64,
) -> Vec<Pod5SubdivideChunk> {
    entries
        .chunks(files_per_chunk as usize)
        .enumerate()
        .map(|(index, chunk_entries)| subdivide_chunk(index as u64 + 1, None, chunk_entries))
        .collect()
}

fn sample_label_subdivide_chunks(entries: &[Pod5ManifestEntry]) -> Vec<Pod5SubdivideChunk> {
    let mut grouped = BTreeMap::<String, Vec<Pod5ManifestEntry>>::new();
    for entry in entries {
        let label = entry
            .relative_path
            .components()
            .next()
            .and_then(|component| component.as_os_str().to_str())
            .unwrap_or("unlabelled")
            .to_string();
        grouped.entry(label).or_default().push(entry.clone());
    }

    grouped
        .into_iter()
        .enumerate()
        .map(|(index, (label, entries))| {
            subdivide_chunk(index as u64 + 1, Some(label.as_str()), &entries)
        })
        .collect()
}

fn placeholder_subdivide_chunk(
    entries: &[Pod5ManifestEntry],
    label: &str,
) -> Vec<Pod5SubdivideChunk> {
    if entries.is_empty() {
        Vec::new()
    } else {
        vec![subdivide_chunk(1, Some(label), entries)]
    }
}

fn subdivide_chunk(
    index: u64,
    label: Option<&str>,
    entries: &[Pod5ManifestEntry],
) -> Pod5SubdivideChunk {
    let relative_paths = entries
        .iter()
        .map(|entry| entry.relative_path.clone())
        .collect::<Vec<_>>();
    let total_bytes = entries.iter().map(|entry| entry.size_bytes).sum();
    Pod5SubdivideChunk {
        index,
        label: label
            .map(ToOwned::to_owned)
            .unwrap_or_else(|| format!("chunk-{index:04}")),
        relative_paths,
        file_count: entries.len() as u64,
        total_bytes,
        read_count: None,
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

/// Format a POD5 verification report as tab-separated text.
pub fn format_verify_report_tsv(report: &Pod5VerifyReport) -> String {
    let mut output =
        String::from("path\tsize_bytes\toverall_status\tcheck\tcategory\tstatus\tdetail");
    for check in &report.checks {
        output.push('\n');
        output.push_str(&format!(
            "{}\t{}\t{}\t{}\t{}\t{}\t{}",
            report.path.display(),
            report.size_bytes,
            verify_status_label(&report.status),
            check.name,
            check.category,
            verify_check_status_label(&check.status),
            check.detail,
        ));
    }
    output
}

fn verify_status_label(status: &VerifyStatus) -> &'static str {
    match status {
        VerifyStatus::Incomplete => "incomplete",
        VerifyStatus::Failed => "failed",
        VerifyStatus::Passed => "passed",
    }
}

fn verify_check_status_label(status: &VerifyCheckStatus) -> &'static str {
    match status {
        VerifyCheckStatus::Passed => "passed",
        VerifyCheckStatus::Failed => "failed",
        VerifyCheckStatus::NotChecked => "not_checked",
    }
}

/// Format folder-level POD5 metadata as tab-separated text.
pub fn format_folder_info_tsv(info: &Pod5FolderInfo) -> String {
    let (integrity_status, integrity_reason) = integrity_tsv_fields(&info.integrity);
    format!(
        "path\tpod5_file_count\ttotal_bytes\ttotal_reads\tflow_cell_ids\tsequencing_kits\tacquisition_start_utc\tacquisition_end_utc\tintegrity_status\tintegrity_reason\tfailed_file_count\tverification_failed_count\tduplicate_file_names\twarnings\n{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}",
        info.path.display(),
        info.pod5_file_count,
        info.total_bytes,
        info.total_reads
            .map(|value| value.to_string())
            .unwrap_or_default(),
        info.flow_cell_ids.join(","),
        info.sequencing_kits.join(","),
        info.acquisition_start_utc.as_deref().unwrap_or(""),
        info.acquisition_end_utc.as_deref().unwrap_or(""),
        integrity_status,
        integrity_reason,
        info.failed_file_count,
        info.verification_failed_count,
        info.duplicate_file_names.join(","),
        info.warnings.join("; "),
    )
}

/// Format a POD5 manifest as tab-separated text.
pub fn format_manifest_tsv(manifest: &Pod5Manifest) -> String {
    let mut output = String::from(
        "schema_version\tsource\trelative_path\tpath\tsize_bytes\tverification_status\tverification_failed_checks",
    );
    for entry in &manifest.entries {
        output.push('\n');
        output.push_str(&format!(
            "{}\t{}\t{}\t{}\t{}\t{}\t{}",
            manifest.schema_version,
            manifest.source.display(),
            entry.relative_path.display(),
            entry.path.display(),
            entry.size_bytes,
            verify_status_label(&entry.verification_status),
            entry.verification_failed_checks,
        ));
    }
    output
}

/// Format a POD5 comparison report as tab-separated text.
pub fn format_compare_report_tsv(report: &Pod5CompareReport) -> String {
    let mut output = String::from(
        "status\tkind\trelative_path\tleft_size_bytes\tright_size_bytes\tleft_verification_status\tright_verification_status",
    );
    for path in &report.missing_from_right {
        output.push('\n');
        output.push_str(&format!(
            "{}\tmissing_from_right\t{}\t\t\t\t",
            compare_status_label(&report.status),
            path.display(),
        ));
    }
    for path in &report.missing_from_left {
        output.push('\n');
        output.push_str(&format!(
            "{}\tmissing_from_left\t{}\t\t\t\t",
            compare_status_label(&report.status),
            path.display(),
        ));
    }
    for change in &report.changed {
        output.push('\n');
        output.push_str(&format!(
            "{}\tchanged\t{}\t{}\t{}\t{}\t{}",
            compare_status_label(&report.status),
            change.relative_path.display(),
            change.left_size_bytes,
            change.right_size_bytes,
            verify_status_label(&change.left_verification_status),
            verify_status_label(&change.right_verification_status),
        ));
    }
    if report.status == CompareStatus::Match {
        output.push('\n');
        output.push_str("match\tmatch\t\t\t\t\t");
    }
    output
}

fn compare_status_label(status: &CompareStatus) -> &'static str {
    match status {
        CompareStatus::Match => "match",
        CompareStatus::Different => "different",
    }
}

/// Format a POD5 subdivision plan as tab-separated text.
pub fn format_subdivide_plan_tsv(plan: &Pod5SubdividePlan) -> String {
    let mut output = String::from(
        "schema_version\tsource\tstrategy\ttarget\tchunk_index\tchunk_label\tfile_count\ttotal_bytes\tread_count\trelative_paths\twarnings",
    );
    let warnings = plan.warnings.join("; ");
    if plan.chunks.is_empty() {
        output.push('\n');
        output.push_str(&format!(
            "{}\t{}\t{}\t{}\t\t\t0\t0\t\t\t{}",
            plan.schema_version,
            plan.source.display(),
            subdivide_strategy_label(&plan.strategy),
            plan.target,
            warnings,
        ));
    }
    for chunk in &plan.chunks {
        output.push('\n');
        output.push_str(&format!(
            "{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}",
            plan.schema_version,
            plan.source.display(),
            subdivide_strategy_label(&plan.strategy),
            plan.target,
            chunk.index,
            chunk.label,
            chunk.file_count,
            chunk.total_bytes,
            chunk
                .read_count
                .map(|value| value.to_string())
                .unwrap_or_default(),
            chunk
                .relative_paths
                .iter()
                .map(|path| path.display().to_string())
                .collect::<Vec<_>>()
                .join(","),
            warnings,
        ));
    }
    output
}

fn subdivide_strategy_label(strategy: &SubdivideStrategy) -> &'static str {
    match strategy {
        SubdivideStrategy::FileCount => "file-count",
        SubdivideStrategy::ElapsedTime => "elapsed-time",
        SubdivideStrategy::ReadCount => "read-count",
        SubdivideStrategy::SampleLabel => "sample-label",
    }
}

/// Format a playback plan report as tab-separated text.
pub fn format_playback_plan_tsv(report: &PlaybackPlanReport) -> String {
    let mut output = String::from(
        "sample\tinput\tmanifest_path\tschedule_seconds\tbatch_index\tbatch_path\tbucket_start_seconds\temit_seconds\tread_count\tsource_pod5_count\tmin_elapsed_seconds\tmax_elapsed_seconds",
    );
    let schedule_seconds = report
        .schedule_seconds
        .iter()
        .map(|seconds| format_float(*seconds))
        .collect::<Vec<_>>()
        .join(",");
    for plan in &report.sample_plans {
        if plan.manifest.batches.is_empty() {
            output.push('\n');
            output.push_str(&format!(
                "{}\t{}\t{}\t{}\t\t\t\t\t0\t{}\t\t",
                plan.sample,
                plan.input.display(),
                plan.manifest_path.display(),
                schedule_seconds,
                plan.manifest.source_pod5_count,
            ));
        }
        for batch in &plan.manifest.batches {
            output.push('\n');
            output.push_str(&format!(
                "{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}",
                plan.sample,
                plan.input.display(),
                plan.manifest_path.display(),
                schedule_seconds,
                batch.batch_index,
                batch.path.display(),
                format_float(batch.bucket_start_seconds),
                format_float(playback_batch_emit_seconds(&plan.manifest, batch)),
                batch.read_count,
                batch.source_pod5_count,
                batch
                    .min_elapsed_seconds
                    .map(format_float)
                    .unwrap_or_default(),
                batch
                    .max_elapsed_seconds
                    .map(format_float)
                    .unwrap_or_default(),
            ));
        }
    }
    output
}

/// Format a dry-run playback emission report as tab-separated text.
pub fn format_playback_emit_tsv(report: &PlaybackEmitReport) -> String {
    let mut output = String::from(
        "speedup\tround_index\tsample\tbatch_index\tbatch_path\temit_seconds\tsequence_wait_seconds\twall_wait_seconds\tread_count",
    );
    for event in &report.events {
        output.push('\n');
        output.push_str(&format!(
            "{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}",
            report.speedup_label,
            event.round_index,
            event.sample,
            event.batch_index,
            event.batch_path.display(),
            format_float(event.emit_seconds),
            format_float(event.sequence_wait_seconds),
            format_float(event.wall_wait_seconds),
            event.read_count,
        ));
    }
    output
}

fn format_float(value: f64) -> String {
    if (value.fract()).abs() < f64::EPSILON {
        format!("{value:.0}")
    } else {
        format!("{value:.6}")
            .trim_end_matches('0')
            .trim_end_matches('.')
            .to_string()
    }
}

/// Load a playback manifest JSON file.
pub fn load_playback_manifest(path: &Path) -> Result<PlaybackManifest, Pod5ToolsError> {
    let content = fs::read_to_string(path).map_err(|error| {
        Pod5ToolsError::new(format!("failed to read {}: {error}", path.display()))
    })?;
    serde_json::from_str(&content).map_err(|error| {
        Pod5ToolsError::new(format!(
            "failed to parse playback manifest {}: {error}",
            path.display()
        ))
    })
}

/// Build a playback plan report from playback manifest JSON files.
pub fn playback_plan_report(
    manifest_paths: &[PathBuf],
    samples: &[String],
    inputs: &[PathBuf],
) -> Result<PlaybackPlanReport, Pod5ToolsError> {
    let sample_plans = playback_sample_plans(manifest_paths, samples, inputs)?;
    let schedule_seconds = playback_schedule(&sample_plans);
    Ok(PlaybackPlanReport {
        sample_plans,
        schedule_seconds,
    })
}

/// Load playback sample plans from manifest paths and user-provided labels.
pub fn playback_sample_plans(
    manifest_paths: &[PathBuf],
    samples: &[String],
    inputs: &[PathBuf],
) -> Result<Vec<PlaybackSamplePlan>, Pod5ToolsError> {
    if manifest_paths.is_empty() {
        return Err(Pod5ToolsError::new(
            "playback requires at least one --manifest",
        ));
    }
    if manifest_paths.len() != samples.len() {
        return Err(Pod5ToolsError::new(format!(
            "playback requires the same number of --manifest and --sample values; received {} manifest(s) and {} sample(s)",
            manifest_paths.len(),
            samples.len()
        )));
    }
    if !inputs.is_empty() && inputs.len() != manifest_paths.len() {
        return Err(Pod5ToolsError::new(format!(
            "playback requires either zero --input values or one per --manifest; received {} manifest(s) and {} input(s)",
            manifest_paths.len(),
            inputs.len()
        )));
    }

    let mut seen = BTreeSet::new();
    manifest_paths
        .iter()
        .enumerate()
        .map(|(index, manifest_path)| {
            let sample = samples[index].trim();
            if sample.is_empty() {
                return Err(Pod5ToolsError::new("sample label is required"));
            }
            if !seen.insert(sample.to_string()) {
                return Err(Pod5ToolsError::new(format!(
                    "sample label '{sample}' was provided more than once"
                )));
            }
            let input = inputs
                .get(index)
                .cloned()
                .unwrap_or_else(|| inferred_playback_input_path(manifest_path));
            Ok(PlaybackSamplePlan {
                sample: sample.to_string(),
                input,
                manifest_path: manifest_path.clone(),
                manifest: load_playback_manifest(manifest_path)?,
            })
        })
        .collect()
}

fn inferred_playback_input_path(manifest_path: &Path) -> PathBuf {
    manifest_path
        .parent()
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("."))
}

/// Build a dry-run playback emission report from sample plans.
pub fn playback_emit_report(
    plans: &[PlaybackSamplePlan],
    speedup: f64,
) -> Result<PlaybackEmitReport, Pod5ToolsError> {
    if !speedup.is_finite() || speedup <= 0.0 {
        return Err(Pod5ToolsError::new(
            "playback speedup must be finite and greater than zero",
        ));
    }
    let schedule = playback_schedule(plans);
    let mut previous_bucket = Some(0.0);
    let mut events = Vec::new();
    for (round_index, bucket) in schedule.iter().enumerate() {
        let sequence_wait_seconds = playback_sequence_wait_seconds(previous_bucket, *bucket);
        let wall_wait_seconds = playback_wall_wait_seconds(sequence_wait_seconds, speedup);
        previous_bucket = Some(*bucket);
        for plan in plans {
            for batch in &plan.manifest.batches {
                let emit_seconds = playback_batch_emit_seconds(&plan.manifest, batch);
                if playback_bucket_key(emit_seconds) == playback_bucket_key(*bucket) {
                    events.push(PlaybackEmitEvent {
                        round_index: round_index as u64 + 1,
                        sample: plan.sample.clone(),
                        batch_path: batch.path.clone(),
                        batch_index: batch.batch_index,
                        emit_seconds,
                        sequence_wait_seconds,
                        wall_wait_seconds,
                        read_count: batch.read_count,
                    });
                }
            }
        }
    }
    Ok(PlaybackEmitReport {
        speedup,
        speedup_label: playback_speedup_label(speedup),
        events,
    })
}

/// Parse a playback duration expressed in seconds or minutes.
pub fn parse_playback_duration_seconds(value: &str) -> Result<f64, Pod5ToolsError> {
    let value = value.trim();
    if value.len() < 2 {
        return Err(Pod5ToolsError::new(format!(
            "invalid playback duration '{value}'; use values such as 300s or 5m"
        )));
    }
    let (number, unit) = value.split_at(value.len() - 1);
    let amount = number.parse::<f64>().map_err(|_| {
        Pod5ToolsError::new(format!(
            "invalid playback duration '{value}'; use values such as 300s or 5m"
        ))
    })?;
    if !amount.is_finite() || amount <= 0.0 {
        return Err(Pod5ToolsError::new(format!(
            "invalid playback duration '{value}'; duration must be greater than zero"
        )));
    }
    match unit {
        "s" | "S" => Ok(amount),
        "m" | "M" => Ok(amount * 60.0),
        _ => Err(Pod5ToolsError::new(format!(
            "invalid playback duration '{value}'; only seconds (s) and minutes (m) are supported"
        ))),
    }
}

/// Parse a playback cutoff value such as `900s`, `15m`, `all`, or `full`.
pub fn parse_playback_cutoff(value: &str) -> Result<PlaybackCutoff, Pod5ToolsError> {
    let trimmed = value.trim();
    if trimmed.eq_ignore_ascii_case("all")
        || trimmed.eq_ignore_ascii_case("full")
        || trimmed.eq_ignore_ascii_case("none")
    {
        return Ok(PlaybackCutoff::All);
    }
    Ok(PlaybackCutoff::ElapsedSeconds(
        parse_playback_duration_seconds(trimmed)?,
    ))
}

/// Parse a wall-clock playback speedup value such as `1`, `2x`, or `5X`.
pub fn parse_playback_speedup(value: &str) -> Result<f64, Pod5ToolsError> {
    let trimmed = value.trim();
    let number = trimmed
        .strip_suffix('x')
        .or_else(|| trimmed.strip_suffix('X'))
        .unwrap_or(trimmed)
        .trim();
    if number.is_empty() {
        return Err(Pod5ToolsError::new(format!(
            "invalid playback speedup '{value}'; use values such as 1x, 2x, or 5x"
        )));
    }
    let speedup = number.parse::<f64>().map_err(|_| {
        Pod5ToolsError::new(format!(
            "invalid playback speedup '{value}'; use values such as 1x, 2x, or 5x"
        ))
    })?;
    if !speedup.is_finite() || speedup <= 0.0 {
        return Err(Pod5ToolsError::new(format!(
            "invalid playback speedup '{value}'; speedup must be greater than zero"
        )));
    }
    Ok(speedup)
}

/// Convert source sequencing wait seconds into wall-clock wait seconds.
pub fn playback_wall_wait_seconds(sequence_wait_seconds: f64, speedup: f64) -> f64 {
    if !speedup.is_finite() || speedup <= 0.0 {
        return 0.0;
    }
    (sequence_wait_seconds.max(0.0) / speedup).max(0.0)
}

/// Calculate sequencing-time wait from the previous emitted bucket.
pub fn playback_sequence_wait_seconds(previous_bucket: Option<f64>, bucket: f64) -> f64 {
    (bucket - previous_bucket.unwrap_or(0.0)).max(0.0)
}

/// Human-readable playback speedup label.
pub fn playback_speedup_label(speedup: f64) -> String {
    if (speedup.fract()).abs() < f64::EPSILON {
        format!("{speedup:.0}x")
    } else {
        format!("{speedup:.2}x")
    }
}

/// Return the source sequencing second at which a batch should be emitted.
pub fn playback_batch_emit_seconds(manifest: &PlaybackManifest, batch: &PlaybackBatch) -> f64 {
    (batch.bucket_start_seconds + manifest.tempo_seconds).max(0.0)
}

/// Stable integer key for comparing playback bucket seconds.
pub fn playback_bucket_key(bucket_start_seconds: f64) -> i64 {
    (bucket_start_seconds * 1_000_000.0).round() as i64
}

/// Merge playback bucket schedules across sample plans.
pub fn playback_schedule(plans: &[PlaybackSamplePlan]) -> Vec<f64> {
    let mut buckets = plans
        .iter()
        .flat_map(|plan| {
            plan.manifest
                .batches
                .iter()
                .map(|batch| playback_batch_emit_seconds(&plan.manifest, batch))
        })
        .collect::<Vec<_>>();
    buckets.sort_by(|left, right| left.partial_cmp(right).unwrap_or(std::cmp::Ordering::Equal));
    buckets.dedup_by(|left, right| playback_bucket_key(*left) == playback_bucket_key(*right));
    buckets
}

/// Make a filesystem-safe label for playback output components.
pub fn safe_filename_component(value: &str) -> String {
    let safe = value
        .chars()
        .map(|character| {
            if character.is_ascii_alphanumeric() || matches!(character, '-' | '_') {
                character
            } else {
                '_'
            }
        })
        .collect::<String>();
    if safe.is_empty() {
        "playback".to_string()
    } else {
        safe
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
    fn cli_parses_verify_with_json_output() {
        let cli = Cli::try_parse_from([
            "pod5-tools",
            "verify",
            "/data/reads.pod5",
            "--format",
            "json",
        ])
        .unwrap();
        let Command::Verify { path, format } = cli.command else {
            panic!("expected verify command");
        };
        assert_eq!(path, PathBuf::from("/data/reads.pod5"));
        assert_eq!(format, OutputFormat::Json);
    }

    #[test]
    fn cli_parses_playback_plan_options() {
        let cli = Cli::try_parse_from([
            "pod5-tools",
            "playback",
            "plan",
            "--manifest",
            "/tmp/playback_manifest.json",
            "--sample",
            "sample-a",
            "--format",
            "json",
        ])
        .unwrap();
        let Command::Playback { command } = cli.command else {
            panic!("expected playback command");
        };
        let PlaybackCommand::Plan {
            manifest,
            sample,
            format,
            ..
        } = command
        else {
            panic!("expected playback plan command");
        };
        assert_eq!(manifest, vec![PathBuf::from("/tmp/playback_manifest.json")]);
        assert_eq!(sample, vec!["sample-a".to_string()]);
        assert_eq!(format, OutputFormat::Json);
    }

    #[test]
    fn cli_parses_playback_emit_speedup() {
        let cli = Cli::try_parse_from([
            "pod5-tools",
            "playback",
            "emit",
            "--manifest",
            "/tmp/playback_manifest.json",
            "--sample",
            "sample-a",
            "--input",
            "/data/source",
            "--speedup",
            "5x",
        ])
        .unwrap();
        let Command::Playback { command } = cli.command else {
            panic!("expected playback command");
        };
        let PlaybackCommand::Emit {
            manifest,
            sample,
            input,
            speedup,
            format,
        } = command
        else {
            panic!("expected playback emit command");
        };
        assert_eq!(manifest, vec![PathBuf::from("/tmp/playback_manifest.json")]);
        assert_eq!(sample, vec!["sample-a".to_string()]);
        assert_eq!(input, vec![PathBuf::from("/data/source")]);
        assert_eq!(speedup, "5x");
        assert_eq!(format, OutputFormat::Tsv);
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
    fn cli_parses_subdivide_plan_options() {
        let cli = Cli::try_parse_from([
            "pod5-tools",
            "subdivide",
            "plan",
            "/data",
            "--strategy",
            "sample-label",
            "--format",
            "json",
        ])
        .unwrap();
        let Command::Subdivide { command } = cli.command else {
            panic!("expected subdivide command");
        };
        let SubdivideCommand::Plan {
            path,
            strategy,
            format,
            ..
        } = command;
        assert_eq!(path, PathBuf::from("/data"));
        assert_eq!(strategy, SubdivideStrategy::SampleLabel);
        assert_eq!(format, OutputFormat::Json);
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

    fn write_signature_fixture(path: &Path) {
        let mut bytes = Vec::new();
        bytes.extend_from_slice(&POD5_SIGNATURE);
        bytes.extend_from_slice(&[0_u8; 16]);
        bytes.extend_from_slice(&POD5_SIGNATURE);
        fs::write(path, bytes).unwrap();
    }

    #[test]
    fn verify_reports_incomplete_for_signature_valid_candidate() {
        let root = tempfile::tempdir().unwrap();
        let pod5 = root.path().join("reads.pod5");
        write_signature_fixture(&pod5);

        let report = verify_pod5_file(&pod5).unwrap();

        assert_eq!(report.status, VerifyStatus::Incomplete);
        assert!(report.checks.iter().any(|check| {
            check.name == "leading_signature" && check.status == VerifyCheckStatus::Passed
        }));
        assert!(report.checks.iter().any(|check| {
            check.name == "trailing_signature" && check.status == VerifyCheckStatus::Passed
        }));
        assert!(
            report
                .checks
                .iter()
                .any(|check| check.status == VerifyCheckStatus::NotChecked)
        );
    }

    #[test]
    fn verify_reports_extension_failure() {
        let root = tempfile::tempdir().unwrap();
        let path = root.path().join("reads.bin");
        write_signature_fixture(&path);

        let report = verify_pod5_file(&path).unwrap();

        assert_eq!(report.status, VerifyStatus::Failed);
        assert!(report.checks.iter().any(|check| {
            check.name == "extension" && check.status == VerifyCheckStatus::Failed
        }));
    }

    #[test]
    fn verify_reports_truncated_file() {
        let root = tempfile::tempdir().unwrap();
        let pod5 = root.path().join("reads.pod5");
        fs::write(&pod5, &POD5_SIGNATURE[..4]).unwrap();

        let report = verify_pod5_file(&pod5).unwrap();

        assert_eq!(report.status, VerifyStatus::Failed);
        assert!(report.checks.iter().any(|check| {
            check.name == "leading_signature" && check.status == VerifyCheckStatus::Failed
        }));
        assert!(report.checks.iter().any(|check| {
            check.name == "trailing_signature" && check.status == VerifyCheckStatus::Failed
        }));
    }

    #[test]
    fn verify_reports_signature_failure() {
        let root = tempfile::tempdir().unwrap();
        let pod5 = root.path().join("reads.pod5");
        fs::write(&pod5, b"not-pod5-but-long-enough").unwrap();

        let report = verify_pod5_file(&pod5).unwrap();

        assert_eq!(report.status, VerifyStatus::Failed);
        assert!(report.checks.iter().any(|check| {
            check.name == "leading_signature" && check.status == VerifyCheckStatus::Failed
        }));
    }

    #[test]
    fn run_verify_emits_tsv_by_default() {
        let root = tempfile::tempdir().unwrap();
        let pod5 = root.path().join("reads.pod5");
        write_signature_fixture(&pod5);

        let cli = Cli::try_parse_from(["pod5-tools", "verify", pod5.to_str().unwrap()]).unwrap();
        let output = run(cli).unwrap();

        assert!(output.starts_with("path\tsize_bytes\toverall_status"));
        assert!(output.contains("\tincomplete\tleading_signature\tsignature\tpassed\t"));
        assert!(output.contains("\tnot_checked\t"));
    }

    #[test]
    fn run_verify_emits_json_when_requested() {
        let root = tempfile::tempdir().unwrap();
        let pod5 = root.path().join("reads.pod5");
        write_signature_fixture(&pod5);

        let cli = Cli::try_parse_from([
            "pod5-tools",
            "verify",
            pod5.to_str().unwrap(),
            "--format",
            "json",
        ])
        .unwrap();
        let output = run(cli).unwrap();

        assert!(output.contains("\"status\": \"incomplete\""));
        assert!(output.contains("\"name\": \"leading_signature\""));
        assert!(output.contains("\"status\": \"not_checked\""));
    }

    #[test]
    fn folderinfo_aggregates_pod5_files_recursively() {
        let root = tempfile::tempdir().unwrap();
        let nested = root.path().join("nested");
        fs::create_dir(&nested).unwrap();
        write_signature_fixture(&root.path().join("reads-a.pod5"));
        write_signature_fixture(&nested.join("reads-b.pod5"));
        fs::write(root.path().join("notes.txt"), b"ignore").unwrap();

        let info = folder_info(root.path(), &FilesystemPod5MetadataReader).unwrap();

        assert_eq!(info.pod5_file_count, 2);
        assert_eq!(info.total_bytes, 64);
        assert_eq!(info.verification_failed_count, 0);
        assert_eq!(info.failed_file_count, 0);
        assert!(matches!(
            info.integrity,
            IntegrityStatus::Unavailable { .. }
        ));
        assert!(
            info.warnings
                .iter()
                .any(|warning| warning.contains("flow cell metadata unavailable"))
        );
    }

    #[test]
    fn folderinfo_reports_duplicate_names_and_verification_failures() {
        let root = tempfile::tempdir().unwrap();
        let sample_a = root.path().join("sample-a");
        let sample_b = root.path().join("sample-b");
        fs::create_dir(&sample_a).unwrap();
        fs::create_dir(&sample_b).unwrap();
        write_signature_fixture(&sample_a.join("reads.pod5"));
        fs::write(sample_b.join("reads.pod5"), b"not-pod5-but-long-enough").unwrap();

        let info = folder_info(root.path(), &FilesystemPod5MetadataReader).unwrap();

        assert_eq!(info.pod5_file_count, 2);
        assert_eq!(info.verification_failed_count, 1);
        assert_eq!(info.duplicate_file_names, vec!["reads.pod5".to_string()]);
        assert!(matches!(info.integrity, IntegrityStatus::Failed { .. }));
        assert!(
            info.warnings
                .iter()
                .any(|warning| warning.contains("duplicate POD5 file names"))
        );
    }

    #[test]
    fn run_folderinfo_emits_tsv_by_default() {
        let root = tempfile::tempdir().unwrap();
        write_signature_fixture(&root.path().join("reads.pod5"));

        let cli = Cli::try_parse_from(["pod5-tools", "folderinfo", root.path().to_str().unwrap()])
            .unwrap();
        let output = run(cli).unwrap();

        assert!(output.starts_with("path\tpod5_file_count\ttotal_bytes"));
        assert!(output.contains("\t1\t32\t"));
        assert!(output.contains("deep POD5 integrity requires the parser backend"));
    }

    #[test]
    fn run_folderinfo_emits_json_when_requested() {
        let root = tempfile::tempdir().unwrap();
        write_signature_fixture(&root.path().join("reads.pod5"));

        let cli = Cli::try_parse_from([
            "pod5-tools",
            "folderinfo",
            root.path().to_str().unwrap(),
            "--format",
            "json",
        ])
        .unwrap();
        let output = run(cli).unwrap();

        assert!(output.contains("\"pod5_file_count\": 1"));
        assert!(output.contains("\"verification_failed_count\": 0"));
    }

    #[test]
    fn manifest_from_folder_records_relative_paths() {
        let root = tempfile::tempdir().unwrap();
        let nested = root.path().join("nested");
        fs::create_dir(&nested).unwrap();
        write_signature_fixture(&root.path().join("reads-a.pod5"));
        write_signature_fixture(&nested.join("reads-b.pod5"));

        let manifest = manifest_from_path(root.path()).unwrap();

        assert_eq!(manifest.schema_version, MANIFEST_SCHEMA_VERSION);
        assert_eq!(manifest.entries.len(), 2);
        assert_eq!(
            manifest.entries[0].relative_path,
            PathBuf::from("nested/reads-b.pod5")
        );
        assert_eq!(manifest.entries[0].size_bytes, 32);
        assert_eq!(
            manifest.entries[0].verification_status,
            VerifyStatus::Incomplete
        );
    }

    #[test]
    fn run_manifest_emits_tsv_by_default() {
        let root = tempfile::tempdir().unwrap();
        write_signature_fixture(&root.path().join("reads.pod5"));

        let cli =
            Cli::try_parse_from(["pod5-tools", "manifest", root.path().to_str().unwrap()]).unwrap();
        let output = run(cli).unwrap();

        assert!(output.starts_with("schema_version\tsource\trelative_path"));
        assert!(output.contains("\treads.pod5\t"));
        assert!(output.contains("\tincomplete\t0"));
    }

    #[test]
    fn run_manifest_writes_json_output_file() {
        let root = tempfile::tempdir().unwrap();
        write_signature_fixture(&root.path().join("reads.pod5"));
        let output_path = root.path().join("manifest.json");

        let cli = Cli::try_parse_from([
            "pod5-tools",
            "manifest",
            root.path().to_str().unwrap(),
            "--format",
            "json",
            "--output",
            output_path.to_str().unwrap(),
        ])
        .unwrap();
        let message = run(cli).unwrap();
        let loaded = manifest_input(&output_path).unwrap();

        assert!(message.contains("wrote manifest"));
        assert_eq!(loaded.entries.len(), 1);
    }

    #[test]
    fn compare_folders_reports_missing_and_changed_entries() {
        let root = tempfile::tempdir().unwrap();
        let left = root.path().join("left");
        let right = root.path().join("right");
        fs::create_dir(&left).unwrap();
        fs::create_dir(&right).unwrap();
        write_signature_fixture(&left.join("same.pod5"));
        write_signature_fixture(&right.join("same.pod5"));
        write_signature_fixture(&left.join("left-only.pod5"));
        write_signature_fixture(&right.join("right-only.pod5"));
        fs::write(right.join("changed.pod5"), b"not-pod5-but-long-enough").unwrap();
        write_signature_fixture(&left.join("changed.pod5"));

        let report = compare_inputs(&left, &right).unwrap();

        assert_eq!(report.status, CompareStatus::Different);
        assert_eq!(
            report.missing_from_right,
            vec![PathBuf::from("left-only.pod5")]
        );
        assert_eq!(
            report.missing_from_left,
            vec![PathBuf::from("right-only.pod5")]
        );
        assert_eq!(report.changed.len(), 1);
        assert_eq!(
            report.changed[0].relative_path,
            PathBuf::from("changed.pod5")
        );
    }

    #[test]
    fn compare_manifest_files_reports_match() {
        let root = tempfile::tempdir().unwrap();
        write_signature_fixture(&root.path().join("reads.pod5"));
        let manifest = manifest_from_path(root.path()).unwrap();
        let left = root.path().join("left.json");
        let right = root.path().join("right.json");
        let manifest_json = serde_json::to_string_pretty(&manifest).unwrap();
        fs::write(&left, &manifest_json).unwrap();
        fs::write(&right, &manifest_json).unwrap();

        let report = compare_inputs(&left, &right).unwrap();
        let output = format_compare_report_tsv(&report);

        assert_eq!(report.status, CompareStatus::Match);
        assert!(output.contains("match\tmatch"));
    }

    #[test]
    fn run_compare_emits_json_when_requested() {
        let root = tempfile::tempdir().unwrap();
        let left = root.path().join("left");
        let right = root.path().join("right");
        fs::create_dir(&left).unwrap();
        fs::create_dir(&right).unwrap();
        write_signature_fixture(&left.join("reads.pod5"));

        let cli = Cli::try_parse_from([
            "pod5-tools",
            "compare",
            left.to_str().unwrap(),
            right.to_str().unwrap(),
            "--format",
            "json",
        ])
        .unwrap();
        let output = run(cli).unwrap();

        assert!(output.contains("\"status\": \"different\""));
        assert!(output.contains("missing_from_right"));
    }

    #[test]
    fn subdivide_file_count_plan_groups_manifest_entries() {
        let manifest = Pod5Manifest {
            schema_version: MANIFEST_SCHEMA_VERSION,
            source: PathBuf::from("/data/run"),
            entries: vec![
                Pod5ManifestEntry {
                    relative_path: PathBuf::from("a.pod5"),
                    path: PathBuf::from("/data/run/a.pod5"),
                    size_bytes: 10,
                    verification_status: VerifyStatus::Incomplete,
                    verification_failed_checks: 0,
                },
                Pod5ManifestEntry {
                    relative_path: PathBuf::from("b.pod5"),
                    path: PathBuf::from("/data/run/b.pod5"),
                    size_bytes: 20,
                    verification_status: VerifyStatus::Incomplete,
                    verification_failed_checks: 0,
                },
                Pod5ManifestEntry {
                    relative_path: PathBuf::from("c.pod5"),
                    path: PathBuf::from("/data/run/c.pod5"),
                    size_bytes: 30,
                    verification_status: VerifyStatus::Incomplete,
                    verification_failed_checks: 0,
                },
            ],
        };

        let plan =
            subdivide_plan_from_manifest(&manifest, SubdivideStrategy::FileCount, 2, None, None)
                .unwrap();

        assert_eq!(plan.schema_version, SUBDIVIDE_PLAN_SCHEMA_VERSION);
        assert_eq!(plan.chunks.len(), 2);
        assert_eq!(plan.chunks[0].label, "chunk-0001");
        assert_eq!(plan.chunks[0].file_count, 2);
        assert_eq!(plan.chunks[0].total_bytes, 30);
        assert_eq!(plan.chunks[1].relative_paths, vec![PathBuf::from("c.pod5")]);
    }

    #[test]
    fn subdivide_sample_label_plan_groups_by_first_relative_component() {
        let manifest = Pod5Manifest {
            schema_version: MANIFEST_SCHEMA_VERSION,
            source: PathBuf::from("/data/run"),
            entries: vec![
                Pod5ManifestEntry {
                    relative_path: PathBuf::from("sample-a/reads-1.pod5"),
                    path: PathBuf::from("/data/run/sample-a/reads-1.pod5"),
                    size_bytes: 10,
                    verification_status: VerifyStatus::Incomplete,
                    verification_failed_checks: 0,
                },
                Pod5ManifestEntry {
                    relative_path: PathBuf::from("sample-b/reads-1.pod5"),
                    path: PathBuf::from("/data/run/sample-b/reads-1.pod5"),
                    size_bytes: 20,
                    verification_status: VerifyStatus::Incomplete,
                    verification_failed_checks: 0,
                },
                Pod5ManifestEntry {
                    relative_path: PathBuf::from("sample-a/reads-2.pod5"),
                    path: PathBuf::from("/data/run/sample-a/reads-2.pod5"),
                    size_bytes: 30,
                    verification_status: VerifyStatus::Incomplete,
                    verification_failed_checks: 0,
                },
            ],
        };

        let plan =
            subdivide_plan_from_manifest(&manifest, SubdivideStrategy::SampleLabel, 1, None, None)
                .unwrap();

        assert_eq!(plan.chunks.len(), 2);
        assert_eq!(plan.chunks[0].label, "sample-a");
        assert_eq!(plan.chunks[0].file_count, 2);
        assert_eq!(plan.chunks[0].total_bytes, 40);
        assert_eq!(plan.chunks[1].label, "sample-b");
    }

    #[test]
    fn subdivide_elapsed_time_plan_reports_metadata_gap() {
        let manifest = Pod5Manifest {
            schema_version: MANIFEST_SCHEMA_VERSION,
            source: PathBuf::from("/data/run"),
            entries: vec![Pod5ManifestEntry {
                relative_path: PathBuf::from("reads.pod5"),
                path: PathBuf::from("/data/run/reads.pod5"),
                size_bytes: 10,
                verification_status: VerifyStatus::Incomplete,
                verification_failed_checks: 0,
            }],
        };

        let plan = subdivide_plan_from_manifest(
            &manifest,
            SubdivideStrategy::ElapsedTime,
            1,
            Some(900),
            None,
        )
        .unwrap();

        assert_eq!(plan.target, "900 second(s) per chunk");
        assert_eq!(plan.chunks[0].label, "elapsed-time-unavailable");
        assert!(plan.warnings[0].contains("acquisition timestamps"));
    }

    #[test]
    fn run_subdivide_plan_emits_tsv_by_default() {
        let root = tempfile::tempdir().unwrap();
        write_signature_fixture(&root.path().join("reads-a.pod5"));
        write_signature_fixture(&root.path().join("reads-b.pod5"));

        let cli = Cli::try_parse_from([
            "pod5-tools",
            "subdivide",
            "plan",
            root.path().to_str().unwrap(),
            "--files-per-chunk",
            "2",
        ])
        .unwrap();
        let output = run(cli).unwrap();

        assert!(output.starts_with("schema_version\tsource\tstrategy"));
        assert!(output.contains("\tfile-count\t2 file(s) per chunk\t1\tchunk-0001\t2\t64"));
        assert!(output.contains("reads-a.pod5,reads-b.pod5"));
    }

    #[test]
    fn run_subdivide_plan_writes_json_output_file() {
        let root = tempfile::tempdir().unwrap();
        write_signature_fixture(&root.path().join("reads.pod5"));
        let output_path = root.path().join("plan.json");

        let cli = Cli::try_parse_from([
            "pod5-tools",
            "subdivide",
            "plan",
            root.path().to_str().unwrap(),
            "--format",
            "json",
            "--output",
            output_path.to_str().unwrap(),
        ])
        .unwrap();
        let message = run(cli).unwrap();
        let plan: Pod5SubdividePlan =
            serde_json::from_str(&fs::read_to_string(&output_path).unwrap()).unwrap();

        assert!(message.contains("wrote subdivide plan"));
        assert_eq!(plan.chunks.len(), 1);
        assert_eq!(plan.chunks[0].file_count, 1);
    }

    fn playback_manifest_fixture(path: &Path, batch_path_prefix: &str, buckets: &[f64]) {
        let batches = buckets
            .iter()
            .enumerate()
            .map(|(index, bucket)| {
                format!(
                    r#"{{
                  "path": "{batch_path_prefix}/batch-{batch_index:03}.pod5",
                  "batch_index": {batch_index},
                  "read_count": 1,
                  "source_pod5_count": 1,
                  "source_pod5_files": ["{batch_path_prefix}/source.pod5"],
                  "bucket_start_seconds": {bucket},
                  "min_elapsed_seconds": {bucket},
                  "max_elapsed_seconds": {max_elapsed}
                }}"#,
                    batch_index = index + 1,
                    max_elapsed = bucket + 1.0,
                )
            })
            .collect::<Vec<_>>()
            .join(",");
        fs::write(
            path,
            format!(
                r#"{{
              "read_count": {},
              "source_pod5_count": 1,
              "source_pod5_files": ["{batch_path_prefix}/source.pod5"],
              "tempo_seconds": 90.0,
              "cutoff_seconds": 180.0,
              "master_table": "{batch_path_prefix}/master.tsv",
              "batches": [{batches}]
            }}"#,
                buckets.len()
            ),
        )
        .unwrap();
    }

    #[test]
    fn playback_schedule_merges_temporal_buckets_across_samples() {
        let plans = vec![
            PlaybackSamplePlan {
                sample: "A_IDH_1".to_string(),
                input: PathBuf::from("/pod5/a"),
                manifest_path: PathBuf::from("/pod5/a/playback_manifest.json"),
                manifest: PlaybackManifest {
                    read_count: 2,
                    source_pod5_count: 1,
                    source_pod5_files: vec![PathBuf::from("/pod5/a/source.pod5")],
                    tempo_seconds: 90.0,
                    cutoff_seconds: Some(180.0),
                    master_table: PathBuf::from("/pod5/a/master.tsv"),
                    batches: vec![
                        PlaybackBatch {
                            path: PathBuf::from("/pod5/a/001.pod5"),
                            batch_index: 1,
                            read_count: 1,
                            source_pod5_count: 1,
                            source_pod5_files: vec![PathBuf::from("/pod5/a/source.pod5")],
                            bucket_start_seconds: 0.0,
                            min_elapsed_seconds: Some(0.0),
                            max_elapsed_seconds: Some(1.0),
                        },
                        PlaybackBatch {
                            path: PathBuf::from("/pod5/a/002.pod5"),
                            batch_index: 2,
                            read_count: 1,
                            source_pod5_count: 1,
                            source_pod5_files: vec![PathBuf::from("/pod5/a/source.pod5")],
                            bucket_start_seconds: 180.0,
                            min_elapsed_seconds: Some(180.0),
                            max_elapsed_seconds: Some(181.0),
                        },
                    ],
                },
            },
            PlaybackSamplePlan {
                sample: "B_IDH_1".to_string(),
                input: PathBuf::from("/pod5/b"),
                manifest_path: PathBuf::from("/pod5/b/playback_manifest.json"),
                manifest: PlaybackManifest {
                    read_count: 1,
                    source_pod5_count: 1,
                    source_pod5_files: vec![PathBuf::from("/pod5/b/source.pod5")],
                    tempo_seconds: 90.0,
                    cutoff_seconds: Some(180.0),
                    master_table: PathBuf::from("/pod5/b/master.tsv"),
                    batches: vec![PlaybackBatch {
                        path: PathBuf::from("/pod5/b/001.pod5"),
                        batch_index: 1,
                        read_count: 1,
                        source_pod5_count: 1,
                        source_pod5_files: vec![PathBuf::from("/pod5/b/source.pod5")],
                        bucket_start_seconds: 90.0,
                        min_elapsed_seconds: Some(90.0),
                        max_elapsed_seconds: Some(91.0),
                    }],
                },
            },
        ];

        assert_eq!(playback_schedule(&plans), vec![90.0, 180.0, 270.0]);
    }

    #[test]
    fn playback_speedup_scales_waits_without_changing_sequence_time() {
        assert_eq!(parse_playback_speedup("1").unwrap(), 1.0);
        assert_eq!(parse_playback_speedup("2x").unwrap(), 2.0);
        assert_eq!(parse_playback_speedup("5X").unwrap(), 5.0);
        assert_eq!(playback_wall_wait_seconds(180.0, 5.0), 36.0);
        assert_eq!(playback_wall_wait_seconds(-1.0, 5.0), 0.0);
        assert_eq!(playback_sequence_wait_seconds(Some(0.0), 180.0), 180.0);
        assert_eq!(playback_sequence_wait_seconds(Some(180.0), 270.0), 90.0);
        assert_eq!(playback_sequence_wait_seconds(Some(270.0), 180.0), 0.0);
        assert_eq!(playback_speedup_label(5.0), "5x");
        assert_eq!(playback_speedup_label(2.5), "2.50x");
    }

    #[test]
    fn playback_speedup_rejects_zero_or_invalid_values() {
        assert!(
            parse_playback_speedup("0x")
                .unwrap_err()
                .to_string()
                .contains("greater than zero")
        );
        assert!(
            parse_playback_speedup("fast")
                .unwrap_err()
                .to_string()
                .contains("invalid playback speedup")
        );
    }

    #[test]
    fn playback_duration_parser_accepts_seconds_and_minutes() {
        assert_eq!(parse_playback_duration_seconds("300s").unwrap(), 300.0);
        assert_eq!(parse_playback_duration_seconds("5m").unwrap(), 300.0);
    }

    #[test]
    fn playback_duration_parser_rejects_unsupported_units() {
        let error = parse_playback_duration_seconds("1h").unwrap_err();
        assert!(error.to_string().contains("seconds"));
    }

    #[test]
    fn playback_cutoff_parser_accepts_all_reads() {
        assert_eq!(parse_playback_cutoff("all").unwrap(), PlaybackCutoff::All);
        assert_eq!(parse_playback_cutoff("full").unwrap(), PlaybackCutoff::All);
        assert_eq!(
            parse_playback_cutoff("900s").unwrap(),
            PlaybackCutoff::ElapsedSeconds(900.0)
        );
        assert_eq!(parse_playback_cutoff("none").unwrap(), PlaybackCutoff::All);
    }

    #[test]
    fn playback_manifest_loads_from_json() {
        let root = tempfile::tempdir().unwrap();
        let manifest_path = root.path().join("playback_manifest.json");
        playback_manifest_fixture(&manifest_path, "/playback", &[0.0]);

        let manifest = load_playback_manifest(&manifest_path).unwrap();

        assert_eq!(manifest.read_count, 1);
        assert_eq!(manifest.batches.len(), 1);
        assert_eq!(
            manifest.batches[0].path,
            PathBuf::from("/playback/batch-001.pod5")
        );
    }

    #[test]
    fn playback_plan_report_loads_multiple_manifests() {
        let root = tempfile::tempdir().unwrap();
        let first = root.path().join("first.json");
        let second = root.path().join("second.json");
        playback_manifest_fixture(&first, "/playback/a", &[0.0, 180.0]);
        playback_manifest_fixture(&second, "/playback/b", &[90.0]);

        let report = playback_plan_report(
            &[first.clone(), second.clone()],
            &["sample-a".to_string(), "sample-b".to_string()],
            &[],
        )
        .unwrap();
        let output = format_playback_plan_tsv(&report);

        assert_eq!(report.schedule_seconds, vec![90.0, 180.0, 270.0]);
        assert!(output.starts_with("sample\tinput\tmanifest_path"));
        assert!(output.contains("sample-a"));
        assert!(output.contains("90,180,270"));
    }

    #[test]
    fn run_playback_plan_writes_json_output_file() {
        let root = tempfile::tempdir().unwrap();
        let manifest = root.path().join("playback_manifest.json");
        let output_path = root.path().join("plan.json");
        playback_manifest_fixture(&manifest, "/playback/a", &[0.0]);

        let cli = Cli::try_parse_from([
            "pod5-tools",
            "playback",
            "plan",
            "--manifest",
            manifest.to_str().unwrap(),
            "--sample",
            "sample-a",
            "--format",
            "json",
            "--output",
            output_path.to_str().unwrap(),
        ])
        .unwrap();
        let message = run(cli).unwrap();
        let report: PlaybackPlanReport =
            serde_json::from_str(&fs::read_to_string(&output_path).unwrap()).unwrap();

        assert!(message.contains("wrote playback plan"));
        assert_eq!(report.sample_plans.len(), 1);
        assert_eq!(report.schedule_seconds, vec![90.0]);
    }

    #[test]
    fn playback_emit_report_calculates_wall_clock_waits() {
        let root = tempfile::tempdir().unwrap();
        let manifest = root.path().join("playback_manifest.json");
        playback_manifest_fixture(&manifest, "/playback/a", &[0.0, 180.0]);
        let plans = playback_sample_plans(&[manifest], &["sample-a".to_string()], &[]).unwrap();

        let report = playback_emit_report(&plans, 5.0).unwrap();
        let output = format_playback_emit_tsv(&report);

        assert_eq!(report.speedup_label, "5x");
        assert_eq!(report.events.len(), 2);
        assert_eq!(report.events[0].emit_seconds, 90.0);
        assert_eq!(report.events[0].wall_wait_seconds, 18.0);
        assert_eq!(report.events[1].sequence_wait_seconds, 180.0);
        assert!(output.contains("5x\t1\tsample-a\t1\t/playback/a/batch-001.pod5\t90\t90\t18\t1"));
    }

    #[test]
    fn run_playback_emit_emits_json_when_requested() {
        let root = tempfile::tempdir().unwrap();
        let manifest = root.path().join("playback_manifest.json");
        playback_manifest_fixture(&manifest, "/playback/a", &[0.0]);

        let cli = Cli::try_parse_from([
            "pod5-tools",
            "playback",
            "emit",
            "--manifest",
            manifest.to_str().unwrap(),
            "--sample",
            "sample-a",
            "--speedup",
            "2x",
            "--format",
            "json",
        ])
        .unwrap();
        let output = run(cli).unwrap();

        assert!(output.contains("\"speedup_label\": \"2x\""));
        assert!(output.contains("\"wall_wait_seconds\": 45.0"));
    }

    #[test]
    fn safe_filename_component_replaces_unsafe_characters() {
        assert_eq!(safe_filename_component("Sample A/1"), "Sample_A_1");
        assert_eq!(safe_filename_component(""), "playback");
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
