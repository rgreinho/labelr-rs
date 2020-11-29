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
    pub organization: Option<String>,
    /// Set the owner name
    #[clap(long, env = "GITHUB_USER")]
    pub owner: Option<String>,
    /// Set repository directory
    #[clap(
        long,
        env = "GITHUB_REPOSITORY",
        parse(from_os_str),
        default_value = "."
    )]
    pub repository: PathBuf,
    /// Set GitHub token
    #[clap(long, env = "GITHUB_TOKEN", setting = ArgSettings::HideEnvValues)]
    pub token: String,
    /// Synchronize the labels
    #[clap(long)]
    pub sync: bool,
    /// Apply labels to entire GitHub organization
    #[clap(long)]
    pub org: bool,
    /// Specify the file containing the labels
    #[clap(parse(from_os_str), default_value = "labels.yml")]
    pub file: PathBuf,
    /// Update existing labels
    #[clap(long)]
    pub update_existing: bool,
}
