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
        Ok(r) => r,
        Err(e) => {
            return Err(eyre!(
                r#"cannot find the remote url from repository located at "{:?}": {}"#,
                path,
                e
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

    // Extract owner and repo from the parsed path (e.g., "owner/repo.git" or "/owner/repo").
    let p = parsed
        .path()
        .trim_start_matches('/')
        .trim_end_matches(".git");
    let parts: Vec<&str> = p.split('/').collect();
    if parts.len() >= 2 {
        let owner = parts[0].to_string();
        let name = parts[1].to_string();
        Ok((name, Some(owner)))
    } else {
        Ok((p.to_string(), None))
    }
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
