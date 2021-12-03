pub fn processor(section: &ftd::p1::Section, doc: &ftd::p2::TDoc) -> ftd::p1::Result<ftd::Value> {
    let toc_items = ToC::parse(section.body(section.line_number, doc.name)?.as_str())
        .map_err(|e| ftd::p1::Error::ParseError {
            message: format!("Cannot parse body: {:?}", e),
            doc_id: doc.name.to_string(),
            line_number: section.line_number,
        })?
        .items
        .iter()
        .map(|item| item.to_toc_item_compat())
        .collect::<Vec<TocItemCompat>>();

    doc.from_json(&toc_items, section)
}

#[derive(serde::Deserialize)]
pub struct TocItemInDoc {
    pub id: String,
    pub title: String,
    pub children: Vec<TocItemInDoc>,
}

impl From<TocItemInDoc> for TocItem {
    fn from(s: TocItemInDoc) -> TocItem {
        TocItem {
            url: format!("/{}/", s.id.as_str()),
            number: vec![],
            title: ftd::markdown_line(s.title.as_str()),
            id: s.id,
            children: s.children.into_iter().map(Into::into).collect(),
        }
    }
}

#[derive(Debug, serde::Serialize)]
pub struct TocItemCompat {
    pub id: String,
    pub url: String,
    pub number: Vec<u8>,
    pub title: String,
    pub children: Vec<TocItemCompat>,
}

#[derive(PartialEq, Debug, Default, serde::Serialize, serde::Deserialize, Clone)]
pub struct TocItem {
    pub id: String,
    pub url: String,
    pub number: Vec<u8>,
    pub title: ftd::Rendered,
    pub children: Vec<TocItem>,
}

impl TocItem {
    pub(crate) fn to_toc_item_compat(&self) -> TocItemCompat {
        TocItemCompat {
            id: self.id.to_string(),
            url: self.url.to_string(),
            number: self.number.to_vec(),
            title: self.title.original.to_string(),
            children: self
                .children
                .iter()
                .map(|item| item.to_toc_item_compat())
                .collect(),
        }
    }
}

#[derive(PartialEq, Debug, Default, Clone, serde::Serialize, serde::Deserialize)]
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
                url: id_to_url(id.as_str()),
                id,
                title: ftd::markdown_line(rest.trim()),
                children: vec![],
                number: vec![],
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

fn id_to_url(id: &str) -> String {
    if id.starts_with("https://") || id.starts_with("http://") || id.starts_with('/') {
        id.to_string()
    } else {
        format!("/{}/", id)
    }
}

impl ToC {
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

        Ok(ToC {
            items: construct_tree_util(parser.finalize()?),
        })
    }
}

#[derive(thiserror::Error, Debug)]
pub enum ParseError {
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
        if !(stack_tree.is_empty() || get_top_level(&stack_tree) <= level) {
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
        }
        stack_tree.push(node);
    }
    stack_tree
}

#[cfg(test)]
mod test {
    use indoc::indoc;
    use pretty_assertions::assert_eq;

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
                    super::TocItem {
                        id: "amitu/realm/intro".to_string(),
                        url: "/amitu/realm/intro/".to_string(),
                        title: ftd::markdown_line("What is Realm?"),
                        number: vec![],
                        children: vec![
                            super::TocItem {
                                id: "amitu/realm/routing".to_string(),
                                url: "/amitu/realm/routing/".to_string(),
                                title: ftd::markdown_line("Routing is Hard"),
                                children: vec![],
                                number: vec![],
                            },
                            super::TocItem {
                                id: "amitu/realm/backend-routing".to_string(),
                                url: "/amitu/realm/backend-routing/".to_string(),
                                title: ftd::markdown_line("What does Realm do?"),
                                children: vec![],
                                number: vec![],
                            },
                            super::TocItem {
                                id: "amitu/realm/type-safety".to_string(),
                                url: "/amitu/realm/type-safety/".to_string(),
                                title: ftd::markdown_line("Backend Data And Type Safety"),
                                children: vec![],
                                number: vec![],
                            },
                        ],
                    },
                    super::TocItem {
                        id: "amitu/realm/tutorial".to_string(),
                        url: "/amitu/realm/tutorial/".to_string(),
                        title: ftd::markdown_line("Tutorial"),
                        children: vec![],
                        number: vec![],
                    },
                    super::TocItem {
                        id: "amitu/realm/how-tos".to_string(),
                        url: "/amitu/realm/how-tos/".to_string(),
                        title: ftd::markdown_line("How To Guides"),
                        number: vec![],
                        children: vec![
                            super::TocItem {
                                id: "amitu/realm/how-to/api".to_string(),
                                url: "/amitu/realm/how-to/api/".to_string(),
                                title: ftd::markdown_line("How to Write An API?"),
                                children: vec![],
                                number: vec![],
                            },
                            super::TocItem {
                                id: "amitu/realm/how-to/form".to_string(),
                                url: "/amitu/realm/how-to/form/".to_string(),
                                title: ftd::markdown_line("How To Validate Forms?"),
                                children: vec![],
                                number: vec![],
                            },
                            super::TocItem {
                                id: "amitu/realm/how-to/ports".to_string(),
                                url: "/amitu/realm/how-to/ports/".to_string(),
                                title: ftd::markdown_line("How to write custom ports?"),
                                children: vec![],
                                number: vec![],
                            },
                            super::TocItem {
                                id: "amitu/realm/how-to/file-upload".to_string(),
                                url: "/amitu/realm/how-to/file-upload/".to_string(),
                                title: ftd::markdown_line("How to handle file uploads?"),
                                children: vec![],
                                number: vec![],
                            },
                            super::TocItem {
                                id: "amitu/realm/how-to/animation".to_string(),
                                url: "/amitu/realm/how-to/animation/".to_string(),
                                title: ftd::markdown_line("How to do animations?"),
                                children: vec![],
                                number: vec![],
                            },
                        ],
                    },
                ]
            }
        );
    }
}
