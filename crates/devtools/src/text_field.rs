//! A small, unwrapped multi-line editor. Each logical line is shaped separately.
use crate::{actions::*, buffer::Buffer, theme::Theme};
use gpui::*;
use std::ops::Range;

const LINE_HEIGHT: f32 = 24.;

/// GPUI panics if a shaped text run contains `\n` / `\r`.
fn flatten_text_run(text: &str) -> SharedString {
    crate::ui::single_line(text)
}

pub struct Edited;
impl EventEmitter<Edited> for TextField {}

#[derive(Clone)]
struct LineLayout {
    start: usize,
    end: usize,
    bounds: Bounds<Pixels>,
    shaped: ShapedLine,
}

pub struct TextField {
    pub focus: FocusHandle,
    pub buffer: Buffer,
    multiline: bool,
    read_only: bool,
    placeholder: SharedString,
    scroll: ScrollHandle,
    layouts: Vec<LineLayout>,
    selecting: bool,
    pending_cursor: bool,
    preferred_x: Option<Pixels>,
}

impl TextField {
    pub fn new(
        multiline: bool,
        read_only: bool,
        placeholder: &'static str,
        cx: &mut Context<Self>,
    ) -> Self {
        Self {
            focus: cx.focus_handle(),
            buffer: Buffer::new(multiline),
            multiline,
            read_only,
            placeholder: flatten_text_run(placeholder),
            scroll: ScrollHandle::new(),
            layouts: Vec::new(),
            selecting: false,
            pending_cursor: false,
            preferred_x: None,
        }
    }
    pub fn focus(&self, window: &mut Window) {
        self.focus.focus(window);
    }
    pub fn set_output(&mut self, text: &str, cx: &mut Context<Self>) {
        self.buffer.set_text_normalized(text);
        self.scroll.set_offset(point(px(0.), px(0.)));
        cx.notify();
    }
    /// Replace the whole buffer (editable). Emits [`Edited`].
    pub fn set_input(&mut self, text: &str, cx: &mut Context<Self>) {
        if self.read_only {
            self.set_output(text, cx);
            return;
        }
        self.buffer.select_all();
        self.edit(None, text, false, None, cx);
        self.scroll.set_offset(point(px(0.), px(0.)));
    }
    pub fn set_placeholder(&mut self, placeholder: &'static str, cx: &mut Context<Self>) {
        // Placeholders are drawn as a single shaped run — never keep newlines.
        self.placeholder = flatten_text_run(placeholder);
        cx.notify();
    }

    /// Switch between line field and multiline editor. Strips newlines when
    /// entering single-line mode.
    pub fn set_multiline(&mut self, multiline: bool, cx: &mut Context<Self>) {
        if self.multiline == multiline {
            return;
        }
        self.multiline = multiline;
        self.buffer.set_multiline(multiline);
        self.preferred_x = None;
        self.scroll.set_offset(point(px(0.), px(0.)));
        cx.notify();
    }

    pub fn clear(&mut self, cx: &mut Context<Self>) {
        self.buffer.select_all();
        self.edit(None, "", false, None, cx);
    }
    pub fn reveal(&mut self, byte: usize, cx: &mut Context<Self>) {
        self.buffer.place_cursor_without_undo(byte, false);
        self.moved(cx);
    }
    fn moved(&mut self, cx: &mut Context<Self>) {
        self.pending_cursor = true;
        cx.notify();
    }
    fn edit(
        &mut self,
        range: Option<Range<usize>>,
        text: &str,
        mark: bool,
        selected: Option<Range<usize>>,
        cx: &mut Context<Self>,
    ) {
        if self.read_only {
            return;
        }
        let changed = self.buffer.replace(range, text, mark, selected);
        self.preferred_x = None;
        self.moved(cx);
        if changed {
            cx.emit(Edited);
        }
    }
    fn horizontal(&mut self, right: bool, extend: bool, cx: &mut Context<Self>) {
        let selection = self.buffer.selection();
        let byte = if !extend && !selection.is_empty() {
            if right {
                selection.end
            } else {
                selection.start
            }
        } else if right {
            self.buffer.next()
        } else {
            self.buffer.previous()
        };
        self.buffer.place_cursor_without_undo(byte, extend);
        self.preferred_x = None;
        self.moved(cx);
    }
    fn edge(&mut self, end: bool, extend: bool, cx: &mut Context<Self>) {
        let range = self.buffer.line_range();
        self.buffer
            .place_cursor_without_undo(if end { range.end } else { range.start }, extend);
        self.preferred_x = None;
        self.moved(cx);
    }
    fn vertical(&mut self, down: bool, extend: bool, cx: &mut Context<Self>) {
        let row = self.buffer.cursor_line();
        let Some(current) = self.layouts.get(row) else {
            return;
        };
        let x = self.preferred_x.unwrap_or_else(|| {
            current
                .shaped
                .x_for_index(self.buffer.cursor() - current.start)
        });
        self.preferred_x = Some(x);
        let row = if down {
            (row + 1).min(self.layouts.len().saturating_sub(1))
        } else {
            row.saturating_sub(1)
        };
        let target = &self.layouts[row];
        let byte = target.start
            + target
                .shaped
                .closest_index_for_x(x)
                .min(target.end - target.start);
        self.buffer.place_cursor_without_undo(byte, extend);
        self.moved(cx);
    }
    fn erase(&mut self, forward: bool, cx: &mut Context<Self>) {
        if self.read_only {
            return;
        }
        let mut range = self.buffer.selection();
        if range.is_empty() {
            if forward {
                range.end = self.buffer.next();
            } else {
                range.start = self.buffer.previous();
            }
        }
        self.edit(Some(range), "", false, None, cx);
    }
    fn copy_selection(&self, cx: &mut Context<Self>) {
        let range = self.buffer.selection();
        if !range.is_empty() {
            cx.write_to_clipboard(ClipboardItem::new_string(
                self.buffer.text()[range].to_owned(),
            ));
        }
    }
    fn mouse_index(&self, position: Point<Pixels>) -> usize {
        let Some(line) = self
            .layouts
            .iter()
            .find(|line| position.y < line.bounds.bottom())
            .or_else(|| self.layouts.last())
        else {
            return 0;
        };
        line.start
            + line
                .shaped
                .closest_index_for_x(position.x - line.bounds.left())
                .min(line.end - line.start)
    }
    fn mouse_down(&mut self, event: &MouseDownEvent, window: &mut Window, cx: &mut Context<Self>) {
        self.focus(window);
        self.selecting = true;
        let byte = self.mouse_index(event.position);
        self.buffer
            .place_cursor_without_undo(byte, event.modifiers.shift);
        self.preferred_x = None;
        self.moved(cx);
    }
    fn mouse_move(&mut self, event: &MouseMoveEvent, _: &mut Window, cx: &mut Context<Self>) {
        if self.selecting {
            let byte = self.mouse_index(event.position);
            self.buffer.place_cursor_without_undo(byte, true);
            self.moved(cx);
        }
    }
    fn mouse_up(&mut self, _: &MouseUpEvent, _: &mut Window, _: &mut Context<Self>) {
        self.selecting = false;
    }
}

impl EntityInputHandler for TextField {
    fn text_for_range(
        &mut self,
        range: Range<usize>,
        actual: &mut Option<Range<usize>>,
        _: &mut Window,
        _: &mut Context<Self>,
    ) -> Option<String> {
        let range = self.buffer.range_from_utf16(range);
        *actual = Some(self.buffer.range_to_utf16(range.clone()));
        Some(self.buffer.text()[range].to_owned())
    }
    fn selected_text_range(
        &mut self,
        _: bool,
        _: &mut Window,
        _: &mut Context<Self>,
    ) -> Option<UTF16Selection> {
        Some(UTF16Selection {
            range: self.buffer.range_to_utf16(self.buffer.selection()),
            reversed: self.buffer.reversed(),
        })
    }
    fn marked_text_range(&self, _: &mut Window, _: &mut Context<Self>) -> Option<Range<usize>> {
        self.buffer
            .marked
            .clone()
            .map(|r| self.buffer.range_to_utf16(r))
    }
    fn unmark_text(&mut self, _: &mut Window, cx: &mut Context<Self>) {
        self.buffer.unmark();
        cx.notify();
    }
    fn replace_text_in_range(
        &mut self,
        range: Option<Range<usize>>,
        text: &str,
        _: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let range = range.map(|r| self.buffer.range_from_utf16(r));
        self.edit(range, text, false, None, cx);
    }
    fn replace_and_mark_text_in_range(
        &mut self,
        range: Option<Range<usize>>,
        text: &str,
        selected: Option<Range<usize>>,
        _: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let range = range.map(|r| self.buffer.range_from_utf16(r));
        self.edit(range, text, true, selected, cx);
    }
    fn bounds_for_range(
        &mut self,
        range: Range<usize>,
        _: Bounds<Pixels>,
        _: &mut Window,
        _: &mut Context<Self>,
    ) -> Option<Bounds<Pixels>> {
        let range = self.buffer.range_from_utf16(range);
        let line = self
            .layouts
            .iter()
            .find(|l| range.start >= l.start && range.start <= l.end)?;
        Some(Bounds::from_corners(
            point(
                line.bounds.left() + line.shaped.x_for_index(range.start - line.start),
                line.bounds.top(),
            ),
            point(
                line.bounds.left()
                    + line
                        .shaped
                        .x_for_index(range.end.min(line.end) - line.start),
                line.bounds.bottom(),
            ),
        ))
    }
    fn character_index_for_point(
        &mut self,
        point: Point<Pixels>,
        _: &mut Window,
        _: &mut Context<Self>,
    ) -> Option<usize> {
        Some(self.buffer.to_utf16(self.mouse_index(point)))
    }
}

struct TextLine {
    field: Entity<TextField>,
    row: usize,
    start: usize,
    text: SharedString,
    placeholder: bool,
}

impl IntoElement for TextLine {
    type Element = Self;
    fn into_element(self) -> Self {
        self
    }
}

impl Element for TextLine {
    type RequestLayoutState = ShapedLine;
    type PrepaintState = (Option<PaintQuad>, Option<PaintQuad>);
    fn id(&self) -> Option<ElementId> {
        None
    }
    fn source_location(&self) -> Option<&'static std::panic::Location<'static>> {
        None
    }
    fn request_layout(
        &mut self,
        _: Option<&GlobalElementId>,
        _: Option<&InspectorElementId>,
        window: &mut Window,
        cx: &mut App,
    ) -> (LayoutId, ShapedLine) {
        let style = window.text_style();
        let run = TextRun {
            len: self.text.len(),
            font: style.font(),
            color: if self.placeholder {
                Theme::get(cx).muted.into()
            } else {
                style.color
            },
            background_color: None,
            underline: None,
            strikethrough: None,
        };
        let mark = self.field.read(cx).buffer.marked.clone();
        let mut runs = Vec::new();
        if !self.placeholder
            && let Some(mark) = mark
        {
            let start = mark.start.saturating_sub(self.start).min(self.text.len());
            let end = mark.end.saturating_sub(self.start).min(self.text.len());
            if start > 0 {
                runs.push(TextRun {
                    len: start,
                    ..run.clone()
                });
            }
            if end > start {
                runs.push(TextRun {
                    len: end - start,
                    underline: Some(UnderlineStyle {
                        color: Some(run.color),
                        thickness: px(1.),
                        wavy: false,
                    }),
                    ..run.clone()
                });
            }
            if end < self.text.len() {
                runs.push(TextRun {
                    len: self.text.len() - end,
                    ..run.clone()
                });
            }
        } else {
            runs.push(run);
        }
        let shaped = window.text_system().shape_line(
            self.text.clone(),
            style.font_size.to_pixels(window.rem_size()),
            &runs,
            None,
        );
        let mut layout = Style::default();
        layout.size.width = (shaped.width + px(8.)).into();
        layout.min_size.width = relative(1.).into();
        layout.size.height = px(LINE_HEIGHT).into();
        layout.flex_shrink = 0.;
        (window.request_layout(layout, [], cx), shaped)
    }
    fn prepaint(
        &mut self,
        _: Option<&GlobalElementId>,
        _: Option<&InspectorElementId>,
        bounds: Bounds<Pixels>,
        line: &mut ShapedLine,
        window: &mut Window,
        cx: &mut App,
    ) -> Self::PrepaintState {
        let theme = Theme::get(cx);
        self.field.update(cx, |field, _| {
            let end = self.start + if self.placeholder { 0 } else { self.text.len() };
            let selection = field.buffer.selection();
            let row = field.buffer.cursor_line();
            let caret = if row == self.row && selection.is_empty() {
                let x = if self.placeholder {
                    px(0.)
                } else {
                    line.x_for_index(field.buffer.cursor() - self.start)
                };
                Some(fill(
                    Bounds::new(
                        point(bounds.left() + x, bounds.top()),
                        size(px(2.), px(LINE_HEIGHT)),
                    ),
                    theme.accent,
                ))
            } else {
                None
            };
            let selection =
                if selection.start <= end && selection.end > self.start && !selection.is_empty() {
                    let left = line.x_for_index(
                        selection
                            .start
                            .saturating_sub(self.start)
                            .min(end - self.start),
                    );
                    let mut right = line.x_for_index(
                        selection
                            .end
                            .saturating_sub(self.start)
                            .min(end - self.start),
                    );
                    if selection.end > end {
                        right += px(8.);
                    }
                    Some(fill(
                        Bounds::from_corners(
                            point(bounds.left() + left, bounds.top()),
                            point(bounds.left() + right, bounds.bottom()),
                        ),
                        theme.selected,
                    ))
                } else {
                    None
                };
            if field.pending_cursor && row == self.row {
                field.pending_cursor = false;
                let viewport = field.scroll.bounds();
                let old = field.scroll.offset();
                let max = field.scroll.max_offset();
                let mut new = old;
                if bounds.top() < viewport.top() {
                    new.y += viewport.top() - bounds.top();
                }
                if bounds.bottom() > viewport.bottom() {
                    new.y -= bounds.bottom() - viewport.bottom();
                }
                let x = bounds.left()
                    + if self.placeholder {
                        px(0.)
                    } else {
                        line.x_for_index(field.buffer.cursor() - self.start)
                    };
                if x < viewport.left() {
                    new.x += viewport.left() - x;
                }
                if x + px(4.) > viewport.right() {
                    new.x -= x + px(4.) - viewport.right();
                }
                new.x = new.x.max(-max.width).min(px(0.));
                new.y = new.y.max(-max.height).min(px(0.));
                if new != old {
                    field.scroll.set_offset(new);
                    window.request_animation_frame();
                }
            }
            (selection, caret)
        })
    }
    fn paint(
        &mut self,
        _: Option<&GlobalElementId>,
        _: Option<&InspectorElementId>,
        bounds: Bounds<Pixels>,
        line: &mut ShapedLine,
        paint: &mut Self::PrepaintState,
        window: &mut Window,
        cx: &mut App,
    ) {
        let field = self.field.read(cx);
        let focused = field.focus.is_focused(window);
        if focused && !field.read_only && field.buffer.cursor_line() == self.row {
            window.handle_input(
                &field.focus,
                ElementInputHandler::new(bounds, self.field.clone()),
                cx,
            );
        }
        if let Some(selection) = paint.0.take() {
            window.paint_quad(selection);
        }
        line.paint(bounds.origin, px(LINE_HEIGHT), window, cx)
            .expect("paint shaped line");
        if focused && let Some(caret) = paint.1.take() {
            window.paint_quad(caret);
        }
        self.field.update(cx, |field, _| {
            let layout = LineLayout {
                start: self.start,
                end: self.start + if self.placeholder { 0 } else { self.text.len() },
                bounds,
                shaped: line.clone(),
            };
            if self.row < field.layouts.len() {
                field.layouts[self.row] = layout;
            } else {
                field.layouts.push(layout);
            }
        });
    }
}

impl Render for TextField {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = Theme::get(cx);
        let mut start = 0;
        let lines: Vec<_> = self
            .buffer
            .text()
            .split('\n')
            .enumerate()
            .map(|(row, text)| {
                let placeholder = self.buffer.text().is_empty();
                let line = TextLine {
                    field: cx.entity(),
                    row,
                    start,
                    text: if placeholder {
                        self.placeholder.clone()
                    } else {
                        // Defend against \r left in a "line" after a naive split.
                        flatten_text_run(text)
                    },
                    placeholder,
                };
                start += text.len() + 1;
                line
            })
            .collect();
        self.layouts.truncate(lines.len());
        div()
            .key_context(if self.multiline {
                "Editor"
            } else {
                "LineField"
            })
            .track_focus(&self.focus)
            .cursor_text()
            .size_full()
            .min_w_0()
            .min_h_0()
            .bg(theme.panel)
            .text_color(theme.text)
            .font(crate::theme::monospace())
            .text_size(px(13.))
            .line_height(px(LINE_HEIGHT))
            .on_action(cx.listener(|this, _: &Left, _, cx| this.horizontal(false, false, cx)))
            .on_action(cx.listener(|this, _: &Right, _, cx| this.horizontal(true, false, cx)))
            .on_action(cx.listener(|this, _: &SelectLeft, _, cx| this.horizontal(false, true, cx)))
            .on_action(cx.listener(|this, _: &SelectRight, _, cx| this.horizontal(true, true, cx)))
            .on_action(cx.listener(|this, _: &Home, _, cx| this.edge(false, false, cx)))
            .on_action(cx.listener(|this, _: &End, _, cx| this.edge(true, false, cx)))
            .on_action(cx.listener(|this, _: &SelectHome, _, cx| this.edge(false, true, cx)))
            .on_action(cx.listener(|this, _: &SelectEnd, _, cx| this.edge(true, true, cx)))
            .on_action(cx.listener(|this, _: &Up, _, cx| this.vertical(false, false, cx)))
            .on_action(cx.listener(|this, _: &Down, _, cx| this.vertical(true, false, cx)))
            .on_action(cx.listener(|this, _: &SelectUp, _, cx| this.vertical(false, true, cx)))
            .on_action(cx.listener(|this, _: &SelectDown, _, cx| this.vertical(true, true, cx)))
            .on_action(cx.listener(|this, _: &Backspace, _, cx| this.erase(false, cx)))
            .on_action(cx.listener(|this, _: &Delete, _, cx| this.erase(true, cx)))
            .on_action(cx.listener(|this, _: &SelectAll, _, cx| {
                this.buffer.select_all();
                this.moved(cx);
            }))
            .on_action(cx.listener(|this, _: &Copy, _, cx| this.copy_selection(cx)))
            .on_action(cx.listener(|this, _: &Cut, _, cx| {
                this.copy_selection(cx);
                if !this.read_only {
                    this.edit(None, "", false, None, cx);
                }
            }))
            .on_action(cx.listener(|this, _: &Paste, _, cx| {
                if !this.read_only
                    && let Some(text) = cx.read_from_clipboard().and_then(|item| item.text())
                {
                    this.edit(None, &text, false, None, cx);
                }
            }))
            .on_action(cx.listener(|this, _: &Undo, _, cx| {
                if !this.read_only && this.buffer.undo() {
                    this.preferred_x = None;
                    this.moved(cx);
                    cx.emit(Edited);
                }
            }))
            .on_action(cx.listener(|this, _: &InsertNewline, _, cx| {
                if this.multiline {
                    this.edit(None, "\n", false, None, cx);
                }
            }))
            .on_mouse_down(MouseButton::Left, cx.listener(Self::mouse_down))
            .on_mouse_move(cx.listener(Self::mouse_move))
            .on_mouse_up(MouseButton::Left, cx.listener(Self::mouse_up))
            .on_mouse_up_out(MouseButton::Left, cx.listener(Self::mouse_up))
            .child(
                div()
                    .id("text-scroll")
                    .size_full()
                    .overflow_scroll()
                    .track_scroll(&self.scroll)
                    .flex()
                    .flex_col()
                    .children(lines),
            )
    }
}
