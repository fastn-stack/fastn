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


function get_color_value(cs: Object, category: string, color_name: string): any {
    let category_data: Object = cs[category as keyof typeof cs];
    let color_data: Object = category_data[color_name as keyof typeof category_data];
    let color_value = color_data['value' as keyof typeof color_data];
    return color_value;
}

function styled_body(body: string): string {
    return `
<span style="color:#c0c5ce;">${body}</span>`;
}

function styled_section(line: string): string {
    var section_type_title = line.replace("-- ", "").replace(":", "");

    var result = `
<span style="color:#65737e;">-- </span><span style="color:#ebcb8b;">${section_type_title}</span><span style="color:#65737e;">: </span>`

    return result;
}

function styled_header(line: string): string {
    var header_splits = line.split(":");

    var result = `
<span style="color:#b48ead;">${header_splits[0]}</span><span style="color:#65737e;">: </span><span style="color:#d08770;">${header_splits[1].trim()} </span>`

    return result;
}


function apply_style(s: string): String {
    var result = new String();
    const lines = s.split(/\r\n|\r|\n/);
    for (var line of lines) {
        if (line.trim().length == 0){
            // Empty line
            result = result.concat(styled_body(" "));
        }
        else if (line.startsWith("--")){
            // Section top
            result = result.concat(styled_section(line));
        }
        else if (!line.startsWith("--") && line.includes(":")){
            // Header
            result = result.concat(styled_header(line));
        }
        else {
            // Body
            result = result.concat(styled_body(line));
        }
    }
    return result;
}

function figma_json_to_ftd(json: string): string[] {
    const cs_data = JSON.parse(json);

    let cs_light: Object = Object.keys(cs_data)
            .filter((key) => key.includes("-light"))
            .reduce((obj, key) => {
                obj = cs_data[key];
                return obj;
          }, {});

    let cs_dark: Object = Object.keys(cs_data)
            .filter((key) => key.includes("-dark"))
            .reduce((obj, key) => {
                obj = cs_data[key];
                return obj;
          }, {});

    let s: string = `
    -- ftd.color base-:
    light: ${get_color_value(cs_light, "Background Colors", "base")}
    dark: ${get_color_value(cs_dark, "Background Colors", "base")}

    -- ftd.color step-1-:
    light: ${get_color_value(cs_light, "Background Colors", "step-1")}
    dark: ${get_color_value(cs_dark, "Background Colors", "step-1")}

    -- ftd.color step-2-:
    light: ${get_color_value(cs_light, "Background Colors", "step-2")}
    dark: ${get_color_value(cs_dark, "Background Colors", "step-2")}

    -- ftd.color overlay-:
    light: ${get_color_value(cs_light, "Background Colors", "overlay")}
    dark: ${get_color_value(cs_dark, "Background Colors", "overlay")}

    -- ftd.color code-:
    light: ${get_color_value(cs_light, "Background Colors", "code")}
    dark: ${get_color_value(cs_dark, "Background Colors", "code")}

    -- ftd.background-colors background-:
    base: $base-
    step-1: $step-1-
    step-2: $step-2-
    overlay: $overlay-
    code: $code-

    -- ftd.color border-:
    light: ${get_color_value(cs_light, "Standalone Colors", "border")}
    dark: ${get_color_value(cs_dark, "Standalone Colors", "border")}

    -- ftd.color border-strong-:
    light: ${get_color_value(cs_light, "Standalone Colors", "border-strong")}
    dark: ${get_color_value(cs_dark, "Standalone Colors", "border-strong")}

    -- ftd.color text-:
    light: ${get_color_value(cs_light, "Standalone Colors", "text")}
    dark: ${get_color_value(cs_dark, "Standalone Colors", "text")}

    -- ftd.color text-strong-:
    light: ${get_color_value(cs_light, "Standalone Colors", "text-strong")}
    dark: ${get_color_value(cs_dark, "Standalone Colors", "text-strong")}

    -- ftd.color shadow-:
    light: ${get_color_value(cs_light, "Standalone Colors", "shadow")}
    dark: ${get_color_value(cs_dark, "Standalone Colors", "shadow")}

    -- ftd.color scrim-:
    light: ${get_color_value(cs_light, "Standalone Colors", "scrim")}
    dark: ${get_color_value(cs_dark, "Standalone Colors", "scrim")}

    -- ftd.color cta-primary-base-:
    light: ${get_color_value(cs_light, "CTA Primary Colors", "base")}
    dark: ${get_color_value(cs_dark, "CTA Primary Colors", "base")}

    -- ftd.color cta-primary-hover-:
    light: ${get_color_value(cs_light, "CTA Primary Colors", "hover")}
    dark: ${get_color_value(cs_dark, "CTA Primary Colors", "hover")}

    -- ftd.color cta-primary-pressed-:
    light: ${get_color_value(cs_light, "CTA Primary Colors", "pressed")}
    dark: ${get_color_value(cs_dark, "CTA Primary Colors", "pressed")}

    -- ftd.color cta-primary-disabled-:
    light: ${get_color_value(cs_light, "CTA Primary Colors", "disabled")}
    dark: ${get_color_value(cs_dark, "CTA Primary Colors", "disabled")}

    -- ftd.color cta-primary-focused-:
    light: ${get_color_value(cs_light, "CTA Primary Colors", "focused")}
    dark: ${get_color_value(cs_dark, "CTA Primary Colors", "focused")}

    -- ftd.color cta-primary-border-:
    light: ${get_color_value(cs_light, "CTA Primary Colors", "border")}
    dark: ${get_color_value(cs_dark, "CTA Primary Colors", "border")}

    -- ftd.color cta-primary-text-:
    light: ${get_color_value(cs_light, "CTA Primary Colors", "text")}
    dark: ${get_color_value(cs_dark, "CTA Primary Colors", "text")}


    -- ftd.cta-colors cta-primary-:
    base: $cta-primary-base-
    hover: $cta-primary-hover-
    pressed: $cta-primary-pressed-
    disabled: $cta-primary-disabled-
    focused: $cta-primary-focused-
    border: $cta-primary-border-
    text: $cta-primary-text-

    -- ftd.color cta-secondary-base-:
    light: ${get_color_value(cs_light, "CTA Secondary Colors", "base")}
    dark: ${get_color_value(cs_dark, "CTA Secondary Colors", "base")}

    -- ftd.color cta-secondary-hover-:
    light: ${get_color_value(cs_light, "CTA Secondary Colors", "hover")}
    dark: ${get_color_value(cs_dark, "CTA Secondary Colors", "hover")}

    -- ftd.color cta-secondary-pressed-:
    light: ${get_color_value(cs_light, "CTA Secondary Colors", "pressed")}
    dark: ${get_color_value(cs_dark, "CTA Secondary Colors", "pressed")}

    -- ftd.color cta-secondary-disabled-:
    light: ${get_color_value(cs_light, "CTA Secondary Colors", "disabled")}
    dark: ${get_color_value(cs_dark, "CTA Secondary Colors", "disabled")}

    -- ftd.color cta-secondary-focused-:
    light: ${get_color_value(cs_light, "CTA Secondary Colors", "focused")}
    dark: ${get_color_value(cs_dark, "CTA Secondary Colors", "focused")}

    -- ftd.color cta-secondary-border-:
    light: ${get_color_value(cs_light, "CTA Secondary Colors", "border")}
    dark: ${get_color_value(cs_dark, "CTA Secondary Colors", "border")}

    -- ftd.color cta-secondary-text-:
    light: ${get_color_value(cs_light, "CTA Secondary Colors", "text")}
    dark: ${get_color_value(cs_dark, "CTA Secondary Colors", "text")}

    -- ftd.cta-colors cta-secondary-:
    base: $cta-secondary-base-
    hover: $cta-secondary-hover-
    pressed: $cta-secondary-pressed-
    disabled: $cta-secondary-disabled-
    focused: $cta-secondary-focused-
    border: $cta-secondary-border-
    text: $cta-secondary-text-

    -- ftd.color cta-tertiary-base-:
    light: ${get_color_value(cs_light, "CTA Tertiary Colors", "base")}
    dark: ${get_color_value(cs_dark, "CTA Tertiary Colors", "base")}

    -- ftd.color cta-tertiary-hover-:
    light: ${get_color_value(cs_light, "CTA Tertiary Colors", "hover")}
    dark: ${get_color_value(cs_dark, "CTA Tertiary Colors", "hover")}

    -- ftd.color cta-tertiary-pressed-:
    light: ${get_color_value(cs_light, "CTA Tertiary Colors", "pressed")}
    dark: ${get_color_value(cs_dark, "CTA Tertiary Colors", "pressed")}

    -- ftd.color cta-tertiary-disabled-:
    light: ${get_color_value(cs_light, "CTA Tertiary Colors", "disabled")}
    dark: ${get_color_value(cs_dark, "CTA Tertiary Colors", "disabled")}

    -- ftd.color cta-tertiary-focused-:
    light: ${get_color_value(cs_light, "CTA Tertiary Colors", "focused")}
    dark: ${get_color_value(cs_dark, "CTA Tertiary Colors", "focused")}

    -- ftd.color cta-tertiary-border-:
    light: ${get_color_value(cs_light, "CTA Tertiary Colors", "border")}
    dark: ${get_color_value(cs_dark, "CTA Tertiary Colors", "border")}

    -- ftd.color cta-tertiary-text-:
    light: ${get_color_value(cs_light, "CTA Tertiary Colors", "text")}
    dark: ${get_color_value(cs_dark, "CTA Tertiary Colors", "text")}

    -- ftd.cta-colors cta-tertiary-:
    base: $cta-tertiary-base-
    hover: $cta-tertiary-hover-
    pressed: $cta-tertiary-pressed-
    disabled: $cta-tertiary-disabled-
    focused: $cta-tertiary-focused-
    border: $cta-tertiary-border-
    text: $cta-tertiary-text-

    -- ftd.color cta-danger-base-:
    light: ${get_color_value(cs_light, "CTA Danger Colors", "base")}
    dark: ${get_color_value(cs_dark, "CTA Danger Colors", "base")}

    -- ftd.color cta-danger-hover-:
    light: ${get_color_value(cs_light, "CTA Danger Colors", "hover")}
    dark: ${get_color_value(cs_dark, "CTA Danger Colors", "hover")}

    -- ftd.color cta-danger-pressed-:
    light: ${get_color_value(cs_light, "CTA Danger Colors", "pressed")}
    dark: ${get_color_value(cs_dark, "CTA Danger Colors", "pressed")}

    -- ftd.color cta-danger-disabled-:
    light: ${get_color_value(cs_light, "CTA Danger Colors", "disabled")}
    dark: ${get_color_value(cs_dark, "CTA Danger Colors", "disabled")}

    -- ftd.color cta-danger-focused-:
    light: ${get_color_value(cs_light, "CTA Danger Colors", "focused")}
    dark: ${get_color_value(cs_dark, "CTA Danger Colors", "focused")}

    -- ftd.color cta-danger-border-:
    light: ${get_color_value(cs_light, "CTA Danger Colors", "border")}
    dark: ${get_color_value(cs_dark, "CTA Danger Colors", "border")}

    -- ftd.color cta-danger-text-:
    light: ${get_color_value(cs_light, "CTA Danger Colors", "text")}
    dark: ${get_color_value(cs_dark, "CTA Danger Colors", "text")}

    -- ftd.cta-colors cta-danger-:
    base: $cta-danger-base-
    hover: $cta-danger-hover-
    pressed: $cta-danger-pressed-
    disabled: $cta-danger-disabled-
    focused: $cta-danger-focused-
    border: $cta-danger-border-
    text: $cta-danger-text-

    -- ftd.color accent-primary-:
    light: ${get_color_value(cs_light, "Accent Colors", "primary")}
    dark: ${get_color_value(cs_dark, "Accent Colors", "primary")}

    -- ftd.color accent-secondary-:
    light: ${get_color_value(cs_light, "Accent Colors", "secondary")}
    dark: ${get_color_value(cs_dark, "Accent Colors", "secondary")}

    -- ftd.color accent-tertiary-:
    light: ${get_color_value(cs_light, "Accent Colors", "tertiary")}
    dark: ${get_color_value(cs_dark, "Accent Colors", "tertiary")}

    -- ftd.pst accent-:
    primary: $accent-primary-
    secondary: $accent-secondary-
    tertiary: $accent-tertiary-

    -- ftd.color error-base-:
    light: ${get_color_value(cs_light, "Error Colors", "base")}
    dark: ${get_color_value(cs_dark, "Error Colors", "base")}

    -- ftd.color error-text-:
    light: ${get_color_value(cs_light, "Error Colors", "text")}
    dark: ${get_color_value(cs_dark, "Error Colors", "text")}

    -- ftd.color error-border-:
    light: ${get_color_value(cs_light, "Error Colors", "border")}
    dark: ${get_color_value(cs_dark, "Error Colors", "border")}

    -- ftd.btb error-btb-:
    base: $error-base-
    text: $error-text-
    border: $error-border-

    -- ftd.color success-base-:
    light: ${get_color_value(cs_light, "Success Colors", "base")}
    dark: ${get_color_value(cs_dark, "Success Colors", "base")}

    -- ftd.color success-text-:
    light: ${get_color_value(cs_light, "Success Colors", "text")}
    dark: ${get_color_value(cs_dark, "Success Colors", "text")}

    -- ftd.color success-border-:
    light: ${get_color_value(cs_light, "Success Colors", "border")}
    dark: ${get_color_value(cs_dark, "Success Colors", "border")}

    -- ftd.btb success-btb-:
    base: $success-base-
    text: $success-text-
    border: $success-border-

    -- ftd.color info-base-:
    light: ${get_color_value(cs_light, "Info Colors", "base")}
    dark: ${get_color_value(cs_dark, "Info Colors", "base")}

    -- ftd.color info-text-:
    light: ${get_color_value(cs_light, "Info Colors", "text")}
    dark: ${get_color_value(cs_dark, "Info Colors", "text")}

    -- ftd.color info-border-:
    light: ${get_color_value(cs_light, "Info Colors", "border")}
    dark: ${get_color_value(cs_dark, "Info Colors", "border")}

    -- ftd.btb info-btb-:
    base: $info-base-
    text: $info-text-
    border: $info-border-

    -- ftd.color warning-base-:
    light: ${get_color_value(cs_light, "Warning Colors", "base")}
    dark: ${get_color_value(cs_dark, "Warning Colors", "base")}

    -- ftd.color warning-text-:
    light: ${get_color_value(cs_light, "Warning Colors", "text")}
    dark: ${get_color_value(cs_dark, "Warning Colors", "text")}

    -- ftd.color warning-border-:
    light: ${get_color_value(cs_light, "Warning Colors", "border")}
    dark: ${get_color_value(cs_dark, "Warning Colors", "border")}

    -- ftd.btb warning-btb-:
    base: $warning-base-
    text: $warning-text-
    border: $warning-border-

    -- ftd.color custom-one-:
    light: ${get_color_value(cs_light, "Custom Colors", "one")}
    dark: ${get_color_value(cs_dark, "Custom Colors", "one")}

    -- ftd.color custom-two-:
    light: ${get_color_value(cs_light, "Custom Colors", "two")}
    dark: ${get_color_value(cs_dark, "Custom Colors", "two")}

    -- ftd.color custom-three-:
    light: ${get_color_value(cs_light, "Custom Colors", "three")}
    dark: ${get_color_value(cs_dark, "Custom Colors", "three")}

    -- ftd.color custom-four-:
    light: ${get_color_value(cs_light, "Custom Colors", "four")}
    dark: ${get_color_value(cs_dark, "Custom Colors", "four")}

    -- ftd.color custom-five-:
    light: ${get_color_value(cs_light, "Custom Colors", "five")}
    dark: ${get_color_value(cs_dark, "Custom Colors", "five")}

    -- ftd.color custom-six-:
    light: ${get_color_value(cs_light, "Custom Colors", "six")}
    dark: ${get_color_value(cs_dark, "Custom Colors", "six")}

    -- ftd.color custom-seven-:
    light: ${get_color_value(cs_light, "Custom Colors", "seven")}
    dark: ${get_color_value(cs_dark, "Custom Colors", "seven")}

    -- ftd.color custom-eight-:
    light: ${get_color_value(cs_light, "Custom Colors", "eight")}
    dark: ${get_color_value(cs_dark, "Custom Colors", "eight")}

    -- ftd.color custom-nine-:
    light: ${get_color_value(cs_light, "Custom Colors", "nine")}
    dark: ${get_color_value(cs_dark, "Custom Colors", "nine")}

    -- ftd.color custom-ten-:
    light: ${get_color_value(cs_light, "Custom Colors", "ten")}
    dark: ${get_color_value(cs_dark, "Custom Colors", "ten")}

    -- ftd.custom-colors custom-:
    one: $custom-one-
    two: $custom-two-
    three: $custom-three-
    four: $custom-four-
    five: $custom-five-
    six: $custom-six-
    seven: $custom-seven-
    eight: $custom-eight-
    nine: $custom-nine-
    ten: $custom-ten-

    -- ftd.color-scheme main:
    background: $background-
    border: $border-
    border-strong: $border-strong-
    text: $text-
    text-strong: $text-strong-
    shadow: $shadow-
    scrim: $scrim-
    cta-primary: $cta-primary-
    cta-secondary: $cta-secondary-
    cta-tertiary: $cta-tertiary-
    cta-danger: $cta-danger-
    accent: $accent-
    error: $error-btb-
    success: $success-btb-
    info: $info-btb-
    warning: $warning-btb-
    custom: $custom-
    `;

    let fs = `<pre>${apply_style(s)}</pre>`;
    return [s,fs];
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
