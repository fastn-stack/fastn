(function() {
    let fastn_virtual = {}

    let id_counter = 0;
    let hydrating = false;
    let ssr = false;

    class Node {
        #id
        #tagName
        #children
        constructor(id, tagName) {
            this.#tagName = tagName;
            this.#id = id;
            this.classList = [];
            this.#children = [];
            this.innerHTML = "";
            this.style = {};
            this.onclick = null;
        }
        appendChild(c) {
            this.#children.push(c);
        }
        toHtml() {

        }
    }

    class Document {
        createElement(tagName) {
            id_counter++;
            if (ssr) {
                return new Node(id_counter, tagName);
            }
            if (tagName === "body") {
                return fastn_virtual.real_document.body;
            }
            if (!hydrating) {
                return fastn_virtual.real_document.createElement(tagName);
            }
            return fastn_virtual.document.getElementByDataID(id_counter);
        }

        getElementByDataID(id) {
            return fastn_virtual.real_document.getElementById(id);
        }
    }

    fastn_virtual.real_document = window.document;

    fastn_virtual.document = new Document();


    fastn_virtual.hydrate = function(main) {
        hydrating = true;
        let body = fastn_virtual.document.createElement("body");
        main(body)
        hydrating = false;
    }

    fastn_virtual.ssr = function(main) {
        ssr = true;
        let body = fastn_virtual.document.createElement("body");
        main(body)
        ssr = false;
        return body.toHtml()
    }

    window.fastn_virtual = fastn_virtual;
})();
