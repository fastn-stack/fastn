let fastn_utils = {
    htmlNode(kind) {
        let node = "div";
        let css = [];
        let attributes = {};
        if (kind === fastn_dom.ElementKind.Column) {
            css.push("ft_column");
        } else if (kind === fastn_dom.ElementKind.Row) {
            css.push("ft_row");
        } else if (kind === fastn_dom.ElementKind.IFrame) {
            node = "iframe";
        } else if (kind === fastn_dom.ElementKind.Image) {
            node = "img";
        } else if (kind === fastn_dom.ElementKind.Div ||
            kind === fastn_dom.ElementKind.ContainerElement ||
            kind === fastn_dom.ElementKind.Text) {
            node = "div";
        } else if (kind === fastn_dom.ElementKind.Rive) {
            node = "canvas";
        } else if (kind === fastn_dom.ElementKind.CheckBox) {
            node = "input";
            attributes["type"] = "checkbox";
        } else if (kind === fastn_dom.ElementKind.TextInput) {
            node = "input";
        }
        return [node, css, attributes];
    },

    getStaticValue(obj) {
        if (obj instanceof fastn.mutableClass) {
           return this.getStaticValue(obj.get());
        } if (obj instanceof fastn.mutableListClass) {
            return obj.getList();
        } else {
           return obj;
        }
    },

    getFlattenStaticValue(obj) {
        let staticValue = fastn_utils.getStaticValue(obj);
        if (Array.isArray(staticValue)) {
            return staticValue.map(func =>
                fastn_utils.getFlattenStaticValue(func.item));
        }
        return staticValue;
    },

    getter(value) {
        if (value.get) {
            return value.get();
        } else {
            return value;
        }
    },

    setter(variable, value) {
        if (variable.set) {
           variable.set(value);
           return true;
        }
        return false;
    },

    defaultPropertyValue(_propertyValue) {
        return null;
    },

    sameResponsiveRole(desktop, mobile) {
       return (desktop.get("font_family") ==  mobile.get("font_family")) &&
       (desktop.get("letter_spacing") ==  mobile.get("letter_spacing")) &&
       (desktop.get("line_height") ==  mobile.get("line_height")) &&
       (desktop.get("size") ==  mobile.get("size")) &&
       (desktop.get("weight") ==  mobile.get("weight"));
    },

    getRoleValues(value) {
        return {
            "font-family": fastn_utils.getStaticValue(value.get("font_family")),
            "letter-spacing": fastn_utils.getStaticValue(value.get("letter_spacing")),
            "font-size": fastn_utils.getStaticValue(value.get("size")),
            "font-weight": fastn_utils.getStaticValue(value.get("weight")),
            "line-height": fastn_utils.getStaticValue(value.get("line_height")),
        };
    },

    clone(value) {
        if (value === null || value === undefined) {
            return value;
        }
        if (value instanceof Mutable) {
            let cloned_value = value.getClone();
            return cloned_value;
        }
        return value;
    },
  
    getEventKey(event) {
        if (65 <= event.keyCode && event.keyCode <= 90) {
            return String.fromCharCode(event.keyCode).toLowerCase();
        }
        else {
            return event.key;
        }
    }
}
