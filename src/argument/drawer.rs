use crate::{argument::value::ValueVariant, state::State, widget::DrawerRef, Args};
use std::{
    ops::Deref,
    sync::{Arc, Mutex},
};
use tui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Span, Spans},
    widgets::{Block, Borders, Paragraph, Widget},
};
use tui_textarea::TextArea;

type DState<'s, 'u> = (&'s State, &'u mut usize);

struct ToRenderImpl<'c, 's, 'r, 'a, 'u> {
    current: &'c Option<(usize, usize)>,
    state: &'s DState<'r, 'u>,
    col_index: usize,
    values: Option<Vec<(String, Text<'a>)>>,
}
impl<'c, 's, 'r, 'a, 'u> ToRenderImpl<'c, 's, 'r, 'a, 'u> {
    fn highlight_style(&self, index: usize) -> Option<Style> {
        self.current.and_then(|(row, col)| {
            ((index == row && self.col_index == col
                || (index == 0 && self.col_index == col || index == row && self.col_index == 0)
                    && *self.state.1 + 1 == self.state.0.position.len())
                && self.state.0.input.is_none())
            .then_some(self.state.0.highlight_style)
        })
    }
    fn text(
        &self,
        index: usize,
        (block, text): (String, Arc<Mutex<TextArea<'a>>>),
    ) -> ToRender<'a> {
        let mut _text = text.lock().unwrap();
        let cursor_style = self.current.map_or(false, |(row, col)| {
            row == index && col == self.col_index && self.state.0.input.is_some()
        });
        _text.set_block(
            Block::default()
                .title(block)
                .borders(Borders::TOP)
                .border_style(Style::default().add_modifier(Modifier::ITALIC)),
        );
        if !cursor_style {
            _text.set_style(self.highlight_style(index).unwrap_or(self.state.0.style));
        }
        let style = _text.style();
        _text.set_cursor_style(if !cursor_style {
            style
        } else {
            Style::default().add_modifier(Modifier::REVERSED)
        });
        _text.set_alignment(if cursor_style {
            Alignment::Left
        } else {
            Alignment::Center
        });
        drop(_text);
        ToRender::Text(text)
    }
    fn spans(&self, index: usize, span: Span<'a>) -> Spans<'a> {
        if let Some(style) = self.current.and_then(|(row, col)| {
            ((index == 0 && self.col_index == col && self.state.0.input.is_none()
                || self.col_index == 0 && index == row && self.state.0.input.is_none()
                || (row == index && col == self.col_index && self.state.0.input.is_some()))
                && *self.state.1 + 1 == self.state.0.position.len())
            .then_some(self.state.0.highlight_style)
        }) {
            Span::styled(span.content, style)
        } else if index == 0 || self.col_index == 0 {
            Span::styled(span.content, Style::default().add_modifier(Modifier::DIM))
        } else {
            span
        }
        .into()
    }
    fn span(&self, index: usize, (block, span): (String, Span<'a>)) -> ToRender<'a> {
        let spans = self.spans(index, span);

        let paragraph = if index > 0 {
            Paragraph::new(spans).block(
                Block::default()
                    .title(block)
                    .borders(if self.col_index > 0 {
                        Borders::TOP
                    } else {
                        Borders::TOP | Borders::RIGHT
                    })
                    .border_style(Style::default().add_modifier(Modifier::ITALIC)),
            )
        } else {
            let paragraph = Paragraph::new(spans);
            if self.col_index == 0 {
                paragraph.block(Block::default().borders(Borders::RIGHT))
            } else {
                paragraph
            }
        }
        .alignment(tui::layout::Alignment::Center);

        ToRender::Paragraph(if let Some(style) = self.highlight_style(index) {
            paragraph.style(style)
        } else {
            paragraph.style(self.state.0.style)
        })
    }
}

enum ToRender<'b> {
    Text(Arc<Mutex<TextArea<'b>>>),
    Paragraph(Paragraph<'b>),
}
impl<'c, 's, 'r, 'a, 'u> From<ToRenderImpl<'c, 's, 'r, 'a, 'u>> for Vec<ToRender<'a>> {
    #[inline]
    fn from(mut to_render: ToRenderImpl<'c, 's, 'r, 'a, 'u>) -> Self {
        to_render
            .values
            .take()
            .unwrap()
            .into_iter()
            .enumerate()
            .map(|(index, (block, span))| match span {
                Text::Text(text) => to_render.text(index, (block, text)),
                Text::Span(span) => to_render.span(index, (block, span)),
            })
            .collect::<Vec<ToRender<'a>>>()
    }
}

enum Text<'b> {
    Text(Arc<Mutex<TextArea<'b>>>),
    Span(Span<'b>),
}

pub struct Drawer<'r, 'a>(pub(super) &'r Args<'a>);

impl<'r, 'a> Drawer<'r, 'a> {
    fn current(&self, state: &DState) -> Option<(usize, usize)> {
        state.0.node(*state.1).and_then(|current| {
            current.args().and_then(|(current, col)| {
                self.names
                    .iter()
                    .enumerate()
                    .find_map(|(number, text)| (*text == *current).then_some((number + 1, col + 1)))
            })
        })
    }
    fn values(&self) -> Vec<Vec<(String, Text<'a>)>> {
        self.columns
            .iter()
            .map(|column| {
                [("".to_string(), Text::Span(Span::from(column.clone())))]
                    .into_iter()
                    .chain(self.names.iter().map(|name| {
                        self.get_value(name, column).map_or(
                            ("".to_string(), Text::Span(Span::from(""))),
                            |value| {
                                (
                                    value.0.to_string(),
                                    match &value.1 {
                                        ValueVariant::TextArea(text) => {
                                            Text::<'a>::Text(text.clone())
                                        }
                                        value => Text::Span(Span::from(value.to_string())),
                                    },
                                )
                            },
                        )
                    }))
                    .collect()
            })
            .collect()
    }
    fn names(&self) -> Vec<(String, Text<'a>)> {
        ["".to_string()]
            .into_iter()
            .chain(self.names.iter().cloned())
            .map(|name| ("".to_string(), Text::Span(Span::from(name))))
            .collect()
    }
    fn chunks(
        area: tui::layout::Rect,
        buf: &mut tui::buffer::Buffer,
        state: &State,
        names: &Vec<(String, Text<'a>)>,
        values: &[Vec<(String, Text<'a>)>],
    ) -> (Vec<Rect>, Vec<Vec<Rect>>) {
        let widths = {
            [names]
                .into_iter()
                .chain(values.iter())
                .map(|values| {
                        values.iter().fold(0usize, |max, (name, value)| {
                            max.max(name.len() + 1).max(match value {
                                Text::Text(text) => text.lock().unwrap().lines().first().map_or(0, |str| str.len()),
                                Text::Span(span) => span.width()
                            })
                        })
                    } + 3)
                .collect::<Vec<usize>>()
        };
        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Length(widths.iter().fold(1usize, |res, width| res + *width) as u16),
                Constraint::Min(2),
            ])
            .split(area);

        let inner_area = {
            let block = Block::default().style(state.style).borders(Borders::RIGHT);
            let inner_area = block.inner(chunks[0]);
            tui::widgets::Widget::render(block, chunks[0], buf);
            inner_area
        };

        let inner = Layout::default()
            .direction(Direction::Horizontal)
            .constraints(
                widths
                    .iter()
                    .map(|width| Constraint::Length(*width as u16))
                    .chain([Constraint::Min(2)].into_iter())
                    .collect::<Vec<Constraint>>(),
            )
            .split(inner_area)
            .into_iter()
            .map(|rect| {
                Layout::default()
                    .direction(Direction::Vertical)
                    .constraints(
                        names
                            .iter()
                            .enumerate()
                            .map(|(index, _)| Constraint::Length(if index > 0 { 2 } else { 1 }))
                            .chain([Constraint::Min(2)].into_iter())
                            .collect::<Vec<Constraint>>(),
                    )
                    .split(rect)
            })
            .collect();
        (chunks, inner)
    }
}

impl<'a, 'r> DrawerRef<'a> for Drawer<'r, 'a> {
    fn render(
        &self,
        area: tui::layout::Rect,
        buf: &mut tui::buffer::Buffer,
        state: DState<'_, '_>,
    ) {
        if *state.1 > state.0.position.len() {
            return;
        }

        let current = self.current(&state);
        let chunks = {
            let values = self.values();
            let names = self.names();

            let (chunks, inner_chunks) = Self::chunks(area, buf, state.0, &names, &values);

            [names]
                .into_iter()
                .chain(values.into_iter())
                .enumerate()
                .map(|(col_index, values)| ToRenderImpl {
                    current: &current,
                    state: &state,
                    col_index,
                    values: Some(values),
                })
                .map(Vec::from)
                .zip(inner_chunks.into_iter())
                .map(|(paragraphes, chunks)| paragraphes.into_iter().zip(chunks.into_iter()))
                .for_each(|paragraphes| {
                    paragraphes.for_each(|(paragraph, area)| match paragraph {
                        ToRender::Text(text) => {
                            text.lock().unwrap().widget().render(area, buf);
                        }
                        ToRender::Paragraph(paragraph) => paragraph.render(area, buf),
                    })
                });

            chunks
        };

        if let Some(branch) = current.and_then(|(name, col)| {
            self.get_value_by_indexes(name - 1, col - 1)
                .and_then(|value| match &value.1 {
                    ValueVariant::Struct(branch) => Some(branch),
                    _ => None,
                })
        }) {
            *state.1 += 1;
            branch.render(chunks[1], buf, state)
        }
    }
}

impl<'a> Deref for Drawer<'_, 'a> {
    type Target = Args<'a>;
    fn deref(&self) -> &Self::Target {
        self.0
    }
}
