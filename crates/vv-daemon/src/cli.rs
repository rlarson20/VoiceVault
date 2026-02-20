use clap::Parser;
use std::path::PathBuf;

/// VoiceVault daemon - Voice-activated personal assistant
#[derive(Parser, Debug)]
#[command(name = "vv-daemon")]
#[command(about = "VoiceVault daemon - Voice-activated personal assistant", long_about = None)]
pub struct Args {
    /// Path to configuration file
    #[arg(short, long)]
    pub config: Option<PathBuf>,

    /// Trigger word phrase (default: from config)
    #[arg(short, long)]
    pub trigger: Option<String>,

    /// Model to use for processing
    #[arg(short, long)]
    pub model: Option<String>,

    /// Path to vault directory
    #[arg(short = 'V', long)]
    pub vault: Option<PathBuf>,

    /// Verbosity level (-v, -vv, -vvv)
    #[arg(short, long, action = clap::ArgAction::Count)]
    pub verbose: u8,

    /// Transcribe a single audio file and exit (skips daemon mode)
    #[arg(long, value_name = "FILE")]
    pub transcribe_only: Option<PathBuf>,
}
