#![allow(unknown_lints)]
#![allow(renamed_and_removed_lints)]
#![allow(too_many_arguments)]

#[derive(Debug)]
pub enum Element {
    Text(Text),
    Integer(Integer),
    Decimal(Decimal),
    Boolean(Boolean),
    Column(Column),
    Row(Row),
    ContainerElement(ContainerElement),
    Image(Image),
    Video(Video),
    Device(Device),
    CheckBox(CheckBox),
    TextInput(TextInput),
    Iframe(Iframe),
    Code(Code),
    Rive(Rive),
    Document(Document),
}

impl Element {
    pub fn from_interpreter_component(
        component: &ftd::interpreter::Component,
        doc: &ftd::interpreter::TDoc,
    ) -> Element {
        match component.name.as_str() {
            "ftd#text" => Element::Text(Text::from(component)),
            "ftd#integer" => Element::Integer(Integer::from(component)),
            "ftd#decimal" => Element::Decimal(Decimal::from(component)),
            "ftd#boolean" => Element::Boolean(Boolean::from(component)),
            "ftd#column" => Element::Column(Column::from(component)),
            "ftd#row" => Element::Row(Row::from(component)),
            "ftd#container" => Element::ContainerElement(ContainerElement::from(component)),
            "ftd#image" => Element::Image(Image::from(component)),
            "ftd#video" => Element::Video(Video::from(component)),
            "ftd#checkbox" => Element::CheckBox(CheckBox::from(component)),
            "ftd#text-input" => Element::TextInput(TextInput::from(component)),
            "ftd#iframe" => Element::Iframe(Iframe::from(component)),
            "ftd#code" => Element::Code(Code::from(component, doc)),
            "ftd#desktop" | "ftd#mobile" => {
                Element::Device(Device::from(component, component.name.as_str()))
            }
            "ftd#rive" => Element::Rive(Rive::from(component)),
            "ftd#document" => Element::Document(Document::from(component)),
            _ => todo!("{}", component.name.as_str()),
        }
    }

    pub fn to_component_statements(
        &self,
        parent: &str,
        index: usize,
        doc: &ftd::interpreter::TDoc,
        rdata: &ftd::js::ResolverData,
        should_return: bool,
        has_rive_components: &mut bool,
    ) -> Vec<fastn_js::ComponentStatement> {
        let mut rdata = rdata.clone();
        match self {
            Element::Text(text) => {
                text.to_component_statements(parent, index, doc, &mut rdata, should_return)
            }
            Element::Integer(integer) => {
                integer.to_component_statements(parent, index, doc, &mut rdata, should_return)
            }
            Element::Decimal(decimal) => {
                decimal.to_component_statements(parent, index, doc, &mut rdata, should_return)
            }
            Element::Boolean(boolean) => {
                boolean.to_component_statements(parent, index, doc, &mut rdata, should_return)
            }
            Element::Column(column) => column.to_component_statements(
                parent,
                index,
                doc,
                &mut rdata,
                should_return,
                has_rive_components,
            ),
            Element::Document(document) => document.to_component_statements(
                parent,
                index,
                doc,
                &mut rdata,
                should_return,
                has_rive_components,
            ),
            Element::Row(row) => row.to_component_statements(
                parent,
                index,
                doc,
                &mut rdata,
                should_return,
                has_rive_components,
            ),
            Element::ContainerElement(container) => container.to_component_statements(
                parent,
                index,
                doc,
                &mut rdata,
                should_return,
                has_rive_components,
            ),
            Element::Image(image) => {
                image.to_component_statements(parent, index, doc, &mut rdata, should_return)
            }
            Element::Video(video) => {
                video.to_component_statements(parent, index, doc, &mut rdata, should_return)
            }
            Element::Device(d) => d.to_component_statements(
                parent,
                index,
                doc,
                &mut rdata,
                should_return,
                has_rive_components,
            ),
            Element::CheckBox(c) => {
                c.to_component_statements(parent, index, doc, &mut rdata, should_return)
            }
            Element::TextInput(t) => {
                t.to_component_statements(parent, index, doc, &mut rdata, should_return)
            }
            Element::Iframe(i) => {
                i.to_component_statements(parent, index, doc, &mut rdata, should_return)
            }
            Element::Code(c) => {
                c.to_component_statements(parent, index, doc, &mut rdata, should_return)
            }
            Element::Rive(rive) => {
                rive.to_component_statements(parent, index, doc, &mut rdata, should_return)
            }
        }
    }
}

#[derive(Debug)]
pub struct CheckBox {
    pub enabled: Option<ftd::js::Value>,
    pub checked: Option<ftd::js::Value>,
    pub common: Common,
}

impl CheckBox {
    pub fn from(component: &ftd::interpreter::Component) -> CheckBox {
        let component_definition = ftd::interpreter::default::get_default_bag()
            .get("ftd#checkbox")
            .unwrap()
            .clone()
            .component()
            .unwrap();

        CheckBox {
            enabled: ftd::js::value::get_optional_js_value(
                "enabled",
                component.properties.as_slice(),
                component_definition.arguments.as_slice(),
            ),
            checked: ftd::js::value::get_optional_js_value(
                "checked",
                component.properties.as_slice(),
                component_definition.arguments.as_slice(),
            ),
            common: Common::from(
                component.properties.as_slice(),
                component_definition.arguments.as_slice(),
                component.events.as_slice(),
            ),
        }
    }

    pub fn to_component_statements(
        &self,
        parent: &str,
        index: usize,
        doc: &ftd::interpreter::TDoc,
        rdata: &mut ftd::js::ResolverData,
        should_return: bool,
    ) -> Vec<fastn_js::ComponentStatement> {
        let mut component_statements = vec![];
        let kernel = create_element(fastn_js::ElementKind::CheckBox, parent, index, rdata);
        component_statements.push(fastn_js::ComponentStatement::CreateKernel(kernel.clone()));
        component_statements.extend(self.common.to_set_properties(
            kernel.name.as_str(),
            doc,
            rdata,
        ));

        if let Some(ref checked) = self.checked {
            component_statements.push(fastn_js::ComponentStatement::SetProperty(
                checked.to_set_property(
                    fastn_js::PropertyKind::Checked,
                    doc,
                    kernel.name.as_str(),
                    rdata,
                ),
            ));
        }
        if let Some(ref enabled) = self.enabled {
            component_statements.push(fastn_js::ComponentStatement::SetProperty(
                enabled.to_set_property(
                    fastn_js::PropertyKind::Enabled,
                    doc,
                    kernel.name.as_str(),
                    rdata,
                ),
            ));
        }

        if should_return {
            component_statements.push(fastn_js::ComponentStatement::Return {
                component_name: kernel.name,
            });
        }
        component_statements
    }
}

#[derive(Debug)]
pub struct TextInput {
    pub placeholder: Option<ftd::js::Value>,
    pub multiline: Option<ftd::js::Value>,
    pub max_length: Option<ftd::js::Value>,
    pub _type: Option<ftd::js::Value>,
    pub value: Option<ftd::js::Value>,
    pub default_value: Option<ftd::js::Value>,
    pub enabled: Option<ftd::js::Value>,
    pub common: Common,
}

impl TextInput {
    pub fn from(component: &ftd::interpreter::Component) -> TextInput {
        let component_definition = ftd::interpreter::default::get_default_bag()
            .get("ftd#text-input")
            .unwrap()
            .clone()
            .component()
            .unwrap();

        TextInput {
            placeholder: ftd::js::value::get_optional_js_value(
                "placeholder",
                component.properties.as_slice(),
                component_definition.arguments.as_slice(),
            ),
            multiline: ftd::js::value::get_optional_js_value(
                "multiline",
                component.properties.as_slice(),
                component_definition.arguments.as_slice(),
            ),
            _type: ftd::js::value::get_optional_js_value(
                "type",
                component.properties.as_slice(),
                component_definition.arguments.as_slice(),
            ),
            value: ftd::js::value::get_optional_js_value(
                "value",
                component.properties.as_slice(),
                component_definition.arguments.as_slice(),
            ),
            default_value: ftd::js::value::get_optional_js_value(
                "default-value",
                component.properties.as_slice(),
                component_definition.arguments.as_slice(),
            ),
            enabled: ftd::js::value::get_optional_js_value(
                "enabled",
                component.properties.as_slice(),
                component_definition.arguments.as_slice(),
            ),
            max_length: ftd::js::value::get_optional_js_value(
                "max-length",
                component.properties.as_slice(),
                component_definition.arguments.as_slice(),
            ),
            common: Common::from(
                component.properties.as_slice(),
                component_definition.arguments.as_slice(),
                component.events.as_slice(),
            ),
        }
    }

    pub fn to_component_statements(
        &self,
        parent: &str,
        index: usize,
        doc: &ftd::interpreter::TDoc,
        rdata: &mut ftd::js::ResolverData,
        should_return: bool,
    ) -> Vec<fastn_js::ComponentStatement> {
        let mut component_statements = vec![];
        let kernel = create_element(fastn_js::ElementKind::TextInput, parent, index, rdata);
        component_statements.push(fastn_js::ComponentStatement::CreateKernel(kernel.clone()));
        component_statements.extend(self.common.to_set_properties(
            kernel.name.as_str(),
            doc,
            rdata,
        ));

        if let Some(ref placeholder) = self.placeholder {
            component_statements.push(fastn_js::ComponentStatement::SetProperty(
                placeholder.to_set_property(
                    fastn_js::PropertyKind::Placeholder,
                    doc,
                    kernel.name.as_str(),
                    rdata,
                ),
            ));
        }
        if let Some(ref multiline) = self.multiline {
            component_statements.push(fastn_js::ComponentStatement::SetProperty(
                multiline.to_set_property(
                    fastn_js::PropertyKind::Multiline,
                    doc,
                    kernel.name.as_str(),
                    rdata,
                ),
            ));
        }
        if let Some(ref _type) = self._type {
            component_statements.push(fastn_js::ComponentStatement::SetProperty(
                _type.to_set_property(
                    fastn_js::PropertyKind::TextInputType,
                    doc,
                    kernel.name.as_str(),
                    rdata,
                ),
            ));
        }
        if let Some(ref enabled) = self.enabled {
            component_statements.push(fastn_js::ComponentStatement::SetProperty(
                enabled.to_set_property(
                    fastn_js::PropertyKind::Enabled,
                    doc,
                    kernel.name.as_str(),
                    rdata,
                ),
            ));
        }
        if let Some(ref value) = self.value {
            component_statements.push(fastn_js::ComponentStatement::SetProperty(
                value.to_set_property(
                    fastn_js::PropertyKind::TextInputValue,
                    doc,
                    kernel.name.as_str(),
                    rdata,
                ),
            ));
        }
        if let Some(ref default_value) = self.default_value {
            component_statements.push(fastn_js::ComponentStatement::SetProperty(
                default_value.to_set_property(
                    fastn_js::PropertyKind::DefaultTextInputValue,
                    doc,
                    kernel.name.as_str(),
                    rdata,
                ),
            ));
        }
        if let Some(ref max_length) = self.max_length {
            component_statements.push(fastn_js::ComponentStatement::SetProperty(
                max_length.to_set_property(
                    fastn_js::PropertyKind::InputMaxLength,
                    doc,
                    kernel.name.as_str(),
                    rdata,
                ),
            ));
        }

        if should_return {
            component_statements.push(fastn_js::ComponentStatement::Return {
                component_name: kernel.name,
            });
        }
        component_statements
    }
}

#[derive(Debug)]
pub struct Iframe {
    pub common: Common,
    pub src: Option<ftd::js::Value>,
    pub srcdoc: Option<ftd::js::Value>,
    pub youtube: Option<ftd::js::Value>,
    pub loading: Option<ftd::js::Value>,
}

impl Iframe {
    pub fn from(component: &ftd::interpreter::Component) -> Iframe {
        let component_definition = ftd::interpreter::default::get_default_bag()
            .get("ftd#iframe")
            .unwrap()
            .clone()
            .component()
            .unwrap();

        Iframe {
            common: Common::from(
                component.properties.as_slice(),
                component_definition.arguments.as_slice(),
                component.events.as_slice(),
            ),
            src: ftd::js::value::get_optional_js_value(
                "src",
                component.properties.as_slice(),
                component_definition.arguments.as_slice(),
            ),
            srcdoc: ftd::js::value::get_optional_js_value(
                "srcdoc",
                component.properties.as_slice(),
                component_definition.arguments.as_slice(),
            ),
            loading: ftd::js::value::get_optional_js_value(
                "loading",
                component.properties.as_slice(),
                component_definition.arguments.as_slice(),
            ),
            youtube: ftd::js::value::get_optional_js_value(
                "youtube",
                component.properties.as_slice(),
                component_definition.arguments.as_slice(),
            ),
        }
    }

    pub fn to_component_statements(
        &self,
        parent: &str,
        index: usize,
        doc: &ftd::interpreter::TDoc,
        rdata: &mut ftd::js::ResolverData,
        should_return: bool,
    ) -> Vec<fastn_js::ComponentStatement> {
        let mut component_statements = vec![];
        let kernel = create_element(fastn_js::ElementKind::IFrame, parent, index, rdata);
        component_statements.push(fastn_js::ComponentStatement::CreateKernel(kernel.clone()));
        component_statements.extend(self.common.to_set_properties(
            kernel.name.as_str(),
            doc,
            rdata,
        ));

        if let Some(ref loading) = self.loading {
            component_statements.push(fastn_js::ComponentStatement::SetProperty(
                loading.to_set_property(
                    fastn_js::PropertyKind::Loading,
                    doc,
                    kernel.name.as_str(),
                    rdata,
                ),
            ));
        }

        if let Some(ref src) = self.src {
            component_statements.push(fastn_js::ComponentStatement::SetProperty(
                src.to_set_property(
                    fastn_js::PropertyKind::Src,
                    doc,
                    kernel.name.as_str(),
                    rdata,
                ),
            ));
        }

        if let Some(ref srcdoc) = self.srcdoc {
            component_statements.push(fastn_js::ComponentStatement::SetProperty(
                srcdoc.to_set_property(
                    fastn_js::PropertyKind::Src,
                    doc,
                    kernel.name.as_str(),
                    rdata,
                ),
            ));
        }

        if let Some(ref youtube) = self.youtube {
            component_statements.push(fastn_js::ComponentStatement::SetProperty(
                youtube.to_set_property(
                    fastn_js::PropertyKind::YoutubeSrc,
                    doc,
                    kernel.name.as_str(),
                    rdata,
                ),
            ));
        }

        if should_return {
            component_statements.push(fastn_js::ComponentStatement::Return {
                component_name: kernel.name,
            });
        }
        component_statements
    }
}

#[derive(Debug)]
pub struct Code {
    pub common: Common,
    pub text_common: TextCommon,
    pub code: ftd::js::Value,
    pub lang: ftd::js::Value,
    pub theme: ftd::js::Value,
    pub show_line_number: ftd::js::Value,
}

impl Code {
    pub fn from(component: &ftd::interpreter::Component, _doc: &ftd::interpreter::TDoc) -> Code {
        let component_definition = ftd::interpreter::default::get_default_bag()
            .get("ftd#code")
            .unwrap()
            .clone()
            .component()
            .unwrap();

        Code {
            common: Common::from(
                component.properties.as_slice(),
                component_definition.arguments.as_slice(),
                component.events.as_slice(),
            ),
            text_common: TextCommon::from(
                component.properties.as_slice(),
                component_definition.arguments.as_slice(),
            ),
            // code: ftd::js::Value::from_str_value(stylized_code.as_str()),
            code: ftd::js::value::get_optional_js_value(
                "text",
                component.properties.as_slice(),
                component_definition.arguments.as_slice(),
            )
            .unwrap(),
            lang: ftd::js::value::get_js_value_with_default(
                "lang",
                component.properties.as_slice(),
                component_definition.arguments.as_slice(),
                ftd::js::Value::from_str_value("txt"),
            ),
            theme: ftd::js::value::get_js_value_with_default(
                "theme",
                component.properties.as_slice(),
                component_definition.arguments.as_slice(),
                ftd::js::Value::from_str_value(ftd::js::CODE_DEFAULT_THEME),
            ),
            show_line_number: ftd::js::value::get_optional_js_value_with_default(
                "show-line-number",
                component.properties.as_slice(),
                component_definition.arguments.as_slice(),
            )
            .unwrap(),
        }
    }

    pub fn to_component_statements(
        &self,
        parent: &str,
        index: usize,
        doc: &ftd::interpreter::TDoc,
        rdata: &mut ftd::js::ResolverData,
        should_return: bool,
    ) -> Vec<fastn_js::ComponentStatement> {
        let mut component_statements = vec![];
        let kernel = create_element(fastn_js::ElementKind::Code, parent, index, rdata);
        component_statements.push(fastn_js::ComponentStatement::CreateKernel(kernel.clone()));

        component_statements.push(fastn_js::ComponentStatement::SetProperty(
            self.code.to_set_property(
                fastn_js::PropertyKind::Code,
                doc,
                kernel.name.as_str(),
                rdata,
            ),
        ));

        component_statements.push(fastn_js::ComponentStatement::SetProperty(
            self.lang.to_set_property(
                fastn_js::PropertyKind::CodeLanguage,
                doc,
                kernel.name.as_str(),
                rdata,
            ),
        ));

        component_statements.push(fastn_js::ComponentStatement::SetProperty(
            self.theme.to_set_property(
                fastn_js::PropertyKind::CodeTheme,
                doc,
                kernel.name.as_str(),
                rdata,
            ),
        ));

        component_statements.push(fastn_js::ComponentStatement::SetProperty(
            self.show_line_number.to_set_property(
                fastn_js::PropertyKind::CodeShowLineNumber,
                doc,
                kernel.name.as_str(),
                rdata,
            ),
        ));

        component_statements.extend(self.common.to_set_properties(
            kernel.name.as_str(),
            doc,
            rdata,
        ));

        component_statements.extend(self.text_common.to_set_properties(
            kernel.name.as_str(),
            doc,
            rdata,
        ));

        if should_return {
            component_statements.push(fastn_js::ComponentStatement::Return {
                component_name: kernel.name,
            });
        }
        component_statements
    }
}

#[derive(Debug)]
pub struct Image {
    pub src: ftd::js::Value,
    pub fit: Option<ftd::js::Value>,
    pub alt: Option<ftd::js::Value>,
    pub common: Common,
}

impl Image {
    pub fn from(component: &ftd::interpreter::Component) -> Image {
        let component_definition = ftd::interpreter::default::get_default_bag()
            .get("ftd#image")
            .unwrap()
            .clone()
            .component()
            .unwrap();
        Image {
            src: ftd::js::value::get_optional_js_value(
                "src",
                component.properties.as_slice(),
                component_definition.arguments.as_slice(),
            )
            .unwrap(),
            fit: ftd::js::value::get_optional_js_value(
                "fit",
                component.properties.as_slice(),
                component_definition.arguments.as_slice(),
            ),
            alt: ftd::js::value::get_optional_js_value(
                "alt",
                component.properties.as_slice(),
                component_definition.arguments.as_slice(),
            ),
            common: Common::from(
                component.properties.as_slice(),
                component_definition.arguments.as_slice(),
                component.events.as_slice(),
            ),
        }
    }

    pub fn to_component_statements(
        &self,
        parent: &str,
        index: usize,
        doc: &ftd::interpreter::TDoc,
        rdata: &mut ftd::js::ResolverData,
        should_return: bool,
    ) -> Vec<fastn_js::ComponentStatement> {
        let mut component_statements = vec![];
        let kernel = create_element(fastn_js::ElementKind::Image, parent, index, rdata);
        component_statements.push(fastn_js::ComponentStatement::CreateKernel(kernel.clone()));
        component_statements.push(fastn_js::ComponentStatement::SetProperty(
            fastn_js::SetProperty {
                kind: fastn_js::PropertyKind::ImageSrc,
                value: self.src.to_set_property_value(doc, rdata),
                element_name: kernel.name.to_string(),
                inherited: rdata.inherited_variable_name.to_string(),
            },
        ));
        if let Some(ref alt) = self.alt {
            component_statements.push(fastn_js::ComponentStatement::SetProperty(
                alt.to_set_property(
                    fastn_js::PropertyKind::Alt,
                    doc,
                    kernel.name.as_str(),
                    rdata,
                ),
            ));
        }
        if let Some(ref fit) = self.fit {
            component_statements.push(fastn_js::ComponentStatement::SetProperty(
                fit.to_set_property(
                    fastn_js::PropertyKind::Fit,
                    doc,
                    kernel.name.as_str(),
                    rdata,
                ),
            ));
        }
        component_statements.extend(self.common.to_set_properties(
            kernel.name.as_str(),
            doc,
            rdata,
        ));

        if should_return {
            component_statements.push(fastn_js::ComponentStatement::Return {
                component_name: kernel.name,
            });
        }
        component_statements
    }
}

#[derive(Debug)]
pub struct Video {
    pub src: ftd::js::Value,
    pub fit: Option<ftd::js::Value>,
    pub controls: Option<ftd::js::Value>,
    pub loop_video: Option<ftd::js::Value>,
    pub muted: Option<ftd::js::Value>,
    pub autoplay: Option<ftd::js::Value>,
    pub poster: Option<ftd::js::Value>,
    pub common: Common,
}

impl Video {
    pub fn from(component: &ftd::interpreter::Component) -> Video {
        let component_definition = ftd::interpreter::default::get_default_bag()
            .get("ftd#video")
            .unwrap()
            .clone()
            .component()
            .unwrap();
        Video {
            src: ftd::js::value::get_optional_js_value(
                "src",
                component.properties.as_slice(),
                component_definition.arguments.as_slice(),
            )
            .unwrap(),
            fit: ftd::js::value::get_optional_js_value(
                "fit",
                component.properties.as_slice(),
                component_definition.arguments.as_slice(),
            ),
            autoplay: ftd::js::value::get_optional_js_value(
                "autoplay",
                component.properties.as_slice(),
                component_definition.arguments.as_slice(),
            ),
            controls: ftd::js::value::get_optional_js_value(
                "controls",
                component.properties.as_slice(),
                component_definition.arguments.as_slice(),
            ),
            loop_video: ftd::js::value::get_optional_js_value(
                "loop",
                component.properties.as_slice(),
                component_definition.arguments.as_slice(),
            ),
            muted: ftd::js::value::get_optional_js_value(
                "muted",
                component.properties.as_slice(),
                component_definition.arguments.as_slice(),
            ),
            poster: ftd::js::value::get_optional_js_value(
                "poster",
                component.properties.as_slice(),
                component_definition.arguments.as_slice(),
            ),
            common: Common::from(
                component.properties.as_slice(),
                component_definition.arguments.as_slice(),
                component.events.as_slice(),
            ),
        }
    }

    pub fn to_component_statements(
        &self,
        parent: &str,
        index: usize,
        doc: &ftd::interpreter::TDoc,
        rdata: &mut ftd::js::ResolverData,
        should_return: bool,
    ) -> Vec<fastn_js::ComponentStatement> {
        let mut component_statements = vec![];
        let kernel = create_element(fastn_js::ElementKind::Video, parent, index, rdata);
        component_statements.push(fastn_js::ComponentStatement::CreateKernel(kernel.clone()));
        component_statements.push(fastn_js::ComponentStatement::SetProperty(
            fastn_js::SetProperty {
                kind: fastn_js::PropertyKind::VideoSrc,
                value: self.src.to_set_property_value(doc, rdata),
                element_name: kernel.name.to_string(),
                inherited: rdata.inherited_variable_name.to_string(),
            },
        ));
        if let Some(ref fit) = self.fit {
            component_statements.push(fastn_js::ComponentStatement::SetProperty(
                fit.to_set_property(
                    fastn_js::PropertyKind::Fit,
                    doc,
                    kernel.name.as_str(),
                    rdata,
                ),
            ));
        }
        if let Some(ref controls) = self.controls {
            component_statements.push(fastn_js::ComponentStatement::SetProperty(
                controls.to_set_property(
                    fastn_js::PropertyKind::Controls,
                    doc,
                    kernel.name.as_str(),
                    rdata,
                ),
            ));
        }
        if let Some(ref autoplay) = self.autoplay {
            component_statements.push(fastn_js::ComponentStatement::SetProperty(
                autoplay.to_set_property(
                    fastn_js::PropertyKind::Autoplay,
                    doc,
                    kernel.name.as_str(),
                    rdata,
                ),
            ));
        }
        if let Some(ref muted) = self.muted {
            component_statements.push(fastn_js::ComponentStatement::SetProperty(
                muted.to_set_property(
                    fastn_js::PropertyKind::Muted,
                    doc,
                    kernel.name.as_str(),
                    rdata,
                ),
            ));
        }
        if let Some(ref loop_video) = self.loop_video {
            component_statements.push(fastn_js::ComponentStatement::SetProperty(
                loop_video.to_set_property(
                    fastn_js::PropertyKind::LoopVideo,
                    doc,
                    kernel.name.as_str(),
                    rdata,
                ),
            ));
        }
        if let Some(ref poster) = self.poster {
            component_statements.push(fastn_js::ComponentStatement::SetProperty(
                poster.to_set_property(
                    fastn_js::PropertyKind::Poster,
                    doc,
                    kernel.name.as_str(),
                    rdata,
                ),
            ));
        }
        component_statements.extend(self.common.to_set_properties(
            kernel.name.as_str(),
            doc,
            rdata,
        ));

        if should_return {
            component_statements.push(fastn_js::ComponentStatement::Return {
                component_name: kernel.name,
            });
        }
        component_statements
    }
}

#[derive(Debug)]
pub struct Text {
    pub text: ftd::js::Value,
    pub common: Common,
    pub text_common: TextCommon,
}

#[derive(Debug)]
pub struct Integer {
    pub value: ftd::js::Value,
    pub common: Common,
    pub text_common: TextCommon,
}

#[derive(Debug)]
pub struct Decimal {
    pub value: ftd::js::Value,
    pub common: Common,
    pub text_common: TextCommon,
}

#[derive(Debug)]
pub struct Boolean {
    pub value: ftd::js::Value,
    pub common: Common,
    pub text_common: TextCommon,
}

#[derive(Debug)]
pub struct Document {
    pub container: Container,
    pub breakpoint_width: Option<ftd::js::Value>,
    pub metadata: DocumentMeta,
}

#[derive(Debug)]
pub struct DocumentMeta {
    pub title: Option<ftd::js::Value>,
    pub favicon: Option<ftd::js::Value>,
    pub og_title: Option<ftd::js::Value>,
    pub twitter_title: Option<ftd::js::Value>,
    pub description: Option<ftd::js::Value>,
    pub og_description: Option<ftd::js::Value>,
    pub twitter_description: Option<ftd::js::Value>,
    pub og_image: Option<ftd::js::Value>,
    pub twitter_image: Option<ftd::js::Value>,
    pub theme_color: Option<ftd::js::Value>,
}

#[derive(Debug)]
pub struct Column {
    pub container: Container,
    pub container_properties: ContainerProperties,
    pub common: Common,
}

#[derive(Debug)]
pub struct InheritedProperties {
    pub colors: Option<ftd::js::Value>,
    pub types: Option<ftd::js::Value>,
}

#[derive(Debug)]
pub struct ContainerProperties {
    pub spacing: Option<ftd::js::Value>,
    pub wrap: Option<ftd::js::Value>,
    pub align_content: Option<ftd::js::Value>,
    pub backdrop_filter: Option<ftd::js::Value>,
}

impl ContainerProperties {
    pub fn from(
        properties: &[ftd::interpreter::Property],
        arguments: &[ftd::interpreter::Argument],
    ) -> ContainerProperties {
        ContainerProperties {
            spacing: ftd::js::value::get_optional_js_value("spacing", properties, arguments),
            wrap: ftd::js::value::get_optional_js_value("wrap", properties, arguments),
            align_content: ftd::js::value::get_optional_js_value(
                "align-content",
                properties,
                arguments,
            ),
            backdrop_filter: ftd::js::value::get_optional_js_value(
                "backdrop-filter",
                properties,
                arguments,
            ),
        }
    }

    pub fn to_set_properties(
        &self,
        element_name: &str,
        doc: &ftd::interpreter::TDoc,
        rdata: &ftd::js::ResolverData,
    ) -> Vec<fastn_js::ComponentStatement> {
        let mut component_statements = vec![];
        if let Some(ref wrap) = self.wrap {
            component_statements.push(fastn_js::ComponentStatement::SetProperty(
                wrap.to_set_property(fastn_js::PropertyKind::Wrap, doc, element_name, rdata),
            ));
        }
        if let Some(ref align_content) = self.align_content {
            component_statements.push(fastn_js::ComponentStatement::SetProperty(
                align_content.to_set_property(
                    fastn_js::PropertyKind::AlignContent,
                    doc,
                    element_name,
                    rdata,
                ),
            ));
        }
        // prioritizing spacing > align-content for justify-content
        if let Some(ref spacing) = self.spacing {
            component_statements.push(fastn_js::ComponentStatement::SetProperty(
                spacing.to_set_property(fastn_js::PropertyKind::Spacing, doc, element_name, rdata),
            ));
        }
        if let Some(ref backdrop_filter) = self.backdrop_filter {
            component_statements.push(fastn_js::ComponentStatement::SetProperty(
                backdrop_filter.to_set_property(
                    fastn_js::PropertyKind::BackdropFilter,
                    doc,
                    element_name,
                    rdata,
                ),
            ));
        }
        component_statements
    }
}

#[derive(Debug)]
pub struct Container {
    pub children: Option<ftd::js::Value>,
    pub inherited: InheritedProperties,
}

impl Container {
    pub fn from(
        properties: &[ftd::interpreter::Property],
        arguments: &[ftd::interpreter::Argument],
    ) -> Container {
        Container {
            children: ftd::js::utils::get_js_value_from_properties(
                ftd::interpreter::utils::get_children_properties_from_properties(properties)
                    .as_slice(),
            ),
            inherited: InheritedProperties::from(properties, arguments),
        }
    }

    pub(crate) fn to_component_statements(
        &self,
        doc: &ftd::interpreter::TDoc,
        rdata: &ftd::js::ResolverData,
        has_rive_components: &mut bool,
        should_return: bool,
    ) -> Vec<fastn_js::ComponentStatement> {
        let mut component_statements = vec![];

        // rdata will have component_name
        let component_name = rdata.component_name.clone().unwrap().to_string();

        let inherited_variables =
            self.inherited
                .get_inherited_variables(doc, rdata, component_name.as_str());

        let inherited_variable_name = inherited_variables
            .as_ref()
            .map(|v| v.name.clone())
            .unwrap_or_else(|| rdata.inherited_variable_name.to_string());

        if let Some(inherited_variables) = inherited_variables {
            component_statements.push(fastn_js::ComponentStatement::StaticVariable(
                inherited_variables,
            ));
        }

        component_statements.extend(self.children.iter().map(|v| {
            fastn_js::ComponentStatement::SetProperty(fastn_js::SetProperty {
                kind: fastn_js::PropertyKind::Children,
                value: v.to_set_property_value_with_ui(
                    doc,
                    &rdata.clone_with_new_inherited_variable(&inherited_variable_name),
                    has_rive_components,
                    should_return,
                ),
                element_name: component_name.to_string(),
                inherited: inherited_variable_name.to_string(),
            })
        }));

        component_statements
    }
}

#[derive(Debug)]
pub struct ContainerElement {
    pub container: Container,
    pub common: Common,
}

#[derive(Debug)]
pub struct Row {
    pub container: Container,
    pub container_properties: ContainerProperties,
    pub common: Common,
}

impl InheritedProperties {
    pub fn from(
        properties: &[ftd::interpreter::Property],
        arguments: &[ftd::interpreter::Argument],
    ) -> InheritedProperties {
        InheritedProperties {
            colors: ftd::js::value::get_optional_js_value("colors", properties, arguments),
            types: ftd::js::value::get_optional_js_value("types", properties, arguments),
        }
    }

    pub(crate) fn get_inherited_variables(
        &self,
        doc: &ftd::interpreter::TDoc,
        rdata: &ftd::js::ResolverData,
        component_name: &str,
    ) -> Option<fastn_js::StaticVariable> {
        let mut inherited_fields = vec![];

        if let Some(ref colors) = self.colors {
            inherited_fields.push((
                "colors".to_string(),
                colors.to_set_property_value(doc, &rdata.clone_with_default_inherited_variable()),
            ));
        }

        if let Some(ref types) = self.types {
            inherited_fields.push((
                "types".to_string(),
                types.to_set_property_value(doc, &rdata.clone_with_default_inherited_variable()),
            ));
        }

        if !inherited_fields.is_empty() {
            Some(fastn_js::StaticVariable {
                name: format!("{}{}", fastn_js::INHERITED_PREFIX, component_name),
                value: fastn_js::SetPropertyValue::Value(fastn_js::Value::Record {
                    fields: inherited_fields,
                    other_references: vec![rdata.inherited_variable_name.to_string()],
                }),
                prefix: None,
            })
        } else {
            None
        }
    }
}

impl Text {
    pub fn from(component: &ftd::interpreter::Component) -> Text {
        let component_definition = ftd::interpreter::default::get_default_bag()
            .get("ftd#text")
            .unwrap()
            .clone()
            .component()
            .unwrap();
        Text {
            text: ftd::js::value::get_optional_js_value(
                "text",
                component.properties.as_slice(),
                component_definition.arguments.as_slice(),
            )
            .unwrap(),
            common: Common::from(
                component.properties.as_slice(),
                component_definition.arguments.as_slice(),
                component.events.as_slice(),
            ),
            text_common: TextCommon::from(
                component.properties.as_slice(),
                component_definition.arguments.as_slice(),
            ),
        }
    }

    pub fn to_component_statements(
        &self,
        parent: &str,
        index: usize,
        doc: &ftd::interpreter::TDoc,
        rdata: &mut ftd::js::ResolverData,
        should_return: bool,
    ) -> Vec<fastn_js::ComponentStatement> {
        let mut component_statements = vec![];
        let kernel = create_element(fastn_js::ElementKind::Text, parent, index, rdata);
        component_statements.push(fastn_js::ComponentStatement::CreateKernel(kernel.clone()));
        component_statements.extend(self.common.to_set_properties_with_text(
            kernel.name.as_str(),
            doc,
            rdata,
            fastn_js::ComponentStatement::SetProperty(fastn_js::SetProperty {
                kind: fastn_js::PropertyKind::StringValue,
                value: self.text.to_set_property_value(doc, rdata),
                element_name: kernel.name.to_string(),
                inherited: rdata.inherited_variable_name.to_string(),
            }),
        ));
        component_statements.extend(self.text_common.to_set_properties(
            kernel.name.as_str(),
            doc,
            rdata,
        ));

        if should_return {
            component_statements.push(fastn_js::ComponentStatement::Return {
                component_name: kernel.name,
            });
        }
        component_statements
    }
}

impl Integer {
    pub fn from(component: &ftd::interpreter::Component) -> Integer {
        let component_definition = ftd::interpreter::default::get_default_bag()
            .get("ftd#integer")
            .unwrap()
            .clone()
            .component()
            .unwrap();
        Integer {
            value: ftd::js::value::get_optional_js_value(
                "value",
                component.properties.as_slice(),
                component_definition.arguments.as_slice(),
            )
            .unwrap(),
            common: Common::from(
                component.properties.as_slice(),
                component_definition.arguments.as_slice(),
                component.events.as_slice(),
            ),
            text_common: TextCommon::from(
                component.properties.as_slice(),
                component_definition.arguments.as_slice(),
            ),
        }
    }

    pub fn to_component_statements(
        &self,
        parent: &str,
        index: usize,
        doc: &ftd::interpreter::TDoc,
        rdata: &mut ftd::js::ResolverData,
        should_return: bool,
    ) -> Vec<fastn_js::ComponentStatement> {
        let mut component_statements = vec![];
        let kernel = create_element(fastn_js::ElementKind::Integer, parent, index, rdata);
        component_statements.push(fastn_js::ComponentStatement::CreateKernel(kernel.clone()));
        component_statements.push(fastn_js::ComponentStatement::SetProperty(
            fastn_js::SetProperty {
                kind: fastn_js::PropertyKind::IntegerValue,
                value: self.value.to_set_property_value(doc, rdata),
                element_name: kernel.name.to_string(),
                inherited: rdata.inherited_variable_name.to_string(),
            },
        ));
        component_statements.extend(self.common.to_set_properties(
            kernel.name.as_str(),
            doc,
            rdata,
        ));
        component_statements.extend(self.text_common.to_set_properties(
            kernel.name.as_str(),
            doc,
            rdata,
        ));
        if should_return {
            component_statements.push(fastn_js::ComponentStatement::Return {
                component_name: kernel.name,
            });
        }
        component_statements
    }
}

impl Decimal {
    pub fn from(component: &ftd::interpreter::Component) -> Decimal {
        let component_definition = ftd::interpreter::default::get_default_bag()
            .get("ftd#decimal")
            .unwrap()
            .clone()
            .component()
            .unwrap();
        Decimal {
            value: ftd::js::value::get_optional_js_value(
                "value",
                component.properties.as_slice(),
                component_definition.arguments.as_slice(),
            )
            .unwrap(),
            common: Common::from(
                component.properties.as_slice(),
                component_definition.arguments.as_slice(),
                component.events.as_slice(),
            ),
            text_common: TextCommon::from(
                component.properties.as_slice(),
                component_definition.arguments.as_slice(),
            ),
        }
    }

    pub fn to_component_statements(
        &self,
        parent: &str,
        index: usize,
        doc: &ftd::interpreter::TDoc,
        rdata: &mut ftd::js::ResolverData,
        should_return: bool,
    ) -> Vec<fastn_js::ComponentStatement> {
        let mut component_statements = vec![];
        let kernel = create_element(fastn_js::ElementKind::Decimal, parent, index, rdata);
        component_statements.push(fastn_js::ComponentStatement::CreateKernel(kernel.clone()));
        component_statements.push(fastn_js::ComponentStatement::SetProperty(
            fastn_js::SetProperty {
                kind: fastn_js::PropertyKind::DecimalValue,
                value: self.value.to_set_property_value(doc, rdata),
                element_name: kernel.name.to_string(),
                inherited: rdata.inherited_variable_name.to_string(),
            },
        ));
        component_statements.extend(self.common.to_set_properties(
            kernel.name.as_str(),
            doc,
            rdata,
        ));
        component_statements.extend(self.text_common.to_set_properties(
            kernel.name.as_str(),
            doc,
            rdata,
        ));
        if should_return {
            component_statements.push(fastn_js::ComponentStatement::Return {
                component_name: kernel.name,
            });
        }
        component_statements
    }
}

impl Boolean {
    pub fn from(component: &ftd::interpreter::Component) -> Boolean {
        let component_definition = ftd::interpreter::default::get_default_bag()
            .get("ftd#boolean")
            .unwrap()
            .clone()
            .component()
            .unwrap();
        Boolean {
            value: ftd::js::value::get_optional_js_value(
                "value",
                component.properties.as_slice(),
                component_definition.arguments.as_slice(),
            )
            .unwrap(),
            common: Common::from(
                component.properties.as_slice(),
                component_definition.arguments.as_slice(),
                component.events.as_slice(),
            ),
            text_common: TextCommon::from(
                component.properties.as_slice(),
                component_definition.arguments.as_slice(),
            ),
        }
    }

    pub fn to_component_statements(
        &self,
        parent: &str,
        index: usize,
        doc: &ftd::interpreter::TDoc,
        rdata: &mut ftd::js::ResolverData,
        should_return: bool,
    ) -> Vec<fastn_js::ComponentStatement> {
        let mut component_statements = vec![];
        let kernel = create_element(fastn_js::ElementKind::Boolean, parent, index, rdata);
        component_statements.push(fastn_js::ComponentStatement::CreateKernel(kernel.clone()));
        component_statements.push(fastn_js::ComponentStatement::SetProperty(
            fastn_js::SetProperty {
                kind: fastn_js::PropertyKind::BooleanValue,
                value: self.value.to_set_property_value(doc, rdata),
                element_name: kernel.name.to_string(),
                inherited: rdata.inherited_variable_name.to_string(),
            },
        ));
        component_statements.extend(self.common.to_set_properties(
            kernel.name.as_str(),
            doc,
            rdata,
        ));
        component_statements.extend(self.text_common.to_set_properties(
            kernel.name.as_str(),
            doc,
            rdata,
        ));
        if should_return {
            component_statements.push(fastn_js::ComponentStatement::Return {
                component_name: kernel.name,
            });
        }
        component_statements
    }
}

impl Document {
    pub fn from(component: &ftd::interpreter::Component) -> Document {
        let component_definition = ftd::interpreter::default::get_default_bag()
            .get("ftd#document")
            .unwrap()
            .clone()
            .component()
            .unwrap();

        Document {
            container: Container::from(
                component.properties.as_slice(),
                component_definition.arguments.as_slice(),
            ),
            breakpoint_width: ftd::js::value::get_optional_js_value(
                "breakpoint",
                component.properties.as_slice(),
                component_definition.arguments.as_slice(),
            ),
            metadata: DocumentMeta::from(
                component.properties.as_slice(),
                component_definition.arguments.as_slice(),
            ),
        }
    }

    pub fn to_component_statements(
        &self,
        parent: &str,
        index: usize,
        doc: &ftd::interpreter::TDoc,
        rdata: &mut ftd::js::ResolverData,
        should_return: bool,
        has_rive_components: &mut bool,
    ) -> Vec<fastn_js::ComponentStatement> {
        let mut component_statements = vec![];
        let kernel = create_element(fastn_js::ElementKind::Document, parent, index, rdata);
        component_statements.push(fastn_js::ComponentStatement::CreateKernel(kernel.clone()));

        if let Some(ref breakpoint_width) = self.breakpoint_width {
            component_statements.push(fastn_js::ComponentStatement::SetProperty(
                breakpoint_width.to_set_property(
                    fastn_js::PropertyKind::BreakpointWidth,
                    doc,
                    kernel.name.as_str(),
                    rdata,
                ),
            ));
        }
        component_statements.extend(self.container.to_component_statements(
            doc,
            rdata,
            has_rive_components,
            false,
        ));

        component_statements.extend(self.metadata.to_component_statements(
            doc,
            rdata,
            kernel.name.as_str(),
        ));

        if should_return {
            component_statements.push(fastn_js::ComponentStatement::Return {
                component_name: kernel.name,
            });
        }
        component_statements
    }
}

impl DocumentMeta {
    pub fn from(
        properties: &[ftd::interpreter::Property],
        arguments: &[ftd::interpreter::Argument],
    ) -> DocumentMeta {
        DocumentMeta {
            favicon: ftd::js::value::get_optional_js_value("favicon", properties, arguments),
            title: ftd::js::value::get_optional_js_value("title", properties, arguments),
            og_title: ftd::js::value::get_optional_js_value("og-title", properties, arguments),
            twitter_title: ftd::js::value::get_optional_js_value(
                "twitter-title",
                properties,
                arguments,
            ),
            description: ftd::js::value::get_optional_js_value(
                "description",
                properties,
                arguments,
            ),
            og_description: ftd::js::value::get_optional_js_value(
                "og-description",
                properties,
                arguments,
            ),
            twitter_description: ftd::js::value::get_optional_js_value(
                "twitter-description",
                properties,
                arguments,
            ),
            og_image: ftd::js::value::get_optional_js_value("og-image", properties, arguments),
            twitter_image: ftd::js::value::get_optional_js_value(
                "twitter-image",
                properties,
                arguments,
            ),
            theme_color: ftd::js::value::get_optional_js_value(
                "theme-color",
                properties,
                arguments,
            ),
        }
    }

    pub fn has_self_reference(&self, value: &ftd::js::Value) -> bool {
        if let ftd::js::Value::Reference(reference) = value {
            return reference.name.starts_with("ftd#document");
        }
        false
    }

    pub fn set_property_value_with_self_reference(
        &self,
        value: &ftd::js::Value,
        value_kind: fastn_js::PropertyKind,
        referenced_value: &Option<ftd::js::Value>,
        component_statements: &mut Vec<fastn_js::ComponentStatement>,
        doc: &ftd::interpreter::TDoc,
        rdata: &ftd::js::ResolverData,
        element_name: &str,
    ) {
        if self.has_self_reference(value) {
            if let Some(ref referenced_value) = referenced_value {
                component_statements.push(fastn_js::ComponentStatement::SetProperty(
                    referenced_value.to_set_property(value_kind, doc, element_name, rdata),
                ));
            }
        } else {
            component_statements.push(fastn_js::ComponentStatement::SetProperty(
                value.to_set_property(value_kind, doc, element_name, rdata),
            ));
        }
    }

    pub(crate) fn to_component_statements(
        &self,
        doc: &ftd::interpreter::TDoc,
        rdata: &ftd::js::ResolverData,
        element_name: &str,
    ) -> Vec<fastn_js::ComponentStatement> {
        let mut component_statements = vec![];

        if let Some(ref favicon) = self.favicon {
            component_statements.push(fastn_js::ComponentStatement::SetProperty(
                favicon.to_set_property(fastn_js::PropertyKind::Favicon, doc, element_name, rdata),
            ));
        }

        if let Some(ref title) = self.title {
            component_statements.push(fastn_js::ComponentStatement::SetProperty(
                title.to_set_property(fastn_js::PropertyKind::MetaTitle, doc, element_name, rdata),
            ));
        }

        if let Some(ref og_title) = self.og_title {
            self.set_property_value_with_self_reference(
                og_title,
                fastn_js::PropertyKind::MetaOGTitle,
                &self.title,
                &mut component_statements,
                doc,
                rdata,
                element_name,
            );
        }

        if let Some(ref twitter_title) = self.twitter_title {
            self.set_property_value_with_self_reference(
                twitter_title,
                fastn_js::PropertyKind::MetaTwitterTitle,
                &self.title,
                &mut component_statements,
                doc,
                rdata,
                element_name,
            );
        }

        if let Some(ref description) = self.description {
            component_statements.push(fastn_js::ComponentStatement::SetProperty(
                description.to_set_property(
                    fastn_js::PropertyKind::MetaDescription,
                    doc,
                    element_name,
                    rdata,
                ),
            ));
        }

        if let Some(ref og_description) = self.og_description {
            self.set_property_value_with_self_reference(
                og_description,
                fastn_js::PropertyKind::MetaOGDescription,
                &self.description,
                &mut component_statements,
                doc,
                rdata,
                element_name,
            );
        }

        if let Some(ref twitter_description) = self.twitter_description {
            self.set_property_value_with_self_reference(
                twitter_description,
                fastn_js::PropertyKind::MetaTwitterDescription,
                &self.description,
                &mut component_statements,
                doc,
                rdata,
                element_name,
            );
        }

        if let Some(ref og_image) = self.og_image {
            component_statements.push(fastn_js::ComponentStatement::SetProperty(
                og_image.to_set_property(
                    fastn_js::PropertyKind::MetaOGImage,
                    doc,
                    element_name,
                    rdata,
                ),
            ));
        }

        if let Some(ref twitter_image) = self.twitter_image {
            self.set_property_value_with_self_reference(
                twitter_image,
                fastn_js::PropertyKind::MetaTwitterImage,
                &self.og_image,
                &mut component_statements,
                doc,
                rdata,
                element_name,
            );
        }

        if let Some(ref theme_color) = self.theme_color {
            component_statements.push(fastn_js::ComponentStatement::SetProperty(
                theme_color.to_set_property(
                    fastn_js::PropertyKind::MetaThemeColor,
                    doc,
                    element_name,
                    rdata,
                ),
            ));
        }

        component_statements
    }
}

impl Column {
    pub fn from(component: &ftd::interpreter::Component) -> Column {
        let component_definition = ftd::interpreter::default::get_default_bag()
            .get("ftd#column")
            .unwrap()
            .clone()
            .component()
            .unwrap();

        Column {
            container: Container::from(
                component.properties.as_slice(),
                component_definition.arguments.as_slice(),
            ),
            container_properties: ContainerProperties::from(
                component.properties.as_slice(),
                component_definition.arguments.as_slice(),
            ),
            common: Common::from(
                component.properties.as_slice(),
                component_definition.arguments.as_slice(),
                component.events.as_slice(),
            ),
        }
    }

    pub fn to_component_statements(
        &self,
        parent: &str,
        index: usize,
        doc: &ftd::interpreter::TDoc,
        rdata: &mut ftd::js::ResolverData,
        should_return: bool,
        has_rive_components: &mut bool,
    ) -> Vec<fastn_js::ComponentStatement> {
        let mut component_statements = vec![];
        let kernel = create_element(fastn_js::ElementKind::Column, parent, index, rdata);
        component_statements.push(fastn_js::ComponentStatement::CreateKernel(kernel.clone()));
        component_statements.extend(self.common.to_set_properties(
            kernel.name.as_str(),
            doc,
            rdata,
        ));

        component_statements.extend(self.container_properties.to_set_properties(
            kernel.name.as_str(),
            doc,
            rdata,
        ));

        component_statements.extend(self.container.to_component_statements(
            doc,
            rdata,
            has_rive_components,
            false,
        ));

        if should_return {
            component_statements.push(fastn_js::ComponentStatement::Return {
                component_name: kernel.name,
            });
        }
        component_statements
    }
}

impl Row {
    pub fn from(component: &ftd::interpreter::Component) -> Row {
        let component_definition = ftd::interpreter::default::get_default_bag()
            .get("ftd#row")
            .unwrap()
            .clone()
            .component()
            .unwrap();
        Row {
            container: Container::from(
                component.properties.as_slice(),
                component_definition.arguments.as_slice(),
            ),
            container_properties: ContainerProperties::from(
                component.properties.as_slice(),
                component_definition.arguments.as_slice(),
            ),
            common: Common::from(
                component.properties.as_slice(),
                component_definition.arguments.as_slice(),
                component.events.as_slice(),
            ),
        }
    }

    pub fn to_component_statements(
        &self,
        parent: &str,
        index: usize,
        doc: &ftd::interpreter::TDoc,
        rdata: &mut ftd::js::ResolverData,
        should_return: bool,
        has_rive_components: &mut bool,
    ) -> Vec<fastn_js::ComponentStatement> {
        let mut component_statements = vec![];
        let kernel = create_element(fastn_js::ElementKind::Row, parent, index, rdata);
        component_statements.push(fastn_js::ComponentStatement::CreateKernel(kernel.clone()));

        component_statements.extend(self.common.to_set_properties(
            kernel.name.as_str(),
            doc,
            rdata,
        ));

        component_statements.extend(self.container_properties.to_set_properties(
            kernel.name.as_str(),
            doc,
            rdata,
        ));

        component_statements.extend(self.container.to_component_statements(
            doc,
            rdata,
            has_rive_components,
            false,
        ));

        if should_return {
            component_statements.push(fastn_js::ComponentStatement::Return {
                component_name: kernel.name,
            });
        }
        component_statements
    }
}

impl ContainerElement {
    pub fn from(component: &ftd::interpreter::Component) -> ContainerElement {
        let component_definition = ftd::interpreter::default::get_default_bag()
            .get("ftd#container")
            .unwrap()
            .clone()
            .component()
            .unwrap();

        ContainerElement {
            container: Container::from(
                component.properties.as_slice(),
                component_definition.arguments.as_slice(),
            ),
            common: Common::from(
                component.properties.as_slice(),
                component_definition.arguments.as_slice(),
                component.events.as_slice(),
            ),
        }
    }

    pub fn to_component_statements(
        &self,
        parent: &str,
        index: usize,
        doc: &ftd::interpreter::TDoc,
        rdata: &mut ftd::js::ResolverData,
        should_return: bool,
        has_rive_components: &mut bool,
    ) -> Vec<fastn_js::ComponentStatement> {
        let mut component_statements = vec![];
        let kernel = create_element(
            fastn_js::ElementKind::ContainerElement,
            parent,
            index,
            rdata,
        );
        component_statements.push(fastn_js::ComponentStatement::CreateKernel(kernel.clone()));

        component_statements.extend(self.common.to_set_properties(
            kernel.name.as_str(),
            doc,
            rdata,
        ));

        component_statements.extend(self.container.to_component_statements(
            doc,
            rdata,
            has_rive_components,
            false,
        ));

        if should_return {
            component_statements.push(fastn_js::ComponentStatement::Return {
                component_name: kernel.name,
            });
        }
        component_statements
    }
}

#[derive(Debug)]
pub struct Device {
    pub container: Container,
    pub device: fastn_js::DeviceType,
}

impl Device {
    pub fn from(component: &ftd::interpreter::Component, device: &str) -> Device {
        let component_definition = ftd::interpreter::default::get_default_bag()
            .get(device)
            .unwrap()
            .clone()
            .component()
            .unwrap();
        Device {
            container: Container::from(
                component.properties.as_slice(),
                component_definition.arguments.as_slice(),
            ),
            device: device.into(),
        }
    }

    pub fn to_component_statements(
        &self,
        parent: &str,
        index: usize,
        doc: &ftd::interpreter::TDoc,
        rdata: &mut ftd::js::ResolverData,
        should_return: bool,
        has_rive_components: &mut bool,
    ) -> Vec<fastn_js::ComponentStatement> {
        let mut component_statements = vec![];
        if let Some(ref device) = rdata.device {
            if device.ne(&self.device) {
                return component_statements;
            }
        }

        let kernel = create_element(
            fastn_js::ElementKind::Device,
            fastn_js::FUNCTION_PARENT,
            index,
            rdata,
        );
        component_statements.push(fastn_js::ComponentStatement::CreateKernel(kernel.clone()));

        component_statements.extend(self.container.to_component_statements(
            doc,
            &rdata.clone_with_new_device(&Some(self.device.clone())),
            has_rive_components,
            true,
        ));
        component_statements.push(fastn_js::ComponentStatement::Return {
            component_name: kernel.name,
        });

        vec![fastn_js::ComponentStatement::DeviceBlock(
            fastn_js::DeviceBlock {
                device: self.device.to_owned(),
                statements: component_statements,
                parent: parent.to_string(),
                should_return,
            },
        )]
    }
}

#[derive(Debug)]
pub struct TextCommon {
    pub text_transform: Option<ftd::js::Value>,
    pub text_indent: Option<ftd::js::Value>,
    pub text_align: Option<ftd::js::Value>,
    pub line_clamp: Option<ftd::js::Value>,
    pub style: Option<ftd::js::Value>,
    pub display: Option<ftd::js::Value>,
    pub link_color: Option<ftd::js::Value>,
    pub text_shadow: Option<ftd::js::Value>,
}

impl TextCommon {
    pub fn from(
        properties: &[ftd::interpreter::Property],
        arguments: &[ftd::interpreter::Argument],
    ) -> TextCommon {
        TextCommon {
            text_transform: ftd::js::value::get_optional_js_value(
                "text-transform",
                properties,
                arguments,
            ),
            text_indent: ftd::js::value::get_optional_js_value(
                "text-indent",
                properties,
                arguments,
            ),
            text_align: ftd::js::value::get_optional_js_value("text-align", properties, arguments),
            line_clamp: ftd::js::value::get_optional_js_value("line-clamp", properties, arguments),
            style: ftd::js::value::get_optional_js_value("style", properties, arguments),
            display: ftd::js::value::get_optional_js_value("display", properties, arguments),
            link_color: ftd::js::value::get_optional_js_value("link-color", properties, arguments),
            text_shadow: ftd::js::value::get_optional_js_value(
                "text-shadow",
                properties,
                arguments,
            ),
        }
    }

    pub fn to_set_properties(
        &self,
        element_name: &str,
        doc: &ftd::interpreter::TDoc,
        rdata: &ftd::js::ResolverData,
    ) -> Vec<fastn_js::ComponentStatement> {
        let mut component_statements = vec![];
        if let Some(ref transform) = self.text_transform {
            component_statements.push(fastn_js::ComponentStatement::SetProperty(
                transform.to_set_property(
                    fastn_js::PropertyKind::TextTransform,
                    doc,
                    element_name,
                    rdata,
                ),
            ));
        }
        if let Some(ref indent) = self.text_indent {
            component_statements.push(fastn_js::ComponentStatement::SetProperty(
                indent.to_set_property(
                    fastn_js::PropertyKind::TextIndent,
                    doc,
                    element_name,
                    rdata,
                ),
            ));
        }
        if let Some(ref align) = self.text_align {
            component_statements.push(fastn_js::ComponentStatement::SetProperty(
                align.to_set_property(fastn_js::PropertyKind::TextAlign, doc, element_name, rdata),
            ));
        }
        if let Some(ref clamp) = self.line_clamp {
            component_statements.push(fastn_js::ComponentStatement::SetProperty(
                clamp.to_set_property(fastn_js::PropertyKind::LineClamp, doc, element_name, rdata),
            ));
        }
        if let Some(ref style) = self.style {
            component_statements.push(fastn_js::ComponentStatement::SetProperty(
                style.to_set_property(fastn_js::PropertyKind::TextStyle, doc, element_name, rdata),
            ));
        }
        if let Some(ref display) = self.display {
            component_statements.push(fastn_js::ComponentStatement::SetProperty(
                display.to_set_property(fastn_js::PropertyKind::Display, doc, element_name, rdata),
            ));
        }
        if let Some(ref link_color) = self.link_color {
            component_statements.push(fastn_js::ComponentStatement::SetProperty(
                link_color.to_set_property(
                    fastn_js::PropertyKind::LinkColor,
                    doc,
                    element_name,
                    rdata,
                ),
            ));
        }
        if let Some(ref text_shadow) = self.text_shadow {
            component_statements.push(fastn_js::ComponentStatement::SetProperty(
                text_shadow.to_set_property(
                    fastn_js::PropertyKind::TextShadow,
                    doc,
                    element_name,
                    rdata,
                ),
            ));
        }
        component_statements
    }
}

#[derive(Debug)]
pub struct Rive {
    pub src: ftd::js::Value,
    pub canvas_width: Option<ftd::js::Value>,
    pub canvas_height: Option<ftd::js::Value>,
    pub state_machines: ftd::js::Value,
    pub autoplay: ftd::js::Value,
    pub artboard: Option<ftd::js::Value>,
    pub common: Common,
}

impl Rive {
    pub fn from(component: &ftd::interpreter::Component) -> Rive {
        let component_definition = ftd::interpreter::default::get_default_bag()
            .get("ftd#rive")
            .unwrap()
            .clone()
            .component()
            .unwrap();

        Rive {
            src: ftd::js::value::get_optional_js_value(
                "src",
                component.properties.as_slice(),
                component_definition.arguments.as_slice(),
            )
            .unwrap(),
            canvas_width: ftd::js::value::get_optional_js_value(
                "canvas-width",
                component.properties.as_slice(),
                component_definition.arguments.as_slice(),
            ),
            canvas_height: ftd::js::value::get_optional_js_value(
                "canvas-height",
                component.properties.as_slice(),
                component_definition.arguments.as_slice(),
            ),
            state_machines: ftd::js::value::get_optional_js_value_with_default(
                "state-machine",
                component.properties.as_slice(),
                component_definition.arguments.as_slice(),
            )
            .unwrap(),
            autoplay: ftd::js::value::get_optional_js_value_with_default(
                "autoplay",
                component.properties.as_slice(),
                component_definition.arguments.as_slice(),
            )
            .unwrap(),
            artboard: ftd::js::value::get_optional_js_value(
                "artboard",
                component.properties.as_slice(),
                component_definition.arguments.as_slice(),
            ),
            common: Common::from(
                component.properties.as_slice(),
                component_definition.arguments.as_slice(),
                component.events.as_slice(),
            ),
        }
    }

    pub fn to_component_statements(
        &self,
        parent: &str,
        index: usize,
        doc: &ftd::interpreter::TDoc,
        rdata: &mut ftd::js::ResolverData,
        should_return: bool,
    ) -> Vec<fastn_js::ComponentStatement> {
        let mut component_statements = vec![];
        let kernel = create_element(fastn_js::ElementKind::Rive, parent, index, rdata);
        component_statements.push(fastn_js::ComponentStatement::CreateKernel(kernel.clone()));

        let rive_name = self
            .common
            .id
            .as_ref()
            .and_then(|v| v.get_string_data())
            .map(|v| {
                format!(
                    indoc::indoc! {"
                        ftd.riveNodes[`{rive_name}__${{ftd.device.get()}}`] = {canvas};
                    "},
                    rive_name = v,
                    canvas = kernel.name,
                )
            });

        let rive_events = ftd::js::utils::get_rive_event(
            self.common.events.as_slice(),
            doc,
            rdata,
            kernel.name.as_str(),
        );

        component_statements.push(fastn_js::ComponentStatement::AnyBlock(format!(
            indoc::indoc! {"
                let extraData = {canvas}.getExtraData();
                extraData.rive = new rive.Rive({{
                    src: fastn_utils.getFlattenStaticValue({src}),
                    canvas: {canvas}.getNode(),
                    autoplay: {get_static_value}({autoplay}),
                    stateMachines: fastn_utils.getFlattenStaticValue({state_machines}),
                    artboard: {artboard},
                    onLoad: (_) => {{
                        extraData.rive.resizeDrawingSurfaceToCanvas();
                    }},
                    {rive_events}
                }});
                {rive_name_content}
            "},
            src = self.src.to_set_property_value(doc, rdata).to_js(),
            canvas = kernel.name,
            get_static_value = fastn_js::GET_STATIC_VALUE,
            autoplay = self.autoplay.to_set_property_value(doc, rdata).to_js(),
            state_machines = self
                .state_machines
                .to_set_property_value(doc, rdata)
                .to_js(),
            artboard = self
                .artboard
                .as_ref()
                .map(|v| v.to_set_property_value(doc, rdata).to_js())
                .unwrap_or_else(|| "null".to_string()),
            rive_events = rive_events,
            rive_name_content = rive_name.unwrap_or_default()
        )));

        component_statements.extend(self.common.to_set_properties(
            kernel.name.as_str(),
            doc,
            rdata,
        ));

        if should_return {
            component_statements.push(fastn_js::ComponentStatement::Return {
                component_name: kernel.name,
            });
        }
        component_statements
    }
}

#[derive(Debug)]
pub struct Common {
    pub id: Option<ftd::js::Value>,
    pub region: Option<ftd::js::Value>,
    pub link: Option<ftd::js::Value>,
    pub link_rel: Option<ftd::js::Value>,
    pub open_in_new_tab: Option<ftd::js::Value>,
    pub align_self: Option<ftd::js::Value>,
    pub width: Option<ftd::js::Value>,
    pub height: Option<ftd::js::Value>,
    pub padding: Option<ftd::js::Value>,
    pub padding_horizontal: Option<ftd::js::Value>,
    pub padding_vertical: Option<ftd::js::Value>,
    pub padding_left: Option<ftd::js::Value>,
    pub padding_right: Option<ftd::js::Value>,
    pub padding_top: Option<ftd::js::Value>,
    pub padding_bottom: Option<ftd::js::Value>,
    pub margin: Option<ftd::js::Value>,
    pub margin_horizontal: Option<ftd::js::Value>,
    pub margin_vertical: Option<ftd::js::Value>,
    pub margin_left: Option<ftd::js::Value>,
    pub margin_right: Option<ftd::js::Value>,
    pub margin_top: Option<ftd::js::Value>,
    pub margin_bottom: Option<ftd::js::Value>,
    pub border_width: Option<ftd::js::Value>,
    pub border_top_width: Option<ftd::js::Value>,
    pub border_bottom_width: Option<ftd::js::Value>,
    pub border_left_width: Option<ftd::js::Value>,
    pub border_right_width: Option<ftd::js::Value>,
    pub border_radius: Option<ftd::js::Value>,
    pub border_top_left_radius: Option<ftd::js::Value>,
    pub border_top_right_radius: Option<ftd::js::Value>,
    pub border_bottom_left_radius: Option<ftd::js::Value>,
    pub border_bottom_right_radius: Option<ftd::js::Value>,
    pub border_style: Option<ftd::js::Value>,
    pub border_style_vertical: Option<ftd::js::Value>,
    pub border_style_horizontal: Option<ftd::js::Value>,
    pub border_left_style: Option<ftd::js::Value>,
    pub border_right_style: Option<ftd::js::Value>,
    pub border_top_style: Option<ftd::js::Value>,
    pub border_bottom_style: Option<ftd::js::Value>,
    pub border_color: Option<ftd::js::Value>,
    pub border_left_color: Option<ftd::js::Value>,
    pub border_right_color: Option<ftd::js::Value>,
    pub border_top_color: Option<ftd::js::Value>,
    pub border_bottom_color: Option<ftd::js::Value>,
    pub color: Option<ftd::js::Value>,
    pub background: Option<ftd::js::Value>,
    pub role: Option<ftd::js::Value>,
    pub z_index: Option<ftd::js::Value>,
    pub sticky: Option<ftd::js::Value>,
    pub top: Option<ftd::js::Value>,
    pub bottom: Option<ftd::js::Value>,
    pub left: Option<ftd::js::Value>,
    pub right: Option<ftd::js::Value>,
    pub overflow: Option<ftd::js::Value>,
    pub overflow_x: Option<ftd::js::Value>,
    pub overflow_y: Option<ftd::js::Value>,
    pub opacity: Option<ftd::js::Value>,
    pub cursor: Option<ftd::js::Value>,
    pub resize: Option<ftd::js::Value>,
    pub max_height: Option<ftd::js::Value>,
    pub max_width: Option<ftd::js::Value>,
    pub min_height: Option<ftd::js::Value>,
    pub min_width: Option<ftd::js::Value>,
    pub whitespace: Option<ftd::js::Value>,
    pub classes: Option<ftd::js::Value>,
    pub anchor: Option<ftd::js::Value>,
    pub shadow: Option<ftd::js::Value>,
    pub css: Option<ftd::js::Value>,
    pub js: Option<ftd::js::Value>,
    pub events: Vec<ftd::interpreter::Event>,
    pub selectable: Option<ftd::js::Value>,
    pub mask: Option<ftd::js::Value>,
}

impl Common {
    pub fn from(
        properties: &[ftd::interpreter::Property],
        arguments: &[ftd::interpreter::Argument],
        events: &[ftd::interpreter::Event],
    ) -> Common {
        Common {
            id: ftd::js::value::get_optional_js_value("id", properties, arguments),
            css: ftd::js::value::get_optional_js_value("css", properties, arguments),
            js: ftd::js::value::get_optional_js_value("js", properties, arguments),
            region: ftd::js::value::get_optional_js_value("region", properties, arguments),
            link: ftd::js::value::get_optional_js_value("link", properties, arguments),
            link_rel: ftd::js::value::get_optional_js_value("rel", properties, arguments),
            open_in_new_tab: ftd::js::value::get_optional_js_value(
                "open-in-new-tab",
                properties,
                arguments,
            ),
            anchor: ftd::js::value::get_optional_js_value("anchor", properties, arguments),
            classes: ftd::js::value::get_optional_js_value("classes", properties, arguments),
            align_self: ftd::js::value::get_optional_js_value("align-self", properties, arguments),
            width: ftd::js::value::get_optional_js_value("width", properties, arguments),
            height: ftd::js::value::get_optional_js_value("height", properties, arguments),
            padding: ftd::js::value::get_optional_js_value("padding", properties, arguments),
            padding_horizontal: ftd::js::value::get_optional_js_value(
                "padding-horizontal",
                properties,
                arguments,
            ),
            padding_vertical: ftd::js::value::get_optional_js_value(
                "padding-vertical",
                properties,
                arguments,
            ),
            padding_left: ftd::js::value::get_optional_js_value(
                "padding-left",
                properties,
                arguments,
            ),
            padding_right: ftd::js::value::get_optional_js_value(
                "padding-right",
                properties,
                arguments,
            ),
            padding_top: ftd::js::value::get_optional_js_value(
                "padding-top",
                properties,
                arguments,
            ),
            padding_bottom: ftd::js::value::get_optional_js_value(
                "padding-bottom",
                properties,
                arguments,
            ),
            margin: ftd::js::value::get_optional_js_value("margin", properties, arguments),
            margin_horizontal: ftd::js::value::get_optional_js_value(
                "margin-horizontal",
                properties,
                arguments,
            ),
            margin_vertical: ftd::js::value::get_optional_js_value(
                "margin-vertical",
                properties,
                arguments,
            ),
            margin_left: ftd::js::value::get_optional_js_value(
                "margin-left",
                properties,
                arguments,
            ),
            margin_right: ftd::js::value::get_optional_js_value(
                "margin-right",
                properties,
                arguments,
            ),
            margin_top: ftd::js::value::get_optional_js_value("margin-top", properties, arguments),
            margin_bottom: ftd::js::value::get_optional_js_value(
                "margin-bottom",
                properties,
                arguments,
            ),
            border_width: ftd::js::value::get_optional_js_value(
                "border-width",
                properties,
                arguments,
            ),
            border_top_width: ftd::js::value::get_optional_js_value(
                "border-top-width",
                properties,
                arguments,
            ),
            border_bottom_width: ftd::js::value::get_optional_js_value(
                "border-bottom-width",
                properties,
                arguments,
            ),
            border_left_width: ftd::js::value::get_optional_js_value(
                "border-left-width",
                properties,
                arguments,
            ),
            border_right_width: ftd::js::value::get_optional_js_value(
                "border-right-width",
                properties,
                arguments,
            ),
            border_radius: ftd::js::value::get_optional_js_value(
                "border-radius",
                properties,
                arguments,
            ),
            border_top_left_radius: ftd::js::value::get_optional_js_value(
                "border-top-left-radius",
                properties,
                arguments,
            ),
            border_top_right_radius: ftd::js::value::get_optional_js_value(
                "border-top-right-radius",
                properties,
                arguments,
            ),
            border_bottom_left_radius: ftd::js::value::get_optional_js_value(
                "border-bottom-left-radius",
                properties,
                arguments,
            ),
            border_bottom_right_radius: ftd::js::value::get_optional_js_value(
                "border-bottom-right-radius",
                properties,
                arguments,
            ),
            border_style: ftd::js::value::get_optional_js_value(
                "border-style",
                properties,
                arguments,
            ),
            border_style_vertical: ftd::js::value::get_optional_js_value(
                "border-style-vertical",
                properties,
                arguments,
            ),
            border_style_horizontal: ftd::js::value::get_optional_js_value(
                "border-style-horizontal",
                properties,
                arguments,
            ),
            border_left_style: ftd::js::value::get_optional_js_value(
                "border-style-left",
                properties,
                arguments,
            ),
            border_right_style: ftd::js::value::get_optional_js_value(
                "border-style-right",
                properties,
                arguments,
            ),
            border_top_style: ftd::js::value::get_optional_js_value(
                "border-style-top",
                properties,
                arguments,
            ),
            border_bottom_style: ftd::js::value::get_optional_js_value(
                "border-style-bottom",
                properties,
                arguments,
            ),
            border_color: ftd::js::value::get_optional_js_value(
                "border-color",
                properties,
                arguments,
            ),
            border_left_color: ftd::js::value::get_optional_js_value(
                "border-left-color",
                properties,
                arguments,
            ),
            border_right_color: ftd::js::value::get_optional_js_value(
                "border-right-color",
                properties,
                arguments,
            ),
            border_top_color: ftd::js::value::get_optional_js_value(
                "border-top-color",
                properties,
                arguments,
            ),
            border_bottom_color: ftd::js::value::get_optional_js_value(
                "border-bottom-color",
                properties,
                arguments,
            ),
            color: ftd::js::value::get_optional_js_value("color", properties, arguments),
            background: ftd::js::value::get_optional_js_value("background", properties, arguments),
            role: ftd::js::value::get_optional_js_value("role", properties, arguments),
            z_index: ftd::js::value::get_optional_js_value("z-index", properties, arguments),
            sticky: ftd::js::value::get_optional_js_value("sticky", properties, arguments),
            top: ftd::js::value::get_optional_js_value("top", properties, arguments),
            bottom: ftd::js::value::get_optional_js_value("bottom", properties, arguments),
            left: ftd::js::value::get_optional_js_value("left", properties, arguments),
            right: ftd::js::value::get_optional_js_value("right", properties, arguments),
            overflow: ftd::js::value::get_optional_js_value("overflow", properties, arguments),
            overflow_x: ftd::js::value::get_optional_js_value("overflow-x", properties, arguments),
            overflow_y: ftd::js::value::get_optional_js_value("overflow-y", properties, arguments),
            opacity: ftd::js::value::get_optional_js_value("opacity", properties, arguments),
            cursor: ftd::js::value::get_optional_js_value("cursor", properties, arguments),
            resize: ftd::js::value::get_optional_js_value("resize", properties, arguments),
            max_height: ftd::js::value::get_optional_js_value("max-height", properties, arguments),
            max_width: ftd::js::value::get_optional_js_value("max-width", properties, arguments),
            min_height: ftd::js::value::get_optional_js_value("min-height", properties, arguments),
            min_width: ftd::js::value::get_optional_js_value("min-width", properties, arguments),
            whitespace: ftd::js::value::get_optional_js_value("white-space", properties, arguments),
            shadow: ftd::js::value::get_optional_js_value("shadow", properties, arguments),
            selectable: ftd::js::value::get_optional_js_value("selectable", properties, arguments),
            mask: ftd::js::value::get_optional_js_value("mask", properties, arguments),
            events: events.to_vec(),
        }
    }

    pub fn to_set_properties_without_role(
        &self,
        element_name: &str,
        doc: &ftd::interpreter::TDoc,
        rdata: &ftd::js::ResolverData,
    ) -> Vec<fastn_js::ComponentStatement> {
        let mut component_statements = vec![];
        for event in self.events.iter() {
            if let Some(event_handler) = event.to_event_handler_js(element_name, doc, rdata) {
                component_statements
                    .push(fastn_js::ComponentStatement::AddEventHandler(event_handler));
            }
        }
        if let Some(ref id) = self.id {
            component_statements.push(fastn_js::ComponentStatement::SetProperty(
                id.to_set_property(fastn_js::PropertyKind::Id, doc, element_name, rdata),
            ));
        }
        if let Some(ref external_css) = self.css {
            component_statements.push(fastn_js::ComponentStatement::SetProperty(
                external_css.to_set_property(fastn_js::PropertyKind::Css, doc, element_name, rdata),
            ));
        }
        if let Some(ref external_js) = self.js {
            component_statements.push(fastn_js::ComponentStatement::SetProperty(
                external_js.to_set_property(fastn_js::PropertyKind::Js, doc, element_name, rdata),
            ));
        }
        if let Some(ref region) = self.region {
            component_statements.push(fastn_js::ComponentStatement::SetProperty(
                region.to_set_property(fastn_js::PropertyKind::Region, doc, element_name, rdata),
            ));
        }
        if let Some(ref align_self) = self.align_self {
            component_statements.push(fastn_js::ComponentStatement::SetProperty(
                align_self.to_set_property(
                    fastn_js::PropertyKind::AlignSelf,
                    doc,
                    element_name,
                    rdata,
                ),
            ));
        }
        if let Some(ref classes) = self.classes {
            component_statements.push(fastn_js::ComponentStatement::SetProperty(
                classes.to_set_property(fastn_js::PropertyKind::Classes, doc, element_name, rdata),
            ));
        }
        if let Some(ref anchor) = self.anchor {
            component_statements.push(fastn_js::ComponentStatement::SetProperty(
                anchor.to_set_property(fastn_js::PropertyKind::Anchor, doc, element_name, rdata),
            ));
        }
        if let Some(ref width) = self.width {
            component_statements.push(fastn_js::ComponentStatement::SetProperty(
                width.to_set_property(fastn_js::PropertyKind::Width, doc, element_name, rdata),
            ));
        }
        if let Some(ref height) = self.height {
            component_statements.push(fastn_js::ComponentStatement::SetProperty(
                height.to_set_property(fastn_js::PropertyKind::Height, doc, element_name, rdata),
            ));
        }
        if let Some(ref padding) = self.padding {
            component_statements.push(fastn_js::ComponentStatement::SetProperty(
                padding.to_set_property(fastn_js::PropertyKind::Padding, doc, element_name, rdata),
            ));
        }
        if let Some(ref padding_horizontal) = self.padding_horizontal {
            component_statements.push(fastn_js::ComponentStatement::SetProperty(
                padding_horizontal.to_set_property(
                    fastn_js::PropertyKind::PaddingHorizontal,
                    doc,
                    element_name,
                    rdata,
                ),
            ));
        }
        if let Some(ref padding_vertical) = self.padding_vertical {
            component_statements.push(fastn_js::ComponentStatement::SetProperty(
                padding_vertical.to_set_property(
                    fastn_js::PropertyKind::PaddingVertical,
                    doc,
                    element_name,
                    rdata,
                ),
            ));
        }
        if let Some(ref padding_left) = self.padding_left {
            component_statements.push(fastn_js::ComponentStatement::SetProperty(
                padding_left.to_set_property(
                    fastn_js::PropertyKind::PaddingLeft,
                    doc,
                    element_name,
                    rdata,
                ),
            ));
        }
        if let Some(ref padding_right) = self.padding_right {
            component_statements.push(fastn_js::ComponentStatement::SetProperty(
                padding_right.to_set_property(
                    fastn_js::PropertyKind::PaddingRight,
                    doc,
                    element_name,
                    rdata,
                ),
            ));
        }
        if let Some(ref padding_top) = self.padding_top {
            component_statements.push(fastn_js::ComponentStatement::SetProperty(
                padding_top.to_set_property(
                    fastn_js::PropertyKind::PaddingTop,
                    doc,
                    element_name,
                    rdata,
                ),
            ));
        }
        if let Some(ref padding_bottom) = self.padding_bottom {
            component_statements.push(fastn_js::ComponentStatement::SetProperty(
                padding_bottom.to_set_property(
                    fastn_js::PropertyKind::PaddingBottom,
                    doc,
                    element_name,
                    rdata,
                ),
            ));
        }
        if let Some(ref margin) = self.margin {
            component_statements.push(fastn_js::ComponentStatement::SetProperty(
                margin.to_set_property(fastn_js::PropertyKind::Margin, doc, element_name, rdata),
            ));
        }
        if let Some(ref margin_horizontal) = self.margin_horizontal {
            component_statements.push(fastn_js::ComponentStatement::SetProperty(
                margin_horizontal.to_set_property(
                    fastn_js::PropertyKind::MarginHorizontal,
                    doc,
                    element_name,
                    rdata,
                ),
            ));
        }
        if let Some(ref margin_vertical) = self.margin_vertical {
            component_statements.push(fastn_js::ComponentStatement::SetProperty(
                margin_vertical.to_set_property(
                    fastn_js::PropertyKind::MarginVertical,
                    doc,
                    element_name,
                    rdata,
                ),
            ));
        }
        if let Some(ref margin_left) = self.margin_left {
            component_statements.push(fastn_js::ComponentStatement::SetProperty(
                margin_left.to_set_property(
                    fastn_js::PropertyKind::MarginLeft,
                    doc,
                    element_name,
                    rdata,
                ),
            ));
        }
        if let Some(ref margin_right) = self.margin_right {
            component_statements.push(fastn_js::ComponentStatement::SetProperty(
                margin_right.to_set_property(
                    fastn_js::PropertyKind::MarginRight,
                    doc,
                    element_name,
                    rdata,
                ),
            ));
        }
        if let Some(ref margin_top) = self.margin_top {
            component_statements.push(fastn_js::ComponentStatement::SetProperty(
                margin_top.to_set_property(
                    fastn_js::PropertyKind::MarginTop,
                    doc,
                    element_name,
                    rdata,
                ),
            ));
        }
        if let Some(ref margin_bottom) = self.margin_bottom {
            component_statements.push(fastn_js::ComponentStatement::SetProperty(
                margin_bottom.to_set_property(
                    fastn_js::PropertyKind::MarginBottom,
                    doc,
                    element_name,
                    rdata,
                ),
            ));
        }
        if let Some(ref border_width) = self.border_width {
            component_statements.push(fastn_js::ComponentStatement::SetProperty(
                border_width.to_set_property(
                    fastn_js::PropertyKind::BorderWidth,
                    doc,
                    element_name,
                    rdata,
                ),
            ));
        }
        if let Some(ref border_top_width) = self.border_top_width {
            component_statements.push(fastn_js::ComponentStatement::SetProperty(
                border_top_width.to_set_property(
                    fastn_js::PropertyKind::BorderTopWidth,
                    doc,
                    element_name,
                    rdata,
                ),
            ));
        }
        if let Some(ref border_bottom_width) = self.border_bottom_width {
            component_statements.push(fastn_js::ComponentStatement::SetProperty(
                border_bottom_width.to_set_property(
                    fastn_js::PropertyKind::BorderBottomWidth,
                    doc,
                    element_name,
                    rdata,
                ),
            ));
        }
        if let Some(ref border_left_width) = self.border_left_width {
            component_statements.push(fastn_js::ComponentStatement::SetProperty(
                border_left_width.to_set_property(
                    fastn_js::PropertyKind::BorderLeftWidth,
                    doc,
                    element_name,
                    rdata,
                ),
            ));
        }
        if let Some(ref border_right_width) = self.border_right_width {
            component_statements.push(fastn_js::ComponentStatement::SetProperty(
                border_right_width.to_set_property(
                    fastn_js::PropertyKind::BorderRightWidth,
                    doc,
                    element_name,
                    rdata,
                ),
            ));
        }
        if let Some(ref border_radius) = self.border_radius {
            component_statements.push(fastn_js::ComponentStatement::SetProperty(
                border_radius.to_set_property(
                    fastn_js::PropertyKind::BorderRadius,
                    doc,
                    element_name,
                    rdata,
                ),
            ));
        }
        if let Some(ref border_top_left_radius) = self.border_top_left_radius {
            component_statements.push(fastn_js::ComponentStatement::SetProperty(
                border_top_left_radius.to_set_property(
                    fastn_js::PropertyKind::BorderTopLeftRadius,
                    doc,
                    element_name,
                    rdata,
                ),
            ));
        }
        if let Some(ref border_top_right_radius) = self.border_top_right_radius {
            component_statements.push(fastn_js::ComponentStatement::SetProperty(
                border_top_right_radius.to_set_property(
                    fastn_js::PropertyKind::BorderTopRightRadius,
                    doc,
                    element_name,
                    rdata,
                ),
            ));
        }
        if let Some(ref border_bottom_left_radius) = self.border_bottom_left_radius {
            component_statements.push(fastn_js::ComponentStatement::SetProperty(
                border_bottom_left_radius.to_set_property(
                    fastn_js::PropertyKind::BorderBottomLeftRadius,
                    doc,
                    element_name,
                    rdata,
                ),
            ));
        }
        if let Some(ref border_bottom_right_radius) = self.border_bottom_right_radius {
            component_statements.push(fastn_js::ComponentStatement::SetProperty(
                border_bottom_right_radius.to_set_property(
                    fastn_js::PropertyKind::BorderBottomRightRadius,
                    doc,
                    element_name,
                    rdata,
                ),
            ));
        }
        if let Some(ref border_style) = self.border_style {
            component_statements.push(fastn_js::ComponentStatement::SetProperty(
                border_style.to_set_property(
                    fastn_js::PropertyKind::BorderStyle,
                    doc,
                    element_name,
                    rdata,
                ),
            ));
        }
        if let Some(ref border_style_vertical) = self.border_style_vertical {
            component_statements.push(fastn_js::ComponentStatement::SetProperty(
                border_style_vertical.to_set_property(
                    fastn_js::PropertyKind::BorderStyleVertical,
                    doc,
                    element_name,
                    rdata,
                ),
            ));
        }
        if let Some(ref border_style_horizontal) = self.border_style_horizontal {
            component_statements.push(fastn_js::ComponentStatement::SetProperty(
                border_style_horizontal.to_set_property(
                    fastn_js::PropertyKind::BorderStyleHorizontal,
                    doc,
                    element_name,
                    rdata,
                ),
            ));
        }
        if let Some(ref border_left_style) = self.border_left_style {
            component_statements.push(fastn_js::ComponentStatement::SetProperty(
                border_left_style.to_set_property(
                    fastn_js::PropertyKind::BorderLeftStyle,
                    doc,
                    element_name,
                    rdata,
                ),
            ));
        }
        if let Some(ref border_right_style) = self.border_right_style {
            component_statements.push(fastn_js::ComponentStatement::SetProperty(
                border_right_style.to_set_property(
                    fastn_js::PropertyKind::BorderRightStyle,
                    doc,
                    element_name,
                    rdata,
                ),
            ));
        }
        if let Some(ref border_top_style) = self.border_top_style {
            component_statements.push(fastn_js::ComponentStatement::SetProperty(
                border_top_style.to_set_property(
                    fastn_js::PropertyKind::BorderTopStyle,
                    doc,
                    element_name,
                    rdata,
                ),
            ));
        }
        if let Some(ref border_bottom_style) = self.border_bottom_style {
            component_statements.push(fastn_js::ComponentStatement::SetProperty(
                border_bottom_style.to_set_property(
                    fastn_js::PropertyKind::BorderBottomStyle,
                    doc,
                    element_name,
                    rdata,
                ),
            ));
        }
        if let Some(ref border_color) = self.border_color {
            component_statements.push(fastn_js::ComponentStatement::SetProperty(
                border_color.to_set_property(
                    fastn_js::PropertyKind::BorderColor,
                    doc,
                    element_name,
                    rdata,
                ),
            ));
        }
        if let Some(ref border_top_color) = self.border_top_color {
            component_statements.push(fastn_js::ComponentStatement::SetProperty(
                border_top_color.to_set_property(
                    fastn_js::PropertyKind::BorderTopColor,
                    doc,
                    element_name,
                    rdata,
                ),
            ));
        }
        if let Some(ref border_bottom_color) = self.border_bottom_color {
            component_statements.push(fastn_js::ComponentStatement::SetProperty(
                border_bottom_color.to_set_property(
                    fastn_js::PropertyKind::BorderBottomColor,
                    doc,
                    element_name,
                    rdata,
                ),
            ));
        }
        if let Some(ref border_left_color) = self.border_left_color {
            component_statements.push(fastn_js::ComponentStatement::SetProperty(
                border_left_color.to_set_property(
                    fastn_js::PropertyKind::BorderLeftColor,
                    doc,
                    element_name,
                    rdata,
                ),
            ));
        }
        if let Some(ref border_right_color) = self.border_right_color {
            component_statements.push(fastn_js::ComponentStatement::SetProperty(
                border_right_color.to_set_property(
                    fastn_js::PropertyKind::BorderRightColor,
                    doc,
                    element_name,
                    rdata,
                ),
            ));
        }
        if let Some(ref overflow) = self.overflow {
            component_statements.push(fastn_js::ComponentStatement::SetProperty(
                overflow.to_set_property(
                    fastn_js::PropertyKind::Overflow,
                    doc,
                    element_name,
                    rdata,
                ),
            ));
        }
        if let Some(ref overflow_x) = self.overflow_x {
            component_statements.push(fastn_js::ComponentStatement::SetProperty(
                overflow_x.to_set_property(
                    fastn_js::PropertyKind::OverflowX,
                    doc,
                    element_name,
                    rdata,
                ),
            ));
        }
        if let Some(ref overflow_y) = self.overflow_y {
            component_statements.push(fastn_js::ComponentStatement::SetProperty(
                overflow_y.to_set_property(
                    fastn_js::PropertyKind::OverflowY,
                    doc,
                    element_name,
                    rdata,
                ),
            ));
        }
        if let Some(ref top) = self.top {
            component_statements.push(fastn_js::ComponentStatement::SetProperty(
                top.to_set_property(fastn_js::PropertyKind::Top, doc, element_name, rdata),
            ));
        }
        if let Some(ref bottom) = self.bottom {
            component_statements.push(fastn_js::ComponentStatement::SetProperty(
                bottom.to_set_property(fastn_js::PropertyKind::Bottom, doc, element_name, rdata),
            ));
        }
        if let Some(ref left) = self.left {
            component_statements.push(fastn_js::ComponentStatement::SetProperty(
                left.to_set_property(fastn_js::PropertyKind::Left, doc, element_name, rdata),
            ));
        }
        if let Some(ref right) = self.right {
            component_statements.push(fastn_js::ComponentStatement::SetProperty(
                right.to_set_property(fastn_js::PropertyKind::Right, doc, element_name, rdata),
            ));
        }
        if let Some(ref z_index) = self.z_index {
            component_statements.push(fastn_js::ComponentStatement::SetProperty(
                z_index.to_set_property(fastn_js::PropertyKind::ZIndex, doc, element_name, rdata),
            ));
        }
        if let Some(ref sticky) = self.sticky {
            component_statements.push(fastn_js::ComponentStatement::SetProperty(
                sticky.to_set_property(fastn_js::PropertyKind::Sticky, doc, element_name, rdata),
            ));
        }
        if let Some(ref color) = self.color {
            component_statements.push(fastn_js::ComponentStatement::SetProperty(
                color.to_set_property(fastn_js::PropertyKind::Color, doc, element_name, rdata),
            ));
        }
        if let Some(ref background) = self.background {
            component_statements.push(fastn_js::ComponentStatement::SetProperty(
                background.to_set_property(
                    fastn_js::PropertyKind::Background,
                    doc,
                    element_name,
                    rdata,
                ),
            ));
        }
        if let Some(ref opacity) = self.opacity {
            component_statements.push(fastn_js::ComponentStatement::SetProperty(
                opacity.to_set_property(fastn_js::PropertyKind::Opacity, doc, element_name, rdata),
            ));
        }
        if let Some(ref cursor) = self.cursor {
            component_statements.push(fastn_js::ComponentStatement::SetProperty(
                cursor.to_set_property(fastn_js::PropertyKind::Cursor, doc, element_name, rdata),
            ));
        }
        if let Some(ref resize) = self.resize {
            component_statements.push(fastn_js::ComponentStatement::SetProperty(
                resize.to_set_property(fastn_js::PropertyKind::Resize, doc, element_name, rdata),
            ));
        }
        if let Some(ref max_height) = self.max_height {
            component_statements.push(fastn_js::ComponentStatement::SetProperty(
                max_height.to_set_property(
                    fastn_js::PropertyKind::MaxHeight,
                    doc,
                    element_name,
                    rdata,
                ),
            ));
        }
        if let Some(ref min_height) = self.min_height {
            component_statements.push(fastn_js::ComponentStatement::SetProperty(
                min_height.to_set_property(
                    fastn_js::PropertyKind::MinHeight,
                    doc,
                    element_name,
                    rdata,
                ),
            ));
        }
        if let Some(ref max_width) = self.max_width {
            component_statements.push(fastn_js::ComponentStatement::SetProperty(
                max_width.to_set_property(
                    fastn_js::PropertyKind::MaxWidth,
                    doc,
                    element_name,
                    rdata,
                ),
            ));
        }
        if let Some(ref min_width) = self.min_width {
            component_statements.push(fastn_js::ComponentStatement::SetProperty(
                min_width.to_set_property(
                    fastn_js::PropertyKind::MinWidth,
                    doc,
                    element_name,
                    rdata,
                ),
            ));
        }
        if let Some(ref whitespace) = self.whitespace {
            component_statements.push(fastn_js::ComponentStatement::SetProperty(
                whitespace.to_set_property(
                    fastn_js::PropertyKind::WhiteSpace,
                    doc,
                    element_name,
                    rdata,
                ),
            ));
        }
        if let Some(ref shadow) = self.shadow {
            component_statements.push(fastn_js::ComponentStatement::SetProperty(
                shadow.to_set_property(fastn_js::PropertyKind::Shadow, doc, element_name, rdata),
            ));
        }
        if let Some(ref link) = self.link {
            component_statements.push(fastn_js::ComponentStatement::SetProperty(
                link.to_set_property(fastn_js::PropertyKind::Link, doc, element_name, rdata),
            ));
        }
        if let Some(ref link_rel) = self.link_rel {
            component_statements.push(fastn_js::ComponentStatement::SetProperty(
                link_rel.to_set_property(fastn_js::PropertyKind::LinkRel, doc, element_name, rdata),
            ));
        }
        if let Some(ref open_in_new_tab) = self.open_in_new_tab {
            component_statements.push(fastn_js::ComponentStatement::SetProperty(
                open_in_new_tab.to_set_property(
                    fastn_js::PropertyKind::OpenInNewTab,
                    doc,
                    element_name,
                    rdata,
                ),
            ));
        }
        if let Some(ref selectable) = self.selectable {
            component_statements.push(fastn_js::ComponentStatement::SetProperty(
                selectable.to_set_property(
                    fastn_js::PropertyKind::Selectable,
                    doc,
                    element_name,
                    rdata,
                ),
            ));
        }
        if let Some(ref mask) = self.mask {
            component_statements.push(fastn_js::ComponentStatement::SetProperty(
                mask.to_set_property(fastn_js::PropertyKind::Mask, doc, element_name, rdata),
            ));
        }
        component_statements
    }

    pub fn to_set_properties_with_text(
        &self,
        element_name: &str,
        doc: &ftd::interpreter::TDoc,
        rdata: &ftd::js::ResolverData,
        text_component_statement: fastn_js::ComponentStatement,
    ) -> Vec<fastn_js::ComponentStatement> {
        // Property dependencies
        // Role <- Text (Role for post_markdown_process) <- Region(Headings need text for auto ids)
        let mut component_statements = vec![];
        if let Some(ref role) = self.role {
            component_statements.push(fastn_js::ComponentStatement::SetProperty(
                role.to_set_property(fastn_js::PropertyKind::Role, doc, element_name, rdata),
            ));
        }
        component_statements.push(text_component_statement);
        component_statements.extend(self.to_set_properties_without_role(element_name, doc, rdata));
        component_statements
    }

    pub fn to_set_properties(
        &self,
        element_name: &str,
        doc: &ftd::interpreter::TDoc,
        rdata: &ftd::js::ResolverData,
    ) -> Vec<fastn_js::ComponentStatement> {
        let mut component_statements = vec![];
        component_statements.extend(self.to_set_properties_without_role(element_name, doc, rdata));
        if let Some(ref role) = self.role {
            component_statements.push(fastn_js::ComponentStatement::SetProperty(
                role.to_set_property(fastn_js::PropertyKind::Role, doc, element_name, rdata),
            ));
        }
        component_statements
    }
}

impl ftd::interpreter::Event {
    pub(crate) fn to_event_handler_js(
        &self,
        element_name: &str,
        doc: &ftd::interpreter::TDoc,
        rdata: &ftd::js::ResolverData,
    ) -> Option<fastn_js::EventHandler> {
        self.name
            .to_js_event_name()
            .map(|event| fastn_js::EventHandler {
                event,
                action: self.action.to_js_function(doc, rdata),
                element_name: element_name.to_string(),
            })
    }
}

impl ftd::interpreter::FunctionCall {
    pub(crate) fn to_js_function(
        &self,
        doc: &ftd::interpreter::TDoc,
        rdata: &ftd::js::ResolverData,
    ) -> fastn_js::Function {
        let mut parameters = vec![];
        let mut name = self.name.to_string();
        let mut function_name = fastn_js::FunctionData::Name(self.name.to_string());
        if let Some((default_module, module_variable_name)) = &self.module_name {
            function_name =
                fastn_js::FunctionData::Definition(fastn_js::SetPropertyValue::Reference(
                    ftd::js::utils::update_reference(name.as_str(), rdata),
                ));
            name = name.replace(
                format!("{module_variable_name}.").as_str(),
                format!("{default_module}#").as_str(),
            );
        }
        let function = doc.get_function(name.as_str(), self.line_number).unwrap();
        for argument in function.arguments {
            if let Some(value) = self.values.get(argument.name.as_str()) {
                parameters.push((
                    argument.name.to_string(),
                    value.to_value().to_set_property_value(doc, rdata),
                ));
            } else if argument.get_default_value().is_none() {
                panic!("Argument value not found {:?}", argument)
            }
        }
        fastn_js::Function {
            name: Box::from(function_name),
            parameters,
        }
    }
}

impl ftd::interpreter::EventName {
    fn to_js_event_name(&self) -> Option<fastn_js::Event> {
        use itertools::Itertools;

        match self {
            ftd::interpreter::EventName::Click => Some(fastn_js::Event::Click),
            ftd::interpreter::EventName::MouseEnter => Some(fastn_js::Event::MouseEnter),
            ftd::interpreter::EventName::MouseLeave => Some(fastn_js::Event::MouseLeave),
            ftd::interpreter::EventName::ClickOutside => Some(fastn_js::Event::ClickOutside),
            ftd::interpreter::EventName::GlobalKey(gk) => Some(fastn_js::Event::GlobalKey(
                gk.iter().map(|v| ftd::js::utils::to_key(v)).collect_vec(),
            )),
            ftd::interpreter::EventName::GlobalKeySeq(gk) => Some(fastn_js::Event::GlobalKeySeq(
                gk.iter().map(|v| ftd::js::utils::to_key(v)).collect_vec(),
            )),
            ftd::interpreter::EventName::Input => Some(fastn_js::Event::Input),
            ftd::interpreter::EventName::Change => Some(fastn_js::Event::Change),
            ftd::interpreter::EventName::Blur => Some(fastn_js::Event::Blur),
            ftd::interpreter::EventName::Focus => Some(fastn_js::Event::Focus),
            ftd::interpreter::EventName::RivePlay(_)
            | ftd::interpreter::EventName::RivePause(_)
            | ftd::interpreter::EventName::RiveStateChange(_) => None,
        }
    }
}

pub fn is_kernel(s: &str) -> bool {
    [
        "ftd#text",
        "ftd#row",
        "ftd#column",
        "ftd#integer",
        "ftd#decimal",
        "ftd#container",
        "ftd#boolean",
        "ftd#desktop",
        "ftd#mobile",
        "ftd#checkbox",
        "ftd#text-input",
        "ftd#iframe",
        "ftd#code",
        "ftd#image",
        "ftd#video",
        "ftd#rive",
        "ftd#document",
    ]
    .contains(&s)
}

pub(crate) fn is_rive_component(s: &str) -> bool {
    "ftd#rive".eq(s)
}

pub(crate) fn create_element(
    element_kind: fastn_js::ElementKind,
    parent: &str,
    index: usize,
    rdata: &mut ftd::js::ResolverData,
) -> fastn_js::Kernel {
    let kernel = fastn_js::Kernel::from_component(element_kind, parent, index);
    *rdata = rdata.clone_with_new_component_name(Some(kernel.name.to_string()));
    kernel
}
