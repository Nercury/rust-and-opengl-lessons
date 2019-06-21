use ui::*;
use crate::na;
use super::presentation::*;

pub struct RustFest {

}

impl RustFest {
    pub fn new() -> RustFest {
        RustFest {}
    }
}

impl Element for RustFest {
    fn inflate(&mut self, base: &mut Base) {
        base.add(
            Presentation::new()
                .with_slide(
                    CombinedSlide::new()
                        .with(
                            TextSlide::new("This experiment contains presentation slides \n\
                            built in Rust. The code is not that nice or performant, because this \n\
                            was done in a hurry. The presentation itself is also not that good. \n\
                            However this experiment was very informative and educational for me, \n\
                            and also quite cool to have something resembling a finished thing, even \n\
                            if it is not backed by a very good implementation.")
                                .centered()
                                .word_wrap(false)
                                .size(30.0)
                        )
                        .with(
                            CombinedSlide::new()
                                .with(
                                TextSlide::new("Use arrows <- and -> to change the slide")
                                    .bold(true)
                                    .centered()
                                    .size(25.0)
                                )
                                .with(
                                    TextSlide::new("Use arrows [ and ] to change the scale")
                                        .bold(true)
                                        .centered()
                                        .size(25.0)
                                )
                                .with(
                                    TextSlide::new("B to toggle element borders, P to toggle the profiler")
                                        .bold(true)
                                        .centered()
                                        .size(25.0)
                                )
                                .with(
                                    TextSlide::new("T wireframe, C perspective camera, F fullscreen")
                                        .bold(true)
                                        .centered()
                                        .size(25.0)
                                )
                                .with(
                                    TextSlide::new("WASD to move in perspective camera, Right Mouse to look around")
                                        .bold(true)
                                        .centered()
                                        .size(25.0)
                                )
                        )
                )
                .with_slide(
                    CombinedSlide::new()
                        .with(
                            TextSlide::new("")
                                .size(40.0)
                                .centered()
                        )
                        .with(
                            TextSlide::new("")
                                .size(40.0)
                        )
                        .with(
                            TextSlide::new("What garbage collected languages\ncan not do")
                                .size(70.0)
                                .centered()
                        )
                        .with(
                            TextSlide::new("")
                                .size(40.0)
                        )
                        .with(
                            TextSlide::new("@nercury")
                                .size(40.0)
                        )
                )
                // Rust is fast. So what? Go is fast, Javascript is fast, C# is fast.
                //
                // And then, we hear that Rust does not have a GC, instead, it has a
                // strict rules we have to learn and obey.
                //
                // At this point Rust must offer something quite special to be worth the bargain.
                .with_slide(
                    TextSlide::new("Why Rust?\n\nWhere is my GC?")
                        .size(70.0)
                        .centered()
                )
                // We are going to skip few of those things. We won't talk about zero-cost
                // abstractions. We won't talk about awesome type system or concurrency
                // without data races.
                .with_slide(
                    CombinedSlide::new()
                        .with(
                            TextSlide::new("Multifaceted answer")
                                .size(70.0)
                                .bold(true)
                                .centered()
                        )
                        .with(
                            TextSlide::new(
                                "Zero-cost abstractions\n\
                                Awesome type system\n\
                                Great concurrency story"
                            )
                                .centered()
                                .size(60.0)
                        )
                        .with(
                            TextSlide::new("")
                                .size(70.0)
                                .bold(true)
                                .centered()
                        )
                )
                // Instead, I will talk how Rust offers precise control over memory, and allows
                // us to create convenient abstractions that are both fast and easy to use.
                //
                // But even if Rust advertises that it can do it, new Rust users run into
                // some problems while trying to do convenient things.
                .with_slide(
                    TextSlide::new(
                        "Control over memory\n\
                            vs.\n\
                            Convenient use of memory"
                    )
                        .centered()
                        .size(80.0)
                )
                // The main problem with ownership and borrowing is that this system is tied to
                // the stack.
                .with_slide(
                    CombinedSlide::new()
                        .with(
                            TextSlide::new("Enter:")
                                .size(70.0)
                                .centered()
                        )
                        .with(
                            TextSlide::new(
                                "The Stack Problem"
                            )
                                .centered()
                                .bold(true)
                                .size(80.0)
                        )
                        .with(
                            TextSlide::new("")
                                .size(70.0)
                                .centered()
                        )
                )
                // For example, we may design a text renderer,
                // that gives us back created text buffers:
                .with_slide(
                    TextSlide::new(
                        r##"
let mut renderer = TextRenderer::new();
let buffer: &mut Buffer
    = renderer.create_buffer("Hello");
"##
                    )
                        .word_wrap(false)
                        .monospaced(true)
                        .highlight("rs")
                        .size(40.0)
                )
                // But of course, this buffer has a convenient method to change text position or rotation:
                .with_slide(
                    TextSlide::new(
                        r##"
let mut renderer = TextRenderer::new();
let buffer: &mut Buffer
    = renderer.create_buffer("Hello");

buffer.translate(1.0, 3.3);
buffer.rotate(90.0);
"##
                    )
                        .word_wrap(false)
                        .monospaced(true)
                        .highlight("rs")
                        .size(40.0)
                )
                // Why do we use a renderer? Well, we want to efficiently render all the
                // text in a single draw call:
                .with_slide(
                    TextSlide::new(
                        r##"
let mut renderer = TextRenderer::new();
let buffer: &mut Buffer
    = renderer.create_buffer("Hello");

buffer.translate(1.0, 3.3);
buffer.rotate(90.0);

renderer.render(&open_gl);

// ERROR! cannot borrow `renderer` as mutable more
// than once at a time!
"##
                    )
                        .word_wrap(false)
                        .monospaced(true)
                        .highlight("rs")
                        .size(40.0)
                )
                // Oh wait, we have to finish borrowing to use renderer again:
                .with_slide(
                    TextSlide::new(
                        r##"
let mut renderer = TextRenderer::new();

{
    let buffer: &mut Buffer
        = renderer.create_buffer("Hello");

    buffer.translate(1.0, 3.3);
    buffer.rotate(90.0);
}

renderer.render(&open_gl);
"##
                    )
                        .word_wrap(false)
                        .monospaced(true)
                        .highlight("rs")
                        .size(40.0)
                )
                // This kind of workaround works, but it ties our solution to the borrow checker
                // requirements. This is not what we wanted! We wanted to keep and pass buffer to
                // any other structure, and keep it around so that we can update the translation and
                // rotation from anywhere, anytime!
                //
                // The most flexible workaround is to keep integer handle instead of whole buffer
                .with_slide(
                    CombinedSlide::new()
                        .with(
                            TextSlide::new("Workaround:")
                                .size(70.0)
                                .centered()
                        )
                        .with(
                            TextSlide::new(
                                "Keep Integer Handle instead of the whole Buffer"
                            )
                                .centered()
                                .bold(true)
                                .size(80.0)
                        )
                        .with(
                            TextSlide::new("")
                                .size(70.0)
                                .centered()
                        )
                )
                // If we get a handle, we can know what buffer we refer to elsewhere in the system:
                .with_slide(
                    TextSlide::new(
                        r##"
let mut renderer = TextRenderer::new();
let buffer_handle: u32
    = renderer.create_buffer("Hello");
"##
                    )
                        .word_wrap(false)
                        .monospaced(true)
                        .highlight("rs")
                        .size(40.0)
                )
                // With that, we can pass this handle anywhere! Whenever we want to do anything with it,
                // we just do it over the renderer:
                .with_slide(
                    TextSlide::new(
                        r##"
renderer.translate(buffer_handle, 1.0, 3.3);
renderer.rotate(buffer_handle, 90.0);
"##
                    )
                        .word_wrap(false)
                        .monospaced(true)
                        .highlight("rs")
                        .size(40.0)
                )
                // This works, and this is quite performant solution.
                //
                // But if we are willing to trade a little bit performance for convenience,
                // there is a third solution.
                .with_slide(
                    CombinedSlide::new()
                        .with(
                            TextSlide::new("Enter:")
                                .size(70.0)
                                .centered()
                        )
                        .with(
                            TextSlide::new(
                                "The Hidden World"
                            )
                                .centered()
                                .bold(true)
                                .size(80.0)
                        )
                        .with(
                            TextSlide::new("")
                                .size(70.0)
                                .centered()
                        )
                )
                //Enter the hidden-world pattern.
                //
                //We take this renderer that works with handles, and rename it to `SharedTextRenderer`:
                .with_slide(
                    TextSlide::new(
                        r##"
let mut renderer = SharedTextRenderer::new();
let buffer_handle: u32 =
    renderer.create_buffer("Hello");

renderer.translate(buffer_handle, 1.0, 3.3);
renderer.rotate(buffer_handle, 90.0);
"##
                    )
                        .word_wrap(false)
                        .monospaced(true)
                        .highlight("rs")
                        .size(40.0)
                )
                // It works as before.
                //
                // But in Rust, we can create a wrapper around it. The SharedTextRenderer will
                // become part of the hidden world, where it will live.
                //
                // We will call the wrapper "TextRenderer"
                .with_slide(
                    TextSlide::new(
                        r##"
use std::rc::Rc;
use std::cell::RefCell;

struct TextRenderer {
    shared: Rc<RefCell<SharedTextRenderer>>,
}
"##
                    )
                        .word_wrap(false)
                        .monospaced(true)
                        .highlight("rs")
                        .size(40.0)
                )
                // Yes, we use the dreaded Rc-RefCell here. However, instead of using it for
                // every single object, we hide whole mechanism under it.
                //
                // We can implement `Clone` for the `TextRenderer`.
                .with_slide(
                    TextSlide::new(
                        r##"
use std::rc::Rc;
use std::cell::RefCell;

#[derive(Clone)]
struct TextRenderer {
    shared: Rc<RefCell<SharedTextRenderer>>,
}
"##
                    )
                        .word_wrap(false)
                        .monospaced(true)
                        .highlight("rs")
                        .size(40.0)
                )
                // What has just happened?
                // We can create as many TextRenderer objects as we want, the ownership is no
                // longer managed by the stack, it is managed by `Rc` wrapper.
                //
                // This by itself is pretty much useless, unless we also create a method that
                // creates buffers using the same principle:
                .with_slide(
                    TextSlide::new(
                        r##"
impl TextRenderer {
    pub fn create_buffer(
        &self, text: &str
    ) -> Buffer {
        let handle = self.shared
            .borrow_mut()
            .create_buffer(text);

        Buffer {
            handle,
            shared: self.shared.clone(),
        }
    }
}
"##
                    )
                        .word_wrap(false)
                        .monospaced(true)
                        .highlight("rs")
                        .size(40.0)
                )
                // The buffer wraps the handle and again, the reference to this hidden world
                // that it shares with the Renderer:
                .with_slide(
                    TextSlide::new(
                        r##"
struct Buffer {
    handle: u32,
    shared: Rc<RefCell<SharedTextRenderer>>,
}
"##
                    )
                        .word_wrap(false)
                        .monospaced(true)
                        .highlight("rs")
                        .size(40.0)
                )
                // This is pretty cool. This means we can store this buffer anywhere in our system,
                // and it will always have an access to the originating renderer.
                //
                // Using this access, it can communicate with renderer.
                // For example, text position changes. We can simply call a method
                // on the buffer that changes text position:
                .with_slide(
                    TextSlide::new(
                        r##"
buffer.translate(1.0, 3.3);
"##
                    )
                        .word_wrap(false)
                        .monospaced(true)
                        .highlight("rs")
                        .size(40.0)
                )
                // Inside, we forward the call to the hidden world, to shared TextRenderer:
                .with_slide(
                    TextSlide::new(
                        r##"
impl Buffer {
    pub fn translate(&self, x: f32, y: f32) {
        self.shared
            .borrow_mut()
            .translate(self.handle, x, y);
    }
}
"##
                    )
                        .word_wrap(false)
                        .monospaced(true)
                        .highlight("rs")
                        .size(40.0)
                )
                // Suppose we are rendering a lot of text, and we have many buffers.
                // If we are rendering with GPU, the most efficient way to do it is to upload
                // the changes to GPU in one sweep.

                // When we call translate for renderer, wwe

                // Because the renderer has access to the hidden world with all actual buffer data
                // inside, it is no problem to do just that.
                //
                // Somewhere, we will call render:
                .with_slide(
                    TextSlide::new(
                        r##"
renderer.render(&open_gl);
"##
                    )
                        .word_wrap(false)
                        .monospaced(true)
                        .highlight("rs")
                        .size(40.0)
                )
                // This render function can quickly find if any of the buffers were recently updated
                .with_slide(
                    TextSlide::new(
                        r##"
impl TextRenderer {
    pub fn render(
        &self, gl: &Gl
    ) -> Buffer {
        let shared = self.shared.borrow_mut();

        if shared.has_invalidated_buffers() {

        }

        // ...
    }
}
"##
                    )
                        .word_wrap(false)
                        .monospaced(true)
                        .highlight("rs")
                        .size(40.0)
                )
                // and if they were, it can upload the updates to GPU
                .with_slide(
                    TextSlide::new(
                        r##"
impl TextRenderer {
    pub fn render(
        &self, gl: &Gl
    ) -> Buffer {
        let shared = self.shared.borrow_mut();

        if shared.has_invalidated_buffers() {
            self.update_buffers(shared.get_draw_commands());
        }

        // ...
    }
}
"##
                    )
                        .word_wrap(false)
                        .monospaced(true)
                        .highlight("rs")
                        .size(40.0)
                )
                // otherwise, it can render everything with a single draw call:

                // here, every bit of text with different color is a separate
                // buffer, that contains font glyph sequence

                // when we animate between slides, each of them gets updated, and the renderer
                // sweeps up the changes and uploads them to the GPU.
                .with_slide(
                    TextSlide::new(
                        r##"
impl TextRenderer {
    pub fn render(
        &self, gl: &Gl
    ) -> Buffer {
        let shared = self.shared.borrow_mut();

        if shared.has_invalidated_buffers() {
            self.update_buffers(shared.get_draw_commands());
        }

        // render everything with a single draw call:
        self.draw(gl); // <-- simplifying heavily here
    }
}
"##
                    )
                        .word_wrap(false)
                        .monospaced(true)
                        .highlight("rs")
                        .size(40.0)
                )
                // One more thing: this buffer becomes our own custom resource, like a file:
                .with_slide(
                    CombinedSlide::new()
                        .with(
                            TextSlide::new("The Buffer Is:")
                                .size(70.0)
                                .centered()
                        )
                        .with(
                            TextSlide::new(
                                "Our Own Custom Resource"
                            )
                                .centered()
                                .bold(true)
                                .size(80.0)
                        )
                        .with(
                            TextSlide::new("Like a File")
                                .size(70.0)
                                .centered()
                        )
                )
                // This means we can remove it when it is no longer used by implementing Drop
                // on it.
                .with_slide(
                    TextSlide::new(
                        r##"
impl Drop for Buffer {
    fn drop(&mut self) {
        self.shared
            .borrow_mut()
            .remove_buffer(self.handle);
    }
}
"##
                    )
                        .word_wrap(false)
                        .monospaced(true)
                        .highlight("rs")
                        .size(40.0)
                )
                // The best part: we don't need to.
                .with_slide(
                    CombinedSlide::new()
                        .with(
                            TextSlide::new("But we don't need to implement Drop")
                                .size(70.0)
                                .centered()
                                .bold(true)
                        )
                        .with(
                            TextSlide::new(
                                "Some resources should be dropped automatically\n\
                            Some do not need to be dropped (like font glyphs)\n\
                            Some can only have one reference to one handle\n\
                            Some may have a handle that is reference-counted"
                            )
                                .bold(true)
                                .size(60.0)
                        )
                        .with(
                            TextSlide::new("Best part: we can pick a solution that works best for our use case")
                                .size(70.0)
                                .centered()
                        )
                )
                // This idea, of course, is known to a seasoned Rustacean. Every time
                // we see an API where two seemingly separate objects communicate with each other
                // we can be pretty sure they share a hidden world between them
                .with_slide(
                    CombinedSlide::new()
                        .with(
                            TextSlide::new("")
                                .size(70.0)
                                .centered()
                        )
                        .with(
                            TextSlide::new(
                                "The Hidden World Idea"
                            )
                                .centered()
                                .bold(true)
                                .size(80.0)
                        )
                        .with(
                            TextSlide::new("... is used a lot")
                                .size(70.0)
                                .centered()
                        )
                )
                // Take as an example a rust channel. It has two ends: transmission and receiving
                // Both can be passed everywhere yet can somehow magically send data to each other
                .with_slide(
                    CombinedSlide::new()
                        .with(
                            TextSlide::new("std::mpsc::channel")
                                .size(70.0)
                                .centered()
                        )
                        .with(
                            TextSlide::new(
                                r##"
use std::sync::mpsc::channel;
use std::thread;

let (sender, receiver) = channel();
"##
                            )
                                .word_wrap(false)
                                .monospaced(true)
                                .highlight("rs")
                                .size(40.0)
                        )
                        .with(
                            TextSlide::new("")
                                .size(70.0)
                                .centered()
                        )
                )
                // So, what the GC languages can not do?
                .with_slide(
                    CombinedSlide::new()
                        .with(
                            TextSlide::new("What garbage collected languages\ncan not do")
                                .size(70.0)
                                .centered()
                        )
                )
                // They don't allow this kind of control. And when it matters, we really
                // have no other choice but to use Rust.
                .with_slide(
                    CombinedSlide::new()
                        .with(
                            TextSlide::new(
                                "Where it matters, they don't allow these kind of choices"
                            )
                                .centered()
                                .size(80.0)
                        )
                )
                // With Rust, we saw how we can both have our cake and eat it too
                // We can have nice API surface that is convenient to use, and we can separate it
                // from implementation that is fasr and hides the implementation details
                .with_slide(
                    CombinedSlide::new()
                        .with(
                            TextSlide::new(
                                "It is possible to have nice things, even though it may need a bit of boilerplate code"
                            )
                                .centered()
                                .size(80.0)
                        )
                )
                .with_slide(
                    CombinedSlide::new()
                        .with(
                            TextSlide::new(
                                "Thank you!"
                            )
                                .centered()
                                .bold(true)
                                .size(90.0)
                        )
                )
                .with_slide(
                    CreditsSlide::new()
                )

        );
    }
}

struct CreditsAnumationState {
    angle: f32,
}

pub struct CreditsSlide {
    text_items: Vec<(primitives::Text, CreditsAnumationState)>,
    width: f32,
}

impl CreditsSlide {
    pub fn new() -> CreditsSlide {
        CreditsSlide {
            text_items: vec![],
            width: 0.0,
        }
    }

    fn init_animation(&mut self) {
        let item_len = self.text_items.len();
        let piece_rads = ::std::f32::consts::PI * 2.0 / item_len as f32;
        for (i, (_, item)) in self.text_items.iter_mut().enumerate() {
            item.angle = i as f32 * piece_rads;
        }
    }

    fn update_animation(&mut self, delta: f32) {
        for (_i, (text, item)) in self.text_items.iter_mut().enumerate() {
            item.angle += 0.5 * delta;
            let transform = na::Rotation3::from_axis_angle(&na::Unit::new_normalize(na::Vector3::new(1.0, 1.0, 1.0)), item.angle);
            text.set_transform(&(
                na::convert::<_, na::Projective3<f32>>(transform)
                    * na::convert::<_, na::Projective3<f32>>(na::Translation::from(na::Vector3::new(self.width / 2.0, 0.0, 0.0)))
            ));
        }
    }
}

impl Element for CreditsSlide {
    fn inflate(&mut self, base: &mut Base) {
        for item in &[
            "nalgebra",
            "nalgebra-glm",
            "ncollide3d",
            "slab",
            "slotmap",
            "metrohash",
            "int_hash",
            "sha-1",
            "byteorder",
            "font-kit",
            "harfbuzz_rs",
            "unicode-segmentation",
            "syntect",
            "lyon_path",
            "lyon_tessellation",
            "gl",
            "gl_generator",
            "log",
            "failure",
            "floating-duration",
            "half",
            "euclid",
            "sdl2",
        ] {
            self.text_items.push(
                (base.primitives()
                     .text(item, true, false, true, [20, 20, 20, 255].into()).unwrap(),
                 CreditsAnumationState {
                     angle: 0.0,
                 })
            )
        }
    }

    fn resize(&mut self, base: &mut Base) {
        let box_size = base.box_size();
        match box_size {
            BoxSize::Hidden => base.enable_update(false),
            BoxSize::Auto => {},
            BoxSize::Fixed { w, h } => {
                self.width = w as f32;
                base.enable_update(true);
                base.resolve_size(Some(ResolvedSize { w, h }));
                self.init_animation();
                self.update_animation(0.0);
            },
        }
    }

    fn update(&mut self, _base: &mut Base, delta: f32) {
        self.update_animation(delta);
    }
}
