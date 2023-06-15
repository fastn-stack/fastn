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
    constructor(id, tagName) {
        this.#tagName = tagName;
        this.#id = id;
        this.classList = new ClassList();
        this.#children = [];
        this.innerHTML = "";
        this.style = {};
        this.onclick = null;
    }
    appendChild(c) {
        this.#children.push(c);
    }
    toHtmlAsString() {
        const openingTag = `<${this.#tagName}${this.getDataIdString()}${this.getClassString()}${this.getStyleString()}>`;
        const closingTag = `</${this.#tagName}>`;
        const innerHTML = this.innerHTML;
        const childNodes = this.#children.map(child => child.toHtmlAsString()).join('');

        return `${openingTag}${innerHTML}${childNodes}${closingTag}`;
    }
    getDataIdString() {
        return ` data-id="${this.#id}"`;
    }
    getClassString() {
        const classList = this.classList.toString();
        return classList ? ` class="${classList}"` : '';
    }
    getStyleString() {
        const styleProperties = Object.entries(this.style)
            .map(([prop, value]) => `${prop}:${value}`)
            .join(';');
        return styleProperties ? ` style="${styleProperties}"` : '';
    }
}

class Document2 {
    createElement(tagName) {
        id_counter++;
        if (ssr) {
            return new Node(id_counter, tagName);
        }
        if (tagName === "body") {
            return fastn_virtual.real_document.body;
        }
        if (!hydrating) {
            return fastn_virtual.real_document.createElement(tagName);
        }
        return fastn_virtual.document.getElementByDataID(id_counter);
    }

    getElementByDataID(id) {
        return fastn_virtual.real_document.querySelector(`[data-id=\"${id}\"]`);
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
    return body.toHtmlAsString()
}
