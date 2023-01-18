
window.ftd = (function() {
    let ftd_data: any = {};
    let exports: Partial<Export> = {};

    exports.init = function (id: string, data: string) {
        let element = document.getElementById(data);
        if (!!element) {
            ftd_data[id] = JSON.parse(element.innerText);
            window.ftd.post_init();
        }
    };

    function handle_function(evt: Event, id: string, action: Action, obj: Element, function_arguments: (FunctionArgument | any)[]) {
        console.log(id, action);
        console.log(action.name);

        let argument: keyof typeof action.values;
        for (argument in action.values) {
            if (action.values.hasOwnProperty(argument)) {
                // @ts-ignore
                let value = action.values[argument][1] !== undefined ? action.values[argument][1]: action.values[argument];
                if (typeof value === 'object') {
                    let function_argument = <FunctionArgument>value;
                    if (!!function_argument.reference) {
                        let obj_value = null;
                        try {
                            obj_value= (<HTMLInputElement>obj).value;
                        } catch {
                            obj_value = null;
                        }
                        let value = resolve_reference(function_argument.reference, ftd_data[id], obj_value);
                        if (!!function_argument.mutable) {
                            function_argument.value = value;
                            function_arguments.push(function_argument);
                        } else {
                            function_arguments.push(deepCopy(value));
                        }
                    }
                } else {
                    function_arguments.push(value);
                }
            }
        }

        return window[action.name](...function_arguments);
    }


    function handle_event(evt: Event, id: string, action: Action, obj: Element) {
        let function_arguments: (FunctionArgument | any)[] = [];
        handle_function(evt, id, action, obj, function_arguments);
        change_value(function_arguments, ftd_data[id], id);
    }

    exports.handle_event = function (evt: Event, id: string, event: string, obj: Element) {
        console_log(id, event);
        let actions = JSON.parse(event);
        for (const action in actions) {
            handle_event(evt, id, actions[action], obj);
        }
    };

    exports.handle_function = function (evt: Event, id: string, event: string, obj: Element) {
        console_log(id, event);
        let actions = JSON.parse(event);
        let function_arguments: (FunctionArgument | any)[] = [];
        return handle_function(evt, id, actions, obj, function_arguments);
    };

    exports.get_value = function (id, variable) {
        let data = ftd_data[id];

        let [var_name, _] = get_name_and_remaining(variable);

        if (data[var_name] === undefined && data[variable] === undefined) {
            console_log(variable, "is not in data, ignoring");
            return;
        }
        return get_data_value(data, variable);
    };

    exports.set_string_for_all = function (variable, value) {
        for (let id in ftd_data) {
            if (!ftd_data.hasOwnProperty(id)) {
                continue;
            }

            // @ts-ignore
            exports.set_value_by_id(id, variable, value);
        }
    };


    exports.set_bool_for_all = function (variable, value) {
        for (let id in ftd_data) {
            if (!ftd_data.hasOwnProperty(id)) {
                continue;
            }

            // @ts-ignore
            exports.set_bool(id, variable, value);
        }
    };

    exports.set_bool = function (id, variable, value) {
        window.ftd.set_value_by_id(id, variable, value);
    };

    exports.set_value = function (variable, value) {
        window.ftd.set_value_by_id("main", variable, value);
    };

    exports.set_value_by_id = function (id, variable, value) {
        let data = ftd_data[id];

        let [var_name, remaining] = get_name_and_remaining(variable);

        if (data[var_name] === undefined && data[variable] === undefined) {
            console_log(variable, "is not in data, ignoring");
            return;
        }
        if (!!window["set_value_" + id] && !!window["set_value_" + id][var_name]) {
            window["set_value_" + id][var_name](data, value, remaining);
        }
        else {
            set_data_value(data, variable, value);
        }
    };

    return exports;
})();
