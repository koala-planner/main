import string
from pprint import pprint

class FONDMerger:
    def __init__(self, domain):
        self.domain = domain
        self._translator_cache = None
    def run(self):
        self.extract_actions()
        self.prune()
        self.remove_redundancies()
        return self.domain
    # get a mapping from nd action name to its number of effects
    def _extract_numbers(self):
        nd_methods = [self.domain["methods"][x] for x in self.domain["methods"] if x.startswith("fond_act__")]
        counts = {}
        for m in nd_methods:
            task_name = m["task"]
            n_effects = [x for x in m["subtasks"] if x.startswith("fond_act__")][0]
            n_effects = int(n_effects.split("of")[1][0])
            counts[task_name] = n_effects
        return counts
    # get a mapping from preprocessed nd action name to its original one
    def _nd_translator(self):
        if self._translator_cache == None:
            action_count = self._extract_numbers()
            translator = {}
            for action, count in action_count.items():
                splitted_name = action.split("[")
                for i in range(count):
                    translator[f"fond_act__{splitted_name[0]}_{str(i+1)}of{count}[{splitted_name[1]}"] = action
            self._translator_cache = translator
            return translator
        else:
            return self._translator_cache
    def extract_actions(self):
        translator = self._nd_translator()
        nd_actions = {}
        to_be_removed = set()
        for action_name in self.domain["actions"]:
            if action_name.startswith("fond_act__"):
                translated_name = translator[action_name]
                action = self.domain["actions"][action_name]
                if translated_name in nd_actions:
                    eff = {}
                    eff["add_eff"] = action["add_eff"]
                    eff["del_eff"] = action["del_eff"]
                    nd_actions[translated_name]["effects"].append(eff)
                else:
                    new_act = self.domain["actions"][action_name]
                    eff = {}
                    add_eff, del_eff = new_act.pop("add_eff"), new_act.pop("del_eff")
                    eff["add_eff"] = add_eff
                    eff["del_eff"] = del_eff
                    new_act["effects"] = [eff,]
                    nd_actions[translated_name] = new_act
                to_be_removed.add(action_name)
        det_actions = set(self.domain["actions"].keys())
        det_actions -= to_be_removed
        for act in det_actions:
            val = self.domain["actions"][act]
            add_eff, del_eff = val.pop("add_eff"), val.pop("del_eff")
            effs = [{"add_eff": add_eff, "del_eff": del_eff}, ]
            self.domain["actions"][act]["effects"] = effs
        for key in to_be_removed:
            self.domain["actions"].pop(key)
        for key in nd_actions:
            self.domain["actions"][key] = nd_actions[key]
    def prune(self):
        expected_counts = self._extract_numbers()
        actual_counts = {k: 0 for k in expected_counts.keys()}
        methods = self.domain["methods"]
        for method_name in methods:
            if methods[method_name]["task"] in expected_counts:
                actual_counts[methods[method_name]["task"]] += 1
        actual_counts = set(actual_counts.items())
        expected_counts = set(expected_counts.items())
        to_be_pruned = [x[0] for x in (expected_counts - actual_counts)]
        print(f"Pruned {len(to_be_pruned)} actions")
        for task in to_be_pruned:
            self.domain["actions"].pop(task)
    def remove_redundancies(self):
        # remove FOND abstract tasks
        nd_actions = list(self._extract_numbers().keys())
        filtered_tasks = list(filter(lambda x: (x not in nd_actions), self.domain["tasks"]["abstract"]))
        self.domain["tasks"] = filtered_tasks
        # remove added methods
        nd_methods = [x for x in self.domain["methods"] if x.startswith("fond_act__")]
        nd_method_preconds = [x for x in self.domain["actions"] if x.startswith("__method_precondition_fond_act__")]
        for method in nd_methods:
            self.domain["methods"].pop(method)
        for method_precond in nd_method_preconds:
            self.domain["actions"].pop(method_precond)