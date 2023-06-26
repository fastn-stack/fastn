let fastn_dom = {};

fastn_dom.classes = {}
fastn_dom.unsanitised_classes = {}
fastn_dom.class_count = 0;
fastn_dom.property_map = {"color": "c", "width": "w"};

fastn_dom.getClassesAsString = function() {
    let classes = Object.entries(fastn_dom.classes).map(entry => {
        return `.${entry[0]} { ${entry[1].property}: ${entry[1].value}; }`;
    });

    if (classes.length == 0) {
        return "";
    }

    return `<style>${classes.join("\n")}</style>`;
}

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
    Width: 3,
}

fastn_dom.Resizing = {
    FillContainer: "100%",
    HugContent: "fit-content",
    Fixed: (value) => { return value; }
}

fastn_dom.Length = {
    Px: (value) => {
        return `${value}px`;
    },
    Em: (value) => {
        return `${value}em`;
    },
    Responsive: (desktop, mobile) => {
        if (ftd.device == "desktop") {
            return desktop;
        } else {
            return mobile ? mobile: desktop;
        }
    }
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
    attachCss(property, value) {
        let property_short = fastn_dom.property_map[property];
        let cls = `${property_short}-${value}`;
        if (!fastn_dom.unsanitised_classes[cls]) {
            fastn_dom.unsanitised_classes[cls] = ++fastn_dom.class_count;
        }
        cls = `c${fastn_dom.unsanitised_classes[cls]}`;
        fastn_dom.classes[cls] = fastn_dom.classes[cls] || { property: property, value: value };
        this.#node.classList.add(cls);
    }
    setStaticProperty(kind, value) {
        // value can be either static or mutable
        let staticValue = fastn_utils.getStaticValue(value);
        if (kind === fastn_dom.PropertyKind.Width) {
            this.attachCss("width", staticValue);
        } else if (kind === fastn_dom.PropertyKind.Color_RGB) {
            this.attachCss("color", staticValue);
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
