use std::collections::HashMap;

use geo::Polygon;

use crate::overpass::OverpassResponse;

pub fn prepare_polygons_from_way(overpass_response: &OverpassResponse) -> Vec<(Polygon, HashMap<String, String>)> {
  let mut polygons = vec![];
  // ToDo filter attributes?!
  // let single_home_list: [&str] = ["building", "building:levels", "building:flats", "leisure", "amenity", "emergency", "addr:housenumber", "housenum_inside"];
  for (_id, element) in overpass_response {
      if element.element_type == "way" {
          let coords: Vec<(f64,f64)> = element
            .nodes.clone().expect("Way without nodes")
            .iter().map(|child| (overpass_response[child].lng.unwrap(), overpass_response[child].lat.unwrap()))
            .collect();
          let line_string = geo::LineString::from(coords);
          polygons.push((Polygon::new(line_string, vec![]), element.tags.clone().unwrap_or(HashMap::new())));
      }
  }
  polygons
}

pub fn prepare_polygons_from_relation(overpass_response: &OverpassResponse) -> Vec<(Polygon, HashMap<String, String>)> {
  let mut polygons = vec![];
  for (_id, element) in overpass_response {
      if element.element_type == "relation" {
          let coords: Vec<(f64,f64)> = element
            .members.clone().expect("Relation without members")
            .iter()
            .map(|member| member.member_ref)
            .map(|child| (overpass_response[&child].lng.unwrap(), overpass_response[&child].lat.unwrap()))
            .collect();
          let line_string = geo::LineString::from(coords);
          polygons.push((Polygon::new(line_string, vec![]), element.tags.clone().unwrap_or(HashMap::new())));
      }
  }
  polygons
}

