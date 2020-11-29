use clap::{crate_name, Clap, IntoApp};
use color_eyre::eyre::Result;
use labelr::cli::Opts;

fn main() -> Result<()> {
    color_eyre::install()?;

    let mut app = Opts::into_app();
    let opts: Opts = labelr::cli::Opts::parse();
    dbg!(opts);
    Ok(())
}
