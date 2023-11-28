from solve import solve
import sys
from glob import glob

if __name__ == "__main__":
    args = sys.argv
    if len(args) != 2:
        print("Invalid number of args")
    else:
        filenames = glob(args[1] + '*')
        domain = None
        problems = []
        for filename in filenames:
            if filename.endswith("domain.hddl"):
                if domain == None:
                    domain = filename
                else:
                    print(f"multiple files found as domain: \n\t1. {domain}\n\t2.{filename}")
            else:
                problems.append(filename)
        problems.sort()
        n_problems = len(problems)
        for i, problem in enumerate(problems):
            print(f"Solving {problem} ({i+1} out of {n_problems})")
            solve(domain, problem)
