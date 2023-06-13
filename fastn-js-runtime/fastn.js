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
        getClosures() {
            return this.#closures;
        }
        set(value) {
            if (this.#value !== value) {
                this.#value = value;
                this.#closures.forEach(closure => closure.update());
            }
        }
        // we have to unlink all nodes, else they will be kept in memory after the node is removed from DOM
        unlink_node(node) {
            this.#closures = this.#closures.filter(closure => closure.getNode() !== node);
        }
        addClosure(closure) {
            this.#closures.push(closure);
        }
    }

    class Proxy {
        #differentiator
        #cached_value
        #closures;
        constructor(differentiator) {
            this.#differentiator = differentiator;
            this.#cached_value = this.#differentiator().get();
            this.#closures = [];
        }
        updateTargetClosure(targets) {
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
        update() {
            this.#cached_value = this.#differentiator().get();
        }
        get() {
            return this.#cached_value;
        }
        set(value) {
            if (this.#cached_value !== value) {
                this.#differentiator().set(value);
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
        let proxy = new Proxy(differentiator);
        proxy.updateTargetClosure(targets);
        return proxy;
    };


    window.fastn = fastn;
})();
