// in src/app/view/utils/paragraph.rs
use unicode_segmentation::UnicodeSegmentation;
use unicode_width::UnicodeWidthStr;

pub trait LineLike: AsRef<str> {}

impl<T> LineLike for T where T: AsRef<str> {}

pub trait Slicable {
    fn slice(&self, offset: usize, len: usize) -> Self;
}

#[derive(Debug)]
pub struct Paragraph<L: LineLike> {
    content: L,
    lines: Vec<L>,
    byte_offset: usize,
    line_offset: usize,
}

impl<L: LineLike + Slicable> Paragraph<L> {
    pub fn build(content: L) -> Self {
        Self {
            content,
            lines: Vec::new(),
            byte_offset: 0,
            line_offset: 0,
        }
    }

    pub fn content(&self) -> &L {
        &self.content
    }

    pub fn lines(&self) -> &[L] {
        &self.lines
    }

    pub fn byte_offset(&self) -> usize {
        self.byte_offset
    }

    pub fn line_offset(&self) -> usize {
        self.line_offset
    }

    pub fn len(&self) -> usize {
        self.content.as_ref().len()
    }

    pub fn reflow(&mut self, lines: Vec<L>, byte_offset: usize, line_offset: usize) {
        self.lines = lines;
        self.byte_offset = byte_offset;
        self.line_offset = line_offset;
    }

    /// Returns cursor position given a byte idx against cached lines, assuming it's in this
    /// paragraph.
    pub fn find_cursor_position(&self, cursor_byte_idx: usize) -> (u16 /*x*/, u16 /*y*/) {
        let mut line_byte_offset = 0;
        for (line_idx, line) in self.lines.iter().enumerate() {
            if cursor_byte_idx < self.byte_offset + line_byte_offset + line.as_ref().len() {
                let x = line.as_ref()[..(cursor_byte_idx - self.byte_offset - line_byte_offset)]
                    .width();
                let y = self.line_offset + line_idx;
                return (x as u16, y as u16);
            }
            line_byte_offset += line.as_ref().len();
        }

        // handle cursor at the end of paragraph
        let x = self.lines.last().map_or(0, |l| l.as_ref().width());
        let y = self.line_offset + self.lines.len() - 1;
        (x as u16, y as u16)
    }

    /// Returns cursor byte idx given a position against cached visual lines, assuming it's in
    /// this paragraph.
    pub fn find_cursor_byte_idx(&self, cursor_position: (u16 /*x*/, u16 /*y*/)) -> usize {
        let (x, y) = cursor_position;
        let mut line_idx = y as usize - self.line_offset;
        // clamp to last line if out of paragraph bound
        line_idx = line_idx.clamp(0, self.lines.len() - 1);

        let line = &self.lines[line_idx];
        let line_byte_offset: usize = self.lines[..line_idx]
            .iter()
            .map(|l| l.as_ref().len())
            .sum();

        // iterate graphemes
        let mut current_width = 0;
        for (grapheme_byte_offset, grapheme) in line.as_ref().grapheme_indices(true) {
            let grapheme_width = UnicodeWidthStr::width(grapheme);
            if current_width + grapheme_width > x as usize {
                return self.byte_offset + line_byte_offset + grapheme_byte_offset;
            }
            current_width += grapheme_width;
        }
        // clamp to last byte if out of line bound
        self.byte_offset + line_byte_offset + line.as_ref().len()
    }
}
