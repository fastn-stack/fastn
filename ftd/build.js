"use strict";
window.ftd = (function () {
    let ftd_data = {};
    let exports = {};
    // Setting up default value on <input>
    const inputElements = document.querySelectorAll('input[data-dv]');
    for (let input_ele of inputElements) {
        // @ts-ignore
        input_ele.defaultValue = input_ele.dataset.dv;
    }
    exports.init = function (id, data) {
        let element = document.getElementById(data);
        if (!!element) {
            ftd_data[id] = JSON.parse(element.innerText);
            window.ftd.post_init();
        }
    };
    exports.data = ftd_data;
    function handle_function(evt, id, action, obj, function_arguments) {
        console.log(id, action);
        console.log(action.name);
        let argument;
        for (argument in action.values) {
            if (action.values.hasOwnProperty(argument)) {
                // @ts-ignore
                let value = action.values[argument][1] !== undefined ? action.values[argument][1] : action.values[argument];
                if (typeof value === 'object') {
                    let function_argument = value;
                    if (!!function_argument && !!function_argument.reference) {
                        let obj_value = null;
                        let obj_checked = null;
                        try {
                            obj_value = obj.value;
                            obj_checked = obj.checked;
                        }
                        catch (_a) {
                            obj_value = null;
                            obj_checked = null;
                        }
                        let value = resolve_reference(function_argument.reference, ftd_data[id], obj_value, obj_checked);
                        if (!!function_argument.mutable) {
                            function_argument.value = value;
                            function_arguments.push(function_argument);
                        }
                        else {
                            function_arguments.push(deepCopy(value));
                        }
                    }
                }
                else {
                    function_arguments.push(value);
                }
            }
        }
        return window[action.name](...function_arguments, function_arguments, ftd_data[id], id);
    }
    function handle_event(evt, id, action, obj) {
        let function_arguments = [];
        handle_function(evt, id, action, obj, function_arguments);
        // @ts-ignore
        if (function_arguments["CHANGE_VALUE"] !== false) {
            change_value(function_arguments, ftd_data[id], id);
        }
    }
    exports.handle_event = function (evt, id, event, obj) {
        window.ftd.utils.reset_full_height();
        console_log(id, event);
        let actions = JSON.parse(event);
        for (const action in actions) {
            handle_event(evt, id, actions[action], obj);
        }
        window.ftd.utils.set_full_height();
    };
    exports.handle_function = function (evt, id, event, obj) {
        console_log(id, event);
        let actions = JSON.parse(event);
        let function_arguments = [];
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
    exports.is_empty = function (str) {
        return (!str || str.length === 0);
    };
    exports.set_list = function (array, value, args, data, id) {
        args["CHANGE_VALUE"] = false;
        window.ftd.clear(array, args, data, id);
        args[0].value = value;
        change_value(args, data, id);
        window.ftd.create_list(args[0].reference, id);
        return array;
    };
    exports.create_list = function (array_name, id) {
        if (!!window.dummy_data_main && !!window.dummy_data_main[array_name]) {
            let data = ftd_data[id];
            let dummys = window.dummy_data_main[array_name](data);
            for (let i in dummys) {
                let [htmls, data_id, start_index] = dummys[i];
                for (let i in htmls) {
                    let nodes = stringToHTML(htmls[i]);
                    let main = document.querySelector(`[data-id="${data_id}"]`);
                    main === null || main === void 0 ? void 0 : main.insertBefore(nodes.children[0], main.children[start_index + parseInt(i)]);
                    /*for (var j = 0, len = nodes.childElementCount; j < len; ++j) {
                        main?.insertBefore(nodes.children[j], main.children[start_index + parseInt(i)]);
                    }*/
                }
            }
        }
    };
    exports.append = function (array, value, args, data, id) {
        array.push(value);
        args["CHANGE_VALUE"] = false;
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
                for (var j = 0, len = nodes.childElementCount; j < len; ++j) {
                    // @ts-ignore
                    main.insertBefore(nodes.children[j], main.children[start_index + list.length - 1]);
                }
            }
        }
        return array;
    };
    exports.insert_at = function (array, value, idx, args, data, id) {
        array.push(value);
        args["CHANGE_VALUE"] = false;
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
                }
                else if (idx < 0) {
                    idx = 0;
                }
                // @ts-ignore
                main.insertBefore(nodes.children[0], main.children[start_index + idx]);
            }
        }
        return array;
    };
    exports.clear = function (array, args, data, id) {
        args["CHANGE_VALUE"] = false;
        // @ts-ignore
        window.ftd.delete_list(args[0].reference, id);
        args[0].value = [];
        change_value(args, data, id);
        return array;
    };
    exports.delete_list = function (array_name, id) {
        if (!!window.dummy_data_main && !!window.dummy_data_main[array_name]) {
            let data = ftd_data[id];
            let length = resolve_reference(array_name, data, null, null).length;
            let dummys = window.dummy_data_main[array_name](data);
            for (let j in dummys) {
                let [_, data_id, start_index] = dummys[j];
                let main = document.querySelector(`[data-id="${data_id}"]`);
                for (var i = length - 1 + start_index; i >= start_index; i--) {
                    main === null || main === void 0 ? void 0 : main.removeChild(main.children[i]);
                }
            }
        }
    };
    exports.delete_at = function (array, idx, args, data, id) {
        // @ts-ignore
        let length = resolve_reference(args[0].reference, data).length;
        if (idx >= length) {
            idx = length - 1;
        }
        else if (idx < 0) {
            idx = 0;
        }
        array.splice(idx, 1);
        args["CHANGE_VALUE"] = false;
        args[0].value = array;
        change_value(args, data, id);
        if (!!window.dummy_data_main && !!window.dummy_data_main[args[0].reference]) {
            let dummys = window.dummy_data_main[args[0].reference](data);
            for (let i in dummys) {
                let [_, data_id, start_index] = dummys[i];
                let main = document.querySelector(`[data-id="${data_id}"]`);
                main === null || main === void 0 ? void 0 : main.removeChild(main.children[start_index + idx]);
            }
        }
        return array;
    };
    exports.http = function (url, method, ...request_data) {
        let method_name = method.trim().toUpperCase();
        if (method_name == "GET") {
            let query_parameters = new URLSearchParams();
            // @ts-ignore
            for (let [header, value] of Object.entries(request_data)) {
                if (header != "url" && header != "function" && header != "method") {
                    let [key, val] = value.length == 2 ? value : [header, value];
                    query_parameters.set(key, val);
                }
            }
            let query_string = query_parameters.toString();
            if (query_string) {
                let get_url = url + "?" + query_parameters.toString();
                window.location.href = get_url;
            }
            else {
                window.location.href = url;
            }
            return;
        }
        let json = request_data[0];
        if (request_data.length !== 1 || (request_data[0].length === 2 && Array.isArray(request_data[0]))) {
            let new_json = {};
            // @ts-ignore
            for (let [header, value] of Object.entries(request_data)) {
                let [key, val] = value.length == 2 ? value : [header, value];
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
            }
            else if (!!response && !!response.reload) {
                window.location.reload();
            }
            else {
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
                    }
                    else {
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
    };
    // source: https://stackoverflow.com/questions/400212/ (cc-by-sa)
    exports.copy_to_clipboard = function (text) {
        if (text.startsWith("\\", 0)) {
            text = text.substring(1);
        }
        if (!navigator.clipboard) {
            fallbackCopyTextToClipboard(text);
            return;
        }
        navigator.clipboard.writeText(text).then(function () {
            console.log('Async: Copying to clipboard was successful!');
        }, function (err) {
            console.error('Async: Could not copy text: ', err);
        });
    };
    exports.set_rive_boolean = function (canva_id, input, value, args, data, id) {
        let canva_with_id = canva_id + ":" + id;
        let rive_const = window.ftd.utils.function_name_to_js_function(canva_with_id);
        const stateMachineName = window[rive_const].stateMachineNames[0];
        const inputs = window[rive_const].stateMachineInputs(stateMachineName);
        // @ts-ignore
        const bumpTrigger = inputs.find(i => i.name === input);
        bumpTrigger.value = value;
    };
    exports.toggle_rive_boolean = function (canva_id, input, args, data, id) {
        let canva_with_id = canva_id + ":" + id;
        let rive_const = window.ftd.utils.function_name_to_js_function(canva_with_id);
        const stateMachineName = window[rive_const].stateMachineNames[0];
        const inputs = window[rive_const].stateMachineInputs(stateMachineName);
        // @ts-ignore
        const trigger = inputs.find(i => i.name === input);
        trigger.value = !trigger.value;
    };
    exports.set_rive_integer = function (canva_id, input, value, args, data, id) {
        let canva_with_id = canva_id + ":" + id;
        let rive_const = window.ftd.utils.function_name_to_js_function(canva_with_id);
        const stateMachineName = window[rive_const].stateMachineNames[0];
        const inputs = window[rive_const].stateMachineInputs(stateMachineName);
        // @ts-ignore
        const bumpTrigger = inputs.find(i => i.name === input);
        bumpTrigger.value = value;
    };
    exports.fire_rive = function (canva_id, input, args, data, id) {
        let canva_with_id = canva_id + ":" + id;
        let rive_const = window.ftd.utils.function_name_to_js_function(canva_with_id);
        const stateMachineName = window[rive_const].stateMachineNames[0];
        const inputs = window[rive_const].stateMachineInputs(stateMachineName);
        // @ts-ignore
        const bumpTrigger = inputs.find(i => i.name === input);
        bumpTrigger.fire();
    };
    exports.play_rive = function (canva_id, input, args, data, id) {
        let canva_with_id = canva_id + ":" + id;
        let rive_const = window.ftd.utils.function_name_to_js_function(canva_with_id);
        window[rive_const].play(input);
    };
    exports.pause_rive = function (canva_id, input, args, data, id) {
        let canva_with_id = canva_id + ":" + id;
        let rive_const = window.ftd.utils.function_name_to_js_function(canva_with_id);
        window[rive_const].pause(input);
    };
    exports.toggle_play_rive = function (canva_id, input, args, data, id) {
        let canva_with_id = canva_id + ":" + id;
        let rive_const = window.ftd.utils.function_name_to_js_function(canva_with_id);
        let r = window[rive_const];
        r.playingAnimationNames.includes(input)
            ? r.pause(input)
            : r.play(input);
    };
    exports.component_data = function (component) {
        let data = {};
        for (let idx in component.getAttributeNames()) {
            let argument = component.getAttributeNames()[idx];
            // @ts-ignore
            data[argument] = eval(component.getAttribute(argument));
        }
        return data;
    };
    exports.call_mutable_value_changes = function (key, id) {
        if (!window.ftd[`mutable_value_${id}`]) {
            return;
        }
        if (!!window.ftd[`mutable_value_${id}`][key]) {
            let changes = window.ftd[`mutable_value_${id}`][key].changes;
            for (let i in changes) {
                changes[i]();
            }
        }
        const pattern = new RegExp(`^${key}\\..+`);
        const result = Object.keys(window.ftd[`mutable_value_${id}`])
            .filter(key => pattern.test(key))
            .reduce((acc, key) => {
            acc[key] = window.ftd[`mutable_value_${id}`][key];
            return acc;
        }, {});
        for (let i in result) {
            let changes = result[i].changes;
            for (let i in changes) {
                changes[i]();
            }
        }
    };
    exports.call_immutable_value_changes = function (key, id) {
        if (!window.ftd[`immutable_value_${id}`]) {
            return;
        }
        if (!!window.ftd[`immutable_value_${id}`][key]) {
            let changes = window.ftd[`immutable_value_${id}`][key].changes;
            for (let i in changes) {
                changes[i]();
            }
        }
        const pattern = new RegExp(`^${key}\\..+`);
        const result = Object.keys(window.ftd[`immutable_value_${id}`])
            .filter(key => pattern.test(key))
            .reduce((acc, key) => {
            acc[key] = window.ftd[`immutable_value_${id}`][key];
            return acc;
        }, {});
        for (let i in result) {
            let changes = result[i].changes;
            for (let i in changes) {
                changes[i]();
            }
        }
    };
    return exports;
})();
window.ftd.post_init = function () {
    const DARK_MODE = "ftd#dark-mode";
    const SYSTEM_DARK_MODE = "ftd#system-dark-mode";
    const FOLLOW_SYSTEM_DARK_MODE = "ftd#follow-system-dark-mode";
    const DARK_MODE_COOKIE = "ftd-dark-mode";
    const COOKIE_SYSTEM_LIGHT = "system-light";
    const COOKIE_SYSTEM_DARK = "system-dark";
    const COOKIE_DARK_MODE = "dark";
    const COOKIE_LIGHT_MODE = "light";
    const DARK_MODE_CLASS = "fpm-dark";
    const MOBILE_CLASS = "ftd-mobile";
    const XL_CLASS = "ftd-xl";
    const FTD_DEVICE = "ftd#device";
    const FTD_BREAKPOINT_WIDTH = "ftd#breakpoint-width";
    let last_device;
    function initialise_device() {
        last_device = get_device();
        console_log("last_device", last_device);
        window.ftd.set_string_for_all(FTD_DEVICE, last_device);
    }
    window.onresize = function () {
        let current = get_device();
        if (current === last_device) {
            return;
        }
        window.ftd.set_string_for_all(FTD_DEVICE, current);
        last_device = current;
        console_log("last_device", last_device);
    };
    /*function update_markdown_colors() {
       // remove all colors from ftd.css: copy every deleted stuff in this function
       let markdown_style_sheet = document.createElement('style');


       markdown_style_sheet.innerHTML = `
       .ft_md a {
           color: ${window.ftd.get_value("main", MARKDOWN_COLOR + ".link.light")};
           background-color: ${window.ftd.get_value("main", MARKDOWN_BACKGROUND_COLOR + ".link.light")};
       }
       body.fpm-dark .ft_md a {
           color: ${window.ftd.get_value("main", MARKDOWN_COLOR + ".link.dark")};
           background-color: ${window.ftd.get_value("main", MARKDOWN_BACKGROUND_COLOR + ".link.dark")};
       }

       .ft_md code {
           color: ${window.ftd.get_value("main", MARKDOWN_COLOR + ".code.light")};
           background-color: ${window.ftd.get_value("main", MARKDOWN_BACKGROUND_COLOR + ".code.light")};
       }
       body.fpm-dark .ft_md code {
           color: ${window.ftd.get_value("main", MARKDOWN_COLOR + ".code.dark")};
           background-color: ${window.ftd.get_value("main", MARKDOWN_BACKGROUND_COLOR + ".code.dark")};
       }

       .ft_md a:visited {
           color: ${window.ftd.get_value("main", MARKDOWN_COLOR + ".link-visited.light")};
           background-color: ${window.ftd.get_value("main", MARKDOWN_BACKGROUND_COLOR + ".link-visited.light")};
       }
       body.fpm-dark .ft_md a:visited {
           color: ${window.ftd.get_value("main", MARKDOWN_COLOR + ".link-visited.dark")};
           background-color: ${window.ftd.get_value("main", MARKDOWN_BACKGROUND_COLOR + ".link-visited.dark")};
       }

       .ft_md a code {
           color: ${window.ftd.get_value("main", MARKDOWN_COLOR + ".link-code.light")};
           background-color: ${window.ftd.get_value("main", MARKDOWN_BACKGROUND_COLOR + ".link-code.light")};
       }
       body.fpm-dark .ft_md a code {
           color: ${window.ftd.get_value("main", MARKDOWN_COLOR + ".link-code.dark")};
           background-color: ${window.ftd.get_value("main", MARKDOWN_BACKGROUND_COLOR + ".link-code.dark")};
       }

       .ft_md a:visited code {
           color: ${window.ftd.get_value("main", MARKDOWN_COLOR + ".link-visited-code.light")};
           background-color: ${window.ftd.get_value("main", MARKDOWN_BACKGROUND_COLOR + ".link-visited-code.light")};
       }
       body.fpm-dark .ft_md a:visited code {
           color: ${window.ftd.get_value("main", MARKDOWN_COLOR + ".link-visited-code.dark")};
           background-color: ${window.ftd.get_value("main", MARKDOWN_BACKGROUND_COLOR + ".link-visited-code.dark")};
       }

       .ft_md ul ol li:before {
           color: ${window.ftd.get_value("main", MARKDOWN_COLOR + ".ul-ol-li-before.light")};
           background-color: ${window.ftd.get_value("main", MARKDOWN_BACKGROUND_COLOR + ".ul-ol-li-before.light")};
       }
       body.fpm-dark .ft_md ul ol li:before {
           color: ${window.ftd.get_value("main", MARKDOWN_COLOR + ".ul-ol-li-before.dark")};
           background-color: ${window.ftd.get_value("main", MARKDOWN_BACKGROUND_COLOR + ".ul-ol-li-before.dark")};
       }
       `;

       document.getElementsByTagName('head')[0].appendChild(markdown_style_sheet);
   }*/
    function get_device() {
        // not at all sure about this functions logic.
        let width = window.innerWidth;
        // in future we may want to have more than one break points, and then
        // we may also want the theme builders to decide where the breakpoints
        // should go. we should be able to fetch fpm variables here, or maybe
        // simply pass the width, user agent etc to fpm and let people put the
        // checks on width user agent etc, but it would be good if we can
        // standardize few breakpoints. or maybe we should do both, some
        // standard breakpoints and pass the raw data.
        // we would then rename this function to detect_device() which will
        // return one of "desktop", "tablet", "mobile". and also maybe have
        // another function detect_orientation(), "landscape" and "portrait" etc,
        // and instead of setting `fpm#mobile: boolean` we set `fpm-ui#device`
        // and `fpm#view-port-orientation` etc.
        let mobile_breakpoint = window.ftd.get_value("main", FTD_BREAKPOINT_WIDTH + ".mobile");
        if (width <= mobile_breakpoint) {
            document.body.classList.add(MOBILE_CLASS);
            if (document.body.classList.contains(XL_CLASS)) {
                document.body.classList.remove(XL_CLASS);
            }
            return "mobile";
        }
        /*if (width > desktop_breakpoint) {
            document.body.classList.add(XL_CLASS);
            if (document.body.classList.contains(MOBILE_CLASS)) {
                document.body.classList.remove(MOBILE_CLASS);
            }
            return "xl";
        }*/
        if (document.body.classList.contains(MOBILE_CLASS)) {
            document.body.classList.remove(MOBILE_CLASS);
        }
        /*if (document.body.classList.contains(XL_CLASS)) {
            document.body.classList.remove(XL_CLASS);
        }*/
        return "desktop";
    }
    /*
        ftd.dark-mode behaviour:

        ftd.dark-mode is a boolean, default false, it tells the UI to show
        the UI in dark or light mode. Themes should use this variable to decide
        which mode to show in UI.

        ftd.follow-system-dark-mode, boolean, default true, keeps track if
        we are reading the value of `dark-mode` from system preference, or user
        has overridden the system preference.

        These two variables must not be set by ftd code directly, but they must
        use `$on-click$: message-host enable-dark-mode`, to ignore system
        preference and use dark mode. `$on-click$: message-host
        disable-dark-mode` to ignore system preference and use light mode and
        `$on-click$: message-host follow-system-dark-mode` to ignore user
        preference and start following system preference.

        we use a cookie: `ftd-dark-mode` to store the preference. The cookie can
        have three values:

           cookie missing /          user wants us to honour system preference
               system-light          and currently its light.

           system-dark               follow system and currently its dark.

           light:                    user prefers light

           dark:                     user prefers light

        We use cookie instead of localstorage so in future `fpm-repo` can see
        users preferences up front and renders the HTML on service wide
        following user's preference.

     */
    window.enable_dark_mode = function () {
        // TODO: coalesce the two set_bool-s into one so there is only one DOM
        //       update
        window.ftd.set_bool_for_all(DARK_MODE, true);
        window.ftd.set_bool_for_all(FOLLOW_SYSTEM_DARK_MODE, false);
        window.ftd.set_bool_for_all(SYSTEM_DARK_MODE, system_dark_mode());
        document.body.classList.add(DARK_MODE_CLASS);
        set_cookie(DARK_MODE_COOKIE, COOKIE_DARK_MODE);
    };
    window.enable_light_mode = function () {
        // TODO: coalesce the two set_bool-s into one so there is only one DOM
        //       update
        window.ftd.set_bool_for_all(DARK_MODE, false);
        window.ftd.set_bool_for_all(FOLLOW_SYSTEM_DARK_MODE, false);
        window.ftd.set_bool_for_all(SYSTEM_DARK_MODE, system_dark_mode());
        if (document.body.classList.contains(DARK_MODE_CLASS)) {
            document.body.classList.remove(DARK_MODE_CLASS);
        }
        set_cookie(DARK_MODE_COOKIE, COOKIE_LIGHT_MODE);
    };
    window.enable_system_mode = function () {
        // TODO: coalesce the two set_bool-s into one so there is only one DOM
        //       update
        window.ftd.set_bool_for_all(FOLLOW_SYSTEM_DARK_MODE, true);
        window.ftd.set_bool_for_all(SYSTEM_DARK_MODE, system_dark_mode());
        if (system_dark_mode()) {
            window.ftd.set_bool_for_all(DARK_MODE, true);
            document.body.classList.add(DARK_MODE_CLASS);
            set_cookie(DARK_MODE_COOKIE, COOKIE_SYSTEM_DARK);
        }
        else {
            window.ftd.set_bool_for_all(DARK_MODE, false);
            if (document.body.classList.contains(DARK_MODE_CLASS)) {
                document.body.classList.remove(DARK_MODE_CLASS);
            }
            set_cookie(DARK_MODE_COOKIE, COOKIE_SYSTEM_LIGHT);
        }
    };
    function set_cookie(name, value) {
        document.cookie = name + "=" + value + "; path=/";
    }
    function system_dark_mode() {
        return !!(window.matchMedia && window.matchMedia('(prefers-color-scheme: dark)').matches);
    }
    function initialise_dark_mode() {
        update_dark_mode();
        start_watching_dark_mode_system_preference();
    }
    function get_cookie(name, def) {
        // source: https://stackoverflow.com/questions/5639346/
        let regex = document.cookie.match('(^|;)\\s*' + name + '\\s*=\\s*([^;]+)');
        return regex !== null ? regex.pop() : def;
    }
    function update_dark_mode() {
        let current_dark_mode_cookie = get_cookie(DARK_MODE_COOKIE, COOKIE_SYSTEM_LIGHT);
        switch (current_dark_mode_cookie) {
            case COOKIE_SYSTEM_LIGHT:
            case COOKIE_SYSTEM_DARK:
                window.enable_system_mode();
                break;
            case COOKIE_LIGHT_MODE:
                window.enable_light_mode();
                break;
            case COOKIE_DARK_MODE:
                window.enable_dark_mode();
                break;
            default:
                console_log("cookie value is wrong", current_dark_mode_cookie);
                window.enable_system_mode();
        }
    }
    function start_watching_dark_mode_system_preference() {
        window.matchMedia('(prefers-color-scheme: dark)').addEventListener("change", update_dark_mode);
    }
    initialise_dark_mode();
    initialise_device();
    window.ftd.utils.set_full_height();
    // update_markdown_colors();
};
const DEVICE_SUFFIX = "____device";
function console_log(...message) {
    if (true) { // false
        console.log(...message);
    }
}
function isObject(obj) {
    return obj != null && typeof obj === 'object' && obj === Object(obj);
}
function stringToHTML(str) {
    var parser = new DOMParser();
    var doc = parser.parseFromString(str, 'text/html');
    return doc.body;
}
;
function get_name_and_remaining(name) {
    let part1 = "";
    let pattern_to_split_at = name;
    let parent_split = split_once(name, "#");
    if (parent_split.length === 2) {
        part1 = parent_split[0] + "#";
        pattern_to_split_at = parent_split[1];
    }
    parent_split = split_once(pattern_to_split_at, ".");
    if (parent_split.length === 2) {
        return [part1 + parent_split[0], parent_split[1]];
    }
    return [name, null];
}
function split_once(name, split_at) {
    const i = name.indexOf(split_at);
    if (i === -1) {
        return [name];
    }
    return [name.slice(0, i), name.slice(i + 1)];
}
function deepCopy(object) {
    if (isObject(object)) {
        return JSON.parse(JSON.stringify(object));
    }
    return object;
}
function change_value(function_arguments, data, id) {
    for (const a in function_arguments) {
        if (isFunctionArgument(function_arguments[a])) {
            if (!!function_arguments[a]["reference"]) {
                let reference = function_arguments[a]["reference"];
                let [var_name, remaining] = (!!data[reference]) ? [reference, null] : get_name_and_remaining(reference);
                if (var_name === "ftd#dark-mode") {
                    if (!!function_arguments[a]["value"]) {
                        window.enable_dark_mode();
                    }
                    else {
                        window.enable_light_mode();
                    }
                }
                else if (!!window["set_value_" + id] && !!window["set_value_" + id][var_name]) {
                    window["set_value_" + id][var_name](data, function_arguments[a]["value"], remaining);
                }
                else {
                    set_data_value(data, reference, function_arguments[a]["value"]);
                }
            }
        }
    }
}
function isFunctionArgument(object) {
    return object.value !== undefined;
}
String.prototype.format = function () {
    var formatted = this;
    for (var i = 0; i < arguments.length; i++) {
        var regexp = new RegExp('\\{' + i + '\\}', 'gi');
        formatted = formatted.replace(regexp, arguments[i]);
    }
    return formatted;
};
String.prototype.replace_format = function () {
    var formatted = this;
    if (arguments.length > 0) {
        // @ts-ignore
        for (let [header, value] of Object.entries(arguments[0])) {
            var regexp = new RegExp('\\{(' + header + '(\\..*?)?)\\}', 'gi');
            let matching = formatted.match(regexp);
            for (let i in matching) {
                try {
                    // @ts-ignore
                    formatted = formatted.replace(matching[i], resolve_reference(matching[i].substring(1, matching[i].length - 1), arguments[0]));
                }
                catch (e) {
                    continue;
                }
            }
        }
    }
    return formatted;
};
function set_data_value(data, name, value) {
    if (!!data[name]) {
        data[name] = deepCopy(set(data[name], null, value));
        return;
    }
    let [var_name, remaining] = get_name_and_remaining(name);
    let initial_value = data[var_name];
    data[var_name] = deepCopy(set(initial_value, remaining, value));
    // tslint:disable-next-line:no-shadowed-variable
    function set(initial_value, remaining, value) {
        if (!remaining) {
            return value;
        }
        let [p1, p2] = split_once(remaining, ".");
        initial_value[p1] = set(initial_value[p1], p2, value);
        return initial_value;
    }
}
function resolve_reference(reference, data, value, checked) {
    if (reference === "VALUE") {
        return value;
    }
    if (reference === "CHECKED") {
        return checked;
    }
    if (!!data[reference]) {
        return deepCopy(data[reference]);
    }
    let [var_name, remaining] = get_name_and_remaining(reference);
    let initial_value = data[var_name];
    while (!!remaining) {
        let [p1, p2] = split_once(remaining, ".");
        initial_value = initial_value[p1];
        remaining = p2;
    }
    return deepCopy(initial_value);
}
function get_data_value(data, name) {
    return resolve_reference(name, data, null, null);
}
function JSONstringify(f) {
    if (typeof f === 'object') {
        return JSON.stringify(f);
    }
    else {
        return f;
    }
}
function download_text(filename, text) {
    const blob = new Blob([text], { type: 'text/plain' });
    const link = document.createElement('a');
    link.href = window.URL.createObjectURL(blob);
    link.download = filename;
    link.click();
}
function len(data) {
    return data.length;
}
function fallbackCopyTextToClipboard(text) {
    const textArea = document.createElement("textarea");
    textArea.value = text;
    // Avoid scrolling to bottom
    textArea.style.top = "0";
    textArea.style.left = "0";
    textArea.style.position = "fixed";
    document.body.appendChild(textArea);
    textArea.focus();
    textArea.select();
    try {
        const successful = document.execCommand('copy');
        const msg = successful ? 'successful' : 'unsuccessful';
        console.log('Fallback: Copying text command was ' + msg);
    }
    catch (err) {
        console.error('Fallback: Oops, unable to copy', err);
    }
    textArea.remove();
}
window.ftd.utils = {};
window.ftd.utils.set_full_height = function () {
    document.body.style.height = `max(${document.documentElement.scrollHeight}px, 100%)`;
};
window.ftd.utils.reset_full_height = function () {
    document.body.style.height = `100%`;
};
window.ftd.utils.get_event_key = function (event) {
    if (65 <= event.keyCode && event.keyCode <= 90) {
        return String.fromCharCode(event.keyCode).toLowerCase();
    }
    else {
        return event.key;
    }
};
window.ftd.utils.function_name_to_js_function = function (s) {
    let new_string = s;
    let startsWithDigit = /^\d/.test(s);
    if (startsWithDigit) {
        new_string = "_" + s;
    }
    new_string = new_string.replace('#', "__").replace('-', "_")
        .replace(':', "___")
        .replace(',', "$")
        .replace("\\\\", "/")
        .replace('\\', "/")
        .replace('/', "_").replace('.', "_");
    return new_string;
};
window.ftd.utils.node_change_call = function (id, key, data) {
    const node_function = `node_change_${id}`;
    const target = window[node_function];
    if (!!target && !!target[key]) {
        target[key](data);
    }
};
window.ftd.utils.set_value_helper = function (data, key, remaining, new_value) {
    if (!!remaining) {
        set_data_value(data, `${key}.${remaining}`, new_value);
    }
    else {
        set_data_value(data, key, new_value);
    }
};
window.ftd.dependencies = {};
window.ftd.dependencies.eval_background_size = function (bg) {
    if (typeof bg === 'object' && !!bg && "size" in bg) {
        let sz = bg.size;
        if (typeof sz === 'object' && !!sz && "x" in sz && "y" in sz) {
            return `${sz.x} ${sz.y}`;
        }
        else {
            return sz;
        }
    }
    else {
        return null;
    }
};
window.ftd.dependencies.eval_background_position = function (bg) {
    if (typeof bg === 'object' && !!bg && "position" in bg) {
        let pos = bg.position;
        if (typeof pos === 'object' && !!pos && "x" in pos && "y" in pos) {
            return `${pos.x} ${pos.y}`;
        }
        else {
            return pos.replace("-", " ");
        }
    }
    else {
        return null;
    }
};
window.ftd.dependencies.eval_background_repeat = function (bg) {
    if (typeof bg === 'object' && !!bg && "repeat" in bg) {
        return bg.repeat;
    }
    else {
        return null;
    }
};
window.ftd.dependencies.eval_background_color = function (bg, data) {
    let img_src = bg;
    if (!data["ftd#dark-mode"] && typeof img_src === 'object' && !!img_src && "light" in img_src) {
        return img_src.light;
    }
    else if (data["ftd#dark-mode"] && typeof img_src === 'object' && !!img_src && "dark" in img_src) {
        return img_src.dark;
    }
    else if (typeof img_src === 'string' && !!img_src) {
        return img_src;
    }
    else {
        return null;
    }
};
window.ftd.dependencies.eval_background_image = function (bg, data) {
    var _a;
    if (typeof bg === 'object' && !!bg && "src" in bg) {
        let img_src = bg.src;
        if (!data["ftd#dark-mode"] && typeof img_src === 'object' && !!img_src && "light" in img_src) {
            return `url("${img_src.light}")`;
        }
        else if (data["ftd#dark-mode"] && typeof img_src === 'object' && !!img_src && "dark" in img_src) {
            return `url("${img_src.dark}")`;
        }
        else {
            return null;
        }
    }
    else if (typeof bg === 'object' && !!bg && "colors" in bg && Object.keys(bg.colors).length) {
        let colors = "";
        // if the bg direction is provided by the user, use it, otherwise default
        let direction = (_a = bg.direction) !== null && _a !== void 0 ? _a : "to bottom";
        let colors_vec = bg.colors;
        for (const c of colors_vec) {
            if (typeof c === 'object' && !!c && "color" in c) {
                let color_value = c.color;
                if (typeof color_value === 'object' && !!color_value && "light" in color_value && "dark" in color_value) {
                    if (colors) {
                        colors = data["ftd#dark-mode"] ? `${colors}, ${color_value.dark}` : `${colors}, ${color_value.light}`;
                    }
                    else {
                        colors = data["ftd#dark-mode"] ? `${color_value.dark}` : `${color_value.light}`;
                    }
                    if ("start" in c)
                        colors = `${colors} ${c.start}`;
                    if ("end" in c)
                        colors = `${colors} ${c.end}`;
                    if ("stop-position" in c)
                        colors = `${colors}, ${c["stop-position"]}`;
                }
            }
        }
        let res = `linear-gradient(${direction}, ${colors})`;
        return res;
    }
    else {
        return null;
    }
};
window.ftd.dependencies.eval_box_shadow = function (shadow, data) {
    if (typeof shadow === 'object' && !!shadow) {
        let inset, blur, spread, x_off, y_off, color;
        inset = "";
        blur = spread = x_off = y_off = "0px";
        color = "black";
        if (("inset" in shadow) && shadow.inset)
            inset = "inset";
        if ("blur" in shadow)
            blur = shadow.blur;
        if ("spread" in shadow)
            spread = shadow.spread;
        if ("x-offset" in shadow)
            x_off = shadow["x-offset"];
        if ("y-offset" in shadow)
            y_off = shadow["y-offset"];
        if ("color" in shadow) {
            if (data["ftd#dark-mode"]) {
                color = shadow.color.dark;
            }
            else {
                color = shadow.color.light;
            }
        }
        // inset, color, x_offset, y_offset, blur, spread
        let res = `${inset} ${color} ${x_off} ${y_off} ${blur} ${spread}`.trim();
        return res;
    }
    else {
        return null;
    }
};
window.ftd.utils.add_extra_in_id = function (node_id) {
    let element = document.querySelector(`[data-id=\"${node_id}\"]`);
    if (element) {
        changeElementId(element, DEVICE_SUFFIX, true);
    }
};
window.ftd.utils.remove_extra_from_id = function (node_id) {
    let element = document.querySelector(`[data-id=\"${node_id}\"]`);
    if (element) {
        changeElementId(element, DEVICE_SUFFIX, false);
    }
};
function changeElementId(element, suffix, add) {
    // check if the current ID is not empty
    if (element.id) {
        // set the new ID for the element
        element.id = updatedID(element.id, add, suffix);
    }
    // get all the children nodes of the element
    // @ts-ignore
    const childrenNodes = element.children;
    // loop through all the children nodes
    for (let i = 0; i < childrenNodes.length; i++) {
        // get the current child node
        const currentNode = childrenNodes[i];
        // recursively call this function for the current child node
        changeElementId(currentNode, suffix, add);
    }
}
function updatedID(str, flag, suffix) {
    // check if the flag is set
    if (flag) {
        // append suffix to the string
        return `${str} ${suffix}`;
    }
    else {
        // remove suffix from the string (if it exists)
        return str.replace(suffix, "");
    }
}
