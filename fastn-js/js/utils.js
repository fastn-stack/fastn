let fastn_utils = {
    htmlNode(kind) {
        let node = "div";
        let css = [];
        let attributes = {};
        if (kind === fastn_dom.ElementKind.Column) {
            css.push(fastn_dom.InternalClass.FT_COLUMN);
        } else if (kind === fastn_dom.ElementKind.Document) {
            css.push(fastn_dom.InternalClass.FT_COLUMN);
            css.push(fastn_dom.InternalClass.FT_FULL_SIZE);
        } else if (kind === fastn_dom.ElementKind.Row) {
            css.push(fastn_dom.InternalClass.FT_ROW);
        } else if (kind === fastn_dom.ElementKind.IFrame) {
            node = "iframe";
            // To allow fullscreen support
            // Reference: https://stackoverflow.com/questions/27723423/youtube-iframe-embed-full-screen
            attributes["allowfullscreen"] = "";
        } else if (kind === fastn_dom.ElementKind.Image) {
            node = "img";
        } else if (kind === fastn_dom.ElementKind.Audio) {
            node = "audio";
        } else if (kind === fastn_dom.ElementKind.Video) {
            node = "video";
        } else if (
            kind === fastn_dom.ElementKind.ContainerElement ||
            kind === fastn_dom.ElementKind.Text
        ) {
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
            let [webcomponent, args] = kind[1];
            node = `${webcomponent}`;
            fastn_dom.webComponent.push(args);
            attributes[fastn_dom.webComponentArgument] =
                fastn_dom.webComponent.length - 1;
        }
        return [node, css, attributes];
    },
    createStyle(cssClass, obj) {
        // Use the benchmarkable CSS system if available
        if (typeof fastn_css !== "undefined") {
            const cssString = fastn_css.createStyle(cssClass, obj);

            if (doubleBuffering) {
                fastn_dom.styleClasses = `${fastn_dom.styleClasses}${cssString}\n`;
            } else {
                let styles = document.getElementById("styles");
                let textNode = document.createTextNode(cssString);
                if (styles.styleSheet) {
                    styles.styleSheet.cssText = cssString;
                } else {
                    styles.appendChild(textNode);
                }
            }
            return cssString;
        }

        // Fallback to original implementation
        if (doubleBuffering) {
            fastn_dom.styleClasses = `${
                fastn_dom.styleClasses
            }${getClassAsString(cssClass, obj)}\n`;
        } else {
            let styles = document.getElementById("styles");
            let newClasses = getClassAsString(cssClass, obj);
            let textNode = document.createTextNode(newClasses);
            if (styles.styleSheet) {
                styles.styleSheet.cssText = newClasses;
            } else {
                styles.appendChild(textNode);
            }
        }
    },
    getStaticValue(obj) {
        if (obj instanceof fastn.mutableClass) {
            return this.getStaticValue(obj.get());
        } else if (obj instanceof fastn.mutableListClass) {
            return obj.getList();
        } /*
        Todo: Make this work
        else if (obj instanceof fastn.recordInstanceClass) {
            return obj.getAllFields();
        }*/ else {
            return obj;
        }
    },
    getInheritedValues(default_args, inherited, function_args) {
        let record_fields = {
            colors: ftd.default_colors.getClone().setAndReturn("is_root", true),
            types: ftd.default_types.getClone().setAndReturn("is_root", true),
        };
        Object.assign(record_fields, default_args);
        let fields = {};
        if (inherited instanceof fastn.recordInstanceClass) {
            fields = inherited.getClonedFields();
            if (fastn_utils.getStaticValue(fields["colors"].get("is_root"))) {
                delete fields.colors;
            }
            if (fastn_utils.getStaticValue(fields["types"].get("is_root"))) {
                delete fields.types;
            }
        }
        Object.assign(record_fields, fields);
        Object.assign(record_fields, function_args);
        return fastn.recordInstance({
            ...record_fields,
        });
    },
    removeNonFastnClasses(node) {
        let classList = node.getNode().classList;
        let extraCodeData = node.getExtraData().code;
        let iterativeClassList = classList;
        if (ssr) {
            iterativeClassList = iterativeClassList.getClasses();
        }
        const internalClassNames = Object.values(fastn_dom.InternalClass);
        const classesToRemove = [];

        for (const className of iterativeClassList) {
            if (
                !className.startsWith("__") &&
                !internalClassNames.includes(className) &&
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
        if (
            !(obj instanceof fastn.mutableClass) &&
            !(obj instanceof fastn.mutableListClass) &&
            !(obj instanceof fastn.recordInstanceClass)
        ) {
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
                    if (fields[objKey] instanceof fastn.mutableClass) {
                        fields[objKey] = fields[objKey].get();
                    }
                }
                return fastn.recordInstance(fields);
            } else {
                return fastn.mutable(obj);
            }
        } else {
            return obj;
        }
    },
    mutableToStaticValue(obj) {
        if (obj instanceof fastn.mutableClass) {
            return this.mutableToStaticValue(obj.get());
        } else if (obj instanceof fastn.mutableListClass) {
            let list = obj.getList();
            return list.map((func) => this.mutableToStaticValue(func.item));
        } else if (obj instanceof fastn.recordInstanceClass) {
            let fields = obj.getAllFields();
            return Object.fromEntries(
                Object.entries(fields).map(([k, v]) => [
                    k,
                    this.mutableToStaticValue(v),
                ]),
            );
        } else {
            return obj;
        }
    },
    flattenMutable(value) {
        if (!(value instanceof fastn.mutableClass)) return value;

        if (value.get() instanceof fastn.mutableClass)
            return this.flattenMutable(value.get());

        return value;
    },
    getFlattenStaticValue(obj) {
        let staticValue = fastn_utils.getStaticValue(obj);
        if (Array.isArray(staticValue)) {
            return staticValue.map((func) =>
                fastn_utils.getFlattenStaticValue(func.item),
            );
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
        if (
            value instanceof fastn.mutableClass ||
            value instanceof fastn.recordInstanceClass
        ) {
            return value.get(index);
        } else if (value instanceof fastn.mutableListClass) {
            return value.get(index).item;
        } else {
            return value;
        }
    },
    setter(variable, value) {
        variable = fastn_utils.flattenMutable(variable);
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
        return (
            desktop.get("font_family") === mobile.get("font_family") &&
            desktop.get("letter_spacing") === mobile.get("letter_spacing") &&
            desktop.get("line_height") === mobile.get("line_height") &&
            desktop.get("size") === mobile.get("size") &&
            desktop.get("weight") === mobile.get("weight")
        );
    },
    getRoleValues(value) {
        let font_families = fastn_utils.getStaticValue(
            value.get("font_family"),
        );
        if (Array.isArray(font_families))
            font_families = font_families
                .map((obj) => fastn_utils.getStaticValue(obj.item))
                .join(", ");
        return {
            "font-family": font_families,
            "letter-spacing": fastn_utils.getStaticValue(
                value.get("letter_spacing"),
            ),
            "font-size": fastn_utils.getStaticValue(value.get("size")),
            "font-weight": fastn_utils.getStaticValue(value.get("weight")),
            "line-height": fastn_utils.getStaticValue(value.get("line_height")),
        };
    },
    clone(value) {
        if (value === null || value === undefined) {
            return value;
        }
        if (
            value instanceof fastn.mutableClass ||
            value instanceof fastn.mutableListClass
        ) {
            return value.getClone();
        }
        if (value instanceof fastn.recordInstanceClass) {
            return value.getClone();
        }
        return value;
    },
    getListItem(value) {
        if (value === undefined) {
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
        } else {
            return event.key;
        }
    },
    createNestedObject(currentObject, path, value) {
        const properties = path.split(".");

        for (let i = 0; i < properties.length - 1; i++) {
            let property = fastn_utils.private.addUnderscoreToStart(
                properties[i],
            );
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
            currentObject.set(innermostProperty, value);
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
        if (fastn_utils.isNull(i)) return;
        i = i.toString();
        const { space_before, space_after } = fastn_utils.private.spaces(i);
        const o = (() => {
            let g = fastn_utils.private.replace_last_occurrence(
                marked.parse(i),
                "<p>",
                "",
            );
            g = fastn_utils.private.replace_last_occurrence(g, "</p>", "");
            return g;
        })();
        return `${fastn_utils.private.repeated_space(
            space_before,
        )}${o}${fastn_utils.private.repeated_space(space_after)}`.replace(
            /\n+$/,
            "",
        );
    },

    process_post_markdown(node, body) {
        if (!ssr) {
            const divElement = document.createElement("div");
            divElement.innerHTML = body;

            const current_node = node;
            const colorClasses = Array.from(current_node.classList).filter(
                (className) => className.startsWith("__c"),
            );
            const roleClasses = Array.from(current_node.classList).filter(
                (className) => className.startsWith("__rl"),
            );
            const tableElements = Array.from(
                divElement.getElementsByTagName("table"),
            );
            const codeElements = Array.from(
                divElement.getElementsByTagName("code"),
            );

            tableElements.forEach((table) => {
                colorClasses.forEach((colorClass) => {
                    table.classList.add(colorClass);
                });
            });

            codeElements.forEach((code) => {
                roleClasses.forEach((roleClass) => {
                    var roleCls = "." + roleClass;
                    let role = fastn_dom.classes[roleCls];
                    let roleValue = role["value"];
                    let fontFamily = roleValue["font-family"];
                    code.style.fontFamily = fontFamily;
                });
            });

            body = divElement.innerHTML;
        }
        return body;
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
        while (Array.isArray(node)) {
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
        let element = fastnVirtual.document.createElement(node);
        for (let key in attributes) {
            element.setAttribute(key, attributes[key]);
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
        const lines = text.split("\n");
        const highlighter = ";; <hl>";
        const result = {
            modifiedText: "",
            highlightedLines: "",
        };

        let highlightedLines = [];
        for (let i = 0; i < lines.length; i++) {
            const line = lines[i];
            const highlighterIndex = line.indexOf(highlighter);

            if (highlighterIndex !== -1) {
                highlightedLines.push(i + 1); // Adding 1 to convert to human-readable line numbers
                result.modifiedText +=
                    line.substring(0, highlighterIndex) +
                    line.substring(highlighterIndex + highlighter.length) +
                    "\n";
            } else {
                result.modifiedText += line + "\n";
            }
        }

        result.highlightedLines =
            fastn_utils.private.mergeNumbers(highlightedLines);

        return result;
    },
    getNodeValue(node) {
        return node.getNode().value;
    },
    getNodeCheckedState(node) {
        return node.getNode().checked;
    },
    setFullHeight() {
        if (!ssr) {
            document.body.style.height = `max(${document.documentElement.scrollHeight}px, 100%)`;
        }
    },
    resetFullHeight() {
        if (!ssr) {
            document.body.style.height = `100%`;
        }
    },
    highlightCode(codeElement, extraCodeData) {
        if (
            !ssr &&
            !fastn_utils.isNull(extraCodeData.language) &&
            !fastn_utils.isNull(extraCodeData.theme)
        ) {
            Prism.highlightElement(codeElement);
        }
    },

    //Taken from: https://byby.dev/js-slugify-string
    slugify(str) {
        return String(str)
            .normalize("NFKD") // split accented characters into their base characters and diacritical marks
            .replace(".", "-")
            .replace(/[\u0300-\u036f]/g, "") // remove all the accents, which happen to be all in the \u03xx UNICODE block.
            .trim() // trim leading or trailing whitespace
            .toLowerCase() // convert to lowercase
            .replace(/[^a-z0-9 -]/g, "") // remove non-alphanumeric characters
            .replace(/\s+/g, "-") // replace spaces with hyphens
            .replace(/-+/g, "-"); // remove consecutive hyphens
    },

    getEventListeners(node) {
        return {
            onclick: node.onclick,
            onmouseleave: node.onmouseleave,
            onmouseenter: node.onmouseenter,
            oninput: node.oninput,
            onblur: node.onblur,
            onfocus: node.onfocus,
        };
    },

    flattenArray(arr) {
        return fastn_utils.private.flattenArray([arr]);
    },
    toSnakeCase(value) {
        return value
            .trim()
            .split("")
            .map((v, i) => {
                const lowercased = v.toLowerCase();
                if (v == " ") {
                    return "_";
                }
                if (v != lowercased && i > 0) {
                    return `_${lowercased}`;
                }
                return lowercased;
            })
            .join("");
    },
    escapeHtmlInCode(str) {
        return str.replace(/[<]/g, "&lt;");
    },

    escapeHtmlInMarkdown(str) {
        if (typeof str !== "string") {
            return str;
        }

        let result = "";
        let ch_map = {
            "<": "&lt;",
            ">": "&gt;",
            "&": "&amp;",
            '"': "&quot;",
            "'": "&#39;",
            "/": "&#47;",
        };
        let foundBackTick = false;
        for (var i = 0; i < str.length; i++) {
            let current = str[i];
            if (current === "`") {
                foundBackTick = !foundBackTick;
            }
            // Ignore escaping html inside backtick (as marked function
            // escape html for backtick content):
            // For instance: In `hello <title>`, `<` and `>` should not be
            // escaped. (`foundBackTick`)
            // Also the `/` which is followed by `<` should be escaped.
            // For instance: `</` should be escaped but `http://` should not
            // be escaped. (`(current === '/' && !(i > 0 && str[i-1] === "<"))`)
            if (
                foundBackTick ||
                (current === "/" && !(i > 0 && str[i - 1] === "<"))
            ) {
                result += current;
                continue;
            }
            result += ch_map[current] ?? current;
        }
        return result;
    },

    // Used to initialize __args__ inside component and UDF js functions
    getArgs(default_args, passed_args) {
        // Note: arguments as variable name not allowed in strict mode
        let args = default_args;
        for (var arg in passed_args) {
            if (!default_args.hasOwnProperty(arg)) {
                args[arg] = passed_args[arg];
                continue;
            }
            if (
                default_args.hasOwnProperty(arg) &&
                fastn_utils.getStaticValue(passed_args[arg]) !== undefined
            ) {
                args[arg] = passed_args[arg];
            }
        }
        return args;
    },

    /**
     * Replaces the children of `document.body` with the children from
     * newChildrenWrapper and updates the styles based on the
     * `fastn_dom.styleClasses`.
     *
     * @param {HTMLElement} newChildrenWrapper - The wrapper element
     * containing the new children.
     */
    replaceBodyStyleAndChildren(newChildrenWrapper) {
        // Update styles based on `fastn_dom.styleClasses`
        let styles = document.getElementById("styles");
        styles.innerHTML = fastn_dom.getClassesAsStringWithoutStyleTag();

        // Replace the children of document.body with the children from
        // newChildrenWrapper
        fastn_utils.private.replaceChildren(document.body, newChildrenWrapper);
    },
};

fastn_utils.private = {
    flattenArray(arr) {
        return arr.reduce((acc, item) => {
            return acc.concat(
                Array.isArray(item)
                    ? fastn_utils.private.flattenArray(item)
                    : item,
            );
        }, []);
    },
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
            if (s[i] !== " ") {
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
            if (s[i] !== " ") {
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
        return Array.from({ length: n }, () => " ").join("");
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

        return mergedRanges.join(",");
    },
    addUnderscoreToStart(text) {
        if (/^\d/.test(text)) {
            return "_" + text;
        }
        return text;
    },

    /**
     * Replaces the children of a parent element with the children from a
     * new children wrapper.
     *
     * @param {HTMLElement} parent - The parent element whose children will
     * be replaced.
     * @param {HTMLElement} newChildrenWrapper - The wrapper element
     * containing the new children.
     * @returns {void}
     */
    replaceChildren(parent, newChildrenWrapper) {
        // Remove existing children of the parent
        var children = parent.children;
        // Loop through the direct children and remove those with tagName 'div'
        for (var i = children.length - 1; i >= 0; i--) {
            var child = children[i];
            if (child.tagName === "DIV") {
                parent.removeChild(child);
            }
        }

        // Cut and append the children from newChildrenWrapper to the parent
        while (newChildrenWrapper.firstChild) {
            parent.appendChild(newChildrenWrapper.firstChild);
        }
    },

    // Cookie related functions ----------------------------------------------
    setCookie(cookieName, cookieValue) {
        cookieName = fastn_utils.getStaticValue(cookieName);
        cookieValue = fastn_utils.getStaticValue(cookieValue);

        // Default expiration period of 30 days
        var expires = "";
        var expirationDays = 30;
        if (expirationDays) {
            var date = new Date();
            date.setTime(date.getTime() + expirationDays * 24 * 60 * 60 * 1000);
            expires = "; expires=" + date.toUTCString();
        }

        document.cookie =
            cookieName +
            "=" +
            encodeURIComponent(cookieValue) +
            expires +
            "; path=/";
    },
    getCookie(cookieName) {
        cookieName = fastn_utils.getStaticValue(cookieName);
        var name = cookieName + "=";
        var decodedCookie = decodeURIComponent(document.cookie);
        var cookieArray = decodedCookie.split(";");

        for (var i = 0; i < cookieArray.length; i++) {
            var cookie = cookieArray[i].trim();
            if (cookie.indexOf(name) === 0) {
                return cookie.substring(name.length, cookie.length);
            }
        }

        return "None";
    },
};

/*Object.prototype.get = function(index) {
    return this[index];
}*/
