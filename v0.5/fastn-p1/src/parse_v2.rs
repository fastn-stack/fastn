pub struct Parser<'a> {
    // pub scanner: &'a mut lyn::Scanner,
    pub output: &'a mut fastn_p1::ParseOutput<'a>,
    pub edit: &'a fastn_p1::Edit,
}

impl<'a> Parser<'a> {
    pub fn new(
        // scanner: &'a mut lyn::Scanner,
        output: &'a mut fastn_p1::ParseOutput<'a>,
        edit: &'a fastn_p1::Edit,
    ) -> Parser<'a> {
        Parser {
            // scanner,
            output,
            edit,
        }
    }
}
