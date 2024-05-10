# About
Koala solves Fully Observable Non Deterministic (FOND) Hierarchical Task Network (HTN) planning problems, expressed in the formalism presented by Chen and Bercher[^1]. Syntactically, the inputs are written down in an extended version of Hierarchical Domain Description Language (HDDL)[^4], where ```oneof``` keyword is used to denote non-deterministic effects. Grounding is done using an unchanged version of PANDA's grounder[^3]. The underlying algorithm is AO* with support for various heuristics[^2].

# Installation
To build the [parser](https://github.com/panda-planner-dev/pandaPIparser) and the [grounder](https://github.com/panda-planner-dev/pandaPIgrounder), follow the instructions on their respective pages. 

To planner is written is Rust. To build it, install Rust from the official website, and run the following command in the ```/planner``` directory.

```cargo make```

# Usage
To solve a planning problem ```/path/to/file/problem.hddl``` in domain ```/path/to/file/doamin.hddl```, on a machine where ```python``` is defined to be an instance of Python 3 interpreter, run the following command.

```python solve.py /path/to/file/doamin.hddl /path/to/file/problem.hddl```

# References
[^1]: [Flexible FOND HTN Planning: A Complexity Analysis](https://bercher.net/publications/2022/Chen2022FlexibleFONDHTNs.pdf)

[^2]: [Laying the Foundations for Solving FOND HTN problems: Grounding, Search, Heuristics (and Benchmark Problems)](#)

[^3]: [On Succinct Groundings of HTN Planning Problems](https://ojs.aaai.org/index.php/AAAI/article/view/6529)

[^4]: [HDDL: An Extension to PDDL for Expressing Hierarchical Planning Problems](https://staff.fnwi.uva.nl/g.behnke/papers/Hoeller2020HDDL.pdf)