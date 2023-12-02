from solve import solve
import sys
from glob import glob
import resource

if __name__ == "__main__":
    mem_limit = 8_589_934_592
    args = sys.argv
    if len(args) < 2:
        print("Provide domain path(s).")
    else:
        resource.setrlimit(resource.RLIMIT_AS, (mem_limit, mem_limit))
        for arg in args[1:]:
            print(f"Solving domain {arg}:")
            filenames = glob(arg + '*')
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
                print(f"\tproblem {i+1} out of {n_problems}: {problem}")
                try:
                    solve(domain, problem)
                except MemoryError:
                    print(f'\t\tmemory limit reached.')

