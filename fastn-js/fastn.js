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
    #old_closure
    #closures;
    #closureInstance;
    constructor(val) {
        this.#value = val;
        this.#old_closure = null;
        this.#closures = [];
        this.#closureInstance = fastn.closure(() => this.#closures.forEach((closure) => closure.update()));
        this.set(val);
    }

    get() {
        return this.#value;
    }

    // x = 10
    // x = 20
    // y = 1
    // x = y
    // y = 20
    // z = 2
    // y = z
    // z = 11
    // x = 20


    set(value) {
        if (this.#old_closure) {
            this.#value.removeClosure(this.#old_closure);
        }

        this.#value = value;

        if (value instanceof Mutable) {
            this.#old_closure = fastn.closure(() => this.#closureInstance.update());
            value.addClosure(this.#old_closure);
        } else {
            this.#old_closure = null;
        }

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
                this.#fields[key].set(obj[key]);
            } else {
                this.#fields[key] = fastn.mutable(obj[key]);
            }
        }
    }
    get(key) {
        return this.#fields[key];
    }
    set(key, value) {
        this.#fields[key].set(fastn.wrapMutable(value));
    }
    replace(obj) {
        for (let key in this.$fields) {
            if (!(key in obj)) {
                throw new Error("RecordInstance.replace: key " + key + " not present in new object");
            }
            this.#fields[key].set(fastn.wrapMutable(obj[key]));
        }
    }
}

fastn.recordInstance = function (obj) {
    return new RecordInstance(obj);
}
