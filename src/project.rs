use serde::{Deserialize, Serialize};
use std::fs;

#[derive(Debug, Serialize, Deserialize)]
pub struct Project {
  pub name: String,
  pub version: String,
  pub authors: Option<Vec<String>>,
  pub description: Option<String>,
  pub kind: String,
  pub entry: String,
}

impl Project {
  pub fn new() -> Self {
    match fs::read_to_string("./project.json") {
      Ok(data) => match serde_json::from_str::<Project>(data.as_str()) {
        Ok(resp) => resp,
        Err(e) => panic!(
          "An error occurred while parsing the project.json file: {}",
          e
        ),
      },
      Err(e) => panic!(
        "An error occurred while trying to open project.json file: {}",
        e
      ),
    }
  }
}
