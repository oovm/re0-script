use std::{
    fmt::{Debug, Formatter},
    fs::read_to_string,
    ops::Range,
};
use url::Url;

pub struct PositionDisplay<'i> {
    file: &'i Option<Url>,
    span: &'i Range<usize>,
}

impl<'i> PositionDisplay<'i> {
    pub fn new(file: &'i Option<Url>, span: &'i Range<usize>) -> Self {
        Self { file, span }
    }
    pub fn as_position(&self) -> Option<(&Url, usize, usize)> {
        let url = self.file.as_ref()?;
        let text = read_to_string(url.to_file_path().ok()?).ok()?;
        let (line, col) = offset_to_line_col(&text, self.span.start);
        Some((url, line, col))
    }
}

fn offset_to_line_col(code: &str, offset: usize) -> (usize, usize) {
    let mut line = 1;
    let mut col = 1;
    let mut current_offset = 0;

    for c in code.chars() {
        if current_offset == offset {
            return (line, col);
        }

        if c == '\n' {
            line += 1;
            col = 1;
        }
        else {
            col += 1;
        }

        current_offset += c.len_utf8();
        // 检查当前偏移量是否超过了源代码的长度
        if current_offset > code.len() {
            break;
        }
    }

    (line, col)
}

impl<'i> Debug for PositionDisplay<'i> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self.as_position() {
            Some((url, line, col)) => write!(f, "{url}:{line}:{col}"),
            None => write!(f, "{}..{}", self.span.start, self.span.end),
        }
    }
}
