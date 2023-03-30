use geo::Contains;
use geo::Point;
use geo::Polygon;
use osmpbfreader::Node;
use osmpbfreader::NodeId;
use osmpbfreader::OsmId;
use osmpbfreader::OsmObj;
use osmpbfreader::Tags;
use osmpbfreader::Way;
use rand::Rng;
use std::collections::BTreeMap;
use std::fmt::Display;

use crate::Config;
use crate::parser::housenumber::HouseNumberList;

pub struct HouseNumberPoint {
    point: Point,
    text: String,
}

pub struct Buildings (
    pub Vec<Building>
);

impl From<Vec<GenericWay>> for Buildings {
    fn from(item: Vec<GenericWay>) -> Self {
        Buildings(item.into_iter().map(|way| Building::from(way)).collect())
    }
}

impl Buildings {
    pub fn distribute_population(mut self, housenumbers: Vec<HouseNumberPoint>, inhabitants_total: u64, config: &Config) -> Self {
        // Calculate flats for all buildings
        self.0 = self.0.into_iter().map(|building| building.estimate_flat_count(&housenumbers, config)).collect();

        // Gather total flat count
        let total_flat_count: u32 = self.0.iter().map(|building| building.tags["flats"].parse::<u32>().unwrap()).sum();

        // Distribute population
        let mut flat_inhabitants = vec![0; total_flat_count as usize];

        let mut inhabitants_to_distribute = inhabitants_total;
        while inhabitants_to_distribute > 0 {
            let flat_index = rand::thread_rng().gen_range(0..total_flat_count - 1) as usize;
            if flat_inhabitants[flat_index] > config.reroll_threshold
                && rand::thread_rng().gen_range(0..config.reroll_probability) > config.reroll_threshold
            {
                continue;
            }
            flat_inhabitants[flat_index] += 1;
            inhabitants_to_distribute -= 1;
        }

        // Add population tag to buildings
        let mut flat_offset = 0;
        self.0 = self.0.into_iter().map(|building| {
            let mut building = building;
            let flat_count = building.tags["flats"].parse::<u32>().unwrap();
            let mut population = 0;
            for flat_index in flat_offset..(flat_offset+flat_count) {
                population += flat_inhabitants[flat_index as usize];
            }
            flat_offset += flat_count;
            building.tags.insert("pop".to_string().into(), population.to_string().into());
            building
        }).collect();

        self
    }

    pub fn exclude_in(mut self, area: &Vec<GenericWay>) -> Self{
        self.0 = self.0.into_iter().filter(|building| {
            !area.into_iter().any(|area| area.polygon.contains(&building.polygon))
        }).collect();
        self
    }
}



pub struct GenericWay {
    pub polygon: Polygon,
    pub tags: Tags,
}

#[derive(Debug)]
pub struct Building {
    pub polygon: Polygon,
    pub tags: Tags,
}

impl From<GenericWay> for Building {
    fn from(item: GenericWay) -> Self {
        Building { polygon: item.polygon, tags: item.tags }
    }
}

impl Building {
    /// Gets the number of house numbers in the building
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

    /// Calculate number of flats inside building by tags
    fn calculate_flat_count(mut self, config: &Config) -> Self {
        let single_home_list: [&str; 2] = ["house", "detached"];
        let apartment_list: [&str; 2] = ["apartments", "residential"];
        let unspecified_list: [&str; 2] = ["terrace", "semidetached_house"];

        // If flat count is defined in tags, this is applied
        if self.tags.contains_key("building:flats") {
            let flat_count = self.tags["building:flats"]
                .parse::<i32>()
                .unwrap();
            self.tags.insert(
                "flats".to_string().into(),
                flat_count.to_string().into(),
            );
            return self
        }

        let house_numbers = self.tags["addr:housenumber_count"]
            .parse::<i32>()
            .unwrap(); // ToDo: Map error here!

        let mut flat_count: i32 = 0;
        if single_home_list.contains(&self.tags["building"].as_str()) {
            flat_count = 1;
        } else if apartment_list.contains(&self.tags["building"].as_str()) || unspecified_list.contains(&self.tags["building"].as_str()) {
            if house_numbers >= 1 {
                flat_count = house_numbers * config.housenumber_factor;
            } else {
                flat_count = 1;
            }
        } else if self.tags["building"] == "yes" && house_numbers >= 1 {
            flat_count = house_numbers;
        }

        // Increase flat count by building levels
        if self.tags.contains_key("building:levels") {
            let levels: i32 = self.tags["building:levels"].parse::<f32>().map_err(|err| {
                println!("Error: {:?} on value {:?}", err, self.tags["building:levels"]);
                0
            }).unwrap().floor() as i32;
            if levels > config.level_threshold {
                flat_count += self.tags["building:levels"]
                    .parse::<i32>()
                    .unwrap()
                    - 4;
                flat_count *= config.level_factor;
            }
        }

        // Insert flat tag
        self.tags.insert(
            "flats".to_string().into(),
            flat_count.to_string().into(),
        );
        self
    }

    /// Estimates number of flats inside building
    pub fn estimate_flat_count(self, house_number_points: &Vec<HouseNumberPoint>, config: &Config) -> Self {
        self.calculate_house_number_count(house_number_points)
            .calculate_flat_count(config)
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


/// Check if osm obj is building
pub fn is_exclude_area(obj: &osmpbfreader::OsmObj, config: &Config) -> bool {
    obj.is_way() && ((obj.tags().contains_key("landuse") && config.exclude_landuse.contains(&obj.tags()["landuse"].to_string()))
    || config.exclude_tags.iter().any(|exclude_tag| obj.tags().contains_key(exclude_tag.as_str())))
}

pub fn load_ways(
    osm_buildings: BTreeMap<OsmId, OsmObj>,
) -> Vec<GenericWay> {
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
    let ways = osm_building_ways
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
        .collect();

    ways
}

pub fn load_housenumbers(
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

    let housenumbers: Vec<HouseNumberPoint> = osm_housenumber_nodes
        .values()
        .map(|obj| {
            let point = Point::new(
                obj.decimicro_lat as f64 / 10000000.,
                obj.decimicro_lon as f64 / 10000000.,
            );
            let text = obj.tags["addr:housenumber"].to_string();
            HouseNumberPoint { point, text }
        })
        .collect();

    housenumbers
}


// ToDo: Filter function 
// let exclude_keys: [&str; 3] = ["leisure", "amenity", "emergency"];
// and areas
