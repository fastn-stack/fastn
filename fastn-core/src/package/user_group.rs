// identities to group, test also
#[derive(Debug, Clone, serde::Serialize)]
pub struct UserIdentity {
    pub key: String,
    pub value: String,
}

impl PartialEq for UserIdentity {
    fn eq(&self, other: &Self) -> bool {
        self.key
            .to_lowercase()
            .eq(other.key.to_lowercase().as_str())
            && self
                .value
                .to_lowercase()
                .eq(other.value.to_lowercase().as_str())
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
    #[serde(rename = "telegram-admin")]
    pub telegram_admin: Vec<String>,
    #[serde(rename = "-telegram-admin")]
    pub excluded_telegram_admin: Vec<String>,
    #[serde(rename = "telegram-group")]
    pub telegram_group: Vec<String>,
    #[serde(rename = "-telegram-group")]
    pub excluded_telegram_group: Vec<String>,
    #[serde(rename = "telegram-channel")]
    pub telegram_channel: Vec<String>,
    #[serde(rename = "-telegram-channel")]
    pub excluded_telegram_channel: Vec<String>,

    #[serde(rename = "github")]
    pub github: Vec<String>,
    #[serde(rename = "-github")]
    pub excluded_github: Vec<String>,
    #[serde(rename = "github-starred")]
    pub github_like: Vec<String>,
    #[serde(rename = "-github-starred")]
    pub excluded_github_like: Vec<String>,
    #[serde(rename = "github-team")]
    pub github_team: Vec<String>,
    #[serde(rename = "-github-team")]
    pub excluded_github_team: Vec<String>,
    #[serde(rename = "github-contributor")]
    pub github_contributor: Vec<String>,
    #[serde(rename = "-github-contributor")]
    pub excluded_github_contributor: Vec<String>,
    #[serde(rename = "github-collaborator")]
    pub github_collaborator: Vec<String>,
    #[serde(rename = "-github-collaborator")]
    pub excluded_github_collaborator: Vec<String>,
    #[serde(rename = "github-watches")]
    pub github_watches: Vec<String>,
    #[serde(rename = "-github-watches")]
    pub excluded_github_watches: Vec<String>,
    #[serde(rename = "github-follows")]
    pub github_follows: Vec<String>,
    #[serde(rename = "-github-follows")]
    pub excluded_github_follows: Vec<String>,
    #[serde(rename = "github-sponsor")]
    pub github_sponsors: Vec<String>,
    #[serde(rename = "-github-sponsor")]
    pub excluded_github_sponsors: Vec<String>,
    #[serde(rename = "discord-server")]
    pub discord_server: Vec<String>,
    #[serde(rename = "-discord-server")]
    pub excluded_discord_server: Vec<String>,
    #[serde(rename = "discord-channel")]
    pub discord_channel: Vec<String>,
    #[serde(rename = "-discord-channel")]
    pub excluded_discord_channel: Vec<String>,
    #[serde(rename = "discord-role")]
    pub discord_role: Vec<String>,
    #[serde(rename = "-discord-role")]
    pub excluded_discord_role: Vec<String>,
    #[serde(rename = "discord-thread")]
    pub discord_thread: Vec<String>,
    #[serde(rename = "-discord-thread")]
    pub excluded_discord_thread: Vec<String>,
    #[serde(rename = "discord-permission")]
    pub discord_permission: Vec<String>,
    #[serde(rename = "-discord-permission")]
    pub excluded_discord_permission: Vec<String>,
    #[serde(rename = "discord-event")]
    pub discord_event: Vec<String>,
    #[serde(rename = "-discord-event")]
    pub excluded_discord_event: Vec<String>,
    #[serde(rename = "twitter-liking")]
    pub twitter_liking: Vec<String>,
    #[serde(rename = "-twitter-liking")]
    pub excluded_twitter_liking: Vec<String>,
    #[serde(rename = "twitter-followers")]
    pub twitter_followers: Vec<String>,
    #[serde(rename = "-twitter-followers")]
    pub excluded_twitter_followers: Vec<String>,
    #[serde(rename = "twitter-follows")]
    pub twitter_follows: Vec<String>,
    #[serde(rename = "-twitter-follows")]
    pub excluded_twitter_follows: Vec<String>,
    #[serde(rename = "twitter-space")]
    pub twitter_space: Vec<String>,
    #[serde(rename = "-twitter-space")]
    pub excluded_twitter_space: Vec<String>,
    #[serde(rename = "twitter-retweet")]
    pub twitter_retweet: Vec<String>,
    #[serde(rename = "-twitter-retweet")]
    pub excluded_twitter_retweet: Vec<String>,
}

#[derive(Debug, serde::Serialize)]
pub struct UserGroupCompat {
    id: String,
    title: Option<String>,
    description: Option<String>,
    // It will contain all group members, like group, email and -email, etc...
    #[serde(rename = "group-members")]
    group_members: Vec<fastn_core::library2022::KeyValueData>,
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
                .map(|i| fastn_core::library2022::KeyValueData::from(i.key, i.value)),
        );

        group_members.extend(self.excluded_identities.iter().map(|i| {
            fastn_core::library2022::KeyValueData::from(format!("-{}", i.key), i.value.to_string())
        }));

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

    pub fn get_identities(
        &self,
        config: &fastn_core::Config,
    ) -> fastn_core::Result<Vec<UserIdentity>> {
        let mut identities = vec![];

        // A group contains child another groups
        for group in self.groups.iter() {
            let user_group = user_group_by_id(config, group.as_str())?.ok_or_else(|| {
                fastn_core::Error::GroupNotFound {
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
        config: &fastn_core::Config,
        identities: &[&UserIdentity],
    ) -> fastn_core::Result<bool> {
        for group_identity in self.identities.iter() {
            for identity in identities.iter() {
                if group_identity.eq(identity) {
                    return Ok(true);
                }
            }
        }

        for group_id in self.groups.iter() {
            let group = user_group_by_id(config, group_id.as_str())?.ok_or_else(|| {
                fastn_core::Error::GroupNotFound {
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
    ) -> fastn_core::Result<std::collections::BTreeMap<String, UserGroup>> {
        Self::are_unique(&user_groups).map_err(|e| {
            crate::sitemap::ParseError::InvalidUserGroup {
                doc_id: "FASTN.ftd".to_string(),
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
    pub fn to_user_group(self) -> fastn_core::Result<UserGroup> {
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
        identities.extend(to_user_identity("telegram-admin", self.telegram_admin));
        excluded_identities.extend(to_user_identity(
            "-telegram-admin",
            self.excluded_telegram_admin,
        ));
        identities.extend(to_user_identity("telegram-group", self.telegram_group));
        excluded_identities.extend(to_user_identity(
            "-telegram-group",
            self.excluded_telegram_group,
        ));
        identities.extend(to_user_identity("telegram-channel", self.telegram_channel));
        excluded_identities.extend(to_user_identity(
            "-telegram-channel",
            self.excluded_telegram_channel,
        ));
        identities.extend(to_user_identity("github", self.github));
        excluded_identities.extend(to_user_identity("-github", self.excluded_github));
        identities.extend(to_user_identity("github-starred", self.github_like));
        excluded_identities.extend(to_user_identity(
            "-github-starred",
            self.excluded_github_like,
        ));
        identities.extend(to_user_identity("github-team", self.github_team));
        excluded_identities.extend(to_user_identity("-github-team", self.excluded_github_team));
        identities.extend(to_user_identity(
            "github-contributor",
            self.github_contributor,
        ));
        excluded_identities.extend(to_user_identity(
            "-github-contributor",
            self.excluded_github_contributor,
        ));
        identities.extend(to_user_identity(
            "github-collaborator",
            self.github_collaborator,
        ));
        excluded_identities.extend(to_user_identity(
            "-github-collaborator",
            self.excluded_github_collaborator,
        ));
        identities.extend(to_user_identity("github-watches", self.github_watches));
        excluded_identities.extend(to_user_identity(
            "-github-watches",
            self.excluded_github_watches,
        ));
        identities.extend(to_user_identity("github-follows", self.github_follows));
        excluded_identities.extend(to_user_identity(
            "-github-follows",
            self.excluded_github_follows,
        ));
        identities.extend(to_user_identity("github-sponsor", self.github_sponsors));
        excluded_identities.extend(to_user_identity(
            "-github-sponsor",
            self.excluded_github_sponsors,
        ));
        identities.extend(to_user_identity("discord-server", self.discord_server));
        excluded_identities.extend(to_user_identity(
            "-discord-server",
            self.excluded_discord_server,
        ));

        identities.extend(to_user_identity("discord-channel", self.discord_channel));
        excluded_identities.extend(to_user_identity(
            "-discord-channel",
            self.excluded_discord_channel,
        ));
        identities.extend(to_user_identity("discord-role", self.discord_role));
        excluded_identities.extend(to_user_identity(
            "-discord-role",
            self.excluded_discord_role,
        ));
        identities.extend(to_user_identity("discord-thread", self.discord_thread));
        excluded_identities.extend(to_user_identity(
            "-discord-thread",
            self.excluded_discord_thread,
        ));
        identities.extend(to_user_identity(
            "discord-permission",
            self.discord_permission,
        ));
        excluded_identities.extend(to_user_identity(
            "-discord-permission",
            self.excluded_discord_permission,
        ));
        identities.extend(to_user_identity("discord-event", self.discord_event));
        excluded_identities.extend(to_user_identity(
            "-discord-event",
            self.excluded_discord_event,
        ));
        identities.extend(to_user_identity("twitter-liking", self.twitter_liking));
        excluded_identities.extend(to_user_identity(
            "-twitter-liking",
            self.excluded_twitter_liking,
        ));
        identities.extend(to_user_identity(
            "twitter-followers",
            self.twitter_followers,
        ));
        excluded_identities.extend(to_user_identity(
            "-twitter-followers",
            self.excluded_twitter_followers,
        ));
        identities.extend(to_user_identity("twitter-follows", self.twitter_follows));
        excluded_identities.extend(to_user_identity(
            "-twitter-follows",
            self.excluded_twitter_follows,
        ));
        identities.extend(to_user_identity("twitter-space", self.twitter_space));
        excluded_identities.extend(to_user_identity(
            "-twitter-space",
            self.excluded_twitter_space,
        ));
        identities.extend(to_user_identity("twitter-retweet", self.twitter_retweet));
        excluded_identities.extend(to_user_identity(
            "-twitter-retweet",
            self.excluded_twitter_retweet,
        ));

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

/// `get_identities` for a `document_name`
/// This will get the identities from groups defined in sitemap
pub fn get_identities(
    config: &crate::Config,
    document_name: &str,
    is_read: bool,
) -> fastn_core::Result<Vec<UserIdentity>> {
    // TODO: cookies or cli parameter

    let readers_writers = if let Some(sitemap) = &config.package.sitemap {
        if is_read {
            sitemap.readers(document_name, &config.package.groups).0
        } else {
            sitemap.writers(document_name, &config.package.groups)
        }
    } else {
        vec![]
    };

    let identities: fastn_core::Result<Vec<Vec<UserIdentity>>> = readers_writers
        .into_iter()
        .map(|g| g.get_identities(config))
        .collect();

    let identities = identities?
        .into_iter()
        .flat_map(|x| x.into_iter())
        .collect();

    Ok(identities)
}

// TODO Doc: group-id should not contain / in it
pub fn user_groups_by_package(
    config: &fastn_core::Config,
    package: &str,
) -> fastn_core::Result<Vec<UserGroup>> {
    // TODO: Need to fix it, It should not read groups from individual FASTN.ftd file
    // IT should read groups from package.groups

    // let package = config.find_package_by_name(package).await?;
    // Ok(package.groups.into_values().collect_vec())

    let fastn_document = config.get_fastn_document(package)?;
    fastn_document
        .get::<Vec<UserGroupTemp>>("fastn#user-group")?
        .into_iter()
        .map(|g| g.to_user_group())
        .collect()
}

/// group_id: "<package_name>/<group_id>" or "<group_id>"
pub fn user_group_by_id(
    config: &fastn_core::Config,
    group_id: &str,
) -> fastn_core::Result<Option<UserGroup>> {
    // If group `id` does not contain `/` then it is current package group_id
    let (package, group_id) = group_id
        .rsplit_once('/')
        .unwrap_or((&config.package.name, group_id));

    Ok(user_groups_by_package(config, package)?
        .into_iter()
        .find(|g| g.id.as_str() == group_id))
}

/// return true if: any input identity is match with any input group's identity.
pub fn belongs_to(
    config: &fastn_core::Config,
    groups: &[&UserGroup],
    identities: &[&UserIdentity],
) -> fastn_core::Result<bool> {
    for group in groups.iter() {
        if group.belongs_to(config, identities)? {
            return Ok(true);
        }
    }
    Ok(false)
}

/// 'email: abrark.asahi@gmail.com => vec[UId{email: abrark.asahi@gmail.com}]
pub fn parse_identities(identities: &str) -> Vec<UserIdentity> {
    use itertools::Itertools;
    let identities = identities.split(',').collect_vec();
    identities
        .into_iter()
        .flat_map(|id| id.split_once(':'))
        .map(|(k, v)| UserIdentity {
            key: k.trim().to_string(),
            value: v.trim().to_string(),
        })
        .collect_vec()
}

/// Get identities from cli `--identities`
pub fn parse_cli_identities() -> Vec<UserIdentity> {
    let identities = fastn_core::utils::parse_from_cli("--identities");
    parse_identities(&identities.unwrap_or_default())
}

/*
access_identities(req: HttpRequest, document_name: str, rw: bool)
 if feature remote
    / get identities by using document_path from sitemap
    sitemap_identities = get_identities(document_name)
    remote_identities: pass these identities to fetch_from_remote(cookies[sid and identities])
 if feature is not remote
    identities: identities-cookie or identities-cli
 */

pub async fn access_identities(
    config: &fastn_core::Config,
    req: &fastn_core::http::Request,
    document_name: &str,
    is_read: bool,
) -> fastn_core::Result<Vec<UserIdentity>> {
    let sitemap_identities = get_identities(config, document_name, is_read)?;
    //dbg!(&sitemap_identities);
    // github-team: fastn-lang/ftd
    // github-starred: fastn-lang/ftd
    // discord-server: abrark.com
    // github-watches: fastn-lang/ftd
    match fastn_core::auth::get_auth_identities(req.cookies(), sitemap_identities.as_slice()).await
    {
        Ok(ids) => Ok(ids),
        Err(fastn_core::Error::GenericError(_err)) => Ok(vec![]),
        e => e,
    }
}

pub mod processor {
    use itertools::Itertools;

    pub fn user_groups(
        section: &ftd::p1::Section,
        doc: &ftd::p2::TDoc,
        config: &fastn_core::Config,
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
        config: &fastn_core::Config,
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
        config: &fastn_core::Config,
    ) -> ftd::p1::Result<ftd::Value> {
        let doc_id = fastn_core::library::document::document_full_id(config, doc)?;
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
                        text: i.to_string(),
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

    // is user can_read the document or not based on defined readers in sitemap
    pub async fn is_reader<'a>(
        section: &ftd::p1::Section,
        doc: &'a ftd::p2::TDoc<'_>,
        config: &fastn_core::Config,
    ) -> ftd::p1::Result<ftd::Value> {
        let doc_id = fastn_core::library::document::document_full_id(config, doc)?;
        let is_reader = config
            .can_read(config.request.as_ref().unwrap(), &doc_id, false)
            .await
            .map_err(|e| ftd::p1::Error::ParseError {
                message: e.to_string(),
                doc_id,
                line_number: section.line_number,
            })?;

        Ok(ftd::Value::Boolean { value: is_reader })
    }
}

#[cfg(test)]
mod tests {
    // TODO:
    #[test]
    fn get_identities() {}
}
