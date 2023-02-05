use clap::{crate_name, ArgAction, Parser, ValueHint};
use std::path::PathBuf;

// Labelr main options.
#[derive(Parser, Debug)]
#[clap(name = crate_name!(), author, about, version)]
pub struct Opts {
    /// Sets the verbosity level
    #[clap(short, long,  action = ArgAction::Count)]
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
        value_parser,
        value_hint = ValueHint::DirPath,
        default_value = "."
    )]
    pub repository: PathBuf,
    /// Set GitHub token
    #[clap(long, env = "GITHUB_TOKEN", hide_env_values = true)]
    pub token: String,
    /// Synchronize the labels
    #[clap(long)]
    pub sync: bool,
    /// Apply labels to entire GitHub organization
    #[clap(long)]
    pub org: bool,
    /// Specify the file containing the labels
    #[clap(default_value = "labels.yml")]
    pub file: PathBuf,
    /// Update existing labels
    #[clap(long)]
    pub update_existing: bool,
}
