const DEVICE_SUFFIX = "____device";

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
    } catch (err) {
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

window.ftd.utils.get_event_key = function (event: any) {
    if (65 <= event.keyCode && event.keyCode <= 90) {
        return String.fromCharCode(event.keyCode).toLowerCase();
    }
    else {
        return event.key;
    }
}

window.ftd.utils.function_name_to_js_function = function (s: string) {
    let new_string = s;
    let startsWithDigit = /^\d/.test(s);
    if (startsWithDigit) {
        new_string = "_" + s;
    }
    new_string = new_string.replace('#', "__") .replace('-', "_")
        .replace(':', "___")
        .replace(',', "$")
        .replace("\\\\", "/")
        .replace('\\', "/")
        .replace('/', "_").replace('.', "_");

    return new_string;
};

window.ftd.utils.node_change_call = function(id: string, key: string, data: any){
    const node_function = `node_change_${id}`;
    const target = window[node_function];

    if(!!target && !!target[key]) {
        target[key](data);
    }
}

window.ftd.utils.set_value_helper = function(data: any, key: string, remaining: string, new_value: any) {
    if (!!remaining) {
        set_data_value(data, `${key}.${remaining}`, new_value);
    } else {
        set_data_value(data, key, new_value);
    }
}

window.ftd.dependencies = {}
window.ftd.dependencies.eval_background_size = function(bg: any) {
    if (typeof bg === 'object' && !!bg && "size" in bg) {
        let sz = bg.size;
        if (typeof sz === 'object' && !!sz && "x" in sz && "y" in sz) {
            return `${sz.x} ${sz.y}`;
        }
        else {
            return sz;
        }
    } else {
        return null;
    }
}

window.ftd.dependencies.eval_background_position = function(bg: any) {
    if (typeof bg === 'object' && !!bg  && "position" in bg) {
        let pos = bg.position;
        if (typeof pos === 'object' && !!pos && "x" in pos && "y" in pos) {
            return `${pos.x} ${pos.y}`;
        }
        else {
            return pos.replace("-", " ");
        }
    } else {
        return null;
    }
}

window.ftd.dependencies.eval_background_repeat = function(bg: any) {
    if (typeof bg === 'object' && !!bg  && "repeat" in bg) {
        return bg.repeat;
    } else {
        return null;
    }
}

window.ftd.dependencies.eval_background_color = function(bg: any, data: any) {
    let img_src = bg;
    if(!data["ftd#dark-mode"] && typeof img_src === 'object' && !!img_src && "light" in img_src) {
        return img_src.light;
    }
    else if(data["ftd#dark-mode"] && typeof img_src === 'object' && !!img_src && "dark" in img_src){
        return img_src.dark;
    }
    else if(typeof img_src === 'string' && !!img_src) {
        return img_src;
    }
    else {
        return null;
    }
}

window.ftd.dependencies.eval_background_image = function(bg: any, data: any) {
    if (typeof bg === 'object' && !!bg && "src" in bg) {
        let img_src = bg.src;
        if(!data["ftd#dark-mode"] && typeof img_src === 'object' && !!img_src && "light" in img_src) {
            return `url("${img_src.light}")`;
        }
        else if(data["ftd#dark-mode"] && typeof img_src === 'object' && !!img_src && "dark" in img_src){
            return `url("${img_src.dark}")`;
        }
        else {
            return null;
        }
    } else if (typeof bg === 'object' && !!bg && "colors" in bg && Object.keys(bg.colors).length) {
       let colors = "";
       // if the bg direction is provided by the user, use it, otherwise default
       let direction = bg.direction ?? "to bottom";

       let colors_vec = bg.colors;

       for (const c of colors_vec) {
            if (typeof c === 'object' && !!c && "color" in c) {
                
                let color_value = c.color;

                if(typeof color_value === 'object' && !!color_value && "light" in color_value && "dark" in color_value) {

                    if (colors) {
                        colors = data["ftd#dark-mode"] ? `${colors}, ${color_value.dark}`: `${colors}, ${color_value.light}`
                    }
                    else {
                        colors = data["ftd#dark-mode"] ? `${color_value.dark}`: `${color_value.light}`
                    }

                    if ("start" in c) colors = `${colors} ${c.start}`;
                    if ("end" in c) colors = `${colors} ${c.end}`;
                    if ("stop-position" in c) colors = `${colors}, ${c["stop-position"]}`;

                }
            }
       }

       let res = `linear-gradient(${direction}, ${colors})`;
       return res;
   }
    else {
        return null;
    }
}

window.ftd.dependencies.eval_box_shadow = function(shadow: any, data: any) {
    if (typeof shadow === 'object' && !!shadow) {
        let inset, blur, spread, x_off, y_off, color;

        inset = "";

        blur = spread = x_off = y_off = "0px";
        
        color = "black";

        if(("inset" in shadow) && shadow.inset) inset = "inset";

        if ("blur" in shadow) blur = shadow.blur;
        if ("spread" in shadow) spread = shadow.spread;
        if ("x-offset" in shadow) x_off = shadow["x-offset"];
        if ("y-offset" in shadow) y_off = shadow["y-offset"];

        if ("color" in shadow) {
            if (data["ftd#dark-mode"]){
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
}

window.ftd.utils.add_extra_in_id = function (node_id: string) {
    let element = document.querySelector(`[data-id=\"${node_id}\"]`);
    if (element) {
        changeElementId(element, DEVICE_SUFFIX, true);
    }
}

window.ftd.utils.remove_extra_from_id = function (node_id: string) {
    let element = document.querySelector(`[data-id=\"${node_id}\"]`);
    if (element) {
        changeElementId(element, DEVICE_SUFFIX, false);
    }
}


function changeElementId(element: Element, suffix: string, add: boolean) {
    // check if the current ID is not empty
    if (element.id) {
        // set the new ID for the element
        element.id =  updatedID(element.id, add, suffix);
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


function updatedID(str: string, flag: boolean, suffix: string) {
    // check if the flag is set
    if (flag) {
        // append suffix to the string
        return `${str} ${suffix}`;
    } else {
        // remove suffix from the string (if it exists)
        return str.replace(suffix, "");
    }
}
