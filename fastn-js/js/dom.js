let fastn_dom = {};

fastn_dom.codeData = {
    availableThemes: {},
    addedCssFile: []
}

fastn_dom.externalCss = new Set();
fastn_dom.externalJs = new Set();

fastn_dom.commentNode = "comment";
fastn_dom.wrapperNode = "wrapper";
fastn_dom.commentMessage = "***FASTN***";

fastn_dom.classes = { }
fastn_dom.unsanitised_classes = {}
fastn_dom.class_count = 0;
fastn_dom.propertyMap = {
    "align-items": "ali",
    "align-self": "as",
    "background-color": "bgc",
    "background-image": "bgi",
    "background-position": "bgp",
    "background-repeat": "bgr",
    "background-size": "bgs",
    "border-bottom-color": "bbc",
    "border-bottom-left-radius": "bblr",
    "border-bottom-right-radius": "bbrr",
    "border-bottom-style": "bbs",
    "border-bottom-width": "bbw",
    "border-color": "bc",
    "border-left-color": "blc",
    "border-left-style": "bls",
    "border-left-width": "blw",
    "border-radius": "br",
    "border-right-color": "brc",
    "border-right-style": "brs",
    "border-right-width": "brw",
    "border-style": "bs",
    "border-top-color": "btc",
    "border-top-left-radius": "btlr",
    "border-top-right-radius": "btrr",
    "border-top-style": "bts",
    "border-top-width": "btw",
    "border-width": "bw",
    "bottom": "b",
    "color": "c",
    "shadow": "sh",
    "cursor": "cur",
    "display": "d",
    "flex-wrap": "fw",
    "font-style": "fst",
    "font-weight": "fwt",
    "gap": "g",
    "height": "h",
    "justify-content": "jc",
    "left": "l",
    "link": "lk",
    "margin": "m",
    "margin-bottom": "mb",
    "margin-horizontal": "mh",
    "margin-left": "ml",
    "margin-right": "mr",
    "margin-top": "mt",
    "margin-vertical": "mv",
    "max-height": "mxh",
    "max-width": "mxw",
    "min-height": "mnh",
    "min-width": "mnw",
    "opacity": "op",
    "overflow": "o",
    "overflow-x": "ox",
    "overflow-y": "oy",
    "padding": "p",
    "padding-bottom": "pb",
    "padding-horizontal": "ph",
    "padding-left": "pl",
    "padding-right": "pr",
    "padding-top": "pt",
    "padding-vertical": "pv",
    "position": "pos",
    "resize": "res",
    "role": "rl",
    "right": "r",
    "sticky": "s",
    "text-align": "ta",
    "text-decoration": "td",
    "text-transform": "tt",
    "top": "t",
    "width": "w",
    "z-index": "z",
    "-webkit-box-orient": "wbo",
    "-webkit-line-clamp": "wlc",
};

// dynamic-class-css.md
fastn_dom.getClassesAsString = function() {
    let classes = Object.entries(fastn_dom.classes).map(entry => {
        return getClassAsString(entry[0], entry[1]);
    });

    /*.ft_text {
        padding: 0;
    }*/
    return `<style id="styles">
    ${classes.join("\n\t")}
    </style>`;
}

function getClassAsString(className, obj) {
    if (typeof obj.value === 'object' && obj.value !== null) {
        let value = "";
        for (let key in obj.value) {
            if (obj.value[key] === undefined || obj.value[key] === null) {
                continue
            }
            value = `${value} ${key}: ${obj.value[key]};`
        }
        return `${className} { ${value} }`
    } else {
        return `${className} { ${obj.property}: ${obj.value}; }`;
    }
}

fastn_dom.ElementKind = {
    Row: 0,
    Column: 1,
    Integer: 2,
    Decimal: 3,
    Boolean: 4,
    Text: 5,
    Image: 6,
    IFrame: 7,
    // To create parent for dynamic DOM
    Comment: 8,
    CheckBox: 9,
    TextInput: 10,
    ContainerElement: 11,
    Rive: 12,
    Document: 13,
    Wrapper: 14,
    Code: 15,
    // Note: This is called internally, it gives `code` as tagName. This is used
    // along with the Code: 15.
    CodeChild: 16
};

fastn_dom.PropertyKind = {
    Color: 0,
    IntegerValue: 1,
    StringValue: 2,
    DecimalValue: 3,
    BooleanValue: 4,
    Width: 5,
    Padding: 6,
    Height: 7,
    Id: 8,
    BorderWidth: 9,
    BorderStyle: 10,
    Margin: 11,
    Background: 12,
    PaddingHorizontal: 13,
    PaddingVertical: 14,
    PaddingLeft: 15,
    PaddingRight: 16,
    PaddingTop: 17,
    PaddingBottom: 18,
    MarginHorizontal: 19,
    MarginVertical: 20,
    MarginLeft: 21,
    MarginRight: 22,
    MarginTop: 23,
    MarginBottom: 24,
    Role: 25,
    ZIndex: 26,
    Sticky: 27,
    Top: 28,
    Bottom: 29,
    Left: 30,
    Right: 31,
    Overflow: 32,
    OverflowX: 33,
    OverflowY: 34,
    Spacing: 35,
    Wrap: 36,
    TextTransform: 37,
    TextIndent: 38,
    TextAlign: 39,
    LineClamp: 40,
    Opacity: 41,
    Cursor: 42,
    Resize: 43,
    MinHeight: 44,
    MaxHeight: 45,
    MinWidth: 46,
    MaxWidth: 47,
    WhiteSpace: 48,
    BorderTopWidth: 49,
    BorderBottomWidth: 50,
    BorderLeftWidth: 51,
    BorderRightWidth: 52,
    BorderRadius: 53,
    BorderTopLeftRadius: 54,
    BorderTopRightRadius: 55,
    BorderBottomLeftRadius: 56,
    BorderBottomRightRadius: 57,
    BorderStyleVertical: 58,
    BorderStyleHorizontal: 59,
    BorderLeftStyle: 60,
    BorderRightStyle: 61,
    BorderTopStyle: 62,
    BorderBottomStyle: 63,
    BorderColor: 64,
    BorderLeftColor: 65,
    BorderRightColor: 66,
    BorderTopColor: 67,
    BorderBottomColor: 68,
    AlignSelf: 69,
    Classes: 70,
    Anchor: 71,
    Link: 72,
    Children: 73,
    OpenInNewTab: 74,
    TextStyle: 75,
    Region: 76,
    AlignContent: 77,
    Display: 78,
    Checked: 79,
    Enabled: 80,
    TextInputType: 81,
    Placeholder: 82,
    Multiline: 83,
    DefaultTextInputValue: 84,
    Loading: 85,
    Src: 86,
    YoutubeSrc: 87,
    Code: 88,
    ImageSrc: 89,
    Alt: 90,
    DocumentProperties: {
        MetaTitle: 91,
        MetaOGTitle: 92,
        MetaTwitterTitle: 93,
        MetaDescription: 94,
        MetaOGDescription: 95,
        MetaTwitterDescription: 96,
        MetaOGImage: 97,
        MetaTwitterImage: 98,
        MetaThemeColor: 99,
    },
    Shadow: 100,
    CodeTheme: 101,
    CodeLanguage: 102,
    CodeShowLineNumber: 103,
    Css: 104,
    Js: 105,
};



fastn_dom.Loading = {
    Lazy: "lazy",
    Eager: "eager",
}

fastn_dom.TextInputType = {
    Text: "text",
    Email: "email",
    Password: "password",
    Url: "url",
    DateTime: "datetime",
    Date: "date",
    Time: "time",
    Month: "month",
    Week: "week",
    Color: "color",
    File: "file",
}

fastn_dom.AlignContent = {
    TopLeft: "top-left",
    TopCenter: "top-center",
    TopRight: "top-right",
    Right: "right",
    Left: "left",
    Center: "center",
    BottomLeft: "bottom-left",
    BottomRight: "bottom-right",
    BottomCenter: "bottom-center",
}

fastn_dom.Region = {
    H1: "h1",
    H2: "h2",
    H3: "h3",
    H4: "h4",
    H5: "h5",
    H6: "h6",
}

fastn_dom.Anchor = {
    Window: [1, "fixed"],
    Parent: [2, "absolute"],
    Id: (value) => { return [3, value]; },
}

fastn_dom.DeviceData = {
    Desktop: "desktop",
    Mobile: "mobile",
}

fastn_dom.TextStyle = {
    Underline: "underline",
    Italic: "italic",
    Strike: "line-through",
    Heavy: "900",
    Extrabold: "800",
    Bold: "700",
    SemiBold: "600",
    Medium: "500",
    Regular: "400",
    Light: "300",
    ExtraLight: "200",
    Hairline: "100",
}

fastn_dom.Resizing = {
    FillContainer: "100%",
    HugContent: "fit-content",
    Auto: "auto",
    Fixed: (value) => { return value; }
}

fastn_dom.Spacing = {
    SpaceEvenly: [1, "space-evenly"],
    SpaceBetween: [2, "space-between"],
    SpaceAround: [3, "space-around"],
    Fixed: (value) => { return [4, value]; }
}


fastn_dom.BorderStyle = {
    Solid: "solid",
    Dashed: "dashed",
    Dotted: "dotted",
    Double: "double",
    Ridge: "ridge",
    Groove: "groove",
    Inset: "inset",
    Outset: "outset",
}

fastn_dom.Overflow = {
    Scroll: "scroll",
    Visible: "visible",
    Hidden: "hidden",
    Auto: "auto",
}

fastn_dom.Display = {
    Block: "block",
    Inline: "inline",
    InlineBlock: "inline-block",
}

fastn_dom.AlignSelf = {
    Start: "start",
    Center: "center",
    End: "end",
}

fastn_dom.TextTransform = {
    None: "none",
    Capitalize: "capitalize",
    Uppercase: "uppercase",
    Lowercase: "lowercase",
    Inherit: "inherit",
    Initial: "initial",
}

fastn_dom.TextAlign = {
    Start: "start",
    Center: "center",
    End: "end",
    Justify: "justify",
}

fastn_dom.Cursor = {
    None: "none",
    Default: "default",
    ContextMenu: "context-menu",
    Help: "help",
    Pointer: "pointer",
    Progress: "progress",
    Wait: "wait",
    Cell: "cell",
    CrossHair: "crosshair",
    Text: "text",
    VerticalText: "vertical-text",
    Alias: "alias",
    Copy: "copy",
    Move: "move",
    NoDrop: "no-drop",
    NotAllowed: "not-allowed",
    Grab: "grab",
    Grabbing: "grabbing",
    EResize: "e-resize",
    NResize: "n-resize",
    NeResize: "ne-resize",
    SResize: "s-resize",
    SeResize: "se-resize",
    SwResize: "sw-resize",
    Wresize: "w-resize",
    Ewresize: "ew-resize",
    NsResize: "ns-resize",
    NeswResize: "nesw-resize",
    NwseResize: "nwse-resize",
    ColResize: "col-resize",
    RowResize: "row-resize",
    AllScroll: "all-scroll",
    ZoomIn: "zoom-in",
    ZoomOut: "zoom-out"
}

fastn_dom.Resize = {
    Vertical: "vertical",
    Horizontal: "horizontal",
    Both: "both",
}

fastn_dom.WhiteSpace = {
    Normal: "normal",
    NoWrap: "nowrap",
    Pre: "pre",
    PreLine: "pre-line",
    PreWrap: "pre-wrap",
    BreakSpaces: "break-spaces",
}



fastn_dom.BackgroundStyle = {
    Solid: (value) => {
        return [1, value];
    },
    Image: (value) => {
        return [2, value];
    },
    LinearGradient: (value) => {
        return [3, value];
    },
}

fastn_dom.BackgroundRepeat = {
    Repeat: "repeat",
    RepeatX: "repeat-x",
    RepeatY: "repeat-y",
    NoRepeat: "no-repeat",
    Space: "space",
    Round: "round",
}

fastn_dom.BackgroundSize = {
    Auto: "auto",
    Cover: "cover",
    Contain: "contain",
    Length: (value) => { return value; },
}

fastn_dom.BackgroundPosition = {
    Left: "left",
    Right: "right",
    Center: "center",
    LeftTop: "left top",
    LeftCenter: "left center",
    LeftBottom: "left bottom",
    CenterTop: "center top",
    CenterCenter: "center center",
    CenterBottom: "center bottom",
    RightTop: "right top",
    RightCenter: "right center",
    RightBottom: "right bottom",
    Length: (value) => { return value; },
}

fastn_dom.LinearGradientDirection = {
    Angle: (value) => { return `${value}deg`; },
    Turn: (value) => { return `${value}turn`; },
    Left: "270deg",
    Right: "90deg",
    Top: "0deg",
    Bottom: "180deg",
    TopLeft: "315deg",
    TopRight: "45deg",
    BottomLeft: "225deg",
    BottomRight: "135deg",
}

fastn_dom.FontSize = {
    Px: (value) => {
        if (value instanceof fastn.mutableClass) {
            return fastn.formula([value], function () { return `${value.get()}px`})
        }
        return `${value}px`;
    },
    Em: (value) => {
        if (value instanceof fastn.mutableClass) {
            return fastn.formula([value], function () { return `${value.get()}em`})
        }
        return `${value}em`;
    },
    Rem: (value) => {
        if (value instanceof fastn.mutableClass) {
            return fastn.formula([value], function () { return `${value.get()}rem`})
        }
        return `${value}rem`;
    },
}

fastn_dom.Length = {
    Px: (value) => {
        if (value instanceof fastn.mutableClass) {
            return fastn.formula([value], function () { return `${value.get()}px`})
        }
        return `${value}px`;
    },
    Em: (value) => {
        if (value instanceof fastn.mutableClass) {
            return fastn.formula([value], function () { return `${value.get()}em`})
        }
        return `${value}em`;
    },
    Rem: (value) => {
        if (value instanceof fastn.mutableClass) {
            return fastn.formula([value], function () { return `${value.get()}rem`})
        }
        return `${value}rem`;
    },
    Percent: (value) => {
        if (value instanceof fastn.mutableClass) {
            return fastn.formula([value], function () { return `${value.get()}%`})
        }
        return `${value}%`;
    },
    Calc: (value) => {
        if (value instanceof fastn.mutableClass) {
            return fastn.formula([value], function () { return `calc(${value.get()})`})
        }
        return `calc(${value})`;
    },
    Vh: (value) => {
        if (value instanceof fastn.mutableClass) {
            return fastn.formula([value], function () { return `${value.get()}vh`})
        }
        return `${value}vh`;
    },
    Vw: (value) => {
        if (value instanceof fastn.mutableClass) {
            return fastn.formula([value], function () { return `${value.get()}vw`})
        }
        return `${value}vw`;
    },
    Vmin: (value) => {
        if (value instanceof fastn.mutableClass) {
            return fastn.formula([value], function () { return `${value.get()}vmin`})
        }
        return `${value}vmin`;
    },
    Vmax: (value) => {
        if (value instanceof fastn.mutableClass) {
            return fastn.formula([value], function () { return `${value.get()}vmax`})
        }
        return `${value}vmax`;
    },
    Responsive: (desktop, mobile) => {
        if (ftd.device.get() === "desktop") {
            return desktop;
        } else {
            return mobile ? mobile: desktop;
        }
    }
}



fastn_dom.Event = {
    Click: 0,
    MouseEnter: 1,
    MouseLeave: 2,
    ClickOutside: 3,
    GlobalKey: (val) => {return [4, val];},
    GlobalKeySeq: (val) => {return [5, val];},
    Input: 6,
    Change: 7,
    Blur: 8,
    Focus: 9,
}

// Node2 -> Intermediate node
// Node -> similar to HTML DOM node (Node2.#node)
class Node2 {
    #node;
    #kind;
    #parent;
    #tagName;
    /**
     * This is where we store all the attached closures, so we can free them
     * when we are done.
     */
    #mutables;
    /**
     * This is where we store the extraData related to node. This is
     * especially useful to store data for integrated external library (like
     * rive).
     */
    #extraData;
    #children;
    constructor(parentOrSibiling, kind) {
        this.#kind = kind;
        this.#parent = parentOrSibiling;
        this.#children = [];
        let sibiling = undefined;

        if (parentOrSibiling instanceof ParentNodeWithSibiling) {
            this.#parent = parentOrSibiling.getParent();
            sibiling = parentOrSibiling.getSibiling();
        }

        this.createNode(kind);

        this.#mutables = [];
        this.#extraData = {};
        /*if (!!parent.parent) {
            parent = parent.parent();
        }*/


        if (this.#parent.getNode) {
            this.#parent = this.#parent.getNode();
        }

        if (fastn_utils.isWrapperNode(this.#tagName)) {
            this.#parent = parentOrSibiling;
            return;
        }
        if (sibiling) {
            this.#parent.insertBefore(this.#node, fastn_utils.nextSibling(sibiling, this.#parent));
        } else {
            this.#parent.appendChild(this.#node);
        }
    }
    createNode(kind) {
        if (kind === fastn_dom.ElementKind.Code) {
            let [node, classes, attributes] = fastn_utils.htmlNode(kind);
            [this.#tagName, this.#node] = fastn_utils.createNodeHelper(node, classes, attributes);
            let codeNode = new Node2(this.#node, fastn_dom.ElementKind.CodeChild);
            this.#children.push(codeNode);
        } else {
            let [node, classes, attributes] = fastn_utils.htmlNode(kind);
            [this.#tagName, this.#node] = fastn_utils.createNodeHelper(node, classes, attributes);
        }
    }
    getTagName(){
        return this.#tagName;
    }
    getParent() {
        return this.#parent;
    }
    // for attaching inline attributes
    attachAttribute(property, value) {
        if (fastn_utils.isNull(value)) {
            this.#node.removeAttribute(property);
        }
        this.#node.setAttribute(property, value);
    }
    updateTagName(name) {
        if (ssr) {
            this.#node.updateTagName(name);
        }
    }

    updateToAnchor() {
        let node_kind = this.#kind;
        if (ssr) {
            if (node_kind !== fastn_dom.ElementKind.Image) this.updateTagName('a');
        }
    }
    updatePositionForNodeById(node_id, value) {
        if (hydrating) {
            const target_node = document.querySelector(`[id="${node_id}"]`);
            if (target_node !== null && target_node !== undefined)
                target_node.style['position'] = value;
        }
    }
    updateParentPosition(value) {
        if (hydrating) {
            let current_node = document.querySelector(`[data-id="${id_counter}"]`);
            if (current_node) {
                let parent_node = current_node.parentNode;
                parent_node.style['position'] = value;
            }
        }
    }

    updateMetaTitle(value) {
        if (!ssr && hydrating) {
            window.document.title = value;
        }
    }
    addMetaTagByName(name, value) {
        if (value === null || value === undefined) {
            this.removeMetaTagByName(name);
            return;
        }
        if (!ssr && hydrating) {
            const metaTag = window.document.createElement('meta');
            metaTag.setAttribute('name', name);
            metaTag.setAttribute('content', value);
            document.head.appendChild(metaTag);
        }
    }
    addMetaTagByProperty(property, value) {
        if (value === null || value === undefined) {
            this.removeMetaTagByProperty(property);
            return;
        }
        if (!ssr && hydrating) {
            const metaTag = window.document.createElement('meta');
            metaTag.setAttribute('property', property);
            metaTag.setAttribute('content', value);
            document.head.appendChild(metaTag);
        }
    }
    removeMetaTagByName(name) {
        if (!ssr && hydrating) {
            const metaTags = document.getElementsByTagName('meta');
            for (let i = 0; i < metaTags.length; i++) {
                const metaTag = metaTags[i];
                if (metaTag.getAttribute('name') === name) {
                    metaTag.remove();
                    break;
                }
            }
        }
    }
    removeMetaTagByProperty(property) {
        if (!ssr && hydrating) {
            const metaTags = document.getElementsByTagName('meta');
            for (let i = 0; i < metaTags.length; i++) {
                const metaTag = metaTags[i];
                if (metaTag.getAttribute('property') === property) {
                    metaTag.remove();
                    break;
                }
            }
        }
    }
    // dynamic-class-css
    attachCss(property, value, createClass, className) {
        const propertyShort = fastn_dom.propertyMap[property] || property;
        let cls = `${propertyShort}-${JSON.stringify(value)}`;
        if (!!className) {
           cls = className;
        } else {
            if (!fastn_dom.unsanitised_classes[cls]) {
                fastn_dom.unsanitised_classes[cls] = ++fastn_dom.class_count;
            }
            cls = `${propertyShort}-${fastn_dom.unsanitised_classes[cls]}`;
        }
        let cssClass = className ? cls : `.${cls}`;

        const obj = { property, value };

        if (value === undefined) {
            if (!ssr && !hydrating) {
                for (const className of this.#node.classList.values()) {
                    if (className.startsWith(`${propertyShort}-`)) {
                        this.#node.classList.remove(className);
                    }
                }
            }
            return cls;
        }

        if (!ssr && !hydrating) {
            if (!!className) {
                if (!fastn_dom.classes[cssClass]) {
                    fastn_dom.classes[cssClass] = fastn_dom.classes[cssClass] || obj;
                    let styles = document.getElementById('styles');
                    styles.innerHTML = `${styles.innerHTML}${getClassAsString(cssClass, obj)}\n`;
                }
                return cls;
            }

            for (const className of this.#node.classList.values()) {
                if (className.startsWith(`${propertyShort}-`)) {
                    this.#node.classList.remove(className);
                }
            }

            if (createClass) {
                if (!fastn_dom.classes[cssClass]) {
                    fastn_dom.classes[cssClass] = fastn_dom.classes[cssClass] || obj;
                    let styles = document.getElementById('styles');
                    styles.innerHTML = `${styles.innerHTML}${getClassAsString(cssClass, obj)}\n`;
                }
                this.#node.style.removeProperty(property);
                this.#node.classList.add(cls);
            } else if (!fastn_dom.classes[cssClass]) {
                if (typeof value === 'object' && value !== null) {
                    for (let key in value) {
                        this.#node.style[key] = value[key];
                    }
                } else {
                    this.#node.style[property] = value;
                }
            } else {
                this.#node.style.removeProperty(property);
                this.#node.classList.add(cls);
            }

            return cls;
        }

        fastn_dom.classes[cssClass] = fastn_dom.classes[cssClass] || obj;

        if (!!className) {
            return cls;
        }

        this.#node.classList.add(cls);
        return cls;
    }
    attachShadow(value) {
        if (fastn_utils.isNull(value)) {
            this.attachCss("box-shadow", value);
            return;
        }

        const color = value.get("color");

        const lightColor = fastn_utils.getStaticValue(color.get("light"));
        const darkColor = fastn_utils.getStaticValue(color.get("dark"));

        const blur = fastn_utils.getStaticValue(value.get("blur"));
        const xOffset = fastn_utils.getStaticValue(value.get("x_offset"));
        const yOffset = fastn_utils.getStaticValue(value.get("y_offset"));
        const spread = fastn_utils.getStaticValue(value.get("spread"));
        const inset = fastn_utils.getStaticValue(value.get("inset"));

        const shadowCommonCss = `${inset ? "inset " : ""}${xOffset} ${yOffset} ${blur} ${spread}`;
        const lightShadowCss =  `${shadowCommonCss} ${lightColor}`;
        const darkShadowCss = `${shadowCommonCss} ${darkColor}`;

        if (lightShadowCss === darkShadowCss) {
            this.attachCss("box-shadow", lightShadowCss, false);
        } else {
            let lightClass = this.attachCss("box-shadow", lightShadowCss, true);
            this.attachCss("box-shadow", darkShadowCss, true, `body.dark .${lightClass}`);
        }
    }
    attachLinearGradientCss(value) {
        if (fastn_utils.isNull(value)) {
            this.attachCss("background-image", value);
            return;
        }
        var lightGradientString = "";
        var darkGradientString = "";

        let colorsList = value.get("colors").get().getList();
        let direction = fastn_utils.getStaticValue(value.get("direction"));
        colorsList.map(function (element) {
            // LinearGradient RecordInstance
            let lg_color = element.item;

            let color = lg_color.get("color").get();
            let lightColor = fastn_utils.getStaticValue(color.get("light"));
            let darkColor = fastn_utils.getStaticValue(color.get("dark"));

            lightGradientString = `${lightGradientString} ${lightColor}`;
            darkGradientString = `${darkGradientString} ${darkColor}`;

            let start = fastn_utils.getStaticValue(lg_color.get("start"));
            if (start !== undefined && start !== null ) {
                lightGradientString = `${lightGradientString} ${start}`;
                darkGradientString = `${darkGradientString} ${start}`;
            }

            let end = fastn_utils.getStaticValue(lg_color.get("end"));
            if (end !== undefined && end !== null ) {
                lightGradientString = `${lightGradientString} ${end}`;
                darkGradientString = `${darkGradientString} ${end}`;
            }

            let stop_position = fastn_utils.getStaticValue(lg_color.get("stop_position"));
            if (stop_position !== undefined && stop_position !== null ) {
                lightGradientString = `${lightGradientString}, ${stop_position}`;
                darkGradientString = `${darkGradientString}, ${stop_position}`;
            }

            lightGradientString = `${lightGradientString},`
            darkGradientString = `${darkGradientString},`
        });

        lightGradientString = lightGradientString.trim().slice(0, -1);
        darkGradientString = darkGradientString.trim().slice(0, -1);

        if (lightGradientString === darkGradientString) {
            this.attachCss("background-image", `linear-gradient(${direction}, ${lightGradientString})`, false);
        } else {
            let lightClass = this.attachCss("background-image", `linear-gradient(${direction}, ${lightGradientString})`,true);
            this.attachCss("background-image", `linear-gradient(${direction}, ${darkGradientString})`, true, `body.dark .${lightClass}`);
        }
    }
    attachBackgroundImageCss(value) {
        if (fastn_utils.isNull(value)) {
            this.attachCss("background-repeat", value);
            this.attachCss("background-position", value);
            this.attachCss("background-size", value);
            this.attachCss("background-image", value);
            return;
        }

        let src = fastn_utils.getStaticValue(value.get("src"));
        let lightValue = fastn_utils.getStaticValue(src.get("light"));
        let darkValue = fastn_utils.getStaticValue(src.get("dark"));

        let position = fastn_utils.getStaticValue(value.get("position"));
        let positionX = null;
        let positionY = null;
        if (position !== null) {
            positionX = fastn_utils.getStaticValue(position.get("x"));
            positionY = fastn_utils.getStaticValue(position.get("y"));

            if (positionX !== null) position = `${positionX}`;
            if (positionY !== null) {
                if (positionX === null) position = `0px ${positionY}`;
                else position = `${position} ${positionY}`;
            }
        }
        let repeat = fastn_utils.getStaticValue(value.get("repeat"));
        let size = fastn_utils.getStaticValue(value.get("size"));
        let sizeX = null;
        let sizeY = null;
        if (size !== null) {
            sizeX = fastn_utils.getStaticValue(size.get("x"));
            sizeY = fastn_utils.getStaticValue(size.get("y"));

            if (sizeX !== null) size = `${sizeX}`;
            if (sizeY !== null) {
                if (sizeX === null) size = `0px ${sizeY}`;
                else size = `${size} ${sizeY}`;
            }
        }

        if (repeat !== null) this.attachCss("background-repeat", repeat);
        if (position !== null) this.attachCss("background-position", position);
        if (size !== null)  this.attachCss("background-size", size);

        if (lightValue === darkValue) {
            this.attachCss("background-image", `url(${lightValue})`, false);
        } else {
            let lightClass = this.attachCss("background-image", `url(${lightValue})`, true);
            this.attachCss("background-image", `url(${darkValue})`, true, `body.dark .${lightClass}`);
        }
    }
    attachExternalCss(css) {
        if (hydrating) {
            let css_tag = document.createElement('link');
            css_tag.rel = 'stylesheet';
            css_tag.type = 'text/css';
            css_tag.href = css;

            let head = document.head || document.getElementsByTagName("head")[0];
            if (!fastn_dom.externalCss.has(css)){
                head.appendChild(css_tag);
                fastn_dom.externalCss.add(css);
            }
        }
    }
    attachExternalJs(js) {
        if (hydrating) {
            let js_tag = document.createElement('script');
            js_tag.src = js;

            let head = document.head || document.getElementsByTagName("head")[0];
            if (!fastn_dom.externalJs.has(js)){
                head.appendChild(js_tag);
                fastn_dom.externalCss.add(js);
            }
        }
    }
    attachColorCss(property, value, visited) {
        if (fastn_utils.isNull(value)) {
            this.attachCss(property, value);
            return;
        }
        let lightValue = fastn_utils.getStaticValue(value.get("light"));
        let darkValue = fastn_utils.getStaticValue(value.get("dark"));
        if (lightValue === darkValue) {
            this.attachCss(property, lightValue, false);
        } else {
            let lightClass = this.attachCss(property, lightValue, true);
            this.attachCss(property, darkValue, true, `body.dark .${lightClass}`);
            if (visited) {
                this.attachCss(property, lightValue, true, `.${lightClass}:visited`);
                this.attachCss(property, darkValue, true, `body.dark  .${lightClass}:visited`);
            }
        }
    }
    attachRoleCss(value) {
        if (fastn_utils.isNull(value)) {
            this.attachCss('role', value);
            return;
        }
        let desktopValue = fastn_utils.getStaticValue(value.get("desktop"));
        let mobileValue = fastn_utils.getStaticValue(value.get("mobile"));
        if (fastn_utils.sameResponsiveRole(desktopValue, mobileValue)) {
            this.attachCss("role", fastn_utils.getRoleValues(desktopValue), true);
        } else {
            let desktopClass = this.attachCss("role", fastn_utils.getRoleValues(desktopValue), true);
            this.attachCss("role", fastn_utils.getRoleValues(mobileValue), true, `body.mobile .${desktopClass}`);
        }
    }
    attachTextStyles(styles) {
        if (fastn_utils.isNull(styles)) {
            this.attachCss('font-style', styles);
            this.attachCss('font-weight', styles);
            this.attachCss('text-decoration', styles);
            return;
        }
        for (var s of styles) {
            switch (s) {
              case 'italic':
                this.attachCss("font-style", s);
                break;
              case 'underline':
              case 'line-through':
                this.attachCss("text-decoration", s);
                break;
              default:
                this.attachCss("font-weight", s);
            }
        }
    }
    attachAlignContent(value, node_kind) {
        if (fastn_utils.isNull(value)) {
            this.attachCss('align-items', value);
            return;
        }
        if (node_kind === fastn_dom.ElementKind.Row) {
            switch (value) {
                case 'top-left':
                case 'left':
                case 'bottom-left':
                    this.attachCss("align-items", "start");
                    break;
                case 'top-center':
                case 'center':
                case 'bottom-center':
                    this.attachCss("align-items", "center");
                    break;
                case 'top-right':
                case 'right':
                case 'bottom-right':
                    this.attachCss("align-items", "end");
                    break;
            }
        }

        if (node_kind === fastn_dom.ElementKind.Column) {
            switch (value) {
                case 'top-left':
                case 'top-center':
                case 'top-right':
                    this.attachCss("align-items", "start");
                    break;
                case 'left':
                case 'center':
                case 'right':
                    this.attachCss("align-items", "center");
                    break;
                case 'bottom-left':
                case 'bottom-center':
                case 'bottom-right':
                    this.attachCss("align-items", "end");
                    break;
            }
        }
    }
    setStaticProperty(kind, value, inherited) {
        // value can be either static or mutable
        let staticValue = fastn_utils.getStaticValue(value);
        if (kind === fastn_dom.PropertyKind.Children) {
            if (fastn_utils.isWrapperNode(this.#tagName)) {
                let parentWithSibiling = this.#parent;
                if (Array.isArray(staticValue)) {
                    staticValue.forEach((func, index) => {
                        if (index !== 0) {
                            parentWithSibiling = new ParentNodeWithSibiling(this.#parent.getParent(), this.#children[index-1]);
                        }
                        this.#children.push(fastn_utils.getStaticValue(func.item)(parentWithSibiling, inherited))
                    });
                } else {
                    this.#children.push(staticValue(parentWithSibiling, inherited));
                }
            } else {
                if (Array.isArray(staticValue)) {
                    staticValue.forEach(func =>
                        this.#children.push(fastn_utils.getStaticValue(func.item)(this, inherited)));
                } else {
                    this.#children.push(staticValue(this, inherited));
                }
            }
        } else if (kind === fastn_dom.PropertyKind.Id) {
            this.#node.id = staticValue;
        } else if (kind === fastn_dom.PropertyKind.Css) {
            let css_list = staticValue.map(obj => fastn_utils.getStaticValue(obj.item));
            css_list.forEach((css) => {
                this.attachExternalCss(css);
            });
        } else if (kind === fastn_dom.PropertyKind.Js) {
            let js_list = staticValue.map(obj => fastn_utils.getStaticValue(obj.item));
            js_list.forEach((js) => {
                this.attachExternalJs(js);
            });
        } else if (kind === fastn_dom.PropertyKind.Width) {
            this.attachCss("width", staticValue);
        } else if (kind === fastn_dom.PropertyKind.Height) {
            this.attachCss("height", staticValue);
        } else if (kind === fastn_dom.PropertyKind.Padding) {
            this.attachCss("padding", staticValue);
        } else if (kind === fastn_dom.PropertyKind.PaddingHorizontal) {
            this.attachCss("padding-left", staticValue);
            this.attachCss("padding-right", staticValue);
        } else if (kind === fastn_dom.PropertyKind.PaddingVertical) {
            this.attachCss("padding-top", staticValue);
            this.attachCss("padding-bottom", staticValue);
        } else if (kind === fastn_dom.PropertyKind.PaddingLeft) {
            this.attachCss("padding-left", staticValue);
        } else if (kind === fastn_dom.PropertyKind.PaddingRight) {
            this.attachCss("padding-right", staticValue);
        } else if (kind === fastn_dom.PropertyKind.PaddingTop) {
            this.attachCss("padding-top", staticValue);
        } else if (kind === fastn_dom.PropertyKind.PaddingBottom) {
            this.attachCss("padding-bottom", staticValue);
        } else if (kind === fastn_dom.PropertyKind.Margin) {
            this.attachCss("margin", staticValue);
        } else if (kind === fastn_dom.PropertyKind.MarginHorizontal) {
            this.attachCss("margin-left", staticValue);
            this.attachCss("margin-right", staticValue);
        } else if (kind === fastn_dom.PropertyKind.MarginVertical) {
            this.attachCss("margin-top", staticValue);
            this.attachCss("margin-bottom", staticValue);
        } else if (kind === fastn_dom.PropertyKind.MarginLeft) {
            this.attachCss("margin-left", staticValue);
        } else if (kind === fastn_dom.PropertyKind.MarginRight) {
            this.attachCss("margin-right", staticValue);
        } else if (kind === fastn_dom.PropertyKind.MarginTop) {
            this.attachCss("margin-top", staticValue);
        } else if (kind === fastn_dom.PropertyKind.MarginBottom) {
            this.attachCss("margin-bottom", staticValue);
        } else if (kind === fastn_dom.PropertyKind.BorderWidth) {
            this.attachCss("border-width", staticValue);
        } else if (kind === fastn_dom.PropertyKind.BorderTopWidth) {
            this.attachCss("border-top-width", staticValue);
        } else if (kind === fastn_dom.PropertyKind.BorderBottomWidth) {
            this.attachCss("border-bottom-width", staticValue);
        } else if (kind === fastn_dom.PropertyKind.BorderLeftWidth) {
            this.attachCss("border-left-width", staticValue);
        } else if (kind === fastn_dom.PropertyKind.BorderRightWidth) {
            this.attachCss("border-right-width", staticValue);
        } else if (kind === fastn_dom.PropertyKind.BorderRadius) {
            this.attachCss("border-radius", staticValue);
        } else if (kind === fastn_dom.PropertyKind.BorderTopLeftRadius) {
            this.attachCss("border-top-left-radius", staticValue);
        } else if (kind === fastn_dom.PropertyKind.BorderTopRightRadius) {
            this.attachCss("border-top-right-radius", staticValue);
        } else if (kind === fastn_dom.PropertyKind.BorderBottomLeftRadius) {
            this.attachCss("border-bottom-left-radius", staticValue);
        } else if (kind === fastn_dom.PropertyKind.BorderBottomRightRadius) {
            this.attachCss("border-bottom-right-radius", staticValue);
        } else if (kind === fastn_dom.PropertyKind.BorderStyle) {
            this.attachCss("border-style", staticValue);
        } else if (kind === fastn_dom.PropertyKind.BorderStyleVertical) {
            this.attachCss("border-top-style", staticValue);
            this.attachCss("border-bottom-style", staticValue);
        } else if (kind === fastn_dom.PropertyKind.BorderStyleHorizontal) {
            this.attachCss("border-left-style", staticValue);
            this.attachCss("border-right-style", staticValue);
        } else if (kind === fastn_dom.PropertyKind.BorderLeftStyle) {
            this.attachCss("border-left-style", staticValue);
        } else if (kind === fastn_dom.PropertyKind.BorderRightStyle) {
            this.attachCss("border-right-style", staticValue);
        } else if (kind === fastn_dom.PropertyKind.BorderTopStyle) {
            this.attachCss("border-top-style", staticValue);
        } else if (kind === fastn_dom.PropertyKind.BorderBottomStyle) {
            this.attachCss("border-bottom-style", staticValue);
        } else if (kind === fastn_dom.PropertyKind.ZIndex) {
            this.attachCss("z-index", staticValue);
        } else if (kind === fastn_dom.PropertyKind.Shadow) {
            this.attachShadow(staticValue);
        } else if (kind === fastn_dom.PropertyKind.Classes) {
            let cls = staticValue.map(obj => fastn_utils.getStaticValue(obj.item));
            cls.forEach((c) => {
               this.#node.classList.add(c);
            });
        } else if (kind === fastn_dom.PropertyKind.Anchor) {
            // todo: this needs fixed for anchor.id = v
            // need to change position of element with id = v to relative
            if (fastn_utils.isNull(staticValue)) {
                this.attachCss("position", staticValue);
                return;
            }

            let anchorType = staticValue[0];
            switch (anchorType) {
              case 1:
                this.attachCss("position", staticValue[1]);
                break;
              case 2:
                this.attachCss("position", staticValue[1]);
                this.updateParentPosition("relative");
                break;
              case 3:
                const parent_node_id = staticValue[1];
                this.attachCss("position", "absolute");
                this.updatePositionForNodeById(parent_node_id, "relative");
                break;
            }
        } else if (kind === fastn_dom.PropertyKind.Sticky) {
            // sticky is boolean type
            switch (staticValue) {
              case 'true':
              case true:
                this.attachCss("position", "sticky");
                break;
              case 'false':
              case false:
                this.attachCss("position", "static");
                break;
              default:
                this.attachCss("position", staticValue);
            }
        } else if (kind === fastn_dom.PropertyKind.Top) {
            this.attachCss("top", staticValue);
        } else if (kind === fastn_dom.PropertyKind.Bottom) {
            this.attachCss("bottom", staticValue);
        } else if (kind === fastn_dom.PropertyKind.Left) {
            this.attachCss("left", staticValue);
        } else if (kind === fastn_dom.PropertyKind.Right) {
            this.attachCss("right", staticValue);
        } else if (kind === fastn_dom.PropertyKind.Overflow) {
            this.attachCss("overflow", staticValue);
        } else if (kind === fastn_dom.PropertyKind.OverflowX) {
            this.attachCss("overflow-x", staticValue);
        } else if (kind === fastn_dom.PropertyKind.OverflowY) {
            this.attachCss("overflow-y", staticValue);
        } else if (kind === fastn_dom.PropertyKind.Spacing) {
            if (fastn_utils.isNull(staticValue)) {
                this.attachCss("justify-content", staticValue);
                this.attachCss("gap", staticValue);
                return;
            }

            let spacingType = staticValue[0];
            switch (spacingType) {
                case fastn_dom.Spacing.SpaceEvenly[0]:
                case fastn_dom.Spacing.SpaceBetween[0]:
                case fastn_dom.Spacing.SpaceAround[0]:
                    this.attachCss("justify-content", staticValue[1]);
                    break;
                case fastn_dom.Spacing.Fixed()[0]:
                    this.attachCss("gap", staticValue[1]);
                    break;
            }

        } else if (kind === fastn_dom.PropertyKind.Wrap) {
            // sticky is boolean type
            switch (staticValue) {
              case 'true':
              case true:
                this.attachCss("flex-wrap", "wrap");
                break;
              case 'false':
              case false:
                this.attachCss("flex-wrap", "no-wrap");
                break;
              default:
                this.attachCss("flex-wrap", staticValue);
            }
        } else if (kind === fastn_dom.PropertyKind.TextTransform) {
            this.attachCss("text-transform", staticValue);
        } else if (kind === fastn_dom.PropertyKind.TextIndent) {
            this.attachCss("text-indent", staticValue);
        } else if (kind === fastn_dom.PropertyKind.TextAlign) {
            this.attachCss("text-align", staticValue);
        } else if (kind === fastn_dom.PropertyKind.LineClamp) {
            // -webkit-line-clamp: staticValue
            // display: -webkit-box, overflow: hidden
            // -webkit-box-orient: vertical
            this.attachCss("-webkit-line-clamp", staticValue);
            this.attachCss("display", "-webkit-box");
            this.attachCss("overflow", "hidden");
            this.attachCss("-webkit-box-orient", "vertical");
        } else if (kind === fastn_dom.PropertyKind.Opacity) {
            this.attachCss("opacity", staticValue);
        } else if (kind === fastn_dom.PropertyKind.Cursor) {
            this.attachCss("cursor", staticValue);
        } else if (kind === fastn_dom.PropertyKind.Resize) {
            // overflow: auto, resize: staticValue
            this.attachCss("resize", staticValue);
            this.attachCss("overflow", "auto");
        } else if (kind === fastn_dom.PropertyKind.MinHeight) {
            this.attachCss("min-height", staticValue);
        } else if (kind === fastn_dom.PropertyKind.MaxHeight) {
            this.attachCss("max-height", staticValue);
        } else if (kind === fastn_dom.PropertyKind.MinWidth) {
            this.attachCss("min-width", staticValue);
        } else if (kind === fastn_dom.PropertyKind.MaxWidth) {
            this.attachCss("max-width", staticValue);
        } else if (kind === fastn_dom.PropertyKind.WhiteSpace) {
            this.attachCss("white-space", staticValue);
        } else if (kind === fastn_dom.PropertyKind.AlignSelf) {
            this.attachCss("align-self", staticValue);
        } else if (kind === fastn_dom.PropertyKind.BorderColor) {
            this.attachColorCss("border-color", staticValue);
        } else if (kind === fastn_dom.PropertyKind.BorderLeftColor) {
            this.attachColorCss("border-left-color", staticValue);
        } else if (kind === fastn_dom.PropertyKind.BorderRightColor) {
            this.attachColorCss("border-right-color", staticValue);
        } else if (kind === fastn_dom.PropertyKind.BorderTopColor) {
            this.attachColorCss("border-top-color", staticValue);
        } else if (kind === fastn_dom.PropertyKind.BorderBottomColor) {
            this.attachColorCss("border-bottom-color", staticValue);
        } else if (kind === fastn_dom.PropertyKind.Color) {
            this.attachColorCss("color", staticValue, true);
        } else if (kind === fastn_dom.PropertyKind.Background) {
            if (fastn_utils.isNull(staticValue)) {
                this.attachColorCss("background-color", staticValue);
                this.attachBackgroundImageCss(staticValue);
                this.attachLinearGradientCss(staticValue);
                return;
            }

            let backgroundType = staticValue[0];
            switch (backgroundType) {
                case fastn_dom.BackgroundStyle.Solid()[0]:
                    this.attachColorCss("background-color", staticValue[1]);
                    break;
                case fastn_dom.BackgroundStyle.Image()[0]:
                    this.attachBackgroundImageCss(staticValue[1]);
                    break;
                case fastn_dom.BackgroundStyle.LinearGradient()[0]:
                    this.attachLinearGradientCss(staticValue[1]);
                    break;
            }
        } else if (kind === fastn_dom.PropertyKind.Display) {
            this.attachCss("display", staticValue);
        } else if (kind === fastn_dom.PropertyKind.Checked) {
            switch (staticValue) {
                case "true":
                case true:
                    this.attachAttribute("checked", "");
                    break;
                default:
                    this.attachAttribute("checked", staticValue);
            }
        } else if (kind === fastn_dom.PropertyKind.Enabled) {
            switch (staticValue) {
                case "false":
                case false:
                    this.attachAttribute("disabled", "");
                    break;
                default:
                    this.attachAttribute("disabled", staticValue);
            }
        } else if (kind === fastn_dom.PropertyKind.TextInputType) {
            this.attachAttribute("type", staticValue);
        } else if (kind === fastn_dom.PropertyKind.DefaultTextInputValue) {
            this.attachAttribute("value", staticValue);
        } else if (kind === fastn_dom.PropertyKind.Placeholder) {
            this.attachAttribute("placeholder", staticValue);
        } else if (kind === fastn_dom.PropertyKind.Multiline) {
            switch (staticValue) {
                case "true":
                case true:
                    this.updateTagName("textarea");
                    break;
                case "false":
                case false:
                    this.updateTagName("input");
                    break;
            }
        } else if (kind === fastn_dom.PropertyKind.Link) {
            // Changing node type to `a` for link
            // todo: needs fix for image links
            this.updateToAnchor();
            this.attachAttribute("href", staticValue);
        } else if (kind === fastn_dom.PropertyKind.OpenInNewTab) {
            // open_in_new_tab is boolean type
            switch (staticValue) {
              case 'true':
              case true:
                this.attachAttribute("target", "_blank");
                break;
              default:
                this.attachAttribute("target", staticValue);
            }
        } else if (kind === fastn_dom.PropertyKind.TextStyle) {
            let styles = staticValue.map(obj => fastn_utils.getStaticValue(obj.item));
            this.attachTextStyles(styles);
        } else if (kind === fastn_dom.PropertyKind.Region) {
            this.updateTagName(staticValue);
            if (this.#node.innerHTML) {
                // todo: need to slugify this id
                this.#node.id = this.#node.innerHTML;
            }
        } else if (kind === fastn_dom.PropertyKind.AlignContent) {
            let node_kind = this.#kind;
            this.attachAlignContent(staticValue, node_kind);
        } else if (kind === fastn_dom.PropertyKind.Loading) {
            this.attachAttribute("loading", staticValue);
        } else if (kind === fastn_dom.PropertyKind.Src) {
            this.attachAttribute("src", staticValue);
        } else if (kind === fastn_dom.PropertyKind.ImageSrc) {
            ftd.dark_mode.addClosure(fastn.closure(() => {
                if (fastn_utils.isNull(staticValue)) {
                    this.attachAttribute("src", staticValue);
                    return;
                }
                const is_dark_mode = ftd.dark_mode.get();
                const src = staticValue.get(is_dark_mode ? 'dark' : 'light');

                this.attachAttribute("src", fastn_utils.getStaticValue(src));
            }).addNodeProperty(this, null, inherited));
            this.#mutables.push(ftd.dark_mode);
        } else if (kind === fastn_dom.PropertyKind.Alt) {
            this.attachAttribute("alt", staticValue);
        } else if (kind === fastn_dom.PropertyKind.YoutubeSrc) {
            if (fastn_utils.isNull(staticValue)) {
                this.attachAttribute("src", staticValue);
                return;
            }
            const id_pattern = "^([a-zA-Z0-9_-]{11})$";
            let id = staticValue.match(id_pattern);
            this.attachAttribute("src", `https:\/\/youtube.com/embed/${id[0]}`);
        } else if (kind === fastn_dom.PropertyKind.Role) {
            this.attachRoleCss(staticValue);
        } else if (kind === fastn_dom.PropertyKind.Code) {
            if (!fastn_utils.isNull(staticValue)) {
                let {
                    modifiedText,
                    highlightedLines
                } = fastn_utils.findAndRemoveHighlighter(staticValue);
                if (highlightedLines.length !== 0) {
                    this.attachAttribute("data-line", highlightedLines);
                }
                staticValue = modifiedText;
            }
            this.#children[0].getNode().innerHTML= staticValue;
        }  else if (kind === fastn_dom.PropertyKind.CodeShowLineNumber) {
            if (staticValue) {
                this.#node.classList.add("line-numbers");
            } else {
                this.#node.classList.remove("line-numbers");
            }
        } else if (kind === fastn_dom.PropertyKind.CodeTheme) {
            if (!ssr) {
                fastn_utils.addCodeTheme(staticValue);
            }
            let theme = staticValue.replace("\.", "-");
            this.#node.classList.add(theme);
            this.#children[0].getNode().classList.add(theme);
        } else if (kind === fastn_dom.PropertyKind.CodeLanguage) {
            let language = `language-${staticValue}`;
            this.#node.classList.add(language);
            this.#children[0].getNode().classList.add(language);
        } else if (kind === fastn_dom.PropertyKind.DocumentProperties.MetaTitle) {
            this.updateMetaTitle(staticValue);
        } else if (kind === fastn_dom.PropertyKind.DocumentProperties.MetaOGTitle) {
            this.addMetaTagByProperty("og:title", staticValue);
        } else if (kind === fastn_dom.PropertyKind.DocumentProperties.MetaTwitterTitle) {
            this.addMetaTagByName("twitter:title", staticValue);
        } else if (kind === fastn_dom.PropertyKind.DocumentProperties.MetaDescription) {
            this.addMetaTagByName("description", staticValue);
        } else if (kind === fastn_dom.PropertyKind.DocumentProperties.MetaOGDescription) {
            this.addMetaTagByProperty("og:description", staticValue);
        } else if (kind === fastn_dom.PropertyKind.DocumentProperties.MetaTwitterDescription) {
            this.addMetaTagByName("twitter:description", staticValue);
        } else if (kind === fastn_dom.PropertyKind.DocumentProperties.MetaOGImage) {
            // staticValue is of ftd.raw-image-src RecordInstance type
            if (fastn_utils.isNull(staticValue)) {
                this.removeMetaTagByProperty("og:image");
                return;
            }
            this.addMetaTagByProperty("og:image", fastn_utils.getStaticValue(staticValue.get('src')));
        } else if (kind === fastn_dom.PropertyKind.DocumentProperties.MetaTwitterImage) {
            // staticValue is of ftd.raw-image-src RecordInstance type
            if (fastn_utils.isNull(staticValue)) {
                this.removeMetaTagByName("twitter:image");
                return;
            }
            this.addMetaTagByName("twitter:image", fastn_utils.getStaticValue(staticValue.get('src')));
        } else if (kind === fastn_dom.PropertyKind.DocumentProperties.MetaThemeColor) {
            // staticValue is of ftd.color RecordInstance type
            if (fastn_utils.isNull(staticValue)) {
                this.removeMetaTagByName("theme-color");
                return;
            }
            this.addMetaTagByName("theme-color", fastn_utils.getStaticValue(staticValue.get('light')));
        } else if (kind === fastn_dom.PropertyKind.IntegerValue
            || kind === fastn_dom.PropertyKind.DecimalValue
            || kind === fastn_dom.PropertyKind.BooleanValue) {
            this.#node.innerHTML = staticValue;
        } else if (kind === fastn_dom.PropertyKind.StringValue) {
            if (!ssr) {
                staticValue = fastn_utils.markdown_inline(staticValue);
            }
            this.#node.innerHTML = staticValue;
        }else {
            throw ("invalid fastn_dom.PropertyKind: " + kind);
        }
    }
    setProperty(kind, value, inherited) {
        if (value instanceof fastn.mutableClass) {
            this.setDynamicProperty(kind, [value], () => { return value.get(); });
        } else {
            this.setStaticProperty(kind, value, inherited);
        }
    }
    setDynamicProperty(kind, deps, func, inherited) {
        let closure = fastn.closure(func).addNodeProperty(this, kind, inherited);
        for (let dep in deps) {
            if (fastn_utils.isNull(deps[dep]) || !deps[dep].addClosure) {
                continue;
            }
            deps[dep].addClosure(closure);
            this.#mutables.push(deps[dep]);
        }
    }
    getNode() {
        return this.#node;
    }
    getExtraData() {
        return this.#extraData
    }
    getChildren() {
        return this.#children;
    }
    mergeFnCalls(current, newFunc) {
        return () => {
            if (current instanceof Function) current();
            if (newFunc instanceof Function) newFunc();
        };
    }
    addEventHandler(event, func) {
        if (event === fastn_dom.Event.Click) {
            let onclickEvents = this.mergeFnCalls(this.#node.onclick, func);
            this.#node.onclick = onclickEvents;
        } else if (event === fastn_dom.Event.MouseEnter) {
            let mouseEnterEvents = this.mergeFnCalls(this.#node.onmouseenter, func);
            this.#node.onmouseenter = mouseEnterEvents;
        } else if (event === fastn_dom.Event.MouseLeave) {
            let mouseLeaveEvents = this.mergeFnCalls(this.#node.onmouseleave, func);
            this.#node.onmouseleave = mouseLeaveEvents;
        } else if (event === fastn_dom.Event.ClickOutside) {
            ftd.clickOutsideEvents.push([this, func]);
        } else if (!!event[0] && event[0] === fastn_dom.Event.GlobalKey()[0]) {
            ftd.globalKeyEvents.push([this, func, event[1]]);
        } else if (!!event[0] && event[0] === fastn_dom.Event.GlobalKeySeq()[0]) {
            ftd.globalKeySeqEvents.push([this, func, event[1]]);
        } else if (event === fastn_dom.Event.Input) {
            let onInputEvents = this.mergeFnCalls(this.#node.oninput, func);
            this.#node.oninput = onInputEvents;
        } else if (event === fastn_dom.Event.Change) {
            let onChangeEvents = this.mergeFnCalls(this.#node.onchange, func);
            this.#node.onchange = onChangeEvents;
        } else if (event === fastn_dom.Event.Blur) {
            let onBlurEvents = this.mergeFnCalls(this.#node.onblur, func);
            this.#node.onblur = onBlurEvents;
        } else if (event === fastn_dom.Event.Focus) {
            let onFocusEvents = this.mergeFnCalls(this.#node.onfocus, func);
            this.#node.onfocus = onFocusEvents;
        }
    }
    destroy() {
        for (let i = 0; i < this.#mutables.length; i++) {
            this.#mutables[i].unlinkNode(this);
        }
        this.#node.remove();
        this.#mutables = null;
        this.#parent = null;
        this.#node = null;
    }
}

class ConditionalDom {
    #marker;
    #parent;
    #node_constructor;
    #condition;
    #mutables;
    #conditionUI;

    constructor(parent, deps, condition, node_constructor) {
        this.#marker = fastn_dom.createKernel(parent, fastn_dom.ElementKind.Comment);
        this.#parent = parent;

        this.#conditionUI = null;
        let closure = fastn.closure(() => {
            if (condition()) {
                if (this.#conditionUI) {
                    if (Array.isArray(this.#conditionUI)) {
                        while (this.#conditionUI.length > 0) {
                            let poppedElement = this.#conditionUI.pop();
                            poppedElement.destroy();
                        }
                    } else {
                        this.#conditionUI.destroy();
                    }
                }
                this.#conditionUI = node_constructor(new ParentNodeWithSibiling(this.#parent, this.#marker));
                if (fastn_utils.isWrapperNode(this.#conditionUI.getTagName())) {
                    this.#conditionUI = this.#conditionUI.getChildren();
                }
            } else if (this.#conditionUI) {
                if (Array.isArray(this.#conditionUI)) {
                    while (this.#conditionUI.length > 0) {
                        let poppedElement = this.#conditionUI.pop();
                        poppedElement.destroy();
                    }
                } else {
                    this.#conditionUI.destroy();
                }
                this.#conditionUI = null;
            }
        })
        deps.forEach(dep => {
            if (!fastn_utils.isNull(dep) && dep.addClosure) {
                dep.addClosure(closure);
            }
        });

        this.#node_constructor = node_constructor;
        this.#condition = condition;
        this.#mutables = [];
    }

    getParent() {
        let nodes =  [this.#marker];
        if (this.#conditionUI) {
            nodes.push(this.#conditionUI);
        }
        nodes
    }
}

fastn_dom.createKernel = function (parent, kind) {
    return new Node2(parent, kind);
}

fastn_dom.conditionalDom = function (parent, deps, condition, node_constructor) {
    return new ConditionalDom(parent, deps, condition, node_constructor);
}

class ParentNodeWithSibiling {
    #parent;
    #sibiling;
    constructor(parent, sibiling) {
        this.#parent = parent;
        this.#sibiling = sibiling;
    }
    getParent() {
        return this.#parent;
    }
    getSibiling() {
        return this.#sibiling;
    }
}

class ForLoop {
    #node_constructor;
    #list;
    #wrapper;
    #parent;
    #nodes;
    constructor(parent, node_constructor, list) {
        this.#wrapper = fastn_dom.createKernel(parent, fastn_dom.ElementKind.Comment);
        this.#parent = parent;
        this.#node_constructor = node_constructor;
        this.#list = list;
        this.#nodes = [];

        for (let idx in list.getList()) {
            let node = this.createNode(idx);
            this.#nodes.push(node);
        }
    }
    createNode(index) {
        let parentWithSibiling = new ParentNodeWithSibiling(this.#parent, this.#wrapper);
        if (index !== 0) {
            parentWithSibiling = new ParentNodeWithSibiling(this.#parent, this.#nodes[index-1]);
        }
        let v = this.#list.get(index);
        return this.#node_constructor(parentWithSibiling, v.item, v.index);
    }

    getWrapper() {
        return this.#wrapper;
    }

    insertNode(index, node) {
        this.#nodes.splice(index, 0, node);
    }

    getParent() {
        return this.#parent;
    }
}

fastn_dom.forLoop = function (parent, node_constructor, list) {
    return new ForLoop(parent, node_constructor, list);
}
