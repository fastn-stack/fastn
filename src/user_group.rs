// identities to group, test also
#[derive(Debug, Clone, serde::Serialize)]
pub struct UserIdentity {
    pub key: String,
    pub value: String,
}

impl PartialEq for UserIdentity {
    fn eq(&self, other: &Self) -> bool {
        self.key.eq(other.key.as_str()) && self.value.eq(other.value.as_str())
    }
}

impl UserIdentity {
    pub fn from(key: &str, value: &str) -> Self {
        Self {
            key: key.to_string(),
            value: value.to_string(),
        }
    }
}

impl ToString for UserIdentity {
    fn to_string(&self) -> String {
        format!("{}: {}", self.key, self.value)
    }
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct UserGroup {
    pub title: Option<String>,
    pub id: String,
    pub identities: Vec<UserIdentity>,
    pub excluded_identities: Vec<UserIdentity>,

    /// if package name is abrark.com and it has user-group with id my-all-readers
    /// so import string will be abrark.com/my-all-readers
    pub groups: Vec<String>,
    pub excluded_groups: Vec<String>,
    pub description: Option<String>,
}

// TODO: Keys should be dynamic
/// This type is needed to deserialize ftd to rust

#[derive(serde::Deserialize, Debug)]
pub struct UserGroupTemp {
    // if package name is abrark.com and it has user-group with id my-all-readers
    // so group string will be abrark.com/my-all-readers
    // keys should be dynamic
    pub id: String,
    pub title: Option<String>,
    pub description: Option<String>,
    #[serde(rename = "group")]
    pub groups: Vec<String>,
    #[serde(rename = "-group")]
    pub excluded_group: Vec<String>,
    #[serde(rename = "email")]
    pub email: Vec<String>,
    #[serde(rename = "-email")]
    pub excluded_email: Vec<String>,
    #[serde(rename = "domain")]
    pub domain: Vec<String>,
    #[serde(rename = "-domain")]
    pub excluded_domain: Vec<String>,
    #[serde(rename = "telegram")]
    pub telegram: Vec<String>,
    #[serde(rename = "-telegram")]
    pub excluded_telegram: Vec<String>,
    #[serde(rename = "github")]
    pub github: Vec<String>,
    #[serde(rename = "-github")]
    pub excluded_github: Vec<String>,
    #[serde(rename = "github-team")]
    pub github_team: Vec<String>,
    #[serde(rename = "-github-team")]
    pub excluded_github_team: Vec<String>,
    #[serde(rename = "discord")]
    pub discord: Vec<String>,
    #[serde(rename = "-discord")]
    pub excluded_discord: Vec<String>,
}

#[derive(Debug, serde::Serialize)]
pub struct UserGroupCompat {
    id: String,
    title: Option<String>,
    description: Option<String>,
    // It will contain all group members, like group, email and -email, etc...
    #[serde(rename = "group-members")]
    group_members: Vec<fpm::library::full_sitemap::KeyValueData>,
    groups: Vec<String>,
}

impl UserGroup {
    pub fn to_group_compat(&self) -> UserGroupCompat {
        let mut group_members = vec![];

        // Group Identities
        group_members.extend(
            self.identities
                .clone()
                .into_iter()
                .map(|i| fpm::library::KeyValueData::from(i.key, i.value)),
        );

        group_members.extend(
            self.excluded_identities.iter().map(|i| {
                fpm::library::KeyValueData::from(format!("-{}", i.key), i.value.to_string())
            }),
        );

        UserGroupCompat {
            id: self.id.clone(),
            title: self.title.clone(),
            description: self.description.clone(),
            group_members,
            groups: self.groups.clone(),
        }
    }

    // TODO: Need to handle excluded_identities and excluded_groups
    // Maybe Logic: group.identities + (For all group.groups(g.group - g.excluded_group)).identities
    //              - group.excluded_identities

    pub fn get_identities(&self, config: &fpm::Config) -> fpm::Result<Vec<UserIdentity>> {
        let mut identities = vec![];

        // A group contains child another groups
        for group in self.groups.iter() {
            let user_group = user_group_by_id(config, group.as_str())?.ok_or_else(|| {
                fpm::Error::GroupNotFound {
                    id: group.to_string(),
                    message: "group not found while getting identities".to_string(),
                }
            })?;

            // Recursive call to get child groups identities
            identities.extend(user_group.get_identities(config)?)
        }
        identities.extend(self.identities.clone());

        Ok(identities)
    }

    /// This function returns `true` if any of given
    /// identities is part of group else return's `false`
    pub fn belongs_to(
        &self,
        config: &fpm::Config,
        identities: &[UserIdentity],
    ) -> fpm::Result<bool> {
        for group_identity in self.identities.iter() {
            for identity in identities.iter() {
                if group_identity.eq(identity) {
                    return Ok(true);
                }
            }
        }

        for group_id in self.groups.iter() {
            let group = user_group_by_id(config, group_id.as_str())?.ok_or_else(|| {
                fpm::Error::GroupNotFound {
                    id: group_id.to_string(),
                    message: format!(
                        "group not found while checking belongs_to with group: {}",
                        self.id.as_str()
                    ),
                }
            })?;

            if group.belongs_to(config, identities)? {
                return Ok(true);
            }
        }

        Ok(false)
    }
}

impl UserGroupTemp {
    pub fn are_unique(groups: &[UserGroupTemp]) -> Result<bool, String> {
        // TODO: Tell all the repeated ids at once, this will only tell one at a time
        // TODO: todo this we have to count frequencies and return error if any frequency is
        // greater that one
        let mut set = std::collections::HashSet::new();
        for group in groups {
            if set.contains(&group.id) {
                return Err(format!(
                    "user-group ids are not unique: repeated id: {}",
                    group.id
                ));
            }
            set.insert(&group.id);
        }
        Ok(true)
    }

    pub fn user_groups(
        user_groups: Vec<UserGroupTemp>,
    ) -> fpm::Result<std::collections::BTreeMap<String, UserGroup>> {
        Self::are_unique(&user_groups).map_err(|e| {
            crate::sitemap::ParseError::InvalidUserGroup {
                doc_id: "FPM.ftd".to_string(),
                message: e,
                row_content: "".to_string(),
            }
        })?;
        let mut groups = std::collections::BTreeMap::new();
        for group in user_groups.into_iter() {
            groups.insert(group.id.to_string(), group.to_user_group()?);
        }
        Ok(groups)
    }

    #[allow(clippy::wrong_self_convention)]
    pub fn to_user_group(self) -> fpm::Result<UserGroup> {
        use itertools::Itertools;
        let mut identities = vec![];
        let mut excluded_identities = vec![];

        fn to_user_identity(name: &str, values: Vec<String>) -> Vec<UserIdentity> {
            values
                .into_iter()
                .map(|v| UserIdentity::from(name, v.as_str()))
                .collect_vec()
        }

        identities.extend(to_user_identity("email", self.email));
        excluded_identities.extend(to_user_identity("-email", self.excluded_email));
        identities.extend(to_user_identity("domain", self.domain));
        excluded_identities.extend(to_user_identity("-domain", self.excluded_domain));
        identities.extend(to_user_identity("domain", self.telegram));
        excluded_identities.extend(to_user_identity("-telegram", self.excluded_telegram));
        identities.extend(to_user_identity("github", self.github));
        excluded_identities.extend(to_user_identity("-github", self.excluded_github));
        identities.extend(to_user_identity("github-team", self.github_team));
        excluded_identities.extend(to_user_identity("-github-team", self.excluded_github_team));
        identities.extend(to_user_identity("discord", self.discord));
        excluded_identities.extend(to_user_identity("-discord", self.excluded_discord));

        Ok(UserGroup {
            id: self.id,
            description: self.description,
            identities,
            excluded_identities,
            title: self.title,
            groups: self.groups,
            excluded_groups: self.excluded_group,
        })
    }
}

/// `get_identities` for a `doc_path`
/// This will get the identities from groups defined in sitemap
pub fn get_identities(
    config: &crate::Config,
    doc_path: &str,
    is_read: bool,
) -> fpm::Result<Vec<String>> {
    // TODO: cookies or cli parameter

    let readers_writers = if let Some(sitemap) = &config.package.sitemap {
        if is_read {
            sitemap.readers(doc_path, &config.package.groups)
        } else {
            sitemap.writers(doc_path, &config.package.groups)
        }
    } else {
        vec![]
    };

    let identities: fpm::Result<Vec<Vec<UserIdentity>>> = readers_writers
        .into_iter()
        .map(|g| g.get_identities(config))
        .collect();

    let identities = identities?
        .into_iter()
        .flat_map(|x| x.into_iter())
        .map(|identity| identity.to_string())
        .collect();

    Ok(identities)
}

// TODO Doc: group-id should not contain / in it
pub fn user_groups_by_package(config: &fpm::Config, package: &str) -> fpm::Result<Vec<UserGroup>> {
    // TODO: Need to fix it, It should not read groups from individual FPM.ftd file
    // IT should read groups from package.groups

    // let package = config.find_package_by_name(package).await?;
    // Ok(package.groups.into_values().collect_vec())

    let fpm_document = config.get_fpm_document(package)?;
    fpm_document
        .get::<Vec<UserGroupTemp>>("fpm#user-group")?
        .into_iter()
        .map(|g| g.to_user_group())
        .collect()
}

/// group_id: "<package_name>/<group_id>" or "<group_id>"
pub fn user_group_by_id(config: &fpm::Config, group_id: &str) -> fpm::Result<Option<UserGroup>> {
    // If group `id` does not contain `/` then it is current package group_id
    let (package, group_id) = group_id
        .rsplit_once('/')
        .unwrap_or((&config.package.name, group_id));

    Ok(user_groups_by_package(config, package)?
        .into_iter()
        .find(|g| g.id.as_str() == group_id))
}

/// if any input identity is part of any input group,
/// this function will return `true`, else `false`.
pub fn belongs_to(
    config: &fpm::Config,
    groups: &[&UserGroup],
    identities: &[UserIdentity],
) -> fpm::Result<bool> {
    for group in groups.iter() {
        if group.belongs_to(config, identities)? {
            return Ok(true);
        }
    }
    Ok(false)
}

pub mod processor {
    use itertools::Itertools;

    pub fn user_groups(
        section: &ftd::p1::Section,
        doc: &ftd::p2::TDoc,
        config: &fpm::Config,
    ) -> ftd::p1::Result<ftd::Value> {
        let g = config
            .package
            .groups
            .iter()
            .map(|(_, g)| g.to_group_compat())
            .collect_vec();
        doc.from_json(&g, section)
    }

    pub fn user_group_by_id(
        section: &ftd::p1::Section,
        doc: &ftd::p2::TDoc,
        config: &fpm::Config,
    ) -> ftd::p1::Result<ftd::Value> {
        let id = section.header.str(doc.name, section.line_number, "id")?;
        let g = config
            .package
            .groups
            .get(id)
            .map(|g| g.to_group_compat())
            .ok_or_else(|| ftd::p1::Error::NotFound {
                key: format!("user-group: `{}` not found", id),
                doc_id: doc.name.to_string(),
                line_number: section.line_number,
            })?;
        doc.from_json(&g, section)
    }

    pub fn get_identities<'a>(
        section: &ftd::p1::Section,
        doc: &'a ftd::p2::TDoc<'_>,
        config: &fpm::Config,
    ) -> ftd::p1::Result<ftd::Value> {
        let doc_id = fpm::library::document::document_full_id(config, doc)?;
        let identities = super::get_identities(config, doc_id.as_str(), true).map_err(|e| {
            ftd::p1::Error::ParseError {
                message: e.to_string(),
                doc_id,
                line_number: section.line_number,
            }
        })?;

        Ok(ftd::Value::List {
            data: identities
                .into_iter()
                .map(|i| ftd::PropertyValue::Value {
                    value: ftd::Value::String {
                        text: i,
                        source: ftd::TextSource::Default,
                    },
                })
                .collect_vec(),
            kind: ftd::p2::Kind::List {
                kind: Box::new(ftd::p2::Kind::String {
                    caption: false,
                    body: false,
                    default: None,
                    is_reference: false,
                }),
                default: None,
                is_reference: false,
            },
        })
    }
}

#[cfg(test)]
mod tests {
    // TODO:
    #[test]
    fn get_identities() {}
}
