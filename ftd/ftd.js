let ftd_utils = {
    to_action: function (a) {
        // pure function (only operates on input)

        if (a.startsWith("toggle")) {
            let target = a.replace("toggle ", "");
            return {action: "toggle", target: target};
        }
        return {};
    },

    parse_js_event: function (action) {
        // pure function (only operates on input)

        const actions_string = action.split(";");
        const actions = [];
        for (const action in actions_string) {
            if (!actions_string.hasOwnProperty(action)) {
                continue;
            }

            let a = actions_string[action].trim();
            if (a !== "") {
                let get_action = ftd_utils.to_action(a)
                if (Object.keys(get_action).length !== 0) {
                    actions.push(get_action);
                }
            }
        }
        return actions;
    },

    is_visible: function (id, affected_id) {
        return (document.getElementById(`${affected_id}:${id}`).style.display !== "none");
    }
};

window.ftd = (function () {
    let ftd_data = {};
    let ftd_external_children = {};



    function external_children_replace(id) {
        let external_children = ftd_external_children[id];
        for (const object in external_children) {
            if (!external_children.hasOwnProperty(object)) {
                continue;
            }

            let conditions = external_children[object];
            for (const idx in conditions) {
                if (!conditions.hasOwnProperty(idx)) {
                    continue;
                }

                let condition = conditions[idx].condition;
                let set_at = conditions[idx].set_at;
                let display = true;
                for (const i in condition) {
                    if (!condition.hasOwnProperty(i)) {
                        continue;
                    }

                    display &= ftd_utils.is_visible(id, conditions[idx].condition[i])
                    if (!display) {
                        break;
                    }
                }
                if (display) {
                    console.log(`${object}::: ${set_at}`);
                    let get_element_set_at = document.getElementById(`${set_at}:${id}`);
                    let object_to_set = document.getElementById(`${object}:${id}`);
                    let parent = object_to_set.parentElement;
                    if (parent !== get_element_set_at) {
                        get_element_set_at.appendChild(object_to_set);
                    }
                    return;
                }
            }

        }
    }

    function handle_event(id, action) {
        let act = action["action"];
        let data = ftd_data[id];
        if (act !== "toggle") {
            console.log("unknown action:", act);
            return;
        }

        let target = action["target"];
        exports.set_bool(id, target, data[target].value !== 'true');
    }

    let exports = {};

    exports.handle_event = function (id, event) {
        console.log(id, event);
        let actions = ftd_utils.parse_js_event(event);
        for (const action in actions) {
            handle_event(id, actions[action])
        }
    }

    exports.set_bool = function (id, variable, value) {
        let data = ftd_data[id];

        if (!data[variable]) {
            console.log(variable, "is not in data, ignoring");
            return;
        }

        data[variable].value = value.toString();

        let dependencies = data[variable].dependencies;

        for (const dependency in dependencies) {
            if (!dependencies.hasOwnProperty(dependency)) {
                continue;
            }

            let display = "none";
            if (data[variable].value === dependencies[dependency]) {
                display = "block";
            }
            document.getElementById(`${dependency}:${id}`).style.display = display;
        }
        external_children_replace(id);
    }

    exports.set_multi_value = function (id, list) {
        for (const idx in list) {
            if (!list.hasOwnProperty(idx)) {
                continue;
            }

            let item = list[idx];
            let [variable, value] = item;
            this.set_bool(id, variable, value);
        }
    }

    exports.init = function (id, data, external_children) {
        ftd_data[id] = data;
        ftd_external_children[id] = external_children;
    }

    return exports;
})();
