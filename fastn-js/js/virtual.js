let fastn_virtual = {}

let id_counter = 0;
let hydrating = false;
let ssr = false;

class ClassList {
    #classes = [];
    add(item) {
        this.#classes.push(item);
    }
    toString() {
        return this.#classes.join(' ');
    }
}

class Node {
    #id
    #tagName
    #children
    #attributes
    #parent
    constructor(id, tagName, parent) {
        this.#tagName = tagName;
        this.#id = id;
        this.classList = new ClassList();
        this.#children = [];
        this.#attributes = {};
        this.innerHTML = "";
        this.style = {};
        this.onclick = null;
        this.#parent = parent;
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

    removeAttribute(attribute) {
        if (attribute in this.#attributes) delete this.#attributes[attribute];
    }

    // Caution: This is only supported in ssr mode
    updateTagName(tagName) {
        this.#tagName = tagName;
    }
    // Caution: This is only supported in ssr mode
    toHtmlAsString() {
        const openingTag = `<${this.#tagName}${this.getDataIdString()}${this.getAttributesString()}${this.getClassString()}${this.getStyleString()}>`;
        const closingTag = `</${this.#tagName}>`;
        const innerHTML = this.innerHTML;
        const childNodes = this.#children.map(child => child.toHtmlAsString()).join('');

        return `${openingTag}${innerHTML}${childNodes}${closingTag}`;
    }
    // Caution: This is only supported in ssr mode
    getDataIdString() {
        return ` data-id="${this.#id}"`;
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
    createElement(tagName, parent) {
        id_counter++;

        parent = fastn_utils.getParent(parent);

        if (ssr) {
            return new Node(id_counter, tagName, parent);
        }

        if (tagName === "body") {
            return window.document.body;
        }

        if (fastn_utils.isCommentNode(tagName)) {
            return window.document.createComment(fastn_dom.commentMessage);
        } else if (hydrating) {
            return this.getElementByDataID(id_counter);
        }else {
            return window.document.createElement(tagName);
        }
    }

    getElementByDataID(id) {
        return window.document.querySelector(`[data-id=\"${id}\"]`);
    }
}

fastn_virtual.document = new Document2();



fastn_virtual.hydrate = function(main) {
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
