mod column;
mod image;
mod row;
mod text;

pub use crate::raw::column::Column;
pub use crate::raw::image::Image;
pub use crate::raw::row::Row;
pub use crate::raw::text::Text;

pub enum Element {
    Row(Row),
    Column(Column),
    Text(Text),
    Image(Image),
}

// page
// -- using: lib
//
// -- pr: This is title of PR
// github-repo: amitu/foo
// number: 12
//
//
// content of lib:
//
// -- enum:
// id: icon
// type: String
// github: /static/github.png
//
// -- enum:
// id: pr-status
// type: String
// default: not-started
// not-started: Not Started
// merged: Merged

// -- ftd/row:
// id: pr    -- defining a new function named pr, can be invoked by `-- pr:`
// $title: $caption:String$     -- this makes caption required, and its type must be String
// $github-repo: $String$       -- compulsory param
// $number: $int:optional$      --
// $status: $pr-status:not-started$
// children: ./title ./status ./gh-icon ./repo-name ./pr-link
//
// --- ftd/text:
// id: title
// text: $title
//
// --- ftd/image:
// id: gh-icon
// src: =icon.github
//
// --- ftd/text:
// id: pr-link
// link: http://github.com/$github-repo/$number
// text: "# $number"  # string expression
// if: $number  # bool expression
//
// --- ftd/null:
// id: pr-link
//
//
//
//
//
// pub enum Symbol {
//    Use(Use),
//    Func(Func),
//    Invocation(Invocation),
//    Enum(Enum),
// }
