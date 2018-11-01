use std::marker::PhantomData;
use ::*;
use std::rc::Rc;
use std::cell::RefCell;
pub use self::shared::Base;

mod shared {
    use na;
    use ::*;
    use std::collections::BTreeMap;
    use std::collections::VecDeque;

    struct Queues {
        next_queue_id: Ix,
        queues: BTreeMap<Ix, VecDeque<Effect>>,
    }

    impl Queues {
        pub fn new() -> Queues {
            Queues {
                next_queue_id: Ix(0),
                queues: BTreeMap::new(),
            }
        }

        pub fn create_queue(&mut self) -> Ix {
            self.queues.insert(self.next_queue_id, VecDeque::new());
            let id = self.next_queue_id;
            self.next_queue_id.inc();
            id
        }

        pub fn delete_queue(&mut self, id: Ix) {
            self.queues.remove(&id);
        }

        fn send(&mut self, e: Effect) {
            println!("event: {:?}", e);

            for (_, q) in self.queues.iter_mut() {
                q.push_back(e);
            }
        }

        pub fn get_queue_mut(&mut self, id: Ix) -> Option<&mut VecDeque<Effect>> {
            self.queues
                .get_mut(&id)
        }
    }

    struct LayoutingOptions {
        force_equal_child_size: bool,
    }

    impl Default for LayoutingOptions {
        fn default() -> Self {
            LayoutingOptions {
                force_equal_child_size: false,
            }
        }
    }

    pub struct Base<'a> {
        id: Ix,
        container: &'a mut Container,
        children: &'a mut Children,
    }

    impl<'a> Base<'a> {
        pub fn add<E: Element + 'static>(&mut self, element: E) -> Ix {
            let id = self.container.add_node(self.id, Box::new(element) as Box<Element>);
            self.children.items.push(Child { id, transform: None });
            id
        }

        pub fn layout_empty(&mut self) -> Option<ResolvedSize> {
            self.children_mut(|_, mut child| {
                child.hide();
            });

            None
        }

        pub fn layout_vertical(&mut self, size: ElementSize, margin: i32) -> Option<ResolvedSize> {
            let options = LayoutingOptions::default();
            let children_len = self.children_len();

            println!("children len = {}", children_len);

            if children_len == 0 {
                return self.layout_empty();
            }

            match size {
                ElementSize::Auto => { None }
                ElementSize::Fixed { w, h } => {
                    let w = w - margin * 2;
                    let h = h - margin * 2 - margin * (children_len as i32 - 1);

                    if w <= 0 || h <= 0 { return self.layout_empty(); }

                    let child_h = h / children_len as i32;
                    if child_h == 0 { return self.layout_empty(); }

                    let mut next_child_offset_y = 0;
                    let mut remaining_h = h;

                    let transform = na::Affine3::<f32>::identity();

                    self.children_mut(|i, mut child| {
                        let set_w = w;
                        let set_h = if options.force_equal_child_size {
                            child_h
                        } else {
                            if i < children_len {
                                remaining_h -= child_h;
                                child_h
                            } else {
                                remaining_h
                            }
                        };

                        let offset_y = next_child_offset_y;
                        let offset_x = 0;

                        println!("child set w = {}, h = {}, x = {}, y = {}", set_w, set_h, offset_x, offset_y);

                        let asked_size = ElementSize::Fixed { w: set_w, h: set_h };
                        let actual_size = child.element_resize(asked_size);


                        next_child_offset_y += set_h;
                    });

                    None
                }
            }
        }

        pub fn children_len(&self) -> usize {
            self.children.items.len()
        }

        pub fn children_mut<F>(&mut self, mut fun: F) where F: for<'r> FnMut(usize, ChildIterItemMut<'r>) {

            // this method uses internal iterator because I failed to make it external

            for (i, child) in self.children.items.iter_mut().enumerate() {
                fun(i, ChildIterItemMut { child, container: self.container });
            }
        }
    }

    pub struct ChildIterItemMut<'a> {
        child: &'a mut Child,
        container: &'a mut Container,
    }

    impl<'a> ChildIterItemMut<'a> {
        pub fn element_resize(&mut self, size: ElementSize) -> Option<ResolvedSize> {
            self.container.resize(self.child.id, size)
        }

        pub fn hide(&mut self) {
            self.container.hide(self.child.id);
        }
    }

    pub struct Child {
        id: Ix,
        transform: Option<na::Affine3<f32>>,
    }

    pub struct Children {
        items: Vec<Child>,
    }

    impl Children {
        pub fn empty() -> Children {
            Children {
                items: Vec::with_capacity(0),
            }
        }
    }

    pub struct Container {
        queues: Queues,

        next_id: Ix,
        _root_id: Option<Ix>,
        nodes: BTreeMap<Ix, NodeSkeleton>,
    }

    impl Container {
        pub fn new() -> Container {
            Container {
                queues: Queues::new(),

                next_id: Ix(0),
                _root_id: None,
                nodes: BTreeMap::new(),
            }
        }

        pub fn delete_node(&mut self, id: Ix) {
            if let Some(mut removed) = self.nodes.remove(&id) {
                let body = removed.steal_body();

                for child in body.children.items {
                    self.delete_node(child.id);
                }

                self.queues.send(Effect::Remove { id })
            }
        }

        pub fn new_root(&mut self, mut element: Box<Element>) -> Ix {
            let root_id = self.next_id.inc();

            self.nodes.clear();

            let mut children = Children::empty();
            element.inflate(Base { id: root_id, container: self, children: &mut children });

            self.nodes.insert(root_id, NodeSkeleton::new(children, element));

            self._root_id = Some(root_id);

            self.queues.send(Effect::Add { id: root_id, parent_id: None });

            root_id
        }

        pub fn add_node(&mut self, parent_id: Ix, mut element: Box<Element>) -> Ix {
            let id = self.next_id.inc();

            let mut children = Children::empty();
            element.inflate(Base { id, container: self, children: &mut children });

            self.nodes.insert(id, NodeSkeleton::new(children, element));

            self.queues.send(Effect::Add { id, parent_id: Some(parent_id) });

            id
        }

        pub fn root_id(&self) -> Option<Ix> {
            self._root_id
        }

        pub fn get_node_mut(&mut self, id: Ix) -> Option<&mut Element> {
            self.nodes
                .get_mut(&id)
                .map(|node| node.element_mut())
        }

        pub fn resize(&mut self, id: Ix, size: ElementSize) -> Option<ResolvedSize> {
            println!("resize id = {:?}, size = {:?}", id, size);

            let mut body = self.nodes.get_mut(&id)?.steal_body();
            let resolved_size = body.resize(id, self, size);

            let skeleton = self.nodes.get_mut(&id)?;
            skeleton.restore_body(body);
            if resolved_size != skeleton.last_queue_size {
                self.queues.send(Effect::Resize { id, size: resolved_size.map(|s| (s.w, s.h)) });
                skeleton.last_queue_size = resolved_size;
            }

            resolved_size
        }

        pub fn hide(&mut self, id: Ix) {
            println!("hide id = {:?}", id);

            let stolen_body = {
                let skeleton = self.nodes.get_mut(&id).expect("hide: self.nodes.get_mut(&id)");
                let resolved_size = None;

                if resolved_size != skeleton.last_queue_size {
                    self.queues.send(Effect::Resize { id, size: resolved_size.map(|s| (s.w, s.h)) });
                    skeleton.last_queue_size = resolved_size;

                    Some(skeleton.steal_body())
                } else { None }
            };

            if let Some(body) = stolen_body {
                for child in body.children.items.iter() {
                    self.hide(child.id);
                }

                let skeleton = self.nodes.get_mut(&id).expect("hide: self.nodes.get_mut(&id)");
                skeleton.restore_body(body);
            }
        }

        pub fn create_queue(&mut self) -> Ix {
            self.queues.create_queue()
        }

        pub fn delete_queue(&mut self, id: Ix) {
            self.queues.delete_queue(id);
        }

        pub fn get_queue_mut(&mut self, id: Ix) -> Option<&mut VecDeque<Effect>> {
            self.queues.get_queue_mut(id)
        }
    }

    pub struct NodeBody {
        children: Children,
        el: Box<Element>,
    }

    impl NodeBody {
        pub fn resize(&mut self, id: Ix, container: &mut Container, size: ElementSize) -> Option<ResolvedSize> {
            let mut children = ::std::mem::replace(&mut self.children, Children::empty());
            let resolved_size = self.el.resize(Base { id, container, children: &mut children }, size);
            ::std::mem::replace(&mut self.children, children);
            resolved_size
        }
    }

    pub struct NodeSkeleton {
        last_queue_size: Option<ResolvedSize>,
        body: Option<NodeBody>,
    }

    impl NodeSkeleton {
        pub fn new(children: Children, element: Box<Element>) -> NodeSkeleton {
            NodeSkeleton {
                last_queue_size: None,
                body: Some(NodeBody {
                    children,
                    el: element,
                }),
            }
        }

        pub fn steal_body(&mut self) -> NodeBody {
            self.body.take().expect("steal_body: encountered stolen value")
        }

        pub fn restore_body(&mut self, body: NodeBody) {
            if let Some(_) = ::std::mem::replace(&mut self.body, Some(body)) {
                unreachable!("restore_body: encountered existing value")
            }
        }

        pub fn element_mut(&mut self) -> &mut Element {
            self.body.as_mut().map(|b| &mut *b.el).expect("element_mut: encountered stolen value")
        }
    }
}

pub struct Tree {
    shared: Rc<RefCell<shared::Container>>,
}

impl Tree {
    pub fn new() -> Tree {
        let shared = Rc::new(RefCell::new(shared::Container::new()));

        Tree {
            shared,
        }
    }

    pub fn create_root<T: Element + 'static>(&self, element: T) -> Leaf<T> {
        Leaf {
            _marker: PhantomData,
            id: self.shared.borrow_mut().new_root(Box::new(element) as Box<Element>),
            shared: self.shared.clone(),
        }
    }

    pub fn events(&self) -> Events {
        Events::new(&self.shared)
    }
}

pub struct Events {
    queue_id: Ix,
    shared: Rc<RefCell<shared::Container>>,
}

impl Events {
    pub fn new(shared: &Rc<RefCell<shared::Container>>) -> Events {
        Events {
            queue_id: shared.borrow_mut().create_queue(),
            shared: shared.clone(),
        }
    }

    pub fn drain_into(&self, output: &mut Vec<Effect>) {
        let mut shared = self.shared.borrow_mut();
        if let Some(queue) = shared.get_queue_mut(self.queue_id) {
            output.extend(queue.drain(..))
        }
    }
}

impl Drop for Events {
    fn drop(&mut self) {
        self.shared.borrow_mut().delete_queue(self.queue_id);
    }
}

pub struct Leaf<T> {
    _marker: PhantomData<T>,
    id: Ix,
    shared: Rc<RefCell<shared::Container>>,
}

impl<T> Leaf<T> {
    pub fn resize(&self, size: ElementSize) -> Option<ResolvedSize> {
        self.shared.borrow_mut().resize(self.id, size)
    }
}

impl<T> Drop for Leaf<T> {
    fn drop(&mut self) {
        self.shared.borrow_mut().delete_node(self.id);
    }
}