use ui::*;
use na;

pub struct CombinedSlide {
    items: Vec<Box<Element>>,
    _margin: f32,
    _item_gap: f32,
}

impl CombinedSlide {
    pub fn new() -> CombinedSlide {
        CombinedSlide {
            items: Vec::new(),
            _margin: 30.0,
            _item_gap: 0.0,
        }
    }

    pub fn with<E: Element + 'static>(mut self, slide: E) -> Self {
        self.items.push(Box::new(slide) as Box<Element>);
        self
    }

    pub fn margin(mut self, margin: f32) -> Self {
        self._margin = margin;
        self
    }

    pub fn gap(mut self, gap: f32) -> Self {
        self._item_gap = gap;
        self
    }
}

impl Element for CombinedSlide {
    fn inflate(&mut self, base: &mut Base) {
        for item in self.items.drain(..) {
            base.add_boxed(item);
        }
    }

    fn resize(&mut self, base: &mut Base) {
        let margin = (self._margin * base.scale()) as i32;
        let item_gap = (self._item_gap * base.scale()) as i32;
        base.layout_vertical(margin, item_gap)
    }
}

struct InterpolationTarget {
    rotation: na::UnitQuaternion<f32>,
    position: na::Vector3<f32>,
    t: f32,
}

struct ElementInfo {
    rotation: na::UnitQuaternion<f32>,
    position: na::Vector3<f32>,
    target: Option<InterpolationTarget>,
}

impl ElementInfo {
    pub fn new() -> ElementInfo {
        ElementInfo {
            rotation: na::UnitQuaternion::<f32>::identity(),
            position: [0.0, 0.0, 0.0].into(),
            target: None,
        }
    }

    pub fn transform(&self) -> na::Projective3<f32> {
        if let Some(ref target) = self.target {
            let alpha = smootherstep(0.0, 1.0, target.t);
            let interpolated_position = self.position.lerp(&target.position, alpha);
            let interpolated_rotation = self.rotation.nlerp(&target.rotation, alpha);

            na::convert::<_, na::Projective3<_>>( na::Translation3::new(interpolated_position.x, interpolated_position.y, interpolated_position.z))
                * na::convert::<_, na::Projective3<_>>(interpolated_rotation)
        } else {
            na::convert::<_, na::Projective3<_>>( na::Translation3::new(self.position.x, self.position.y, self.position.z))
                * na::convert::<_, na::Projective3<_>>(self.rotation)
        }
    }

    pub fn transition_from(&mut self, source_pos: &na::Vector3<f32>) {
        self.target = Some(InterpolationTarget {
            rotation: na::UnitQuaternion::<f32>::identity(),
            position: [0.0, 0.0, 0.0].into(),
            t: 0.0,
        });
        self.position = *source_pos;
    }

    pub fn transition_to(&mut self, target_pos: &na::Vector3<f32>) {
        self.target = Some(InterpolationTarget {
            rotation: na::UnitQuaternion::<f32>::identity(),
            position: *target_pos,
            t: 0.0,
        });
        self.position = [0.0, 0.0, 0.0].into();
    }

    pub fn is_transitioning(&self) -> bool {
        self.target.is_some()
    }
}

fn smootherstep(edge0: f32, edge1: f32, x: f32) -> f32 {
    let x = na::clamp((x - edge0) / (edge1 - edge0), 0.0, 1.0);
    x * x * x * (x * (x * 6.0 - 15.0) + 10.0)
}

pub struct Presentation {
    slides: Vec<Box<Element>>,
    num_elements: usize,
    slide_index: usize,
    element_info: Vec<ElementInfo>,
    last_size: ResolvedSize,
}

impl Presentation {
    pub fn new() -> Presentation {
        Presentation {
            slides: vec![],
            num_elements: 0,
            slide_index: 0,
            element_info: vec![],
            last_size: ResolvedSize::zero(),
        }
    }

    pub fn with_slide<E: Element + 'static>(mut self, slide: E) -> Self {
        self.slides.push(Box::new(slide) as Box<Element>);
        self.num_elements += 1;
        self.element_info.push(ElementInfo::new());
        self
    }
}

impl Element for Presentation {
    fn inflate(&mut self, base: &mut Base) {
        for slide in self.slides.drain(..) {
            base.add_boxed(slide);
        }
        base.enable_actions(true);
    }

    fn resize(&mut self, base: &mut Base) {
        if self.num_elements == 0 {
            return base.layout_empty();
        }

        let current_slide_index = self.slide_index;
        let size = match base.box_size() {
            BoxSize::Hidden => return base.layout_empty(),
            BoxSize::Auto => {
                let mut resolved_child_size = None;

                base.children_mut(|i, mut child| {
                    if i == current_slide_index {
                        resolved_child_size = child.element_resize(BoxSize::Auto);
                    } else {
                        if !self.element_info[i].is_transitioning() {
                            child.element_resize(BoxSize::Hidden);
                        }
                    }
                });

                resolved_child_size
            },
            BoxSize::Fixed { w, h } => {
                let mut resolved_child_size = None;

                base.children_mut(|i, mut child| {
                    if i == current_slide_index {
                        resolved_child_size = child.element_resize(BoxSize::Fixed { w, h });
                    } else {
                        if !self.element_info[i].is_transitioning() {
                            child.element_resize(BoxSize::Hidden);
                        }
                    }

                    child.element_transform(&self.element_info[i].transform());
                });

                resolved_child_size
            },
        };

        if let Some(size) = size {
            self.last_size = size;
        }
        base.resolve_size(size);
    }

    fn update(&mut self, base: &mut Base, delta: f32) {
        let mut someone_updating = false;

        const UPDATE_TIME: f32 = 0.5;
        const UPDATE_SPEED: f32 = 1.0 / UPDATE_TIME;

        base.children_mut(|i, mut child| {
            let (end, update) = match self.element_info[i].target {
                Some(ref mut target) => {
                    target.t += UPDATE_SPEED * delta;
                    (target.t >= 1.0, true)
                },
                None => (false, false),
            };

            if update {
                if end {
                    self.element_info[i].position = self.element_info[i].target.as_mut().unwrap().position;
                    self.element_info[i].rotation = self.element_info[i].target.as_mut().unwrap().rotation;
                    self.element_info[i].target = None;
                }

                child.element_transform(&self.element_info[i].transform());

                someone_updating = true;
            }
        });

        if !someone_updating {
            base.enable_update(false);
        }
    }

    fn action(&mut self, base: &mut Base, action: UiAction) {
        match action {
            UiAction::NextSlide => {
                if self.slide_index < self.num_elements - 1 {
                    let previous_index = self.slide_index;
                    self.slide_index += 1;

                    self.element_info[self.slide_index].transition_from(
                        &[self.last_size.w as f32, 0.0, 0.0].into()
                    );
                    self.element_info[previous_index].transition_to(
                        &[-self.last_size.w as f32, 0.0, 0.0].into()
                    );
                    base.enable_update(true);
                    base.invalidate_size();
                }
            },
            UiAction::PreviousSlide => {
                if self.slide_index > 0 {
                    let previous_index = self.slide_index;
                    self.slide_index -= 1;

                    self.element_info[self.slide_index].transition_from(
                        &[-self.last_size.w as f32, 0.0, 0.0].into()
                    );
                    self.element_info[previous_index].transition_to(
                        &[self.last_size.w as f32, 0.0, 0.0].into()
                    );
                    base.enable_update(true);
                    base.invalidate_size();
                }
            },
        }
    }
}

#[derive(Copy, Clone, Eq, PartialEq)]
pub enum Align {
    Left,
    Center,
}

pub struct TextSlide {
    single_line: Option<primitives::Text>,
    multi_lines: Vec<primitives::Text>,

    text_string: String,
    text_size: f32,

    align: Align,

    _bold: bool,
    _italic: bool,
    _monospaced: bool,

    _word_wrap: bool,

    _color: na::Vector4<u8>,

    _highlighter: Option<String>,
    highlighted_lines: Option<Vec<(syntect::highlighting::Style, usize)>>,
}

impl TextSlide {
    pub fn new(text: &str) -> TextSlide {
        TextSlide {
            single_line: None,
            multi_lines: Vec::with_capacity(32),

            text_string: text.into(),
            text_size: 60.0,

            align: Align::Left,

            _bold: false,
            _italic: false,
            _monospaced: false,

            _word_wrap: true,

            _color: [0, 0, 0, 255].into(),

            _highlighter: None,
            highlighted_lines: None,
        }
    }

    pub fn size(mut self, size: f32) -> TextSlide {
        self.text_size = size;
        self
    }

    pub fn centered(mut self) -> TextSlide {
        self.align = Align::Center;
        self
    }

    pub fn bold(mut self, value: bool) -> TextSlide {
        self._bold = value;
        self
    }

    pub fn italic(mut self, value: bool) -> TextSlide {
        self._italic = value;
        self
    }

    pub fn monospaced(mut self, value: bool) -> TextSlide {
        self._monospaced = value;
        self
    }

    pub fn word_wrap(mut self, value: bool) -> TextSlide {
        self._word_wrap = value;
        self
    }

    pub fn color(mut self, color: na::Vector4<u8>) -> TextSlide {
        self._color = color;
        self
    }

    pub fn highlight(mut self, lang: &str) -> TextSlide {
        self._highlighter = Some(lang.into());
        self.update_highlighted_lines();
        self
    }

    fn update_highlighted_lines(&mut self) {
        match self._highlighter {
            Some(ref lang) => {
                use syntect::easy::HighlightLines;
                use syntect::parsing::SyntaxSet;
                use syntect::highlighting::{ThemeSet};
                use syntect::util::{LinesWithEndings};

                let ps = SyntaxSet::load_defaults_newlines();
                let ts = ThemeSet::load_defaults();

                let syntax = ps.find_syntax_by_extension(&lang[..]).unwrap();
                let mut h = HighlightLines::new(syntax, &ts.themes["InspiredGitHub"]);

                self.highlighted_lines = Some(
                    LinesWithEndings::from(&self.text_string)
                        .flat_map(|line|
                            h.highlight(line, &ps)
                                .into_iter()
                                .map(|(style, s)| (style, s.len()))
                        )
                        .collect()
                );
            },
            None => self.highlighted_lines = None,
        }
    }
}

impl Element for TextSlide {
    fn inflate(&mut self, base: &mut Base) {
        let mut text = base.primitives().text(self.text_string.clone(), self._bold, self._italic, self._monospaced, self._color).expect("failed to create text");
        text.set_size(self.text_size);
        self.single_line = Some(text);
    }

    fn resize(&mut self, base: &mut Base) {
        let box_size = base.box_size();

        let resolved_size: Option<ResolvedSize> = match box_size {
            BoxSize::Hidden => return base.layout_empty(),
            BoxSize::Auto => {
                let m = match self.single_line.as_mut() {
                    None => return base.layout_empty(),
                    Some(line) => {
                        let m = match line.measurement().measure() {
                            None => return base.layout_empty(),
                            Some(s) => s,
                        };

                        line.set_hidden(false);
                        line.set_position(0.0, m.ascent + m.line_gap / 2.0);

                        m
                    }
                };

                for line in self.multi_lines.iter_mut() {
                    line.set_hidden(true);
                }

                Some(ResolvedSize {
                    w: m.width.round() as i32,
                    h: m.height.round() as i32
                })
            },
            BoxSize::Fixed { w, h, .. } => {
                let (metrics, mut positions) = match self.single_line.as_mut() {
                    None => return base.layout_empty(),
                    Some(line) => {
                        line.set_hidden(true);
                        match line.measurement().measure() {
                            None => return base.layout_empty(),
                            Some(m) => (m, line.measurement().glyph_positions())
                        }
                    }
                };

                enum Separator {
                    Space,
                    NewLine,
                }

                #[derive(Debug, Copy, Clone)]
                enum Line {
                    Empty(Segment),
                    WordWrap(Segment),
                    ParagraphBreak(Segment),
                }

                #[derive(Debug, Copy, Clone)]
                struct Segment {
                    offset: u32,
                    len: u32,
                    width: f32,
                }

                fn is_separator(text: &str, word_wrap: bool, m: &primitives::GlyphMeasurement) -> Option<Separator> {
                    let s: &str = &text[m.byte_offset as usize..(m.byte_offset + m.len) as usize];
                    if s.chars().any(|c| c == '\n') {
                        Some(Separator::NewLine)
                    } else if word_wrap && s.chars().all(|c| c.is_whitespace()) {
                        Some(Separator::Space)
                    } else {
                        None
                    }
                }

                let max_line_w = w as f32;

                let mut lines = Vec::new();

                let mut leading_space = None;
                let mut current_line = None;
                let mut candidate_word_whitespace = None;
                let mut candidate_word = None;

                while let Some(g) = positions.next() {
                    match (current_line, candidate_word_whitespace, candidate_word) {
                        (None, None, None) => match is_separator(&self.text_string, self._word_wrap, &g) {
                            Some(Separator::NewLine) => {
                                lines.push(Line::Empty(Segment {
                                    offset: g.byte_offset,
                                    len: g.len,
                                    width: g.x_advance,
                                }));
                                leading_space = None;
                            },
                            Some(Separator::Space) => match leading_space {
                                None => leading_space = Some(Segment {
                                    offset: g.byte_offset,
                                    len: g.len,
                                    width: g.x_advance,
                                }),
                                Some(_) => {
                                    let leading_space = leading_space.as_mut().unwrap();

                                    leading_space.len += g.len;
                                    leading_space.width += g.x_advance;
                                }
                            },
                            None => match leading_space {
                                None => current_line = Some(Segment {
                                    offset: g.byte_offset,
                                    len: g.len,
                                    width: g.x_advance,
                                }),
                                Some(r_leading_space) => {
                                    current_line = Some(Segment {
                                        offset: r_leading_space.offset,
                                        len: r_leading_space.len + g.len,
                                        width: r_leading_space.width + g.x_advance,
                                    });
                                    leading_space = None;
                                }
                            }
                        }
                        (Some(r_current_line), None, None) => match is_separator(&self.text_string, self._word_wrap, &g) {
                            Some(Separator::NewLine) => {
                                lines.push(Line::ParagraphBreak(r_current_line));
                                current_line = None;
                            },
                            Some(Separator::Space) => candidate_word_whitespace = Some(Segment {
                                offset: g.byte_offset,
                                len: g.len,
                                width: g.x_advance,
                            }),
                            None => {
                                let current_line = current_line.as_mut().unwrap();

                                current_line.len += g.len;
                                current_line.width += g.x_advance;
                            }
                        }
                        (Some(r_current_line), Some(_), None) => match is_separator(&self.text_string, self._word_wrap, &g) {
                            Some(Separator::NewLine) => {
                                lines.push(Line::ParagraphBreak(r_current_line));
                                current_line = None;
                                candidate_word_whitespace = None;
                            },
                            Some(Separator::Space) => {
                                let candidate_word_whitespace = candidate_word_whitespace.as_mut().unwrap();

                                candidate_word_whitespace.len += g.len;
                                candidate_word_whitespace.width += g.x_advance;
                            },
                            None => candidate_word = Some(Segment {
                                offset: g.byte_offset,
                                len: g.len,
                                width: g.x_advance,
                            })
                        }
                        (Some(r_current_line), Some(r_candidate_word_whitespace), Some(r_candidate_word)) => match is_separator(&self.text_string, self._word_wrap, &g) {
                            Some(Separator::NewLine) => {
                                let line_width_with_candidate_word = r_current_line.width + r_candidate_word_whitespace.width + r_candidate_word.width;
                                if line_width_with_candidate_word <= max_line_w {
                                    lines.push(Line::ParagraphBreak(Segment {
                                        width: line_width_with_candidate_word,
                                        offset: r_current_line.offset,
                                        len: r_current_line.len + r_candidate_word_whitespace.len + r_candidate_word.len,
                                    }));
                                } else {
                                    lines.push(Line::WordWrap(r_current_line));
                                    lines.push(Line::ParagraphBreak(r_candidate_word));
                                }
                                current_line = None;
                                candidate_word_whitespace = None;
                                candidate_word = None;
                            },
                            Some(Separator::Space) => {
                                let line_width_with_candidate_word = r_current_line.width + r_candidate_word_whitespace.width + r_candidate_word.width;
                                if line_width_with_candidate_word <= max_line_w {
                                    {
                                        let current_line = current_line.as_mut().unwrap();
                                        current_line.len += r_candidate_word_whitespace.len + r_candidate_word.len;
                                        current_line.width += r_candidate_word_whitespace.width + r_candidate_word.width;
                                    }

                                    candidate_word_whitespace = Some(Segment {
                                        offset: g.byte_offset,
                                        len: g.len,
                                        width: g.x_advance,
                                    });
                                    candidate_word = None;
                                } else {
                                    lines.push(Line::WordWrap(r_current_line));
                                    current_line = candidate_word;
                                    candidate_word = None;
                                    candidate_word_whitespace = Some(Segment {
                                        offset: g.byte_offset,
                                        len: g.len,
                                        width: g.x_advance,
                                    });
                                }
                            },
                            None => {
                                let candidate_word = candidate_word.as_mut().unwrap();

                                candidate_word.len += g.len;
                                candidate_word.width += g.x_advance;
                            }
                        }
                        _ => unreachable!("invalid state"),
                    }
                }

                match (current_line, candidate_word_whitespace, candidate_word) {
                    (None, None, None) => lines.push(Line::Empty(Segment {
                        offset: self.text_string.len() as u32,
                        len: 0,
                        width: 0.0,
                    })),
                    (Some(r_current_line), None, None) => {
                        lines.push(Line::ParagraphBreak(r_current_line))
                    },
                    (Some(r_current_line), Some(_), None) => {
                        lines.push(Line::ParagraphBreak(r_current_line))
                    },
                    (Some(r_current_line), Some(r_candidate_word_whitespace), Some(r_candidate_word)) => {
                        let line_width_with_candidate_word = r_current_line.width + r_candidate_word_whitespace.width + r_candidate_word.width;
                        if line_width_with_candidate_word <= max_line_w {
                            lines.push(Line::ParagraphBreak(Segment {
                                width: line_width_with_candidate_word,
                                offset: r_current_line.offset,
                                len: r_current_line.len + r_candidate_word_whitespace.len + r_candidate_word.len,
                            }));
                        } else {
                            lines.push(Line::WordWrap(r_current_line));
                            lines.push(Line::ParagraphBreak(r_candidate_word));
                        }
                    },
                    _ => unreachable!("invalid state"),
                }


                let mut max_text_width: f32 = lines.iter().map(|l| match l {
                    Line::Empty(_) => 0.0,
                    Line::ParagraphBreak(s) => s.width,
                    Line::WordWrap(s) => s.width,
                }).max_by(|x, y| if x > y {
                    ::std::cmp::Ordering::Greater
                } else {
                    ::std::cmp::Ordering::Less
                }).unwrap_or(0.0);
                let max_text_height = lines.len() as f32 * metrics.height;

                let left_offset = max_line_w / 2.0 - max_text_width / 2.0;

                self.multi_lines.clear();

                let mut highlight_index = self.highlighted_lines.as_ref().map(|_| 0);
                let mut highlight_index_byte = 0;

                let mut top: f32 = h as f32 / 2.0 - max_text_height / 2.0 + metrics.descent;
                for line in lines.iter() {
                    let (_new_line, do_render, s) = match line {
                        Line::WordWrap(s) => (false, true, s),
                        Line::ParagraphBreak(s) => (true, true, s),
                        Line::Empty(s) => (true, false, s),
                    };

                    let (mut x, y) = if self.align == Align::Center {
                        (left_offset + max_text_width / 2.0 - s.width / 2.0, top + metrics.height)
                    } else {
                        (left_offset, top + metrics.height)
                    };

                    match highlight_index {
                        None => {
                            if do_render {
                                let mut text = base.primitives()
                                    .text(self.text_string[s.offset as usize..(s.offset + s.len) as usize].to_string(),
                                          self._bold, self._italic, self._monospaced, self._color)
                                    .unwrap();
                                text.set_position(x, y);
                                text.set_size(self.text_size);
                                self.multi_lines.push(text);
                            }
                        },
                        Some(mut ix) => {
                            let mut mismatch = s.offset as i32 - highlight_index_byte as i32;

                            while mismatch > 0 {
                                let (_h_item, h_len) = self.highlighted_lines.as_ref().unwrap()[ix];
                                highlight_index_byte += h_len;
                                ix += 1;
                                mismatch = s.offset as i32 - highlight_index_byte as i32;
                            }

                            if do_render {
                                while (highlight_index_byte as u32) < s.offset + s.len {
                                    let (h_item, h_len) = self.highlighted_lines.as_ref().unwrap()[ix];

                                    let mut text = &self.text_string[highlight_index_byte as usize..(highlight_index_byte as usize + h_len)];

                                    while text.ends_with("\n") {
                                        text = &text[..text.len() - 1];
                                    }

                                    let mut text = base.primitives()
                                        .text(text,
                                              self._bold, self._italic, self._monospaced, [h_item.foreground.r, h_item.foreground.g, h_item.foreground.b, h_item.foreground.a].into())
                                        .unwrap();
                                    text.set_position(x, y);
                                    text.set_size(self.text_size);
                                    x += text.measurement().measure().unwrap().width;

                                    self.multi_lines.push(text);

                                    highlight_index_byte += h_len;
                                    ix += 1;
                                }
                            }

                            highlight_index = Some(ix);
                        }
                    }

                    top += metrics.height;
                }

                Some(ResolvedSize {
                    w,
                    h
                })
            },
        };

        base.resolve_size(resolved_size);
    }
}



