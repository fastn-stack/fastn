let ftd = {
    // source: https://stackoverflow.com/questions/400212/ (cc-by-sa)
    riveNodes: {},
    is_empty(value) {
        value = fastn_utils.getFlattenStaticValue(value);
        return fastn_utils.isNull(value) || value.length === 0;
    },

    len(data) {
        if (!!data && data instanceof fastn.mutableListClass) {
            if (data.getLength)
                return data.getLength();
            return -1;
        }
        if (!!data && data.length) {
            return data.length;
        }
        return -2;
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

    // Todo: Implement this (Remove highlighter)
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

    get(value, index) {
         return fastn_utils.getStaticValue(fastn_utils.getterByKey(value, index));
    },

    component_data(component) {
        let attributesIndex = component.getAttribute(fastn_dom.webComponentArgument);
        let attributes = fastn_dom.webComponent[attributesIndex];
        return Object.fromEntries(
            Object.entries(attributes).map(([k,v]) => {
                // Todo: check if argument is mutable reference or not
                    if (v instanceof fastn.mutableClass) {
                        v = fastn.webComponentVariable.mutable(v);
                    } else if (v instanceof fastn.mutableListClass) {
                        v = fastn.webComponentVariable.mutableList(v);
                    } else if (v instanceof fastn.recordInstanceClass) {
                        v = fastn.webComponentVariable.record(v);
                    } else {
                        v = fastn.webComponentVariable.static(v);
                    }
                    return [k, v];
                }
            )
        );
    }
};

// ftd.append($a = $people, v = Tom)
ftd.append = function (list, item) { list.push(item) }
ftd.pop = function (list) { list.pop() }
ftd.insert_at = function (list, index, item) { list.insertAt(index, item) }
ftd.delete_at = function (list, index) { list.deleteAt(index) }
ftd.clear_all = function (list) { list.clearAll() }
ftd.set_list = function (list, value) { list.set(value) }

ftd.http = function (url, method, body, headers) {
    if (url instanceof fastn.mutableClass) url = url.get();
    if (method instanceof fastn.mutableClass) method = method.get();
    method = method.trim().toUpperCase();
    const init = {
        method,
        headers: {}
    };
    if(headers && headers instanceof fastn.recordInstanceClass) {
        Object.assign(init.headers, headers.toObject());
    }
    if(method !== 'GET') {
        init.headers['Content-Type'] = 'application/json';
    }
    if(body && body instanceof fastn.recordInstanceClass && method !== 'GET') {
        init.body = JSON.stringify(body.toObject());
    }
    fetch(url, init)
    .then(res => {
        if(!res.ok) {
            return new Error("[http]: Request failed", res)
        }

        return res.json();
    })
    .then(json => {
        console.log("[http]: Response OK", json);
    })
    .catch(console.error);
}

ftd.navigate = function(url, request_data) {
    let query_parameters = new URLSearchParams();
    if(request_data instanceof RecordInstance) {
        // @ts-ignore
        for (let [header, value] of Object.entries(request_data.toObject())) {
            if (header != "url" && header != "function" && header != "method") {
                let [key, val] = value.length == 2 ? value : [header, value];
                query_parameters.set(key, val);
            }
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
}

ftd.toggle_dark_mode = function () {
    const is_dark_mode = ftd.get(ftd.dark_mode);
    if(is_dark_mode) {
        enable_light_mode();
    } else {
        enable_dark_mode();
    }
};

const len = ftd.len;

ftd.storage = {
    set(key, value) {
        key = key instanceof fastn.mutableClass ? key.get() : key;

        value = fastn_utils.getFlattenStaticValue(value);

        localStorage.setItem(key, value && typeof value === 'object' ? JSON.stringify(value) : value);
    },
    get(key) {
        key = key instanceof fastn.mutableClass ? key.get() : key;

        if(ssr && !hydrating) {
            return;
        }

        const item = localStorage.getItem(key);

        if(!item) {
            return;
        }

        try {
            const obj = JSON.parse(item);

            return fastn_utils.staticToMutables(obj);
        } catch {
            return item;
        }
    },
    delete(key) {
        key = key instanceof fastn.mutableClass ? key.get() : key;

        localStorage.removeItem(key);
    }
}
