// this file contains types like mutable etc
// functions for interacting with DOM are defined in dom.js
(function() {
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
        addNodeProperty(node, property) {
            this.#node = node;
            this.#property = property;
            this.update_ui();

            return this;
        }
        update() {
            this.#cached_value = this.#formula();
            this.update_ui();
        }
        getNode() {
            return this.#node;
        }
        update_ui() {
            if (!this.#node || !this.#property) {
                return;
            }

            this.#node.setStaticProperty(this.#property, this.#cached_value);
        }
    }

    class Mutable {
        #value;
        #closures;
        constructor(val) {
            this.#value = val;
            this.#closures = [];
        }
        get() {
            return this.#value;
        }
        set(value) {
            this.#value = value;
            this.#closures.forEach(closure => closure.update());
        }
        // we have to unlink all nodes, else they will be kept in memory after the node is removed from DOM
        unlink_node(node) {
            this.#closures = this.#closures.filter(closure => closure.getNode() !== node);
        }
        addClosure(closure) {
            this.#closures.push(closure);
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
        let m = new Mutable(closure.get());
        for (let dep in deps) {
            deps[dep].addClosure(new Closure(function () {
                closure.update();
                m.set(closure.get());
            }));
        }

        return m;
    }

    window.fastn = fastn;
})();
