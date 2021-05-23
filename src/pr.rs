use std::convert::{TryFrom, TryInto};

#[derive(PartialEq, Debug, Clone, serde_derive::Serialize, Default)]
pub struct PR {
    pub title: crate::Rendered,
    pub number: i32,
    pub github_repo: String,
    pub status: crate::ValueWithDefault<Status>,
    pub open_status: crate::ValueWithDefault<OpenStatus>,
    pub body: Option<crate::Rendered>,
}

impl PR {
    pub fn unique_id(&self) -> String {
        format!("{}/{}", self.github_repo.clone(), self.number.to_string())
    }
}

#[derive(PartialEq, Debug, Clone, serde_derive::Serialize, serde_derive::Deserialize)]
#[serde(tag = "type")]
pub enum Status {
    UnderDevelopment,
    ReviewPending,
    AsPerDocs,
}

impl ToString for Status {
    fn to_string(&self) -> String {
        match self {
            Status::UnderDevelopment => "under-development".to_string(),
            Status::ReviewPending => "review-pending".to_string(),
            Status::AsPerDocs => "as-per-docs".to_string(),
        }
    }
}

impl TryFrom<&str> for Status {
    type Error = crate::document::ParseError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        Ok(match value {
            "under-development" => Status::UnderDevelopment,
            "review-pending" => Status::ReviewPending,
            "as-per-docs" => Status::AsPerDocs,
            t => {
                return Err(crate::document::ParseError::ValidationError(t.to_string()));
            }
        })
    }
}

impl Default for Status {
    fn default() -> Self {
        Status::UnderDevelopment
    }
}

#[derive(PartialEq, Debug, Clone, serde_derive::Serialize)]
#[serde(tag = "type")]
pub enum OpenStatus {
    Open,
    Closed,
}

impl TryFrom<&str> for OpenStatus {
    type Error = crate::document::ParseError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        Ok(match value {
            "closed" => OpenStatus::Closed,
            "open" => OpenStatus::Open,
            t => {
                return Err(crate::document::ParseError::ValidationError(t.to_string()));
            }
        })
    }
}

impl Default for OpenStatus {
    fn default() -> Self {
        OpenStatus::Open
    }
}

impl PR {
    pub fn new(
        title: String,
        number: i32,
        github_repo: String,
        status: Status,
        open_status: OpenStatus,
        body: Option<String>,
    ) -> Self {
        Self {
            title: crate::Rendered::line(title.as_str()),
            number,
            github_repo,
            body: body.map(|x| crate::Rendered::from(&x)),
            status: crate::ValueWithDefault::Found(status),
            open_status: crate::ValueWithDefault::Found(open_status),
        }
    }

    pub fn from_p1(p1: &crate::p1::Section) -> Result<Self, crate::document::ParseError> {
        Ok(Self {
            title: match &p1.caption {
                Some(v) => crate::Rendered::line(v),
                None => return crate::document::err("caption is required for -- pr:")?,
            },
            number: p1.header.i32("number")?,
            github_repo: p1.header.string("github-repo")?,
            status: match p1.header.str_optional("status")? {
                None => crate::ValueWithDefault::Default(Status::default()),
                Some(v) => crate::ValueWithDefault::Found(v.try_into()?),
            },
            open_status: match p1
                .header
                .str_optional("open-status")?
                .or(p1.header.str_optional("open_status")?)
            {
                None => crate::ValueWithDefault::Default(OpenStatus::Open),
                Some("open") => crate::ValueWithDefault::Found(OpenStatus::Open),
                Some("closed") => crate::ValueWithDefault::Found(OpenStatus::Closed),
                Some(t) => {
                    return crate::document::err(
                        format!(
                            "unknown pr open_status: {}, allowed values: open, closed",
                            t
                        )
                        .as_str(),
                    )?;
                }
            },
            body: p1.body.clone().map(|x| crate::Rendered::from(x.as_str())),
        })
    }

    pub fn to_p1(&self) -> crate::p1::Section {
        let mut p1 = crate::p1::Section::with_name("pr")
            .and_caption(&self.title.original)
            .add_header("number", &self.number.to_string().as_str())
            .add_header("github-repo", self.github_repo.as_str())
            .and_optional_body(&self.body.as_ref().map(|x| x.original.to_string()));

        if let crate::ValueWithDefault::Found(status) = &self.status {
            p1 = p1.add_header("status", status.to_string().as_str());
        };
        match self.open_status {
            crate::ValueWithDefault::Found(OpenStatus::Open) => {
                p1.add_header("open-status", "open")
            }
            crate::ValueWithDefault::Found(OpenStatus::Closed) => {
                p1.add_header("open-status", "closed")
            }
            crate::ValueWithDefault::Default(_) => p1,
        }
    }
}
