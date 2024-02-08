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