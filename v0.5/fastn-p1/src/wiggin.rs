/// converts a section list, with interleaved `-- end: <section-name>`, into a nested section list
///
/// example:
/// [{section: "foo"}, {section: "bar"}, "-- end: foo"] -> [{section: "foo", children: [{section: "bar"}]}]
#[expect(unused)]
pub fn ender(
    _source: &str,
    _o: &mut fastn_p1::ParseOutput,
    _sections: Vec<fastn_p1::Section>,
) -> Vec<fastn_p1::Section> {
    todo!()
}

enum Mark<'input> {
    #[expect(dead_code)]
    Start(&'input str),
    #[expect(dead_code)]
    End(&'input str),
}

#[expect(unused)]
trait Extractor {
    /// returns the name of the section, and if it start or ends the section
    fn name<'input>(&self, source: &'input str) -> Result<Mark<'input>, fastn_p1::SingleError>;
}

impl Extractor for fastn_p1::Section {
    fn name<'input>(&self, source: &'input str) -> Result<Mark<'input>, fastn_p1::SingleError> {
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
}
