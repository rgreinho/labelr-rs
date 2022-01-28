use eyre::{eyre, Result};
use git2::Repository;
use git_url_parse::GitUrl;
use std::env;
use std::fs;
use std::path::PathBuf;

pub fn get_repo_info_from_remote(path: PathBuf) -> Result<(String, Option<String>)> {
    let repo = Repository::discover(&path)?;
    let remote = repo.find_remote("origin")?;
    let remote_url = match remote.url() {
        Some(r) => r,
        None => {
            return Err(eyre!(
                r#"cannot find the remote url from repository located at "{:?}""#,
                path,
            ))
        }
    };
    let parsed = match GitUrl::parse(remote_url) {
        Ok(p) => p,
        Err(e) => {
            return Err(eyre!(
                r#"cannot parse remote url from repository "{:?}": {}"#,
                path,
                e
            ))
        }
    };
    Ok((parsed.name, parsed.owner))
}

pub fn infer_repo_info(
    path: PathBuf,
    owner: Option<String>,
    organization: &Option<String>,
) -> Result<(String, String)> {
    // Get the owner and repository from the remote URL.
    let (repository, infered_owner) = match get_repo_info_from_remote(path.clone()) {
        Ok(info) => info,

        // If we cannot infer the values from the remote URL, we can fallback to other
        // options.
        Err(_) => {
            // The name should fallback to the name of the directory specified in `path`.
            let r = match fs::canonicalize(path) {
                Ok(f) => match f.file_name() {
                    Some(f) => match f.to_str() {
                        Some(f) => String::from(f),
                        None => return Err(eyre!("invalid repository name")),
                    },
                    None => return Err(eyre!("invalid repository path")),
                },
                Err(e) => return Err(eyre!("the repository path does not exist: {}", e)),
            };

            // The owner should fallback to either the"GITHUB_USER" environment variable,
            // either the provided fallback value.
            let o = match env::var("GITHUB_USER") {
                Ok(o) => Some(o),
                Err(_) => owner,
            };
            (r, o)
        }
    };

    // Retrieve the owner from the infered_owner.
    let owner = match infered_owner {
        Some(o) => o,

        // If no owner was found, let's try the GITHUB_ORGANIZATION environment variable.
        None => match env::var("GITHUB_ORGANIZATION") {
            Ok(o) => o,
            Err(_) => match organization {
                // If no owner was found, fallback to the provided organization value.
                Some(org) => org.to_string(),
                None => return Err(eyre!("no owner name or organization was found")),
            },
        },
    };

    Ok((repository, owner))
}
