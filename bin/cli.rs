use clap::{ArgAction, Parser};
use djwavfixer::{Result, WavFile};
use std::fmt::Write;
use std::num::NonZeroUsize;
use std::path::PathBuf;
use std::{path, thread};

/// Command-line interface for djwavfixer
#[derive(Parser, Debug)]
#[command(author, version, about)]
pub struct Cli {
    /// Directory or file to process
    pub path: PathBuf,

    #[arg(long, action=ArgAction::SetTrue)]
    pub recursive: bool,

    /// Use async processing
    #[arg(long, required = false, default_missing_value = "false")]
    pub use_async: bool,

    /// Number of threads to use(default is number of logical cores)
    #[arg(long, required = false)]
    pub num_threads: Option<NonZeroUsize>,

    /// Whether to only print fixable files
    #[arg(long, required = false, default_missing_value = "false")]
    pub ignore_unfixable: bool,

    /// Whether to only print that need fixing
    #[arg(long, required = false, default_missing_value = "false")]
    pub ignore_valid: bool,

    /// Apply fixes to files if possible
    #[arg(long, required = false, default_missing_value = "false")]
    pub fix: bool,

    #[arg(long, required = false, default_missing_value = "false")]
    pub silent: bool,
}

fn get_files(cli: &Cli) -> Result<Vec<WavFile>> {
    let path_to_read = path::absolute(&cli.path)?;
    let files = if path_to_read.is_dir() {
        djwavfixer::get_all_wav_files_in_directory(&path_to_read, cli.recursive)?
    } else if path_to_read.is_file() {
        vec![path_to_read.clone()]
    } else {
        return Err(djwavfixer::DJWavFixerError::GeneralError(format!(
            "The specified path `{}` is neither a file nor a directory.",
            path_to_read.display()
        )));
    };

    let num_threads = cli
        .num_threads
        .unwrap_or(thread::available_parallelism()?)
        .get();

    let pool = (num_threads > 1)
        .then(|| {
            rayon::ThreadPoolBuilder::new()
                .num_threads(num_threads)
                .build()
        })
        .transpose()?;

    let mut read_files = if let Some(pool) = pool.as_ref() {
        djwavfixer::load_wav_files_rayon(&files, pool)?
    } else {
        djwavfixer::load_wav_files(&files)?
    };

    if read_files.is_empty() {
        log::warn!(
            "No valid WAV files found in the specified directory `{}`{}.",
            path_to_read.display(),
            if cli.recursive { "(Recursively)" } else { "" }
        );
        return Ok(vec![]);
    }

    if cli.ignore_valid {
        read_files.retain(|file_res| file_res.needs_fixing().unwrap_or(true));
    }

    if cli.ignore_unfixable {
        read_files.retain(|file_res| file_res.can_fix().unwrap_or_default());
    }

    if read_files.is_empty() {
        log::info!("All WAV files are valid for players.");
        return Ok(vec![]);
    }

    read_files.sort_by(|a, b| a.path().cmp(b.path()));

    Ok(read_files)
}

fn run_with_cli(cli: Cli) -> Result<()> {
    let read_files = get_files(&cli)?;
    if read_files.is_empty() {
        // Error message already logged in get_files
        return Ok(());
    }

    log::info!("Found WAV files:");
    let file_information = read_files.iter().enumerate().try_fold(
        "\n".to_string(),
        |mut acc, (file_number, file)| {
            writeln!(acc, "{}:", file_number + 1)?;
            file.write_information(&mut acc)?;
            Result::Ok(acc)
        },
    )?;

    log::info!("{}", file_information.trim_end());

    Ok(())
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    if !cli.silent {
        simple_logger::init_with_env().expect("Could not initialize logger");
    }
    run_with_cli(cli)
}
