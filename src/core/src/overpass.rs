use reqwest::blocking::Client;
use handlebars::Handlebars;
use serde_json::json;
use std::{fs, collections::HashMap};

use crate::Config;

pub const QUERY_FILENAME: &'static str = "./res/query.ovql";

#[derive(serde::Deserialize, Clone, Debug)]
pub struct OverpassResponseElement {
  #[serde(rename(deserialize = "type"))]
  pub element_type: String,
  pub nodes: Option<Vec<u64>>,
  pub id: u64,
  pub tags: Option<HashMap<String, String>>,
  pub lat: Option<f64>,
  #[serde(rename(deserialize = "lon"))]
  pub lng: Option<f64>,
}

#[derive(serde::Deserialize, Debug)]
struct RawOverpassResponse {
    elements: Vec<OverpassResponseElement>,
}

pub type OverpassResponse = HashMap<u64, OverpassResponseElement>;


pub async fn query_overpass(area_name: &str, tag_name: &str, element_type: &str, config: &Config) -> reqwest::Result<OverpassResponse> {

  let contents = fs::read_to_string(QUERY_FILENAME).expect("Something went wrong reading the file");

  let reg = Handlebars::new();
  let request_body = reg.render_template(&contents, &json!({"area": area_name, "tag": tag_name, "type": element_type})).expect("Something went wrong formatting");

  let client = Client::new();
  let response = client.post(&config.REQUEST_URL).body(request_body).send()?;

  let raw_response: RawOverpassResponse = response.json()?;

  let mut response: OverpassResponse = HashMap::new();
  for element in raw_response.elements {
    response.insert(element.id, element);
  }
  // ToDo: Handle multipolygons
  Ok(response)
}

