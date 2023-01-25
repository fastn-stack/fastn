function enable_dark_mode() {
    window.enable_system_mode();
}
function enable_light_mode() {
    window.enable_system_mode();
}
function enable_system_mode() {
    window.enable_system_mode();
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
// source: https://stackoverflow.com/questions/400212/ (cc-by-sa)
function copy_to_clipboard(text: string) {
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

function http(url: string, method: string, ...request_data: any) {
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
