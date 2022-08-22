# OpenPopulationEstimator

This tool can be used to estimate the inhabitant count per house in an area with a known population.
The result is based on a very simple heuristic, so low accuracy should be expected.

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
./OpenHousePopulator
```

Output:
A GeoJson with all buildings including the additional fields 'pop' (population), 'flats' (household estimation), 'housenumbers' (calculated house numbers)
