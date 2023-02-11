use reqwest::blocking::Client;
use handlebars::Handlebars;
use serde_json::json;
use std::{fs, collections::HashMap};

use crate::Config;

pub const ELEMENTS_QUERY_FILENAME: &'static str = "./res/elements_query.ovql";
pub const POINT_QUERY_FILENAME: &'static str = "./res/point_query.ovql";

#[derive(serde::Deserialize, Debug, Clone)]
pub struct OverpassMember {
  #[serde(rename(deserialize = "type"))]
  pub member_type: String,
  #[serde(rename(deserialize = "ref"))]
  pub member_ref: u64,
  pub role: String
}

// ToDo: Separate all Overpass types
#[derive(serde::Deserialize, Clone, Debug)]
pub struct OverpassResponseElement {
  #[serde(rename(deserialize = "type"))]
  pub element_type: String,
  pub nodes: Option<Vec<u64>>,
  pub members: Option<Vec<OverpassMember>>,
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

pub async fn query_overpass_elements(area_name: &str, tag_name: &str, element_type: &str, config: &Config) -> reqwest::Result<OverpassResponse> {
  let contents = fs::read_to_string(ELEMENTS_QUERY_FILENAME).expect("Query file could not be read");
  let reg = Handlebars::new();
  let request_body = reg.render_template(&contents, &json!({"area": area_name, "tag": tag_name, "type": element_type})).expect("Something went wrong formatting");

  let response = send_overpass_request(request_body, config).await
    .expect("Overpass response could not be retrieved.");
  Ok(response)
}

pub async fn query_overpass_point(lat: &f32, lng: &f32, tag_name: &str, element_type: &str, config: &Config) -> reqwest::Result<OverpassResponse> {
  let contents = fs::read_to_string(POINT_QUERY_FILENAME).expect("Query file could not be read");
  let reg = Handlebars::new();

  let request_body = reg.render_template(&contents, &json!({"lng": lng.to_string(), "lat": lat.to_string(), "tag": tag_name, "type": element_type})).expect("Something went wrong formatting");

  print!("{}", request_body);
  let response = send_overpass_request(request_body, config).await
    .expect("Overpass response could not be retrieved.");
  Ok(response)
}

pub async fn send_overpass_request(request_body: String, config: &Config) -> reqwest::Result<OverpassResponse> {
  let client = Client::new();
  let response = client.post(&config.request_url).body(request_body).send().expect("Overpass endpoint did not respond").error_for_status()?;

  let raw_response: RawOverpassResponse = response.json().expect("JSON parsing failed");

  // ToDo: Handle multipolygons
  let mut response: OverpassResponse = HashMap::new();
  for element in raw_response.elements {
    response.insert(element.id, element);
  }
  Ok(response)
}
