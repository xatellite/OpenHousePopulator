use geo::Centroid;
use geo::Contains;
use geo::Point;
use geo::Polygon;
use osmpbfreader::Node;
use osmpbfreader::NodeId;
use osmpbfreader::OsmId;
use osmpbfreader::OsmObj;
use osmpbfreader::Tags;
use osmpbfreader::Way;
use rand::prelude::Distribution;
use rand::Rng;
use statrs::distribution::Categorical;
use std::collections::BTreeMap;
use std::fmt::Display;

use crate::config::Config;
use crate::parser::housenumber::HouseNumberList;

#[derive(Debug, PartialEq, Clone)]
pub enum GenericGeometry {
    GenericPolygon(Polygon),
    GenericPoint(Point),
}

pub struct GenericWay {
    pub polygon: Polygon,
    pub tags: Tags,
}

impl GenericWay {
    /// Gets the number of house numbers in the area
    fn calculate_house_number_count(&self, house_number_points: &[HouseNumberPoint]) -> usize {
        // Count house numbers of way (tags)
        let mut house_numbers = self
            .tags
            .get("addr:housenumber")
            .and_then(|housenumber| HouseNumberList::try_from(housenumber.as_str()).ok())
            .unwrap_or_default();

        // Count house numbers beeing positioned inside area (of way)
        house_number_points
            .iter()
            .filter(|house_number| self.polygon.contains(&house_number.point))
            .for_each(|house_number| {
                house_numbers.merge(
                    HouseNumberList::try_from(house_number.text.as_str()).unwrap_or_default(),
                );
            });

        house_numbers.count()
    }

    /// Calculate number of flats inside building by tags
    fn calculate_flat_count(&self, house_numbers: usize, config: &Config) -> usize {
        // If flat count is defined in tags, this is applied
        if self.tags.contains_key("building:flats") {
            let flat_count = self.tags["building:flats"].parse::<usize>().unwrap();
            return flat_count;
        }

        // If its a single home house, return 1
        if config
            .single_home_list
            .contains(&self.tags["building"].to_string())
        {
            return 1;
        }

        // Otherwise estimate flat count by building type
        let mut flat_count: usize = 0;
        if config
            .apartment_list
            .contains(&self.tags["building"].to_string())
            || config
                .unspecified_list
                .contains(&self.tags["building"].to_string())
        {
            if house_numbers >= 1 {
                flat_count = house_numbers * config.housenumber_factor;
            } else {
                flat_count = 4;
            }
        } else if self.tags["building"] == "yes" && house_numbers >= 1 {
            flat_count = house_numbers;
        }

        // Increase flat count by building levels if specified
        if self.tags.contains_key("building:levels") {
            let levels = self.tags["building:levels"]
                .parse::<f32>()
                .map_err(|err| {
                    println!(
                        "Error: {:?} on value {:?}",
                        err, self.tags["building:levels"]
                    );
                    0
                })
                .unwrap()
                .floor() as usize;
            flat_count = flat_count * levels * config.level_factor;
        }

        flat_count
    }

    /// Estimates number of flats inside building
    pub fn calculate_building_metrics(
        &self,
        house_number_points: &[HouseNumberPoint],
        config: &Config,
    ) -> Building {
        let house_number_count = self.calculate_house_number_count(house_number_points);
        let flat_count = self.calculate_flat_count(house_number_count, config);
        Building {
            geometry: GenericGeometry::GenericPolygon(self.polygon.clone()),
            flats: flat_count,
            pop: 0,
        }
    }

    fn contains(&self, geometry: &GenericGeometry) -> bool {
        match geometry {
            GenericGeometry::GenericPolygon(polygon) => self.polygon.contains(polygon),
            GenericGeometry::GenericPoint(point2) => self.polygon.contains(point2),
        }
    }
}

pub struct HouseNumberPoint {
    point: Point,
    text: String,
}

pub struct Buildings(pub(crate) Vec<Building>);

impl IntoIterator for Buildings {
    type Item = Building;

    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl FromIterator<Building> for Buildings {
    fn from_iter<T: IntoIterator<Item = Building>>(iter: T) -> Self {
        Buildings(iter.into_iter().collect())
    }
}

impl From<(Vec<GenericWay>, &Vec<HouseNumberPoint>, &Config)> for Buildings {
    fn from(item: (Vec<GenericWay>, &Vec<HouseNumberPoint>, &Config)) -> Self {
        Buildings(
            item.0
                .into_iter()
                .map(|way| Building::from((way, item.1, item.2)))
                .collect(),
        )
    }
}

impl Buildings {
    /// Distributes a known population to buildings
    pub fn distribute_population(&mut self, inhabitants_total: u64, config: &Config) {
        // Gather total flat count
        let total_flat_count: usize = self.0.iter().map(|building| building.flats).sum();

        // Distribute population
        let mut flat_inhabitants: Vec<u64> = vec![0; total_flat_count];
        let mut inhabitants_to_distribute = inhabitants_total;
        while inhabitants_to_distribute > 0 {
            let flat_index = rand::thread_rng().gen_range(0..total_flat_count - 1);
            if flat_inhabitants[flat_index] > config.reroll_threshold
                && rand::thread_rng().gen_range(0..config.reroll_probability)
                    > config.reroll_threshold.try_into().unwrap()
            {
                continue;
            }
            flat_inhabitants[flat_index] += 1;
            inhabitants_to_distribute -= 1;
        }

        // Add population tag to buildings
        let mut flat_offset = 0;
        self.0.iter_mut().for_each(|building| {
            let mut building = building;
            let flat_count = building.flats;
            let mut population: u64 = 0;
            for flat_inhabitant_count in flat_inhabitants.iter().skip(flat_offset).take(flat_count)
            {
                population += flat_inhabitant_count;
            }
            flat_offset += flat_count;
            building.pop += population;
        });
    }

    /// Estimates the population of buildings by applying german household sizes by occurrence probability
    pub fn estimate_population(&mut self) {
        let dist = Categorical::new(&[0.20737853, 0.33310260, 0.17661846, 0.18911436, 0.09378605])
            .unwrap();
        self.0.iter_mut().for_each(|building| {
            building.pop = dist
                .clone()
                .sample_iter(&mut rand::thread_rng())
                .take(building.flats)
                .map(|p| (p as u64) + 1)
                .sum();
        })
    }

    pub fn into_inner(self) -> Vec<Building> {
        self.0
    }

    pub fn iter(&self) -> core::slice::Iter<'_, Building> {
        self.0.iter()
    }

    pub(crate) fn exclude_in(mut self, area: &[GenericWay]) -> Self {
        self.0
            .retain(|building| !area.iter().any(|area| area.contains(&building.geometry)));
        self
    }

    pub(crate) fn centroid(&mut self) {
        self.0.iter_mut().for_each(|building| {
            building.centroid();
        });
    }
}

/// A building is a area or point with information about estimated flats and population
#[derive(Debug, PartialEq, Clone)]
pub struct Building {
    pub geometry: GenericGeometry,
    pub flats: usize,
    pub pop: u64,
}

impl From<(GenericWay, &Vec<HouseNumberPoint>, &Config)> for Building {
    fn from(item: (GenericWay, &Vec<HouseNumberPoint>, &Config)) -> Self {
        item.0.calculate_building_metrics(item.1, item.2)
    }
}

impl Display for Building {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Building {{ polygon: {:?}, flats: {:?}, population: {:?} }}",
            self.geometry, self.flats, self.pop
        )
    }
}

impl Building {
    pub fn centroid(&mut self) {
        match &self.geometry {
            GenericGeometry::GenericPolygon(polygon) => {
                self.geometry = GenericGeometry::GenericPoint(polygon.centroid().unwrap());
            }
            GenericGeometry::GenericPoint(_) => {}
        }
    }
}

/// Check if osm obj is building
pub(crate) fn is_building(obj: &osmpbfreader::OsmObj) -> bool {
    (obj.is_node() || obj.is_way()) && obj.tags().contains_key("building")
}

/// Check if osm obj is housenumber
pub(crate) fn is_housenumber_node(obj: &osmpbfreader::OsmObj) -> bool {
    obj.is_node() && obj.tags().contains_key("addr:housenumber")
}

/// Check if osm obj is part of the exclude areas
pub(crate) fn is_exclude_area(obj: &osmpbfreader::OsmObj, config: &Config) -> bool {
    obj.is_way()
        && ((obj.tags().contains_key("landuse")
            && config
                .exclude_landuse
                .contains(&obj.tags()["landuse"].to_string()))
            || config
                .exclude_tags
                .iter()
                .any(|exclude_tag| obj.tags().contains_key(exclude_tag.as_str())))
}

pub(crate) fn load_ways(osm_buildings: BTreeMap<OsmId, OsmObj>) -> Vec<GenericWay> {
    // Extract buildings and all nodes
    let osm_building_ways: Vec<Way> = osm_buildings
        .clone()
        .into_iter()
        .filter_map(|(_, obj)| match obj {
            OsmObj::Way(inner) => Some(inner),
            _ => None,
        })
        .collect();

    let osm_building_nodes: BTreeMap<NodeId, Node> = osm_buildings
        .into_iter()
        .filter_map(|(_, obj)| match obj {
            OsmObj::Node(inner) => Some(inner),
            _ => None,
        })
        .map(|node| (node.id, node))
        .collect();

    // Create geometry for buildings
    osm_building_ways
        .into_iter()
        .map(|obj| {
            let coords: Vec<(f64, f64)> = obj
                .nodes
                .iter()
                .map(|node_id| {
                    (
                        osm_building_nodes[node_id].decimicro_lon as f64 / 10000000.,
                        osm_building_nodes[node_id].decimicro_lat as f64 / 10000000.,
                    )
                })
                .collect();
            let line_string = geo::LineString::from(coords);
            let polygon = Polygon::new(line_string, vec![]); // Make to confex hull to make centroid
            GenericWay {
                polygon,
                tags: obj.tags,
            }
        })
        .collect()
}

pub(crate) fn load_housenumbers(
    osm_housenumbers: BTreeMap<OsmId, OsmObj>,
) -> Vec<HouseNumberPoint> {
    let osm_housenumber_nodes: BTreeMap<NodeId, Node> = osm_housenumbers
        .into_iter()
        .filter_map(|(_, obj)| match obj {
            OsmObj::Node(inner) => Some(inner),
            _ => None,
        })
        .map(|node| (node.id, node))
        .collect(); // TODO: How to get rid of this map function?

    osm_housenumber_nodes
        .values()
        .map(|obj| {
            let point = Point::new(
                obj.decimicro_lat as f64 / 10000000.,
                obj.decimicro_lon as f64 / 10000000.,
            );
            let text = obj.tags["addr:housenumber"].to_string();
            HouseNumberPoint { point, text }
        })
        .collect()
}
