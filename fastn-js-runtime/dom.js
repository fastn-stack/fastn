(function() {
    let fastn_dom = {};

    fastn_dom.ElementKind = {
        Row: 0,
        Column: 1,
        Integer: 2,
        Decimal: 3,
        Boolean: 4,
        Text: 5,
        Image: 6,
        IFrame: 7,
    };

    fastn_dom.PropertyKind = {
        Width_Px: 0,
        Color_RGB: 1,
    }

    class Node {
        constructor(parent, kind) {
            let (n, c) = html_node(kind);
            this.#node = document.createElement(n);
            this.#node.classList.append(c);
            parent.addChild(this.#node);
            // this is where store all the closures attached, so we can free them when we are done
            this.#mutables = [];
        }

        set_static_property(kind, value) {
            if (kind === fastn_dom.PropertyKind.Width_Px) {
                this.#node.style.width = value + "px";
            } else if (kind === fastn_dom.PropertyKind.Color_RGB) {
                this.#node.style.color = value;
            } else {
                throw ("invalid fastn_dom.PropertyKind: " + kind);
            }
        }

        destroy() {
            for (let i = 0; i < this.#mutables.length; i++) {
                this.#mutables[i].unlink_node(this);
            }
            this.#mutables = null;
            this.#node = null;
        }

        function html_node(e) {
            if (e === fastn_dom.ElementKind.Row) return ("div", "row");
            if (e === fastn_dom.ElementKind.Column) return ("div", "row");
            if (e === fastn_dom.ElementKind.Integer) return ("div", "row");
            if (e === fastn_dom.ElementKind.Decimal) return ("div", "row");
            if (e === fastn_dom.ElementKind.Boolean) return ("div", "row");
            if (e === fastn_dom.ElementKind.Text) return ("div", "row");
            if (e === fastn_dom.ElementKind.Image) return ("div", "row");
            if (e === fastn_dom.ElementKind.IFrame) return ("div", "row");

            throw ("invalid fastn_dom.ElementKind: " + e);
        }

    }


    fastn_dom.create_kernel = function (parent, kind) {
        return new Node(parent, kind);
    }

    window.fastn_dom = fastn_dom;
})();
