// this file contains types like mutable etc
// functions for interacting with DOM are defined in dom.js
(function() {
    let fastn = {};

    class Closure {
        #cached_value;
        #node;
        #property;
        #formula;
        constructor(func, node, property) {
            this.#cached_value = func();
            this.#node = node;
            this.#formula = func;
            this.#property = property;
            this.update_ui();
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

    fastn.closure = function (func, node, property) {
        return new Closure(func, node, property)
    }

    window.fastn = fastn;
})();
