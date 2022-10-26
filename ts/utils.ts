function console_log(...message: any) {
    if (true) { // false
        console.log(...message);
    }
}

function isObject(obj: object) {
    return obj === Object(obj);
}