//! # OpenHousePopulator
//!
//! This tool automatically distributes a given amount of inhabitants to osm buildings.
//! The calculation is based on predefined heuristics, calculating a flat count per building and randomly distributing people.

mod pbf;
pub mod geometry;
mod parser;

use pbf::{is_building, is_housenumber_node, load_buildings, Building, load_housenumbers, Buildings};

use std::fmt::Display;
use std::path::Path;

#[derive(Debug)]
pub enum Error {
    OverpassError(reqwest::Error),
    IOError(std::io::Error),
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::OverpassError(err) => write!(f, "failed to query overpass api: {}", err),
            Self::IOError(err) => write!(f, "io error occured: {}", err),
            _ => write!(f, "some error occured"),
        }
    }
}

impl std::error::Error for Error {}

#[derive(Clone, serde::Deserialize)]
pub struct Config {
    pub level_threshold: i32,
    pub reroll_threshold: i32,
    pub reroll_probability: i32,
    pub level_factor: i32,
    pub housenumber_factor: i32,
    pub request_url: String,
}

/// Takes pbf and inhabitants count and calculates geojson
pub fn spread_population(
    file: &Path,
    _inhabitants: &u64,
    _centroid: &bool,
    _config: &Config,
) -> Result<Buildings, Error> {
    // Read pbf file

    let r = std::fs::File::open(file).map_err(Error::IOError)?;
    let mut pbf = osmpbfreader::OsmPbfReader::new(r);

    let osm_buildings = pbf.get_objs_and_deps(is_building).unwrap();
    let osm_housenumbers = pbf.get_objs_and_deps(is_housenumber_node).unwrap();
    let mut buildings = load_buildings(osm_buildings);
    let housenumbers = load_housenumbers(osm_housenumbers);
    buildings = buildings.distribute_population(housenumbers, _inhabitants.clone(), _config);
    // println!("{:?}", buildings);
    Ok(buildings)
}
