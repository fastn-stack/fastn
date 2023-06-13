window.fastn_utils = {
    htmlNode(kind) {
        let node = "div";
        let css = ["ft_common"];
        if (kind === fastn_dom.ElementKind.Column) {
            css.push("ft_column");
        } else if (kind === fastn_dom.ElementKind.Row) {
            css.push("ft_row");
        } else if (kind === fastn_dom.ElementKind.IFrame) {
            node = "iframe";
        } else if (kind === fastn_dom.ElementKind.Image) {
            node = "img";
        }
        return [node, css];
    },
    getValue(obj) {
        if (!!obj.get) {
           return obj.get();
        } else {
           return obj;
        }
    },
    deepEqual(obj1, obj2) {
        if (obj1 === obj2) {
            return true;
        }
        // Check if the objects are of the same type
        if (typeof obj1 !== 'object' || obj1 === null || typeof obj2 !== 'object' || obj2 === null) {
            return false;
        }

        // Check if the objects have the same keys
        const keys1 = Object.keys(obj1);
        const keys2 = Object.keys(obj2);
        if (keys1.length !== keys2.length) {
            return false;
        }

        // Check if the values of the keys are deeply equal
        for (let key of keys1) {
            if (!this.deepEqual(obj1[key], obj2[key])) {
                return false;
            }
        }

        // Objects are deeply equal
        return true;
    }
}
