function assertKindIdIsUnique() {
    let maps = [fastn_dom.PropertyKind, fastn_dom.ElementKind];
    for (let idx in maps) {
        let ids = [];
        let values = Object.values(maps[idx]);
        for (let vidx in values) {
            if (values[vidx] in ids) {
                throw `${values[vidx]} already found`;
            }
            ids.push(values[vidx]);
        }
    }
}

assertKindIdIsUnique();
