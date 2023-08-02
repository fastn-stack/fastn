let ftd = {
    // source: https://stackoverflow.com/questions/400212/ (cc-by-sa)
    riveNodes: {},
    is_empty(value) {
        value = fastn_utils.getFlattenStaticValue(value);
        return fastn_utils.isNull(value) || value.length === 0;
    },

    copy_to_clipboard(args) {
        let text = args.a;
        if (text.startsWith("\\", 0)) {
            text = text.substring(1);
        }
        if (!navigator.clipboard) {
            fallbackCopyTextToClipboard(text);
            return;
        }
        navigator.clipboard.writeText(text).then(function() {
            console.log('Async: Copying to clipboard was successful!');
        }, function(err) {
            console.error('Async: Could not copy text: ', err);
        });
    },

    clean_code(args) {
        return args.a;
    },

    set_rive_boolean(args, node) {
        if (!!args.rive) {
            let riveNode = ftd.riveNodes[`${args.rive}__${ftd.device.get()}`];
            node = riveNode ? riveNode: node;
        }
        let riveConst = node.getExtraData().rive;
        const stateMachineName = riveConst.stateMachineNames[0];
        const inputs = riveConst.stateMachineInputs(stateMachineName);
        const bumpTrigger = inputs.find(i => i.name === args.input);
        bumpTrigger.value = args.value;
    },

    toggle_rive_boolean(args, node) {
        if (!!args.rive) {
            let riveNode = ftd.riveNodes[`${args.rive}__${ftd.device.get()}`];
            node = riveNode ? riveNode: node;
        }
        let riveConst = node.getExtraData().rive;
        const stateMachineName = riveConst.stateMachineNames[0];
        const inputs = riveConst.stateMachineInputs(stateMachineName);
        const trigger = inputs.find(i => i.name === args.input);
        trigger.value = !trigger.value;
    },

    set_rive_integer(args, node) {
        if (!!args.rive) {
            let riveNode = ftd.riveNodes[`${args.rive}__${ftd.device.get()}`];
            node = riveNode ? riveNode: node;
        }
        let riveConst = node.getExtraData().rive;
        const stateMachineName = riveConst.stateMachineNames[0];
        const inputs = riveConst.stateMachineInputs(stateMachineName);
        const trigger = inputs.find(i => i.name === args.input);
        trigger.value = args.value;
    },

    fire_rive(args, node) {
        if (!!args.rive) {
            let riveNode = ftd.riveNodes[`${args.rive}__${ftd.device.get()}`];
            node = riveNode ? riveNode: node;
        }
        let riveConst = node.getExtraData().rive;
        const stateMachineName = riveConst.stateMachineNames[0];
        const inputs = riveConst.stateMachineInputs(stateMachineName);
        const trigger = inputs.find(i => i.name === args.input);
        trigger.fire();
    },

    play_rive(args, node) {
        if (!!args.rive) {
            let riveNode = ftd.riveNodes[`${args.rive}__${ftd.device.get()}`];
            node = riveNode ? riveNode: node;
        }
        node.getExtraData().rive.play(args.input);
    },

    pause_rive(args, node) {
        if (!!args.rive) {
            let riveNode = ftd.riveNodes[`${args.rive}__${ftd.device.get()}`];
            node = riveNode ? riveNode: node;
        }
        node.getExtraData().rive.pause(args.input);
    },

    toggle_play_rive(args, node) {
        if (!!args.rive) {
            let riveNode = ftd.riveNodes[`${args.rive}__${ftd.device.get()}`];
            node = riveNode ? riveNode: node;
        }
        let riveConst = node.getExtraData().rive
        riveConst.playingAnimationNames.includes(args.input)
            ? riveConst.pause(args.input)
            : riveConst.play(args.input);
    },

};

// ftd.append($a = $people, v = Tom)
ftd.append = function (a, v) { a.push(v) }

ftd.http = function (url, method, ...request_data) {
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
