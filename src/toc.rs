#[derive(PartialEq, Debug, Default, Serialize, Clone)]
pub struct TocItem {
    pub id: String,
    pub url: String,
    pub title: crate::Rendered,
    pub children: Vec<TocItem>,
}

impl TocItem {
    fn fix_id(&mut self, collection: &str) {
        let without_cr = match id_without_cr(collection) {
            Some(id) => id,
            None => return,
        };
        if self
            .id
            .starts_with(format!("{}/", without_cr.as_str()).as_str())
        {
            self.id = self.id.replacen(without_cr.as_str(), collection, 1);
            self.url = format!("/{}/", self.id.as_str());
        }
    }

    pub fn with_collection(mut self, collection: &str) -> Self {
        self.fix_id(collection);
        if !self.id.starts_with(format!("{}/", collection).as_str()) {
            self.url = format!(
                "/{}/?collection={}",
                self.id.as_str(),
                percent_encoding::utf8_percent_encode(
                    collection,
                    percent_encoding::NON_ALPHANUMERIC
                )
                .to_string()
            );
        }
        self.children = self
            .children
            .into_iter()
            .map(|v| v.with_collection(collection))
            .collect();
        self
    }

    fn h_tags_range(level: i32) -> i32 {
        if level > 6 {
            return 6;
        }
        level
    }

    pub fn get_module_title(&self, id: &str) -> Option<String> {
        if id == self.id {
            return Some(self.title.original.clone());
        }

        for toc in self.children.iter() {
            if let Some(title) = toc.get_module_title(id) {
                return Some(title);
            }
        }

        None
    }

    pub fn with_title_and_id(title: &str, id: &str) -> Self {
        TocItem {
            url: format!("/{}/", id),
            id: id.to_string(),
            title: crate::Rendered::line(title),
            children: vec![],
        }
    }

    #[cfg(test)]
    pub fn from(s: &str) -> (Self, usize) {
        /*
        possible values of s:
            "- foo\n  Foo is awesome\n\n"          -- length = 0
            "- bar\n    bar is super awesome\n\n"  -- length = 1
            "- baz\n  baz ka baaza\n"              -- length = 0
        */
        let s = s.trim();
        let (id, title) = {
            let parts: Vec<&str> = s.splitn(2, '\n').collect();
            (parts[0], parts[1])
        };
        let id = id.replacen("- ", "", 1);
        let title = title.replace("\r", "");
        let title = title.replace("\n", "");
        let mut title = title.chars();
        let mut c;
        let mut i = 0;
        loop {
            c = title.next().unwrap();
            if c != ' ' {
                break;
            }
            i += 1;
        }
        let title = c.to_string() + title.as_str();
        let id = id.trim().to_string();
        (
            TocItem {
                url: format!("/{}/", id.as_str()),
                id,
                title: crate::Rendered::line(title.as_str()),
                children: vec![],
            },
            i / 2 - 1,
        )
    }

    pub fn to_html_string(&self, id: &str, level: i32) -> String {
        // <h3>
        //      <a href="/{{ toc.id }}/?collection={{ id|urlescape }}">
        //          {{ toc.title.rendered }}
        //      </a>
        // </h3>
        let h = Self::h_tags_range(level);
        format!(
            r#"<h{}><a href="/{}/{}">{}</a></h{}>"#,
            h,
            self.id,
            if self.id.starts_with(id) {
                "".to_string()
            } else {
                format!(
                    "?collection={}",
                    percent_encoding::utf8_percent_encode(id, percent_encoding::NON_ALPHANUMERIC)
                        .to_string()
                )
            },
            self.title.rendered,
            h
        )
    }

    pub fn to_str(&self, space: i32) -> String {
        let space = " ".repeat(space as usize * 2);
        format!(
            "{space}- {id}\n{space}  {title}\n",
            id = self.id,
            title = self.title.original,
            space = space,
        )
    }
}

#[derive(PartialEq, Debug, Default, Clone, Serialize)]
pub struct ToC {
    pub items: Vec<TocItem>,
}

#[derive(Debug, Clone, PartialEq)]
enum ParsingState {
    WaitingForID,
    WaitingForTitle((String, usize)),
}

#[derive(Debug)]
pub struct Parser {
    state: ParsingState,
    sections: Vec<(TocItem, usize)>,
}

impl Parser {
    fn read_id(&mut self, line: &str) -> Result<(), ParseError> {
        if line.trim().is_empty() {
            return Ok(());
        }

        let mut iter = line.chars();
        let mut x = 0;
        loop {
            match iter.next() {
                Some(' ') => {
                    iter.next();
                    x += 1;
                }
                Some('-') => {
                    break;
                }
                Some(c) => {
                    return Err(ParseError::InputError(format!(
                        "expecting \"-\", found: {}",
                        c
                    )));
                }
                None => {
                    return Err(ParseError::InputError(format!(
                        "line ended too soon: {}",
                        line
                    )));
                }
            }
        }
        let rest: String = iter.collect();
        if rest.trim().is_empty() {
            return Err(ParseError::InputError(format!(
                "line ended too soon: {}",
                line
            )));
        }
        self.state = ParsingState::WaitingForTitle((rest, x));

        Ok(())
    }

    fn read_title(&mut self, line: &str, id: String, size: usize) -> Result<(), ParseError> {
        if line.trim().is_empty() {
            return Err(ParseError::InputError("found empty line".to_string()));
        }

        let mut iter = line.chars();
        let mut x = 0;
        let c;
        loop {
            match iter.next() {
                Some(' ') => {
                    iter.next();
                    x += 1;
                }
                Some('-') => {
                    return Err(ParseError::InputError(
                        "line starts with -, not allowed".to_string(),
                    ));
                }
                Some(c_) => {
                    c = c_;
                    break;
                }
                None => {
                    return Err(ParseError::InputError(
                        "improperly indented line found".to_string(),
                    ))
                }
            }
        }
        if x != size + 1 {
            return Err(ParseError::InputError(format!(
                "improperly indented, expected {} spaces, found: {}",
                (size + 1) * 2,
                x * 2
            )));
        }
        let rest: String = iter.collect();
        let rest = c.to_string() + rest.as_str();

        let id = id.trim().to_string();
        self.sections.push((
            TocItem {
                url: if id.starts_with("https://")
                    || id.starts_with("http://")
                    || id.starts_with('/')
                {
                    id.to_string()
                } else {
                    format!("/{}/", id.as_str())
                },
                id,
                title: crate::Rendered::line(rest.trim()),
                children: vec![],
            },
            size,
        ));
        self.state = ParsingState::WaitingForID;
        Ok(())
    }

    fn finalize(self) -> Result<Vec<(TocItem, usize)>, ParseError> {
        if self.state != ParsingState::WaitingForID {
            return Err(ParseError::InputError("title not found".to_string()));
        };

        Ok(self.sections)
    }
}

impl ToC {
    pub fn to_string(&self, name: &str) -> String {
        format!("-- {}:\n\n{}", name, toc_str(&self.items))
    }

    pub fn to_p1(&self, name: &str) -> crate::p1::Section {
        crate::p1::Section::with_name(name).and_body(toc_str(&self.items).as_str())
    }

    pub fn parse(s: &str) -> Result<Self, ParseError> {
        let mut parser = Parser {
            state: ParsingState::WaitingForID,
            sections: vec![],
        };

        for line in s.split('\n') {
            let state = parser.state.clone();
            match state {
                ParsingState::WaitingForID => parser.read_id(line)?,
                ParsingState::WaitingForTitle((id, size)) => parser.read_title(line, id, size)?,
            }
        }

        // let record: Vec<(TocItem, usize)> = r
        //     .into_inner()
        //     .map(|item| {
        //         let (item, level): (TocItem, usize) = TocItem::from(item.as_str());
        //         (item, level)
        //     })
        //     .collect();

        Ok(ToC {
            items: construct_tree_util(parser.finalize()?),
        })
    }

    pub fn from_p1(p1: &crate::p1::Section) -> Result<Self, ParseError> {
        match p1.body {
            Some(ref b) => Self::parse(b.as_str()),
            None => Err(ParseError::InputError(
                "caption must be present for heading".to_string(),
            )),
        }
    }

    pub fn contains_path(&self, path: &str) -> bool {
        fn contains_path_util(path: &str, items: &[TocItem]) -> bool {
            if items.is_empty() {
                return false;
            }

            for item in items {
                if item.url.contains(path) {
                    return true;
                }
                if contains_path_util(path, &item.children) {
                    return true;
                };
            }
            false
        }

        contains_path_util(path, &self.items)
    }
}

#[cfg(test)]
fn toc_html_util(toc_item: &TocItem, id: &str, depth: i32, container: &mut Vec<String>) {
    let t = toc_item.to_html_string(id, depth);
    container.push(t);
    for item in toc_item.children.iter() {
        toc_html_util(item, id, depth + 1, container);
    }
}

#[cfg(test)]
pub fn toc_html(toc: &[TocItem], id: &str) -> String {
    // recursively traverse through self.toc, and wrap each item in hN, and
    // a link. N of hN depends on depth, starting depth is 3
    // [TocItem{id=foo, title=bar, children=[]}]
    // <h3>
    //      <a href="/{{ toc.id }}/?collection={{ id|urlescape }}&title={{ toc.title.original|urlescape }}>
    //          {{ toc.title.rendered }}
    //      </a>
    // </h3>
    // <h4>
    //      <a href="/{{ toc.id }}/?collection={{ id|urlescape }}&title={{ toc.title.original|urlescape }}>
    //          {{ toc.title.rendered }}
    //      </a>
    // </h4>

    let mut toc_html_container = vec![];
    for x in toc.iter() {
        toc_html_util(x, id, 3, &mut toc_html_container);
    }
    toc_html_container.join("")
}

fn toc_str_util(toc_item: &TocItem, depth: i32, container: &mut Vec<String>) {
    let t = toc_item.to_str(depth);
    container.push(t);
    for item in toc_item.children.iter() {
        toc_str_util(item, depth + 1, container);
    }
}

pub fn toc_str(toc: &[TocItem]) -> String {
    let mut toc_html_container = vec![];
    for x in toc.iter() {
        toc_str_util(x, 0, &mut toc_html_container);
    }
    toc_html_container.join("")
}

use thiserror::Error as Error_;

#[derive(Error_, Debug)]
pub enum ParseError {
    #[error("P1Error: {0}")]
    P1Error(crate::p1::Error),
    #[error("InputError: {0}")]
    InputError(String),
}

#[derive(Debug)]
struct LevelTree {
    level: usize,
    item: TocItem,
}

impl LevelTree {
    fn new(level: usize, item: TocItem) -> Self {
        Self { level, item }
    }
}

fn get_top_level(stack: &[LevelTree]) -> usize {
    stack.last().map(|x| x.level).unwrap()
}

fn construct_tree_util(mut elements: Vec<(TocItem, usize)>) -> Vec<TocItem> {
    if elements.is_empty() {
        return vec![];
    }
    let smallest_level = elements.get(0).unwrap().1;
    elements.push((TocItem::default(), smallest_level));
    // println!("Elements: {:#?}", elements);
    let mut tree = construct_tree(elements, smallest_level);
    let _garbage = tree.pop();
    tree.into_iter().map(|x| x.item).collect()
}

fn construct_tree(elements: Vec<(TocItem, usize)>, smallest_level: usize) -> Vec<LevelTree> {
    let mut stack_tree = vec![];
    for (toc_item, level) in elements.into_iter() {
        if level < smallest_level {
            panic!("Level should not be lesser than smallest level");
        }
        let node = LevelTree::new(level, toc_item);
        if stack_tree.is_empty() || get_top_level(&stack_tree) <= level {
            stack_tree.push(node);
        } else {
            let top = stack_tree.pop().unwrap();
            let mut top_level = top.level;
            let mut children = vec![top];
            while level < top_level {
                loop {
                    if stack_tree.is_empty() {
                        panic!("Tree should not be empty here")
                    }
                    let mut cur_element = stack_tree.pop().unwrap();
                    if stack_tree.is_empty() || cur_element.level < top_level {
                        // Means found children's parent, needs to append children to its parents
                        // and update top level accordingly
                        // parent level should equal to top_level - 1
                        //println!(
                        //    "TopLevel: {}, CurrentLevel: {}",
                        //    top_level, cur_element.level
                        //);

                        // println!("Children: {:?}", children);
                        assert_eq!(cur_element.level as i32, (top_level as i32) - 1);
                        cur_element
                            .item
                            .children
                            .append(&mut children.into_iter().rev().map(|x| x.item).collect());
                        top_level = cur_element.level;
                        children = vec![];
                        stack_tree.push(cur_element);
                        break;
                    } else if cur_element.level == top_level {
                        // if popped element is same as already popped element it is adjacent
                        // element, needs to push into children and find parent in stack
                        children.push(cur_element);
                    } else {
                        panic!(
                            "Stacked elements level should never be greater than top element level"
                        );
                    }
                }
            }
            assert!(level >= top_level);
            stack_tree.push(node);
        }
    }
    stack_tree
}

pub fn id_without_cr(id: &str) -> Option<String> {
    if !id.contains('~') {
        return None;
    }

    let mut parts = id.splitn(3, '/');
    let namespace = parts.next().unwrap().to_string();
    let mut next = parts.next().unwrap().split('~');
    let collection = next.next().unwrap().to_string();

    Some(match parts.next() {
        Some(d) => format!("{}/{}/{}", namespace, collection, d),
        None => format!("{}/{}", namespace, collection),
    })
}

#[cfg(test)]
mod test {
    use super::TocItem;
    use indoc::indoc;
    use pretty_assertions::assert_eq;

    #[test]
    fn with_collection() {
        let t = super::TocItem {
            id: "amitu/hello/first".to_string(),
            url: "/amitu/hello/first/".to_string(),
            title: crate::Rendered::line("Hello"),
            children: vec![super::TocItem {
                id: "amitu/hello/world".to_string(),
                url: "/amitu/hello/world/".to_string(),
                title: crate::Rendered::line("World"),
                children: vec![],
            }],
        };

        assert_eq!(
            t.clone().with_collection("foo/bar"),
            super::TocItem {
                id: "amitu/hello/first".to_string(),
                url: "/amitu/hello/first/?collection=foo%2Fbar".to_string(),
                title: crate::Rendered::line("Hello"),
                children: vec![super::TocItem {
                    id: "amitu/hello/world".to_string(),
                    url: "/amitu/hello/world/?collection=foo%2Fbar".to_string(),
                    title: crate::Rendered::line("World"),
                    children: vec![],
                }],
            }
        );

        assert_eq!(
            t.clone().with_collection("amitu/hello~1"),
            super::TocItem {
                id: "amitu/hello~1/first".to_string(),
                url: "/amitu/hello~1/first/".to_string(),
                title: crate::Rendered::line("Hello"),
                children: vec![super::TocItem {
                    id: "amitu/hello~1/world".to_string(),
                    url: "/amitu/hello~1/world/".to_string(),
                    title: crate::Rendered::line("World"),
                    children: vec![],
                }],
            }
        );

        assert_eq!(
            super::TocItem {
                id: "amitu/hello/first".to_string(),
                url: "/amitu/hello/first/".to_string(),
                title: crate::Rendered::line("Hello"),
                children: vec![super::TocItem {
                    id: "amitu/yo/world".to_string(),
                    url: "/amitu/yo/world/".to_string(),
                    title: crate::Rendered::line("World"),
                    children: vec![],
                }],
            }
            .with_collection("amitu/hello~1"),
            super::TocItem {
                id: "amitu/hello~1/first".to_string(),
                url: "/amitu/hello~1/first/".to_string(),
                title: crate::Rendered::line("Hello"),
                children: vec![super::TocItem {
                    id: "amitu/yo/world".to_string(),
                    url: "/amitu/yo/world/?collection=amitu%2Fhello%7E1".to_string(),
                    title: crate::Rendered::line("World"),
                    children: vec![],
                }],
            }
        );
    }

    #[test]
    fn id_without_cr() {
        assert_eq!(super::id_without_cr("foo/bar"), None);
        assert_eq!(super::id_without_cr("foo/bar/baz"), None);
        assert_eq!(
            super::id_without_cr("foo/bar~12"),
            Some("foo/bar".to_string())
        );
        assert_eq!(
            super::id_without_cr("foo/bar~10/baz"),
            Some("foo/bar/baz".to_string())
        );
    }

    macro_rules! p {
        ($s:expr, $t: expr,) => {
            p!($s, $t)
        };
        ($s:expr, $t: expr) => {
            assert_eq!(
                super::ToC::parse($s).unwrap_or_else(|e| panic!("{}", e)),
                $t
            )
        };
    }

    #[test]
    fn parse() {
        p!(
            &indoc!(
                "
                - amitu/realm/intro
                  What is Realm?
                  - amitu/realm/routing
                    Routing is Hard
                  - amitu/realm/backend-routing
                    What does Realm do?
                  - amitu/realm/type-safety
                    Backend Data And Type Safety
                - amitu/realm/tutorial
                  Tutorial
                - amitu/realm/how-tos
                  How To Guides
                  - amitu/realm/how-to/api
                    How to Write An API?
                  - amitu/realm/how-to/form
                    How To Validate Forms?
                  - amitu/realm/how-to/ports
                    How to write custom ports?
                  - amitu/realm/how-to/file-upload
                    How to handle file uploads?
                  - amitu/realm/how-to/animation
                    How to do animations?
                "
            ),
            super::ToC {
                items: vec![
                    TocItem {
                        id: "amitu/realm/intro".to_string(),
                        url: "/amitu/realm/intro/".to_string(),
                        title: crate::Rendered::line("What is Realm?"),
                        children: vec![
                            TocItem {
                                id: "amitu/realm/routing".to_string(),
                                url: "/amitu/realm/routing/".to_string(),
                                title: crate::Rendered::line("Routing is Hard"),
                                children: vec![],
                            },
                            TocItem {
                                id: "amitu/realm/backend-routing".to_string(),
                                url: "/amitu/realm/backend-routing/".to_string(),
                                title: crate::Rendered::line("What does Realm do?"),
                                children: vec![],
                            },
                            TocItem {
                                id: "amitu/realm/type-safety".to_string(),
                                url: "/amitu/realm/type-safety/".to_string(),
                                title: crate::Rendered::line("Backend Data And Type Safety"),
                                children: vec![],
                            },
                        ],
                    },
                    TocItem {
                        id: "amitu/realm/tutorial".to_string(),
                        url: "/amitu/realm/tutorial/".to_string(),
                        title: crate::Rendered::line("Tutorial"),
                        children: vec![],
                    },
                    TocItem {
                        id: "amitu/realm/how-tos".to_string(),
                        url: "/amitu/realm/how-tos/".to_string(),
                        title: crate::Rendered::line("How To Guides"),
                        children: vec![
                            TocItem {
                                id: "amitu/realm/how-to/api".to_string(),
                                url: "/amitu/realm/how-to/api/".to_string(),
                                title: crate::Rendered::line("How to Write An API?"),
                                children: vec![],
                            },
                            TocItem {
                                id: "amitu/realm/how-to/form".to_string(),
                                url: "/amitu/realm/how-to/form/".to_string(),
                                title: crate::Rendered::line("How To Validate Forms?"),
                                children: vec![],
                            },
                            TocItem {
                                id: "amitu/realm/how-to/ports".to_string(),
                                url: "/amitu/realm/how-to/ports/".to_string(),
                                title: crate::Rendered::line("How to write custom ports?"),
                                children: vec![],
                            },
                            TocItem {
                                id: "amitu/realm/how-to/file-upload".to_string(),
                                url: "/amitu/realm/how-to/file-upload/".to_string(),
                                title: crate::Rendered::line("How to handle file uploads?"),
                                children: vec![],
                            },
                            TocItem {
                                id: "amitu/realm/how-to/animation".to_string(),
                                url: "/amitu/realm/how-to/animation/".to_string(),
                                title: crate::Rendered::line("How to do animations?"),
                                children: vec![],
                            },
                        ],
                    },
                ]
            }
        );
    }

    #[test]
    fn to_string() {
        assert_eq!(
            super::ToC {
                items: vec![
                    super::TocItem {
                        id: "hello".to_string(),
                        url: "/hello/".to_string(),
                        title: crate::Rendered::line("Hello"),
                        children: vec![]
                    },
                    super::TocItem {
                        id: "world".to_string(),
                        url: "/world/".to_string(),
                        title: crate::Rendered::line("World"),
                        children: vec![]
                    }
                ]
            }
            .to_string("toc"),
            indoc!(
                "
               -- toc:

               - hello
                 Hello
               - world
                 World
               "
            )
        );

        assert_eq!(
            super::ToC {
                items: vec![super::TocItem {
                    id: "hello".to_string(),
                    url: "/hello/".to_string(),
                    title: crate::Rendered::line("Hello"),
                    children: vec![super::TocItem {
                        id: "world".to_string(),
                        url: "/world/".to_string(),
                        title: crate::Rendered::line("World"),
                        children: vec![]
                    }]
                },]
            }
            .to_string("toc"),
            indoc!(
                "
               -- toc:

               - hello
                 Hello
                 - world
                   World
               "
            )
        );

        assert_eq!(
            super::ToC {
                items: vec![
                    super::TocItem {
                        id: "hello".to_string(),
                        url: "/hello/".to_string(),
                        title: crate::Rendered::line("Hello"),
                        children: vec![super::TocItem {
                            id: "world".to_string(),
                            url: "/world/".to_string(),
                            title: crate::Rendered::line("World"),
                            children: vec![]
                        }]
                    },
                    super::TocItem {
                        id: "world".to_string(),
                        url: "/world/".to_string(),
                        title: crate::Rendered::line("World"),
                        children: vec![]
                    }
                ]
            }
            .to_string("toc"),
            indoc!(
                "
               -- toc:

               - hello
                 Hello
                 - world
                   World
               - world
                 World
               "
            )
        );
    }
}
