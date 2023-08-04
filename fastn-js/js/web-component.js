class MutableVariable {
    #value;
    #closures;
    constructor(value) {
        this.#value = value;
        this.#closures = [];
        this.#value.addClosure(fastn.closure(() => this.#closures.forEach((closure) => closure.update())));
    }

    get() {
        return fastn_utils.getStaticValue(this.#value);
    }

    set(value) {
        this.#value.set(value);
    }

    on_change(func) {
        this.#closures.push(fastn.closure(func));
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
        this.#closures.push(fastn.closure(func));
    }
}

fastn.webComponentVariable =  {
    mutable: (value) => {
        return new MutableVariable(value);
    },
    static: (value) => {
        return new StaticVariable(value);
    }
}
