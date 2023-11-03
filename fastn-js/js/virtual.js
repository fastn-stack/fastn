let fastn_virtual = {}

let id_counter = 0;
let hydrating = false;
let ssr = false;
let rerender = false;

class ClassList {
    #classes = [];
    add(item) {
        this.#classes.push(item);
    }

    remove(itemToRemove) {
        this.#classes.filter(item => item !== itemToRemove)
    }
    toString() {
        return this.#classes.join(' ');
    }
    getClasses() {
        return this.#classes;
    }
}

class Node {
    id
    #dataId
    #tagName
    #children
    #attributes
    constructor(id, tagName) {
        this.#tagName = tagName;
        this.#dataId = id;
        this.classList = new ClassList();
        this.#children = [];
        this.#attributes = {};
        this.innerHTML = "";
        this.style = {};
        this.onclick = null;
        this.id = null;
    }
    appendChild(c) {
        this.#children.push(c);
    }

    insertBefore(node, index) {
        this.#children.splice(index, 0, node);
    }

    getChildren() {
        return this.#children;
    }

    setAttribute(attribute, value) {
        this.#attributes[attribute] = value;
    }

    getAttribute(attribute) {
        return this.#attributes[attribute];
    }

    removeAttribute(attribute) {
        if (attribute in this.#attributes) delete this.#attributes[attribute];
    }

    // Caution: This is only supported in ssr mode
    updateTagName(tagName) {
        this.#tagName = tagName;
    }
    // Caution: This is only supported in ssr mode
    toHtmlAsString() {
        const openingTag = `<${this.#tagName}${this.getDataIdString()}${this.getIdString()}${this.getAttributesString()}${this.getClassString()}${this.getStyleString()}>`;
        const closingTag = `</${this.#tagName}>`;
        const innerHTML = this.innerHTML;
        const childNodes = this.#children.map(child => child.toHtmlAsString()).join('');

        return `${openingTag}${innerHTML}${childNodes}${closingTag}`;
    }
    // Caution: This is only supported in ssr mode
    getDataIdString() {
        return ` data-id="${this.#dataId}"`;
    }
    // Caution: This is only supported in ssr mode
    getIdString() {
        return fastn_utils.isNull(this.id) ? "": ` id="${this.id}"`;
    }
    // Caution: This is only supported in ssr mode
    getClassString() {
        const classList = this.classList.toString();
        return classList ? ` class="${classList}"` : '';
    }
    // Caution: This is only supported in ssr mode
    getStyleString() {
        const styleProperties = Object.entries(this.style)
            .map(([prop, value]) => `${prop}:${value}`)
            .join(';');
        return styleProperties ? ` style="${styleProperties}"` : '';
    }
    // Caution: This is only supported in ssr mode
    getAttributesString() {
        const nodeAttributes = Object.entries(this.#attributes)
            .map(([attribute, value]) => {
                if (value !== undefined && value !== null && value !== "") {
                    return `${attribute}=\"${value}\"`;
                }
                return `${attribute}`;
            }).join(' ');
        return nodeAttributes ? ` ${nodeAttributes}` : '';
    }
}

class Document2 {
    createElement(tagName) {
        id_counter++;

        if (ssr) {
            return new Node(id_counter, tagName);
        }

        if (tagName === "body") {
            return window.document.body;
        }

        if (fastn_utils.isWrapperNode(tagName)) {
            return window.document.createComment(fastn_dom.commentMessage);
        }
        if (hydrating) {
            let node = this.getElementByDataID(id_counter);
            if (fastn_utils.isCommentNode(tagName)) {
                let comment= window.document.createComment(fastn_dom.commentMessage);
                node.parentNode.replaceChild(comment, node);
                return comment;
            }
            return node;
        } else {
            if (fastn_utils.isCommentNode(tagName)) {
                return window.document.createComment(fastn_dom.commentMessage);
            }
            return window.document.createElement(tagName);
        }
    }

    getElementByDataID(id) {
        return window.document.querySelector(`[data-id=\"${id}\"]`);
    }
}

fastn_virtual.document = new Document2();

function addClosureToBreakpointWidth() {
    let closure = new Closure(function() {
        let current = ftd.get_device();
        let lastDevice = ftd.device.get();
        if (current === lastDevice) {
            return;
        }
        console.log("Closure - last_device", lastDevice, "current_device", current);
        ftd.device.set(current);
    });

    ftd.breakpoint_width.addClosure(closure);
}

fastn_virtual.hydrate = function(main) {
    addClosureToBreakpointWidth();
    let current_device = ftd.get_device();
    let found_device = ftd.device.get();
    if (current_device !== found_device) {
        rerender = true
        ftd.device = fastn.mutable(current_device);
        let styles = document.getElementById("styles");
        styles.innerText = "";
        var children = document.body.children;
        // Loop through the direct children and remove those with tagName 'div'
        for (var i = children.length - 1; i >= 0; i--) {
            var child = children[i];
            if (child.tagName === 'DIV') {
                document.body.removeChild(child);
            }
        }

        main(document.body);
        rerender = false;
        styles.innerHTML = fastn_dom.styleClasses;
        return;
    }
    hydrating = true;
    let body = fastn_virtual.document.createElement("body");
    main(body);
    id_counter = 0;
    hydrating = false;
}

fastn_virtual.ssr = function(main) {
    ssr = true;
    let body = fastn_virtual.document.createElement("body");
    main(body)
    ssr = false;
    id_counter = 0;
    return body.toHtmlAsString() + fastn_dom.getClassesAsString();
}
