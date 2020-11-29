pub mod cli;

use eyre::{eyre, Result};
use git2::Repository;
use git_url_parse::GitUrl;
use hubcaps::labels::LabelOptions;
use serde::{Deserialize, Deserializer};
use std::convert::TryFrom;
use std::fs;
use std::io::Write;
use std::path::PathBuf;
use tempfile::NamedTempFile;

#[derive(Debug, Deserialize, Clone)]
pub struct Label {
    // #[serde(deserialize_with = "no_pound")]
    pub color: String,
    pub name: String,
    pub description: Option<String>,
}

impl PartialEq for Label {
    fn eq(&self, other: &Self) -> bool {
        self.color == other.color
            && self.name == other.name
            && self.description == other.description
    }
}

impl TryFrom<&str> for Label {
    type Error = serde_yaml::Error;

    fn try_from(s: &str) -> Result<Self, Self::Error> {
        let l: Label = serde_yaml::from_str(&s)?;
        Ok(l)
    }
}

impl From<LabelOptions> for Label {
    fn from(item: LabelOptions) -> Self {
        Label {
            color: item.color.replace("#", ""),
            name: item.name,
            description: Some(item.description),
        }
    }
}

impl Label {
    pub fn to_label_options(&self) -> LabelOptions {
        LabelOptions {
            color: self.color.replace("#", ""),
            name: self.name.clone(),
            description: match self.description.clone() {
                Some(d) => d,
                None => String::from(""),
            },
        }
    }
}

// Buggy deserialization:
//    Message("invalid type: string \"#FEFEFE\", expected a borrowed string",
//    Some(Pos { marker: Marker { index: 23, line: 3, col: 11 }, path: "labels[0].color" }))
fn no_pound<'de, D>(deserializer: D) -> std::result::Result<String, D::Error>
where
    D: Deserializer<'de>,
{
    let s: &str = Deserialize::deserialize(deserializer)?;
    Ok(s.replace("#", ""))
}

#[derive(Debug, Deserialize)]
pub struct Labels {
    pub labels: Vec<Label>,
}

impl Labels {
    pub fn try_from_file(path: PathBuf) -> Result<Self> {
        let contents = fs::read_to_string(path)?;
        let labels: Labels = serde_yaml::from_str(&contents)?;
        Ok(labels)
    }
}

impl PartialEq for Labels {
    fn eq(&self, other: &Self) -> bool {
        self.labels == other.labels
    }
}

pub fn get_repo_info(path: PathBuf) -> Result<(String, Option<String>)> {
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

// impl<'a> TryFrom(&str) for Labels {
//     type Error = serde_yaml::Error;

//     fn try_from(s: &str) -> Result<Self, Self::Error> {
//         let l: Vec<Labels> = serde_yaml::from_str(&s)?;
//         Ok(l)
//     }
// }

#[cfg(test)]
mod tests {
    use super::*;

    const SINGLE_LABEL: &str = "---\ncolor: \"#FEFEFE\"\nname: bug\ndescription: This is a bug";
    const LABEL_LIST: &str = "---\n- color: \"#FEFEFE\"\n  name: bug\n  description: This is a bug";
    const LABELS: &str =
        "---\nlabels:\n  - color: \"#FEFEFE\"\n    name: bug\n    description: This is a bug";

    #[test]
    fn deserialize_single() {
        let actual = Label::try_from(SINGLE_LABEL).unwrap();
        let expected = Label {
            color: "#FEFEFE".to_string(),
            name: "bug".to_string(),
            description: Some("This is a bug".to_string()),
        };
        assert_eq!(actual, expected);
    }

    #[test]
    fn deserialize_list() {
        let actual: Vec<Label> = serde_yaml::from_str(&LABEL_LIST).unwrap();
        let expected = vec![Label {
            color: "#FEFEFE".to_string(),
            name: "bug".to_string(),
            description: Some("This is a bug".to_string()),
        }];
        assert_eq!(actual, expected);
    }

    #[test]
    fn deserialize_list_struc() {
        let actual: Labels = serde_yaml::from_str(&LABELS).unwrap();
        let expected = Labels {
            labels: vec![Label {
                color: "#FEFEFE".to_string(),
                name: "bug".to_string(),
                description: Some("This is a bug".to_string()),
            }],
        };
        assert_eq!(actual, expected);
    }

    #[test]
    fn deserialize_from_file() {
        let mut tmpfile: NamedTempFile = NamedTempFile::new().unwrap();
        let _ = write!(tmpfile, "{}", LABELS);
        let actual = Labels::try_from_file(tmpfile.path().into()).unwrap();
        let expected = Labels {
            labels: vec![Label {
                color: "#FEFEFE".to_string(),
                name: "bug".to_string(),
                description: Some("This is a bug".to_string()),
            }],
        };
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_get_repo_info() {
        let (name, owner) = get_repo_info(PathBuf::from(".")).unwrap();
        assert_eq!(name, "labelr-rs");
        assert_eq!(owner.unwrap(), "rgreinho");
    }
}
