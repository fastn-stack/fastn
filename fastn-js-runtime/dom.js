(function() {
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
        Width_Px: 0,
        Color_RGB: 1,
        IntegerValue: 2,
        StringValue: 3,
    }

    fastn_dom.Event = {
        Click: 0
    }

    class Node {
        #node;
        #parent
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
            if (kind === fastn_dom.PropertyKind.Width_Px) {
                this.#node.style.width = value + "px";
            } else if (kind === fastn_dom.PropertyKind.Color_RGB) {
                this.#node.style.color = value;
            } else if (kind === fastn_dom.PropertyKind.IntegerValue ||
                kind === fastn_dom.PropertyKind.StringValue
            ) {
                this.#node.innerHTML = value;
            } else {
                throw ("invalid fastn_dom.PropertyKind: " + kind);
            }
        }
        setDynamicProperty(kind, deps, func) {
            let closure = fastn.closure(func).addNodeProperty(this, kind);
            for (let dep in deps) {
                deps[dep].addClosure(closure);
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

        constructor(parent, deps, condition, dom) {
            let domNode = fastn_dom.createKernel(parent, fastn_dom.ElementKind.Div);

            let conditionUI = null;
            let closure = fastn.closure(() => {
                if (condition()) {
                    if (!!conditionUI) {
                        conditionUI.destroy();
                    }
                    conditionUI = dom(domNode);
                } else if (!!conditionUI) {
                    conditionUI.destroy();
                    conditionUI = null;
                }
            })
            deps.forEach(dep => dep.addClosure(closure));

            domNode.done();

            this.#parent = domNode;
            this.#node_constructor = dom;
            this.#condition = condition;
            this.#mutables = [];
        }
    }

    fastn_dom.createKernel = function (parent, kind) {
        return new Node(parent, kind);
    }

    fastn_dom.conditionalDom = function (parent, deps, condition, dom) {
        return new ConditionalDom(parent, deps, condition, dom);
    }

    window.fastn_dom = fastn_dom;
})();
