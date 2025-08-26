pub mod editor;
pub mod highlight;
pub mod parser;
pub mod render;
pub use editor::Editor;
pub use highlight::Highlighter;
pub use parser::Parser;

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum Item<'a> {
    /// `\n`
    // TODO(emilk): add Style here so empty heading still uses up the right amount of space.
    Newline,

    /// Text
    Text(Style, &'a str),

    /// title, url
    Hyperlink(Style, &'a str, &'a str),

    /// leading space before e.g. a [`Self::BulletPoint`].
    Indentation(usize),

    /// >
    QuoteIndent,

    /// - a point well made.
    BulletPoint,

    /// 1. numbered list. The string is the number(s).
    NumberedPoint(&'a str),

    /// ---
    Separator,

    /// language, code
    CodeBlock(&'a str, &'a str),
}

#[derive(Copy, Clone, Debug, Default, Eq, PartialEq)]
pub struct Style {
    /// # heading (large text)
    pub heading: bool,

    /// > quoted (slightly dimmer color or other font style)
    pub quoted: bool,

    /// `code` (monospace, some other color)
    pub code: bool,

    /// self.strong* (emphasized, e.g. bold)
    pub strong: bool,

    /// _underline_
    pub underline: bool,

    /// ~strikethrough~
    pub strikethrough: bool,

    /// /italics/
    pub italics: bool,

    /// $small$
    pub small: bool,

    /// ^raised^
    pub raised: bool,
}
