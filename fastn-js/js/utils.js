let fastn_utils = {
    htmlNode(kind) {
        let node = "div";
        let css = [];
        let attributes = {};
        if (kind === fastn_dom.ElementKind.Column ||
            kind === fastn_dom.ElementKind.Document) {
            css.push("ft_column");
        } else if (kind === fastn_dom.ElementKind.Row) {
            css.push("ft_row");
        } else if (kind === fastn_dom.ElementKind.IFrame) {
            node = "iframe";
        } else if (kind === fastn_dom.ElementKind.Image) {
            node = "img";
        } else if (kind === fastn_dom.ElementKind.Div ||
            kind === fastn_dom.ElementKind.ContainerElement ||
            kind === fastn_dom.ElementKind.Text) {
            node = "div";
        } else if (kind === fastn_dom.ElementKind.Rive) {
            node = "canvas";
        } else if (kind === fastn_dom.ElementKind.CheckBox) {
            node = "input";
            attributes["type"] = "checkbox";
        } else if (kind === fastn_dom.ElementKind.TextInput) {
            node = "input";
        }
        return [node, css, attributes];
    },

    getStaticValue(obj) {
        if (obj instanceof fastn.mutableClass) {
           return this.getStaticValue(obj.get());
        } if (obj instanceof fastn.mutableListClass) {
            return obj.getList();
        } else {
           return obj;
        }
    },

    getFlattenStaticValue(obj) {
        let staticValue = fastn_utils.getStaticValue(obj);
        if (Array.isArray(staticValue)) {
            return staticValue.map(func =>
                fastn_utils.getFlattenStaticValue(func.item));
        }
        return staticValue;
    },

    getter(value) {
        if (value.get) {
            return value.get();
        } else {
            return value;
        }
    },

    setter(variable, value) {
        if (variable.set) {
           variable.set(value);
           return true;
        }
        return false;
    },

    defaultPropertyValue(_propertyValue) {
        return null;
    },

    sameResponsiveRole(desktop, mobile) {
       return (desktop.get("font_family") ==  mobile.get("font_family")) &&
       (desktop.get("letter_spacing") ==  mobile.get("letter_spacing")) &&
       (desktop.get("line_height") ==  mobile.get("line_height")) &&
       (desktop.get("size") ==  mobile.get("size")) &&
       (desktop.get("weight") ==  mobile.get("weight"));
    },

    getRoleValues(value) {
        return {
            "font-family": fastn_utils.getStaticValue(value.get("font_family")),
            "letter-spacing": fastn_utils.getStaticValue(value.get("letter_spacing")),
            "font-size": fastn_utils.getStaticValue(value.get("size")),
            "font-weight": fastn_utils.getStaticValue(value.get("weight")),
            "line-height": fastn_utils.getStaticValue(value.get("line_height")),
        };
    },

    clone(value) {
        if (value === null || value === undefined) {
            return value;
        }
        if (value instanceof Mutable) {
            let cloned_value = value.getClone();
            return cloned_value;
        }
        return value;
    },
  
    getEventKey(event) {
        if (65 <= event.keyCode && event.keyCode <= 90) {
            return String.fromCharCode(event.keyCode).toLowerCase();
        }
        else {
            return event.key;
        }
    },

    createNestedObject(currentObject, path, value) {
        const properties = path.split('.');

        for (let i = 0; i < properties.length - 1; i++) {
            const property = properties[i];
            if (currentObject instanceof fastn.recordInstanceClass) {
                if (currentObject.get(property) === undefined) {
                    currentObject.set(property, fastn.recordInstance({}));
                }
                currentObject = currentObject.get(property).get();
            } else {
                if (!currentObject.hasOwnProperty(property)) {
                    currentObject[property] = fastn.recordInstance({});
                }
                currentObject = currentObject[property];
            }
        }

        const innermostProperty = properties[properties.length - 1];
        if (currentObject instanceof fastn.recordInstanceClass) {
            currentObject.set(innermostProperty, value)
        } else {
            currentObject[innermostProperty] = value;
        }
    },

    /**
     * Takes an input string and processes it as inline markdown using the
     * 'marked' library. The function removes the last occurrence of
     * wrapping <p> tags (i.e. <p> tag found at the end) from the result and
     * adjusts spaces around the content.
     *
     * @param {string} i - The input string to be processed as inline markdown.
     * @returns {string} - The processed string with inline markdown.
     */
    markdown_inline(i) {
        const { space_before, space_after } = fastn_utils.private.spaces(i);
        const o = (() => {
            let g = fastn_utils.private.replace_last_occurrence(marked.parse(i), "<p>", "");
            g = fastn_utils.private.replace_last_occurrence(g, "</p>", "");
            return g;
        })();
        return `${fastn_utils.private.repeated_space(space_before)}${o}${fastn_utils.private.repeated_space(space_after)}`;
    },
}


fastn_utils.private = {
    /**
     * Helper function for `fastn_utils.markdown_inline` to find the number of
     * spaces before and after the content.
     *
     * @param {string} s - The input string.
     * @returns {Object} - An object with 'space_before' and 'space_after' properties
     * representing the number of spaces before and after the content.
     */
    spaces(s) {
        let space_before = 0;
        for (let i = 0; i < s.length; i++) {
            if (s[i] !== ' ') {
                space_before = i;
                break;
            }
            space_before = i + 1;
        }
        if (space_before === s.length) {
            return { space_before, space_after: 0 };
        }

        let space_after = 0;
        for (let i = s.length - 1; i >= 0; i--) {
            if (s[i] !== ' ') {
                space_after = s.length - 1 - i;
                break;
            }
            space_after = i + 1;
        }

        return { space_before, space_after };
    },

    /**
     * Helper function for `fastn_utils.markdown_inline` to replace the last
     * occurrence of a substring in a string.
     *
     * @param {string} s - The input string.
     * @param {string} old_word - The substring to be replaced.
     * @param {string} new_word - The replacement substring.
     * @returns {string} - The string with the last occurrence of 'old_word' replaced by 'new_word'.
     */
    replace_last_occurrence(s, old_word, new_word) {
        if (!s.includes(old_word)) {
            return s;
        }

        const idx = s.lastIndexOf(old_word);
        return s.slice(0, idx) + new_word + s.slice(idx + old_word.length);
    },

    /**
     * Helper function for `fastn_utils.markdown_inline` to generate a string
     * containing a specified number of spaces.
     *
     * @param {number} n - The number of spaces to be generated.
     * @returns {string} - A string with 'n' spaces concatenated together.
     */
    repeated_space(n) {
        return Array.from({ length: n }, () => ' ').join('');
    }
}
