use clap::{crate_name, Clap, IntoApp};
use color_eyre::{eyre::eyre, eyre::Report, eyre::Result, Section, SectionExt};
use futures::future::{self, try_join_all};
use futures::prelude::*;
use hubcaps::{
    errors::Error, labels::Label, labels::LabelOptions, repositories::Repository,
    repositories::UserRepoListOptions, Credentials, Github,
};
use labelr::cli::Opts;
use labelr::{get_repo_info, Labels};
use std::collections::HashMap;
use std::collections::HashSet;
use std::fs;
use std::iter::FromIterator;

#[tokio::main]
async fn main() -> Result<(), Report> {
    color_eyre::install()?;

    // Will be useful to add shell completion.
    // let mut app = Opts::into_app();

    let opts: Opts = labelr::cli::Opts::parse();
    dbg!(&opts);

    // Collect the information in the following order:
    //   1. from repository
    //   2. from environment variables
    //   3. from CLI

    // Get the owner and repository.
    let (repository, infered_owner) = match get_repo_info(opts.repository.clone()) {
        Ok(info) => info,

        // If we cannot infer the values from the repository, the name should
        // be the name of the directory specified in opts.repository, and the
        // owner should come from opts.owner.
        Err(_) => match fs::canonicalize(opts.repository) {
            Ok(f) => match f.file_name() {
                Some(f) => match f.to_str() {
                    Some(f) => (String::from(f), opts.owner),
                    None => return Err(eyre!("invalid repository name")),
                },
                None => return Err(eyre!("invalid repository path")),
            },
            Err(e) => return Err(eyre!("the repository path does not exist: {}", e)),
        },
    };

    // Retrieve the owner from the infered_owner.
    let owner = match infered_owner {
        Some(o) => o,

        // If no owner was found, the organization will be used.
        None => match opts.organization {
            Some(org) => org,
            None => return Err(eyre!("no owner name or organization was found")),
        },
    };

    dbg!(&repository);
    dbg!(&owner);

    // Load label file.
    let labels = Labels::try_from_file(opts.file)?;

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
        dbg!(&user_repos);
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
            let mut tasks = Vec::new();
            for l in existing_labels.iter() {
                println!("Deleting label: \"{}\"", &l.name);
                tasks.push(ghlabels.delete(&l.name));
            }
            try_join_all(tasks).await?;
        }

        // Apply the labels.
        let mut tasks = Vec::new();
        for label in &labels.labels {
            // In syncing mode, we simply create a new label since all the
            // existing ones were deleted.
            if opts.sync {
                println!("Creating label: \"{}\"", label.name);
                tasks.push(ghlabels.create(&label.to_label_options()));
            } else {
                // Otherwise we check whether the label exists.
                if existing_labels.iter().any(|l| label.name == l.name) {
                    // And either update it.
                    if opts.update_existing {
                        println!("Updating existing label: \"{}\"", label.name);
                        tasks.push(ghlabels.update(&label.name, &label.to_label_options()));
                    }
                    // Or skip it.
                    else {
                        println!("Skipping existing label: \"{}\"", label.name);
                    }
                }
                // If the label does not exist we simply create it.
                else {
                    println!("Creating label: \"{}\"", label.name);
                    tasks.push(ghlabels.create(&label.to_label_options()));
                }
            }
        }

        // Process all the tasks.
        try_join_all(tasks).await?;
    }
    Ok(())
}

// This section bellow works well, but we can probably do better.
// match ghlabels.create(&bl.to_label_options()).await {
//     Ok(v) => println!("Label \"{}\" created", v.name),
//     Err(e) => {
//         dbg!(&e);
//         println!("cannot create label \"{}\"", bl.name);
//         match e {
//             Error::Fault { code, error } => {
//                 println!("client error: {} - {}", code, error.message);
//                 match error.errors {
//                     Some(errors) => {
//                         for err in errors.iter() {
//                             println!("reason: {}", err.code);
//                         }
//                         if errors.iter().any(|e| e.code == "already_exists") {
//                             if opts.update_existing {
//                                 println!("updating existing label: \"{}\"", bl.name);
//                                 ghlabels.update(&bl.name, &bl.to_label_options()).await?;
//                             } else {
//                                 println!("skipping existing label: \"{}\"", bl.name);
//                             }
//                         }
//                     }
//                     None => println!("unknown error"),
//                 }
//             }
//             _ => println!("other reason: {}", e),
//         }
//     }
// };
