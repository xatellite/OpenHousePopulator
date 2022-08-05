import random
from qgis.core import *
from qgis.PyQt.QtCore import *
from parameters import *

# ToDo: Define max, min flats + add building:levels support
def calculate_inhabitants(buildings, inhabitants):

    single_list = ["house", "detached", "residential", "terrace", "semidetached_house"]
    exclude_keys = ["leisure", "amenity", "emergency"] # ToDo: social_facility=assisted_living should be considered
    required_attributes = ["building", "building:levels", "building:flats", "leisure", "amenity", "emergency", "addr:housenumber", "housenum_inside"]


    def add_flat_count():
        count = 0
        for building in buildings.getFeatures():
            
            should_be_excluded = False
            for exclude in exclude_keys:
                if building[exclude] != NULL:
                    should_be_excluded = True
            if should_be_excluded:
                flats_in_building = 0
            # Handle single family homes
            elif building["building"] in single_list:
                flats_in_building = 1
            # Handle appartments
            elif building["building"] == "apartments":
                if "building:flats" in building and building["building:flats"] != NULL:
                    flats_in_building = int(building["building:flats"])
                    count += flats_in_building
                    building["flat_count"] = flats_in_building
                    buildings.updateFeature(building)
                    # Break if flat count is known
                    continue
                elif building["housenum_inside"] > 1:
                    flats_in_building = int(building["housenum_inside"] * HOUSENUMBER_FACTOR)
                else:
                    flats_in_building = 1
            elif building["building"] == "yes":
                # ToDo: Check if building in commercial zone
                if building["housenum_inside"] > 0 or building["addr:housenumber"]:
                    if building["housenum_inside"] > 1:
                        flats_in_building = int(building["housenum_inside"])
                    else:
                        flats_in_building = 1
                else:
                    flats_in_building = 0
            else:
                flats_in_building = 0

            if building["building:levels"] != NULL and float(building["building:levels"]) > LEVEL_THRESHOLD:
                flats_in_building = flats_in_building + float(building["building:levels"]) - 4
                flats_in_building = flats_in_building * LEVEL_FACTOR
            count += flats_in_building
            building["flat_count"] = flats_in_building
            buildings.updateFeature(building)
        return count
        
    def assign_people_to_flats(total, already_assigned):
        left_over = total - already_assigned
        
        flat_count = 0
        # Adding one person per flat
        for building in buildings.getFeatures():
            if building["flat_count"] != NULL:
                flat_count += building["flat_count"]

        # Random distribute people
        flats_assign = [0] * flat_count
        while left_over > 0:
            random_index = random.randint(0, flat_count - 1)
            flats_assign
            if flats_assign[random_index] > REROLL_THRESHOLD:
                if (random.randint(0,100) < REROLL_PROPAPILITY):
                    continue
            flats_assign[random_index] += 1
            left_over -= 1

        count = 0
        flat_index = 0
        for building in buildings.getFeatures():
            people_assigned = 0
            if building["flat_count"] != NULL:
                for flat_index_it in range(flat_index, building["flat_count"] + flat_index):
                    people_assigned += flats_assign[flat_index_it]
                building["pop"] = building["flat_count"] + people_assigned
                buildings.updateFeature(building)
                flat_index += building["flat_count"]
                count += building["flat_count"] + people_assigned

    building_provider = buildings.dataProvider()
    attribute_indexes = building_provider.attributeIndexes()
    attributes_to_be_removed = []
    for attribute_index in attribute_indexes:
        if buildings.attributeDisplayName(attribute_index) in required_attributes:
            continue
        attributes_to_be_removed.append(attribute_index)
    building_provider.deleteAttributes(attributes_to_be_removed)#
    building_provider.addAttributes([QgsField("flat_count", QVariant.Int)])
    building_provider.addAttributes([QgsField("pop", QVariant.Double, "double", 10, 4)])
    buildings.updateFields()

    buildings.startEditing()
    count = add_flat_count()
    buildings.commitChanges()
    
    buildings.startEditing()
    assign_people_to_flats(inhabitants, count)
    buildings.commitChanges()

    building_provider = buildings.dataProvider()
    indexes = building_provider.attributeIndexes()
    building_provider.deleteAttributes(indexes[1:-2])
    buildings.updateFields()

    # Reducing Attributes - Remove if you need default osm attributes
    return buildings
