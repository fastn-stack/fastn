pub struct Parser<'a> {
    // scanner: &'a mut lyn::Scanner,
    output: &'a mut fastn_p1::ParseOutput<'a>,
    edit: &'a fastn_p1::Edit,
}

impl<'a> Parser<'a> {
    fn new(
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
