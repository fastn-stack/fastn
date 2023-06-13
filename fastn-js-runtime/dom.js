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
    };

    fastn_dom.PropertyKind = {
        Width_Px: 0,
        Color_RGB: 1,
        IntegerValue: 2,
    }

    fastn_dom.Event = {
        Click: 0
    }

    class Node {
        #node;
        #mutables;
        #closed;
        #children;
        constructor(parent, kind) {
            if (!kind) {
                this.#node = parent;
            } else {
                parent.assert_is_open();
                let [node, classes] = fastn_utils.htmlNode(kind);
                this.#node = document.createElement(node);
                for (let c in classes) {
                    this.#node.classList.add(classes[c]);
                }
                parent.getNode().appendChild(this.#node);
                parent.addChild(this);
            }
            // this is where store all the closures attached, so we can free them when we are done
            this.#mutables = [];
            this.#closed = false;
            this.#children = [];
        }
        addChild(node) {
            this.#children.push(node);
        }
        getNode() {
            return this.#node;
        }
        assert_is_closed() {
            for (let i = 0; i < this.#children.length; i++) {
                this.#children[i].assert_is_closed();
            }
            if (!this.#closed) throw ("fastn_dom.Node is not closed");
        }
        assert_is_open() {
            if (this.#closed) throw ("fastn_dom.Node is closed");
        }
        done() {
            this.assert_is_open()
            this.#closed = true;
        }
        setStaticProperty(kind, value) {
            if (kind === fastn_dom.PropertyKind.Width_Px) {
                this.#node.style.width = value + "px";
            } else if (kind === fastn_dom.PropertyKind.Color_RGB) {
                this.#node.style.color = value;
            } else if (kind === fastn_dom.PropertyKind.IntegerValue) {
                this.#node.innerHTML = value;
            } else {
                throw ("invalid fastn_dom.PropertyKind: " + kind);
            }
        }
        setDynamicProperty(kind, deps, func) {
            this.assert_is_open()
            let closure = fastn.closure(func).addNodeProperty(this, kind);
            for (let dep in deps) {
                deps[dep].addClosure(closure);
            }
        }
        addEventHandler(event, func) {
            this.assert_is_open()
            if (event === fastn_dom.Event.Click) {
                this.#node.onclick = func;
            }
        }
        destroy() {
            for (let i = 0; i < this.#mutables.length; i++) {
                this.#mutables[i].unlinkNode(this);
            }
            this.#mutables = null;
            this.#node = null;
        }
    }

    fastn_dom.createKernel = function (parent, kind) {
        return new Node(parent, kind);
    }

    fastn_dom.node = function (node) {
        return new Node(node)
    }

    window.fastn_dom = fastn_dom;
})();
