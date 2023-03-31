
window.ftd = (function() {
    let ftd_data: any = {};
    let exports: Partial<Export> = {};

    // Setting up default value on <input>
    const inputElements = document.querySelectorAll('input[data-dv]');
    for (let input_ele of inputElements) {
        // @ts-ignore
        (<HTMLInputElement> input_ele).defaultValue = input_ele.dataset.dv;
    }

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
        window.ftd.utils.reset_height();
        console_log(id, event);
        let actions = JSON.parse(event);
        for (const action in actions) {
            handle_event(evt, id, actions[action], obj);
        }
        window.ftd.utils.full_height();
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

        let [var_name, remaining] = data[variable] === undefined
            ? get_name_and_remaining(variable)
            : [variable, null];

        if (data[var_name] === undefined && data[variable] === undefined) {
            console_log(variable, "is not in data, ignoring");
            return;
        }

        window.ftd.delete_list(var_name, id);
        if (!!window["set_value_" + id] && !!window["set_value_" + id][var_name]) {
            window["set_value_" + id][var_name](data, value, remaining);
        }
        else {
            set_data_value(data, variable, value);
        }
        window.ftd.create_list(var_name, id);
    };

    exports.is_empty = function(str: any) {
        return (!str || str.length === 0 );
    }

    exports.set_list = function(array: any[], value: any[], args: any, data: any, id: string) {
        args["CHANGE_VALUE"]= false;
        window.ftd.clear(array, args, data, id);
        args[0].value = value;
        change_value(args, data, id);
        window.ftd.create_list(args[0].reference, id);
        return array;
    }

    exports.create_list = function (array_name: string, id: string) {
        if (!!window.dummy_data_main && !!window.dummy_data_main[array_name]) {
            let data = ftd_data[id];
            let dummys = window.dummy_data_main[array_name](data);
            for (let i in dummys) {
                let [htmls, data_id, start_index] = dummys[i];
                for (let i in htmls) {
                    let nodes = stringToHTML(htmls[i]);
                    let main: HTMLElement | null = document.querySelector(`[data-id="${data_id}"]`);
                    main?.insertBefore(nodes.children[0], main.children[start_index + parseInt(i)]);
                    /*for (var j = 0, len = nodes.childElementCount; j < len; ++j) {
                        main?.insertBefore(nodes.children[j], main.children[start_index + parseInt(i)]);
                    }*/
                }
            }
        }
    }

    exports.append = function(array: any[], value: any, args: any, data: any, id: string) {
        array.push(value);
        args["CHANGE_VALUE"]= false;
        args[0].value = array;
        change_value(args, data, id);
        if (!!window.dummy_data_main && !!window.dummy_data_main[args[0].reference]) {
            // @ts-ignore
            let list = resolve_reference(args[0].reference, data);
            let dummys = window.dummy_data_main[args[0].reference](data, "LAST");
            for (let i in dummys) {
                let [html, data_id, start_index]  = dummys[i];
                let nodes = stringToHTML(html);
                let main = document.querySelector(`[data-id="${data_id}"]`);
                for (var j = 0, len = nodes.childElementCount; j < len; ++j) {
                    // @ts-ignore
                    main.insertBefore(nodes.children[j], main.children[start_index + list.length - 1]);
                }
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
            let dummys = window.dummy_data_main[args[0].reference](data, "LAST");
            for (let i in dummys) {
                let [html, data_id, start_index] = dummys[i];
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
        }
        return array;
    }

    exports.clear = function(array: any[], args: any, data: any, id: string) {
        args["CHANGE_VALUE"]= false;
        // @ts-ignore
        window.ftd.delete_list(args[0].reference, id);
        args[0].value = [];
        change_value(args, data, id);
        return array;
    }

    exports.delete_list = function (array_name: string, id: string) {
        if (!!window.dummy_data_main && !!window.dummy_data_main[array_name]) {
            let data = ftd_data[id];
            let length = resolve_reference(array_name, data, null, null).length;
            let dummys = window.dummy_data_main[array_name](data);
            for (let j in dummys) {
                let [_, data_id, start_index] = dummys[j];
                let main: HTMLElement | null = document.querySelector(`[data-id="${data_id}"]`);
                for (var i = length - 1 + start_index; i >= start_index; i--) {
                    main?.removeChild(main.children[i]);
                }
            }
        }
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
            let dummys = window.dummy_data_main[args[0].reference](data);
            for (let i in dummys) {
                let [_, data_id, start_index] = dummys[i];
                let main = document.querySelector(`[data-id="${data_id}"]`);
                main?.removeChild(main.children[start_index + idx]);
            }
        }
        return array;
    }

    exports.http = function(url: string, method: string, ...request_data: any) {
        let method_name = method.trim().toUpperCase();

        if (method_name == "GET") {
            let query_parameters = new URLSearchParams();

            // @ts-ignore
            for (let [header, value] of Object.entries(request_data)) {
                if (header != "url" && header != "function" && header != "method")
                {
                    let [key, val] = value.length == 2 ? value: [header, value];
                    query_parameters.set(key, val);
                }
            }
            let query_string = query_parameters.toString();
            if (query_string) {
                let get_url = url + "?" + query_parameters.toString();
                window.location.href = get_url;
            }
            else{
                window.location.href = url;
            }
            return;
        }

        let json = request_data[0];

        if(request_data.length !== 1 || (request_data[0].length === 2 && Array.isArray(request_data[0]))) {
            let new_json: any = {};

            // @ts-ignore
            for (let [header, value] of Object.entries(request_data)) {
                let [key, val] = value.length == 2 ? value: [header, value];
                new_json[key] = val;
            }
            json = new_json;
        }

        let xhr = new XMLHttpRequest();
        xhr.open(method_name, url);
        xhr.setRequestHeader("Accept", "application/json");
        xhr.setRequestHeader("Content-Type", "application/json");

        xhr.onreadystatechange = function () {
            if (xhr.readyState !== 4) {
                // this means request is still underway
                // https://developer.mozilla.org/en-US/docs/Web/API/XMLHttpRequest/readyState
                return;
            }

            if (xhr.status > 500) {
                console.log("Error in calling url: ", request_data.url, xhr.responseText);
                return;
            }

            let response = JSON.parse(xhr.response);
            if (!!response && !!response.redirect) {
                // Warning: we don't handle header location redirect
                window.location.href = response.redirect;
            } else if (!!response && !!response.reload) {
                window.location.reload();
            } else {
                let data = {};

                if (!!response.errors) {
                    for (let key of Object.keys(response.errors)) {
                        let value = response.errors[key];
                        if (Array.isArray(value)) {
                            // django returns a list of strings
                            value = value.join(" ");
                            // also django does not append `-error`
                            key = key + "-error";
                        }
                        // @ts-ignore
                        data[key] = value;
                    }
                }

                if (!!response.data) {
                    if (!!data) {
                        console_log("both .errrors and .data are present in response, ignoring .data");
                    } else {
                        data = response.data;
                    }
                }

                for (let ftd_variable of Object.keys(data)) {
                    // @ts-ignore
                    window.ftd.set_value(ftd_variable, data[ftd_variable]);
                }
            }
        };
        xhr.send(JSON.stringify(json));
    }

    // source: https://stackoverflow.com/questions/400212/ (cc-by-sa)
    exports.copy_to_clipboard = function (text: string) {
        if (!navigator.clipboard) {
            fallbackCopyTextToClipboard(text);
            return;
        }
        navigator.clipboard.writeText(text).then(function() {
            console.log('Async: Copying to clipboard was successful!');
        }, function(err) {
            console.error('Async: Could not copy text: ', err);
        });
    }

    exports.component_data = function (component: HTMLElement) {
        let data = {};
        for (let idx in component.getAttributeNames()) {
            let argument = component.getAttributeNames()[idx];
            // @ts-ignore
            data[argument] = eval(<string>component.getAttribute(argument));
        }
        return data;
    }

    exports.call_mutable_value_changes = function(key: string, id: string) {
        if (!window.ftd[`mutable_value_${id}`]) {
            return;
        }
        if (!!window.ftd[`mutable_value_${id}`][key]) {
            let changes = window.ftd[`mutable_value_${id}`][key].changes;
            for(let i in changes) { changes[i](); }
        }
        const pattern = new RegExp(`^${key}\\..+`);
        const result = Object.keys(window.ftd[`mutable_value_${id}`])
            .filter(key => pattern.test(key))
            .reduce((acc: Record<string, any>, key) => {
                acc[key] = window.ftd[`mutable_value_${id}`][key];
                return acc;
            }, {});
        for(let i in result) {
            let changes = result[i].changes;
            for(let i in changes) { changes[i](); }
        }
    }

    exports.call_immutable_value_changes = function(key: string, id: string) {
        if (!window.ftd[`immutable_value_${id}`]) {
            return;
        }
        if (!!window.ftd[`immutable_value_${id}`][key]) {
            let changes = window.ftd[`immutable_value_${id}`][key].changes;
            for(let i in changes) { changes[i](); }
        }
        const pattern = new RegExp(`^${key}\\..+`);
        const result = Object.keys(window.ftd[`immutable_value_${id}`])
            .filter(key => pattern.test(key))
            .reduce((acc: Record<string, any>, key) => {
                acc[key] = window.ftd[`immutable_value_${id}`][key];
                return acc;
            }, {});
        for(let i in result) {
            let changes = result[i].changes;
            for(let i in changes) { changes[i](); }
        }
    }

    return exports;
})();
