let fastn_utils = {
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
        } else if (kind === fastn_dom.ElementKind.Div) {
            node = "div";
            css = [];
        }
        return [node, css];
    },
    getStaticValue(obj) {
        if (obj instanceof fastn.mutableClass) {
           return this.getStaticValue(obj.get());
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
        // Check for class instances
        if (obj1.constructor !== obj2.constructor) {
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
        // Check for class instance variables
        if (obj1 instanceof fastn.mutableClass && obj2 instanceof fastn.mutableClass) {
            return obj1.equalMutable(obj2);
        }
        // Objects are deeply equal
        return true;
    },

    /**
     * Retrieves all mutables present in an object.
     * This function recursively traverses the object and collects all instances
     * of mutable classes, storing them in an array.
     *
     * @param {Object} obj - The object to traverse.
     * @returns {Array} - An array containing all found mutables.
     */
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

    /**
     * This function compares the mutables found in both old and new values and returns
     * an array of mutables that are present in the new value but not in the old value and
     * also an array of mutables that are present in the old value but not in the new value
     *
     * @param {any} oldValue - The old value to compare.
     * @param {any} newValue - The new value to compare.
     * @returns {{newMutables: *[], oldMutables: *[]}} - An object containing 'newMutables' and 'oldMutables' arrays.
     */
    getNewAndOldMutables(oldValue, newValue) {
        const oldMutables = this.getAllMutables(oldValue);
        const newMutables = this.getAllMutables(newValue);

        const newMutablesOnly = newMutables.filter(mutable => !oldMutables.includes(mutable));
        const oldMutablesOnly = oldMutables.filter(mutable => !newMutables.includes(mutable));

        return {
            newMutables: newMutablesOnly,
            oldMutables: oldMutablesOnly
        };
    }
}
