use eyre::Result;
use futures::future::try_join_all;
use octocrab::Octocrab;
use serde::{Deserialize, Serialize, Serializer};
use std::fs;
use std::path::PathBuf;
use tracing::{event, Level};

#[derive(Debug, Deserialize, Clone, PartialEq, Eq)]
pub struct Label {
    pub color: String,
    pub name: String,
    pub description: Option<String>,
}

impl TryFrom<&str> for Label {
    type Error = serde_yaml::Error;

    fn try_from(s: &str) -> Result<Self, Self::Error> {
        serde_yaml::from_str(s)
    }
}

#[derive(Serialize)]
pub struct LabelBody {
    pub name: String,
    #[serde(serialize_with = "serialize_color")]
    pub color: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

fn serialize_color<S>(color: &str, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    serializer.serialize_str(&color.replace('#', ""))
}

impl From<&Label> for LabelBody {
    fn from(label: &Label) -> Self {
        LabelBody {
            name: label.name.clone(),
            color: label.color.clone(),
            description: label.description.clone(),
        }
    }
}

impl From<Label> for LabelBody {
    fn from(label: Label) -> Self {
        LabelBody {
            name: label.name,
            color: label.color,
            description: label.description,
        }
    }
}

#[cfg(test)]
mod extra_tests {
    use super::*;

    #[test]
    fn to_label_body_creates_expected_json() {
        let label = Label {
            color: "#FFEEAA".to_string(),
            name: "bug".to_string(),
            description: Some("a bug".to_string()),
        };
        let body: LabelBody = (&label).into();
        let v = serde_json::to_value(&body).unwrap();
        assert_eq!(v["name"], "bug");
        assert_eq!(v["color"], "FFEEAA");
        assert_eq!(v["description"], "a bug");
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

#[derive(Debug, Deserialize, PartialEq, Eq)]
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
    octo: &Octocrab,
    owner: &str,
    repo: &str,
    labels: Vec<octocrab::models::Label>,
) -> Result<()> {
    let mut tasks = Vec::new();
    for l in labels.iter() {
        event!(Level::INFO, "Deleting label: \"{}\"", &l.name);
        let uri = format!("/repos/{}/{}/labels/{}", owner, repo, l.name);
        tasks.push(octo._delete(uri, None::<&()>));
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
