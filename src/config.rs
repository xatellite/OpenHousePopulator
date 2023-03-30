#[derive(Clone, serde::Deserialize)]
pub struct Config {
    pub(crate) reroll_threshold: u64,
    pub(crate) reroll_probability: i32,
    pub(crate) level_factor: usize,
    pub(crate) housenumber_factor: usize,
    pub(crate) exclude_landuse: Vec<String>,
    pub(crate) exclude_tags: Vec<String>,
    pub(crate) single_home_list: Vec<String>,
    pub(crate) apartment_list: Vec<String>,
    pub(crate) unspecified_list: Vec<String>,
}

impl Config {
    pub fn builder() -> ConfigBuilder {
        ConfigBuilder::new()
    }
}

pub struct ConfigBuilder {
    reroll_threshold: u64,
    reroll_probability: i32,
    level_factor: usize,
    housenumber_factor: usize,
    exclude_landuse: Vec<String>,
    exclude_tags: Vec<String>,
    single_home_list: Vec<String>,
    apartment_list: Vec<String>,
    unspecified_list: Vec<String>,
}

impl ConfigBuilder {
    pub fn new() -> ConfigBuilder {
        ConfigBuilder {
            reroll_threshold: 90,
            reroll_probability: 2,
            level_factor: 2,
            housenumber_factor: 3,
            exclude_landuse: vec![
                "allotments".to_string(),
                "commercial".to_string(),
                "industrial".to_string(),
                "military".to_string(),
                "retail".to_string(),
            ],
            exclude_tags: vec!["amenity".to_string(), "leisure".to_string()],
            single_home_list: vec!["house".to_string(), "detached".to_string()],
            apartment_list: vec!["apartments".to_string(), "residential".to_string()],
            unspecified_list: vec!["terrace".to_string(), "semidetached_house".to_string()],
        }
    }

    pub fn reroll_threshold(mut self, reroll_threshold: u64) -> ConfigBuilder {
        self.reroll_threshold = reroll_threshold;
        self
    }

    pub fn reroll_probability(mut self, reroll_probability: i32) -> ConfigBuilder {
        self.reroll_probability = reroll_probability;
        self
    }

    pub fn level_factor(mut self, level_factor: usize) -> ConfigBuilder {
        self.level_factor = level_factor;
        self
    }

    pub fn housenumber_factor(mut self, housenumber_factor: usize) -> ConfigBuilder {
        self.housenumber_factor = housenumber_factor;
        self
    }

    pub fn exclude_landuse(mut self, exclude_landuse: Vec<String>) -> ConfigBuilder {
        self.exclude_landuse = exclude_landuse;
        self
    }

    pub fn exclude_tags(mut self, exclude_tags: Vec<String>) -> ConfigBuilder {
        self.exclude_tags = exclude_tags;
        self
    }

    pub fn single_home_list(mut self, single_home_list: Vec<String>) -> ConfigBuilder {
        self.single_home_list = single_home_list;
        self
    }

    pub fn apartment_list(mut self, apartment_list: Vec<String>) -> ConfigBuilder {
        self.apartment_list = apartment_list;
        self
    }

    pub fn unspecified_list(mut self, unspecified_list: Vec<String>) -> ConfigBuilder {
        self.unspecified_list = unspecified_list;
        self
    }

    pub fn build(self) -> Config {
        Config {
            reroll_threshold: self.reroll_threshold,
            reroll_probability: self.reroll_probability,
            level_factor: self.level_factor,
            housenumber_factor: self.housenumber_factor,
            exclude_landuse: self.exclude_landuse,
            exclude_tags: self.exclude_tags,
            single_home_list: self.single_home_list,
            apartment_list: self.apartment_list,
            unspecified_list: self.unspecified_list,
        }
    }
}
