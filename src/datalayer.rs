use geo::Contains;
use geo::ConvexHull;
use geo::Point;
use geo::Polygon;
use osmpbfreader::Node;
use osmpbfreader::NodeId;
use osmpbfreader::OsmId;
use osmpbfreader::OsmObj;
use osmpbfreader::Tags;
use osmpbfreader::Way;
use std::collections::BTreeMap;
use std::fmt::Display;

use crate::parser::housenumber::HouseNumberList;

struct HouseNumberPoint {
    point: Point,
    text: String,
}

#[derive(Debug)]
pub struct Building {
    polygon: Polygon,
    tags: Tags,
}

impl Building {
    fn calculate_house_number_count(mut self, house_number_points: &Vec<HouseNumberPoint>) -> Self {
        let mut house_numbers = HouseNumberList::new();
        if self.tags.contains_key("addr:housenumber") {
            house_numbers = match HouseNumberList::try_from(self.tags["addr:housenumber"].as_str())
            {
                Ok(house_number) => house_number,
                Err(err) => {
                    println!("Error: {:?}", err);
                    HouseNumberList::new()
                }
            };
        }
        for house_number in house_number_points {
            if self.polygon.contains(&house_number.point) {
                // Check house number not the same
                house_numbers.merge(
                    HouseNumberList::try_from(house_number.text.as_str()).unwrap_or_default(),
                );
            }
        }
        self.tags.insert(
            "addr:housenumber_count".to_string().into(),
            house_numbers.count().to_string().into(),
        );
        self
    }
}

impl Display for Building {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Building {{ polygon: {:?}, tags: {:?} }}",
            self.polygon, self.tags
        )
    }
}

/// Check if osm obj is building
pub fn is_building(obj: &osmpbfreader::OsmObj) -> bool {
    (obj.is_node() || obj.is_way()) && obj.tags().contains_key("building")
}

/// Check if osm obj is building
pub fn is_housenumber_node(obj: &osmpbfreader::OsmObj) -> bool {
    obj.is_node() && obj.tags().contains_key("addr:housenumber")
}

pub fn load_buildings(
    osm_buildings: BTreeMap<OsmId, OsmObj>,
    osm_housenumber: BTreeMap<OsmId, OsmObj>,
) -> Vec<Building> {
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

    let osm_housenumber_nodes: BTreeMap<NodeId, Node> = osm_housenumber
        .into_iter()
        .filter_map(|(key, obj)| match obj {
            OsmObj::Node(inner) => Some(inner),
            _ => None,
        })
        .map(|node| (node.id, node))
        .collect(); // TODO: How to get rid of this map function?

    // Create geometry for buildings
    let mut buildings: Vec<Building> = osm_building_ways
        .into_iter()
        .map(|obj| {
            let coords: Vec<(f64, f64)> = obj
                .nodes
                .iter()
                .map(|node_id| {
                    (
                        osm_building_nodes[node_id].decimicro_lat as f64 / 10000000.,
                        osm_building_nodes[node_id].decimicro_lon as f64 / 10000000.,
                    )
                })
                .collect();
            let line_string = geo::LineString::from(coords);
            let polygon = Polygon::new(line_string, vec![]).convex_hull();
            Building {
                polygon: polygon,
                tags: obj.tags,
            }
        })
        .collect();

    let housenumbers: Vec<HouseNumberPoint> = osm_housenumber_nodes
        .iter()
        .map(|(_, obj)| {
            let point = Point::new(
                obj.decimicro_lat as f64 / 10000000.,
                obj.decimicro_lon as f64 / 10000000.,
            );
            let text = obj.tags["addr:housenumber"].to_string();
            HouseNumberPoint {
                point: point,
                text: text,
            }
        })
        .collect();
    // Calculate house number count

    buildings = buildings
        .into_iter()
        .map(|building| building.calculate_house_number_count(&housenumbers))
        .collect();

    // ToDo: Add house number count to tags here
    // Enrich tags with flat count

    // ToDo: Calculate flat count here

    buildings
}
