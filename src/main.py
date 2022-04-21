from qgis.core import *
import sys
from os.path import exists
from address_mapping import calculate_inhabitants
from done_box_logging import log_box
import os


# Get needed information
region_name = input("Please enter OSM name of region:") # i.g. "Gemeinde Groß-Enzersdorf"
region_inhabitants = int(input("Total population of region:")) # i.g. 11740
tmp_prefix = "../tmp/" + region_name.replace(' ', '_') + "_"

# Setup Standalone Qgis
task = log_box("Starting Qgis")
app = QgsApplication([],True, None)
app.setPrefixPath("/usr", True)
app.initQgis()
sys.path.append("/usr/share/qgis/python/plugins")
sys.path.append(os.path.expanduser("~") + "/.local/share/QGIS/QGIS3/profiles/default/python/plugins")

import processing
from processing.core.Processing import Processing
from processing.tools import *
from QuickOSM.quick_osm import QuickOSMPlugin
Processing.initialize()
QuickOSMPlugin.initProcessing(QuickOSMPlugin)
task.end()


def load_or_query_data(file, layer_name, key, type):
  if exists(tmp_prefix + str(file)):
    layer = QgsVectorLayer(tmp_prefix + str(file), layer_name, "ogr")
    return layer
  alg_params = {
    "KEY": key,
    "SERVER": "http://www.overpass-api.de/api/interpreter",
    "TIMEOUT": 100,
    "VALUE": "",
    "AREA": region_name,
  }
  query_result = processing.run("quickosm:downloadosmdatainareaquery", alg_params)
  layer = query_result["OUTPUT_" + str(type)] 
  QgsVectorFileWriter.writeAsVectorFormat(layer, tmp_prefix + str(file), "UTF-8", layer.crs())
  return layer

task = log_box("Loading building Data")
layer_buildings = load_or_query_data("layer_buildings.gpkg", "buildings", "building", "MULTIPOLYGONS")
task.end()

task = log_box("Loading house number Data")
layer_house_numbers = load_or_query_data("layer_house_numbers.gpkg", "house_numbers", "addr:housenumber", "POINTS")
task.end()

task = log_box("Count house numbers in buildings")
alg_params = {
  "POLYGONS": layer_buildings,
  "POINTS": layer_house_numbers,
  "OUTPUT": "memory",
  "FIELD": "housenum_inside"
}
count_output = processing.run("native:countpointsinpolygon", alg_params)["OUTPUT"]
layer_buildings_ext = QgsVectorLayer(count_output, "buildings_ext", "ogr")
QgsVectorFileWriter.writeAsVectorFormat(layer_buildings_ext, tmp_prefix + "layer_buildings_ext.gpkg", "UTF-8", layer_buildings_ext.crs())
task.end()

# Calculate inhabitants for building
task = log_box("Calculating Inhabitants")
calculate_inhabitants(layer_buildings_ext, region_inhabitants, region_name)
task.end()

# cleanup
os.remove(count_output)