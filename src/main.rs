use clap::Parser;
use color_eyre::{eyre::eyre, eyre::Report, eyre::Result};
use labelr::cli::Opts;
use labelr::git::infer_repo_info;
use labelr::label::{delete_labels, Labels};
use rand::Rng;
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

    // Create the GitHub client using octocrab with the provided token.
    let octo = octocrab::OctocrabBuilder::default()
        .personal_token(opts.token.clone())
        .build()
        .expect("failed to build octocrab client");

    // Prepare the collection of repository identifiers (owner, repo).
    let mut repos: Vec<(String, String)> = Vec::new();

    // List organisation repositories.
    if opts.org {
        // List all organization repositories with pagination
        let page = octo
            .orgs(&owner)
            .list_repos()
            .per_page(100u8)
            .send()
            .await?;
        let user_repos = octo.all_pages::<octocrab::models::Repository>(page).await?;
        for user_repo in user_repos.iter() {
            repos.push((owner.clone(), user_repo.name.clone()));
        }
    }
    // Or use only the current repository.
    else {
        repos.push((owner.clone(), repository));
    }

    // Retry helper: retries on GitHub 429 or server errors and on transient transport errors.
    use labelr::retry::retry_octocrab;

    // Process repositories concurrently with bounded parallelism.
    use futures::StreamExt;
    use std::sync::Arc;

    let octo = Arc::new(octo);
    let concurrency = 8usize;

    let sync = opts.sync;
    let update_existing = opts.update_existing;

    let repo_stream = futures::stream::iter(repos.into_iter().map(|(owner, repo_name)| {
        let octo = octo.clone();
        let labels_vec = labels.labels.clone();
        async move {
            // List existing labels for the repository with retries.
            let labels_route = format!("/repos/{}/{}/labels", owner, repo_name);
            let existing_labels: Vec<octocrab::models::Label> =
                retry_octocrab(|| octo.get(&labels_route, None::<&()>), 5)
                    .await
                    .map_err(|e| eyre!(e))?;

            // Delete existing labels if syncing mode is enabled.
            if sync {
                // Retry delete_labels a few times on error with exponential backoff.
                let mut attempt = 0u32;
                loop {
                    match delete_labels(&octo, &owner, &repo_name, existing_labels.clone()).await {
                        Ok(_) => break,
                        Err(e) => {
                            attempt += 1;
                            if attempt >= 3 {
                                return Err(eyre!(e));
                            }
                            let base_secs = 2u64.pow(attempt - 1);
                            let mut rng = rand::thread_rng();
                            let jitter_ms = rng.gen_range(0..(base_secs * 1000));
                            let backoff =
                                std::time::Duration::from_millis(base_secs * 1000 + jitter_ms);
                            tokio::time::sleep(backoff).await;
                        }
                    }
                }
            }

            // Apply the labels sequentially per repository with retries.
            for label in &labels_vec {
                let body = serde_json::to_value(labelr::label::LabelBody::from(label))?;

                if sync {
                    event!(Level::INFO, "Creating label: \"{}\"", label.name);
                    let create_route = format!("/repos/{}/{}/labels", owner, repo_name);
                    let _created: octocrab::models::Label =
                        retry_octocrab(|| octo.post(&create_route, Some(&body)), 5)
                            .await
                            .map_err(|e| eyre!(e))?;
                } else {
                    if existing_labels.iter().any(|l| label.name == l.name) {
                        if update_existing {
                            event!(Level::INFO, "Updating existing label: \"{}\"", label.name);
                            let patch_route =
                                format!("/repos/{}/{}/labels/{}", owner, repo_name, label.name);
                            let _updated: octocrab::models::Label =
                                retry_octocrab(|| octo.patch(&patch_route, Some(&body)), 5)
                                    .await
                                    .map_err(|e| eyre!(e))?;
                        } else {
                            event!(Level::INFO, "Skipping existing label: \"{}\"", label.name);
                        }
                    } else {
                        event!(Level::INFO, "Creating label: \"{}\"", label.name);
                        let create_route = format!("/repos/{}/{}/labels", owner, repo_name);
                        let _created: octocrab::models::Label =
                            retry_octocrab(|| octo.post(&create_route, Some(&body)), 5)
                                .await
                                .map_err(|e| eyre!(e))?;
                    }
                }
            }

            Ok(())
        }
    }))
    .buffer_unordered(concurrency);

    let results: Vec<Result<(), Report>> = repo_stream.collect().await;
    for r in results.into_iter() {
        r?;
    }
    Ok(())
}
