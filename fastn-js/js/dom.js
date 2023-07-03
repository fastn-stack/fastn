let fastn_dom = {};

fastn_dom.classes = {}
fastn_dom.unsanitised_classes = {}
fastn_dom.class_count = 0;
fastn_dom.property_map = {
    "color": "c",
    "width": "w",
    "padding": "p",
    "margin": "m",
    "height": "h",
    "border-width": "bw",
    "border-style": "bs",
    "background-color": "bgc"
};

// dynamic-class-css.md
fastn_dom.getClassesAsString = function() {
    let classes = Object.entries(fastn_dom.classes).map(entry => {
        return getClassAsString(entry[0], entry[1]);
    });

    return `<style id="styles">
    /*.ft_text {
        padding: 0;
    }*/
    ${classes.join("\n")}
    </style>`;
}

function getClassAsString(className, obj) {
    return `${className} { ${obj.property}: ${obj.value}; }`;
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
    Color: 0,
    IntegerValue: 1,
    StringValue: 2,
    Width: 3,
    Padding: 4,
    Height: 5,
    Id: 6,
    BorderWidth: 7,
    BorderStyle: 8,
    Margin: 9,
    Background: 10,
}

fastn_dom.Resizing = {
    FillContainer: "100%",
    HugContent: "fit-content",
    Fixed: (value) => { return value; }
}

fastn_dom.BorderStyle = {
    Solid: "solid",
    Dashed: "dashed",
    Dotted: "dotted",
    Double: "double",
    Ridge: "ridge",
    Groove: "groove",
    Inset: "inset",
    Outset: "outset",
}

fastn_dom.BackgroundStyle = {
    Solid: (value) => { return value; }
}

fastn_dom.Length = {
    Px: (value) => {
        if (value instanceof fastn.mutableClass) {
            return fastn.formula([value], function () { return `${value.get()}px`})
        }
        return `${value}px`;
    },
    Em: (value) => {
        if (value instanceof fastn.mutableClass) {
            return fastn.formula([value], function () { return `${value.get()}em`})
        }
        return `${value}em`;
    },
    Rem: (value) => {
        if (value instanceof fastn.mutableClass) {
            return fastn.formula([value], function () { return `${value.get()}rem`})
        }
        return `${value}rem`;
    },
    Percent: (value) => {
        if (value instanceof fastn.mutableClass) {
            return fastn.formula([value], function () { return `${value.get()}%`})
        }
        return `${value}%`;
    },
    Calc: (value) => {
        if (value instanceof fastn.mutableClass) {
            return fastn.formula([value], function () { return `calc(${value.get()})`})
        }
        return `calc(${value})`;
    },
    Vh: (value) => {
        if (value instanceof fastn.mutableClass) {
            return fastn.formula([value], function () { return `${value.get()}vh`})
        }
        return `${value}vh`;
    },
    Vw: (value) => {
        if (value instanceof fastn.mutableClass) {
            return fastn.formula([value], function () { return `${value.get()}vw`})
        }
        return `${value}vw`;
    },
    Vmin: (value) => {
        if (value instanceof fastn.mutableClass) {
            return fastn.formula([value], function () { return `${value.get()}vmin`})
        }
        return `${value}vmin`;
    },
    Vmax: (value) => {
        if (value instanceof fastn.mutableClass) {
            return fastn.formula([value], function () { return `${value.get()}vmax`})
        }
        return `${value}vmax`;
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
        /*if (!!parent.parent) {
            parent = parent.parent();
        }*/
        if (this.#parent.getNode) {
            this.#parent = this.#parent.getNode();
        }
        this.#parent.appendChild(this.#node);
    }
    parent() {
        return this.#parent;
    }
    // dynamic-class-css
    attachCss(property, value, createClass, className) {
        const propertyShort = fastn_dom.property_map[property] || property;
        let cls = `${propertyShort}-${value}`;
        if (!!className) {
           cls = className;
        } else {
            if (!fastn_dom.unsanitised_classes[cls]) {
                fastn_dom.unsanitised_classes[cls] = ++fastn_dom.class_count;
            }
            cls = `${propertyShort}-${fastn_dom.unsanitised_classes[cls]}`;
        }
        let cssClass = className ? cls : `.${cls}`;

        const obj = { property, value };

        if (value === undefined) {
            if (!ssr && !hydrating) {
                for (const className of this.#node.classList.values()) {
                    if (className.startsWith(`${propertyShort}-`)) {
                        this.#node.classList.remove(className);
                    }
                }
            }
            return cls;
        }

        if (!ssr && !hydrating) {
            if (!!className) {
                if (!fastn_dom.classes[cssClass]) {
                    fastn_dom.classes[cssClass] = fastn_dom.classes[cssClass] || obj;
                    let styles = document.getElementById('styles');
                    styles.innerHTML = `${styles.innerHTML}${getClassAsString(cssClass, obj)}\n`;
                }
                return cls;
            }

            for (const className of this.#node.classList.values()) {
                if (className.startsWith(`${propertyShort}-`)) {
                    this.#node.classList.remove(className);
                }
            }

            if (createClass) {
                if (!fastn_dom.classes[cssClass]) {
                    fastn_dom.classes[cssClass] = fastn_dom.classes[cssClass] || obj;
                    let styles = document.getElementById('styles');
                    styles.innerHTML = `${styles.innerHTML}${getClassAsString(cssClass, obj)}\n`;
                }
                this.#node.style.removeProperty(property);
                this.#node.classList.add(cls);
            } else if (!fastn_dom.classes[cssClass]) {
                this.#node.style[property] = value;
            } else {
                this.#node.style.removeProperty(property);
                this.#node.classList.add(cls);
            }

            return cls;
        }

        fastn_dom.classes[cssClass] = fastn_dom.classes[cssClass] || obj;

        if (!!className) {
            return cls;
        }

        this.#node.classList.add(cls);
        return cls;
    }

    attachColorCss(property, value) {
        let lightValue = fastn_utils.getStaticValue(value.get("light"));
        let darkValue = fastn_utils.getStaticValue(value.get("dark"));
        if (lightValue === darkValue) {
            this.attachCss(property, lightValue, false);
        } else {
            let lightClass = this.attachCss(property, lightValue, true);
            this.attachCss(property, darkValue, true, `body.dark .${lightClass}`);
        }
    }

    setStaticProperty(kind, value) {
        // value can be either static or mutable
        let staticValue = fastn_utils.getStaticValue(value);
        if (kind === fastn_dom.PropertyKind.Id) {
            this.#node.id = staticValue;
        } else if (kind === fastn_dom.PropertyKind.Width) {
            this.attachCss("width", staticValue);
        } else if (kind === fastn_dom.PropertyKind.Height) {
            this.attachCss("height", staticValue);
        } else if (kind === fastn_dom.PropertyKind.Padding) {
            this.attachCss("padding", staticValue);
        } else if (kind === fastn_dom.PropertyKind.Margin) {
            this.attachCss("margin", staticValue);
        } else if (kind === fastn_dom.PropertyKind.BorderWidth) {
            this.attachCss("border-width", staticValue);
        } else if (kind === fastn_dom.PropertyKind.BorderStyle) {
            this.attachCss("border-style", staticValue);
        } else if (kind === fastn_dom.PropertyKind.Color) {
            this.attachColorCss("color", staticValue);
        } else if (kind === fastn_dom.PropertyKind.Background) {
            this.attachColorCss("background-color", staticValue);
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
            if (!deps[dep].addClosure) {
                continue;
            }
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
                if (this.#conditionUI) {
                    this.#conditionUI.destroy();
                }
                this.#conditionUI = node_constructor(domNode);
            } else if (this.#conditionUI) {
                this.#conditionUI.destroy();
                this.#conditionUI = null;
            }
        })
        deps.forEach(dep => dep.addClosure(closure));


        this.#parent = domNode;
        this.#node_constructor = node_constructor;
        this.#condition = condition;
        this.#mutables = [];
    }

    getParent() {
        return this.#parent;
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
    }
    createNode(index) {
        let v = this.#list.get(index);
        this.#node_constructor(this.#wrapper, v.item, v.index);
    }

    getParent() {
        return this.#wrapper;
    }
}

fastn_dom.forLoop = function (parent, node_constructor, list) {
    return new ForLoop(parent, node_constructor, list);
}
