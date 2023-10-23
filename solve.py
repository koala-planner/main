import sys
import os
import subprocess

def solve(domain, problem):
    path = os.getcwd()
    parser_path = path + "/parser/pandaPIparser"
    grounder_path = path + "/grounder/pandaPIgrounder/"
    serilazer_path = path + "/serializer/"
    planner_path = path + "/search/"
    # Parsing
    parsed = subprocess.run(
        [ parser_path,
         path + f"/{domain}", path + f"/{problem}"],
         capture_output=True)
    with open(grounder_path + "parsed.htn", "w+") as f:
        f.write(parsed.stdout.decode("utf-8"))
    print("Parsing Complete.")
    # Grounding
    grounded = subprocess.run(
        [grounder_path + "pandaPIgrounder",
        grounder_path + "parsed.htn",
        serilazer_path + "result.ground"], capture_output=True
    )
    # os.remove(grounder_path + "parsed.htn")
    print("Grounding Complete.")
    # Serializing
    subprocess.run(
        ["python3", serilazer_path + "htn_parser.py",
         serilazer_path + "result.ground", planner_path + "result.json"],
        capture_output=False)
    # os.remove(serilazer_path + "result.ground")
    print("Conversion to JSON Complete.")
    # Search
    subprocess.run(
        [planner_path + "target/debug/planner", planner_path + "result.json"],
        capture_output=False)

if __name__ == "__main__":
    args = sys.argv
    if len(args) != 3:
        print("Invalid number of args")
    else:
        solve(args[1], args[2])
