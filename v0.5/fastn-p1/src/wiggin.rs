/// calls `inner_ender` for all the embedded section inside section in the
/// list and then calls `ender` for the list itself
#[expect(unused)]
pub fn ender<T: SectionProxy>(
    _source: &str,
    _o: &mut fastn_p1::ParseOutput,
    _sections: Vec<T>,
) -> Vec<T> {
    todo!()
}

/// converts a section list, with interleaved `-- end: <section-name>`, into a nested section list
///
/// example:
/// [{section: "foo"}, {section: "bar"}, "-- end: foo"] -> [{section: "foo", children: [{section: "bar"}]}]
#[expect(unused)]
pub fn inner_ender<T: SectionProxy>(
    _source: &str,
    _o: &mut fastn_p1::ParseOutput,
    _sections: Vec<T>,
) -> Vec<T> {
    todo!()
}

pub enum Mark<'input> {
    #[expect(dead_code)]
    Start(&'input str),
    #[expect(dead_code)]
    End(&'input str),
}

/// we are using a proxy trait so we can write tests against a fake type, and then implement the
/// trait for the real Section type
#[expect(unused)]
pub trait SectionProxy: Sized {
    /// returns the name of the section, and if it starts or ends the section
    fn name<'input>(
        &'input self,
        source: &'input str,
    ) -> Result<Mark<'input>, fastn_p1::SingleError>;
    fn add_children(&mut self, children: Vec<Self>);
}

impl SectionProxy for fastn_p1::Section {
    fn name<'input>(
        &'input self,
        source: &'input str,
    ) -> Result<Mark<'input>, fastn_p1::SingleError> {
        let span = &self.init.name.name.name;
        let name = &source[span.start..span.end];
        if name != "end" {
            return Ok(Mark::Start(name));
        }

        let caption = match self.caption.as_ref() {
            Some(caption) => caption,
            None => return Err(fastn_p1::SingleError::SectionNameNotFoundForEnd),
        };

        let v = match (caption.get(0), caption.len()) {
            (Some(fastn_p1::SES::String(span)), 1) => &source[span.start..span.end].trim(),
            (Some(_), _) => return Err(fastn_p1::SingleError::EndContainsData),
            (None, _) => return Err(fastn_p1::SingleError::SectionNameNotFoundForEnd),
        };

        // if v is not a single word, we have a problem
        if v.contains(' ') || v.contains('\t') {
            // SES::String cannot contain new lines.
            return Err(fastn_p1::SingleError::EndContainsData);
        }

        Ok(Mark::End(v))
    }

    fn add_children(&mut self, children: Vec<Self>) {
        self.children = children;
    }
}

#[allow(dead_code)] // #[expect(dead_code)] is not working
struct DummySection {
    name: String,
    is_end: bool,
    children: Vec<DummySection>,
}

impl SectionProxy for DummySection {
    fn name<'input>(
        &'input self,
        _source: &'input str,
    ) -> Result<Mark<'input>, fastn_p1::SingleError> {
        if self.is_end {
            Ok(Mark::End(&self.name))
        } else {
            Ok(Mark::Start(&self.name))
        }
    }

    fn add_children(&mut self, children: Vec<Self>) {
        self.children = children;
    }
}
