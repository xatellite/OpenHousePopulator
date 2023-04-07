//! # OpenHousePopulator
//!
//! This tool automatically distributes a given amount of inhabitants to osm buildings.
//! The calculation is based on predefined heuristics, calculating a flat count per building and randomly distributing people.

mod config;
pub mod geometry;
mod parser;
mod pbf;

use osmpbfreader::OsmPbfReader;
use pbf::{
    is_building, is_exclude_area, is_housenumber_node, load_housenumbers, load_ways, Buildings,
};

use std::fmt::Display;

pub use crate::config::Config;
pub use crate::pbf::{Building, GenericGeometry};

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
        }
    }
}

impl std::error::Error for Error {}

/// Calculates the population of houses in a given pbf
pub fn calculate_buildings<T: std::io::Read + std::io::Seek>(
    pbf: &mut OsmPbfReader<T>,
    centroid: bool,
    config: &Config,
) -> Result<Buildings, Error> {
    // Retrieve objects from pbf
    log::info!("Loading objects from pbf...");
    let osm_buildings = pbf.get_objs_and_deps(is_building).unwrap();
    let osm_housenumbers = pbf.get_objs_and_deps(is_housenumber_node).unwrap();
    let osm_exclude_areas = pbf
        .get_objs_and_deps(|obj| is_exclude_area(obj, config))
        .unwrap();

    log::info!("Loading ways...");
    let building_ways = load_ways(osm_buildings);
    log::info!("Loading housenumbers...");
    let housenumbers = load_housenumbers(osm_housenumbers);
    log::info!("Creating buildings...");
    let mut buildings = Buildings::from((building_ways, &housenumbers, config));
    log::info!("Loading exclude areas...");
    let areas = load_ways(osm_exclude_areas);
    if centroid {
        log::info!("Calculating centroids...");
        buildings.centroid();
    }
    log::info!("Exclude areas...");
    buildings = buildings.exclude_in(&areas);
    log::info!("Distributing population...");

    Ok(buildings)
}
