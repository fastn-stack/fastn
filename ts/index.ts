
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

    exports.data = ftd_data;

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
                    if (!!function_argument && !!function_argument.reference) {
                        let obj_value = null;
                        let obj_checked = null;
                        try {
                            obj_value= (<HTMLInputElement>obj).value;
                            obj_checked = (<HTMLInputElement>obj).checked;
                        } catch {
                            obj_value = null;
                            obj_checked = null;
                        }
                        let value = resolve_reference(function_argument.reference, ftd_data[id], obj_value, obj_checked);
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

        return window[action.name](...function_arguments, function_arguments, ftd_data[id], id);
    }


    function handle_event(evt: Event, id: string, action: Action, obj: Element) {
        let function_arguments: (FunctionArgument | any)[] = [];
        handle_function(evt, id, action, obj, function_arguments);
        // @ts-ignore
        if (function_arguments["CHANGE_VALUE"] !== false) {
            change_value(function_arguments, ftd_data[id], id);
        }
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

    exports.is_empty = function(str: any) {
        return (!str || str.length === 0 );
    }

    exports.set_list = function(array: any[], value: any[], args: any, data: any, id: string) {
        args["CHANGE_VALUE"]= false;
        window.ftd.clear(array, args, data, id);
        args[0].value = value;
        change_value(args, data, id);
        if (!!window.dummy_data_main && !!window.dummy_data_main[args[0].reference]) {
            // @ts-ignore
            let list = resolve_reference(args[0].reference, data);
            let [htmls, data_id, start_index] = window.dummy_data_main[args[0].reference](data);
            for(let i in htmls){
                let nodes = stringToHTML(htmls[i]);
                let main = document.querySelector(`[data-id="${data_id}"]`);
                for (var j = 0, len = nodes.childElementCount; j < len; ++j) {
                    // @ts-ignore
                    main.insertBefore(nodes.children[j], main.children[start_index + i]);
                }
            }
        }
        return array;
    }

    exports.append = function(array: any[], value: any, args: any, data: any, id: string) {
        array.push(value);
        args["CHANGE_VALUE"]= false;
        args[0].value = array;
        change_value(args, data, id);
        if (!!window.dummy_data_main && !!window.dummy_data_main[args[0].reference]) {
            // @ts-ignore
            let list = resolve_reference(args[0].reference, data);
            let [html, data_id, start_index] = window.dummy_data_main[args[0].reference](data, "LAST");
            let nodes = stringToHTML(html);
            let main = document.querySelector(`[data-id="${data_id}"]`);
            for (var j=0, len = nodes.childElementCount ; j < len; ++j) {
                // @ts-ignore
                main.insertBefore(nodes.children[j], main.children[start_index + list.length - 1]);
            }
        }
        return array;
    }

    exports.insert_at = function(array: any[], value: any, idx: number, args: any, data: any, id: string) {
        array.push(value);
        args["CHANGE_VALUE"]= false;
        args[0].value = array;
        change_value(args, data, id);
        if (!!window.dummy_data_main && !!window.dummy_data_main[args[0].reference]) {
            // @ts-ignore
            let list = resolve_reference(args[0].reference, data);
            let [html, data_id, start_index] = window.dummy_data_main[args[0].reference](data, "LAST");
            let nodes = stringToHTML(html);
            let main = document.querySelector(`[data-id="${data_id}"]`);
            if (idx >= list.length) {
                idx = list.length - 1;
            } else if (idx < 0) {
                idx = 0;
            }
            // @ts-ignore
            main.insertBefore(nodes.children[0], main.children[start_index + idx]);
        }
        return array;
    }

    exports.clear = function(array: any[], args: any, data: any, id: string) {
        args["CHANGE_VALUE"]= false;
        // @ts-ignore
        let length = resolve_reference(args[0].reference, data).length;
        args[0].value = [];
        change_value(args, data, id);
        if (!!window.dummy_data_main && !!window.dummy_data_main[args[0].reference]) {
            let [_, data_id, start_index] = window.dummy_data_main[args[0].reference](data);
            let main = document.querySelector(`[data-id="${data_id}"]`);
            for(var i = length - 1 + start_index; i >= start_index; i--) {
                // @ts-ignore
                main.removeChild(main.children[i]);
            }
        }
        return array;
    }


    exports.delete_at = function(array: any[], idx: number, args: any, data: any, id: string) {
        // @ts-ignore
        let length = resolve_reference(args[0].reference, data).length;
        if (idx >= length) {
            idx = length-1;
        } else if (idx < 0) {
            idx = 0;
        }
        array.splice(idx, 1);
        args["CHANGE_VALUE"]= false;
        args[0].value = array;
        change_value(args, data, id);
        if (!!window.dummy_data_main && !!window.dummy_data_main[args[0].reference]) {
            // @ts-ignore
            let [_, data_id, start_index] = window.dummy_data_main[args[0].reference](data);
            let main = document.querySelector(`[data-id="${data_id}"]`);
            // @ts-ignore
            main.removeChild(main.children[start_index + idx]);
        }
        return array;
    }

    return exports;
})();
