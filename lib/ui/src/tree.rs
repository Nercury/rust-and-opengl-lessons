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
        pub fn enable_update(&mut self, state: bool) {
            let skeleton = self.container.nodes.get_mut(&self.id).expect("enable_update: self.container.nodes.get_mut(&self.id)");
            skeleton.updated_enabled = state;
        }

        pub fn add<E: Element + 'static>(&mut self, element: E) -> Ix {
            let id = self.container.add_node(self.id, Box::new(element) as Box<Element>);
            self.children.items.push(Child::new(id));
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

            if children_len == 0 {
                return self.layout_empty();
            }

            match size {
                ElementSize::Auto => { None }
                ElementSize::Fixed { w, h } => {
                    let w_without_margin = w - margin * 2;
                    let h_without_margin = h - margin * 2 - margin * (children_len as i32 - 1);

                    if w_without_margin <= 0 || h_without_margin <= 0 { return self.layout_empty(); }

                    let child_h = h_without_margin / children_len as i32;
                    if child_h == 0 { return self.layout_empty(); }

                    let mut next_child_offset_y = margin;
                    let mut remaining_h = h_without_margin;

                    self.children_mut(|i, mut child| {
                        let set_w = w_without_margin;
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
                        let offset_x = margin;

                        let asked_size = ElementSize::Fixed { w: set_w, h: set_h };
                        let _actual_size = child.element_resize(asked_size); // layout ignores actual size even if it is clipping

                        child.set_translation(offset_x, offset_y);

                        next_child_offset_y += set_h + margin;
                    });

                    Some(ResolvedSize { w, h })
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

        pub fn set_translation(&mut self, x: i32, y: i32) {
            if (x, y) != self.child.translation2d || !self.child.transform_propagated {
                self.child.translation2d = (x, y);
                self.propagate_transform();
            }
        }

        fn propagate_transform(&mut self) {
            let transform = na::Translation3::<f32>::new(self.child.translation2d.0 as f32, self.child.translation2d.1 as f32, 0.0);
            self.container.transform(self.child.id, &na::convert(transform));
            self.child.transform_propagated = true;
        }

        pub fn hide(&mut self) {
            self.container.hide(self.child.id);
        }
    }

    pub enum PivotPoint {
        Fractional { x: f32, y: f32 },
        Fixed { x: i32, y: i32 },
    }

    pub struct Child {
        id: Ix,
        translation2d: (i32, i32),
        rotation2d: f32,
        pivot2d: PivotPoint,
        transform_propagated: bool,
    }

    impl Child {
        pub fn new(id: Ix) -> Child {
            Child {
                id,
                translation2d: (0, 0),
                rotation2d: 0.0,
                pivot2d: PivotPoint::Fixed { x: 0, y: 0 },
                transform_propagated: false,
            }
        }
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

        fn mutate<
            IA,
            I,
            O,
            OA,
            InputFunT,
            MutFunT,
            OutputFunT
        >(
            &mut self,
            id: Ix,
            input_arg: IA,
            mut input_fun: InputFunT, // input_arg comes in, I comes out (access to NodeSkeleton and Queues)
            mut mut_fun: MutFunT, // I comes in, O comes out (access to Container and Body)
            mut output_fun: OutputFunT, // O comes in, OA is returned (access to NodeSkeleton and Queues)
        ) -> OA
            where
                InputFunT: FnMut(&mut NodeSkeleton, &mut Queues, IA) -> I,
                MutFunT: FnMut(&mut NodeBody, &mut Container, I) -> O,
                OutputFunT: FnMut(&mut NodeSkeleton, &mut Queues, O) -> OA
        {
            let (mut body, input) = {
                let skeleton = self.nodes.get_mut(&id).expect("mutate 1: self.nodes.get_mut(&id)");
                let input = input_fun(skeleton, &mut self.queues, input_arg);
                let body = skeleton.steal_body();
                (body, input)
            };

            let output = mut_fun(&mut body, self, input);

            let skeleton = self.nodes.get_mut(&id).expect("mutate 2: self.nodes.get_mut(&id)");
            skeleton.restore_body(body);

            output_fun(skeleton, &mut self.queues, output)
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

        pub fn new_root(&mut self, element: Box<Element>) -> Ix {
            let root_id = self.next_id.inc();

            self.queues.send(Effect::Add { id: root_id, parent_id: None });

            let mut skeleton = NodeSkeleton::new(Children::empty(), &na::Projective3::identity(), element);
            let mut body = skeleton.steal_body();

            self.nodes.clear();
            self.nodes.insert(root_id, skeleton);
            self._root_id = Some(root_id);

            body.el.inflate(Base { id: root_id, container: self, children: &mut body.children });

            let skeleton = self.nodes.get_mut(&root_id).expect("new_root: self.nodes.get_mut(&root_id)");
            skeleton.restore_body(body);

            self.queues.send(Effect::Transform { id: root_id, absolute_transform: na::Projective3::identity() });

            root_id
        }

        pub fn add_node(&mut self, parent_id: Ix, element: Box<Element>) -> Ix {
            let id = self.next_id.inc();

            self.queues.send(Effect::Add { id, parent_id: Some(parent_id) });

            let parent_absolute_transform = self.nodes.get(&parent_id).expect("add_node 1: self.nodes.get(&parent_id)").absolute_transform();
            let mut skeleton = NodeSkeleton::new(Children::empty(), &parent_absolute_transform, element);
            let mut body = skeleton.steal_body();

            self.nodes.insert(id, skeleton);

            body.el.inflate(Base { id, container: self, children: &mut body.children });

            let skeleton = self.nodes.get_mut(&id).expect("add_node 2: self.nodes.get_mut(&id)");
            skeleton.restore_body(body);

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
            self.mutate(
                id,
                size,
                |_skeleton, _q, size| size,
                |body, container, size| {
                    body.resize(id, container, size)
                },
                |skeleton, q, resolved_size| {
                    if resolved_size != skeleton.last_queue_size {
                        q.send(Effect::Resize { id, size: resolved_size.map(|s| (s.w, s.h)) });
                        skeleton.last_queue_size = resolved_size;
                    }
                    resolved_size
                },
            )
        }

        pub fn transform(&mut self, id: Ix, relative_transform: &na::Projective3<f32>) {
            self.mutate(
                id,
                relative_transform,
                |skeleton, _q, relative_transform| {
                    skeleton.relative_transform = relative_transform.clone();
                    skeleton.absolute_transform()
                },
                |body, container, absolute_transform| {
                    for child in &body.children.items {
                        container.parent_transform(child.id, &absolute_transform);
                    }
                    absolute_transform
                },
                |_skeleton, q, absolute_transform| {
                    q.send(Effect::Transform { id, absolute_transform });
                }
            )
        }

        pub fn parent_transform(&mut self, id: Ix, parent_transform: &na::Projective3<f32>) {
            self.mutate(
                id,
                parent_transform,
                |skeleton, _q, parent_transform| {
                    skeleton.parent_transform = parent_transform.clone();
                    skeleton.absolute_transform()
                },
                |body, container, absolute_transform| {
                    for child in &body.children.items {
                        container.parent_transform(child.id, &absolute_transform);
                    }
                    absolute_transform
                },
                |_skeleton, q, absolute_transform| {
                    q.send(Effect::Transform { id, absolute_transform });
                }
            )
        }

        pub fn hide(&mut self, id: Ix) {
            self.mutate(
                id,
                (),
                |skeleton, q, _| {
                    let resolved_size = None;

                    if resolved_size != skeleton.last_queue_size {
                        q.send(Effect::Resize { id, size: resolved_size.map(|s| (s.w, s.h)) });
                        skeleton.last_queue_size = resolved_size;

                        true
                    } else {
                        false
                    }
                },
                |body, container, hide_children| {
                    for child in body.children.items.iter() {
                        container.hide(child.id);
                    }
                },
                |_skeleton, _q, param| {
                }
            )
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
        parent_transform: na::Projective3<f32>,
        relative_transform: na::Projective3<f32>,
        body: Option<NodeBody>,
        updated_enabled: bool,
    }

    impl NodeSkeleton {
        pub fn new(children: Children, parent_transform: &na::Projective3<f32>, element: Box<Element>) -> NodeSkeleton {
            NodeSkeleton {
                last_queue_size: None,
                parent_transform: parent_transform.clone(),
                relative_transform: na::Projective3::identity(),
                body: Some(NodeBody {
                    children,
                    el: element,
                }),
                updated_enabled: false,
            }
        }

        pub fn absolute_transform(&self) -> na::Projective3<f32> {
            &self.parent_transform * &self.relative_transform
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