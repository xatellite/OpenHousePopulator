import openhousepopulator_core
import os
import csv
from parameters import LEVEL_THRESHOLD, REROLL_THRESHOLD, REROLL_PROBABILITY,LEVEL_FACTOR, HOUSENUMBER_FACTOR, OVERPASS_ENDPOINT

config = openhousepopulator_core.Config(
  LEVEL_THRESHOLD,
  REROLL_THRESHOLD,
  REROLL_PROBABILITY,
  LEVEL_FACTOR,
  HOUSENUMBER_FACTOR,
  OVERPASS_ENDPOINT
)

def process_district(district, inhabitants, centroid):
  openhousepopulator_core.populate(district, inhabitants, centroid, config)

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

