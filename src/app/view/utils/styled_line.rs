use ratatui::{
    style::Style,
    text::{Line, Span},
};

use crate::app::view::utils::paragraph::Slicable;

#[derive(Debug, PartialEq, Clone)]
pub struct StyleSlice {
    byte_offset: usize,
    len: usize,
    /// A Ratatui `Style`.
    style: Style,
}

impl StyleSlice {
    fn new(byte_offset: usize, len: usize, style: Style) -> Self {
        Self {
            byte_offset,
            len,
            style,
        }
    }

    fn start_idx(&self) -> usize {
        self.byte_offset
    }

    fn end_idx(&self) -> usize {
        self.byte_offset + self.len
    }
}

#[derive(Debug, Default, PartialEq, Clone)]
pub struct StyledLine {
    /// The literal text to render.
    content: String,

    /// A Ratatui `Style` applied to the whole line.
    style: Style,

    /// Style slices in current line.
    style_slices: Vec<StyleSlice>,
}

impl From<String> for StyledLine {
    fn from(value: String) -> Self {
        let style_slice = StyleSlice::new(0, value.len(), Style::default());
        Self {
            content: value,
            style: Style::default(),
            style_slices: vec![style_slice],
        }
    }
}

impl StyledLine {
    pub fn with_style(mut self, style: Style) -> Self {
        self.style = style;
        self
    }

    pub fn content(&self) -> &str {
        &self.content
    }

    pub fn style_slices_mut(&mut self) -> &mut Vec<StyleSlice> {
        &mut self.style_slices
    }

    /// Appends content to line.
    pub fn append(&mut self, content: impl Into<String>, style: Style) {
        let content: String = content.into();
        let byte_offset = self.content.len();
        let style_slice = StyleSlice::new(byte_offset, content.len(), style);
        self.content += &content;
        self.style_slices.push(style_slice);
    }

    /// Inserts prefix span to line.
    pub fn insert_prefix(&mut self, prefix: Span) {
        let len = prefix.content.len();
        self.content.insert_str(0, &prefix.content);
        for style_slices in &mut self.style_slices {
            style_slices.byte_offset += len;
        }
        self.style_slices
            .insert(0, StyleSlice::new(0, len, prefix.style));
    }

    /// Patchs `style` onto given byte index range. None means patch style to the whole line.
    pub fn patch_style(
        self,
        style: Style,
        range: Option<(usize /*start_offset*/, usize /*end_offset*/)>,
    ) -> Self {
        if let Some((start_offset, end_offset)) = range {
            let style_slices = self
                .style_slices
                .into_iter()
                .flat_map(|s| {
                    if start_offset > s.end_idx() || end_offset <= s.start_idx() {
                        return vec![s];
                    }

                    let mut new_slices: Vec<StyleSlice> = vec![];
                    if s.start_idx() < start_offset {
                        new_slices.push(StyleSlice {
                            byte_offset: s.start_idx(),
                            len: start_offset - s.start_idx(),
                            style: s.style,
                        })
                    }
                    new_slices.push(StyleSlice {
                        byte_offset: start_offset.max(s.start_idx()),
                        len: end_offset.min(s.end_idx()) - start_offset.max(s.start_idx()),
                        style: s.style.patch(style),
                    });
                    if s.end_idx() > end_offset {
                        new_slices.push(StyleSlice {
                            byte_offset: end_offset,
                            len: s.end_idx() - end_offset,
                            style: s.style,
                        })
                    }
                    new_slices
                })
                .collect();
            Self {
                content: self.content,
                style: self.style,
                style_slices,
            }
        } else {
            Self {
                content: self.content,
                style: self.style.patch(style),
                style_slices: self.style_slices,
            }
        }
    }
}

impl Slicable for StyledLine {
    /// Returns a subslice of `StyledLine` starting at `offset` of length `len`.
    fn slice(&self, offset: usize, len: usize) -> StyledLine {
        let style_slices = self
            .style_slices
            .iter()
            .filter_map(|s| {
                if s.start_idx() >= offset + len || s.end_idx() < offset {
                    None
                } else {
                    let start = s.start_idx().max(offset);
                    let len = s
                        .end_idx()
                        .min(offset.saturating_add(len))
                        .saturating_sub(start);
                    Some(StyleSlice::new(start.saturating_sub(offset), len, s.style))
                }
            })
            .collect();
        StyledLine {
            content: self
                .content
                .get(offset..offset + len)
                .unwrap_or("")
                .to_string(),
            style: self.style,
            style_slices,
        }
    }
}

impl AsRef<str> for StyledLine {
    fn as_ref(&self) -> &str {
        self.content()
    }
}

impl From<&StyledLine> for Line<'_> {
    fn from(line: &StyledLine) -> Self {
        let spans: Vec<Span> = line
            .style_slices
            .iter()
            .map(|s| {
                let content: String = line.content[s.byte_offset..s.byte_offset + s.len].into();
                Span::from(content).style(s.style)
            })
            .collect();
        let tui_line = Line::from(spans);
        tui_line.patch_style(line.style)
    }
}
