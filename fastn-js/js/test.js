function assertKindIdIsUnique() {
    let maps = [fastn_dom.PropertyKind, fastn_dom.ElementKind, fastn_dom.Event, fastn_dom.propertyMap];
    for (let idx in maps) {
        let ids = new Set();
        let values = Object.values(maps[idx]);
        for (let vidx in values) {
            let innerValue = values[vidx];
            assertKindIdIsUniqueForValue(innerValue, ids);
        }
    }
}

function assertKindIdIsUniqueForValue(value, ids) {

    if (value instanceof Function) {
        value = value()[0];
    } else if (value instanceof Object) {
        for (key in value) {
            let innerValue = value[key];
            if (innerValue instanceof Object) {
                assertKindIdIsUniqueForObject(innerValue, ids);
            }

            if (ids.has(innerValue)) {
                throw `${innerValue} already found`;
            }
            ids.add(innerValue);

        }
        return;
    } else if (value instanceof Array) {
        value = value[0];
    }

    if (ids.has(value)) {
        throw `${value} already found`;
    }
    ids.add(value);
}

assertKindIdIsUnique();
