function styled_body(body) {
    return `<span style="color:#c0c5ce;">${body}</span>`;
}
function styled_section(line) {
    var section_splits = line.split(":");
    var section_type_title = section_splits[0].replace("-- ", "")
    var result = `<span style="color:#65737e;">-- </span><span style="color:#ebcb8b;">${section_type_title}</span><span style="color:#65737e;">: </span>`;
    if(section_splits[1] != null){
        result = result + `<span style="color:#a3be8c;">${section_splits[1].trim()} </span>`
    }
    return result;
}
function styled_header(line) {
    var header_splits = line.split(":");
    var result = `<span style="color:#b48ead;">${header_splits[0]}</span><span style="color:#65737e;">: </span>`;
    if(header_splits[1] != null){
        result = result + `<span style="color:#d08770;">${header_splits[1].trim()} </span>`
    }
    return result;
}
function apply_style(s) {
    var result = new String();
    const lines = s.split(/\r\n|\r|\n/);
    for (var line of lines) {
        line = line.trim();
        if (line.length == 0) {
            // Empty line
            result = result.concat(styled_body(" "));
            result = result.concat("\n");
        }
        else if (line.startsWith("--")) {
            // Section top
            result = result.concat(styled_section(line));
            result = result.concat("\n");
        }
        else if (!line.startsWith("--") && line.includes(":")) {
            // Header
            result = result.concat(styled_header(line));
            result = result.concat("\n");
        }
        else {
            // Body
            result = result.concat(styled_body(line));
            result = result.concat("\n");
        }
    }
    return result;
}

function get_raw_data(data, value_name, imports_used) {
    if (data != null && "value" in data && "type" in data) {
        let value_type = data["type"];
        let value = data["value"];

        if (value_type == "string" || value_type == "integer" || value_type == "decimal" || value_type == "boolean") {
            return `${value_name}: ${value}`;
        }
        else if (value_type == "reference") {
            let value_parts = value.split("#", 2);
            let doc = value_parts[0];
            let element = value_parts[1];

            let result;
            if (doc.includes("assets")) {
                let doc_parts = doc.split("/");
                let assets_alias = `${doc_parts[doc_parts.length - 2]}-assets`
                imports_used.add(`${doc} as ${assets_alias}`);
                result = `${value_name}: $${assets_alias}.${element}`;
            }
            else {
                result = `${value_name}: $${doc}.${element}`;
            }
            return result;
        }
        else {
            return `${value_name}.${value_type}: ${value}`;
        }
    }
    return null;
}

function get_type_data(types, category, imports_used) {
    let category_data = types[category];
    let result = "";

    if ("font-family" in category_data) {
        let ff_data = get_raw_data(category_data["font-family"], "font-family", imports_used);
        if (ff_data != null) {
            result += ff_data;
            result += "\n";
        }
    }

    if ("line-height" in category_data) {
        let ff_data = get_raw_data(category_data["line-height"], "line-height", imports_used);
        if (ff_data != null) {
            result += ff_data;
            result += "\n";
        }
    }

    if ("letter-spacing" in category_data) {
        let ff_data = get_raw_data(category_data["letter-spacing"], "letter-spacing", imports_used);
        if (ff_data != null) {
            result += ff_data;
            result += "\n";
        }
    }

    if ("weight" in category_data) {
        let ff_data = get_raw_data(category_data["weight"], "weight", imports_used);
        if (ff_data != null) {
            result += ff_data;
            result += "\n";
        }
    }

    if ("size" in category_data) {
        let ff_data = get_raw_data(category_data["size"], "size", imports_used);
        if (ff_data != null) {
            result += ff_data;
            result += "\n";
        }
    }

    return result;
}

function get_asset_imports_string(imports_used) {
    let all_imports = "";
    for (i of imports_used) {
        all_imports += `-- import: ${i}\n`
    }
    return all_imports;
}

function typo_to_ftd(json) {
    const typo_data = JSON.parse(json);
    let typo_desktop = Object.keys(typo_data)
        .filter((key) => key.includes("-desktop"))
        .reduce((obj, key) => {
        obj = typo_data[key];
        return obj;
    }, {});
    let typo_mobile = Object.keys(typo_data)
        .filter((key) => key.includes("-mobile"))
        .reduce((obj, key) => {
        obj = typo_data[key];
        return obj;
    }, {});

    let imports_used = new Set();

    let s =  `
    ;; HEADING HERO ----------------
    -- ftd.type heading-hero-mobile:
    ${get_type_data(typo_mobile, "heading-hero", imports_used)}

    -- ftd.type heading-hero-desktop:
    ${get_type_data(typo_desktop, "heading-hero", imports_used)}

    -- ftd.responsive-type heading-hero:
    desktop: $heading-hero-desktop
    mobile: $heading-hero-mobile


    ;; HEADING LARGE ----------------
    -- ftd.type heading-large-mobile:
    ${get_type_data(typo_mobile, "heading-large", imports_used)}

    -- ftd.type heading-large-desktop:
    ${get_type_data(typo_desktop, "heading-large", imports_used)}

    -- ftd.responsive-type heading-large:
    desktop: $heading-large-desktop
    mobile: $heading-large-mobile


    ;; HEADING MEDIUM ----------------
    -- ftd.type heading-medium-mobile:
    ${get_type_data(typo_mobile, "heading-medium", imports_used)}

    -- ftd.type heading-medium-desktop:
    ${get_type_data(typo_desktop, "heading-medium", imports_used)}

    -- ftd.responsive-type heading-medium:
    desktop: $heading-medium-desktop
    mobile: $heading-medium-mobile


    ;; HEADING SMALL ---------------
    -- ftd.type heading-small-mobile:
    ${get_type_data(typo_mobile, "heading-small", imports_used)}

    -- ftd.type heading-small-desktop:
    ${get_type_data(typo_desktop, "heading-small", imports_used)}

    -- ftd.responsive-type heading-small:
    desktop: $heading-small-desktop
    mobile: $heading-small-mobile


    ;; HEADING TINY ----------------
    -- ftd.type heading-tiny-mobile:
    ${get_type_data(typo_mobile, "heading-tiny", imports_used)}

    -- ftd.type heading-tiny-desktop:
    ${get_type_data(typo_desktop, "heading-tiny", imports_used)}

    -- ftd.responsive-type heading-tiny:
    desktop: $heading-tiny-desktop
    mobile: $heading-tiny-mobile


    ;; COPY LARGE ---------------
    -- ftd.type copy-large-mobile:
    ${get_type_data(typo_mobile, "copy-large", imports_used)}

    -- ftd.type copy-large-desktop:
    ${get_type_data(typo_desktop, "copy-large", imports_used)}

    -- ftd.responsive-type copy-large:
    desktop: $copy-large-desktop
    mobile: $copy-large-mobile


    ;; COPY REGULAR ---------------
    -- ftd.type copy-regular-mobile:
    ${get_type_data(typo_mobile, "copy-regular", imports_used)}

    -- ftd.type copy-regular-desktop:
    ${get_type_data(typo_desktop, "copy-regular", imports_used)}

    -- ftd.responsive-type copy-regular:
    desktop: $copy-regular-desktop
    mobile: $copy-regular-mobile


    ;; COPY SMALL ----------------
    -- ftd.type copy-small-mobile:
    ${get_type_data(typo_mobile, "copy-small", imports_used)}

    -- ftd.type copy-small-desktop:
    ${get_type_data(typo_desktop, "copy-small", imports_used)}

    -- ftd.responsive-type copy-small:
    desktop: $copy-small-desktop
    mobile: $copy-small-mobile


    ;; FINE PRINT ----------------
    -- ftd.type fine-print-mobile:
    ${get_type_data(typo_mobile, "fine-print", imports_used)}

    -- ftd.type fine-print-desktop:
    ${get_type_data(typo_desktop, "fine-print", imports_used)}

    -- ftd.responsive-type fine-print:
    desktop: $fine-print-desktop
    mobile: $fine-print-mobile


    ;; BLOCK QUOTE --------------
    -- ftd.type blockquote-mobile:
    ${get_type_data(typo_mobile, "blockquote", imports_used)}

    -- ftd.type blockquote-desktop:
    ${get_type_data(typo_desktop, "blockquote", imports_used)}

    -- ftd.responsive-type blockquote:
    desktop: $blockquote-desktop
    mobile: $blockquote-mobile


    ;; SOURCE CODE ---------------
    -- ftd.type source-code-mobile:
    ${get_type_data(typo_mobile, "source-code", imports_used)}

    -- ftd.type source-code-desktop:
    ${get_type_data(typo_desktop, "source-code", imports_used)}

    -- ftd.responsive-type source-code:
    desktop: $source-code-desktop
    mobile: $source-code-mobile


    ;; LABEL LARGE ----------------
    -- ftd.type label-large-mobile:
    ${get_type_data(typo_mobile, "label-large", imports_used)}

    -- ftd.type label-large-desktop:
    ${get_type_data(typo_desktop, "label-large", imports_used)}

    -- ftd.responsive-type label-large:
    desktop: $label-large-desktop
    mobile: $label-large-mobile


    ;; LABEL SMALL ----------------
    -- ftd.type label-small-mobile:
    ${get_type_data(typo_mobile, "label-small", imports_used)}

    -- ftd.type label-small-desktop:
    ${get_type_data(typo_desktop, "label-small", imports_used)}

    -- ftd.responsive-type label-small:
    desktop: $label-small-desktop
    mobile: $label-small-mobile


    ;; BUTTON LARGE ----------------
    -- ftd.type button-large-mobile:
    ${get_type_data(typo_mobile, "button-large", imports_used)}

    -- ftd.type button-large-desktop:
    ${get_type_data(typo_desktop, "button-large", imports_used)}

    -- ftd.responsive-type button-large:
    desktop: $button-large-desktop
    mobile: $button-large-mobile


    ;; BUTTON MEDIUM ----------------
    -- ftd.type button-medium-mobile:
    ${get_type_data(typo_mobile, "button-medium", imports_used)}

    -- ftd.type button-medium-desktop:
    ${get_type_data(typo_desktop, "button-medium", imports_used)}

    -- ftd.responsive-type button-medium:
    desktop: $button-medium-desktop
    mobile: $button-medium-mobile


    ;; BUTTON SMALL ----------------
    -- ftd.type button-small-mobile:
    ${get_type_data(typo_mobile, "button-small", imports_used)}

    -- ftd.type button-small-desktop:
    ${get_type_data(typo_desktop, "button-small", imports_used)}

    -- ftd.responsive-type button-small:
    desktop: $button-small-desktop
    mobile: $button-small-mobile


    ;; LINK ----------------
    -- ftd.type link-mobile:
    ${get_type_data(typo_mobile, "link", imports_used)}

    -- ftd.type link-desktop:
    ${get_type_data(typo_desktop, "link", imports_used)}

    -- ftd.responsive-type link:
    desktop: $link-desktop
    mobile: $link-mobile

    ;; TYPE-DATA --------------
    -- ftd.type-data types:
    heading-hero: $heading-hero
    heading-large: $heading-large
    heading-medium: $heading-medium
    heading-small: $heading-small
    heading-tiny: $heading-tiny
    copy-large: $copy-large
    copy-regular: $copy-regular
    copy-small: $copy-small
    fine-print: $fine-print
    blockquote: $blockquote
    source-code: $source-code
    label-large: $label-large
    label-small: $label-small
    button-large: $button-large
    button-medium: $button-medium
    button-small: $button-small
    link: $link
    `

    let imports_string = get_asset_imports_string(imports_used);
    let final = `${imports_string}${s}`.split("\n").map(s => s.trim()).join("\n");;

    let fs = `<pre>${apply_style(final)}</pre>`;
    return [final, fs];
}
