let fastn_utils = {
    htmlNode(kind) {
        let node = "div";
        let css = [];
        let attributes = {};
        if (kind === fastn_dom.ElementKind.Column) {
            css.push("ft_column");
        } else if (kind === fastn_dom.ElementKind.Document) {
            css.push("ft_column");
            css.push("full");
        } else if (kind === fastn_dom.ElementKind.Row) {
            css.push("ft_row");
        } else if (kind === fastn_dom.ElementKind.IFrame) {
            node = "iframe";
        } else if (kind === fastn_dom.ElementKind.Image) {
            node = "img";
        } else if (kind === fastn_dom.ElementKind.ContainerElement ||
            kind === fastn_dom.ElementKind.Text) {
            node = "div";
        } else if (kind === fastn_dom.ElementKind.Rive) {
            node = "canvas";
        } else if (kind === fastn_dom.ElementKind.CheckBox) {
            node = "input";
            attributes["type"] = "checkbox";
        } else if (kind === fastn_dom.ElementKind.TextInput) {
            node = "input";
        } else if (kind === fastn_dom.ElementKind.Comment) {
            node = fastn_dom.commentNode;
        } else if (kind === fastn_dom.ElementKind.Wrapper) {
            node = fastn_dom.wrapperNode;
        } else if (kind === fastn_dom.ElementKind.Code) {
           node = "pre";
        } else if (kind === fastn_dom.ElementKind.CodeChild) {
            node = "code";
        } else if (kind[0] === fastn_dom.ElementKind.WebComponent()[0]) {
            let {webcomponent, arguments} = kind[1];
            node = `${webcomponent}`;
            fastn_dom.webComponent.push(arguments);
            attributes[fastn_dom.webComponentArgument] = fastn_dom.webComponent.length - 1;
        }
        return [node, css, attributes];
    },
    getStaticValue(obj) {
        if (obj instanceof fastn.mutableClass) {
           return this.getStaticValue(obj.get());
        } else if (obj instanceof fastn.mutableListClass) {
            return obj.getList();
        }/*
        Todo: Make this work
        else if (obj instanceof fastn.recordInstanceClass) {
            return obj.getAllFields();
        }*/ else {
           return obj;
        }
    },
    removeNonFastnClasses(node) {
        let classList = node.getNode().classList;
        let extraCodeData = node.getExtraData().code;
        let iterativeClassList = classList;
        if (ssr) {
            iterativeClassList = iterativeClassList.getClasses();
        }
        const classesToRemove = [];

        for (const className of iterativeClassList) {
            if (!className.startsWith('__') &&
                className !== extraCodeData?.language &&
                className !== extraCodeData?.theme
            ) {
                classesToRemove.push(className);
            }
        }

        for (const classNameToRemove of classesToRemove) {
            classList.remove(classNameToRemove);
        }
    },
    staticToMutables(obj) {
        if (!(obj instanceof fastn.mutableClass) &&
            !(obj instanceof fastn.mutableListClass) &&
            !(obj instanceof fastn.recordInstanceClass))
        {
            if (Array.isArray(obj)) {
                let list = [];
                for (let index in obj) {
                    list.push(fastn_utils.staticToMutables(obj[index]));
                }
                return fastn.mutableList(list);
            } else if (obj instanceof Object) {
                let fields = {};
                for (let objKey in obj) {
                    fields[objKey] = fastn_utils.staticToMutables(obj[objKey]);
                }
                return fastn.recordInstance(fields);
            } else {
                return fastn.mutable(obj);
            }
        } else {
            return obj;
        }
    },
    getFlattenStaticValue(obj) {
        let staticValue = fastn_utils.getStaticValue(obj);
        if (Array.isArray(staticValue)) {
            return staticValue.map(func =>
                fastn_utils.getFlattenStaticValue(func.item));
        } /*
        Todo: Make this work
        else if (typeof staticValue === 'object' && fastn_utils.isNull(staticValue)) {
            return Object.fromEntries(
                Object.entries(staticValue).map(([k,v]) =>
                    [k, fastn_utils.getFlattenStaticValue(v)]
                )
            );
        }*/
        return staticValue;
    },
    getter(value) {
        if (value instanceof fastn.mutableClass) {
            return value.get();
        } else {
            return value;
        }
    },
    // Todo: Merge getterByKey with getter
    getterByKey(value, index) {
        if (value instanceof fastn.mutableClass
            || value instanceof fastn.recordInstanceClass) {
            return value.get(index);
        } else if (value instanceof fastn.mutableListClass) {
            return value.get(index).item;
        } else {
            return value;
        }
    },
    setter(variable, value) {
        if (!fastn_utils.isNull(variable) && variable.set) {
           variable.set(value);
           return true;
        }
        return false;
    },
    defaultPropertyValue(_propertyValue) {
        return null;
    },
    sameResponsiveRole(desktop, mobile) {
       return (desktop.get("font_family") ===  mobile.get("font_family")) &&
       (desktop.get("letter_spacing") ===  mobile.get("letter_spacing")) &&
       (desktop.get("line_height") ===  mobile.get("line_height")) &&
       (desktop.get("size") ===  mobile.get("size")) &&
       (desktop.get("weight") ===  mobile.get("weight"));
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
        if (value instanceof fastn.mutableClass ||
            value instanceof fastn.mutableListClass )
        {
            return value.getClone();
        }
           if (value instanceof fastn.recordInstanceClass) {
            return value.getClone();
        }
        return value;
    },
    getListItem(value) {
        if (value === undefined){
            return null;
        }
        if (value instanceof Object && value.hasOwnProperty("item")) {
            value = value.item;
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
            let property = fastn_utils.private.addUnderscoreToStart(properties[i]);
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
    isNull(a) {
        return a === null || a === undefined;
    },
    isCommentNode(node) {
      return node === fastn_dom.commentNode;
    },
    isWrapperNode(node) {
        return node === fastn_dom.wrapperNode;
    },
    nextSibling(node, parent) {
        // For Conditional DOM
        if (Array.isArray(node)) {
            node = node[node.length - 1];
        }
        if (node.nextSibling) {
          return node.nextSibling;
        }
        if (node.getNode && node.getNode().nextSibling !== undefined) {
            return node.getNode().nextSibling;
        }
        return parent.getChildren().indexOf(node.getNode()) + 1;
    },
    createNodeHelper(node, classes, attributes) {
        let tagName = node;
        let element = fastn_virtual.document.createElement(node);
        for (let key in attributes) {
            element.setAttribute(key, attributes[key])
        }
        for (let c in classes) {
            element.classList.add(classes[c]);
        }

        return [tagName, element];
    },
    addCssFile(url) {
        // Create a new link element
        const linkElement = document.createElement("link");

        // Set the attributes of the link element
        linkElement.rel = "stylesheet";
        linkElement.href = url;

        // Append the link element to the head section of the document
        document.head.appendChild(linkElement);
    },
    addCodeTheme(theme) {
        if (!fastn_dom.codeData.addedCssFile.includes(theme)) {
            let themeCssUrl = fastn_dom.codeData.availableThemes[theme];
            fastn_utils.addCssFile(themeCssUrl);
            fastn_dom.codeData.addedCssFile.push(theme);
        }
    },
    /**
     * Searches for highlighter occurrences in the text, removes them,
     * and returns the modified text along with highlighted line numbers.
     *
     * @param {string} text - The input text to process.
     * @returns {{ modifiedText: string, highlightedLines: number[] }}
     *   Object containing modified text and an array of highlighted line numbers.
     *
     * @example
     * const text = `/-- ftd.text: Hello ;; hello
     *
     * -- some-component: caption-value
     * attr-name: attr-value ;; <hl>
     *
     *
     * -- other-component: caption-value ;; <hl>
     * attr-name: attr-value`;
     *
     * const result = findAndRemoveHighlighter(text);
     * console.log(result.modifiedText);
     * console.log(result.highlightedLines);
     */
    findAndRemoveHighlighter(text) {
        const lines = text.split('\n');
        const highlighter = ';; <hl>';
        const result = {
            modifiedText: '',
            highlightedLines: ''
        };

        let highlightedLines = [];
        for (let i = 0; i < lines.length; i++) {
            const line = lines[i];
            const highlighterIndex = line.indexOf(highlighter);

            if (highlighterIndex !== -1) {
                highlightedLines.push(i + 1); // Adding 1 to convert to human-readable line numbers
                result.modifiedText += line.substring(0, highlighterIndex) + line.substring(highlighterIndex + highlighter.length) + '\n';
            } else {
                result.modifiedText += line + '\n';
            }
        }

        result.highlightedLines = fastn_utils.private.mergeNumbers(highlightedLines);

        return result;
    },
    getNodeValue(node) {
        return node.getNode().value;
    },
    setFullHeight() {
        if(!ssr) {
            document.body.style.height = `max(${document.documentElement.scrollHeight}px, 100%)`;
        }
    },
    resetFullHeight() {
        if(!ssr) {
            document.body.style.height = `100%`;
        }
    },
    highlightCode(codeElement, extraCodeData) {
        if (!ssr && !fastn_utils.isNull(extraCodeData.language) && !fastn_utils.isNull(extraCodeData.theme)) {
            Prism.highlightElement(codeElement);
        }
    },
    toSnakeCase(value) {
        return value.trim().split('').map((v, i) => {
            const lowercased = v.toLowerCase();
            if(v == " ") {
              return "_";
            }
            if(v != lowercased && i > 0) {
                return `_${lowercased}`
            }
            return lowercased;
        }).join('');
    }
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
    },
    /**
     * Merges consecutive numbers in a comma-separated list into ranges.
     *
     * @param {string} input - Comma-separated list of numbers.
     * @returns {string} Merged number ranges.
     *
     * @example
     * const input = '1,2,3,5,6,7,8,9,11';
     * const output = mergeNumbers(input);
     * console.log(output); // Output: '1-3,5-9,11'
     */
    mergeNumbers(numbers) {
        if (numbers.length === 0) {
            return "";
        }
        const mergedRanges = [];

        let start = numbers[0];
        let end = numbers[0];

        for (let i = 1; i < numbers.length; i++) {
            if (numbers[i] === end + 1) {
                end = numbers[i];
            } else {
                if (start === end) {
                    mergedRanges.push(start.toString());
                } else {
                    mergedRanges.push(`${start}-${end}`);
                }
                start = end = numbers[i];
            }
        }

        if (start === end) {
            mergedRanges.push(start.toString());
        } else {
            mergedRanges.push(`${start}-${end}`);
        }

        return mergedRanges.join(',');
    },
    addUnderscoreToStart(text) {
        if (/^\d/.test(text)) {
            return '_' + text;
        }
        return text;
    }
}


/*Object.prototype.get = function(index) {
    return this[index];
}*/
