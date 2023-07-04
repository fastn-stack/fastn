let fastn = {};

class Closure {
    #cached_value;
    #node;
    #property;
    #formula;
    constructor(func) {
        this.#cached_value = func();
        this.#formula = func;
    }

    get() {
        return this.#cached_value;
    }
    getFormula() {
        return this.#formula;
    }
    addNodeProperty(node, property) {
        this.#node = node;
        this.#property = property;
        this.updateUi();

        return this;
    }
    update() {
        this.#cached_value = this.#formula();
        this.updateUi();
    }
    getNode() {
        return this.#node;
    }
    updateUi() {
        if (!this.#node || !this.#property || !this.#node.getNode()) {
            return;
        }

        this.#node.setStaticProperty(this.#property, this.#cached_value);
    }
}

class Mutable {
    #value;
    #old_closure
    #closures;
    #closureInstance;
    constructor(val) {
        this.#value = null;
        this.#old_closure = null;
        this.#closures = [];
        this.#closureInstance = fastn.closure(() => this.#closures.forEach((closure) => closure.update()));
        this.set(val);
    }
    get(key) {
        if (!!key && (this.#value instanceof RecordInstance || this.#value instanceof MutableList)) {
            return this.#value.get(key)
        }
        return this.#value;
    }
    setWithoutUpdate(value) {
        if (this.#old_closure) {
            this.#value.removeClosure(this.#old_closure);
        }

        if (this.#value instanceof RecordInstance) {
            this.#value.replace(value);
        } else {
            this.#value = value;
        }

        if (this.#value instanceof Mutable) {
            this.#old_closure = fastn.closure(() => this.#closureInstance.update());
            this.#value.addClosure(this.#old_closure);
        } else {
            this.#old_closure = null;
        }
    }
    set(value) {
        this.setWithoutUpdate(value);

        this.#closureInstance.update();
    }
    // we have to unlink all nodes, else they will be kept in memory after the node is removed from DOM
    unlinkNode(node) {
        this.#closures = this.#closures.filter(closure => closure.getNode() !== node);
    }
    addClosure(closure) {
        this.#closures.push(closure);
    }
    removeClosure(closure) {
        this.#closures = this.#closures.filter(c => c !== closure);
    }
    equalMutable(other) {
        if (!fastn_utils.deepEqual(this.get(), other.get())) {
            return false;
        }
        const thisClosures = this.#closures;
        const otherClosures = other.#closures;

        return thisClosures === otherClosures;
    }
}

class Proxy {
    #differentiator
    #cached_value
    #closures;
    #closureInstance;
    constructor(targets, differentiator) {
        this.#differentiator = differentiator;
        this.#cached_value = this.#differentiator().get();
        this.#closures = [];

        let proxy = this;
        for (let idx in targets) {
            targets[idx].addClosure(new Closure(function () {
                proxy.update();
                proxy.#closures.forEach(closure => closure.update());
            }));
            targets[idx].addClosure(this);
        }
    }
    addClosure(closure) {
        this.#closures.push(closure);
    }
    removeClosure(closure) {
        this.#closures = this.#closures.filter(c => c !== closure);
    }
    update() {
        this.#cached_value = this.#differentiator().get();
    }
    get(key) {
        if (!!key && (this.#cached_value instanceof RecordInstance || this.#cached_value instanceof MutableList)) {
            return this.#cached_value.get(key)
        }
        return this.#cached_value;
    }
    set(value) {
        // Todo: Optimization removed. Reuse optimization later again
        /*if (fastn_utils.deepEqual(this.#cached_value, value)) {
            return;
        }*/
        this.#differentiator().set(value);
    }
}

class MutableList {
    #list;
    #watchers;
    #dom_constructors;
    constructor(list) {
        this.#list = [];
        for (let idx in list) {
            this.#list.push( { item: fastn.wrapMutable(list[idx]), index: new Mutable(parseInt(idx)) });
        }
        this.#watchers = [];
    }
    forLoop(root, dom_constructor) {
        let l = fastn_dom.forLoop(root, dom_constructor, this);
        this.#watchers.push(l);
        return l;
    }
    getList() {
        return this.#list;
    }
    get(idx) {
        return this.#list[idx];
    }
    set(idx, value) {
        this.#list[idx].item.set(value);
    }
    insertAt(idx, value) {
        let mutable = fastn.wrapMutable(value);
        this.#list.splice(idx, 0, { item: mutable, index: new Mutable(idx) });
        // for every item after the inserted item, update the index
        for (let i = idx + 1; i < this.#list.length; i++) {
            this.#list[i].index.set(i);
        }

        for (let i in this.#watchers) {
            this.#watchers[i].createNode(idx);
        }
    }
    push(value) {
        this.insertAt(this.#list.length, value);
    }
    deleteAt(idx) {
        this.#list.splice(idx, 1);
        // for every item after the deleted item, update the index
        for (let i = idx; i < this.#list.length; i++) {
            this.#list[i].index.set(i);
        }
    }
}

fastn.mutable = function (val) {
    return new Mutable(val)
};

fastn.closure = function (func) {
    return new Closure(func);
}

fastn.formula = function (deps, func) {
    let closure = fastn.closure(func);
    let mutable = new Mutable(closure.get());
    for (let idx in deps) {
        deps[idx].addClosure(new Closure(function () {
            closure.update();
            mutable.set(closure.get());
        }));
    }

    return mutable;
}

fastn.proxy = function (targets, differentiator) {
    return new Proxy(targets, differentiator);
};

fastn.mutableClass = Mutable;

fastn.wrapMutable = function (obj) {
    if (!(obj instanceof Mutable)
        && !(obj instanceof RecordInstance)
        && !(obj instanceof MutableList)
    ) {
        obj = new Mutable(obj);
    }
    return obj;
}

fastn.mutableList = function (list) {
    return new MutableList(list);
}

class RecordInstance {
    #fields;
    constructor(obj) {
        this.#fields = {};

        for (let key in obj) {
            if (obj[key] instanceof fastn.mutableClass) {
                this.#fields[key] = fastn.mutable(null)
                this.#fields[key].setWithoutUpdate(obj[key]);
            } else {
                this.#fields[key] = fastn.mutable(obj[key]);
            }
        }
    }
    get(key) {
        return this.#fields[key];
    }
    set(key, value) {
        this.#fields[key].set(value);
    }
    replace(obj) {
        for (let key in this.#fields) {
            if (!(key in obj.#fields)) {
                throw new Error("RecordInstance.replace: key " + key + " not present in new object");
            }
            this.#fields[key] = fastn.wrapMutable(obj.#fields[key]);
        }
    }
}

fastn.recordInstance = function (obj) {
    return new RecordInstance(obj);
}
let fastn_dom = {};

fastn_dom.ElementKind = {
    Row: 0,
    Column: 1,
    Integer: 2,
    Decimal: 3,
    Boolean: 4,
    Text: 5,
    Image: 6,
    IFrame: 7,
    // To create parent for dynamic DOM
    Div: 8,
};

fastn_dom.PropertyKind = {
    Color_RGB: 0,
    IntegerValue: 1,
    StringValue: 2,
}

fastn_dom.Event = {
    Click: 0
}

class Node2 {
    #node;
    #parent;
    #mutables;
    constructor(parent, kind) {
        let [node, classes] = fastn_utils.htmlNode(kind);
        this.#node = fastn_virtual.document.createElement(node);
        for (let c in classes) {
            this.#node.classList.add(classes[c]);
        }
        this.#parent = parent;
        // this is where we store all the attached closures, so we can free them when we are done
        this.#mutables = [];
    }
    parent() {
        return this.#parent;
    }
    done() {
        let parent = this.#parent;
        /*if (!!parent.parent) {
            parent = parent.parent();
        }*/
        if (!!parent.getNode) {
            parent = parent.getNode();
        }
        parent.appendChild(this.#node);
    }
    setStaticProperty(kind, value) {
        // value can be either static or mutable
        let staticValue = fastn_utils.getStaticValue(value);
        if (kind === fastn_dom.PropertyKind.Width) {
            this.#node.style.width = staticValue;
        if (kind === fastn_dom.PropertyKind.Padding) {
            this.#node.style.padding = staticValue;
        } else if (kind === fastn_dom.PropertyKind.Color_RGB) {
            this.#node.style.color = staticValue;
        } else if (kind === fastn_dom.PropertyKind.IntegerValue ||
            kind === fastn_dom.PropertyKind.StringValue
        ) {
            this.#node.innerHTML = staticValue;
        } else {
            throw ("invalid fastn_dom.PropertyKind: " + kind);
        }
    }
    setProperty(kind, value) {
        if (value instanceof fastn.mutableClass) {
            this.setDynamicProperty(kind, [value], () => { return value.get(); });
        } else {
            this.setStaticProperty(kind, value);
        }
    }
    setDynamicProperty(kind, deps, func) {
        let closure = fastn.closure(func).addNodeProperty(this, kind);
        for (let dep in deps) {
            deps[dep].addClosure(closure);
            this.#mutables.push(deps[dep]);
        }
    }
    getNode() {
        return this.#node;
    }
    addEventHandler(event, func) {
        if (event === fastn_dom.Event.Click) {
            this.#node.onclick = func;
        }
    }
    destroy() {
        for (let i = 0; i < this.#mutables.length; i++) {
            this.#mutables[i].unlinkNode(this);
        }
        this.#node.remove();
        this.#mutables = null;
        this.#parent = null;
        this.#node = null;
    }
}

class ConditionalDom {
    #parent;
    #node_constructor;
    #condition;
    #mutables;
    #conditionUI;

    constructor(parent, deps, condition, node_constructor) {
        let domNode = fastn_dom.createKernel(parent, fastn_dom.ElementKind.Div);

        this.#conditionUI = null;
        let closure = fastn.closure(() => {
            if (condition()) {
                if (!!this.#conditionUI) {
                    this.#conditionUI.destroy();
                }
                this.#conditionUI = node_constructor(domNode);
            } else if (!!this.#conditionUI) {
                this.#conditionUI.destroy();
                this.#conditionUI = null;
            }
        })
        deps.forEach(dep => dep.addClosure(closure));

        domNode.done();

        this.#parent = domNode;
        this.#node_constructor = node_constructor;
        this.#condition = condition;
        this.#mutables = [];
    }
}

fastn_dom.createKernel = function (parent, kind) {
    return new Node2(parent, kind);
}

fastn_dom.conditionalDom = function (parent, deps, condition, node_constructor) {
    return new ConditionalDom(parent, deps, condition, node_constructor);
}

class ForLoop {
    #node_constructor;
    #list;
    #wrapper;
    constructor(parent, node_constructor, list) {
        this.#wrapper = fastn_dom.createKernel(parent, fastn_dom.ElementKind.Div);
        this.#node_constructor = node_constructor;
        this.#list = list;
        for (let idx in list.getList()) {
            // let v = list.get(idx);
            // node_constructor(this.#wrapper, v.item, v.index).done();
            this.createNode(idx);
        }
        this.#wrapper.done();
    }
    createNode(index) {
        let v = this.#list.get(index);
        this.#node_constructor(this.#wrapper, v.item, v.index).done();
    }
}

fastn_dom.forLoop = function (parent, node_constructor, list) {
    return new ForLoop(parent, node_constructor, list);
}
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
            return window.document.body;
        }

        if (hydrating) {
            return this.getElementByDataID(id_counter);
        } else {
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
    return body.toHtmlAsString()
}
function main(parent) {
let i0 = fastn_dom.createKernel(parent, fastn_dom.ElementKind.Text); i0.done();}
fastn_virtual.ssr(main)
