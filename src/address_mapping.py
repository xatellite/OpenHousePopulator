import random
from qgis.core import *
from qgis.PyQt.QtCore import *

def calculate_inhabitants(buildings, inhabitants):

    keylist = ["house", "detached", "residential", "terrace"]

    building_provider = buildings.dataProvider()
    building_provider.addAttributes([QgsField("flat_count",QVariant.Int)])
    building_provider.addAttributes([QgsField("pop", QVariant.Double, "double", 10, 4)])
    buildings.updateFields()


    def add_address_count():
        count = 0
        for building in buildings.getFeatures():
            if building["building"] in keylist:
                flats_in_building = 1
            elif building["building"] == "apartments":
                if building["building:flats"]:
                    flats_in_building = int(building["building:flats"])
                    continue
                elif building["housenum_inside"] > 1:
                    flats_in_building = int(building["housenum_inside"] * 4)
                else:
                    flats_in_building = 1
            elif building["building"] == "yes":
                if building["housenum_inside"] > 0 or building["addr:housenumber"]:
                    if building["housenum_inside"] > 1:
                        flats_in_building = int(building["housenum_inside"])
                    else:
                        flats_in_building = 1
                else:
                    flats_in_building = 0
            else:
                flats_in_building = 0
            count += flats_in_building
            building['flat_count'] = flats_in_building
            buildings.updateFeature(building)
        return count
        
    def asign_people_to_flats(total, already_assigned):
        left_over = total - already_assigned
        
        flat_count = 0
        for building in buildings.getFeatures():
            if building['flat_count'] != NULL:
                #s = s + ar
                flat_count += building['flat_count']

        flats_assign = [0] * flat_count
        for person in range(0, left_over):
            flats_assign[random.randint(0, flat_count - 1)] += 1

        count = 0
        flat_index = 0
        for building in buildings.getFeatures():
            people_assigned = 0
            if building['flat_count'] != NULL:
                for flat_index_it in range(flat_index, building['flat_count'] + flat_index):
                    people_assigned += flats_assign[flat_index_it]
                building['pop'] = building['flat_count'] + people_assigned
                buildings.updateFeature(building)
                flat_index += building['flat_count']
                count += building['flat_count'] + people_assigned

    buildings.startEditing()
    count = add_address_count()
    asign_people_to_flats(inhabitants, count)
    buildings.commitChanges()

    QgsVectorFileWriter.writeAsVectorFormat(buildings, "../out/buildings_ext_pop.gpkg", "UTF-8", buildings.crs())