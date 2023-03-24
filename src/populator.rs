use geo::Contains;
use geo::Polygon;
use rand::prelude::*;
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;

use serde_json::Value;
use std::collections::HashMap;

use crate::geometry::{prepare_polygons_from_way, write_polygons_to_geojson};
use crate::overpass::OverpassResponse;
use crate::Config;

// TODO: New function: distribute_inhabitants


pub fn count_inhabitants(
    buildings: OverpassResponse,
    house_numbers: OverpassResponse,
    mut inhabitants: u64,
    district: &str,
    apply_centroid: bool,
    config: &Config,
) -> std::io::Result<()> {
    let mut building_informations = prepare_polygons_from_way(&buildings);
    let house_number_points = prepare_points(house_numbers);

    // Count house numbers inside building
    let enriched_building_polygons =
        count_points_inside(&mut building_informations, house_number_points);

    let single_home_list: [&str; 2] = ["house", "detached"];
    let apartment_list: [&str; 2] = ["apartments", "residential"];
    let unspecified_list: [&str; 2] = ["terrace", "semidetached_house"];
    let exclude_keys: [&str; 3] = ["leisure", "amenity", "emergency"];
    let out_keys: [&str; 3] = ["housenumbers", "flats", "pop"];

    let mut flat_list = vec![];
    // Apply flat count to buildings
    for building_polygon in &enriched_building_polygons {
        let mut flat_count: usize = 0;
        let house_numbers = building_polygon.2;
        if exclude_keys
            .iter()
            .any(|key| building_polygon.1.contains_key(*key))
        {
            flat_list.push(flat_count);
            continue;
        }
        if single_home_list.contains(&building_polygon.1["building"].as_str()) {
            flat_count = 1;
        } else if apartment_list.contains(&building_polygon.1["building"].as_str()) {
            if building_polygon.1.contains_key("building:flats") {
                flat_count = building_polygon.1["building:flats"]
                    .parse::<usize>()
                    .unwrap();
                flat_list.push(flat_count);
                continue;
            } else if house_numbers >= 1 {
                flat_count = building_polygon.2 * config.housenumber_factor;
            } else {
                flat_count = 1;
            }
        } else if unspecified_list.contains(&building_polygon.1["building"].as_str()) {
            if building_polygon.1.contains_key("building:flats") {
                flat_count = building_polygon.1["building:flats"]
                    .parse::<usize>()
                    .unwrap();
                flat_list.push(flat_count);
                continue;
            } else if house_numbers >= 1 {
                flat_count = house_numbers;
            } else {
                flat_count = 1;
            }
        } else if building_polygon.1["building"] == "yes" && house_numbers >= 1 {
            // ToDo: Check not in commercial zone
            flat_count = house_numbers;
        }

        if building_polygon.1.contains_key("building:levels") {
            match building_polygon.1["building:levels"].parse::<usize>() {
                Ok(levels) => {
                    if levels > config.level_threshold {
                        flat_count += building_polygon.1["building:levels"]
                            .parse::<usize>()
                            .unwrap()
                            - 4;
                        flat_count *= config.level_factor;
                    }
                }
                Err(error) => print!(
                    "Parsing failed for {:?}",
                    building_polygon.1["building:levels"]
                ),
            }
        }

        flat_list.push(flat_count);
    }

    let total_flats: usize = flat_list.iter().sum();
    if total_flats < 1 {
        panic!("no flats found in area")
    }
    // Distribute inhabitants to flats
    let mut flat_inhabitants = vec![0; total_flats];

    while inhabitants > 0 {
        let flat_index = rand::thread_rng().gen_range(0..total_flats - 1);
        if flat_inhabitants[flat_index] > config.reroll_threshold
            && rand::thread_rng().gen_range(0..config.reroll_probability) > config.reroll_threshold
        {
            continue;
        }
        flat_inhabitants[flat_index] += 1;
        inhabitants -= 1;
    }

    let mut flat_list_iter_index = 0;
    let mut population = vec![];
    for flats_in_building in flat_list.clone() {
        let mut house_inhabitants = 0;
        if flats_in_building > 0 {
            for flat_list_roll_index in
                flat_list_iter_index..(flat_list_iter_index + flats_in_building)
            {
                house_inhabitants += flat_inhabitants[flat_list_roll_index];
            }
            flat_list_iter_index += flats_in_building;
        }
        population.push(house_inhabitants);
    }

    let mut index: usize = 0;
    let polygon_information = building_informations
        .into_iter()
        .map(|building| {
            let mut result_hashmap: HashMap<String, Value> = HashMap::new();
            result_hashmap.insert("pop".to_string(), population[index].into());
            result_hashmap.insert("flats".to_string(), flat_list[index].into());
            index += 1;
            (building.0, result_hashmap)
        })
        .collect();

    let geojson = write_polygons_to_geojson(&polygon_information, apply_centroid);

    // Create a temporary file.
    let temp_directory = PathBuf::from("./out/");
    let file_name = district.to_string() + ".geojson";
    let temp_file = temp_directory.join(&file_name);

    let mut file = File::create(temp_file).unwrap();
    write!(file, "{}", geojson.to_string())?;

    Ok(())
}
