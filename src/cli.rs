use clap::{crate_name, ArgSettings, Clap};
use std::path::PathBuf;

// Labelr main options.
#[derive(Clap, Debug)]
#[clap(name = crate_name!(), author, about, version)]
pub struct Opts {
    /// Sets the verbosity level
    #[clap(short, long, parse(from_occurrences))]
    pub verbose: u8,
    /// Set organization name
    #[clap(long, env = "GITHUB_ORGANIZATION")]
    pub organization: String,
    /// Set the owner name
    #[clap(long, env = "GITHUB_USER")]
    pub owner: String,
    /// Set repostiory directory
    #[clap(long, env = "GITHUB_REPOSITORY", parse(from_os_str))]
    pub repository: PathBuf,
    /// Set GitHub token
    #[clap(long, env = "GITHUB_TOKEN", setting = ArgSettings::HideEnvValues)]
    pub token: String,
}
