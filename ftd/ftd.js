// all ftd_utils are meant to be pure functions only: they can only depend on the
// input passed, not on closures or global data etc
let ftd_utils = {
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
        if (act === "toggle") {
            let target = action["target"];
            exports.set_bool(id, target, data[target].value !== 'true');
        } else if (act === "increment") {
            let target = action["target"];
            let increment = 1;
            if (action["parameters"].by !== undefined) {
                increment = parseInt(action["parameters"].by[0]);
            }
            let clamp_max = undefined;
            let clamp_min = undefined;
            if (action["parameters"]["clamp"] !== undefined) {
                let clamp_value = action["parameters"]["clamp"];
                if (clamp_value.length === 1) {
                    clamp_max = parseInt(clamp_value[0]);
                }
                if (clamp_value.length === 2) {
                    clamp_min = parseInt(clamp_value[0]);
                    clamp_max = parseInt(clamp_value[1]);
                }
            }
            exports.increment_decrement_value(id, target, increment, clamp_min, clamp_max);

        } else if (act === "decrement") {
            let target = action["target"];
            let decrement = -1;
            if (action["parameters"].by !== undefined) {
                decrement = -parseInt(action["parameters"].by[0]);
            }

            let clamp_max = undefined;
            let clamp_min = undefined;
            if (action["parameters"]["clamp"] !== undefined) {
                let clamp_value = action["parameters"]["clamp"];
                if (clamp_value.length === 1) {
                    clamp_max = parseInt(clamp_value[0]);
                }
                if (clamp_value.length === 2) {
                    clamp_min = parseInt(clamp_value[0]);
                    clamp_max = parseInt(clamp_value[1]);
                }
            }

            exports.increment_decrement_value(id, target, decrement, clamp_min, clamp_max);
        } else {
            console.log("unknown action:", act);
            return;
        }

    }

    let exports = {};

    exports.handle_event = function (id, event) {
        console.log(id, event);
        let actions = JSON.parse(event);
        for (const action in actions) {
            handle_event(id, actions[action])
        }
    }

    exports.increment_decrement_value = function (id, variable, increment_by, clamp_min, clamp_max) {
        let data = ftd_data[id];

        if (!data[variable]) {
            console.log(variable, "is not in data, ignoring");
            return;
        }

        let value = parseInt(data[variable].value);
        value += increment_by;

        if (clamp_max !== undefined) {
            let min = (clamp_min === undefined) ? 0: clamp_min
            if (clamp_max < value) {
                value = min;
            }
            if (clamp_min > value) {
                value = clamp_max;
            }
        }
        data[variable].value = value.toString();

        let dependencies = data[variable].dependencies;
        for (const dependency in dependencies) {
            if (!dependencies.hasOwnProperty(dependency)) {
                continue;
            }
            if (dependencies[dependency] === "value") {
                document.getElementById(`${dependency}:${id}`).innerText = data[variable].value;
            } else {
                let display = "none";
                if (data[variable].value === dependencies[dependency]) {
                    let is_flex = document.getElementById(`${dependency}:${id}`).style.flexDirection.length;
                    if (is_flex) {
                        display = "flex";
                    } else {
                        display = "block";
                    }
                }
                document.getElementById(`${dependency}:${id}`).style.display = display;
            }
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
                let is_flex = document.getElementById(`${dependency}:${id}`).style.flexDirection.length;
                if (is_flex) {
                    display = "flex";
                } else {
                    display = "block";
                }
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
