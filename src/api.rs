use crate::document::ParseError;

#[derive(PartialEq, Debug, Clone, serde_derive::Serialize)]
pub enum Method {
    Get,
    Post,
    Put,
    Patch,
    Delete,
    Head,
    Trace,
}

impl Default for Method {
    fn default() -> Self {
        Method::Get
    }
}

impl ToString for Method {
    fn to_string(&self) -> String {
        self.as_str().into()
    }
}

impl Method {
    fn as_str(&self) -> &'static str {
        match self {
            Method::Get => "get",
            Method::Post => "post",
            Method::Put => "put",
            Method::Patch => "patch",
            Method::Delete => "delete",
            Method::Head => "head",
            Method::Trace => "trace",
        }
    }
    fn from(s: &str) -> Result<Method, ParseError> {
        Ok(match s.to_lowercase().as_str() {
            "get" => Method::Get,
            "post" => Method::Post,
            "put" => Method::Put,
            "patch" => Method::Patch,
            "delete" => Method::Delete,
            "head" => Method::Head,
            "trace" => Method::Trace,
            t => return Err(format!("unknown value: {}", t).into()),
        })
    }
}

/*
-- api-object: Pet Object
name: pet

Pet is the coolness.

--- parameter: name
type: String

Foo is awesome.

-- api: Get Pets
method: POST
url: /api/get-pets/$name

this api gets pets.

--- url-parameter: name
type: String

This is the name.

--- parameter: foo
type: String

Foo is awesome.

--- example: some example
lang: py

yoo

--- response:
format: xml

--- error: description of error
code: 404

{
    "message": "Some message"
}

 */

#[derive(PartialEq, Debug, Clone, serde_derive::Serialize, Default)]
pub struct Api {
    pub id: Option<String>,
    pub title: String,
    pub method: Method,
    pub url: String,
    pub description: std::collections::HashMap<String, String>,
    pub url_parameters: Vec<Parameter>,
    pub parameters: Vec<Parameter>,
    pub examples: Vec<Example>,
    pub errors: Vec<Error>,
}

impl ToString for Api {
    fn to_string(&self) -> String {
        self.to_p1().to_string().trim_end().to_string()
    }
}

impl Api {
    pub fn to_p1(&self) -> crate::p1::Section {
        let mut p1 = crate::p1::Section::with_name("api")
            .and_caption(self.title.as_str())
            .add_header("method", self.method.as_str())
            .add_header("url", self.url.as_str())
            .add_optional_header("id", &self.id);

        if let Some(c) = self.description.get("") {
            p1 = p1.and_body(c.as_str())
        }

        for (k, v) in self.description.iter() {
            if k.is_empty() {
                continue;
            }
            p1 = p1.add_sub_section(
                crate::p1::SubSection::with_name("description")
                    .add_header("lang", k)
                    .and_body(v),
            )
        }

        for p in self.url_parameters.iter() {
            p1 = p1.add_sub_section(p.to_p1("url-parameter"));
        }
        for p in self.parameters.iter() {
            p1 = p1.add_sub_section(p.to_p1("parameter"));
        }
        for p in self.examples.iter() {
            p1 = p1.add_sub_section(p.to_p1()); // TODO: this is wrong
        }

        p1
    }

    pub fn from_p1(p1: &crate::p1::Section) -> Result<Self, crate::document::ParseError> {
        let mut m = Api::default();
        m.id = p1.header.string_optional("id")?;
        m.title = match p1.caption {
            Some(ref c) => c.clone(),
            None => return Err("title is required".into()),
        };
        m.method = match p1.header.str_optional("method")? {
            Some(m) => Method::from(m)?,
            None => Method::default(),
        };
        m.url = p1.header.string("url")?;
        if let Some(ref b) = p1.body {
            m.description.insert("".to_string(), b.clone());
        }

        for s in p1.sub_sections.0.iter() {
            match s.name.as_str() {
                "parameter" => m.parameters.push(Parameter::from_p1(s)?),
                "url-parameter" => m.url_parameters.push(Parameter::from_p1(s)?),
                "example" => m.examples.push(Example::from_p1(s)?),
                "description" => {
                    if let Some(ref b) = s.body {
                        m.description
                            .insert(s.header.string("lang")?, b.to_string());
                    }
                }
                t => return Err(format!("unknown sub section: {}, valid values are parameter, url-parameter, example and description", t).into()),
            }
        }
        Ok(m)
    }
}

// -- comic:
// --- panel:
// character: betty
// posture: standing
//
// -- meme:
// id: pulp-fiction-where-is-everybody
// top: after i open vim
// bottom: asd

#[derive(PartialEq, Debug, Clone, serde_derive::Serialize, Default)]
pub struct ApiObject {
    pub id: Option<String>,
    pub title: Option<String>,
    pub name: String,
    pub description: String,
    pub properties: Vec<Parameter>,
}

impl ToString for ApiObject {
    fn to_string(&self) -> String {
        self.to_p1().to_string().trim_end().to_string()
    }
}

impl ApiObject {
    pub fn from_p1(p1: &crate::p1::Section) -> Result<Self, crate::document::ParseError> {
        let mut m = ApiObject {
            id: p1.header.string_optional("id")?,
            title: p1.caption.clone(),
            name: p1.header.string("name")?,
            description: p1.body.clone().unwrap_or_else(|| "".to_string()),
            ..Default::default()
        };
        for s in p1.sub_sections.0.iter() {
            m.properties.push(Parameter::from_p1(s)?);
        }
        Ok(m)
    }

    pub fn to_p1(&self) -> crate::p1::Section {
        todo!()
    }
}

#[derive(PartialEq, Debug, Clone, serde_derive::Serialize, Default)]
pub struct Parameter {
    pub name: String,
    #[serde(rename = "type")]
    pub type_: String,
    pub required: Option<bool>,
    pub default: Option<String>,
    pub description: Option<String>,
}

impl Parameter {
    fn from_p1(p1: &crate::p1::SubSection) -> Result<Self, crate::document::ParseError> {
        Ok(Parameter {
            name: p1.header.string("name")?,
            type_: p1.header.string("type")?,
            required: p1.header.bool_optional("required")?,
            default: p1.header.string_optional("default")?,
            description: p1.body.clone(),
        })
    }

    fn to_p1(&self, name: &str) -> crate::p1::SubSection {
        crate::p1::SubSection::with_name(name)
            .add_header("name", self.name.as_str())
            .add_header("type", self.type_.as_str())
            .add_optional_header_bool("required", self.required)
            .add_optional_header("default", &self.default)
            .and_optional_body(&self.description)
    }
}

#[derive(PartialEq, Debug, Clone, serde_derive::Serialize, Default)]
pub struct Example {
    pub title: Option<String>,
    pub description: Option<String>,
    pub lang: String,
    pub request: String,
    pub format: String,
    pub response: String,
}

impl Example {
    fn from_p1(p1: &crate::p1::SubSection) -> Result<Self, crate::document::ParseError> {
        Ok(Example {
            title: p1.caption.clone(),
            description: p1.body.clone(),
            lang: p1.header.string("lang")?,
            ..Default::default()
        })
    }

    fn to_p1(&self) -> crate::p1::SubSection {
        crate::p1::SubSection::with_name("example")
    }
}

#[derive(PartialEq, Debug, Clone, serde_derive::Serialize, Default)]
pub struct Error {
    code: String,
    message: String,
}
