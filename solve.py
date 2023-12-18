import sys
import os
import subprocess

# Timeout in minutes
def solve(domain, problem, timeout=30):
    path = os.getcwd()
    parser_path = path + "/parser/pandaPIparser"
    grounder_path = path + "/grounder/pandaPIgrounder/"
    serilazer_path = path + "/serializer/"
    planner_path = path + "/planner/"
    # Parsing
    parsed = subprocess.run(
        [parser_path,
         path + f"/{domain}", path + f"/{problem}"],
         capture_output=True)
    with open(grounder_path + "parsed.htn", "w+") as f:
        f.write(parsed.stdout.decode("utf-8"))
    # Grounding
    if os.path.isfile(grounder_path + "parsed.htn"):
        subprocess.run(
            [grounder_path + "pandaPIgrounder",
            grounder_path + "parsed.htn",
            serilazer_path + "result.sas+"], capture_output=True
        )
        os.remove(grounder_path + "parsed.htn")
    else:
        print(f"\t\tfailed to parse {problem}", file=sys.stderr)
        return
    # Serializing
    if os.path.isfile(serilazer_path + "result.sas+"):
        subprocess.run(
            ["python3", serilazer_path + "htn_parser.py",
            serilazer_path + "result.sas+", planner_path + "result.json"],
            capture_output=True)
        os.remove(serilazer_path + "result.sas+")
    else:
        print(f"\t\tfailed to ground {problem}", file=sys.stderr)
        return
    # Search
    if os.path.isfile(planner_path + "result.json"):
        try:
            result = subprocess.run(
                [planner_path + "target/release/planner", planner_path + "result.json"],
                capture_output=True, timeout= 60 * timeout)
            with open(path + f"/{problem}_solution.txt", "x") as f:
                f.write(result.stdout.decode("utf-8"))
        except subprocess.TimeoutExpired:
            print(f'\t\ttimeout for {problem}')
        os.remove(planner_path + "result.json")
    else:
        print(f"failed to serialize {problem}", file=sys.stderr)

if __name__ == "__main__":
    import sys
    domain = sys.argv[1]
    problem = sys.argv[2]
    solve(domain, problem)
