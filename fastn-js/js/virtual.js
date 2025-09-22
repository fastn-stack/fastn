let fastnVirtual = {};

let id_counter = 0;
let ssr = false;
let doubleBuffering = false;

class ClassList {
    #classes = [];
    add(item) {
        this.#classes.push(item);
    }

    remove(itemToRemove) {
        this.#classes.filter((item) => item !== itemToRemove);
    }
    toString() {
        return this.#classes.join(" ");
    }
    getClasses() {
        return this.#classes;
    }
}

class Node {
    id;
    #dataId;
    #tagName;
    #children;
    #attributes;
    constructor(id, tagName) {
        if (typeof fastn_perf !== "undefined") fastn_perf.count("node_constructor");
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
        const openingTag = `<${
            this.#tagName
        }${this.getDataIdString()}${this.getIdString()}${this.getAttributesString()}${this.getClassString()}${this.getStyleString()}>`;
        const closingTag = `</${this.#tagName}>`;
        const innerHTML = this.innerHTML;
        const childNodes = this.#children
            .map((child) => child.toHtmlAsString())
            .join("");

        return `${openingTag}${innerHTML}${childNodes}${closingTag}`;
    }
    // Caution: This is only supported in ssr mode
    getDataIdString() {
        return ` data-id="${this.#dataId}"`;
    }
    // Caution: This is only supported in ssr mode
    getIdString() {
        return fastn_utils.isNull(this.id) ? "" : ` id="${this.id}"`;
    }
    // Caution: This is only supported in ssr mode
    getClassString() {
        const classList = this.classList.toString();
        return classList ? ` class="${classList}"` : "";
    }
    // Caution: This is only supported in ssr mode
    getStyleString() {
        const styleProperties = Object.entries(this.style)
            .map(([prop, value]) => `${prop}:${value}`)
            .join(";");
        return styleProperties ? ` style="${styleProperties}"` : "";
    }
    // Caution: This is only supported in ssr mode
    getAttributesString() {
        const nodeAttributes = Object.entries(this.#attributes)
            .map(([attribute, value]) => {
                if (value !== undefined && value !== null && value !== "") {
                    return `${attribute}=\"${value}\"`;
                }
                return `${attribute}`;
            })
            .join(" ");
        return nodeAttributes ? ` ${nodeAttributes}` : "";
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
        if (fastn_utils.isCommentNode(tagName)) {
            return window.document.createComment(fastn_dom.commentMessage);
        }
        return window.document.createElement(tagName);
    }
}

fastnVirtual.document = new Document2();

function addClosureToBreakpointWidth() {
    let closure = fastn.closureWithoutExecute(function () {
        let current = ftd.get_device();
        let lastDevice = ftd.device.get();
        if (current === lastDevice) {
            return;
        }
        console.log("last_device", lastDevice, "current_device", current);
        ftd.device.set(current);
    });

    ftd.breakpoint_width.addClosure(closure);
}

fastnVirtual.doubleBuffer = function (main) {
    addClosureToBreakpointWidth();
    let parent = document.createElement("div");
    let current_device = ftd.get_device();
    ftd.device = fastn.mutable(current_device);
    doubleBuffering = true;
    fastnVirtual.root = parent;
    main(parent);
    fastn_utils.replaceBodyStyleAndChildren(parent);
    doubleBuffering = false;
    fastnVirtual.root = document.body;
};

fastnVirtual.ssr = function (main) {
    ssr = true;
    let body = fastnVirtual.document.createElement("body");
    main(body);
    ssr = false;
    id_counter = 0;

    let meta_tags = "";
    if (globalThis.__fastn_meta) {
        for (const [key, value] of Object.entries(globalThis.__fastn_meta)) {
            let meta;
            if (value.kind === "property") {
                meta = `<meta property="${key}" content="${value.value}">`;
            } else if (value.kind === "name") {
                meta = `<meta name="${key}" content="${value.value}">`;
            } else if (value.kind === "title") {
                meta = `<title>${value.value}</title>`;
            }
            if (meta) {
                meta_tags += meta;
            }
        }
    }

    return [body.toHtmlAsString() + fastn_dom.getClassesAsString(), meta_tags];
};
