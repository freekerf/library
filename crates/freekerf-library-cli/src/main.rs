//! `freekerf-library`: command line for maintaining the FreeKerf library.

mod commands;

use std::path::PathBuf;
use std::process::ExitCode;

use clap::{Parser, Subcommand, ValueEnum};

/// Validate, package and import the FreeKerf materials library.
#[derive(Debug, Parser)]
#[command(name = "freekerf-library", version, about)]
struct Cli {
    /// Library root (repository checkout).
    #[arg(long, global = true, default_value = ".")]
    root: PathBuf,
    #[command(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
    /// Validate every document (schema, lint, semantic rules).
    Validate {
        /// Fail on warnings too.
        #[arg(long)]
        deny_warnings: bool,
        /// Output format.
        #[arg(long, value_enum, default_value_t = Format::Text)]
        format: Format,
    },
    /// Generate `schema/v1/*.schema.json` from the Rust model.
    Schema {
        /// Only check that the committed files are up to date.
        #[arg(long)]
        check: bool,
    },
    /// Build the release package (`library-vX.Y.Z.tar.zst` + index JSON).
    Package {
        /// Release version (semver, without `v`).
        #[arg(long)]
        version: String,
        /// Output directory.
        #[arg(long, default_value = "dist")]
        out: PathBuf,
        /// mtime written in the archive (defaults to $SOURCE_DATE_EPOCH or 0).
        #[arg(long, env = "SOURCE_DATE_EPOCH", default_value_t = 0)]
        mtime: u64,
    },
    /// Import a third-party library.
    #[command(subcommand)]
    Import(ImportCommand),
}

#[derive(Debug, Subcommand)]
enum ImportCommand {
    /// Import LaserGRBL `StandardMaterials.psh`.
    Lasergrbl {
        /// The `.psh` file.
        #[arg(long)]
        input: PathBuf,
        /// Output directory (relative to the root).
        #[arg(long, default_value = "data/materials/imported/lasergrbl")]
        out: PathBuf,
        /// Model → laser overrides (relative to the root).
        #[arg(long, default_value = "tools/lasergrbl/models.toml")]
        models: PathBuf,
        /// Upstream URL recorded in `source.url`.
        #[arg(long, default_value = freekerf_library::import::lasergrbl::DEFAULT_SOURCE_URL)]
        source_url: String,
        /// Delete existing `.toml` files in the output directory first.
        #[arg(long)]
        clean: bool,
    },
}

/// Diagnostic output format.
#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
enum Format {
    Text,
    Json,
}

fn main() -> ExitCode {
    let cli = Cli::parse();
    let result = match cli.command {
        Command::Validate {
            deny_warnings,
            format,
        } => commands::validate::run(&cli.root, deny_warnings, format == Format::Json),
        Command::Schema { check } => commands::schema::run(&cli.root, check),
        Command::Package { version, out, mtime } => commands::package::run(&cli.root, &version, &out, mtime),
        Command::Import(ImportCommand::Lasergrbl {
            input,
            out,
            models,
            source_url,
            clean,
        }) => commands::import::lasergrbl(&commands::import::Args {
            root: &cli.root,
            input: &input,
            out: &out,
            models: &models,
            source_url: &source_url,
            clean,
        }),
    };
    match result {
        Ok(code) => code,
        Err(error) => {
            eprintln!("error: {error:#}");
            ExitCode::FAILURE
        }
    }
}
