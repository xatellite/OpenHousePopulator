use geo::Centroid;
use geo::Contains;
use geo::Intersects;
use geo::Point;
use geo::Polygon;
use indicatif::{ProgressBar, ProgressIterator, ProgressStyle};
use osmpbfreader::Node;
use osmpbfreader::NodeId;
use osmpbfreader::OsmId;
use osmpbfreader::OsmObj;
use osmpbfreader::Relation;
use osmpbfreader::Tags;
use osmpbfreader::Way;
use rand::prelude::Distribution;
use rand::Rng;
use serde::Deserialize;
use serde::Serialize;
use statrs::distribution::Categorical;
use std::collections::BTreeMap;
use std::fmt::Display;

fn progress_bar(len: u64, msg: &'static str) -> ProgressBar {
    let pb = ProgressBar::new(len);
    pb.set_message(msg);
    pb.set_style(
        ProgressStyle::with_template(
            "{msg}: [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} ({eta})",
        )
        .unwrap()
        .progress_chars("=> "),
    );
    pb
}

use crate::config::Config;
use crate::parser::housenumber::HouseNumberList;

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub enum GenericGeometry {
    GenericPolygon(Polygon),
    GenericPoint(Point),
}

pub struct GenericWay {
    pub id: i64,
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
            .filter(|house_number| self.polygon.intersects(&house_number.point))
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
                .unwrap_or(0.0)
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
            osm_id: self.id,
            building_class: self.tags.get("building").map(|s| s.to_string()).unwrap_or_default(),
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

#[derive(Debug, Serialize, Deserialize, Clone)]
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
        let pb = progress_bar(item.0.len() as u64, "Processing buildings");
        Buildings(
            item.0
                .into_iter()
                .progress_with(pb)
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
        let pb = progress_bar(inhabitants_total, "Distributing population");
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
            pb.inc(1);
        }
        pb.finish_and_clear();

        // Add population tag to buildings
        let mut flat_offset = 0;
        self.0.iter_mut().for_each(|building| {
            let building = building;
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
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct Building {
    pub osm_id: i64,
    pub building_class: String,
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
                id: obj.id.0,
                polygon,
                tags: obj.tags,
            }
        })
        .collect()
}

/// Identifies which OSM object defines the area boundary to filter buildings by
#[derive(Debug, Clone)]
pub enum AreaId {
    Way(i64),
    Relation(i64),
}

pub(crate) fn is_area_way(obj: &OsmObj, way_id: i64) -> bool {
    matches!(obj, OsmObj::Way(w) if w.id.0 == way_id)
}

pub(crate) fn is_area_relation(obj: &OsmObj, relation_id: i64) -> bool {
    matches!(obj, OsmObj::Relation(r) if r.id.0 == relation_id)
}

/// Build a polygon from a single OSM way (used as area boundary)
pub(crate) fn load_area_way_polygon(
    objs: &BTreeMap<OsmId, OsmObj>,
    way_id: i64,
) -> Option<Polygon> {
    let way = objs.values().find_map(|obj| match obj {
        OsmObj::Way(w) if w.id.0 == way_id => Some(w),
        _ => None,
    })?;
    let nodes = collect_nodes(objs);
    way_nodes_to_polygon(&way.nodes, &nodes)
}

/// Build a polygon from an OSM relation (multipolygon boundary)
pub(crate) fn load_area_relation_polygon(
    objs: &BTreeMap<OsmId, OsmObj>,
    relation_id: i64,
) -> Option<Polygon> {
    let relation: &Relation = objs.values().find_map(|obj| match obj {
        OsmObj::Relation(r) if r.id.0 == relation_id => Some(r),
        _ => None,
    })?;

    let outer_way_ids: Vec<i64> = relation
        .refs
        .iter()
        .filter(|r| r.role == "outer" || r.role.is_empty())
        .filter_map(|r| match r.member {
            OsmId::Way(wid) => Some(wid.0),
            _ => None,
        })
        .collect();

    let nodes = collect_nodes(objs);

    let ways_node_ids: Vec<Vec<NodeId>> = outer_way_ids
        .iter()
        .filter_map(|wid| {
            objs.values().find_map(|obj| match obj {
                OsmObj::Way(w) if w.id.0 == *wid => Some(w.nodes.clone()),
                _ => None,
            })
        })
        .collect();

    if ways_node_ids.is_empty() {
        return None;
    }

    let stitched = stitch_ways(ways_node_ids);
    way_nodes_to_polygon(&stitched, &nodes)
}

fn collect_nodes(objs: &BTreeMap<OsmId, OsmObj>) -> BTreeMap<NodeId, &Node> {
    objs.values()
        .filter_map(|obj| match obj {
            OsmObj::Node(n) => Some((n.id, n)),
            _ => None,
        })
        .collect()
}

fn way_nodes_to_polygon(node_ids: &[NodeId], nodes: &BTreeMap<NodeId, &Node>) -> Option<Polygon> {
    let coords: Vec<(f64, f64)> = node_ids
        .iter()
        .filter_map(|nid| nodes.get(nid))
        .map(|n| {
            (
                n.decimicro_lon as f64 / 10_000_000.0,
                n.decimicro_lat as f64 / 10_000_000.0,
            )
        })
        .collect();
    if coords.len() < 3 {
        return None;
    }
    Some(Polygon::new(geo::LineString::from(coords), vec![]))
}

/// Stitch a list of ordered node-ID sequences into one closed ring
fn stitch_ways(ways: Vec<Vec<NodeId>>) -> Vec<NodeId> {
    if ways.is_empty() {
        return vec![];
    }
    if ways.len() == 1 {
        return ways.into_iter().next().unwrap();
    }

    let mut result: Vec<NodeId> = ways[0].clone();
    let mut remaining: Vec<Vec<NodeId>> = ways[1..].to_vec();

    while !remaining.is_empty() {
        let last = *result.last().unwrap();
        if let Some(pos) = remaining
            .iter()
            .position(|w| w.first() == Some(&last) || w.last() == Some(&last))
        {
            let mut next = remaining.remove(pos);
            if next.last() == Some(&last) {
                next.reverse();
            }
            result.extend_from_slice(&next[1..]);
        } else {
            break;
        }
    }

    result
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
                obj.decimicro_lon as f64 / 10000000.,
                obj.decimicro_lat as f64 / 10000000.,
            );
            let text = obj.tags["addr:housenumber"].to_string();
            HouseNumberPoint { point, text }
        })
        .collect()
}
