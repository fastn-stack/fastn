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
        // Check for strict equality
        if (obj1 === obj2) {
            return true;
        }
        // Check for non-object types and null values
        if (typeof obj1 !== 'object' || obj1 === null || typeof obj2 !== 'object' || obj2 === null) {
            return false;
        }
        // Check for equal number of keys
        const keys1 = Object.keys(obj1);
        const keys2 = Object.keys(obj2);
        if (keys1.length !== keys2.length) {
            return false;
        }
        // Recursively compare values of each key
        for (let key of keys1) {
            if (!this.deepEqual(obj1[key], obj2[key])) {
                return false;
            }
        }
        // Objects are deeply equal
        return true;
    },

    getAllMutables(obj) {
        const nodes = [];

        function traverse(obj) {
            if (obj instanceof fastn.mutableClass) {
                nodes.push(obj);
            }

            if (typeof obj === 'object' && obj !== null) {
                for (let key in obj) {
                    traverse(obj[key]);
                }
            }
        }

        traverse(obj);

        return nodes;
    },

    newMutables(oldValue, newValue) {
        const oldMutables = new Set(fastn_utils.getAllMutables(oldValue));
        const newMutables = fastn_utils.getAllMutables(newValue).filter(mutable => !oldMutables.has(mutable));
        return newMutables;
    }
}
