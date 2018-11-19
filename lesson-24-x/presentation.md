# What the GC languages can not do

For a developer who uses garbage collected languages like C#, Java, Javascript,
Go, Python, the value proposition of Rust might seem dubious.

Rust prevents segfaults, GC languages do too.

Rust is fast. So what? Go is fast, Javascript is fast, C# is fast.

And then, we hear that Rust does not have a GC, instead, it has a
strict rules we have to learn and obey.

At this point Rust must offer something quite special to be worth the bargain.

We are going to skip few of those things. We won't talk about zero-cost
abstractions. We won't talk about awesome type system or concurrency
without data races.

Instead, I will talk how Rust offers precise control over memory, and allows
us to create convenient abstractions that are both fast and easy to use.

But even if Rust advertises that it can do it, new Rust users run into
some problems while trying to do convenient things.

So, what kind of problems?

## The stack problem

The main problem with ownership and borrowing is that this system is tied to
the stack.

For example, we may design a text renderer,
that gives us back created text buffers:

```rust
let mut renderer = TextRenderer::new();
let buffer: &mut Buffer = renderer.create_buffer("Hello");
```

But of course, this buffer has a convenient method to change text position or rotation:

```rust
let mut renderer = TextRenderer::new();
let buffer: &mut Buffer = renderer.create_buffer("Hello");

buffer.translate(1.0, 3.3);
buffer.rotate(90.0);
```

Why do we use a renderer? Well, we want to efficiently render all the
text in a single draw call:

```rust
let mut renderer = TextRenderer::new();
let buffer: &mut Buffer = renderer.create_buffer("Hello");

buffer.translate(1.0, 3.3);
buffer.rotate(90.0);

renderer.render(&open_gl); // cannot borrow `renderer` as mutable more than once at a time
```

Oh wait, we have to finish borrowing to use renderer again:

```rust
let mut renderer = TextRenderer::new();

{
    let buffer: &mut Buffer = renderer.create_buffer("Hello");

    buffer.translate(1.0, 3.3);
    buffer.rotate(90.0);
}

renderer.render(&open_gl); // all good!
```

This kind of workaround works, but it ties our solution to the borrow checker
requirements. This is not what we wanted! We wanted to keep and pass buffer to
any other structure, and keep it around so that we can update the translation and
rotation from anywhere, anytime!

## Handle escape

This would not happen if buffer was not a reference, but a a handle:

```rust
let mut renderer = TextRenderer::new();
let buffer_handle: u32 = renderer.create_buffer("Hello");
```

In the simplest case, the handle can be a possition in a vector or a key
in hashmap. The `slab` can be used to generated this kind of handles. The plain 
u32 handles can also be replaced with generational indices with the help of `slotmap`
crate.

With that, we can pass this handle anywhere! Whenever we want to do anything with it,
we just do it over the renderer:

```rust
renderer.translate(buffer_handle, 1.0, 3.3);
renderer.rotate(buffer_handle, 90.0);
```

This works, and this is quite performant solution. 

But if we are willing to trade a little bit performance for convenience,
there is a third solution.

## Enter the hidden world

Enter the hidden-world pattern.

We take this renderer that works with handles, and rename it to `SharedTextRenderer`:

```rust
let mut renderer = SharedTextRenderer::new();
let buffer_handle: u32 = renderer.create_buffer("Hello");

renderer.translate(buffer_handle, 1.0, 3.3);
renderer.rotate(buffer_handle, 90.0);
```

We create a wrapper around it, and name it `TextRenderer`. Inside, it has
a hidden shared mutable world, where the `SharedTextRenderer` actually lives:

```rust
use std::rc::Rc;
use std::cell::RefCell;

struct TextRenderer {
    shared: Rc<RefCell<SharedTextRenderer>>,
}
```

We wrap the buffer in the same exact way, except it additionally stores the
handle:

```rust
struct Buffer {
    handle: u32,
    shared: Rc<RefCell<SharedTextRenderer>>,
}
```

Then we can forward buffer creation into the inner world:

```rust
impl TextRenderer {
    pub fn create_buffer(&self, text: &str) -> Buffer {
        let handle = self.shared.borrow_mut().create_buffer(text);

        Buffer {
            handle,
            shared: self.shared.clone(),
        }
    }
}
```

Inside the buffer we can forward methods to change position or rotation into
the inner world too:

```rust
impl Buffer {
    pub fn rotate(&self, angle: f32) {
        self.shared.borrow_mut().rotate(self.handle, angle);
    }

    pub fn translate(&self, x: f32, y: f32) {
        self.shared.borrow_mut().translate(self.handle, x, y);
    }
}
```

Crucially, we gain one more additional thing: we can automatically remove the
buffer when it is no longer in use - we can do that in drop:

```rust
impl Drop for Buffer {
    fn drop(&mut self) {
        self.shared.borrow_mut().remove_buffer(self.handle);
    }
}
```

In the end, we can have the code like this:

```rust
let renderer = TextRenderer::new();
let buffer: Buffer = renderer.create_buffer("Hello");

buffer.translate(1.0, 3.3);
buffer.rotate(90.0);

renderer.render(&open_gl);
```

And we can store the buffer anywhere we like, because it always carries along
the hidden world.

## The possibilities

### Convenience

Inside of buffer, we are working with plain handles. So if there was say, buffer
hierarchy, we could traverse the tree in any way we like:

```rust
let items = buffer.parent().unwrap()
                  .parent().unwrap()
                  .children();
```

This solution required no unsafe code. However, if the performance is a concern,
the implementation could be replaced with something unsafe and more fine-tuned.

### Optimizations

We can optimize `TextRenderer` `render` as we need. For example, we may
have `invalidated` boolean flag that is set to `true` when any of the buffers
modifies the data. Then the render can come along and sweep all the changes in one go
and then upload them to the GPU, and then set the `invalidated` back to `false`.

We can also pick the most efficient storage possible for the hidden world data.

Let's say our data are font glyphs. We know there is a limited
number of glyphs, so we can use the hidden world as a shared arena where the objects are
added into list and never removed, unless the whole world is destroyed. In that
case we don't implement `drop` for the items that wrap the handle.

## Existing implementations

Look no further than a `std::mpsc::channel`!

```rust
use std::sync::mpsc::channel;
use std::thread;

let (sender, receiver) = channel();
```

Here we create two different objects that somehow know about each other's state.
How? Well, they share the hidden world. The implementation in standard library is unsafe,
however it is perfectly possible to write slower bus safe implementation of channels.

## What the garbage collector could not do

The name of the game is control. We can fine tune the properties however we want.

The downside of it all is that it requires a bit of boilerplate. However, new crates
start appearing that implements variants of this pattern. One example that I know is
the `lifeguard` crate.