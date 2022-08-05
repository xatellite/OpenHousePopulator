import argparse
from process import process_district, process_list

# create a parser object
parser = argparse.ArgumentParser(description="Distribute residences to houses")

# add argument
parser.add_argument("-d", "--district", type = str, nargs = 1, metavar = "name", default = None, help = "District to be evaluated")
parser.add_argument("-i", "--inhabitants", type = int, nargs = 1, metavar = "inhabitants", default = None, help = "Inhabitants of district to be distributed")
parser.add_argument("-l", "--list", type = str, nargs = 1, metavar = "file", default = None, help = "List of districts and inhabitants to be evaluated")
parser.add_argument("-c", "--centroid", action='store_false', help = "Add if output should contain centroids")

# parse the arguments from standard input
args = parser.parse_args()
 
if args.district != None and args.inhabitants != None:
    process_district(args.district[0], args.inhabitants[0], args.centroid)
elif args.list != None:
    process_list(args.list[0], args.centroid)
else:
  print("Parameter missing. Please check -h / --help for info.")
