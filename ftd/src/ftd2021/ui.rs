use std::fmt::Display;

#[derive(serde::Deserialize, Clone, Debug, PartialEq, serde::Serialize)]
#[serde(tag = "type")]
pub enum Element {
    TextBlock(TextBlock),
    Code(Code),
    Image(Image),
    Row(Row),
    Column(Column),
    IFrame(IFrame),
    Input(Input),
    Integer(Text),
    Boolean(Text),
    Decimal(Text),
    Scene(Scene),
    Grid(Grid),
    Markup(Markups),
    Null,
}

#[derive(serde::Deserialize, Debug, PartialEq, Clone, serde::Serialize, Default)]
pub struct Markups {
    pub text: ftd::ftd2021::Rendered,
    pub common: Box<Common>,
    pub text_align: TextAlign,
    pub line: bool,
    pub style: Style,
    pub font: Option<Type>,
    pub line_clamp: Option<i64>,
    pub text_indent: Option<Length>,
    pub children: Vec<Markup>,
}

impl Markups {
    pub(crate) fn to_text(&self) -> Text {
        Text {
            text: self.text.to_owned(),
            line: self.line,
            common: self.common.clone(),
            text_align: self.text_align.to_owned(),
            style: self.style.to_owned(),
            font: self.font.to_owned(),
            line_clamp: self.line_clamp,
            text_indent: self.text_indent.to_owned(),
        }
    }
}

#[derive(serde::Deserialize, Debug, PartialEq, Clone, serde::Serialize)]
pub struct Markup {
    pub itext: IText,
    pub children: Vec<Markup>,
}

#[derive(serde::Deserialize, Debug, PartialEq, Clone, serde::Serialize)]
pub enum IText {
    Text(Text),
    TextBlock(TextBlock),
    Integer(Text),
    Boolean(Text),
    Decimal(Text),
    Markup(Markups),
}

impl Element {
    pub(crate) fn set_children_count_variable(
        elements: &mut [ftd::Element],
        local_variables: &ftd::Map<ftd::ftd2021::p2::Thing>,
    ) {
        // Simplified implementation that handles Box<Common> properly
        for child in elements.iter_mut() {
            match child {
                Element::Row(row) => {
                    Self::set_children_count_variable(&mut row.container.children, local_variables);
                    if let Some((_, _, external_children)) = &mut row.container.external_children {
                        Self::set_children_count_variable(external_children, local_variables);
                    }
                }
                Element::Column(col) => {
                    Self::set_children_count_variable(&mut col.container.children, local_variables);
                    if let Some((_, _, external_children)) = &mut col.container.external_children {
                        Self::set_children_count_variable(external_children, local_variables);
                    }
                }
                Element::Scene(scene) => {
                    Self::set_children_count_variable(
                        &mut scene.container.children,
                        local_variables,
                    );
                    if let Some((_, _, external_children)) = &mut scene.container.external_children
                    {
                        Self::set_children_count_variable(external_children, local_variables);
                    }
                }
                Element::Grid(grid) => {
                    Self::set_children_count_variable(
                        &mut grid.container.children,
                        local_variables,
                    );
                    if let Some((_, _, external_children)) = &mut grid.container.external_children {
                        Self::set_children_count_variable(external_children, local_variables);
                    }
                }
                Element::Markup(markup) => {
                    // Process markup children recursively
                    for markup_child in markup.children.iter_mut() {
                        if let IText::Markup(_nested_markup) = &mut markup_child.itext {
                            // This is simplified - just recurse
                        }
                    }
                }
                // Skip text processing for now due to complexity
                _ => continue,
            }
        }
    }

    pub(crate) fn set_default_locals(elements: &mut [ftd::Element]) {
        for child in elements.iter_mut() {
            match child {
                Element::TextBlock(text_block) => {
                    Self::check_mouse_events(&mut text_block.common);
                }
                Element::Code(code) => {
                    Self::check_mouse_events(&mut code.common);
                }
                Element::Image(image) => {
                    Self::check_mouse_events(&mut image.common);
                }
                Element::IFrame(iframe) => {
                    Self::check_mouse_events(&mut iframe.common);
                }
                Element::Input(input) => {
                    Self::check_mouse_events(&mut input.common);
                }
                Element::Integer(text) | Element::Boolean(text) | Element::Decimal(text) => {
                    Self::check_mouse_events(&mut text.common);
                }
                Element::Markup(markup) => {
                    Self::check_mouse_events_common(&mut markup.common);
                }
                Element::Row(row) => {
                    Self::set_default_locals(&mut row.container.children);
                    if let Some((_, _, external_children)) = &mut row.container.external_children {
                        Self::set_default_locals(external_children);
                    }
                    Self::check_mouse_events(&mut row.common);
                }
                Element::Column(col) => {
                    Self::set_default_locals(&mut col.container.children);
                    if let Some((_, _, external_children)) = &mut col.container.external_children {
                        Self::set_default_locals(external_children);
                    }
                    Self::check_mouse_events(&mut col.common);
                }
                Element::Scene(scene) => {
                    Self::set_default_locals(&mut scene.container.children);
                    if let Some((_, _, external_children)) = &mut scene.container.external_children
                    {
                        Self::set_default_locals(external_children);
                    }
                    Self::check_mouse_events(&mut scene.common);
                }
                Element::Grid(grid) => {
                    Self::set_default_locals(&mut grid.container.children);
                    if let Some((_, _, external_children)) = &mut grid.container.external_children {
                        Self::set_default_locals(external_children);
                    }
                    Self::check_mouse_events(&mut grid.common);
                }
                Element::Null => continue,
            }
        }
    }

    fn check_mouse_events(common: &mut Box<Common>) -> Option<String> {
        if let Some(ref mut condition) = common.condition
            && condition.variable.contains("MOUSE-IN")
        {
            let result = condition.variable.clone();
            common
                .events
                .extend(ftd::ftd2021::p2::Event::mouse_event(&result));
            return Some(result);
        }
        if let Some(ref reference) = common.reference
            && reference.contains("MOUSE-IN")
        {
            let result = reference.clone();
            common
                .events
                .extend(ftd::ftd2021::p2::Event::mouse_event(&result));
            return Some(result);
        }
        for (_, v) in common.conditional_attribute.iter_mut() {
            for (condition, _) in &mut v.conditions_with_value {
                if condition.variable.contains("MOUSE-IN") {
                    let result = condition.variable.clone();
                    common
                        .events
                        .extend(ftd::ftd2021::p2::Event::mouse_event(&result));
                    return Some(result);
                }
            }
        }
        None
    }

    fn check_mouse_events_common(common: &mut Common) -> Option<String> {
        if let Some(ref mut condition) = common.condition
            && condition.variable.contains("MOUSE-IN")
        {
            let result = condition.variable.clone();
            common
                .events
                .extend(ftd::ftd2021::p2::Event::mouse_event(&result));
            return Some(result);
        }
        if let Some(ref reference) = common.reference
            && reference.contains("MOUSE-IN")
        {
            let result = reference.clone();
            common
                .events
                .extend(ftd::ftd2021::p2::Event::mouse_event(&result));
            return Some(result);
        }
        for (_, v) in common.conditional_attribute.iter_mut() {
            for (condition, _) in &mut v.conditions_with_value {
                if condition.variable.contains("MOUSE-IN") {
                    let result = condition.variable.clone();
                    common
                        .events
                        .extend(ftd::ftd2021::p2::Event::mouse_event(&result));
                    return Some(result);
                }
            }
        }
        None
    }

    pub fn set_id(children: &mut [ftd::Element], index_vec: &[usize], external_id: Option<String>) {
        for (idx, child) in children.iter_mut().enumerate() {
            match child {
                Self::TextBlock(text_block) => {
                    Self::set_element_data_id(
                        &mut text_block.common.data_id,
                        &text_block.common.is_dummy,
                        index_vec,
                        idx,
                        &external_id,
                    );
                }
                Self::Code(code) => {
                    Self::set_element_data_id(
                        &mut code.common.data_id,
                        &code.common.is_dummy,
                        index_vec,
                        idx,
                        &external_id,
                    );
                }
                Self::Image(image) => {
                    Self::set_element_data_id(
                        &mut image.common.data_id,
                        &image.common.is_dummy,
                        index_vec,
                        idx,
                        &external_id,
                    );
                }
                Self::IFrame(iframe) => {
                    Self::set_element_data_id(
                        &mut iframe.common.data_id,
                        &iframe.common.is_dummy,
                        index_vec,
                        idx,
                        &external_id,
                    );
                }
                Self::Input(input) => {
                    Self::set_element_data_id(
                        &mut input.common.data_id,
                        &input.common.is_dummy,
                        index_vec,
                        idx,
                        &external_id,
                    );
                }
                Self::Integer(text) | Self::Boolean(text) | Self::Decimal(text) => {
                    Self::set_element_data_id(
                        &mut text.common.data_id,
                        &text.common.is_dummy,
                        index_vec,
                        idx,
                        &external_id,
                    );
                }
                Self::Markup(markup) => {
                    Self::set_element_id_common(
                        &mut markup.common.data_id,
                        &markup.common.is_dummy,
                        index_vec,
                        idx,
                        &external_id,
                    );
                    Self::set_markup_id(&mut markup.children, index_vec, external_id.clone());
                }
                Self::Row(row) => {
                    Self::set_element_data_id(
                        &mut row.common.data_id,
                        &row.common.is_dummy,
                        index_vec,
                        idx,
                        &external_id,
                    );
                    let mut new_index_vec = index_vec.to_vec();
                    new_index_vec.push(idx);
                    Self::set_id(
                        &mut row.container.children,
                        &new_index_vec,
                        external_id.clone(),
                    );
                    if let Some((id, container, external_children)) =
                        &mut row.container.external_children
                        && let Some(ftd::Element::Column(col)) = external_children.first_mut()
                    {
                        let index_string: String = new_index_vec
                            .iter()
                            .map(|v| v.to_string())
                            .collect::<Vec<String>>()
                            .join(",");
                        let external_id = Some({
                            if let Some(ref ext_id) = external_id {
                                format!("{ext_id}.{id}-external:{index_string}")
                            } else {
                                format!("{id}-external:{index_string}")
                            }
                        });
                        col.common.data_id.clone_from(&external_id);
                        if let Some(val) = container.first_mut() {
                            new_index_vec.append(&mut val.to_vec());
                            Self::set_id(&mut col.container.children, &new_index_vec, external_id);
                        }
                    }
                }
                Self::Column(col) => {
                    Self::set_element_data_id(
                        &mut col.common.data_id,
                        &col.common.is_dummy,
                        index_vec,
                        idx,
                        &external_id,
                    );
                    let mut new_index_vec = index_vec.to_vec();
                    new_index_vec.push(idx);
                    Self::set_id(
                        &mut col.container.children,
                        &new_index_vec,
                        external_id.clone(),
                    );
                    if let Some((id, container, external_children)) =
                        &mut col.container.external_children
                        && let Some(ftd::Element::Column(nested_col)) =
                            external_children.first_mut()
                    {
                        let index_string: String = new_index_vec
                            .iter()
                            .map(|v| v.to_string())
                            .collect::<Vec<String>>()
                            .join(",");
                        let external_id = Some({
                            if let Some(ref ext_id) = external_id {
                                format!("{ext_id}.{id}-external:{index_string}")
                            } else {
                                format!("{id}-external:{index_string}")
                            }
                        });
                        nested_col.common.data_id.clone_from(&external_id);
                        if let Some(val) = container.first_mut() {
                            new_index_vec.append(&mut val.to_vec());
                            Self::set_id(
                                &mut nested_col.container.children,
                                &new_index_vec,
                                external_id,
                            );
                        }
                    }
                }
                Self::Scene(scene) => {
                    Self::set_element_data_id(
                        &mut scene.common.data_id,
                        &scene.common.is_dummy,
                        index_vec,
                        idx,
                        &external_id,
                    );
                    let mut new_index_vec = index_vec.to_vec();
                    new_index_vec.push(idx);
                    Self::set_id(
                        &mut scene.container.children,
                        &new_index_vec,
                        external_id.clone(),
                    );
                }
                Self::Grid(grid) => {
                    Self::set_element_data_id(
                        &mut grid.common.data_id,
                        &grid.common.is_dummy,
                        index_vec,
                        idx,
                        &external_id,
                    );
                    let mut new_index_vec = index_vec.to_vec();
                    new_index_vec.push(idx);
                    Self::set_id(
                        &mut grid.container.children,
                        &new_index_vec,
                        external_id.clone(),
                    );
                }
                Self::Null => continue,
            }
        }
    }

    fn set_element_data_id(
        data_id: &mut Option<String>,
        is_dummy: &bool,
        index_vec: &[usize],
        idx: usize,
        external_id: &Option<String>,
    ) {
        let index_string = if *is_dummy {
            Self::get_index_string(index_vec, None)
        } else {
            Self::get_index_string(index_vec, Some(idx))
        };

        let external_part = if let Some(ext_id) = external_id {
            format!(":{ext_id}")
        } else {
            "".to_string()
        };

        let dummy_part = if *is_dummy { ":dummy" } else { "" };

        if let Some(id) = data_id {
            *id = format!("{id}:{index_string}{external_part}{dummy_part}");
        } else {
            *data_id = Some(format!("{index_string}{external_part}{dummy_part}"));
        }
    }

    fn set_element_id_common(
        data_id: &mut Option<String>,
        is_dummy: &bool,
        index_vec: &[usize],
        idx: usize,
        external_id: &Option<String>,
    ) {
        // Same logic but for non-boxed Common (Markups)
        Self::set_element_data_id(data_id, is_dummy, index_vec, idx, external_id);
    }

    fn get_index_string(index_vec: &[usize], idx: Option<usize>) -> String {
        let mut full_index = index_vec.to_vec();
        if let Some(idx) = idx {
            full_index.push(idx);
        }
        if full_index.is_empty() {
            "0".to_string()
        } else {
            full_index
                .iter()
                .map(|v| v.to_string())
                .collect::<Vec<String>>()
                .join(",")
        }
    }

    fn set_markup_id(
        children: &mut [ftd::Markup],
        index_vec: &[usize],
        external_id: Option<String>,
    ) {
        for (idx, child) in children.iter_mut().enumerate() {
            let mut new_index_vec = index_vec.to_vec();
            new_index_vec.push(idx);

            match &mut child.itext {
                IText::Text(t) | IText::Integer(t) | IText::Boolean(t) | IText::Decimal(t) => {
                    Self::set_element_data_id(
                        &mut t.common.data_id,
                        &t.common.is_dummy,
                        index_vec,
                        idx,
                        &external_id,
                    );
                }
                IText::TextBlock(tb) => {
                    Self::set_element_data_id(
                        &mut tb.common.data_id,
                        &tb.common.is_dummy,
                        index_vec,
                        idx,
                        &external_id,
                    );
                }
                IText::Markup(markup) => {
                    Self::set_element_data_id(
                        &mut markup.common.data_id,
                        &markup.common.is_dummy,
                        index_vec,
                        idx,
                        &external_id,
                    );
                    Self::set_markup_id(&mut markup.children, &new_index_vec, external_id.clone());
                }
            }
        }
    }

    pub fn get_external_children_condition(
        &self,
        external_open_id: &Option<String>,
        external_children_container: &[Vec<usize>],
    ) -> Vec<ftd::ExternalChildrenCondition> {
        let mut result = Vec::new();

        let (_id, external_children, children) = match self {
            Self::Row(row) => (
                &row.common.data_id,
                &row.container.external_children,
                &row.container.children,
            ),
            Self::Column(col) => (
                &col.common.data_id,
                &col.container.external_children,
                &col.container.children,
            ),
            Self::Scene(scene) => (
                &scene.common.data_id,
                &scene.container.external_children,
                &scene.container.children,
            ),
            Self::Grid(grid) => (
                &grid.common.data_id,
                &grid.container.external_children,
                &grid.container.children,
            ),
            _ => return result, // Non-container elements return empty
        };

        // Simplified version - just collect basic external children info
        if let Some((open_id, _container, ext_children)) = external_children {
            result.push(ftd::ExternalChildrenCondition {
                condition: vec![],       // Simplified - no specific conditions for now
                set_at: open_id.clone(), // Use the open_id as set_at
            });

            // Recursively process external children
            for child in ext_children {
                result.extend(child.get_external_children_condition(
                    external_open_id,
                    external_children_container,
                ));
            }
        }

        // Process regular children
        for child in children {
            result.extend(
                child
                    .get_external_children_condition(external_open_id, external_children_container),
            );
        }

        result
    }
    pub fn get_external_children_dependencies(
        children: &[ftd::Element],
    ) -> ftd::ExternalChildrenDependenciesMap {
        let mut d: ftd::ExternalChildrenDependenciesMap = Default::default();
        for child in children {
            let container = match child {
                ftd::Element::Row(ftd::Row { container, .. }) => container,
                ftd::Element::Column(ftd::Column { container, .. }) => container,
                ftd::Element::Scene(ftd::Scene { container, .. }) => container,
                ftd::Element::Grid(ftd::Grid { container, .. }) => container,
                _ => continue,
            };
            let all_locals = ftd::Element::get_external_children_dependencies(&container.children);
            for (k, v) in all_locals {
                d.insert(k.to_string(), v);
            }
            if let Some((external_open_id, external_children_container, external_children)) =
                &container.external_children
                && let Some(ftd::Element::Column(col)) = external_children.first()
            {
                let external_children_condition: Vec<ftd::ExternalChildrenCondition> = child
                    .get_external_children_condition(
                        &Some(external_open_id.to_string()),
                        external_children_container,
                    );
                d.insert(
                    col.common.data_id.as_ref().expect("").to_string(),
                    external_children_condition,
                );
                let all_locals =
                    ftd::Element::get_external_children_dependencies(&col.container.children);
                for (k, v) in all_locals {
                    d.insert(k.to_string(), v);
                }
            }
        }
        d
    }

    pub fn get_event_dependencies(children: &[ftd::Element], data: &mut ftd::DataDependenciesMap) {
        for child in children {
            let (font, common) = match child {
                ftd::Element::Column(ftd::Column {
                    common, container, ..
                })
                | ftd::Element::Row(ftd::Row {
                    common, container, ..
                })
                | ftd::Element::Scene(ftd::Scene {
                    common, container, ..
                })
                | ftd::Element::Grid(ftd::Grid {
                    common, container, ..
                }) => {
                    ftd::Element::get_event_dependencies(&container.children, data);
                    if let Some((_, _, external_children)) = &container.external_children {
                        ftd::Element::get_event_dependencies(external_children, data);
                    }
                    (&None, common)
                }
                ftd::Element::Markup(ftd::Markups {
                    font,
                    common,
                    children,
                    ..
                }) => {
                    markup_get_event_dependencies(children, data);
                    (font, common)
                }
                ftd::Element::Code(ftd::Code { font, common, .. })
                | ftd::Element::Integer(ftd::Text { font, common, .. })
                | ftd::Element::Boolean(ftd::Text { font, common, .. })
                | ftd::Element::Decimal(ftd::Text { font, common, .. })
                | ftd::Element::Input(ftd::Input { font, common, .. }) => (font, common),
                ftd::Element::IFrame(ftd::IFrame { common, .. })
                | ftd::Element::TextBlock(ftd::TextBlock { common, .. })
                | ftd::Element::Image(ftd::Image { common, .. }) => (&None, common),
                ftd::Element::Null => continue,
            };
            value_condition(&common.reference, &common.data_id, data);
            color_condition(common, &common.data_id, data);
            font_condition(&common.data_id, data, font, &common.conditional_attribute);
            image_condition(&common.data_id, data, &common.background_image);
            style_condition(&common.conditional_attribute, &common.data_id, data);
            visibility_condition(&common.condition, &common.data_id, data);
        }

        fn markup_get_event_dependencies(
            children: &[ftd::Markup],
            data: &mut ftd::DataDependenciesMap,
        ) {
            for child in children {
                let (font, common) = match child.itext {
                    IText::Text(ref t)
                    | IText::Integer(ref t)
                    | IText::Boolean(ref t)
                    | IText::Decimal(ref t) => (&t.font, &t.common),
                    IText::TextBlock(ref t) => (&None, &t.common),
                    IText::Markup(ref t) => {
                        markup_get_event_dependencies(&t.children, data);
                        (&t.font, &t.common)
                    }
                };
                markup_get_event_dependencies(&child.children, data);
                value_condition(&common.reference, &common.data_id, data);
                color_condition(common, &common.data_id, data);
                font_condition(&common.data_id, data, font, &common.conditional_attribute);
                image_condition(&common.data_id, data, &common.background_image);
                style_condition(&common.conditional_attribute, &common.data_id, data);
                visibility_condition(&common.condition, &common.data_id, data);
            }
        }

        fn value_condition(
            reference: &Option<String>,
            id: &Option<String>,
            data: &mut ftd::DataDependenciesMap,
        ) {
            if let Some(reference) = reference {
                let id = id.clone().expect("universal id should be present");

                let (variable, remaining) =
                    ftd::ftd2021::p2::utils::get_doc_name_and_remaining(reference).unwrap();
                if let Some(ftd::Data { dependencies, .. }) = data.get_mut(&variable) {
                    let json = ftd::Dependencies {
                        dependency_type: ftd::DependencyType::Value,
                        condition: None,
                        parameters: Default::default(),
                        remaining,
                    };
                    if let Some(dependencies) = dependencies.get_mut(&id) {
                        let mut d = serde_json::from_value::<Vec<ftd::Dependencies>>(
                            dependencies.to_owned(),
                        )
                        .unwrap();
                        d.push(json);
                        *dependencies = serde_json::to_value(&d).unwrap();
                    } else {
                        dependencies.insert(id, serde_json::to_value(vec![json]).unwrap());
                    }
                }
            }
        }

        fn color_condition(
            common: &ftd::Common,
            id: &Option<String>,
            data: &mut ftd::DataDependenciesMap,
        ) {
            let id = id.clone().expect("universal id should be present");
            if let Some(ref color) = common.color {
                color_condition(
                    color,
                    id.as_str(),
                    data,
                    "color",
                    &common.conditional_attribute,
                );
            }
            if let Some(ref color) = common.background_color {
                color_condition(
                    color,
                    id.as_str(),
                    data,
                    "background-color",
                    &common.conditional_attribute,
                );
            }
            if let Some(ref color) = common.border_top_color {
                color_condition(
                    color,
                    id.as_str(),
                    data,
                    "background-color",
                    &common.conditional_attribute,
                );
            }
            if let Some(ref color) = common.border_right_color {
                color_condition(
                    color,
                    id.as_str(),
                    data,
                    "background-color",
                    &common.conditional_attribute,
                );
            }
            if let Some(ref color) = common.border_bottom_color {
                color_condition(
                    color,
                    id.as_str(),
                    data,
                    "background-color",
                    &common.conditional_attribute,
                );
            }
            if let Some(ref color) = common.border_left_color {
                color_condition(
                    color,
                    id.as_str(),
                    data,
                    "background-color",
                    &common.conditional_attribute,
                );
            }
            if let Some(ref color) = common.border_color {
                color_condition(
                    color,
                    id.as_str(),
                    data,
                    "border-color",
                    &common.conditional_attribute,
                );
            }

            fn color_condition(
                _color: &ftd::Color,
                id: &str,
                data: &mut ftd::DataDependenciesMap,
                style: &str,
                conditional_attribute: &ftd::Map<ftd::ConditionalAttribute>,
            ) {
                let (reference, value) = if let Some(ftd::ConditionalAttribute {
                    default:
                        Some(ConditionalValue {
                            reference: Some(reference),
                            value,
                            ..
                        }),
                    ..
                }) = conditional_attribute.get(style)
                {
                    (reference.to_string(), value.to_owned())
                // } else if let Some(ref reference) = color.reference {
                //     (
                //         reference.to_string(),
                //         serde_json::json!({ "light": ftd::html::color(&color.light), "dark": ftd::html::color(&color.dark), "$kind$": "light" }),
                //     )
                } else {
                    return;
                };
                let parameters = {
                    let mut parameters = ftd::Map::new();
                    parameters.insert(
                        style.to_string(),
                        ftd::ConditionalValueWithDefault {
                            value: ConditionalValue {
                                value,
                                important: false,
                                reference: None,
                            },
                            default: None,
                        },
                    );
                    let dependents = conditional_attribute
                        .get(style)
                        .unwrap_or(&ConditionalAttribute {
                            attribute_type: AttributeType::Style,
                            conditions_with_value: vec![],
                            default: None,
                        })
                        .conditions_with_value
                        .iter()
                        .map(|(v, _)| v.variable.to_string())
                        .collect::<Vec<String>>();
                    parameters.insert(
                        "children".to_string(),
                        ftd::ConditionalValueWithDefault {
                            value: ConditionalValue {
                                value: serde_json::to_value(dependents).unwrap(),
                                important: false,
                                reference: None,
                            },
                            default: None,
                        },
                    );
                    parameters
                };
                let (variable, remaining) =
                    ftd::ftd2021::p2::utils::get_doc_name_and_remaining(&reference).unwrap();
                if let Some(ftd::Data { dependencies, .. }) = data.get_mut(&variable) {
                    let json = ftd::Dependencies {
                        dependency_type: ftd::DependencyType::Style,
                        condition: None,
                        parameters,
                        remaining,
                    };
                    if let Some(dependencies) = dependencies.get_mut(id) {
                        let mut d = serde_json::from_value::<Vec<ftd::Dependencies>>(
                            dependencies.to_owned(),
                        )
                        .unwrap();
                        d.push(json);
                        *dependencies = serde_json::to_value(&d).unwrap();
                    } else {
                        dependencies
                            .insert(id.to_string(), serde_json::to_value(vec![json]).unwrap());
                    }
                }
            }
        }

        fn font_condition(
            id: &Option<String>,
            data: &mut ftd::DataDependenciesMap,
            font: &Option<Type>,
            conditions: &ftd::Map<ConditionalAttribute>,
        ) {
            let id = id.clone().expect("universal id should be present");
            if !conditions
                .keys()
                .any(|x| ["line-height", "font-size"].contains(&x.as_str()))
            {
                //mention all font attributes.
                // since font is not conditional attribute yet so this will always pass
                return;
            }
            if let Some(type_) = font {
                font_condition(type_, id.as_str(), data);
            }

            fn font_condition(type_: &ftd::Type, id: &str, data: &mut ftd::DataDependenciesMap) {
                let (reference, value) = if let Some(ref reference) = type_.reference {
                    let desktop = serde_json::to_value(&type_.desktop).unwrap();
                    let mobile = serde_json::to_value(&type_.mobile).unwrap();
                    let xl = serde_json::to_value(&type_.xl).unwrap();
                    (
                        reference.to_string(),
                        serde_json::json!({ "desktop": desktop,
                            "mobile": mobile,
                            "xl": xl,
                            "$kind$": "desktop"
                        }),
                    )
                } else {
                    return;
                };
                let parameters = {
                    let mut parameters = ftd::Map::new();
                    parameters.insert(
                        "font".to_string(),
                        ftd::ConditionalValueWithDefault {
                            value: ConditionalValue {
                                value,
                                important: false,
                                reference: None,
                            },
                            default: None,
                        },
                    );
                    parameters
                };

                let (variable, remaining) =
                    ftd::ftd2021::p2::utils::get_doc_name_and_remaining(&reference).unwrap();

                if let Some(ftd::Data { dependencies, .. }) = data.get_mut(&variable) {
                    let json = ftd::Dependencies {
                        dependency_type: ftd::DependencyType::Style,
                        condition: None,
                        parameters,
                        remaining,
                    };
                    if let Some(dependencies) = dependencies.get_mut(id) {
                        let mut d = serde_json::from_value::<Vec<ftd::Dependencies>>(
                            dependencies.to_owned(),
                        )
                        .unwrap();
                        d.push(json);
                        *dependencies = serde_json::to_value(&d).unwrap();
                    } else {
                        dependencies
                            .insert(id.to_string(), serde_json::to_value(vec![json]).unwrap());
                    }
                }
            }
        }

        fn image_condition(
            id: &Option<String>,
            data: &mut ftd::DataDependenciesMap,
            background_image: &Option<ImageSrc>,
        ) {
            let id = id.clone().expect("universal id should be present");
            if let Some(image_src) = background_image {
                image_condition(image_src, id.as_str(), data);
            }

            fn image_condition(
                image_src: &ftd::ImageSrc,
                id: &str,
                data: &mut ftd::DataDependenciesMap,
            ) {
                let (reference, value) = if let Some(ref reference) = image_src.reference {
                    (
                        reference.to_string(),
                        serde_json::json!({ "light": format!("url({})", image_src.light),
                            "dark": format!("url({})", image_src.light),
                            "$kind$": "light"
                        }),
                    )
                } else {
                    return;
                };

                let parameters = {
                    let mut parameters = ftd::Map::new();
                    parameters.insert(
                        "background-image".to_string(),
                        ftd::ConditionalValueWithDefault {
                            value: ConditionalValue {
                                value,
                                important: false,
                                reference: None,
                            },
                            default: None,
                        },
                    );
                    parameters
                };

                let (variable, remaining) =
                    ftd::ftd2021::p2::utils::get_doc_name_and_remaining(&reference).unwrap();

                if let Some(ftd::Data { dependencies, .. }) = data.get_mut(&variable) {
                    let json = ftd::Dependencies {
                        dependency_type: ftd::DependencyType::Style,
                        condition: None,
                        parameters,
                        remaining,
                    };
                    if let Some(dependencies) = dependencies.get_mut(id) {
                        let mut d = serde_json::from_value::<Vec<ftd::Dependencies>>(
                            dependencies.to_owned(),
                        )
                        .unwrap();
                        d.push(json);
                        *dependencies = serde_json::to_value(&d).unwrap();
                    } else {
                        dependencies
                            .insert(id.to_string(), serde_json::to_value(vec![json]).unwrap());
                    }
                }
            }
        }

        fn style_condition(
            conditional_attributes: &ftd::Map<ConditionalAttribute>,
            id: &Option<String>,
            data: &mut ftd::DataDependenciesMap,
        ) {
            for (k, v) in conditional_attributes {
                if let ftd::ConditionalAttribute {
                    attribute_type: ftd::AttributeType::Style,
                    conditions_with_value,
                    default,
                } = v
                {
                    for (condition, value) in conditions_with_value {
                        let id = id.clone().expect("universal id should be present");
                        let (variable, remaining) =
                            ftd::ftd2021::p2::utils::get_doc_name_and_remaining(
                                &condition.variable,
                            )
                            .unwrap();
                        if let Some(ftd::Data { dependencies, .. }) = data.get_mut(&variable) {
                            let json = ftd::Dependencies {
                                dependency_type: ftd::DependencyType::Style,
                                condition: Some(condition.value.to_owned()),
                                parameters: std::iter::IntoIterator::into_iter([(
                                    k.to_string(),
                                    ftd::ConditionalValueWithDefault {
                                        value: value.clone(),
                                        default: default.clone(),
                                    },
                                )])
                                .collect(),
                                remaining,
                            };
                            if let Some(dependencies) = dependencies.get_mut(&id) {
                                let mut d = serde_json::from_value::<Vec<ftd::Dependencies>>(
                                    dependencies.to_owned(),
                                )
                                .unwrap();
                                d.push(json);
                                *dependencies = serde_json::to_value(&d).unwrap();
                            } else {
                                dependencies.insert(
                                    id.to_string(),
                                    serde_json::to_value(vec![json]).unwrap(),
                                );
                            }
                        } else {
                            panic!("{} should be declared", condition.variable)
                        }
                        if let Some(ref reference) = value.reference {
                            let (variable, remaining) =
                                ftd::ftd2021::p2::utils::get_doc_name_and_remaining(reference)
                                    .unwrap();
                            if let Some(ftd::Data { dependencies, .. }) = data.get_mut(&variable) {
                                let json = ftd::Dependencies {
                                    dependency_type: ftd::DependencyType::Variable,
                                    condition: None,
                                    remaining,
                                    parameters: std::iter::IntoIterator::into_iter([(
                                        k.to_string(),
                                        ftd::ConditionalValueWithDefault {
                                            value: ftd::ConditionalValue {
                                                value: serde_json::json!({ "$variable$": condition.variable, "$node$": id}),
                                                important: false,
                                                reference: None,
                                            },
                                            default: None,
                                        },
                                    )])
                                        .collect(),
                                };
                                if let Some(dependencies) = dependencies.get_mut("$style$") {
                                    let mut d = serde_json::from_value::<Vec<ftd::Dependencies>>(
                                        dependencies.to_owned(),
                                    )
                                    .unwrap();
                                    d.push(json);
                                    *dependencies = serde_json::to_value(&d).unwrap();
                                } else {
                                    dependencies.insert(
                                        "$style$".to_string(),
                                        serde_json::to_value(vec![json]).unwrap(),
                                    );
                                }
                            }
                        }
                    }
                }
            }
        }

        fn visibility_condition(
            condition: &Option<ftd::Condition>,
            id: &Option<String>,
            data: &mut ftd::DataDependenciesMap,
        ) {
            if let Some(condition) = condition {
                let id = id.clone().expect("universal id should be present");
                let (variable, remaining) =
                    ftd::ftd2021::p2::utils::get_doc_name_and_remaining(&condition.variable)
                        .unwrap();
                if let Some(ftd::Data { dependencies, .. }) = data.get_mut(&variable) {
                    let json = ftd::Dependencies {
                        dependency_type: ftd::DependencyType::Visible,
                        condition: Some(condition.value.to_owned()),
                        parameters: Default::default(),
                        remaining,
                    };
                    if let Some(dependencies) = dependencies.get_mut(&id) {
                        let mut d = serde_json::from_value::<Vec<ftd::Dependencies>>(
                            dependencies.to_owned(),
                        )
                        .unwrap();
                        d.push(json);
                        *dependencies = serde_json::to_value(&d).unwrap();
                    } else {
                        dependencies.insert(id, serde_json::to_value(vec![json]).unwrap());
                    }
                } else {
                    panic!("{} should be declared 2", condition.variable)
                }
            }
        }
    }

    pub fn get_device_dependencies(
        document: &ftd::ftd2021::p2::Document,
        data: &mut ftd::DataDependenciesMap,
    ) {
        let doc = ftd::ftd2021::p2::TDoc {
            name: document.name.as_str(),
            aliases: &document.aliases,
            bag: &document.data,
            local_variables: &mut Default::default(),
            referenced_local_variables: &mut Default::default(),
        };
        for (k, v) in document.data.iter() {
            if !data.contains_key(k) {
                continue;
            }
            let keys = if let ftd::ftd2021::p2::Thing::Variable(ftd::Variable { value, .. }) = v {
                get_ftd_type_variables(value, &doc, k)
            } else {
                continue;
            };
            let dependencies =
                if let Some(ftd::Data { dependencies, .. }) = data.get_mut("ftd#device") {
                    dependencies
                } else {
                    continue;
                };
            let mut device_json = vec![];
            for k in keys {
                let mobile_json = ftd::Dependencies {
                    dependency_type: ftd::DependencyType::Variable,
                    condition: Some(serde_json::Value::String("mobile".to_string())),
                    remaining: None,
                    parameters: std::iter::IntoIterator::into_iter([(
                        k.to_string(),
                        ftd::ConditionalValueWithDefault {
                            value: ConditionalValue {
                                value: serde_json::Value::String("mobile".to_string()),
                                important: false,
                                reference: None,
                            },
                            default: None,
                        },
                    )])
                    .collect(),
                };

                let xl_json = ftd::Dependencies {
                    dependency_type: ftd::DependencyType::Variable,
                    condition: Some(serde_json::Value::String("xl".to_string())),
                    remaining: None,
                    parameters: std::iter::IntoIterator::into_iter([(
                        k.to_string(),
                        ftd::ConditionalValueWithDefault {
                            value: ConditionalValue {
                                value: serde_json::Value::String("xl".to_string()),
                                important: false,
                                reference: None,
                            },
                            default: None,
                        },
                    )])
                    .collect(),
                };

                let desktop_json = ftd::Dependencies {
                    dependency_type: ftd::DependencyType::Variable,
                    condition: Some(serde_json::Value::String("desktop".to_string())),
                    remaining: None,
                    parameters: std::iter::IntoIterator::into_iter([(
                        k.to_string(),
                        ftd::ConditionalValueWithDefault {
                            value: ConditionalValue {
                                value: serde_json::Value::String("desktop".to_string()),
                                important: false,
                                reference: None,
                            },
                            default: None,
                        },
                    )])
                    .collect(),
                };

                device_json.push(mobile_json);
                device_json.push(xl_json);
                device_json.push(desktop_json);
            }

            if let Some(dependencies) = dependencies.get_mut("$value#kind$") {
                let mut d =
                    serde_json::from_value::<Vec<ftd::Dependencies>>(dependencies.to_owned())
                        .unwrap();
                d.extend(device_json);
                *dependencies = serde_json::to_value(&d).unwrap();
            } else {
                dependencies.insert(
                    "$value#kind$".to_string(),
                    serde_json::to_value(&device_json).unwrap(),
                );
            }
        }

        fn get_ftd_type_variables(
            property_value: &ftd::PropertyValue,
            doc: &ftd::ftd2021::p2::TDoc,
            key: &str,
        ) -> Vec<String> {
            match property_value.kind() {
                ftd::ftd2021::p2::Kind::Record { name, .. }
                    if ["ftd#type"].contains(&name.as_str()) =>
                {
                    return vec![key.to_string()];
                }
                ftd::ftd2021::p2::Kind::Record { .. } => {
                    if let Ok(ftd::Value::Record { fields, .. }) = property_value.resolve(0, doc) {
                        let mut reference = vec![];
                        for (k, field) in fields.iter() {
                            reference.extend(get_ftd_type_variables(field, doc, k));
                        }
                        return reference
                            .into_iter()
                            .map(|v| format!("{key}.{v}"))
                            .collect();
                    }
                }
                _ => {}
            }
            vec![]
        }
    }

    pub fn get_dark_mode_dependencies(
        document: &ftd::ftd2021::p2::Document,
        data: &mut ftd::DataDependenciesMap,
    ) {
        let doc = ftd::ftd2021::p2::TDoc {
            name: document.name.as_str(),
            aliases: &document.aliases,
            bag: &document.data,
            local_variables: &mut Default::default(),
            referenced_local_variables: &mut Default::default(),
        };
        for (k, v) in document.data.iter() {
            if !data.contains_key(k) {
                continue;
            }
            let keys = if let ftd::ftd2021::p2::Thing::Variable(ftd::Variable { value, .. }) = v {
                get_ftd_type_variables(value, &doc, k)
            } else {
                continue;
            };
            let dependencies =
                if let Some(ftd::Data { dependencies, .. }) = data.get_mut("ftd#dark-mode") {
                    dependencies
                } else {
                    continue;
                };
            let dark_mode_json = keys
                .iter()
                .map(|k| ftd::Dependencies {
                    dependency_type: ftd::DependencyType::Variable,
                    condition: Some(serde_json::Value::Bool(true)),
                    remaining: None,
                    parameters: std::iter::IntoIterator::into_iter([(
                        k.to_string(),
                        ftd::ConditionalValueWithDefault {
                            value: ConditionalValue {
                                value: serde_json::Value::String("dark".to_string()),
                                important: false,
                                reference: None,
                            },
                            default: Some(ConditionalValue {
                                value: serde_json::Value::String("light".to_string()),
                                important: false,
                                reference: None,
                            }),
                        },
                    )])
                    .collect(),
                })
                .collect::<Vec<ftd::Dependencies>>();

            if let Some(dependencies) = dependencies.get_mut("$value#kind$") {
                let mut d =
                    serde_json::from_value::<Vec<ftd::Dependencies>>(dependencies.to_owned())
                        .unwrap();
                d.extend(dark_mode_json);
                *dependencies = serde_json::to_value(&d).unwrap();
            } else {
                dependencies.insert(
                    "$value#kind$".to_string(),
                    serde_json::to_value(&dark_mode_json).unwrap(),
                );
            }
        }

        fn get_ftd_type_variables(
            property_value: &ftd::PropertyValue,
            doc: &ftd::ftd2021::p2::TDoc,
            key: &str,
        ) -> Vec<String> {
            match property_value.kind() {
                ftd::ftd2021::p2::Kind::Record { name, .. }
                    if ["ftd#image-src", "ftd#color"].contains(&name.as_str()) =>
                {
                    return vec![key.to_string()];
                }
                ftd::ftd2021::p2::Kind::Record { .. } => {
                    if let Ok(ftd::Value::Record { fields, .. }) = property_value.resolve(0, doc) {
                        let mut reference = vec![];
                        for (k, field) in fields.iter() {
                            reference.extend(get_ftd_type_variables(field, doc, k));
                        }
                        return reference
                            .into_iter()
                            .map(|v| format!("{key}.{v}"))
                            .collect();
                    }
                }
                _ => {}
            }
            vec![]
        }
    }

    pub fn get_variable_dependencies(
        document: &ftd::ftd2021::p2::Document,
        data: &mut ftd::DataDependenciesMap,
    ) {
        let doc = ftd::ftd2021::p2::TDoc {
            name: document.name.as_str(),
            aliases: &document.aliases,
            bag: &document.data,
            local_variables: &mut Default::default(),
            referenced_local_variables: &mut Default::default(),
        };
        for (k, v) in document.data.iter() {
            if !data.contains_key(k) {
                continue;
            }
            let (conditions, default) = if let ftd::ftd2021::p2::Thing::Variable(ftd::Variable {
                conditions,
                value: default,
                ..
            }) = v
            {
                (conditions, default)
            } else {
                continue;
            };
            let default = match default.resolve(0, &doc) {
                Ok(v) => v,
                _ => continue,
            };
            for (condition, value) in conditions {
                let condition = if let Ok(condition) = condition.to_condition(
                    0,
                    &ftd::ftd2021::p2::TDoc {
                        name: document.name.as_str(),
                        aliases: &document.aliases,
                        bag: &document.data,
                        local_variables: &mut Default::default(),
                        referenced_local_variables: &mut Default::default(),
                    },
                ) {
                    condition
                } else {
                    continue;
                };
                let value = match value.resolve(0, &doc) {
                    Ok(value) => match value.to_serde_value() {
                        Some(v) => v,
                        None => continue,
                    },
                    _ => continue,
                };

                let (variable, remaining) =
                    ftd::ftd2021::p2::utils::get_doc_name_and_remaining(&condition.variable)
                        .unwrap();
                let dependencies =
                    if let Some(ftd::Data { dependencies, .. }) = data.get_mut(&variable) {
                        dependencies
                    } else {
                        continue;
                    };
                let json = ftd::Dependencies {
                    dependency_type: ftd::DependencyType::Variable,
                    condition: Some(condition.value.to_owned()),
                    remaining,
                    parameters: std::iter::IntoIterator::into_iter([(
                        k.to_string(),
                        ftd::ConditionalValueWithDefault {
                            value: ConditionalValue {
                                value,
                                important: false,
                                reference: None,
                            },
                            default: default.to_serde_value().map(|value| ConditionalValue {
                                value,
                                important: false,
                                reference: None,
                            }),
                        },
                    )])
                    .collect(),
                };
                if let Some(dependencies) = dependencies.get_mut("$value$") {
                    let mut d =
                        serde_json::from_value::<Vec<ftd::Dependencies>>(dependencies.to_owned())
                            .unwrap();
                    d.push(json);
                    *dependencies = serde_json::to_value(&d).unwrap();
                } else {
                    dependencies.insert(
                        "$value$".to_string(),
                        serde_json::to_value(vec![json]).unwrap(),
                    );
                }
            }
        }
    }

    pub fn is_open_container(&self, is_container_children_empty: bool) -> bool {
        match self {
            ftd::Element::Column(e) => e.container.is_open(is_container_children_empty),
            ftd::Element::Row(e) => e.container.is_open(is_container_children_empty),
            ftd::Element::Scene(e) => e.container.is_open(is_container_children_empty),
            ftd::Element::Grid(e) => e.container.is_open(is_container_children_empty),
            _ => false,
        }
    }

    pub fn append_at(&self) -> Option<String> {
        match self {
            ftd::Element::Column(e) => e.container.append_at.to_owned(),
            ftd::Element::Row(e) => e.container.append_at.to_owned(),
            ftd::Element::Scene(e) => e.container.append_at.to_owned(),
            ftd::Element::Grid(e) => e.container.append_at.to_owned(),
            _ => None,
        }
    }

    pub fn number_of_children(&self) -> usize {
        match self {
            ftd::Element::Column(e) => e.container.children.len(),
            ftd::Element::Row(e) => e.container.children.len(),
            ftd::Element::Scene(e) => e.container.children.len(),
            ftd::Element::Grid(e) => e.container.children.len(),
            _ => 0,
        }
    }

    pub fn container_id(&self) -> Option<String> {
        match self {
            ftd::Element::Column(e) => e.common.data_id.clone(),
            ftd::Element::Row(e) => e.common.data_id.clone(),
            ftd::Element::Scene(e) => e.common.data_id.clone(),
            ftd::Element::Grid(e) => e.common.data_id.clone(),
            _ => None,
        }
    }

    pub fn set_container_id(&mut self, name: Option<String>) {
        match self {
            ftd::Element::Column(e) => e.common.data_id = name,
            ftd::Element::Row(e) => e.common.data_id = name,
            ftd::Element::Scene(e) => e.common.data_id = name,
            ftd::Element::Grid(e) => e.common.data_id = name,
            _ => {}
        }
    }

    pub fn set_element_id(&mut self, name: Option<String>) {
        match self {
            ftd::Element::Column(ftd::Column { common, .. })
            | ftd::Element::Row(ftd::Row { common, .. })
            | ftd::Element::TextBlock(ftd::TextBlock { common, .. })
            | ftd::Element::Code(ftd::Code { common, .. })
            | ftd::Element::Image(ftd::Image { common, .. })
            | ftd::Element::IFrame(ftd::IFrame { common, .. })
            | ftd::Element::Markup(ftd::Markups { common, .. })
            | ftd::Element::Input(ftd::Input { common, .. })
            | ftd::Element::Integer(ftd::Text { common, .. })
            | ftd::Element::Boolean(ftd::Text { common, .. })
            | ftd::Element::Decimal(ftd::Text { common, .. })
            | ftd::Element::Scene(ftd::Scene { common, .. })
            | ftd::Element::Grid(ftd::Grid { common, .. }) => common.id = name,
            ftd::Element::Null => {}
        }
    }

    pub fn set_condition(&mut self, condition: Option<ftd::Condition>) {
        match self {
            ftd::Element::Column(ftd::Column { common, .. })
            | ftd::Element::Row(ftd::Row { common, .. })
            | ftd::Element::TextBlock(ftd::TextBlock { common, .. })
            | ftd::Element::Code(ftd::Code { common, .. })
            | ftd::Element::Image(ftd::Image { common, .. })
            | ftd::Element::IFrame(ftd::IFrame { common, .. })
            | ftd::Element::Markup(ftd::Markups { common, .. })
            | ftd::Element::Input(ftd::Input { common, .. })
            | ftd::Element::Integer(ftd::Text { common, .. })
            | ftd::Element::Boolean(ftd::Text { common, .. })
            | ftd::Element::Decimal(ftd::Text { common, .. })
            | ftd::Element::Scene(ftd::Scene { common, .. })
            | ftd::Element::Grid(ftd::Grid { common, .. }) => common,
            ftd::Element::Null => return,
        }
        .condition = condition;
    }

    pub fn set_non_visibility(&mut self, is_not_visible: bool) {
        match self {
            ftd::Element::Column(ftd::Column { common, .. })
            | ftd::Element::Row(ftd::Row { common, .. })
            | ftd::Element::TextBlock(ftd::TextBlock { common, .. })
            | ftd::Element::Code(ftd::Code { common, .. })
            | ftd::Element::Image(ftd::Image { common, .. })
            | ftd::Element::IFrame(ftd::IFrame { common, .. })
            | ftd::Element::Markup(ftd::Markups { common, .. })
            | ftd::Element::Input(ftd::Input { common, .. })
            | ftd::Element::Integer(ftd::Text { common, .. })
            | ftd::Element::Boolean(ftd::Text { common, .. })
            | ftd::Element::Decimal(ftd::Text { common, .. })
            | ftd::Element::Scene(ftd::Scene { common, .. })
            | ftd::Element::Grid(ftd::Grid { common, .. }) => common,
            ftd::Element::Null => return,
        }
        .is_not_visible = is_not_visible;
    }

    pub fn set_events(&mut self, events: &mut Vec<ftd::Event>) {
        match self {
            ftd::Element::Column(ftd::Column { common, .. })
            | ftd::Element::Row(ftd::Row { common, .. })
            | ftd::Element::TextBlock(ftd::TextBlock { common, .. })
            | ftd::Element::Code(ftd::Code { common, .. })
            | ftd::Element::Image(ftd::Image { common, .. })
            | ftd::Element::IFrame(ftd::IFrame { common, .. })
            | ftd::Element::Markup(ftd::Markups { common, .. })
            | ftd::Element::Input(ftd::Input { common, .. })
            | ftd::Element::Integer(ftd::Text { common, .. })
            | ftd::Element::Boolean(ftd::Text { common, .. })
            | ftd::Element::Decimal(ftd::Text { common, .. })
            | ftd::Element::Scene(ftd::Scene { common, .. })
            | ftd::Element::Grid(ftd::Grid { common, .. }) => common,
            ftd::Element::Null => return,
        }
        .events
        .append(events)
    }

    pub fn get_heading_region(&self) -> Option<&ftd::Region> {
        match self {
            ftd::Element::Column(e) => e.common.region.as_ref().filter(|v| v.is_heading()),
            ftd::Element::Row(e) => e.common.region.as_ref().filter(|v| v.is_heading()),
            _ => None,
        }
    }

    pub fn get_mut_common(&mut self) -> Option<&mut ftd::Common> {
        match self {
            ftd::Element::Column(e) => Some(&mut e.common),
            ftd::Element::Row(e) => Some(&mut e.common),
            ftd::Element::Markup(e) => Some(&mut e.common),
            ftd::Element::TextBlock(e) => Some(&mut e.common),
            ftd::Element::Code(e) => Some(&mut e.common),
            ftd::Element::Image(e) => Some(&mut e.common),
            ftd::Element::IFrame(e) => Some(&mut e.common),
            ftd::Element::Input(e) => Some(&mut e.common),
            ftd::Element::Integer(e) => Some(&mut e.common),
            ftd::Element::Boolean(e) => Some(&mut e.common),
            ftd::Element::Decimal(e) => Some(&mut e.common),
            ftd::Element::Scene(e) => Some(&mut e.common),
            ftd::Element::Grid(e) => Some(&mut e.common),
            ftd::Element::Null => None,
        }
    }

    pub fn get_common(&self) -> Option<&ftd::Common> {
        match self {
            ftd::Element::Column(e) => Some(&e.common),
            ftd::Element::Row(e) => Some(&e.common),
            ftd::Element::Markup(e) => Some(&e.common),
            ftd::Element::TextBlock(e) => Some(&e.common),
            ftd::Element::Code(e) => Some(&e.common),
            ftd::Element::Image(e) => Some(&e.common),
            ftd::Element::IFrame(e) => Some(&e.common),
            ftd::Element::Input(e) => Some(&e.common),
            ftd::Element::Integer(e) => Some(&e.common),
            ftd::Element::Boolean(e) => Some(&e.common),
            ftd::Element::Decimal(e) => Some(&e.common),
            ftd::Element::Scene(e) => Some(&e.common),
            ftd::Element::Grid(e) => Some(&e.common),
            ftd::Element::Null => None,
        }
    }

    pub fn get_container(&self) -> Option<&ftd::Container> {
        match self {
            ftd::Element::Column(e) => Some(&e.container),
            ftd::Element::Row(e) => Some(&e.container),
            ftd::Element::Scene(e) => Some(&e.container),
            ftd::Element::Grid(e) => Some(&e.container),
            _ => None,
        }
    }

    pub fn renest_on_region(elements: &mut Vec<ftd::Element>) {
        let mut region: Option<(usize, &Region)> = None;
        let mut insert: Vec<(usize, usize)> = Default::default();
        for (idx, element) in elements.iter().enumerate() {
            match element {
                ftd::Element::Column(ftd::Column { common, .. })
                | ftd::Element::Row(ftd::Row { common, .. }) => {
                    let r = common.region.as_ref().filter(|v| v.is_heading());
                    if let Some(r) = r {
                        if let Some((place_at, r1)) = region {
                            if r.get_lower_priority_heading().contains(r1) || r == r1 {
                                insert.push((place_at, idx));
                                region = Some((idx, r));
                            }
                        } else {
                            region = Some((idx, r));
                        }
                    }
                }
                _ => continue,
            }
        }
        if let Some((place_at, _)) = region {
            insert.push((place_at, elements.len()));
        }

        for (place_at, end) in insert.iter().rev() {
            let mut children = elements[place_at + 1..*end].to_vec();
            if children.is_empty() {
                continue;
            }
            match elements[*place_at] {
                ftd::Element::Column(ftd::Column {
                    ref mut container, ..
                })
                | ftd::Element::Row(ftd::Row {
                    ref mut container, ..
                }) => {
                    if let Some(ref id) = container.append_at {
                        match &mut container.external_children {
                            Some((_, _, e)) => {
                                if let Some(ftd::Element::Column(col)) = e.first_mut() {
                                    col.container.children.extend(children);
                                } else {
                                    let mut main = ftd::ftd2021::p2::interpreter::default_column();
                                    main.container.children.extend(children);
                                    e.push(ftd::Element::Column(main))
                                }
                            }
                            _ => panic!("{id} has no external_children data"),
                        }
                    } else {
                        container.children.append(&mut children);
                    }
                }
                _ => continue,
            }
            for idx in (place_at + 1..*end).rev() {
                elements.remove(idx);
            }
        }

        for element in &mut *elements {
            match element {
                ftd::Element::Column(ftd::Column { container, .. })
                | ftd::Element::Row(ftd::Row { container, .. }) => {
                    if let Some((_, _, ref mut e)) = container.external_children {
                        ftd::Element::renest_on_region(e);
                    }
                    ftd::Element::renest_on_region(&mut container.children);
                }
                _ => continue,
            }
        }
    }
}

#[derive(serde::Deserialize, PartialEq, Debug, Clone, serde::Serialize)]
#[serde(tag = "type")]
pub enum Length {
    Fill,
    Shrink,
    Auto,
    FitContent,
    Px { value: i64 },
    Portion { value: i64 },
    Percent { value: i64 },
    Calc { value: String },
    VH { value: i64 },
    VW { value: i64 },
    VMIN { value: i64 },
    VMAX { value: i64 },
}

impl Length {
    pub fn from(l: Option<String>, doc_id: &str) -> ftd::ftd2021::p1::Result<Option<ftd::Length>> {
        let l = match l {
            Some(l) => l,
            None => return Ok(None),
        };

        if l == "fill" {
            return Ok(Some(Length::Fill));
        }

        if l == "shrink" {
            return Ok(Some(Length::Shrink));
        }
        if l == "auto" {
            return Ok(Some(Length::Auto));
        }

        if l.starts_with("calc ") {
            let v = ftd::ftd2021::p2::utils::get_name("calc", l.as_str(), doc_id)?;
            return match v.parse() {
                Ok(v) => Ok(Some(Length::Calc { value: v })),
                Err(_) => {
                    ftd::ftd2021::p2::utils::e2(format!("{v} is not a valid integer"), doc_id, 0)
                } // TODO
            };
        }

        if l == "fit-content" {
            return Ok(Some(Length::FitContent));
        }

        if l.starts_with("portion ") {
            let v = ftd::ftd2021::p2::utils::get_name("portion", l.as_str(), doc_id)?;
            return match v.parse() {
                Ok(v) => Ok(Some(Length::Portion { value: v })),
                Err(_) => {
                    ftd::ftd2021::p2::utils::e2(format!("{v} is not a valid integer"), doc_id, 0)
                } // TODO
            };
        }
        if l.starts_with("percent ") {
            let v = ftd::ftd2021::p2::utils::get_name("percent", l.as_str(), doc_id)?;
            return match v.parse() {
                Ok(v) => Ok(Some(Length::Percent { value: v })),
                Err(_) => {
                    ftd::ftd2021::p2::utils::e2(format!("{v} is not a valid integer"), doc_id, 0)
                } // TODO
            };
        }
        if l.starts_with("vh ") {
            let v = ftd::ftd2021::p2::utils::get_name("vh", l.as_str(), doc_id)?;
            return match v.parse() {
                Ok(v) => Ok(Some(Length::VH { value: v })),
                Err(_) => {
                    ftd::ftd2021::p2::utils::e2(format!("{v} is not a valid integer"), doc_id, 0)
                } // TODO
            };
        }
        if l.starts_with("vw ") {
            let v = ftd::ftd2021::p2::utils::get_name("vw", l.as_str(), doc_id)?;
            return match v.parse() {
                Ok(v) => Ok(Some(Length::VW { value: v })),
                Err(_) => {
                    ftd::ftd2021::p2::utils::e2(format!("{v} is not a valid integer"), doc_id, 0)
                } // TODO
            };
        }

        if l.starts_with("vmin ") {
            let v = ftd::ftd2021::p2::utils::get_name("vmin", l.as_str(), doc_id)?;
            return match v.parse() {
                Ok(v) => Ok(Some(Length::VMIN { value: v })),
                Err(_) => {
                    ftd::ftd2021::p2::utils::e2(format!("{v} is not a valid integer"), doc_id, 0)
                } // TODO
            };
        }

        if l.starts_with("vmax ") {
            let v = ftd::ftd2021::p2::utils::get_name("vmax", l.as_str(), doc_id)?;
            return match v.parse() {
                Ok(v) => Ok(Some(Length::VMAX { value: v })),
                Err(_) => {
                    ftd::ftd2021::p2::utils::e2(format!("{v} is not a valid integer"), doc_id, 0)
                } // TODO
            };
        }

        match l.parse() {
            Ok(v) => Ok(Some(Length::Px { value: v })),
            Err(_) => ftd::ftd2021::p2::utils::e2(format!("{l} is not a valid integer"), doc_id, 0),
        }
    }
}

#[derive(serde::Deserialize, Debug, PartialEq, Clone, serde::Serialize)]
#[serde(tag = "type")]
pub enum Position {
    Center,
    Top,
    Bottom,
    Left,
    Right,
    TopLeft,
    TopRight,
    BottomLeft,
    BottomRight,
}

impl Default for Position {
    fn default() -> ftd::Position {
        Self::TopLeft
    }
}

impl Position {
    pub fn from(
        l: Option<String>,
        doc_id: &str,
    ) -> ftd::ftd2021::p1::Result<Option<ftd::Position>> {
        Ok(match l.as_deref() {
            Some("center") => Some(Self::Center),
            Some("top") => Some(Self::Top),
            Some("bottom") => Some(Self::Bottom),
            Some("left") => Some(Self::Left),
            Some("right") => Some(Self::Right),
            Some("top-left") => Some(Self::TopLeft),
            Some("top-right") => Some(Self::TopRight),
            Some("bottom-left") => Some(Self::BottomLeft),
            Some("bottom-right") => Some(Self::BottomRight),
            Some(t) => {
                return ftd::ftd2021::p2::utils::e2(
                    format!("{t} is not a valid alignment"),
                    doc_id,
                    0,
                );
            } // TODO
            None => None,
        })
    }
}

// https://package.elm-lang.org/packages/mdgriffith/elm-ui/latest/Element-Region
#[derive(serde::Deserialize, Debug, PartialEq, Clone, serde::Serialize)]
pub enum Region {
    H0,
    H1,
    H2,
    H3,
    H4,
    H5,
    H6,
    H7,
    Title,
    MainContent,
    Navigation,
    Aside,
    Footer,
    Description,
    Announce,
    AnnounceUrgently,
}

impl Display for Region {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let str = match self {
            Self::H0 => "h0",
            Self::H1 => "h1",
            Self::H2 => "h2",
            Self::H3 => "h3",
            Self::H4 => "h4",
            Self::H5 => "h5",
            Self::H6 => "h6",
            Self::H7 => "h7",
            Self::Title => "title",
            Self::MainContent => "main",
            Self::Navigation => "navigation",
            Self::Aside => "aside",
            Self::Footer => "footer",
            Self::Description => "description",
            Self::Announce => "announce",
            Self::AnnounceUrgently => "announce-urgently",
        }
        .to_string();
        write!(f, "{str}")
    }
}

impl Region {
    pub fn from(l: Option<String>, doc_id: &str) -> ftd::ftd2021::p1::Result<Option<ftd::Region>> {
        Ok(Some(match l.as_deref() {
            Some("h0") => Self::H0,
            Some("h1") => Self::H1,
            Some("h2") => Self::H2,
            Some("h3") => Self::H3,
            Some("h4") => Self::H4,
            Some("h5") => Self::H5,
            Some("h6") => Self::H6,
            Some("h7") => Self::H7,
            Some("title") => Self::Title,
            Some("main") => Self::MainContent,
            Some("navigation") => Self::Navigation,
            Some("aside") => Self::Aside,
            Some("footer") => Self::Footer,
            Some("description") => Self::Description,
            Some("announce") => Self::Announce,
            Some("announce-urgently") => Self::AnnounceUrgently,
            Some(t) => {
                return ftd::ftd2021::p2::utils::e2(
                    format!("{t} is not a valid alignment"),
                    doc_id,
                    0,
                );
            } // TODO
            None => return Ok(None),
        }))
    }

    pub fn is_heading(&self) -> bool {
        matches!(
            self,
            ftd::Region::H0
                | ftd::Region::H1
                | ftd::Region::H2
                | ftd::Region::H3
                | ftd::Region::H4
                | ftd::Region::H5
                | ftd::Region::H6
                | ftd::Region::H7
        )
    }

    /// returns heading priority value based on heading size
    ///
    /// Priority Order of Headings
    ///
    /// h0 > h1 > h2 > h3 > h4 > h5 > h6 > h7
    ///
    /// will throw error if tried to compute heading priority
    /// of any non-heading region
    pub fn heading_priority_value(&self, doc_id: &str) -> ftd::ftd2021::p1::Result<i32> {
        match self {
            Self::H0 => Ok(0),
            Self::H1 => Ok(-1),
            Self::H2 => Ok(-2),
            Self::H3 => Ok(-3),
            Self::H4 => Ok(-4),
            Self::H5 => Ok(-5),
            Self::H6 => Ok(-6),
            Self::H7 => Ok(-7),
            _ => ftd::ftd2021::p2::utils::e2(
                format!("{self} is not a valid heading region"),
                doc_id,
                0,
            ),
        }
    }

    pub fn is_primary_heading(&self) -> bool {
        matches!(self, ftd::Region::H0 | ftd::Region::H1)
    }

    pub fn is_title(&self) -> bool {
        matches!(self, ftd::Region::Title)
    }

    pub fn get_lower_priority_heading(&self) -> Vec<ftd::Region> {
        let mut list = vec![];
        if matches!(
            self,
            ftd::Region::Title
                | ftd::Region::MainContent
                | ftd::Region::Navigation
                | ftd::Region::Aside
                | ftd::Region::Footer
                | ftd::Region::Description
                | ftd::Region::Announce
                | ftd::Region::AnnounceUrgently
        ) {
            return list;
        }
        for region in [
            ftd::Region::H7,
            ftd::Region::H6,
            ftd::Region::H5,
            ftd::Region::H4,
            ftd::Region::H3,
            ftd::Region::H2,
            ftd::Region::H1,
        ] {
            if self.to_string() == region.to_string() {
                return list;
            }
            list.push(region);
        }
        list
    }
}

#[derive(serde::Deserialize, Debug, PartialEq, Clone, serde::Serialize)]
#[serde(tag = "type")]
pub enum Overflow {
    Hidden,
    Visible,
    Auto,
    Scroll,
}

impl Overflow {
    pub fn from(
        l: Option<String>,
        doc_id: &str,
    ) -> ftd::ftd2021::p1::Result<Option<ftd::Overflow>> {
        Ok(Option::from(match l.as_deref() {
            Some("hidden") => Self::Hidden,
            Some("visible") => Self::Visible,
            Some("auto") => Self::Auto,
            Some("scroll") => Self::Scroll,
            Some(t) => {
                return ftd::ftd2021::p2::utils::e2(
                    format!("{t} is not a valid property"),
                    doc_id,
                    0,
                );
            } // TODO
            None => return Ok(None),
        }))
    }
}

#[derive(serde::Deserialize, Debug, PartialEq, Clone, serde::Serialize)]
#[serde(tag = "type")]
pub enum Anchor {
    Window,
    Parent,
}

impl Anchor {
    pub fn from(l: Option<String>, doc_id: &str) -> ftd::ftd2021::p1::Result<Option<ftd::Anchor>> {
        let l = match l {
            Some(l) => l,
            None => return Ok(None),
        };

        Ok(Some(match l.as_str() {
            "window" => ftd::Anchor::Window,
            "parent" => ftd::Anchor::Parent,
            t => {
                return ftd::ftd2021::p2::utils::e2(
                    format!(
                        "invalid value for `absolute` expected `window` or `parent` found: {t}"
                    ),
                    doc_id,
                    0, // TODO
                );
            }
        }))
    }

    pub fn to_position(&self) -> String {
        match self {
            ftd::Anchor::Window => "fixed",
            ftd::Anchor::Parent => "absolute",
        }
        .to_string()
    }
}

#[derive(serde::Deserialize, Debug, PartialEq, Clone, serde::Serialize)]
#[serde(tag = "type")]
pub enum GradientDirection {
    BottomToTop,
    TopToBottom,
    RightToLeft,
    LeftToRight,
    BottomRightToTopLeft,
    BottomLeftToTopRight,
    TopRightToBottomLeft,
    TopLeftBottomRight,
    Center,
    Angle { value: i64 },
}

impl GradientDirection {
    pub fn from(
        l: Option<String>,
        doc_id: &str,
    ) -> ftd::ftd2021::p1::Result<Option<ftd::GradientDirection>> {
        let l = match l {
            Some(l) => l,
            None => return Ok(None),
        };

        if l == "bottom to top" {
            return Ok(Some(GradientDirection::BottomToTop));
        }
        if l == "top to bottom" {
            return Ok(Some(GradientDirection::TopToBottom));
        }
        if l == "right to left" {
            return Ok(Some(GradientDirection::RightToLeft));
        }
        if l == "left to right" {
            return Ok(Some(GradientDirection::LeftToRight));
        }
        if l == "bottom-left to top-right" {
            return Ok(Some(GradientDirection::BottomLeftToTopRight));
        }
        if l == "bottom-right to top-left" {
            return Ok(Some(GradientDirection::BottomRightToTopLeft));
        }
        if l == "top-right to bottom-left" {
            return Ok(Some(GradientDirection::TopRightToBottomLeft));
        }
        if l == "top-left to bottom-right" {
            return Ok(Some(GradientDirection::TopLeftBottomRight));
        }
        if l == "center" {
            return Ok(Some(GradientDirection::Center));
        }
        if l.starts_with("angle ") {
            let v = ftd::ftd2021::p2::utils::get_name("angle", l.as_str(), doc_id)?;
            return match v.parse() {
                Ok(v) => Ok(Some(GradientDirection::Angle { value: v })),
                Err(_) => {
                    ftd::ftd2021::p2::utils::e2(format!("{v} is not a valid integer"), doc_id, 0)
                }
            };
        }
        Ok(None)
    }
}

#[derive(serde::Deserialize, Debug, PartialEq, Clone, serde::Serialize)]
pub enum AttributeType {
    Style,
    Attribute,
}

#[derive(serde::Deserialize, Debug, PartialEq, Clone, serde::Serialize)]
pub struct ConditionalAttribute {
    pub attribute_type: AttributeType,
    pub conditions_with_value: Vec<(ftd::Condition, ConditionalValue)>,
    pub default: Option<ConditionalValue>,
}

#[derive(serde::Deserialize, Debug, PartialEq, Clone, serde::Serialize, Default)]
pub struct ConditionalValue {
    pub value: serde_json::Value,
    pub important: bool,
    pub reference: Option<String>,
}

#[derive(serde::Deserialize, Debug, PartialEq, Default, Clone, serde::Serialize)]
pub struct Common {
    pub conditional_attribute: ftd::Map<ConditionalAttribute>,
    pub condition: Option<ftd::Condition>,
    pub is_not_visible: bool,
    pub is_dummy: bool,
    pub events: Vec<ftd::Event>,
    pub reference: Option<String>,
    pub region: Option<Region>,
    pub classes: Option<String>,
    pub padding: Option<i64>,
    pub padding_vertical: Option<i64>,
    pub padding_horizontal: Option<i64>,
    pub padding_left: Option<i64>,
    pub padding_right: Option<i64>,
    pub padding_top: Option<i64>,
    pub padding_bottom: Option<i64>,
    pub border_top_radius: Option<i64>,
    pub border_bottom_radius: Option<i64>,
    pub border_left_radius: Option<i64>,
    pub border_right_radius: Option<i64>,
    pub width: Option<Length>,
    pub max_width: Option<Length>,
    pub min_width: Option<Length>,
    pub height: Option<Length>,
    pub min_height: Option<Length>,
    pub max_height: Option<Length>,
    pub color: Option<Color>,
    pub background_color: Option<Color>,
    pub border_color: Option<Color>,
    pub border_width: i64,
    pub border_radius: i64,
    pub id: Option<String>,
    pub data_id: Option<String>,
    pub overflow_x: Option<Overflow>,
    pub overflow_y: Option<Overflow>,
    pub border_top: Option<i64>,
    pub border_left: Option<i64>,
    pub border_right: Option<i64>,
    pub border_bottom: Option<i64>,
    pub border_top_color: Option<Color>,
    pub border_bottom_color: Option<Color>,
    pub border_left_color: Option<Color>,
    pub border_right_color: Option<Color>,
    pub margin_top: Option<i64>,
    pub margin_left: Option<i64>,
    pub margin_right: Option<i64>,
    pub margin_bottom: Option<i64>,
    pub link: Option<String>,
    pub open_in_new_tab: bool,
    pub sticky: bool,
    pub top: Option<i64>,
    pub bottom: Option<i64>,
    pub left: Option<i64>,
    pub right: Option<i64>,
    pub submit: Option<String>,
    pub cursor: Option<String>,
    pub shadow_offset_x: Option<i64>,
    pub shadow_offset_y: Option<i64>,
    pub shadow_size: Option<i64>,
    pub shadow_blur: Option<i64>,
    pub shadow_color: Option<Color>,
    pub anchor: Option<ftd::Anchor>,
    pub gradient_direction: Option<GradientDirection>,
    pub gradient_colors: Vec<ColorValue>,
    pub background_image: Option<ImageSrc>,
    pub background_repeat: bool,
    pub background_parallax: bool,
    pub scale: Option<f64>,
    pub scale_x: Option<f64>,
    pub scale_y: Option<f64>,
    pub rotate: Option<i64>,
    pub move_up: Option<i64>,
    pub move_down: Option<i64>,
    pub move_left: Option<i64>,
    pub move_right: Option<i64>,
    pub position: Option<Position>,
    pub inner: bool,
    pub z_index: Option<i64>,
    pub slot: Option<String>,
    pub grid_column: Option<String>,
    pub grid_row: Option<String>,
    pub white_space: Option<String>,
    pub border_style: Option<String>,
    pub text_transform: Option<String>,
    pub title: Option<String>,
    pub heading_number: Option<Vec<String>>,
    // TODO: background-image, un-cropped, tiled, tiled{X, Y}
    // TODO: border-style: solid, dashed, dotted
    // TODO: border-{shadow, glow}
}

#[derive(serde::Deserialize, Debug, PartialEq, Clone, serde::Serialize)]
#[serde(tag = "type")]
pub enum Spacing {
    SpaceEvenly,
    SpaceBetween,
    SpaceAround,
    Absolute { value: String },
}

impl Spacing {
    pub fn from(l: Option<String>) -> ftd::ftd2021::p1::Result<Option<ftd::Spacing>> {
        Ok(match l.as_deref() {
            Some("space-evenly") => Some(ftd::Spacing::SpaceEvenly),
            Some("space-between") => Some(ftd::Spacing::SpaceBetween),
            Some("space-around") => Some(ftd::Spacing::SpaceAround),
            Some(t) => Some(ftd::Spacing::Absolute {
                value: t.to_string(),
            }),
            None => return Ok(None),
        })
    }
}

#[derive(serde::Deserialize, Debug, PartialEq, Default, Clone, serde::Serialize)]
pub struct Container {
    pub children: Vec<ftd::Element>,
    pub external_children: Option<(String, Vec<Vec<usize>>, Vec<ftd::Element>)>,
    pub open: Option<bool>,
    pub append_at: Option<String>,
    pub wrap: bool,
}

impl Container {
    pub fn is_open(&self, is_container_children_empty: bool) -> bool {
        self.open
            .unwrap_or(self.children.is_empty() && is_container_children_empty)
    }
}

/// https://html.spec.whatwg.org/multipage/urls-and-fetching.html#lazy-loading-attributes
#[derive(serde::Deserialize, Debug, Default, PartialEq, Clone, serde::Serialize)]
#[serde(tag = "type")]
pub enum Loading {
    #[default]
    Lazy,
    Eager,
}

impl Loading {
    pub fn from(s: &str, doc_id: &str) -> ftd::ftd2021::p1::Result<Loading> {
        match s {
            "lazy" => Ok(Loading::Lazy),
            "eager" => Ok(Loading::Eager),
            _ => ftd::ftd2021::p2::utils::e2(
                format!("{s} is not a valid alignment, allowed: lazy, eager"),
                doc_id,
                0,
            ),
        }
    }

    pub fn to_html(&self) -> &'static str {
        match self {
            Loading::Lazy => "lazy",
            Loading::Eager => "eager",
        }
    }
}

#[derive(serde::Deserialize, Debug, Default, PartialEq, Clone, serde::Serialize)]
pub struct Image {
    pub src: ImageSrc,
    pub description: Option<String>,
    pub common: Box<Common>,
    pub crop: bool,
    /// images can load lazily.
    pub loading: Loading,
}

#[derive(serde::Deserialize, Debug, Default, PartialEq, Clone, serde::Serialize)]
pub struct Row {
    pub container: Container,
    pub spacing: Option<Spacing>,
    pub common: Box<Common>,
}

#[derive(serde::Deserialize, Debug, Default, PartialEq, Clone, serde::Serialize)]
pub struct Scene {
    pub container: Container,
    pub spacing: Option<Spacing>,
    pub common: Box<Common>,
}

#[derive(serde::Deserialize, Debug, Default, PartialEq, Clone, serde::Serialize)]
pub struct Grid {
    pub slots: String,
    pub slot_widths: Option<String>,
    pub slot_heights: Option<String>,
    pub spacing: Option<i64>,
    pub spacing_vertical: Option<i64>,
    pub spacing_horizontal: Option<i64>,
    pub inline: bool,
    pub auto_flow: Option<String>,
    pub container: Container,
    pub common: Box<Common>,
}

#[derive(serde::Deserialize, Debug, PartialEq, Clone, Default, serde::Serialize)]
pub struct Column {
    pub container: Container,
    pub spacing: Option<Spacing>,
    pub common: Box<Common>,
}

#[derive(serde::Deserialize, Default, Debug, PartialEq, Clone, serde::Serialize)]
#[serde(tag = "type")]
pub enum TextAlign {
    #[default]
    Left,
    Right,
    Center,
    Justify,
}

impl TextAlign {
    pub fn from(l: Option<String>, doc_id: &str) -> ftd::ftd2021::p1::Result<ftd::TextAlign> {
        Ok(match l.as_deref() {
            Some("center") => ftd::TextAlign::Center,
            Some("left") => ftd::TextAlign::Left,
            Some("right") => ftd::TextAlign::Right,
            Some("justify") => ftd::TextAlign::Justify,
            Some(t) => {
                return ftd::ftd2021::p2::utils::e2(
                    format!("{t} is not a valid alignment, allowed: center, left, right, justify"),
                    doc_id,
                    0,
                );
            }
            None => return Ok(ftd::TextAlign::Left),
        })
    }
}

#[derive(serde::Deserialize, Default, Debug, PartialEq, Clone, serde::Serialize)]
#[serde(tag = "type")]
pub enum FontDisplay {
    Swap,
    #[default]
    Block,
}

impl FontDisplay {
    pub fn from(l: Option<String>, doc_id: &str) -> ftd::ftd2021::p1::Result<ftd::FontDisplay> {
        Ok(match l.as_deref() {
            Some("swap") => ftd::FontDisplay::Swap,
            Some("block") => ftd::FontDisplay::Block,
            Some(t) => {
                return ftd::ftd2021::p2::utils::e2(
                    format!("{t} is not a valid alignment, allowed: swap, block"),
                    doc_id,
                    0,
                );
            } // TODO
            None => return Ok(ftd::FontDisplay::Block),
        })
    }
}

#[derive(serde::Deserialize, Debug, Default, PartialEq, Clone, serde::Serialize)]
pub struct ImageSrc {
    pub light: String,
    pub dark: String,
    pub reference: Option<String>,
}

impl ImageSrc {
    pub fn from(
        l: &ftd::Map<ftd::PropertyValue>,
        doc: &ftd::ftd2021::p2::TDoc,
        line_number: usize,
        reference: Option<String>,
    ) -> ftd::ftd2021::p1::Result<ImageSrc> {
        let properties = l
            .iter()
            .map(|(k, v)| v.resolve(line_number, doc).map(|v| (k.to_string(), v)))
            .collect::<ftd::ftd2021::p1::Result<ftd::Map<ftd::Value>>>()?;
        Ok(ImageSrc {
            light: ftd::ftd2021::p2::utils::string_optional("light", &properties, doc.name, 0)?
                .unwrap_or_default(),
            dark: ftd::ftd2021::p2::utils::string_optional("dark", &properties, doc.name, 0)?
                .unwrap_or_default(),
            reference,
        })
    }
}

#[derive(serde::Deserialize, Debug, PartialEq, Clone, serde::Serialize)]
pub struct FontSize {
    pub line_height: i64,
    #[serde(rename = "font-size")]
    pub size: i64,
    pub letter_spacing: i64,
    pub reference: Option<String>,
}

impl FontSize {
    pub fn from(
        l: &ftd::Map<ftd::PropertyValue>,
        doc: &ftd::ftd2021::p2::TDoc,
        line_number: usize,
        reference: Option<String>,
    ) -> ftd::ftd2021::p1::Result<FontSize> {
        let properties = l
            .iter()
            .map(|(k, v)| v.resolve(line_number, doc).map(|v| (k.to_string(), v)))
            .collect::<ftd::ftd2021::p1::Result<ftd::Map<ftd::Value>>>()?;
        Ok(FontSize {
            line_height: ftd::ftd2021::p2::utils::int("line-height", &properties, doc.name, 0)?,
            size: ftd::ftd2021::p2::utils::int("size", &properties, doc.name, 0)?,
            letter_spacing: ftd::ftd2021::p2::utils::int(
                "letter-spacing",
                &properties,
                doc.name,
                0,
            )?,
            reference,
        })
    }
}

#[derive(serde::Deserialize, Debug, PartialEq, Clone, serde::Serialize)]
pub struct Type {
    pub font: String,
    pub desktop: FontSize,
    pub mobile: FontSize,
    pub xl: FontSize,
    pub weight: i64,
    pub style: Style,
    pub reference: Option<String>,
}

impl Type {
    pub fn from(
        l: &ftd::Map<ftd::PropertyValue>,
        doc: &ftd::ftd2021::p2::TDoc,
        line_number: usize,
        reference: Option<String>,
    ) -> ftd::ftd2021::p1::Result<Type> {
        let properties = l
            .iter()
            .map(|(k, v)| v.resolve(line_number, doc).map(|v| (k.to_string(), v)))
            .collect::<ftd::ftd2021::p1::Result<ftd::Map<ftd::Value>>>()?;
        return Ok(Type {
            font: ftd::ftd2021::p2::utils::string("font", &properties, doc.name, 0)?,
            desktop: get_font_size(l, doc, line_number, "desktop")?,
            mobile: get_font_size(l, doc, line_number, "mobile")?,
            xl: get_font_size(l, doc, line_number, "xl")?,
            weight: ftd::ftd2021::p2::utils::int("weight", &properties, doc.name, 0)?,
            style: ftd::Style::from(
                ftd::ftd2021::p2::utils::string_optional("style", &properties, doc.name, 0)?,
                doc.name,
            )?,
            reference,
        });

        fn get_font_size(
            l: &ftd::Map<ftd::PropertyValue>,
            doc: &ftd::ftd2021::p2::TDoc,
            line_number: usize,
            name: &str,
        ) -> ftd::ftd2021::p1::Result<FontSize> {
            let properties = l
                .iter()
                .map(|(k, v)| v.resolve(line_number, doc).map(|v| (k.to_string(), v)))
                .collect::<ftd::ftd2021::p1::Result<ftd::Map<ftd::Value>>>()?;

            let property_value =
                ftd::ftd2021::p2::utils::record_optional(name, &properties, doc.name, 0)?
                    .ok_or_else(|| ftd::ftd2021::p1::Error::ParseError {
                        message: format!("expected record, for: `{name}` found: `None`"),
                        doc_id: doc.name.to_string(),
                        line_number,
                    })?;

            let reference = {
                let mut reference = None;
                if let Some(val) = l.get(name) {
                    reference = val.get_reference();
                }
                reference
            };
            FontSize::from(&property_value, doc, line_number, reference)
        }
    }
}

#[derive(serde::Deserialize, Debug, PartialEq, Clone, serde::Serialize)]
#[serde(tag = "type")]
pub enum NamedFont {
    Monospace,
    Serif,
    SansSerif,
    Named { value: String },
}

impl NamedFont {
    pub fn from(l: Option<String>) -> ftd::ftd2021::p1::Result<ftd::NamedFont> {
        Ok(match l.as_deref() {
            Some("monospace") => ftd::NamedFont::Monospace,
            Some("serif") => ftd::NamedFont::Serif,
            Some("sansSerif") => ftd::NamedFont::SansSerif,
            Some(t) => ftd::NamedFont::Named {
                value: t.to_string(),
            },
            None => return Ok(ftd::NamedFont::Serif),
        })
    }
}

impl Display for NamedFont {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let str = match self {
            ftd::NamedFont::Monospace => "monospace",
            ftd::NamedFont::Serif => "serif",
            ftd::NamedFont::SansSerif => "sansSerif",
            ftd::NamedFont::Named { value } => value,
        }
        .to_string();
        write!(f, "{str}")
    }
}

#[derive(serde::Deserialize, Debug, PartialEq, Default, Clone, serde::Serialize)]
pub struct ExternalFont {
    pub url: String,
    pub name: String,
    pub display: FontDisplay,
}

#[derive(serde::Deserialize, Debug, PartialEq, Clone, serde::Serialize)]
#[serde(tag = "type")]
pub enum Weight {
    Heavy,
    ExtraBold,
    Bold,
    SemiBold,
    Medium,
    Regular,
    Light,
    ExtraLight,
    HairLine,
}

#[derive(serde::Deserialize, Debug, PartialEq, Default, Clone, serde::Serialize)]
pub struct Style {
    pub italic: bool,
    pub underline: bool,
    pub strike: bool,
    pub weight: Option<ftd::Weight>,
}

impl Style {
    pub fn from(l: Option<String>, doc_id: &str) -> ftd::ftd2021::p1::Result<ftd::Style> {
        fn add_in_map(style: &str, map: &mut ftd::Map<i32>) {
            if !map.contains_key(style) {
                map.insert(style.to_string(), 1);
                return;
            }
            map.insert(style.to_string(), map[style] + 1);
        }

        let mut s = Style {
            italic: false,
            underline: false,
            strike: false,
            weight: Default::default(),
        };
        let l = match l {
            Some(v) => v,
            None => return Ok(s),
        };
        let mut booleans: ftd::Map<i32> = Default::default();
        let mut weights: ftd::Map<i32> = Default::default();

        for part in l.split_ascii_whitespace() {
            match part {
                "italic" => {
                    s.italic = true;
                    add_in_map("italic", &mut booleans);
                }
                "underline" => {
                    s.underline = true;
                    add_in_map("underline", &mut booleans);
                }
                "strike" => {
                    s.strike = true;
                    add_in_map("strike", &mut booleans);
                }
                "heavy" => {
                    s.weight = Some(ftd::Weight::Heavy);
                    add_in_map("heavy", &mut weights);
                }
                "extra-bold" => {
                    s.weight = Some(ftd::Weight::ExtraBold);
                    add_in_map("extra-bold", &mut weights);
                }
                "bold" => {
                    s.weight = Some(ftd::Weight::Bold);
                    add_in_map("bold", &mut weights);
                }
                "semi-bold" => {
                    s.weight = Some(ftd::Weight::SemiBold);
                    add_in_map("semi-bold", &mut weights);
                }
                "medium" => {
                    s.weight = Some(ftd::Weight::Medium);
                    add_in_map("medium", &mut weights);
                }
                "regular" => {
                    s.weight = Some(ftd::Weight::Regular);
                    add_in_map("regular", &mut weights);
                }
                "light" => {
                    s.weight = Some(ftd::Weight::Light);
                    add_in_map("light", &mut weights);
                }
                "extra-light" => {
                    s.weight = Some(ftd::Weight::ExtraLight);
                    add_in_map("extra-light", &mut weights);
                }
                "hairline" => {
                    s.weight = Some(ftd::Weight::HairLine);
                    add_in_map("hairline", &mut weights);
                }
                t => {
                    return ftd::ftd2021::p2::utils::e2(
                        format!("{t} is not a valid style"),
                        doc_id,
                        0,
                    );
                }
            }
        }

        // Checks if there is repeatation in Underline,italic,strike
        for (style, count) in booleans.iter() {
            if count > &1 {
                return Err(ftd::ftd2021::p1::Error::ForbiddenUsage {
                    message: format!("\'{}\' repeated {} times in \'{}\'", style, count, &l),
                    doc_id: doc_id.to_string(),
                    line_number: 0,
                });
            }
        }

        // Checks if there is conflict in font weights
        if weights.len() > 1 {
            return Err(ftd::ftd2021::p1::Error::ForbiddenUsage {
                message: format!("Conflicting weights {:?} in \'{}\'", weights.keys(), &l),
                doc_id: doc_id.to_string(),
                line_number: 0,
            });
        }

        // Checks if there is repeatation in font weights
        for (weight, count) in weights.iter() {
            if count > &1 {
                return Err(ftd::ftd2021::p1::Error::ForbiddenUsage {
                    message: format!("\'{}\' repeated {} times in \'{}\'", weight, count, &l),
                    doc_id: doc_id.to_string(),
                    line_number: 0,
                });
            }
        }

        Ok(s)
    }
}

#[derive(serde::Deserialize, Default, Debug, PartialEq, Clone, serde::Serialize)]
#[serde(tag = "type")]
pub enum TextFormat {
    // FTD, // TODO
    #[default]
    Markdown,
    Code {
        lang: String,
    },
    Text,
}

impl TextFormat {
    pub fn from(
        l: Option<String>,
        lang: Option<String>,
        doc_id: &str,
    ) -> ftd::ftd2021::p1::Result<ftd::TextFormat> {
        Ok(match l.as_deref() {
            Some("markup") => ftd::TextFormat::Markdown,
            Some("code") => ftd::TextFormat::Code {
                lang: lang.unwrap_or_else(|| "txt".to_string()),
            },
            Some("text") => ftd::TextFormat::Text,
            Some(t) => {
                return ftd::ftd2021::p2::utils::e2(
                    format!("{t} is not a valid format"),
                    doc_id,
                    0,
                );
            } // TODO
            None => return Ok(ftd::TextFormat::Markdown),
        })
    }
}

#[derive(serde::Deserialize, Debug, PartialEq, Default, Clone, serde::Serialize)]
pub struct IFrame {
    pub src: String,
    /// iframe can load lazily.
    pub loading: Loading,
    pub common: Box<Common>,
}

#[derive(serde::Deserialize, Debug, PartialEq, Default, Clone, serde::Serialize)]
pub struct Text {
    pub text: ftd::ftd2021::Rendered,
    pub line: bool,
    pub common: Box<Common>,
    pub text_align: TextAlign,
    pub text_indent: Option<Length>,
    pub style: Style,
    pub font: Option<Type>,
    pub line_clamp: Option<i64>,
    // TODO: line-height
    // TODO: region (https://package.elm-lang.org/packages/mdgriffith/elm-ui/latest/Element-Region)
    // TODO: family (maybe we need a type to represent font-family?)
    // TODO: letter-spacing
    // TODO: word-spacing
    // TODO: font-variants [small-caps, slashed-zero, feature/index etc]
    // TODO: shadow, glow
}

#[derive(serde::Deserialize, Debug, PartialEq, Default, Clone, serde::Serialize)]
pub struct TextBlock {
    pub text: ftd::ftd2021::Rendered,
    pub line: bool,
    pub common: Box<Common>,
    pub text_align: TextAlign,
    pub style: Style,
    pub size: Option<i64>,
    pub font: Vec<NamedFont>,
    pub line_height: Option<i64>,
    pub line_clamp: Option<i64>,
    pub text_indent: Option<Length>,
}

#[derive(serde::Deserialize, Debug, PartialEq, Default, Clone, serde::Serialize)]
pub struct Code {
    pub text: ftd::ftd2021::Rendered,
    pub common: Box<Common>,
    pub text_align: TextAlign,
    pub style: Style,
    pub font: Option<Type>,
    pub line_clamp: Option<i64>,
    pub text_indent: Option<Length>,
}

#[derive(serde::Deserialize, Debug, PartialEq, Default, Clone, serde::Serialize)]
pub struct Color {
    pub light: ColorValue,
    pub dark: ColorValue,
    pub reference: Option<String>,
}

impl Color {
    pub fn from(
        l: (Option<ftd::Map<ftd::PropertyValue>>, Option<String>),
        doc: &ftd::ftd2021::p2::TDoc,
        line_number: usize,
    ) -> ftd::ftd2021::p1::Result<Option<Color>> {
        let reference = l.1;
        let l = if let Some(l) = l.0 {
            l
        } else {
            return Ok(None);
        };

        let properties = l
            .iter()
            .map(|(k, v)| v.resolve(line_number, doc).map(|v| (k.to_string(), v)))
            .collect::<ftd::ftd2021::p1::Result<ftd::Map<ftd::Value>>>()?;
        Ok(Some(Color {
            light: ftd::ftd2021::p2::element::color_from(
                ftd::ftd2021::p2::utils::string_optional("light", &properties, doc.name, 0)?,
                doc.name,
            )?
            .unwrap(),
            dark: ftd::ftd2021::p2::element::color_from(
                ftd::ftd2021::p2::utils::string_optional("dark", &properties, doc.name, 0)?,
                doc.name,
            )?
            .unwrap(),
            reference,
        }))
    }
}

#[derive(serde::Deserialize, Debug, PartialEq, Default, Clone, serde::Serialize)]
pub struct ColorValue {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub alpha: f32,
}

#[derive(serde::Deserialize, Debug, PartialEq, Default, Clone, serde::Serialize)]
pub struct Input {
    pub common: Box<Common>,
    pub placeholder: Option<String>,
    pub value: Option<String>,
    pub type_: Option<String>,
    pub multiline: bool,
    pub font: Option<Type>,
    pub default_value: Option<String>,
}
