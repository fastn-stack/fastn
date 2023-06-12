// this file contains types like mutable etc
// functions for interacting with DOM are defined in dom.js

(function() {
    let fastn = {};

    class Mutable {
        constructor(val) {
            this.#value = val;
        }
    }

    fastn.mutable = function (val) {
        return new Mutable(val)
    };

    window.fastn = fastn;
})();
