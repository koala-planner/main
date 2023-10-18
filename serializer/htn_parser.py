import sys
from htn_serializer import HTNSerializer
from fond_merger import FONDMerger

if __name__ == "__main__" :
    serializer = HTNSerializer(sys.argv[1])
    result = serializer.run()
    merger = FONDMerger(result)
    result = merger.run()

    import json
    with open(sys.argv[2], "w+") as f:
        json.dump(result, f, indent=4)