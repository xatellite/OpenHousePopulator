use rand::prelude::*;
use geo::Polygon;
use geo::Contains;
use geo::Centroid;
use geojson::GeoJson;
use geojson::Feature;
use geojson::FeatureCollection;
use std::path::PathBuf;
use std::fs::File;
use std::io::Write;

use std::{collections::HashMap};

use crate::overpass::OverpassResponse;
use crate::Config;

fn prepare_polygons(buildings: &OverpassResponse) -> Vec<(Polygon, HashMap<String, String>)> {
  let mut building_polygons = vec![];
  // ToDo filter attributes
  // let single_home_list: [&str] = ["building", "building:levels", "building:flats", "leisure", "amenity", "emergency", "addr:housenumber", "housenum_inside"];
  for (_id, element) in buildings {
      if element.element_type == "way" {
          let coords: Vec<(f64,f64)> = element.nodes.clone().expect("Way without nodes").iter().map(|child| (buildings[child].lng.unwrap(), buildings[child].lat.unwrap())).collect();
          let line_string = geo::LineString::from(coords);
          building_polygons.push((Polygon::new(line_string, vec![]), element.tags.clone().unwrap_or(HashMap::new())));
      }
  }
  building_polygons
}

fn prepare_points(house_numbers: OverpassResponse) ->  Vec<(String, geo::Point)> {
  // Filter non house number entries
  house_numbers.iter()
  .filter(|(_, hn)| hn.lat.is_some() && hn.element_type == "node")
  .map(|(_, hn)| {
    if hn.tags.as_ref().unwrap().contains_key("addr:housenumber") {
      return (hn.tags.as_ref().unwrap()["addr:housenumber"].to_string(), geo::Point::new(hn.lng.unwrap(), hn.lat.unwrap()))
    }
    ("".to_string(), geo::Point::new(hn.lng.unwrap(), hn.lat.unwrap()))
  })
  .collect()
}

fn unfold_housenumber(house_number: String, mut house_numbers: Vec<String>) -> Vec<String> {
  let split_numbers: Vec<&str> = house_number.split(",").collect();
  split_numbers.iter().for_each(|house_number| {
    let house_number_range: Vec<&str> = house_number.split("-").collect();
    let mut house_numbers_to_add: Vec<String> = vec![];
    if house_number_range.len() > 2 {
      house_numbers_to_add.push(house_number.to_string());
    }
    else if house_number_range.len() > 1 {
      let first_number_part: Vec<&str> = house_number_range[0].split("/").collect();
      let first_number = first_number_part[first_number_part.len()].parse::<usize>().unwrap_or(0);

      let last_number_part: Vec<&str> = house_number_range[1].split("/").collect();
      let last_number = last_number_part[last_number_part.len()].parse::<usize>().unwrap_or(0);
      let range = first_number..last_number + 1;
      // ToDo Handle Range
      for item in range {
        house_numbers_to_add.push(item.to_string());
      }
    }
    else {
      house_numbers_to_add.push(house_number.to_string());
    }
    for house_number in house_numbers_to_add {
      if !house_numbers.contains(&house_number) {
        house_numbers.push(house_number);
      }
    }
  });
  house_numbers
}

fn count_points_inside(building_polygons: &mut Vec<(Polygon, HashMap<String, String>)>, house_number_points: Vec<(String, geo::Point)>) {
  for building_polygon in building_polygons {
    let mut house_numbers = vec![];
    if building_polygon.1.contains_key("addr:housenumber") {
      house_numbers = unfold_housenumber(building_polygon.1["addr:housenumber"].to_string(), house_numbers);
    }
    for (house_number, point) in house_number_points.iter() {
      if building_polygon.0.contains(point) {
        // Check house number not the same
        house_numbers = unfold_housenumber(house_number.to_string(), house_numbers);
      }
    };
    building_polygon.1.insert("housenumbers".to_string(), house_numbers.len().to_string());
  }
}

fn write_polygons_to_geojson(building_polygons: Vec<(Polygon, HashMap<String, String>)>, apply_centroid: bool) -> GeoJson {
  
  let mut features = vec![];

  for (geometry, tags) in building_polygons {
    let mut tags_map = serde_json::Map::new();
    for (key,value) in tags {
      tags_map.insert(key, serde_json::to_value(value).unwrap());
    }

    let geojson_geomentry;
    // ToDo: centroid
    if apply_centroid {
      let point = geometry.centroid().expect("not a centroid");
      geojson_geomentry = Some(geojson::Geometry::from(&point));
    } else {
      geojson_geomentry = Some(geojson::Geometry::from(&geometry));
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


pub fn count_inhabitants(buildings: OverpassResponse, house_numbers: OverpassResponse, mut inhabitants: u64, district: &str, apply_centroid: bool, config: &Config) -> std::io::Result<()>{

  let mut building_polygons = prepare_polygons(&buildings);
  let house_number_points = prepare_points(house_numbers);

  // Count house numbers inside building
  count_points_inside(&mut building_polygons, house_number_points);

  let single_home_list: [&str; 2] = ["house", "detached"];
  let apartment_list: [&str; 2] = ["apartments", "residential"];
  let unspecified_list: [&str; 2] = ["terrace", "semidetached_house"];
  let exclude_keys: [&str; 3] = ["leisure", "amenity", "emergency"];
  let out_keys: [&str; 3] = ["housenumbers", "flats", "pop"];
  let mut total_flats = 0;

  // Apply flat count to buildings
  for building_polygon in &mut building_polygons {
    let mut flat_count = 0;
    let house_numbers = building_polygon.1["housenumbers"].parse::<usize>().unwrap();
    if exclude_keys.iter().any(|key| building_polygon.1.contains_key(*key)) {
      building_polygon.1.insert("flats".to_string(), flat_count.to_string());
      total_flats += flat_count;
      continue;
    }
    if single_home_list.contains(&building_polygon.1["building"].as_str()) {
      flat_count = 1;
    }
    else if apartment_list.contains(&building_polygon.1["building"].as_str()){
      if building_polygon.1.contains_key("building:flats") {
        flat_count = building_polygon.1["building:flats"].parse::<usize>().unwrap();
        building_polygon.1.insert("flats".to_string(), flat_count.to_string());
        total_flats += flat_count;
        continue;
      } else if house_numbers >= 1 {
        flat_count = building_polygon.1["housenumbers"].parse::<usize>().unwrap() * config.HOUSENUMBER_FACTOR;
      } else {
        flat_count = 1;
      }
    }
    else if unspecified_list.contains(&building_polygon.1["building"].as_str()) {
      if building_polygon.1.contains_key("building:flats") {
        flat_count = building_polygon.1["building:flats"].parse::<usize>().unwrap();
        building_polygon.1.insert("flats".to_string(), flat_count.to_string());
        total_flats += flat_count;
        continue;
      } else if house_numbers >= 1 {
        flat_count = house_numbers;
      } else {
        flat_count = 1;
      }
    }
    else if building_polygon.1["building"] == "yes" && house_numbers >= 1 {
      // ToDo: Check not in commercial zone
      flat_count = house_numbers;
    }

    if building_polygon.1.contains_key("building:levels") {
      let levels: usize = building_polygon.1["building:levels"].parse::<usize>().unwrap();
      if levels > config.LEVEL_THRESHOLD {
        flat_count += building_polygon.1["building:levels"].parse::<usize>().unwrap() - 4;
        flat_count *= config.LEVEL_FACTOR;
      }
    }

    building_polygon.1.insert("flats".to_string(), flat_count.to_string());
    total_flats += flat_count;
  }

  if total_flats < 1 {
    panic!("no flats found in area")
  }
  // Distribute inhabitants to flats
  let mut flat_inhabitants = vec![0; total_flats];

  while inhabitants > 0 {
    let flat_index: usize = rand::thread_rng().gen_range(0..total_flats - 1);
    if flat_inhabitants[flat_index] > config.REROLL_THRESHOLD && rand::thread_rng().gen_range(0..config.REROLL_PROBABILITY) > config.REROLL_THRESHOLD {
      continue;
    }
    flat_inhabitants[flat_index] += 1;
    inhabitants -= 1;
  }
  
  let mut flat_list_iter_index = 0;
  for building_polygon in &mut building_polygons {
    let flats_in_building = building_polygon.1["flats"].parse::<usize>().unwrap();
    
    let mut house_inhabitants = 0;
    if flats_in_building > 0 {
      for flat_list_roll_index in flat_list_iter_index..(flat_list_iter_index + flats_in_building) {
        house_inhabitants += flat_inhabitants[flat_list_roll_index];
      }
      flat_list_iter_index += flats_in_building;
    }
    building_polygon.1.insert("pop".to_string(), house_inhabitants.to_string());
  }

  // ToDo: Make filter optional
  for building_polygon in &mut building_polygons {
    let mut filtered_tags: HashMap<String, String> = HashMap::new();
    for (key, value) in &building_polygon.1 {
      if out_keys.contains(&key.as_str()) {
        filtered_tags.insert(key.to_string(), value.to_string());
      }
    }
    building_polygon.1 = filtered_tags;
  }

  let geojson = write_polygons_to_geojson(building_polygons, apply_centroid);
  
  // Create a temporary file.
  let temp_directory = PathBuf::from("../../out/");
  let file_name = district.to_string() + ".geojson";
  let temp_file = temp_directory.join(&file_name);

  let mut file = File::create(temp_file).unwrap();
  write!(file, "{}", geojson.to_string())?;

  Ok(())
}
