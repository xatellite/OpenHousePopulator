mod overpass;
mod populator;
mod geometry;

use std::collections::HashMap;

use futures::executor::block_on;

use crate::geometry::prepare_polygons_from_relation;

#[derive(Clone, serde::Deserialize)]
pub struct Config {
    pub level_threshold: usize,
    pub reroll_threshold: usize,
    pub reroll_probability: usize,
    pub level_factor: usize,
    pub housenumber_factor: usize,
    pub request_url: String,
}

pub fn get_area_by_point(lat: &f32, lng: &f32, config: &Config) -> Result<HashMap<String, u32>, ()> {
    let areas = match block_on(overpass::query_overpass_point(lat, lng, "admin_level=8", "area", config)) {
        Ok(result) => result,
        Err(error) => panic!("{}", error)
    };

    print!("{:?}\n", areas.len());

    let polygons = prepare_polygons_from_relation(&areas);
    print!("{:?}\n", polygons.len());
    print!("{:?}\n", polygons);

    // Collect result hashmap
    let mut result_map = HashMap::new();


    for (_, area) in areas.iter() {
        if area.tags.is_some() {
            let keys = area.tags.as_ref().unwrap();
            if keys.contains_key("admin_level") &&  keys.contains_key("name"){
                result_map.insert(keys["name"].clone(), keys["admin_level"].parse::<u32>().unwrap());
                // Save outline to file

            }
        }
    }

    // return result
    Ok(result_map)
}

pub fn spread_population(district: &str, inhabitants: u64, centroid: bool, config: &Config) -> Result<(), std::io::Error> {
  let buildings = match block_on(overpass::query_overpass_elements(district, "building", "area", config)) {
      Ok(result) => result,
      Err(error) => panic!("{}", error)
  };

  let house_numbers = match block_on(overpass::query_overpass_elements(district, "addr:housenumber", "node", config)) {
      Ok(result) => result,
      Err(error) => panic!("{}", error)
  };

  return populator::count_inhabitants(buildings, house_numbers, inhabitants, district, centroid, config);
}