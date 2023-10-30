pub async fn pwd() -> fastn_core::Result<fastn_core::http::Response> {
    if !is_tutor() {
        return Ok(fastn_core::not_found!("this only works in tutor mode"));
    }

    fastn_core::http::api_ok(std::env::current_dir()?.to_string_lossy())
}

pub async fn shutdown() -> fastn_core::Result<fastn_core::http::Response> {
    if !is_tutor() {
        return Ok(fastn_core::not_found!("this only works in tutor mode"));
    }

    println!("/-/shutdown/ called, shutting down");
    std::process::exit(0);
}

pub async fn process(
    value: ftd::ast::VariableValue,
    kind: ftd::interpreter::Kind,
    doc: &ftd::interpreter::TDoc<'_>,
) -> ftd::interpreter::Result<ftd::interpreter::Value> {
    if !fastn_core::tutor::is_tutor() {
        return Err(ftd::interpreter::Error::OtherError(
            "tutor process only works in tutor mode".to_string(),
        ));
    }

    let state =
        match tokio::fs::read(dirs::home_dir().unwrap().join(".fastn").join("tutor.json")).await {
            Ok(v) => serde_json::from_slice(&v)?,
            Err(e) => match e.kind() {
                std::io::ErrorKind::NotFound => TutorStateFS::default(),
                _ => return Err(e.into()),
            },
        }
        .to_state(std::env::current_dir()?)?;

    doc.from_json(&state, &kind, &value)
}

#[derive(Debug, Default, serde::Deserialize)]
struct TutorStateFS {
    done: Vec<String>,
    current: String,
}

#[derive(Debug, serde::Serialize, PartialEq)]
struct TutorState {
    workshops: Vec<Workshop>,
}

impl TutorStateFS {
    fn to_state<T: AsRef<std::path::Path>>(
        self: TutorStateFS,
        path: T,
    ) -> ftd::interpreter::Result<TutorState> {
        use itertools::Itertools;

        let mut workshops = vec![];
        static RE: once_cell::sync::Lazy<regex::Regex> =
            once_cell::sync::Lazy::new(|| regex::Regex::new(r"^[a-zA-Z]-[a-zA-Z]+.*$").unwrap());

        for entry in std::fs::read_dir(path)?.sorted_by(sort_path) {
            let entry = entry?;
            let path = entry.path();

            if !path.is_dir() {
                continue;
            }
            if !RE.is_match(&path.file_name().unwrap().to_string_lossy()) {
                continue;
            }

            workshops.push(Workshop::load(&path, &self)?);
        }

        Ok(TutorState { workshops })
    }
}

fn sort_path(
    a: &std::io::Result<std::fs::DirEntry>,
    b: &std::io::Result<std::fs::DirEntry>,
) -> std::cmp::Ordering {
    a.as_ref().unwrap().path().cmp(&b.as_ref().unwrap().path())
}

#[derive(Debug, serde::Serialize, PartialEq)]
struct Workshop {
    title: String,
    url: String,
    done: bool,
    current: bool,
    tutorials: Vec<Tutorial>,
}

impl Workshop {
    fn load(path: &std::path::Path, state: &TutorStateFS) -> ftd::interpreter::Result<Self> {
        use itertools::Itertools;

        let mut tutorials = vec![];
        let id = path.file_name().unwrap().to_string_lossy();

        static RE: once_cell::sync::Lazy<regex::Regex> =
            once_cell::sync::Lazy::new(|| regex::Regex::new(r"^[0-9][0-9]-[a-zA-Z]+.*$").unwrap());

        for entry in std::fs::read_dir(path)?.sorted_by(sort_path) {
            let entry = entry?;
            let path = entry.path();

            if !path.is_dir() {
                continue;
            }
            if !RE.is_match(&path.file_name().unwrap().to_string_lossy()) {
                continue;
            }

            tutorials.push(Tutorial::load(&id, &path, state)?);
        }

        Ok(Workshop {
            title: title_from_readme(path)?,
            url: format!("/{id}/"),
            done: !tutorials.iter().any(|t| !t.done),
            current: tutorials.iter().any(|t| t.current),
            tutorials,
        })
    }
}

fn title_from_readme(folder: &std::path::Path) -> ftd::interpreter::Result<String> {
    let content = std::fs::read_to_string(folder.join("README.md"))?;
    let (title, _about) = match content.split_once("\n\n") {
        Some(v) => v,
        None => {
            return Err(ftd::interpreter::Error::OtherError(
                "invalid README.md".into(),
            ))
        }
    };
    Ok(title.replacen("# ", "", 1))
}

#[derive(Debug, serde::Serialize, PartialEq)]
struct Tutorial {
    id: String,
    url: String,
    title: String,
    done: bool,
    current: bool,
}

impl Tutorial {
    fn load(
        parent: &str,
        path: &std::path::Path,
        state: &TutorStateFS,
    ) -> ftd::interpreter::Result<Self> {
        let id = format!("{parent}/{}", path.file_name().unwrap().to_string_lossy());

        Ok(Tutorial {
            title: title_from_readme(path)?,
            done: state.done.contains(&id),
            current: state.current == id,
            url: format!("/{id}/"),
            id,
        })
    }
}

pub fn is_tutor() -> bool {
    // https://github.com/orgs/fastn-stack/discussions/1414
    // with either of these are passed we allow APIs like /-/shutdown/, `/-/start/` etc
    std::env::args().any(|e| e == "tutor" || e == "--tutor")
}

#[cfg(test)]
mod test {
    use pretty_assertions::assert_eq;

    #[test]
    fn test() {
        let mut ts = super::TutorState {
            workshops: vec![
                super::Workshop {
                    title: "Build Websites Using `fastn`".to_string(),
                    url: "/a-website/".to_string(),
                    done: false,
                    current: false,
                    tutorials: vec![super::Tutorial {
                        id: "a-website/01-hello-world".to_string(),
                        url: "/a-website/01-hello-world/".to_string(),
                        title: "Install and start using `fastn`".to_string(),
                        done: false,
                        current: false,
                    }],
                },
                super::Workshop {
                    title: "Build User Interfaces Using `fastn`".to_string(),
                    url: "/b-ui/".to_string(),
                    done: false,
                    current: false,
                    tutorials: vec![super::Tutorial {
                        id: "b-ui/01-hello-world".to_string(),
                        url: "/b-ui/01-hello-world/".to_string(),
                        title: "Install and start using `fastn`".to_string(),
                        done: false,
                        current: false,
                    }],
                },
            ],
        };

        assert_eq!(
            super::TutorStateFS::default()
                .to_state("tutor-tests/one")
                .unwrap(),
            ts,
        );

        ts.workshops[0].tutorials[0].done = true;
        ts.workshops[0].done = true;
        ts.workshops[1].current = true;
        ts.workshops[1].tutorials[0].current = true;

        assert_eq!(
            super::TutorStateFS {
                done: vec!["a-website/01-hello-world".to_string()],
                current: "b-ui/01-hello-world".to_string(),
            }
            .to_state("tutor-tests/one")
            .unwrap(),
            ts,
        );
    }
}
