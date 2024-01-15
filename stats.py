from glob import glob
from pprint import pprint
folders = glob("./fond_domains/*")
result = {}

for folder in folders:
    instances = glob(folder + "/*")
    solved_instances = [x for x in instances if "solution.txt" in x]
    domain = folder.split("/")[-1]
    result[domain] = {}
    for solved_instance in solved_instances:
        problem = solved_instance.split('/')[-1].split('.hddl')[0]
        result[domain][problem] = {}
        with open(solved_instance, "r") as f:
            for line in f.readlines():
                split = line.split(": ")
                result[domain][problem][split[0]] = int(split[1].rstrip())

pprint(result)