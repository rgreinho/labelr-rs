use clap::Parser;
use color_eyre::{eyre::eyre, eyre::Report, eyre::Result};
use futures::future::try_join_all;
use hubcaps::{repositories::Repository, repositories::UserRepoListOptions, Credentials, Github};
use labelr::cli::Opts;
use labelr::git::infer_repo_info;
use labelr::label::{delete_labels, Labels};
use tracing::{event, Level};

#[tokio::main]
async fn main() -> Result<(), Report> {
    color_eyre::install()?;

    // Will be useful to add shell completion.
    // let mut app = Opts::into_app();

    let opts: Opts = labelr::cli::Opts::parse();
    dbg!(&opts);

    // Configure tracing.
    let log_level = match opts.verbose {
        0 => Level::WARN,
        1 => Level::INFO,
        2 => Level::DEBUG,
        _ => Level::TRACE,
    };
    tracing_subscriber::fmt().with_max_level(log_level).init();

    // Collect the information in the following order:
    //   1. from repository
    //   2. from environment variables
    //   3. from CLI

    // Get the owner and repository.
    let (repository, owner) = match infer_repo_info(opts.repository, opts.owner, &opts.organization)
    {
        Ok(values) => values,
        Err(e) => return Err(eyre!("cannot infer the repository/owner values: {}", e)),
    };

    // Load label file.
    let labels = Labels::try_from_file(opts.file).expect("cannot load the label file");

    // Create the github client.
    let github = Github::new(
        concat!(env!("CARGO_PKG_NAME"), "/", env!("CARGO_PKG_VERSION")),
        Credentials::Token(opts.token),
    )?;

    // Prepare the collection of repositories to process.
    let mut repos = Vec::<Repository>::new();

    // List organisation repositories.
    if opts.org {
        let ghrepos = github.user_repos(&owner);
        let user_repos = ghrepos
            .list(&UserRepoListOptions::builder().build())
            .await?;
        for user_repo in &user_repos {
            repos.push(github.repo(&owner, &user_repo.name));
        }
    }
    // Or use only the current repository.
    else {
        repos = vec![github.repo(&owner, &repository)];
    }

    // Process each repository.
    for repo in repos {
        // Get the label service.
        let ghlabels = repo.labels();

        // List existing labels.
        let existing_labels = ghlabels.list().await?;

        // Delete existing labels if syncing mode is enabled.
        if opts.sync {
            delete_labels(repo.labels(), repo.labels().list().await?).await?;
        }

        // Apply the labels.
        let mut tasks = Vec::new();
        for label in &labels.labels {
            // In syncing mode, we simply create a new label since all the
            // existing ones were deleted.
            if opts.sync {
                event!(Level::INFO, "Creating label: \"{}\"", label.name);
                tasks.push(ghlabels.create(&label.to_label_options()));
            } else {
                // Otherwise we check whether the label exists.
                if existing_labels.iter().any(|l| label.name == l.name) {
                    // And either update it.
                    if opts.update_existing {
                        event!(Level::INFO, "Updating existing label: \"{}\"", label.name);
                        tasks.push(ghlabels.update(&label.name, &label.to_label_options()));
                    }
                    // Or skip it.
                    else {
                        event!(Level::INFO, "Skipping existing label: \"{}\"", label.name);
                    }
                }
                // If the label does not exist we simply create it.
                else {
                    event!(Level::INFO, "Creating label: \"{}\"", label.name);
                    tasks.push(ghlabels.create(&label.to_label_options()));
                }
            }
        }

        // Process all the tasks..
        try_join_all(tasks).await?;
    }
    Ok(())
}
