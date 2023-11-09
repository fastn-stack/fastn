const ftd = (function() {
    const exports = {};
    
    const riveNodes = {};

    const global = {};

    exports.global = global;

    exports.riveNodes = riveNodes;

    exports.is_empty = value => {
        value = fastn_utils.getFlattenStaticValue(value);
        return fastn_utils.isNull(value) || value.length === 0;
    };

    exports.len = data => {
        if (!!data && data instanceof fastn.mutableListClass) {
            if (data.getLength)
                return data.getLength();
            return -1;
        }
        if (!!data && data instanceof fastn.mutableClass) {
            let inner_data = data.get();
            return exports.len(inner_data);
        }
        if (!!data && data.length) {
            return data.length;
        }
        return -2;
    };

    exports.copy_to_clipboard = args => {
        let text = args.a;
        if (text instanceof fastn.mutableClass) text = fastn_utils.getStaticValue(text);
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
    };

    // Todo: Implement this (Remove highlighter)
    exports.clean_code = args => args.a;

    exports.set_rive_boolean = (args, node) => {
        if (!!args.rive) {
            let riveNode = riveNodes[`${args.rive}__${exports.device.get()}`];
            node = riveNode ? riveNode: node;
        }
        let riveConst = node.getExtraData().rive;
        const stateMachineName = riveConst.stateMachineNames[0];
        const inputs = riveConst.stateMachineInputs(stateMachineName);
        const bumpTrigger = inputs.find(i => i.name === args.input);
        bumpTrigger.value = args.value;
    };

    exports.toggle_rive_boolean = (args, node) => {
        if (!!args.rive) {
            let riveNode = riveNodes[`${args.rive}__${exports.device.get()}`];
            node = riveNode ? riveNode: node;
        }
        let riveConst = node.getExtraData().rive;
        const stateMachineName = riveConst.stateMachineNames[0];
        const inputs = riveConst.stateMachineInputs(stateMachineName);
        const trigger = inputs.find(i => i.name === args.input);
        trigger.value = !trigger.value;
    };

    exports.set_rive_integer = (args, node) => {
        if (!!args.rive) {
            let riveNode = riveNodes[`${args.rive}__${exports.device.get()}`];
            node = riveNode ? riveNode: node;
        }
        let riveConst = node.getExtraData().rive;
        const stateMachineName = riveConst.stateMachineNames[0];
        const inputs = riveConst.stateMachineInputs(stateMachineName);
        const trigger = inputs.find(i => i.name === args.input);
        trigger.value = args.value;
    };

    exports.fire_rive = (args, node) => {
        if (!!args.rive) {
            let riveNode = riveNodes[`${args.rive}__${exports.device.get()}`];
            node = riveNode ? riveNode: node;
        }
        let riveConst = node.getExtraData().rive;
        const stateMachineName = riveConst.stateMachineNames[0];
        const inputs = riveConst.stateMachineInputs(stateMachineName);
        const trigger = inputs.find(i => i.name === args.input);
        trigger.fire();
    };

    exports.play_rive = (args, node) => {
        if (!!args.rive) {
            let riveNode = riveNodes[`${args.rive}__${exports.device.get()}`];
            node = riveNode ? riveNode: node;
        }
        node.getExtraData().rive.play(args.input);
    };

    exports.pause_rive = (args, node) => {
        if (!!args.rive) {
            let riveNode = riveNodes[`${args.rive}__${exports.device.get()}`];
            node = riveNode ? riveNode: node;
        }
        node.getExtraData().rive.pause(args.input);
    };

    exports.toggle_play_rive = (args, node) => {
        if (!!args.rive) {
            let riveNode = riveNodes[`${args.rive}__${exports.device.get()}`];
            node = riveNode ? riveNode: node;
        }
        let riveConst = node.getExtraData().rive
        riveConst.playingAnimationNames.includes(args.input)
            ? riveConst.pause(args.input)
            : riveConst.play(args.input);
    };

    exports.get = (value, index) => {
         return fastn_utils.getStaticValue(fastn_utils.getterByKey(value, index));
    };

    exports.component_data = component => {
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
    };

    exports.append = function (list, item) { list.push(item) };
    exports.pop = function (list) { list.pop() };
    exports.insert_at = function (list, index, item) { list.insertAt(index, item) };
    exports.delete_at = function (list, index) { list.deleteAt(index) }
    exports.clear_all = function (list) { list.clearAll() }
    exports.clear = exports.clear_all;
    exports.set_list = function (list, value) { list.set(value) }

    exports.http = function (url, method, body, headers) {
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

    exports.navigate = function(url, request_data) {
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

    exports.toggle_dark_mode = function () {
        const is_dark_mode = exports.get(exports.dark_mode);
        if(is_dark_mode) {
            enable_light_mode();
        } else {
            enable_dark_mode();
        }
    };

    exports.local_storage = {
        _get_key(key) {
            if (key instanceof fastn.mutableClass) {
                key = key.get();
            }
            const packageNamePrefix = __fastn_package_name__ ? `${__fastn_package_name__}_` : "";
            const snakeCaseKey = fastn_utils.toSnakeCase(key);
        
            return `${packageNamePrefix}${snakeCaseKey}`;
        },
        set(key, value) {
            key = this._get_key(key);
            value = fastn_utils.getFlattenStaticValue(value);
            localStorage.setItem(key, value && typeof value === 'object' ? JSON.stringify(value) : value);
        },
        get(key) {
            key = this._get_key(key);
            if(ssr) {
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
            key = this._get_key(key);
            localStorage.removeItem(key);
        }
    }

    // LEGACY

    function legacyNameToJS(s) {
        let name = s.toString();
    
        if (name[0].charCodeAt(0) >= 48 && name[0].charCodeAt(0) <= 57) {
            name = '_' + name;
        }
    
        return name
            .replaceAll('#', "__")
            .replaceAll('-', "_")
            .replaceAll(':', "___")
            .replaceAll(',', "$")
            .replaceAll('\\', "/")
            .replaceAll('/', '_')
            .replaceAll('.', "_");
    }

    function getDocNameAndRemaining(s) {
        let part1 = "";
        let patternToSplitAt = s;
        
        const split1 = s.split('#');
        if (split1.length === 2) {
            part1 = split1[0] + '#';
            patternToSplitAt = split1[1];
        }
    
        const split2 = patternToSplitAt.split('.');
        if (split2.length === 2) {
            return [part1 + split2[0], split2[1]];
        } else {
            return [s, null];
        }
    }

    function isMutable(obj) {
        return obj instanceof fastn.mutableClass ||
        obj instanceof fastn.mutableListClass ||
        obj instanceof fastn.recordInstanceClass;
    }

    exports.set_value = function(variable, value) {
        const [var_name, remaining] = getDocNameAndRemaining(variable);
        let name = legacyNameToJS(var_name);
        if(global[name] === undefined) {
            console.log(`[ftd-legacy]: ${variable} is not in global map, ignoring`);
            return;
        }
        const mutable = global[name];
        if(!isMutable(mutable)) {
            console.log(`[ftd-legacy]: ${variable} is not a mutable, ignoring`);
            return;
        }
        if(remaining) {
            mutable.get(remaining).set(value);
        } else {
            mutable.set(value);
        }
    }    

    exports.get_value = function(variable) {
        const [var_name, remaining] = getDocNameAndRemaining(variable);
        let name = legacyNameToJS(var_name);
        if(global[name] === undefined) {
            console.log(`[ftd-legacy]: ${variable} is not in global map, ignoring`);
            return;
        }
        const value = global[name];
        if(isMutable(value)) {
            if(remaining) {
                return value.get(remaining);
            } else {
                return value.get();
            }
        } else {
            return value;
        }
    }

    return exports;
})();

const len = ftd.len;

const global = ftd.global;
