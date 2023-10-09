import re
from htn_serializer import HTNSerializer
from fond_merger import FONDMerger

serializer = HTNSerializer("result.ground")
result = serializer.run()
merger = FONDMerger(result)
result = merger.run()

import json
with open("result.json", "w+") as f:
    json.dump(result, f, indent=4)
