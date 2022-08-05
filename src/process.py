from helpers.address_mapping import calculate_inhabitants
from helpers.done_box_logging import log_box
from qgis.core import *
import datetime
import os
import sys
import csv

# Setup Standalone Qgis
app = QgsApplication([],True, None)
app.setPrefixPath("/usr", True)
app.initQgis()

# CHANGE OF PLUGIN FOLDERS HERE!
sys.path.append("/usr/share/qgis/python/plugins")
sys.path.append(os.path.expanduser("~") + "/.local/share/QGIS/QGIS3/profiles/default/python/plugins")
import processing
from processing.core.Processing import Processing
from processing.tools import *
from QuickOSM.quick_osm import QuickOSMPlugin
Processing.initialize()
QuickOSMPlugin.initProcessing(QuickOSMPlugin)

def process_district(district, inhabitants, centroid, silent=False):
  tmp_prefix = "../tmp/" + district.lower().replace(' ', '_') + "_"
  task = log_box("Loading building Data", silent=silent)

  def load_or_query_data(tmp_prefix, file, layer_name, region_name, key, type):
    if os.path.exists(tmp_prefix + str(file)):
      layer = QgsVectorLayer(tmp_prefix + str(file), layer_name, "ogr")
      return layer
    alg_params = {
      "KEY": key,
      "SERVER": "http://www.overpass-api.de/api/interpreter",
      "TIMEOUT": 300,
      "VALUE": "",
      "AREA": region_name,
    }
    query_result = processing.run("quickosm:downloadosmdatainareaquery", alg_params)
    layer = query_result["OUTPUT_" + str(type)] 
    QgsVectorFileWriter.writeAsVectorFormat(layer, tmp_prefix + str(file), "UTF-8", layer.crs())
    return layer

  layer_buildings = load_or_query_data(tmp_prefix, "layer_buildings.gpkg", "buildings", district, "building", "MULTIPOLYGONS")
  task.end()

  task = log_box("Loading house number Data", silent=silent)
  layer_house_numbers = load_or_query_data(tmp_prefix, "layer_house_numbers.gpkg", "house_numbers", district, "addr:housenumber", "POINTS")
  task.end()

  task = log_box("Count house numbers in buildings", silent=silent)
  alg_params = {
    "POLYGONS": layer_buildings,
    "POINTS": layer_house_numbers,
    "OUTPUT": "../tmp/memory_"+district,
    "FIELD": "housenum_inside"
  }
  count_output = processing.run("native:countpointsinpolygon", alg_params)["OUTPUT"]
  layer_buildings_ext = QgsVectorLayer(count_output, "buildings_ext", "ogr")
  QgsVectorFileWriter.writeAsVectorFormat(layer_buildings_ext, tmp_prefix + "layer_buildings_ext.gpkg", "UTF-8", layer_buildings_ext.crs())
  task.end()

  # Calculate inhabitants for building
  task = log_box("Calculating inhabitants", silent=silent)
  buildings = calculate_inhabitants(layer_buildings_ext, inhabitants)
  task.end()

  export_name = "../out/Population_"+str(district.replace("_", ""))

  # Centroid geometry if requested
  if centroid:
    task = log_box("Calculating centroids", silent=silent)
    export_name = export_name + "_centroids"
    QgsVectorFileWriter.writeAsVectorFormat(
      buildings,
      '../tmp/tmp.gpkg',
      "UTF-8",
      buildings.crs()
    )
    centroid_output = processing.run("native:centroids", {
      'INPUT':'../tmp/tmp.gpkg',
      'ALL_PARTS':False,
      'OUTPUT': "../tmp/memory_" + district
    })["OUTPUT"]
    buildings = QgsVectorLayer(centroid_output, "buildings_ext", "ogr")
    task.end()

  x = datetime.datetime.now()
  export_name = export_name + "_" + x.strftime("%x").replace("/", "-") + ".gpkg"
  task = log_box("Saving result", silent=silent)
  QgsVectorFileWriter.writeAsVectorFormat(
    buildings,
    export_name,
    "UTF-8",
    buildings.crs()
  )
  task.end()

  # cleanup
  os.remove(count_output)


def process_list_entry(town_name, inhabitants, centroid):
  try:
    process_district(town_name, inhabitants, centroid)
  except:
    with open("../tmp/error.csv", "a") as error_file:
      csv_writer = csv.writer(error_file)
      csv_writer.writerow([town_name, inhabitants])
      error_file.close()
    return


def process_list(list, centroid):
  directories = os.listdir("../out")
  present_districts = []
  for file in directories:
    present_districts.append(file.split("_")[1])

  with open(list, newline='') as csvfile:
    csv_reader = csv.reader(csvfile, delimiter=',', quotechar='"')
    for index, row in enumerate(csv_reader):
      town_name = row[0].split(",")[0].split("(")[0]
      if town_name in present_districts:
        continue
      inhabitants = int(row[1].replace(" ", ""))
      process_list_entry(town_name, inhabitants, centroid) 
    csvfile.close()

