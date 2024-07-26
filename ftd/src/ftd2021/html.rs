use crate::IText;

#[derive(serde::Deserialize, Debug, PartialEq, Default, Clone, serde::Serialize)]
pub struct Node {
    pub condition: Option<ftd::Condition>,
    pub events: Vec<ftd::Event>,
    pub classes: Vec<String>,
    pub node: String,
    pub attrs: ftd::Map<String>,
    pub style: ftd::Map<String>,
    pub children: Vec<Node>,
    pub external_children: Vec<Node>,
    pub open_id: Option<String>,
    pub external_children_container: Vec<Vec<usize>>,
    pub children_style: ftd::Map<String>,
    pub text: Option<String>,
    pub null: bool,
}

impl Node {
    pub fn fixed_children_style(&self, index: usize) -> ftd::Map<String> {
        if index == 1 {
            let mut list: ftd::Map<String> = Default::default();
            for (key, value) in self.children_style.iter() {
                if key == "margin-left" || key == "margin-top" {
                    continue;
                }
                list.insert(key.clone(), value.clone());
            }
            list
        } else {
            self.children_style.clone()
        }
    }

    pub fn is_visible(&self, data: &ftd::DataDependenciesMap) -> bool {
        if self.null {
            return false;
        }

        match self.condition {
            Some(ref v) => v.is_true(data),
            None => true,
        }
    }

    #[allow(clippy::too_many_arguments)]
    pub fn to_dnode(
        &self,
        style: &ftd::Map<String>,
        data: &ftd::DataDependenciesMap,
        external_children: &mut Option<Vec<Self>>,
        external_open_id: &Option<String>,
        external_children_container: &[Vec<usize>],
        is_parent_visible: bool,
        parent_id: &str,
        is_last: bool,
    ) -> ftd::ftd2021::dnode::DNode {
        let style = {
            let mut s = self.style.clone();
            s.extend(style.clone());
            s
        };

        let all_children = {
            let mut children: Vec<ftd::Node> = self.children.to_vec();
            // #[allow(clippy::blocks_in_conditions)]
            if let Some(ext_children) = external_children {
                if *external_open_id
                    == self.attrs.get("data-id").map(|v| {
                        if v.contains(':') {
                            let mut part = v.splitn(2, ':');
                            part.next().unwrap().trim().to_string()
                        } else {
                            v.to_string()
                        }
                    })
                    && self.open_id.is_none()
                    && external_children_container.is_empty()
                    && ((self.is_visible(data) && is_parent_visible) || is_last)
                {
                    for child in ext_children.iter() {
                        if let Some(data_id) = child.attrs.get("data-id") {
                            for child in child.children.iter() {
                                let mut child = child.clone();
                                child.attrs.insert(
                                    "data-ext-id".to_string(),
                                    format!("{}:{}", data_id, parent_id),
                                );
                                children.push(child);
                            }
                        }
                    }
                    *external_children = None;
                }
            }
            children
        };

        let (open_id, external_children_container) =
            if self.open_id.is_some() && external_children_container.is_empty() {
                (&self.open_id, self.external_children_container.as_slice())
            } else {
                (external_open_id, external_children_container)
            };

        let mut ext_child = None;
        let mut is_borrowed_ext_child = false;

        let ext_child: &mut Option<Vec<Self>> = {
            if external_children_container.is_empty() {
                &mut ext_child
            } else if self.open_id.is_some() && !self.external_children.is_empty() {
                ext_child = Some(self.external_children.clone());
                &mut ext_child
            } else {
                is_borrowed_ext_child = true;
                external_children
            }
        };

        let mut index = 0;
        let mut index_of_visible_children = 0;

        let children = {
            let mut children: Vec<ftd::ftd2021::dnode::DNode> = vec![];
            for (i, v) in all_children.iter().enumerate() {
                if v.node.is_empty() {
                    continue;
                }

                let (external_container, is_last) = {
                    let mut external_container = vec![];
                    while index < external_children_container.len() {
                        if let Some(container) = external_children_container[index].get(0) {
                            if container < &i {
                                index += 1;
                                continue;
                            }
                            let external_child_container =
                                external_children_container[index][1..].to_vec();
                            if container == &i {
                                if !external_child_container.is_empty() {
                                    external_container.push(external_child_container)
                                }
                            } else {
                                break;
                            }
                        }
                        index += 1;
                    }
                    let is_last = {
                        let mut last = external_container.len() <= 1
                            && (index >= external_children_container.len()
                                || !is_other_sibling_visible(
                                    i,
                                    &all_children,
                                    index,
                                    external_children_container,
                                ));
                        if is_borrowed_ext_child {
                            last = is_last && last;
                        }
                        last
                    };

                    (external_container, is_last)
                };

                if v.is_visible(data) {
                    index_of_visible_children += 1;
                }

                children.push(v.to_dnode(
                    &self.fixed_children_style(index_of_visible_children),
                    data,
                    ext_child,
                    open_id,
                    external_container.as_slice(),
                    is_parent_visible && self.is_visible(data),
                    parent_id,
                    is_last,
                ));
            }
            children
        };

        let attrs = {
            let mut attrs = self.attrs.to_owned();
            let oid = if let Some(oid) = attrs.get("data-id") {
                format!("{}:{}", oid, parent_id)
            } else {
                format!("{}:root", parent_id)
            };
            attrs.insert("data-id".to_string(), oid);
            attrs
        };

        return ftd::ftd2021::dnode::DNode {
            classes: self.classes.to_owned(),
            node: self.node.to_owned(),
            attrs,
            style,
            children,
            text: self.text.to_owned(),
            null: self.null.to_owned(),
            events: self.events.to_owned(),
            visible: self.is_visible(data),
        };

        fn is_other_sibling_visible(
            index: usize,
            all_children: &[Node],
            ext_child_container_index: usize,
            external_children_container: &[Vec<usize>],
        ) -> bool {
            for external_child_container in external_children_container
                .iter()
                .skip(ext_child_container_index)
            {
                if let Some(container) = external_child_container.get(0) {
                    if container < &index {
                        continue;
                    }
                    if let Some(child) = all_children.get(*container) {
                        if !child.node.is_empty() {
                            return true;
                        }
                    }
                }
            }
            false
        }
    }

    pub fn to_html(
        &self,
        style: &ftd::Map<String>,
        data: &ftd::DataDependenciesMap,
        id: &str,
    ) -> String {
        self.to_dnode(style, data, &mut None, &None, &[], true, id, false)
            .to_html(id)
    }

    pub fn get_target_node(&mut self, container: Vec<usize>) -> &mut Self {
        let mut current = self;
        for i in container.iter() {
            current = &mut current.children[*i];
        }
        current
    }
}

impl ftd::Element {
    pub fn to_node(&self, doc_id: &str, collector: &mut ftd::Collector) -> Node {
        match self {
            Self::Row(i) => i.to_node(doc_id, collector),
            Self::Scene(i) => i.to_node(doc_id, collector),
            Self::Grid(i) => i.to_node(doc_id, collector),
            Self::Markup(i) => i.to_node(doc_id, collector),
            Self::TextBlock(i) => i.to_node(doc_id, collector),
            Self::Code(i) => i.to_node(doc_id, collector),
            Self::Image(i) => i.to_node(doc_id, collector),
            Self::Column(i) => i.to_node(doc_id, true, collector),
            Self::IFrame(i) => i.to_node(doc_id, collector),
            Self::Input(i) => i.to_node(doc_id, collector),
            Self::Integer(i) => i.to_node(doc_id, collector),
            Self::Boolean(i) => i.to_node(doc_id, collector),
            Self::Decimal(i) => i.to_node(doc_id, collector),
            Self::Null => Node {
                condition: None,
                events: vec![],
                classes: vec![],
                node: "".to_string(),
                attrs: Default::default(),
                style: Default::default(),
                children: vec![],
                external_children: Default::default(),
                open_id: None,
                external_children_container: vec![],
                children_style: Default::default(),
                text: None,
                null: true,
            },
        }
    }

    // TODO: only when wasm feature is enabled
    pub fn to_dom(&self, _id: &str) {
        todo!()
    }
}

impl Node {
    fn from_common(
        node: &str,
        common: &ftd::Common,
        doc_id: &str,
        collector: &mut ftd::Collector,
    ) -> Self {
        let mut classes = common.add_class();
        Node {
            condition: common.condition.clone(),
            node: s(node),
            attrs: common.attrs(),
            style: common.style(doc_id, collector, &mut classes),
            children: vec![],
            external_children: Default::default(),
            open_id: None,
            external_children_container: vec![],
            children_style: common.children_style(),
            text: None,
            classes,
            null: common.is_dummy,
            events: common.events.clone(),
        }
    }

    fn from_container(
        common: &ftd::Common,
        container: &ftd::Container,
        doc_id: &str,
        collector: &mut ftd::Collector,
    ) -> Self {
        let mut attrs = common.attrs();
        attrs.extend(container.attrs());
        let mut classes = common.add_class();
        classes.extend(container.add_class());
        let mut style = common.style(doc_id, collector, &mut classes);
        style.extend(container.style());

        let mut children_style = common.children_style();
        children_style.extend(container.children_style());
        let node = common.node();

        let (id, external_children_container, external_children) = {
            if let Some((id, external_children_container, child)) = &container.external_children {
                (
                    Some(id.to_string()),
                    external_children_container.clone(),
                    child.iter().map(|v| v.to_node(doc_id, collector)).collect(),
                )
            } else {
                (None, vec![], vec![])
            }
        };

        Node {
            condition: common.condition.clone(),
            node: s(node.as_str()), // TODO: use better tags based on common.region
            attrs,
            style,
            classes,
            children_style,
            text: None,
            children: Default::default(),
            external_children,
            open_id: id,
            external_children_container,
            null: common.is_dummy,
            events: common.events.clone(),
        }
    }
}

impl ftd::Scene {
    pub fn to_node(&self, doc_id: &str, collector: &mut ftd::Collector) -> Node {
        let node = {
            let mut node = Node {
                node: s("div"),
                ..Default::default()
            };
            if let Some(ref data_id) = self.common.data_id {
                node.attrs
                    .insert(s("data-id"), format!("{}:scene", data_id));
            } else {
                node.attrs.insert(s("data-id"), s("scene:root"));
            }
            node.style.insert(s("position"), s("relative"));
            let children = {
                let parent = {
                    let mut node = if let Some(ref img) = self.common.background_image {
                        let mut n = Node {
                            node: s("img"),
                            ..Default::default()
                        };
                        n.attrs.insert(s("src"), img.light.to_string());
                        n.attrs.insert(s("alt"), ftd::html::escape("Scene"));
                        n
                    } else {
                        Node {
                            node: s("div"),
                            ..Default::default()
                        }
                    };
                    node.style.insert(s("width"), s("100%"));
                    if !self.common.is_not_visible {
                        node.style.insert(s("display"), s("block"));
                    }
                    if let Some(p) = &self.common.height {
                        let (key, value) = length(p, "height");
                        node.style.insert(s(key.as_str()), value);
                    } else {
                        node.style.insert(s("height"), s("auto"));
                    }
                    if let Some(ref data_id) = self.common.data_id {
                        node.attrs
                            .insert(s("data-id"), format!("{}:scene-bg", data_id));
                    }
                    node
                };
                let mut children: Vec<Node> = self
                    .container
                    .children
                    .iter()
                    .map(|v| {
                        let mut n = v.to_node(doc_id, collector);
                        n.style.insert(s("position"), s("absolute"));
                        n
                    })
                    .collect();
                children.insert(0, parent);
                children
            };

            let (id, external_children_container, external_children) = {
                if let Some((id, external_children_container, child)) =
                    &self.container.external_children
                {
                    (
                        Some(id.to_string()),
                        external_children_container.clone(),
                        child
                            .iter()
                            .map(|v| {
                                let mut n = v.to_node(doc_id, collector);
                                n.style.insert(s("position"), s("absolute"));
                                n
                            })
                            .collect(),
                    )
                } else {
                    (None, vec![], vec![])
                }
            };

            node.children = children;
            node.open_id = id;
            node.external_children = external_children;
            node.external_children_container = external_children_container;
            node
        };

        let mut main_node = Node::from_common("div", &self.common, doc_id, collector);
        if self.common.width.is_none() {
            main_node.style.insert(s("width"), s("1000px"));
        }
        if let Some(ref p) = self.spacing {
            let (key, value) = spacing(p, "margin-left");
            match p {
                ftd::Spacing::Absolute { value } => {
                    main_node.children_style.insert(key, format!("{}px", value));
                    main_node
                        .attrs
                        .insert(s("data-spacing"), format!("margin-left:{}px", value))
                }
                _ => main_node.style.insert(key, value),
            };
        }
        main_node.children = vec![node];
        main_node
    }
}

impl ftd::Grid {
    pub fn to_node(&self, doc_id: &str, collector: &mut ftd::Collector) -> Node {
        let mut n = Node::from_container(&self.common, &self.container, doc_id, collector);
        if self.inline {
            n.style.insert(s("display"), s("inline-grid"));
        } else {
            n.style.insert(s("display"), s("grid"));
        }
        let areas = self
            .slots
            .split('|')
            .map(|v| v.trim())
            .collect::<Vec<&str>>();
        let mut css_areas = s("");
        for area in areas {
            css_areas = format!("{}'{}'", css_areas, area);
        }
        n.style.insert(s("grid-template-areas"), css_areas);

        if let Some(ref columns) = self.slot_widths {
            n.style
                .insert(s("grid-template-columns"), s(columns.trim()));
        }
        if let Some(ref rows) = self.slot_heights {
            n.style.insert(s("grid-template-rows"), s(rows.trim()));
        }
        if let Some(ref gap) = self.spacing {
            n.style.insert(s("grid-gap"), format!("{}px", gap));
        }
        if let Some(ref gap) = self.spacing_vertical {
            n.style.insert(s("column-gap"), format!("{}px", gap));
        }
        if let Some(ref gap) = self.spacing_horizontal {
            n.style.insert(s("row-gap"), format!("{}px", gap));
        }
        if let Some(ref auto_flow) = self.auto_flow {
            n.style.insert(s("grid-auto-flow"), s(auto_flow));
        }

        n.children = {
            let mut children = vec![];
            for child in self.container.children.iter() {
                let mut child_node = child.to_node(doc_id, collector);
                let common = if let Some(common) = child.get_common() {
                    common
                } else {
                    children.push(child_node);
                    continue;
                };
                if common.anchor.is_some() {
                    children.push(child_node);
                    continue;
                }
                if let Some(ref position) = common.position {
                    for (key, value) in grid_align(position) {
                        child_node.style.insert(s(key.as_str()), value);
                    }
                }
                children.push(child_node);
            }
            children
        };

        n
    }
}

impl ftd::Row {
    pub fn to_node(&self, doc_id: &str, collector: &mut ftd::Collector) -> Node {
        let mut n = Node::from_container(&self.common, &self.container, doc_id, collector);
        if !self.common.is_not_visible {
            n.style.insert(s("display"), s("flex"));
        }
        n.style.insert(s("flex-direction"), s("row"));
        if self.container.wrap {
            n.style.insert(s("flex-wrap"), s("wrap"));
        } else {
            n.style.insert(s("flex-wrap"), s("nowrap"));
        }

        n.style.insert(s("align-items"), s("flex-start"));

        n.style.insert(s("justify-content"), s("flex-start"));

        if let Some(ref p) = self.spacing {
            let (key, value) = spacing(p, "margin-left");
            match p {
                ftd::Spacing::Absolute { value } => {
                    n.children_style.insert(key, format!("{}px", value));
                    n.attrs
                        .insert(s("data-spacing"), format!("margin-left:{}px", value))
                }
                _ => n.style.insert(key, value),
            };
        }

        n.children = {
            let mut children = vec![];
            for child in self.container.children.iter() {
                let mut child_node = child.to_node(doc_id, collector);
                let common = if let Some(common) = child.get_common() {
                    common
                } else {
                    children.push(child_node);
                    continue;
                };
                if common.anchor.is_some() {
                    children.push(child_node);
                    continue;
                }
                if let Some(ref position) = common.position {
                    for (key, value) in row_align(position) {
                        child_node.style.insert(s(key.as_str()), value);
                    }
                }
                children.push(child_node);
            }
            children
        };

        n
    }
}

impl ftd::Column {
    pub fn to_node(
        &self,
        doc_id: &str,
        evaluate_children: bool,
        collector: &mut ftd::Collector,
    ) -> Node {
        let mut n = Node::from_container(&self.common, &self.container, doc_id, collector);
        if !self.common.is_not_visible {
            n.style.insert(s("display"), s("flex"));
        }
        n.style.insert(s("flex-direction"), s("column"));
        if self.container.wrap {
            n.style.insert(s("flex-wrap"), s("wrap"));
        } else {
            n.style.insert(s("flex-wrap"), s("nowrap"));
        }
        n.style.insert(s("align-items"), s("flex-start"));

        n.style.insert(s("justify-content"), s("flex-start"));

        if let Some(ref p) = self.spacing {
            let (key, value) = spacing(p, "margin-top");
            match p {
                ftd::Spacing::Absolute { value } => {
                    n.children_style.insert(key, format!("{}px", value));
                    n.attrs
                        .insert(s("data-spacing"), format!("margin-top:{}px", value))
                }
                _ => n.style.insert(key, value),
            };
        }

        if evaluate_children {
            n.children = {
                let mut children = vec![];
                for child in self.container.children.iter() {
                    let mut child_node = child.to_node(doc_id, collector);
                    let common = if let Some(common) = child.get_common() {
                        common
                    } else {
                        children.push(child_node);
                        continue;
                    };
                    if common.anchor.is_some() {
                        children.push(child_node);
                        continue;
                    }
                    if let Some(ref position) = common.position {
                        for (key, value) in column_align(position) {
                            child_node.style.insert(s(key.as_str()), value);
                        }
                    }
                    children.push(child_node);
                }
                children
            };
        }

        n
    }
}

/// One instance of Collector is created during entire page render. It collects
/// all the classes needed, and all the fonts needed to render the page. At the
/// end of Node generation, values collected in this struct is included in
/// generated HTML.
#[derive(Debug, PartialEq, Clone, Default)]
pub struct Collector {
    /// this stores all the classes in the document
    pub classes: ftd::Map<StyleSpec>,
    pub key: i32,
}

#[derive(Debug, PartialEq, Clone, Default)]
pub struct StyleSpec {
    pub prefix: Option<String>,
    pub styles: ftd::Map<String>,
}

impl ftd::Collector {
    pub(crate) fn new() -> ftd::Collector {
        ftd::Collector {
            classes: Default::default(),
            key: -1,
        }
    }

    fn get_classes(&mut self, styles: ftd::Map<String>) -> Vec<String> {
        self.classes
            .iter()
            .filter(|(_, values)| values.styles.eq(&styles))
            .map(|(k, _)| k.to_string())
            .collect()
    }

    fn insert_class_font(&mut self, font: &ftd::Type) -> String {
        let mut styles: ftd::Map<String> = Default::default();
        styles.insert(s("font-family"), font.font.to_string());
        styles.insert(s("line-height"), format!("{}px", font.desktop.line_height));
        styles.insert(
            s("letter-spacing"),
            format!("{}px", font.desktop.letter_spacing),
        );
        styles.insert(s("font-size"), format!("{}px", font.desktop.size));
        styles.insert(s("font-weight"), font.weight.to_string());
        if font.style.italic {
            styles.insert(s("font-style"), s("italic"));
        }
        if font.style.underline {
            styles.insert(s("text-decoration"), s("underline"));
        }
        if font.style.strike {
            styles.insert(s("text-decoration"), s("line-through"));
        }

        if let Some(ref weight) = font.style.weight {
            let (key, value) = style(weight);
            styles.insert(s(key.as_str()), value);
        }
        // if self.common.conditional_attribute.keys().any(|x| styles.keys().contains(&x)) {
        //     // todo: then don't make class
        //     // since font is not a conditional attribute this is not yet needed
        // }
        let desktop_style = styles.clone();

        styles.insert(s("line-height"), format!("{}px", font.mobile.line_height));
        styles.insert(
            s("letter-spacing"),
            format!("{}px", font.mobile.letter_spacing),
        );
        styles.insert(s("font-size"), format!("{}px", font.mobile.size));
        let mobile_style = styles.clone();

        styles.insert(s("line-height"), format!("{}px", font.xl.line_height));
        styles.insert(s("letter-spacing"), format!("{}px", font.xl.letter_spacing));
        styles.insert(s("font-size"), format!("{}px", font.xl.size));
        let xl_style = styles;

        let classes = self.get_classes(desktop_style.clone());

        for class in classes {
            let mobile_class = format!("body.ftd-mobile .{}", class);
            let mobile_style_spec = if let Some(mobile_style_spec) = self.classes.get(&mobile_class)
            {
                mobile_style_spec
            } else {
                continue;
            };

            let xl_class = format!("body.ftd-xl .{}", class);
            let xl_style_spec = if let Some(xl_style_spec) = self.classes.get(&xl_class) {
                xl_style_spec
            } else {
                continue;
            };

            if mobile_style_spec.styles.eq(&mobile_style) && xl_style_spec.styles.eq(&xl_style) {
                return class;
            }
        }
        let class = self.insert_class(desktop_style, None);
        self.insert_class(mobile_style, Some(format!("body.ftd-mobile .{}", class)));
        self.insert_class(xl_style, Some(format!("body.ftd-xl .{}", class)));
        class
    }

    fn insert_class_color(&mut self, col: &ftd::Color, key: &str) -> String {
        let mut styles: ftd::Map<String> = Default::default();
        styles.insert(s(key), color(&col.light));
        let light_style = styles.clone();

        styles.insert(s(key), color(&col.dark));
        let dark_style = styles;

        let classes = self.get_classes(light_style.clone());

        for class in classes {
            let dark_class = format!("body.fpm-dark .{}", class);
            let dark_style_spec = if let Some(dark_style_spec) = self.classes.get(&dark_class) {
                dark_style_spec
            } else {
                continue;
            };

            if dark_style_spec.styles.eq(&dark_style) {
                return class;
            }
        }
        let class = self.insert_class(light_style, None);
        self.insert_class(dark_style, Some(format!("body.fpm-dark .{}", class)));
        class
    }

    fn insert_class(&mut self, styles: ftd::Map<String>, prefix: Option<String>) -> String {
        if let Some(ref prefix) = prefix {
            if self.classes.contains_key(prefix) {
                return prefix.to_owned();
            }
            self.classes.insert(
                prefix.to_string(),
                ftd::StyleSpec {
                    prefix: Some(prefix.to_string()),
                    styles,
                },
            );
            return prefix.to_string();
        }
        self.key += 1;
        let class_name = get_full_class_name(&self.key, &styles);
        self.classes
            .insert(class_name.to_string(), ftd::StyleSpec { prefix, styles });
        return class_name;

        fn get_full_class_name(key: &i32, styles: &ftd::Map<String>) -> String {
            let styles = styles
                .keys()
                .filter_map(|v| v.get(0..1))
                .collect::<Vec<&str>>()
                .join("");
            format!("{}_{}", styles, key)
        }
    }

    pub(crate) fn to_css(&self) -> String {
        let mut styles = "".to_string();
        for (k, v) in self.classes.iter() {
            let current_styles = v
                .styles
                .iter()
                .map(|(k, v)| format!("{}: {}", *k, ftd::html::escape(v))) // TODO: escape needed?
                .collect::<Vec<String>>()
                .join(";\n");
            if let Some(ref prefix) = v.prefix {
                styles = format!(
                    indoc::indoc! {"
                        {styles}
    
                        {prefix} {{
                            {current_styles}
                        }}
    
                    "},
                    styles = styles,
                    prefix = prefix,
                    current_styles = current_styles
                );
                continue;
            }
            styles = format!(
                indoc::indoc! {"
                    {styles}

                    .{class_name} {{
                        {current_styles}
                    }}

                "},
                styles = styles,
                class_name = k,
                current_styles = current_styles
            );
        }
        styles
    }
}

impl ftd::Text {
    pub fn to_node(&self, doc_id: &str, collector: &mut ftd::Collector) -> Node {
        // TODO: proper tag based on self.common.region
        // TODO: if format is not markup use pre
        let node = self.common.node();
        let mut n = Node::from_common(node.as_str(), &self.common, doc_id, collector);
        n.classes.push("ft_md".to_string());
        n.text = Some(self.text.rendered.clone());
        let (key, value) = text_align(&self.text_align);
        n.style.insert(s(key.as_str()), value);
        if !self.line && self.font.is_none() {
            n.style.insert(s("line-height"), s("26px"));
        }

        if let Some(ref font) = self.font {
            n.classes.push(collector.insert_class_font(font));
        }

        if self.style.italic {
            n.style.insert(s("font-style"), s("italic"));
        }
        if self.style.underline {
            n.style.insert(s("text-decoration"), s("underline"));
        }
        if self.style.strike {
            n.style.insert(s("text-decoration"), s("line-through"));
        }

        if let Some(p) = &self.line_clamp {
            n.style.insert(s("display"), "-webkit-box".to_string());
            n.style.insert(s("overflow"), "hidden".to_string());
            n.style.insert(s("-webkit-line-clamp"), format!("{}", p));
            n.style
                .insert(s("-webkit-box-orient"), "vertical".to_string());
        }

        if let Some(ref weight) = self.style.weight {
            let (key, value) = style(weight);
            n.style.insert(s(key.as_str()), value);
        }

        // TODO: text styles
        n
    }
}

impl ftd::TextBlock {
    pub fn to_node(&self, doc_id: &str, collector: &mut ftd::Collector) -> Node {
        // TODO: proper tag based on self.common.region
        // TODO: if format is not markup use pre
        let node = self.common.node();
        let mut n = Node::from_common(node.as_str(), &self.common, doc_id, collector);
        n.text = Some(self.text.rendered.clone());
        let (key, value) = text_align(&self.text_align);
        n.style.insert(s(key.as_str()), value);
        if let Some(p) = self.size {
            n.style.insert(s("font-size"), format!("{}px", p));
        }
        if let Some(p) = self.line_height {
            n.style.insert(s("line-height"), format!("{}px", p));
        } else if !&self.line {
            n.style.insert(s("line-height"), s("26px"));
        }

        if !self.font.is_empty() {
            let family = self
                .font
                .iter()
                .map(|v| v.to_string())
                .collect::<Vec<String>>()
                .join(", ");
            n.style.insert(s("font-family"), family);
        }

        if self.style.italic {
            n.style.insert(s("font-style"), s("italic"));
        }
        if self.style.underline {
            n.style.insert(s("text-decoration"), s("underline"));
        }
        if self.style.strike {
            n.style.insert(s("text-decoration"), s("line-through"));
        }

        if let Some(p) = &self.line_clamp {
            n.style.insert(s("display"), "-webkit-box".to_string());
            n.style.insert(s("overflow"), "hidden".to_string());
            n.style.insert(s("-webkit-line-clamp"), format!("{}", p));
            n.style
                .insert(s("-webkit-box-orient"), "vertical".to_string());
        }
        if let Some(indent) = &self.text_indent {
            let (key, value) = length(indent, "text-indent");
            n.style.insert(s(key.as_str()), value);
        }

        if let Some(ref weight) = self.style.weight {
            let (key, value) = style(weight);
            n.style.insert(s(key.as_str()), value);
        }

        n
    }
}

impl ftd::Code {
    pub fn to_node(&self, doc_id: &str, collector: &mut ftd::Collector) -> Node {
        let node = self.common.node();
        let mut n = Node::from_common(node.as_str(), &self.common, doc_id, collector);
        n.text = Some(self.text.rendered.clone());
        let (key, value) = text_align(&self.text_align);
        n.style.insert(s(key.as_str()), value);

        if self.font.is_none() {
            n.style.insert(s("line-height"), s("26px"));
        }

        if let Some(ref font) = self.font {
            n.classes.push(collector.insert_class_font(font));
        }

        if self.style.italic {
            n.style.insert(s("font-style"), s("italic"));
        }
        if self.style.underline {
            n.style.insert(s("text-decoration"), s("underline"));
        }
        if self.style.strike {
            n.style.insert(s("text-decoration"), s("line-through"));
        }

        if let Some(p) = &self.line_clamp {
            n.style.insert(s("display"), "-webkit-box".to_string());
            n.style.insert(s("overflow"), "hidden".to_string());
            n.style.insert(s("-webkit-line-clamp"), format!("{}", p));
            n.style
                .insert(s("-webkit-box-orient"), "vertical".to_string());
        }

        if let Some(p) = &self.text_indent {
            let (key, value) = length(p, "text-indent");
            n.style.insert(s(key.as_str()), value);
        }

        if let Some(ref weight) = self.style.weight {
            let (key, value) = style(weight);
            n.style.insert(s(key.as_str()), value);
        }

        n
    }
}

impl ftd::Image {
    pub fn to_node(&self, doc_id: &str, collector: &mut ftd::Collector) -> Node {
        return match self.common.link {
            Some(_) => {
                let mut n = Node::from_common("a", &self.common, doc_id, collector);
                if let Some(ref id) = self.common.data_id {
                    n.attrs.insert(
                        s("data-id"),
                        ftd::html::escape(format!("{}:parent", id).as_str()),
                    );
                }
                let mut img = update_img(
                    self,
                    Node {
                        node: s("img"),
                        ..Default::default()
                    },
                );
                img.style.insert(s("width"), s("100%"));
                img.style.insert(s("height"), s("100%"));
                n.children.push(img);
                n
            }
            None => update_img(
                self,
                Node::from_common("img", &self.common, doc_id, collector),
            ),
        };

        fn update_img(img: &ftd::Image, mut n: ftd::Node) -> ftd::Node {
            n.attrs.insert(s("loading"), s(img.loading.to_html()));
            if let Some(ref id) = img.common.data_id {
                n.attrs.insert(s("data-id"), ftd::html::escape(id));
            }
            n.attrs
                .insert(s("src"), ftd::html::escape(img.src.light.as_str()));
            if let Some(ref description) = img.description {
                n.attrs.insert(s("alt"), ftd::html::escape(description));
            }

            if img.crop {
                n.style.insert(s("object-fit"), s("cover"));
                n.style.insert(s("object-position"), s("0 0"));
                if img.common.width.is_none() {
                    n.style.insert(s("width"), s("100%"));
                }
            }

            n
        }
    }
}

impl ftd::IFrame {
    pub fn to_node(&self, doc_id: &str, collector: &mut ftd::Collector) -> Node {
        let mut n = Node::from_common("iframe", &self.common, doc_id, collector);
        n.attrs
            .insert(s("src"), ftd::html::escape(self.src.as_str()));
        n.attrs.insert(s("allow"), s("fullscreen"));
        n.attrs.insert(s("allowfullscreen"), s("allowfullscreen"));
        n
    }
}

impl ftd::Markups {
    pub fn to_node(&self, doc_id: &str, collector: &mut ftd::Collector) -> Node {
        let node = self.common.node();
        let mut n = Node::from_common(node.as_str(), &self.common, doc_id, collector);
        n.classes.push("ft_md".to_string());
        let (key, value) = text_align(&self.text_align);
        n.style.insert(s(key.as_str()), value);

        if !self.line && self.font.is_none() {
            n.style.insert(s("line-height"), s("26px"));
        }

        if let Some(ref font) = self.font {
            n.classes.push(collector.insert_class_font(font));
        }

        if self.style.italic {
            n.style.insert(s("font-style"), s("italic"));
        }
        if self.style.underline {
            n.style.insert(s("text-decoration"), s("underline"));
        }
        if self.style.strike {
            n.style.insert(s("text-decoration"), s("line-through"));
        }

        if let Some(p) = &self.line_clamp {
            n.style.insert(s("display"), "-webkit-box".to_string());
            n.style.insert(s("overflow"), "hidden".to_string());
            n.style.insert(s("-webkit-line-clamp"), format!("{}", p));
            n.style
                .insert(s("-webkit-box-orient"), "vertical".to_string());
        }

        if let Some(p) = &self.text_indent {
            let (key, value) = length(p, "text-indent");
            n.style.insert(s(key.as_str()), value);
        }

        if self.children.is_empty() {
            n.text = Some(self.text.rendered.clone());
        }

        if let Some(ref weight) = self.style.weight {
            let (key, value) = style(weight);
            n.style.insert(s(key.as_str()), value);
        }

        n.children = self
            .children
            .iter()
            .map(|v| v.to_node(doc_id, !self.line, collector))
            .collect();
        n
    }
}

impl ftd::Markup {
    pub fn to_node(
        &self,
        doc_id: &str,
        is_paragraph: bool,
        collector: &mut ftd::Collector,
    ) -> Node {
        let mut n = match self.itext {
            ftd::IText::Text(ref t)
            | ftd::IText::Integer(ref t)
            | ftd::IText::Boolean(ref t)
            | ftd::IText::Decimal(ref t) => t.to_node(doc_id, collector),
            ftd::IText::TextBlock(ref t) => t.to_node(doc_id, collector),
            IText::Markup(ref t) => t.to_node(doc_id, collector),
        };
        if n.node.eq("div") {
            if is_paragraph {
                n.node = s("p");
            } else {
                n.node = s("span");
            }
        }
        if self.children.is_empty() {
            return n;
        } else {
            n.text = None;
        }
        n.children = self
            .children
            .iter()
            .map(|v| v.to_node(doc_id, false, collector))
            .collect();
        n
    }
}

impl ftd::Input {
    pub fn to_node(&self, doc_id: &str, collector: &mut ftd::Collector) -> Node {
        let node = if self.multiline { "textarea" } else { "input" };

        let mut n = Node::from_common(node, &self.common, doc_id, collector);

        if let Some(ref font) = self.font {
            n.classes.push(collector.insert_class_font(font));
        }

        if let Some(ref p) = self.placeholder {
            n.attrs.insert(s("placeholder"), ftd::html::escape(p));
        }
        if let Some(ref type_) = self.type_ {
            n.attrs.insert(s("type"), ftd::html::escape(type_));
        }
        if let Some(ref p) = self.value {
            if self.multiline {
                n.text = Some(p.to_string());
            } else {
                n.attrs.insert(s("value"), ftd::html::escape(p));
            }
        }
        // add defaultValue attribute if passed
        if let Some(ref def_value) = self.default_value {
            n.attrs.insert(s("data-dv"), ftd::html::escape(def_value));
        }
        n
    }
}

impl ftd::Common {
    fn node(&self) -> String {
        match &self.link {
            Some(_) => "a",
            None => match &self.submit {
                Some(_) => "form",
                _ => match self.region.as_ref() {
                    Some(ftd::Region::H0) => "h1",
                    Some(ftd::Region::H1) => "h2",
                    Some(ftd::Region::H2) => "h3",
                    Some(ftd::Region::H3) => "h4",
                    Some(ftd::Region::H4) => "h5",
                    Some(ftd::Region::H5) => "h6",
                    Some(ftd::Region::H6) => "h7",
                    _ => "div",
                },
            },
        }
        .to_string()
    }

    fn children_style(&self) -> ftd::Map<String> {
        Default::default()
    }

    fn add_class(&self) -> Vec<String> {
        self.classes
            .clone()
            .unwrap_or_default()
            .split(',')
            .map(|v| v.trim().to_string())
            .collect()
    }

    fn style(
        &self,
        doc_id: &str,
        collector: &mut ftd::Collector,
        classes: &mut Vec<String>,
    ) -> ftd::Map<String> {
        let mut d: ftd::Map<String> = Default::default();

        d.insert(s("text-decoration"), s("none"));
        if !self.events.is_empty() && self.cursor.is_none() {
            d.insert(s("cursor"), s("pointer"));
        }
        if self.is_not_visible {
            d.insert(s("display"), s("none"));
        }

        if let Some(p) = self.padding {
            d.insert(s("padding"), format!("{}px", p));
        }
        if let Some(p) = self.padding_left {
            d.insert(s("padding-left"), format!("{}px", p));
        }
        if let Some(ref cursor) = self.cursor {
            d.insert(s("cursor"), s(cursor));
        }
        if let Some(p) = self.padding_vertical {
            d.insert(s("padding-top"), format!("{}px", p));
            d.insert(s("padding-bottom"), format!("{}px", p));
        }
        if let Some(p) = self.padding_horizontal {
            d.insert(s("padding-left"), format!("{}px", p));
            d.insert(s("padding-right"), format!("{}px", p));
        }
        if let Some(p) = self.padding_right {
            d.insert(s("padding-right"), format!("{}px", p));
        }
        if let Some(p) = self.padding_top {
            d.insert(s("padding-top"), format!("{}px", p));
        }
        if let Some(p) = self.padding_bottom {
            d.insert(s("padding-bottom"), format!("{}px", p));
        }

        if let Some(p) = self.border_top_radius {
            d.insert(s("border-top-left-radius"), format!("{}px !important", p));
            d.insert(s("border-top-right-radius"), format!("{}px !important", p));
        }

        if let Some(p) = self.border_left_radius {
            d.insert(s("border-top-left-radius"), format!("{}px !important", p));
            d.insert(
                s("border-bottom-left-radius"),
                format!("{}px !important", p),
            );
        }

        if let Some(p) = self.border_right_radius {
            d.insert(s("border-top-right-radius"), format!("{}px !important", p));
            d.insert(
                s("border-bottom-right-radius"),
                format!("{}px !important", p),
            );
        }

        if let Some(p) = self.border_bottom_radius {
            d.insert(
                s("border-bottom-right-radius"),
                format!("{}px !important", p),
            );
            d.insert(
                s("border-bottom-left-radius"),
                format!("{}px !important", p),
            );
        }

        if let Some(p) = &self.width {
            let (key, value) = length(p, "width");
            d.insert(s(key.as_str()), value);
        }
        if let Some(p) = &self.min_width {
            let (key, value) = length(p, "min-width");
            d.insert(s(key.as_str()), value);
        }
        if let Some(p) = &self.max_width {
            let (key, value) = length(p, "max-width");
            d.insert(s(key.as_str()), value);
        }
        if let Some(p) = &self.height {
            let (key, value) = length(p, "height");
            d.insert(s(key.as_str()), value);
        }
        if let Some(p) = &self.min_height {
            let (key, value) = length(p, "min-height");
            d.insert(s(key.as_str()), value);
        }
        if let Some(p) = &self.max_height {
            let (key, value) = length(p, "max-height");
            d.insert(s(key.as_str()), value);
        }
        if let Some(p) = self.border_left {
            d.insert(s("border-left-width"), format!("{}px !important", p));
        }
        if let Some(p) = self.border_right {
            d.insert(s("border-right-width"), format!("{}px !important", p));
        }
        if let Some(p) = self.border_top {
            d.insert(s("border-top-width"), format!("{}px !important", p));
        }
        if let Some(p) = self.border_bottom {
            d.insert(s("border-bottom-width"), format!("{}px !important", p));
        }
        if let Some(p) = self.margin_left {
            d.insert(s("margin-left"), format!("{}px", p));
        }
        if let Some(p) = self.margin_right {
            d.insert(s("margin-right"), format!("{}px", p));
        }
        if let Some(p) = self.margin_top {
            d.insert(s("margin-top"), format!("{}px", p));
        }
        if let Some(p) = self.margin_bottom {
            d.insert(s("margin-bottom"), format!("{}px", p));
        }
        if let Some(p) = &self.background_color {
            if self.conditional_attribute.contains_key("background-color") {
                d.insert(s("background-color"), color(&p.light));
            } else {
                classes.push(collector.insert_class_color(p, "background-color"));
            }
        }
        if let Some(p) = &self.border_top_color {
            if self.conditional_attribute.contains_key("border-top-color") {
                d.insert(s("border-top-color"), color(&p.light));
            } else {
                classes.push(collector.insert_class_color(p, "border-top-color"));
            }
        }
        if let Some(p) = &self.border_bottom_color {
            if self
                .conditional_attribute
                .contains_key("border-bottom-color")
            {
                d.insert(s("border-bottom-color"), color(&p.light));
            } else {
                classes.push(collector.insert_class_color(p, "border-bottom-color"));
            }
        }
        if let Some(p) = &self.border_right_color {
            if self
                .conditional_attribute
                .contains_key("border-right-color")
            {
                d.insert(s("border-right-color"), color(&p.light));
            } else {
                classes.push(collector.insert_class_color(p, "border-right-color"));
            }
        }
        if let Some(p) = &self.border_left_color {
            if self.conditional_attribute.contains_key("border-left-color") {
                d.insert(s("border-left-color"), color(&p.light));
            } else {
                classes.push(collector.insert_class_color(p, "border-left-color"));
            }
        }
        if let Some(p) = &self.color {
            if self.conditional_attribute.contains_key("color") {
                d.insert(s("color"), color(&p.light));
            } else {
                classes.push(collector.insert_class_color(p, "color"));
            }
        }
        if let Some(p) = &self.border_color {
            if self.conditional_attribute.contains_key("border-color") {
                d.insert(s("border-color"), color(&p.light));
            } else {
                classes.push(collector.insert_class_color(p, "border-color"));
            }
        }
        if let Some(p) = &self.overflow_x {
            let (key, value) = overflow(p, "overflow-x");
            d.insert(s(key.as_str()), value);
        }
        if let Some(p) = &self.overflow_y {
            let (key, value) = overflow(p, "overflow-y");
            d.insert(s(key.as_str()), value);
        }
        if self.sticky {
            d.insert(s("position"), s("sticky"));
        }
        if let Some(p) = &self.top {
            d.insert(s("top"), format!("{}px", p));
        }
        if let Some(p) = &self.bottom {
            d.insert(s("bottom"), format!("{}px", p));
        }
        if let Some(p) = &self.left {
            d.insert(s("left"), format!("{}px", p));
        }
        if let Some(p) = &self.right {
            d.insert(s("right"), format!("{}px", p));
        }
        if self.submit.is_some() {
            d.insert(s("cursor"), s("pointer"));
        }
        if self.link.is_some() {
            d.insert(s("cursor"), s("pointer"));
        }
        if let Some(p) = &self.z_index {
            d.insert(s("z-index"), p.to_string());
        }
        if let Some(p) = &self.slot {
            d.insert(s("grid-area"), s(p));
        }
        if let Some(p) = &self.grid_column {
            d.insert(s("grid-column"), s(p));
        }
        if let Some(p) = &self.grid_row {
            d.insert(s("grid-row"), s(p));
        }
        if let Some(p) = &self.text_transform {
            d.insert(s("text-transform"), s(p));
        }
        if self.shadow_size.is_some()
            || self.shadow_blur.is_some()
            || self.shadow_offset_x.is_some()
            || self.shadow_offset_y.is_some()
        {
            let shadow_color = match &self.shadow_color {
                Some(p) => &p.light,
                None => &ftd::ColorValue {
                    r: 0,
                    g: 0,
                    b: 0,
                    alpha: 0.25,
                },
            };

            d.insert(
                s("box-shadow"),
                format!(
                    "{}px {}px {}px {}px {}",
                    self.shadow_offset_x.unwrap_or(0),
                    self.shadow_offset_y.unwrap_or(0),
                    self.shadow_blur.unwrap_or(0),
                    self.shadow_size.unwrap_or(0),
                    color(shadow_color),
                ),
            );
        }
        if let Some(p) = &self.anchor {
            d.insert(s("position"), p.to_position());
        }
        if let Some(p) = &self.gradient_direction {
            d.insert(s("background-image"), gradient(p, &self.gradient_colors));
        }
        if let Some(p) = &self.background_image {
            d.insert(s("background-image"), format!("url({})", p.light));
            if self.background_repeat {
                d.insert(s("background-repeat"), s("repeat"));
            } else {
                d.insert(s("background-size"), s("cover"));
                d.insert(s("background-position"), s("center"));
            }
            if self.background_parallax {
                d.insert(s("background-attachment"), s("fixed"));
            }
        }

        match &self.anchor {
            Some(_)
                if !((self.left.is_some() || self.right.is_some())
                    && (self.top.is_some() || self.bottom.is_some())) =>
            {
                let position = if let Some(ref position) = self.position {
                    position
                } else {
                    &ftd::Position::TopLeft
                };
                for (key, value) in non_static_container_align(position, self.inner) {
                    d.insert(s(key.as_str()), value);
                }
            }
            _ => {}
        }

        let translate = get_translate(
            &self.move_left,
            &self.move_right,
            &self.move_up,
            &self.move_down,
            &self.scale,
            &self.scale_x,
            &self.scale_y,
            &self.rotate,
            doc_id,
        )
        .unwrap();

        if let Some(p) = translate {
            let data = if let Some(d) = d.get_mut("transform") {
                format!("{} {}", d, p)
            } else {
                p
            };
            d.insert(s("transform"), data);
        }

        if let Some(p) = &self.border_style {
            d.insert(s("border-style"), s(p));
        } else {
            d.insert(s("border-style"), s("solid"));
        }
        d.insert(s("border-width"), format!("{}px", self.border_width));
        d.insert(s("border-radius"), format!("{}px", self.border_radius));
        d.insert(s("box-sizing"), s("border-box"));

        if let Some(ref p) = self.white_space {
            d.insert(s("white-space"), s(p));
        } else {
            d.insert(s("white-space"), s("initial"));
        }

        d
    }

    fn attrs(&self) -> ftd::Map<String> {
        let mut d: ftd::Map<String> = Default::default();
        if let Some(ref id) = self.data_id {
            d.insert(s("data-id"), ftd::html::escape(id));
        }
        if let Some(ref id) = self.id {
            d.insert(s("id"), ftd::html::escape(id));
        }
        // TODO(move-to-ftd): the link should be escaped
        if let Some(ref link) = self.link {
            d.insert(s("href"), link.to_string());
        }
        if let Some(ref title) = self.title {
            d.insert(s("title"), ftd::html::escape(title));
        }
        if self.open_in_new_tab {
            d.insert(s("target"), ftd::html::escape("_blank"));
        }
        if self.submit.is_some() {
            d.insert(s("onclick"), "this.submit()".to_string());
        }
        d
    }
}
impl ftd::Container {
    fn style(&self) -> ftd::Map<String> {
        let mut d: ftd::Map<String> = Default::default();
        let mut count = count_children_with_absolute_parent(&self.children);
        if let Some((_, _, ref ext_children)) = self.external_children {
            count += count_children_with_absolute_parent(ext_children);
        }
        if count != 0 {
            d.insert(s("position"), s("relative"));
        }
        return d;

        fn count_children_with_absolute_parent(children: &[ftd::Element]) -> usize {
            children
                .iter()
                .filter(|v| {
                    let mut bool = false;
                    if let Some(common) = v.get_common() {
                        if Some(ftd::Anchor::Parent) == common.anchor {
                            bool = true;
                        }
                    }
                    bool
                })
                .count()
        }
    }
    fn children_style(&self) -> ftd::Map<String> {
        let d: ftd::Map<String> = Default::default();
        d
    }

    fn attrs(&self) -> ftd::Map<String> {
        let d: ftd::Map<String> = Default::default();
        d
    }
    fn add_class(&self) -> Vec<String> {
        let d: Vec<String> = Default::default();
        d
    }
}

fn s(s: &str) -> String {
    s.to_string()
}

pub fn color(c: &ftd::ColorValue) -> String {
    let ftd::ColorValue { r, g, b, alpha } = c;
    format!("rgba({},{},{},{})", r, g, b, alpha)
}

pub fn length(l: &ftd::Length, f: &str) -> (String, String) {
    let s = f.to_string();
    match l {
        ftd::Length::Fill => (s, "100%".to_string()),
        ftd::Length::Auto => (s, "auto".to_string()),
        ftd::Length::Px { value } => (s, format!("{}px", value)),
        ftd::Length::Portion { value } => ("flex-grow".to_string(), value.to_string()),
        ftd::Length::Percent { value } => (s, format!("{}%", value)),
        ftd::Length::FitContent => (s, "fit-content".to_string()),
        ftd::Length::Calc { value } => (s, format!("calc({})", value)),
        ftd::Length::VH { value } => (s, format!("{}vh", value)),
        ftd::Length::VW { value } => (s, format!("{}vw", value)),
        ftd::Length::VMIN { value } => (s, format!("{}vmin", value)),
        ftd::Length::VMAX { value } => (s, format!("{}vmax", value)),

        _ => (s, "100%".to_string()),
        //        ftd::Length::Shrink => (s, "width".to_string()),   TODO
    }
}

fn text_align(l: &ftd::TextAlign) -> (String, String) {
    match l {
        ftd::TextAlign::Center => ("text-align".to_string(), "center".to_string()),
        ftd::TextAlign::Left => ("text-align".to_string(), "left".to_string()),
        ftd::TextAlign::Right => ("text-align".to_string(), "right".to_string()),
        ftd::TextAlign::Justify => ("text-align".to_string(), "justify".to_string()),
    }
}

pub fn overflow(l: &ftd::Overflow, f: &str) -> (String, String) {
    let s = f.to_string();
    match l {
        ftd::Overflow::Auto => (s, "auto".to_string()),
        ftd::Overflow::Hidden => (s, "hidden".to_string()),
        ftd::Overflow::Scroll => (s, "scroll".to_string()),
        ftd::Overflow::Visible => (s, "visible".to_string()),
    }
}

fn gradient(d: &ftd::GradientDirection, c: &[ftd::ColorValue]) -> String {
    let color = c.iter().map(color).collect::<Vec<String>>().join(",");
    let gradient_style = match d {
        ftd::GradientDirection::BottomToTop => "linear-gradient(to top ".to_string(),
        ftd::GradientDirection::TopToBottom => "linear-gradient(to bottom ".to_string(),
        ftd::GradientDirection::LeftToRight => "linear-gradient(to right".to_string(),
        ftd::GradientDirection::RightToLeft => "linear-gradient(to left".to_string(),
        ftd::GradientDirection::BottomRightToTopLeft => "linear-gradient(to top left".to_string(),
        ftd::GradientDirection::TopLeftBottomRight => "linear-gradient(to bottom right".to_string(),
        ftd::GradientDirection::BottomLeftToTopRight => "linear-gradient(to top right".to_string(),
        ftd::GradientDirection::TopRightToBottomLeft => {
            "linear-gradient(to bottom left".to_string()
        }
        ftd::GradientDirection::Center => "radial-gradient(circle ".to_string(),
        ftd::GradientDirection::Angle { value } => format!("linear-gradient({}deg", value),
    };
    format!("{}, {} )", gradient_style, color)
}

pub fn anchor(l: &ftd::Anchor) -> String {
    match l {
        ftd::Anchor::Parent => "absolute".to_string(),
        ftd::Anchor::Window => "fixed".to_string(),
    }
}

fn row_align(l: &ftd::Position) -> Vec<(String, String)> {
    match l {
        ftd::Position::Center => vec![
            ("align-self".to_string(), "center".to_string()),
            ("margin-bottom".to_string(), "auto".to_string()),
            ("margin-top".to_string(), "auto".to_string()),
        ],
        ftd::Position::Top => vec![
            ("align-self".to_string(), "center".to_string()),
            ("margin-bottom".to_string(), "auto".to_string()),
        ],
        ftd::Position::Bottom => vec![
            ("align-self".to_string(), "center".to_string()),
            ("margin-bottom".to_string(), "0".to_string()),
            ("margin-top".to_string(), "auto".to_string()),
        ],
        _ => vec![],
    }
}

pub(crate) fn column_align(l: &ftd::Position) -> Vec<(String, String)> {
    match l {
        ftd::Position::Center => vec![
            ("align-self".to_string(), "center".to_string()),
            ("margin-left".to_string(), "auto".to_string()),
            ("margin-right".to_string(), "auto".to_string()),
        ],
        ftd::Position::Left => vec![
            ("align-self".to_string(), "flex-start".to_string()),
            ("margin-right".to_string(), "auto".to_string()),
        ],
        ftd::Position::Right => vec![
            ("align-self".to_string(), "flex-end".to_string()),
            ("margin-left".to_string(), "auto".to_string()),
        ],
        _ => vec![],
    }
}

fn grid_align(l: &ftd::Position) -> Vec<(String, String)> {
    match l {
        ftd::Position::Center => vec![
            ("align-self".to_string(), "center".to_string()),
            ("justify-self".to_string(), "center".to_string()),
        ],
        ftd::Position::Top => vec![
            ("align-self".to_string(), "start".to_string()),
            ("justify-self".to_string(), "center".to_string()),
        ],
        ftd::Position::Left => vec![
            ("align-self".to_string(), "center".to_string()),
            ("justify-self".to_string(), "start".to_string()),
        ],
        ftd::Position::Right => vec![
            ("align-self".to_string(), "center".to_string()),
            ("justify-self".to_string(), "end".to_string()),
        ],
        ftd::Position::Bottom => vec![
            ("align-self".to_string(), "end".to_string()),
            ("justify-self".to_string(), "center".to_string()),
        ],
        ftd::Position::TopLeft => vec![
            ("align-self".to_string(), "flex-start".to_string()),
            ("justify-self".to_string(), "flex-start".to_string()),
        ],
        ftd::Position::TopRight => vec![
            ("align-self".to_string(), "flex-start".to_string()),
            ("justify-self".to_string(), "flex-end".to_string()),
        ],
        ftd::Position::BottomLeft => vec![
            ("align-self".to_string(), "flex-end".to_string()),
            ("justify-self".to_string(), "flex-start".to_string()),
        ],
        ftd::Position::BottomRight => vec![
            ("align-self".to_string(), "flex-end".to_string()),
            ("justify-self".to_string(), "flex-end".to_string()),
        ],
    }
}

/*fn container_align(l: &ftd::Position) -> Vec<(String, String)> {
    match l {
        ftd::Position::Center => vec![
            ("align-self".to_string(), "center".to_string()),
            ("margin-bottom".to_string(), "auto".to_string()),
            ("margin-top".to_string(), "auto".to_string()),
        ],
        ftd::Position::Top => vec![
            ("align-self".to_string(), "center".to_string()),
            ("margin-bottom".to_string(), "auto".to_string()),
        ],
        ftd::Position::Left => vec![
            ("align-self".to_string(), "flex-start".to_string()),
            ("margin-bottom".to_string(), "auto".to_string()),
            ("margin-top".to_string(), "auto".to_string()),
        ],
        ftd::Position::Right => vec![
            ("align-self".to_string(), "flex-end".to_string()),
            ("margin-bottom".to_string(), "auto".to_string()),
            ("margin-top".to_string(), "auto".to_string()),
            ("margin-left".to_string(), "auto".to_string()),
        ],
        ftd::Position::Bottom => vec![
            ("align-self".to_string(), "center".to_string()),
            ("margin-bottom".to_string(), "0".to_string()),
            ("margin-top".to_string(), "auto".to_string()),
        ],
        ftd::Position::TopLeft => vec![("align-self".to_string(), "flex-start".to_string())],
        ftd::Position::TopRight => vec![
            ("align-self".to_string(), "flex-end".to_string()),
            ("margin-left".to_string(), "auto".to_string()),
        ],
        ftd::Position::BottomLeft => vec![
            ("align-self".to_string(), "flex-start".to_string()),
            ("margin-bottom".to_string(), "0".to_string()),
            ("margin-top".to_string(), "auto".to_string()),
        ],
        ftd::Position::BottomRight => vec![
            ("align-self".to_string(), "flex-end".to_string()),
            ("margin-bottom".to_string(), "0".to_string()),
            ("margin-top".to_string(), "auto".to_string()),
            ("margin-left".to_string(), "auto".to_string()),
        ],
    }
}*/

fn non_static_container_align(l: &ftd::Position, inner: bool) -> Vec<(String, String)> {
    match l {
        ftd::Position::Center => vec![
            ("left".to_string(), "50%".to_string()),
            ("top".to_string(), "50%".to_string()),
            ("transform".to_string(), "translate(-50%,-50%)".to_string()),
        ],
        ftd::Position::Top => {
            if inner {
                vec![
                    ("top".to_string(), "0".to_string()),
                    ("left".to_string(), "50%".to_string()),
                    ("transform".to_string(), "translateX(-50%)".to_string()),
                ]
            } else {
                vec![
                    ("bottom".to_string(), "100%".to_string()),
                    ("left".to_string(), "50%".to_string()),
                    ("transform".to_string(), "translateX(-50%)".to_string()),
                ]
            }
        }
        ftd::Position::Left => {
            if inner {
                vec![
                    ("left".to_string(), "0".to_string()),
                    ("top".to_string(), "50%".to_string()),
                    ("transform".to_string(), "translateY(-50%)".to_string()),
                ]
            } else {
                vec![
                    ("right".to_string(), "100%".to_string()),
                    ("top".to_string(), "50%".to_string()),
                    ("transform".to_string(), "translateY(-50%)".to_string()),
                ]
            }
        }
        ftd::Position::Right => {
            if inner {
                vec![
                    ("right".to_string(), "0".to_string()),
                    ("top".to_string(), "50%".to_string()),
                    ("transform".to_string(), "translateY(-50%)".to_string()),
                ]
            } else {
                vec![
                    ("left".to_string(), "100%".to_string()),
                    ("top".to_string(), "50%".to_string()),
                    ("transform".to_string(), "translateY(-50%)".to_string()),
                ]
            }
        }
        ftd::Position::Bottom => {
            if inner {
                vec![
                    ("bottom".to_string(), "0".to_string()),
                    ("left".to_string(), "50%".to_string()),
                    ("transform".to_string(), "translateX(-50%)".to_string()),
                ]
            } else {
                vec![
                    ("top".to_string(), "100%".to_string()),
                    ("left".to_string(), "50%".to_string()),
                    ("transform".to_string(), "translateX(-50%)".to_string()),
                ]
            }
        }
        ftd::Position::TopLeft => {
            if inner {
                vec![
                    ("top".to_string(), "0".to_string()),
                    ("left".to_string(), "0".to_string()),
                ]
            } else {
                vec![
                    ("bottom".to_string(), "100%".to_string()),
                    ("right".to_string(), "100%".to_string()),
                ]
            }
        }
        ftd::Position::TopRight => {
            if inner {
                vec![
                    ("top".to_string(), "0".to_string()),
                    ("right".to_string(), "0".to_string()),
                ]
            } else {
                vec![
                    ("bottom".to_string(), "100%".to_string()),
                    ("left".to_string(), "100%".to_string()),
                ]
            }
        }
        ftd::Position::BottomLeft => {
            if inner {
                vec![
                    ("bottom".to_string(), "0".to_string()),
                    ("left".to_string(), "0".to_string()),
                ]
            } else {
                vec![
                    ("top".to_string(), "100%".to_string()),
                    ("right".to_string(), "100%".to_string()),
                ]
            }
        }
        ftd::Position::BottomRight => {
            if inner {
                vec![
                    ("bottom".to_string(), "0".to_string()),
                    ("right".to_string(), "0".to_string()),
                ]
            } else {
                vec![
                    ("top".to_string(), "100%".to_string()),
                    ("left".to_string(), "100%".to_string()),
                ]
            }
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn get_translate(
    left: &Option<i64>,
    right: &Option<i64>,
    up: &Option<i64>,
    down: &Option<i64>,
    scale: &Option<f64>,
    scale_x: &Option<f64>,
    scale_y: &Option<f64>,
    rotate: &Option<i64>,
    doc_id: &str,
) -> ftd::ftd2021::p1::Result<Option<String>> {
    let mut translate = match (left, right, up, down) {
        (Some(_), Some(_), Some(_), Some(_)) => {
            return ftd::ftd2021::p2::utils::e2(
                "move-up, move-down, move-left and move-right all 4 can't be used at once!",
                doc_id,
                0, // TODO
            );
        }
        (Some(_), Some(_), _, _) => {
            return ftd::ftd2021::p2::utils::e2(
                "move-left, move-right both can't be used at once!",
                doc_id,
                0, // TODO
            );
        }
        (_, _, Some(_), Some(_)) => {
            // TODO
            return ftd::ftd2021::p2::utils::e2(
                "move-up, move-down both can't be used at once!",
                doc_id,
                0,
            );
        }
        (Some(l), None, None, None) => Some(format!("translateX(-{}px) ", l)),
        (Some(l), None, Some(u), None) => Some(format!("translate(-{}px, -{}px) ", l, u)),
        (Some(l), None, None, Some(d)) => Some(format!("translate(-{}px, {}px) ", l, d)),
        (None, Some(r), None, None) => Some(format!("translateX({}px) ", r)),
        (None, Some(r), Some(u), None) => Some(format!("translate({}px, -{}px) ", r, u)),
        (None, Some(r), None, Some(d)) => Some(format!("translate({}px, {}px) ", r, d)),
        (None, None, Some(u), None) => Some(format!("translateY(-{}px) ", u)),
        (None, None, None, Some(d)) => Some(format!("translateY({}px) ", d)),
        _ => None,
    };

    if let Some(ref scale) = scale {
        if let Some(d) = translate {
            translate = Some(format!("{} scale({})", d, scale));
        } else {
            translate = Some(format!("scale({})", scale));
        };
    }
    if let Some(ref scale) = scale_x {
        if let Some(d) = translate {
            translate = Some(format!("{} scaleX({})", d, scale));
        } else {
            translate = Some(format!("scaleX({})", scale));
        };
    }
    if let Some(ref scale) = scale_y {
        if let Some(d) = translate {
            translate = Some(format!("{} scaleY({})", d, scale));
        } else {
            translate = Some(format!("scaleY({})", scale));
        };
    }
    if let Some(ref rotate) = rotate {
        if let Some(d) = translate {
            translate = Some(format!("{} rotate({}deg)", d, rotate));
        } else {
            translate = Some(format!("rotate({}deg)", rotate));
        };
    }
    Ok(translate)
}

pub fn spacing(l: &ftd::Spacing, f: &str) -> (String, String) {
    match l {
        ftd::Spacing::SpaceEvenly => (s("justify-content"), s("space-evenly")),
        ftd::Spacing::SpaceBetween => (s("justify-content"), s("space-between")),
        ftd::Spacing::SpaceAround => (s("justify-content"), s("space-around")),
        ftd::Spacing::Absolute { value } => (s(f), s(value)),
    }
}

fn style(l: &ftd::Weight) -> (String, String) {
    match l {
        ftd::Weight::Heavy => ("font-weight".to_string(), "900".to_string()),
        ftd::Weight::ExtraBold => ("font-weight".to_string(), "800".to_string()),
        ftd::Weight::Bold => ("font-weight".to_string(), "700".to_string()),
        ftd::Weight::SemiBold => ("font-weight".to_string(), "600".to_string()),
        ftd::Weight::Medium => ("font-weight".to_string(), "500".to_string()),
        ftd::Weight::Regular => ("font-weight".to_string(), "400".to_string()),
        ftd::Weight::Light => ("font-weight".to_string(), "300".to_string()),
        ftd::Weight::ExtraLight => ("font-weight".to_string(), "200".to_string()),
        ftd::Weight::HairLine => ("font-weight".to_string(), "100".to_string()),
    }
}
