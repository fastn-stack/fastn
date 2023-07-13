let fastn_utils = {
    htmlNode(kind) {
        let node = "div";
        let css = [];
        if (kind === fastn_dom.ElementKind.Column) {
            css.push("ft_column");
        } else if (kind === fastn_dom.ElementKind.Row) {
            css.push("ft_row");
        } else if (kind === fastn_dom.ElementKind.IFrame) {
            node = "iframe";
        } else if (kind === fastn_dom.ElementKind.Image) {
            node = "img";
        } else if (kind === fastn_dom.ElementKind.Div) {
            node = "div";
        } else if (kind === fastn_dom.ElementKind.Text) {
            // css.push("ft_text");
        }
        return [node, css];
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
    }
}
