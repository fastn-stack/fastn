class MutableVariable {
    #value;
    #closures;
    constructor(value) {
        this.#value = value;
        this.#closures = [];
    }

    get() {
        return fastn_utils.getStaticValue(this.#value);
    }

    set(value) {
        this.#value.set(value);
    }

    on_change(func) {
        this.#value.addClosure(fastn.closureWithoutExecute(func));
    }
}

class MutableListVariable {
    #value;
    #closures;
    constructor(value) {
        this.#value = value;
        this.#closures = [];
        // this.#value.addClosure(fastn.closure(() => this.#closures.forEach((closure) => closure.update())));
    }

    get() {
        return fastn_utils.getStaticValue(this.#value);
    }

    set(list) {
        list = list.map((value) => {
            if (value instanceof Object) {
                value = new RecordInstance(value);
            }
            return value;
        })

        this.#value.set(list);
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
            this.#value.addClosure(fastn.closure(() => this.#closures.forEach((closure) => closure.update())));
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

fastn.webComponentVariable =  {
    mutable: (value) => {
        return new MutableVariable(value);
    },
    mutableList: (value) => {
        return new MutableListVariable(value);
    },
    static: (value) => {
        return new StaticVariable(value);
    }
}
