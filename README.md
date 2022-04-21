# OpenPopulationEstimator

This tool can be used to estimate the inhabitant count per house in an area with a known population.
The result is based on a very simple heuristic, so low accuracy should be expected.
Qgis needs to be installed, including the QickOSM plugin in the default profile

```
cd src
python main.py
```

Output:
A Geopackage with all buildings including the additional field 'pop'
