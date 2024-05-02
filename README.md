 
<div align="center">
  <a href="https://github.com/xatellite/OpenHousePopulator">
    <img src="res/logo.png" alt="Logo" width="150" height="150">
  </a>

  <h2 align="center">OpenHousePopulator</h3>
  <p align="center">
    This tool can be used to estimate the inhabitant count per house in an area with a known population using <a href="https://www.openstreetmap.org">OpenStreetMap</a> data.
    The result is based on a very simple heuristic, so low accuracy should be expected.
  </p>
</div>



## Setup

### Step 1: Install Rust:


**Arch** based:
```
$ sudo pacman -S rust
```

**Debian** based:
```
$ sudo apt-get update
$ sudo apt-get upgrade
$ sudo apt-get install rust
```

For use under **Windows / MacOS** install rust using the default installer from the official website: [www.rust-lang.org](https://www.rust-lang.org/)
### Step 2: Setup OpenHousePopulator

```
$ chmod 755 setup.sh
$ ./setup.sh
```

## Execution

To execute the CLI run:

```
cd bin

// Show help
./OpenHousePopulator --help

// Populate example area
./OpenHousePopulator populate -f "./res/Gmunden.osm.pbf" -i 7602 --centroid
```

Where `-i` (mandatory) describes the number of inhabitants in the area of the `.osm.pbf` file. The `--centroid` (optional) parameter puts the data in a GeoJSON `Point` geometry instead into the buildings geometry. 

Output:
A GeoJson with all buildings including the additional fields 'pop' (population), 'flats' (household estimation)

## Configuration

You can configure the following parameters in the `config.json` file:

- reroll_threshold: The minimum population count to start rerolling (populate next building).
- reroll_probability: The probability to reroll a building (over threshold).
- level_factor: The factor to multiply the level count with (if multi-storey).
- housenumber_factor: The factor to multiply the house number count with.
- exclude_landuse: Areas to exclude buildings in. (e.g. ["industrial", "commercial"]).
- exclude_tags: Areas to exclude by tag (e.g. ["amenity", "leisure"]).
- single_home_list:  List of building values to be considered single home houses (e.g. ["house", "detached"]).
- apartment_list: List of buildings values to be considered apartments (e.g. ["apartments", "residential"]).
- unspecified_list: List of buildings values to be considered unspecified (e.g. ["terrace", "semidetached_house"]).