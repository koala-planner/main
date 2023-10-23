import re
from pprint import pprint
class HTNSerializer:
    # open raw *.ground file
    def __init__(self, path):
        s = ""
        with open(path, "r") as f:
            s = f.read()
        self.raw_str = s
        self.serialized = {}
    # split to a vector of eleven sections that correspond to the original file    
    def preprocess(self):
        sections = self.raw_str.split(";;")[1:]
        processed_sections = {}
        single_line_defs = ("initial_state", "goal", "initial_abstract_task")
        for section in sections:
            lines = section.splitlines()
            group_name = re.sub(r'[^\w]', ' ', lines[0]).strip()
            group_name = re.sub(r'[^\w]', '_', group_name).lower()
            group_val = lines[1:][1:] if group_name not in single_line_defs else lines[1:]
            processed_sections[group_name] = [x for x in group_val if x != ""]
        processed_sections["tasks"] = processed_sections.pop("tasks__primitive_and_abstract")
        self.serialized = processed_sections
    # TODO: Complete this
    # Process Actions
    def process_actions(self):
        raw_actions = self.serialized["actions"]
        processed_actions = {}
        for i in range(0,len(raw_actions), 4):
            action = {}
            action["cost"] = int(raw_actions[i])
            precond_ids = [int(x) for x in raw_actions[i+1].split()[:-1]]
            precond_names = [self.serialized["state_features"][i] for i in precond_ids]
            action["precond"] = precond_names
            action["add_eff"] = self.process_effects(raw_actions[i+2])
            action["del_eff"] = self.process_effects(raw_actions[i+3])
            processed_actions[self.serialized["tasks"][i//4]] = action
        self.serialized["actions"] = processed_actions
    # TODO: Complete this 
    # TODO: investigate what _splitting_method is in the output
    # Process methods
    def process_methods(self):
        raw_methods = self.serialized["methods"]
        processed_methods = {}
        for i in range(0,len(raw_methods), 4):
            method = {}
            # number of primitive tasks in network (for id)
            tasks = self.serialized["tasks"]
            task_id = int(raw_methods[i+1])
            method["task"] = tasks[task_id]
            subtask_ids = raw_methods[i+2].split()[:-1]
            subtask_names = [] 
            for subtask_id in subtask_ids:
                subtask_names.append(tasks[int(subtask_id)])
            method["subtasks"] = subtask_names
            orderings = raw_methods[i+3].split()[:-1]
            method["orderings"] = [(int(orderings[x]), int(orderings[x+1])) for x in range(0, len(orderings), 2)]
            name = raw_methods[i] + "_" + str(i // 4)
            processed_methods[name] = method
        self.serialized["methods"] = processed_methods
    # Process tasks
    def process_tasks(self):
        raw_tasks = self.serialized["tasks"]
        tasks = {}
        for i, task in enumerate(raw_tasks):
            tasks[i] = task[2:]
        self.serialized["tasks"] = tasks
    # TODO: Add support for conditional effects
    def process_effects(self, effects):
        splited = effects.split()
        blocks = []
        cursor = 0
        while splited[cursor] != '-1':
            length = int(splited[cursor]) +1
            block = []
            for i in range(length + 1):
                cursor += i
                val = int(splited[cursor])
                eff_name = self.serialized["state_features"][val]
                block.append(val)
            cursor+=1
            blocks.append(block)
        cursor+=1
        effects = {"unconditional": [], "conditional": []}
        for block in blocks:
            # unconditional effects
            if len(block) == 2:
                id = int(block[1])
                eff_name = self.serialized["state_features"][id]
                effects["unconditional"].append(eff_name)
            # conditional effects
            else: 
                cond_eff = {}
                cond_ids = [int(x) for x in block[:-1]]
                cond_names = [self.serialized["state_features"][id] for id in cond_ids]
                cond_eff["condition"] = cond_names
                cond_eff["effect"] = self.serialized["state_features"][int(block[-1])]
        return effects
    def process_init(self):
        init_state = self.serialized["initial_state"][0].split()
        facts = [self.serialized["state_features"][int(id)] for id in init_state[:-1]]
        self.serialized["initial_state"] = facts
        initial_task_id = int(self.serialized["initial_abstract_task"][0])
        n_primitives = len(self.serialized["actions"].keys())
        self.serialized["initial_abstract_task"] = self.serialized["tasks"][initial_task_id]
    def run(self):
        self.preprocess()
        self.process_tasks()
        self.process_actions()
        self.process_methods()
        self.process_init()
        raw_tasks = self.serialized["tasks"]
        tasks = { "primitive": [], "abstract": []}
        for i in range(0, len(self.serialized["actions"])):
            tasks["primitive"].append(raw_tasks[i])
        for i in range(len(self.serialized["actions"]), len(raw_tasks)):
            tasks["abstract"].append(raw_tasks[i])
        self.serialized["tasks"] = tasks
        return self.serialized