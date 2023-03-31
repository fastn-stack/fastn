function console_log(...message: any) {
    if (true) { // false
        console.log(...message);
    }
}

function isObject(obj: object) {
    return obj != null && typeof obj === 'object' && obj === Object(obj);
}

function stringToHTML(str: string) {
    var parser = new DOMParser();
    var doc = parser.parseFromString(str, 'text/html');
    return doc.body;
};

function get_name_and_remaining(name: string): [string, string | null] {
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


function split_once(name: string, split_at: string) {
    const i = name.indexOf(split_at);
    if (i === -1) {
        return [name];
    }
    return [name.slice(0, i), name.slice(i + 1)];
}

function deepCopy(object: any) {
    if (isObject(object)) {
        return JSON.parse(JSON.stringify(object));
    }
    return object;
}

function change_value(function_arguments: (FunctionArgument | any)[], data: {
    [key: string]: any;
}, id: string) {
    for (const a in function_arguments) {
        if (isFunctionArgument(function_arguments[a])) {
            if (!!function_arguments[a]["reference"]) {
                let reference: string = <string>function_arguments[a]["reference"];
                let [var_name, remaining] = (!!data[reference]) ? [reference, null] : get_name_and_remaining(reference);
                if (var_name === "ftd#dark-mode") {
                    if (!!function_arguments[a]["value"]) {
                        window.enable_dark_mode();
                    } else {
                        window.enable_light_mode();
                    }
                } else if (!!window["set_value_" + id] && !!window["set_value_" + id][var_name]) {
                    window["set_value_" + id][var_name](data, function_arguments[a]["value"], remaining);
                } else {
                    set_data_value(data, reference, function_arguments[a]["value"]);
                }
            }
        }
    }
}

function isFunctionArgument(object: any): object is FunctionArgument {
    return (<FunctionArgument>object).value !== undefined;
}

String.prototype.format = function() {
    var formatted = this;
    for (var i = 0; i < arguments.length; i++) {
        var regexp = new RegExp('\\{'+i+'\\}', 'gi');
        formatted = formatted.replace(regexp, arguments[i]);
    }
    return formatted;
};


String.prototype.replace_format = function() {
    var formatted = this;
    if (arguments.length > 0){
        // @ts-ignore
        for (let [header, value] of Object.entries(arguments[0])) {
            var regexp = new RegExp('\\{('+header+'(\\..*?)?)\\}', 'gi');
            let matching = formatted.match(regexp);
            for(let i in matching) {
                try {
                    // @ts-ignore
                    formatted = formatted.replace(matching[i], resolve_reference(matching[i].substring(1, matching[i].length -1), arguments[0]));
                } catch (e) {
                    continue
                }

            }
        }
    }
    return formatted;
};


function set_data_value(data: any, name: string, value: any) {
    if (!!data[name]) {
        data[name] = deepCopy(set(data[name], null, value));
        return;
    }
    let [var_name, remaining] = get_name_and_remaining(name);
    let initial_value = data[var_name];
    data[var_name] = deepCopy(set(initial_value, remaining, value));

    // tslint:disable-next-line:no-shadowed-variable
    function set(initial_value: any, remaining: string | null, value: string) {
        if (!remaining) {
            return value;
        }
        let [p1, p2] = split_once(remaining, ".");
        initial_value[p1] = set(initial_value[p1], p2, value);
        return initial_value;
    }
}

function resolve_reference(reference: string, data: any, value: any, checked: any) {
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


function get_data_value(data: any, name: string) {
    return resolve_reference(name, data, null, null)
}

function JSONstringify(f: any) {
    if(typeof f === 'object') {
        return JSON.stringify(f);
    } else {
        return f;
    }
}

function download_text(filename: string, text: string){
    const blob = new Blob([text], { type: 'text/plain' });
    const link = document.createElement('a');
    link.href = window.URL.createObjectURL(blob);
    link.download = filename;
    link.click();
}

function len(data: any[]) {
    return data.length;
}

function fallbackCopyTextToClipboard(text: string) {
    var textArea = document.createElement("textarea");
    textArea.value = text;

    // Avoid scrolling to bottom
    textArea.style.top = "0";
    textArea.style.left = "0";
    textArea.style.position = "fixed";

    document.body.appendChild(textArea);
    textArea.focus();
    textArea.select();

    try {
        var successful = document.execCommand('copy');
        var msg = successful ? 'successful' : 'unsuccessful';
        console.log('Fallback: Copying text command was ' + msg);
    } catch (err) {
        console.error('Fallback: Oops, unable to copy', err);
    }

    document.body.removeChild(textArea);
}

window.ftd.utils = {};
window.ftd.utils.full_height = function () {
    document.body.style.height = `max(${document.documentElement.scrollHeight}px, 100%)`;
};

window.ftd.utils.reset_height = function () {
    document.body.style.height = `100%`;
};
