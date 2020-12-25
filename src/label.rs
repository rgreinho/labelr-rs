use eyre::Result;
use futures::future::try_join_all;
use hubcaps::labels::LabelOptions;
use serde::Deserialize;
use std::convert::TryFrom;
use std::fs;
use std::path::PathBuf;
use tracing::{event, Level};

#[derive(Debug, Deserialize, Clone, PartialEq)]
pub struct Label {
    // #[serde(deserialize_with = "no_pound")]
    pub color: String,
    pub name: String,
    pub description: Option<String>,
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
// fn no_pound<'de, D>(deserializer: D) -> std::result::Result<String, D::Error>
// where
//     D: Deserializer<'de>,
// {
//     let s: &str = Deserialize::deserialize(deserializer)?;
//     Ok(s.replace("#", ""))
// }

#[derive(Debug, Deserialize, PartialEq)]
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

pub async fn delete_labels(
    ghlabels: hubcaps::labels::Labels,
    labels: Vec<hubcaps::labels::Label>,
) -> Result<()> {
    let mut tasks = Vec::new();
    for l in labels.iter() {
        event!(Level::INFO, "Deleting label: \"{}\"", &l.name);
        tasks.push(ghlabels.delete(&l.name));
    }
    try_join_all(tasks).await?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::git::get_repo_info_from_remote;
    use std::io::Write;
    use tempfile::NamedTempFile;

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
        let (name, owner) = get_repo_info_from_remote(PathBuf::from(".")).unwrap();
        assert_eq!(name, "labelr-rs");
        assert_eq!(owner.unwrap(), "rgreinho");
    }
}
