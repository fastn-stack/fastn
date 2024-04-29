const fastn = (function (fastn) {
    class Closure {
        #cached_value;
        #node;
        #property;
        #formula;
        #inherited;
        constructor(func, execute = true) {
            if (execute) {
                this.#cached_value = func();
            }
            this.#formula = func;
        }

        get() {
            return this.#cached_value;
        }
        getFormula() {
            return this.#formula;
        }
        addNodeProperty(node, property, inherited) {
            this.#node = node;
            this.#property = property;
            this.#inherited = inherited;
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
            if (
                !this.#node ||
                this.#property === null ||
                this.#property === undefined ||
                !this.#node.getNode()
            ) {
                return;
            }

            this.#node.setStaticProperty(
                this.#property,
                this.#cached_value,
                this.#inherited,
            );
        }
    }

    class Mutable {
        #value;
        #old_closure;
        #closures;
        #closureInstance;
        constructor(val) {
            this.#value = null;
            this.#old_closure = null;
            this.#closures = [];
            this.#closureInstance = fastn.closure(() =>
                this.#closures.forEach((closure) => closure.update()),
            );
            this.set(val);
        }
        get(key) {
            if (
                !fastn_utils.isNull(key) &&
                (this.#value instanceof RecordInstance ||
                    this.#value instanceof MutableList ||
                    this.#value instanceof Mutable)
            ) {
                return this.#value.get(key);
            }
            return this.#value;
        }
        setWithoutUpdate(value) {
            if (this.#old_closure) {
                this.#value.removeClosure(this.#old_closure);
            }

            if (this.#value instanceof RecordInstance) {
                // this.#value.replace(value); will replace the record type
                // variable instance created which we don't want.
                // color: red
                // color if { something }: $orange-green
                // The `this.#value.replace(value);` will replace the value of
                // `orange-green` with `{light: red, dark: red}`
                this.#value = value;
            } else {
                this.#value = value;
            }

            if (this.#value instanceof Mutable) {
                this.#old_closure = fastn.closureWithoutExecute(() =>
                    this.#closureInstance.update(),
                );
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
            this.#closures = this.#closures.filter(
                (closure) => closure.getNode() !== node,
            );
        }
        addClosure(closure) {
            this.#closures.push(closure);
        }
        removeClosure(closure) {
            this.#closures = this.#closures.filter((c) => c !== closure);
        }
        equalMutable(other) {
            if (!fastn_utils.deepEqual(this.get(), other.get())) {
                return false;
            }
            const thisClosures = this.#closures;
            const otherClosures = other.#closures;

            return thisClosures === otherClosures;
        }
        getClone() {
            return new Mutable(fastn_utils.clone(this.#value));
        }
    }

    class Proxy {
        #differentiator;
        #cached_value;
        #closures;
        #closureInstance;
        constructor(targets, differentiator) {
            this.#differentiator = differentiator;
            this.#cached_value = this.#differentiator().get();
            this.#closures = [];

            let proxy = this;
            for (let idx in targets) {
                targets[idx].addClosure(
                    new Closure(function () {
                        proxy.update();
                        proxy.#closures.forEach((closure) => closure.update());
                    }),
                );
                targets[idx].addClosure(this);
            }
        }
        addClosure(closure) {
            this.#closures.push(closure);
        }
        removeClosure(closure) {
            this.#closures = this.#closures.filter((c) => c !== closure);
        }
        update() {
            this.#cached_value = this.#differentiator().get();
        }
        get(key) {
            if (
                !!key &&
                (this.#cached_value instanceof RecordInstance ||
                    this.#cached_value instanceof MutableList ||
                    this.#cached_value instanceof Mutable)
            ) {
                return this.#cached_value.get(key);
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
        #closures;
        constructor(list) {
            this.#list = [];
            for (let idx in list) {
                this.#list.push({
                    item: fastn.wrapMutable(list[idx]),
                    index: new Mutable(parseInt(idx)),
                });
            }
            this.#watchers = [];
            this.#closures = [];
        }
        addClosure(closure) {
            this.#closures.push(closure);
        }
        unlinkNode(node) {
            this.#closures = this.#closures.filter(
                (closure) => closure.getNode() !== node,
            );
        }
        forLoop(root, dom_constructor) {
            let l = fastn_dom.forLoop(root, dom_constructor, this);
            this.#watchers.push(l);
            return l;
        }
        getList() {
            return this.#list;
        }
        getLength() {
            return this.#list.length;
        }
        get(idx) {
            if (fastn_utils.isNull(idx)) {
                return this.getList();
            }
            return this.#list[idx];
        }
        set(index, value) {
            if (value === undefined) {
                value = index;
                if (!(value instanceof MutableList)) {
                    if (!Array.isArray(value)) {
                        value = [value];
                    }
                    value = new MutableList(value);
                }

                let list = value.#list;
                this.#list = [];
                for (let i in list) {
                    this.#list.push(list[i]);
                }

                for (let i in this.#watchers) {
                    this.#watchers[i].createAllNode();
                }
            } else {
                index = fastn_utils.getFlattenStaticValue(index);
                this.#list[index].item.set(value);
            }

            this.#closures.forEach((closure) => closure.update());
        }
        insertAt(index, value) {
            index = fastn_utils.getFlattenStaticValue(index);
            let mutable = fastn.wrapMutable(value);
            this.#list.splice(index, 0, {
                item: mutable,
                index: new Mutable(index),
            });
            // for every item after the inserted item, update the index
            for (let i = index + 1; i < this.#list.length; i++) {
                this.#list[i].index.set(i);
            }

            for (let i in this.#watchers) {
                this.#watchers[i].createNode(index);
            }
            this.#closures.forEach((closure) => closure.update());
        }
        push(value) {
            this.insertAt(this.#list.length, value);
        }
        deleteAt(index) {
            index = fastn_utils.getFlattenStaticValue(index);
            this.#list.splice(index, 1);
            // for every item after the deleted item, update the index
            for (let i = index; i < this.#list.length; i++) {
                this.#list[i].index.set(i);
            }

            for (let i in this.#watchers) {
                let forLoop = this.#watchers[i];
                forLoop.deleteNode(index);
            }
            this.#closures.forEach((closure) => closure.update());
        }
        clearAll() {
            this.#list = [];
            for (let i in this.#watchers) {
                this.#watchers[i].deleteAllNode();
            }
            this.#closures.forEach((closure) => closure.update());
        }
        pop() {
            this.deleteAt(this.#list.length - 1);
        }
        getClone() {
            let current_list = this.#list;
            let new_list = [];
            for (let idx in current_list) {
                new_list.push(fastn_utils.clone(current_list[idx].item));
            }
            return new MutableList(new_list);
        }
    }

    fastn.mutable = function (val) {
        return new Mutable(val);
    };

    fastn.closure = function (func) {
        return new Closure(func);
    };

    fastn.closureWithoutExecute = function (func) {
        return new Closure(func, false);
    };

    fastn.formula = function (deps, func) {
        let closure = fastn.closure(func);
        let mutable = new Mutable(closure.get());
        for (let idx in deps) {
            if (fastn_utils.isNull(deps[idx]) || !deps[idx].addClosure) {
                continue;
            }
            deps[idx].addClosure(
                new Closure(function () {
                    closure.update();
                    mutable.set(closure.get());
                }),
            );
        }

        return mutable;
    };

    fastn.proxy = function (targets, differentiator) {
        return new Proxy(targets, differentiator);
    };

    fastn.wrapMutable = function (obj) {
        if (
            !(obj instanceof Mutable) &&
            !(obj instanceof RecordInstance) &&
            !(obj instanceof MutableList)
        ) {
            obj = new Mutable(obj);
        }
        return obj;
    };

    fastn.mutableList = function (list) {
        return new MutableList(list);
    };

    class RecordInstance {
        #fields;
        #closures;
        constructor(obj) {
            this.#fields = {};
            this.#closures = [];

            for (let key in obj) {
                if (obj[key] instanceof fastn.mutableClass) {
                    this.#fields[key] = fastn.mutable(null);
                    this.#fields[key].setWithoutUpdate(obj[key]);
                } else {
                    this.#fields[key] = fastn.mutable(obj[key]);
                }
            }
        }
        getAllFields() {
            return this.#fields;
        }
        getClonedFields() {
            let clonedFields = {};
            for (let key in this.#fields) {
                let field_value = this.#fields[key];
                if (
                    field_value instanceof fastn.recordInstanceClass ||
                    field_value instanceof fastn.mutableClass ||
                    field_value instanceof fastn.mutableListClass
                ) {
                    clonedFields[key] = this.#fields[key].getClone();
                } else {
                    clonedFields[key] = this.#fields[key];
                }
            }
            return clonedFields;
        }
        addClosure(closure) {
            this.#closures.push(closure);
        }
        unlinkNode(node) {
            this.#closures = this.#closures.filter(
                (closure) => closure.getNode() !== node,
            );
        }
        get(key) {
            return this.#fields[key];
        }
        set(key, value) {
            if (value === undefined) {
                value = key;
                if (!(value instanceof RecordInstance)) {
                    value = new RecordInstance(value);
                }

                let fields = {};
                for (let key in value.#fields) {
                    this.#fields[key].set(value.#fields[key]);
                }

            } else if (this.#fields[key] === undefined) {
                this.#fields[key] = fastn.mutable(null);
                this.#fields[key].setWithoutUpdate(value);
            } else {
                this.#fields[key].set(value);
            }
            this.#closures.forEach((closure) => closure.update());
        }
        setAndReturn(key, value) {
            this.set(key, value);
            return this;
        }
        replace(obj) {
            for (let key in this.#fields) {
                if (!(key in obj.#fields)) {
                    throw new Error(
                        "RecordInstance.replace: key " +
                            key +
                            " not present in new object",
                    );
                }
                this.#fields[key] = fastn.wrapMutable(obj.#fields[key]);
            }
            this.#closures.forEach((closure) => closure.update());
        }
        toObject() {
            return Object.fromEntries(
                Object.entries(this.#fields).map(([key, value]) => [
                    key,
                    fastn_utils.getFlattenStaticValue(value),
                ]),
            );
        }
        getClone() {
            let current_fields = this.#fields;
            let cloned_fields = {};
            for (let key in current_fields) {
                let value = fastn_utils.clone(current_fields[key]);
                if (value instanceof fastn.mutableClass) {
                    value = value.get();
                }
                cloned_fields[key] = value;
            }
            return new RecordInstance(cloned_fields);
        }
    }

    class Module {
        #name;
        #global;
        constructor(name, global) {
            this.#name = name;
            this.#global = global;
        }

        getName() {
            return this.#name;
        }

        get(function_name) {
            return this.#global[`${this.#name}__${function_name}`];
        }
    }

    fastn.recordInstance = function (obj) {
        return new RecordInstance(obj);
    };

    fastn.color = function (r, g, b) {
        return `rgb(${r},${g},${b})`;
    };

    fastn.mutableClass = Mutable;
    fastn.mutableListClass = MutableList;
    fastn.recordInstanceClass = RecordInstance;
    fastn.module = function (name, global) {
        return new Module(name, global);
    };
    fastn.moduleClass = Module;

    return fastn;
})({});
