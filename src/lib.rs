pub mod cli;

use eyre::Result;
use serde::Deserialize;
use std::convert::TryFrom;
use std::fs;
use std::io::Write;
use std::path::PathBuf;
use tempfile::NamedTempFile;

#[derive(Debug, Deserialize)]
pub struct Label {
    pub color: String,
    pub name: String,
    pub description: String,
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
            description: "This is a bug".to_string(),
        };
        assert_eq!(actual, expected);
    }

    #[test]
    fn deserialize_list() {
        let actual: Vec<Label> = serde_yaml::from_str(&LABEL_LIST).unwrap();
        let expected = vec![Label {
            color: "#FEFEFE".to_string(),
            name: "bug".to_string(),
            description: "This is a bug".to_string(),
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
                description: "This is a bug".to_string(),
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
                description: "This is a bug".to_string(),
            }],
        };
        assert_eq!(actual, expected);
    }
}
