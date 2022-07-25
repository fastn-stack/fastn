use itertools::Itertools;

// identities to group, test also
#[derive(Debug, Clone)]
pub struct UserGroup {
    pub title: Option<String>,
    pub id: String,
    pub identities: Vec<(String, String)>,
    pub excluded_identities: Vec<(String, String)>,

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

/*
.ftd code
-- fpm.user-group list email-groups:
$processor$: user-groups

-- fpm.user-group list email-groups:
$processor$: user-groups
 */

#[derive(Debug, serde::Serialize)]
pub struct UserGroupCompat {
    id: String,
    title: Option<String>,
    description: Option<String>,
    // It will contain all group members, like group, email and -email, etc...
    #[serde(rename = "group-members")]
    group_members: Vec<fpm::library::full_sitemap::KeyValueData>,
}

impl UserGroup {
    pub fn to_group_compat(&self) -> UserGroupCompat {
        let mut group_members = vec![];

        group_members.extend(self.identities.clone());
        for (k, v) in self.excluded_identities.iter() {
            group_members.push((format!("-{}", k), v.to_string()));
        }

        for import in self.groups.iter() {
            group_members.push(("group".to_string(), import.to_string()));
        }

        UserGroupCompat {
            id: self.id.clone(),
            title: self.title.clone(),
            description: self.description.clone(),
            group_members: group_members
                .into_iter()
                .map(|(key, value)| fpm::library::full_sitemap::KeyValueData { key, value })
                .collect_vec(),
        }
    }
}

impl UserGroupTemp {
    pub fn to_user_group(self) -> fpm::Result<UserGroup> {
        let mut identities = vec![];
        let mut excluded_identities = vec![];

        fn to_key_value(name: &str, values: Vec<String>) -> Vec<(String, String)> {
            values
                .into_iter()
                .map(|v| (name.to_string(), v))
                .collect_vec()
        }

        identities.extend(to_key_value("email", self.email));
        excluded_identities.extend(to_key_value("-email", self.excluded_email));
        identities.extend(to_key_value("domain", self.domain));
        excluded_identities.extend(to_key_value("-domain", self.excluded_domain));
        identities.extend(to_key_value("domain", self.telegram));
        excluded_identities.extend(to_key_value("-telegram", self.excluded_telegram));
        identities.extend(to_key_value("github", self.github));
        excluded_identities.extend(to_key_value("-github", self.excluded_github));
        identities.extend(to_key_value("github-team", self.github_team));
        excluded_identities.extend(to_key_value("-github-team", self.excluded_github_team));
        identities.extend(to_key_value("discord", self.discord));
        excluded_identities.extend(to_key_value("-discord", self.excluded_discord));

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

pub mod processor {
    use itertools::Itertools;

    pub fn user_groups(
        section: &ftd::p1::Section,
        doc: &ftd::p2::TDoc,
        config: &fpm::Config,
    ) -> ftd::p1::Result<ftd::Value> {
        let g = config
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
}

// fn a_minus_b_ref<'a>(
//     a: &'a [(&'a str, &'a str)],
//     b: &'a [(&'a str, &'a str)],
// ) -> Vec<&'a (&'a str, &'a str)> {
//     let mut excluded: HashMap<_, HashSet<_>> = HashMap::new();
//     for (k, v) in b {
//         if excluded.contains_key(k) {
//             let t = excluded.get_mut(k).unwrap();
//             t.insert(v);
//         } else {
//             let mut t = HashSet::new();
//             t.insert(v);
//             excluded.insert(k, t);
//         }
//     }
//
//     let is_in_b = |k: &str, v: &str| {
//         if let Some(set) = excluded.get(&k) {
//             return set.contains(&v);
//         }
//         false
//     };
//     a.into_iter().filter(|(k, v)| !is_in_b(k, v)).collect_vec()
// }
//
// fn a_minus_b<'a>(
//     a: &'a Vec<(String, String)>,
//     b: &'a Vec<(String, String)>,
// ) -> Vec<(String, String)> {
//     let mut excluded: HashMap<_, HashSet<_>> = HashMap::new();
//     for (k, v) in b {
//         if excluded.contains_key(k) {
//             let t = excluded.get_mut(k).unwrap();
//             t.insert(v);
//         } else {
//             let mut t = HashSet::new();
//             t.insert(v);
//             excluded.insert(k, t);
//         }
//     }
//     let is_in_b = |k: &String, v: &String| {
//         if let Some(set) = excluded.get(&k) {
//             return set.contains(v);
//         }
//         false
//     };
//     //TODO: Remove .map(|(k, v)| (k.to_string(), v.to_string()))
//     a.into_iter()
//         .filter(|(k, v)| !is_in_b(k, v))
//         .map(|(k, v)| (k.to_string(), v.to_string()))
//         .collect_vec()
// }

// pub fn get_group_members(&self, config: &fpm::Config) -> Vec<(String, String)> {
//     if self.import.len() == 0 {
//         return UserGroup::a_minus_b(&self.identities, &self.excluded_identities);
//     }
//
//     let mut group_identities = vec![];
//     for group in self.import.iter() {
//         let (package, group_id) = group
//             .rsplit_once('/')
//             .ok_or_else(|| ftd::p1::Error::ParseError {
//                 message: format!("import_identity: {}, does not contain `/`", group),
//                 doc_id: "FPM.ftd".to_string(),
//                 line_number: 0,
//             })
//             .unwrap();
//
//         // TODO: Remove unwrap
//         let fpm_document = config.get_fpm_document(package).unwrap();
//         let user_groups: Vec<UserGroupTemp> = fpm_document.get("fpm#user-group").unwrap();
//         let user_group = user_groups.into_iter().find(|g| g.id.eq(group_id)).unwrap();
//         let user_group = user_group.to_user_group();
//         let user_group_members = user_group.get_group_members(config);
//         group_identities.extend(user_group_members);
//     }
//
//     //TODO: Remove Clone
//     group_identities.extend(self.identities.clone());
//     return UserGroup::a_minus_b(&group_identities, &self.excluded_identities);
// }
