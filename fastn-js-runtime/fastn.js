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
            if (!this.#node || !this.#property) {
                return;
            }

            this.#node.setStaticProperty(this.#property, this.#cached_value);
        }
    }

    class Mutable {
        #value;
        #closures;
        #closureInstance;
        constructor(val) {
            this.#value = val;
            this.#closures = [];
            this.#closureInstance = fastn.closure(() => this.#closures.forEach((closure) => closure.update()));
        }
        get() {
            return this.#value;
        }
        set(value) {
            const oldValue = this.#value;

            // Todo: Optimization removed. Reuse optimization later again
            /*if (fastn_utils.deepEqual(oldValue, value)) {
                return;
            }*/

            this.#value = value;

            // Get mutables present in the new value but not in the old value
            // Also mutables present in the old value but not in the new value
            const { newMutables, oldMutables} =
                fastn_utils.getNewAndOldMutables(oldValue, value);
            // Add closures to the new mutables
            newMutables.forEach((mutable) =>
                mutable.addClosure(this.#closureInstance)
            );
            // Remove closures from the old mutables
            oldMutables.forEach((mutable) =>
                mutable.removeClosure(this.#closureInstance)
            );

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
        get() {
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

    window.fastn = fastn;
})();
