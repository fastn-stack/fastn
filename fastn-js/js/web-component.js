class MutableVariable {
    #value;
    constructor(value) {
        this.#value = value;
    }

    get() {
        return fastn_utils.getStaticValue(this.#value);
    }

    set(value) {
        this.#value.set(value);
    }
    // Todo: Remove closure when node is removed.
    on_change(func) {
        this.#value.addClosure(fastn.closureWithoutExecute(func));
    }
}

class MutableListVariable {
    #value;
    constructor(value) {
        this.#value = value;
    }
    get() {
        return fastn_utils.getStaticValue(this.#value);
    }
    set(index, list) {
        if (list === undefined) {
            this.#value.set(fastn_utils.staticToMutables(index));
            return;
        }
        this.#value.set(index, fastn_utils.staticToMutables(list));
    }
    insertAt(index, value) {
        this.#value.insertAt(index, fastn_utils.staticToMutables(value));
    }
    deleteAt(index) {
        this.#value.deleteAt(index);
    }
    push(value) {
        this.#value.push(value);
    }
    pop() {
        this.#value.pop();
    }
    clearAll() {
        this.#value.clearAll();
    }
    on_change(func) {
        this.#value.addClosure(fastn.closureWithoutExecute(func));
    }
}

class RecordVariable {
    #value;
    constructor(value) {
        this.#value = value;
    }

    get() {
        return fastn_utils.getStaticValue(this.#value);
    }

    set(record) {
        this.#value.set(fastn_utils.staticToMutables(record));
    }

    on_change(func) {
        this.#value.addClosure(fastn.closureWithoutExecute(func));
    }
}
class StaticVariable {
    #value;
    #closures;
    constructor(value) {
        this.#value = value;
        this.#closures = [];
        if (this.#value instanceof fastn.mutableClass) {
            this.#value.addClosure(
                fastn.closure(() =>
                    this.#closures.forEach((closure) => closure.update()),
                ),
            );
        }
    }

    get() {
        return fastn_utils.getStaticValue(this.#value);
    }

    on_change(func) {
        if (this.#value instanceof fastn.mutableClass) {
            this.#value.addClosure(fastn.closure(func));
        }
    }
}

fastn.webComponentVariable = {
    mutable: (value) => {
        return new MutableVariable(value);
    },
    mutableList: (value) => {
        return new MutableListVariable(value);
    },
    static: (value) => {
        return new StaticVariable(value);
    },
    record: (value) => {
        return new RecordVariable(value);
    },
};
