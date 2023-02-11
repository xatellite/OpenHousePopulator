use std::collections::HashMap;

use geo::ConvexHull;
use geo::Polygon;
use geo::Centroid;
use geojson::GeoJson;
use geojson::Feature;
use geojson::FeatureCollection;

use crate::overpass::OverpassResponse;

fn collect_coordinates(overpass_response: &OverpassResponse, node_list: Vec<u64>) -> Vec<(f64,f64)>{
  let coords: Vec<(f64,f64)> =  node_list
    .iter()
    .map(|child| (overpass_response[child].lng.unwrap(), overpass_response[child].lat.unwrap()))
    .collect();
  coords
}

pub fn prepare_polygons_from_way(overpass_response: &OverpassResponse) -> Vec<(Polygon, HashMap<String, String>)> {
  let mut polygons = vec![];
  // ToDo filter attributes?!
  // let single_home_list: [&str] = ["building", "building:levels", "building:flats", "leisure", "amenity", "emergency", "addr:housenumber", "housenum_inside"];
  for (_id, element) in overpass_response {
      if element.element_type == "way" {
          let coords: Vec<(f64,f64)> = collect_coordinates(overpass_response, element.nodes.clone().expect("Way without nodes"));
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
          let resolved_cords: Vec<Vec<(f64,f64)>> = element
            .members.clone().expect("Relation without members")
            .iter()
            .map(|member|
              collect_coordinates( overpass_response, overpass_response[&member.member_ref].nodes.clone().unwrap())
            ).collect();
          let coords: Vec<(f64,f64)> = resolved_cords.into_iter().flatten().collect();
          let line_string = geo::LineString::from(coords);
          polygons.push((Polygon::new(line_string, vec![]).convex_hull(), element.tags.clone().unwrap_or(HashMap::new())));
      }
  }
  polygons
}

pub fn write_polygons_to_geojson(building_polygons: &Vec<(Polygon, HashMap<String, String>)>, apply_centroid: bool) -> GeoJson {
  
  let mut features = vec![];

  for (geometry, tags) in building_polygons {
    let mut tags_map = serde_json::Map::new();
    for (key,value) in tags {
      tags_map.insert(key.clone(), serde_json::to_value(value).unwrap());
    }

    let geojson_geomentry;

    if apply_centroid {
      let point = geometry.centroid().expect("not a centroid");
      geojson_geomentry = Some(geojson::Geometry::from(&point));
    } else {
      geojson_geomentry = Some(geojson::Geometry::from(geometry));
    }

    let feature = Feature {
      bbox: None,
      geometry: geojson_geomentry,
      id: None,
      // See the next section about Feature properties
      properties: Some(tags_map),
      foreign_members: None,
    };
    features.push(feature);
  }

  GeoJson::from(FeatureCollection {
    bbox: None,
    features: features,
    foreign_members: None,
  })
}
