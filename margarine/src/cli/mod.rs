mod artifacts;
mod compile;
mod distribution;
mod installation;
mod test;
mod toolchain;
mod update;

use std::path::PathBuf;

use clap::{Parser, Subcommand};
use colourful::ColourBrush;
use margarine::CompilationTarget;

use artifacts::ArtifactsLock;

pub const TICK_GLYPH : &str = "✓";

#[derive(Parser)]
#[command(
    name = "margarine",
    version = margarine::DISPLAY_VERSION,
    propagate_version = true,
    about = "a unified development toolchain for the margarine programming language",
    after_help = format!(
        "{}\n  {} {} {}\n  {} {}\n\n{} {}",
        "Quick start:".bold().underline(),
        "margarine run".bold(),
        "hello.mar".cyan(),
        "# compile & execute".dim(),
        "margarine test".bold(),
        "tests/core.mar".cyan(),
        "docs:".dim(),
        "https://github.com/todaymare/margarine",
    ),
    arg_required_else_help = true,
)]

struct Cli {
    /// Print elapsed time for each compilation stage
    #[arg(long, global = true)]
    timings: bool,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {

    /// Compile a source file into an executable or shared library
    Build {
        /// Source path
        #[arg(value_parser = existing_file_path)]
        path: PathBuf,

        /// Compilation target: default (host), arm64-apple-darwin, x86_64-unknown-linux-gnu, aarch64-unknown-linux-gnu, or wasm32-unknown-unknown
        #[arg(long, default_value = "default")]
        target: CompilationTarget,

        /// Build a native shared library instead of an executable
        #[arg(long)]
        shared: bool,

        /// Output path
        #[arg(short, long)]
        output: Option<String>,

        /// Cache directory
        #[arg(long)]
        cache: Option<String>,

        /// Reset the build cache before compiling
        #[arg(long)]
        update: bool,
    },

    /// Compile and run a source file
    Run {
        /// Source path
        #[arg(value_parser = existing_file_path)]
        path: PathBuf,

        /// Compilation target: default (host), arm64-apple-darwin, x86_64-unknown-linux-gnu, aarch64-unknown-linux-gnu, or wasm32-unknown-unknown
        #[arg(long, default_value = "default")]
        target: CompilationTarget,

        /// Cache directory
        #[arg(long)]
        cache: Option<String>,

        /// Output path
        #[arg(short, long)]
        output: Option<String>,

        /// Reset the build cache before compiling
        #[arg(long)]
        update: bool,

        /// Arguments passed to the program
        #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
        program_args: Vec<String>,
    },

    /// Check a source file for errors without producing output files
    Check {
        /// Source path
        #[arg(value_parser = existing_file_path)]
        path: PathBuf,

        /// Compilation target: default (host), arm64-apple-darwin, x86_64-unknown-linux-gnu, aarch64-unknown-linux-gnu, or wasm32-unknown-unknown
        #[arg(long, default_value = "default")]
        target: CompilationTarget,

        /// Cache directory
        #[arg(long)]
        cache: Option<String>,

        /// Reset the build cache before checking
        #[arg(long)]
        update: bool,
    },


    /// Compile and run tests
    Test {
        /// Source path
        #[arg(default_value = ".", value_parser = existing_path)]
        path: PathBuf,

        /// Test name filter
        filter: Option<String>,

        /// Compilation target: default (host), arm64-apple-darwin, x86_64-unknown-linux-gnu, aarch64-unknown-linux-gnu, or wasm32-unknown-unknown
        #[arg(long, default_value = "default")]
        target: CompilationTarget,

        /// Cache directory
        #[arg(long)]
        cache: Option<String>,

        /// Reset the build cache before testing
        #[arg(long)]
        update: bool,
    },


    /// Check for a newer release and self-update
    Update,

    /// Manage compiler target toolchains
    Toolchain {
        #[command(subcommand)]
        command: ToolchainCommands,
    },


    /// Remove build artifacts
    Clean {
        /// Cache directory
        #[arg(long)]
        cache: Option<String>,
    },
}


#[derive(Subcommand)]
enum ToolchainCommands {
    /// Install a compiler target toolchain
    Add {
        /// Toolchain target
        target: CompilationTarget,
    },
}

/// Process exit codes, complementing clap's own usage-error code (2).
pub(super) const COMPILE_ERROR: i32 = 1;
pub(super) const LINK_ERROR: i32 = 3;
pub(super) const PROGRAM_ERROR: i32 = 4;

pub(super) type CliResult<T> = Result<T, CliError>;

#[derive(Debug)]
pub(super) struct CliError {
    code: i32,
    message: String,
}

impl CliError {
    pub(super) fn link(message: impl std::fmt::Display) -> Self {
        Self { code: LINK_ERROR, message: message.to_string() }
    }
}

pub(crate) fn main() -> i32 {
    let cli = Cli::parse();
    if cli.timings {
        std::env::set_var("MARGARINE_TIMING", "1");
    }

    let command = cli.command;
    let _lock =
        if matches!(
            &command,
            Commands::Build { .. }
            | Commands::Run { .. }
            | Commands::Check { .. }
            | Commands::Test { .. }
            | Commands::Clean { .. }
        ) {
            Some(ArtifactsLock::acquire())
        } else {
            None
        };
    let result = execute(command);

    match result {
        Ok(code) => code,
        Err(error) => {
            let code = error.code;
            let _ = runtime_error(error.message).print();
            eprintln!();
            code
        },
    }
}

fn execute(command: Commands) -> CliResult<i32> {
    match command {
        Commands::Build { path, target, shared, output, cache, update } => {
            let cache = artifacts::reset_cache_if(update, cache);
            let mode = 
            if shared {
                margarine::BuildMode::Shared
            } else {
                margarine::BuildMode::Executable
            };

            compile::compile_and_link(&path, target, mode, output, cache)?;
            Ok(0)
        },
        Commands::Run { path, target, output, cache, update, program_args } => {
            let cache = artifacts::reset_cache_if(update, cache);
            compile::run(&path, target, output, cache, program_args)
        },
        Commands::Test { path, filter, target, cache, update } => {
            let cache = artifacts::reset_cache_if(update, cache);
            test::execute(&path, filter, target, cache)
        },
        Commands::Check { path, target, cache, update } => {
            let cache = artifacts::reset_cache_if(update, cache);
            compile::check(&path, target, cache)
        },
        Commands::Update => update::execute(),
        Commands::Toolchain { command: ToolchainCommands::Add { target } } => {
            toolchain::add(target)
        },
        Commands::Clean { cache } => {
            artifacts::clean(&cache.unwrap_or_else(|| "artifacts".to_string()));
            Ok(0)
        },
    }
}

fn runtime_error(message: impl std::fmt::Display) -> clap::error::Error {
    use clap::error::{Error, ErrorKind};

    let cmd = <Cli as clap::CommandFactory>::command();
    Error::raw(ErrorKind::Io, message).with_cmd(&cmd)
}

fn existing_path(path: &str) -> Result<PathBuf, String> {
    if std::fs::exists(path).unwrap_or(false) {
        Ok(PathBuf::from(path))
    } else {
        Err(format!("path does not exist: {path}"))
    }
}

fn existing_file_path(path: &str) -> Result<PathBuf, String> {
    let path = existing_path(path)?;
    if path.is_file() {
        Ok(path)
    } else {
        Err(format!("path is not a file: {}", path.display()))
    }
}

#[cfg(test)]
mod tests {
    use super::runtime_error;

    #[test]
    fn runtime_errors_do_not_render_usage_guidance() {
        let rendered = runtime_error("build script failed").to_string();

        assert!(rendered.contains("error: build script failed"));
        assert!(!rendered.contains("Usage:"));
        assert!(!rendered.contains("For more information"));
    }
}
