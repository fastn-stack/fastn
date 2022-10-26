function console_log(...message: any) {
    if (true) { // false
        console.log(...message);
    }
}

function isObject(obj: object) {
    return obj === Object(obj);
}

function resolve_reference(reference: string, data: any) {
    return data[reference];
}

function change_value(function_arguments: FunctionArgument[], data: {
    [key: string]: any;
}) {
    for (const a in function_arguments) {
        if (!!function_arguments[a]["reference"]) {
            let reference: string = <string>function_arguments[a]["reference"];
            data[reference] = function_arguments[a]["value"];
        }
    }
}

function isFunctionArgument(object: any): object is FunctionArgument {
    return 'member' in object;
}