use std::marker::PhantomData;
use ::*;
use std::rc::Rc;
use std::cell::RefCell;
pub use self::shared::Children;

mod shared {
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

    pub struct Children<'a> {
        nodes: &'a mut BTreeMap<Ix, NodeSkeleton>,
        ids: &'a mut Vec<Ix>,
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

                for child in body.children {
                    self.delete_node(child);
                }

                self.queues.send(Effect::Remove { id })
            }
        }

        pub fn new_root(&mut self, element: Box<Element>) -> Ix {
            let root_id = self.next_id;

            self.next_id.inc();

            self.nodes.clear();
            self.nodes.insert(root_id, NodeSkeleton::new(element));

            self._root_id = Some(root_id);

            self.queues.send(Effect::Add { id: root_id, size: None });

            root_id
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
            let mut body = self.nodes.get_mut(&id)?.steal_body();
            let resolved_size = body.resize(&mut self.nodes, size);
            self.nodes.get_mut(&id)?.restore_body(body);
            resolved_size
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
        children: Vec<Ix>,
        el: Box<Element>,
    }

    impl NodeBody {
        pub fn children_ids(&self) -> impl Iterator<Item = &Ix> {
            self.children.iter()
        }

        pub fn resize(&mut self, nodes: &mut BTreeMap<Ix, NodeSkeleton>, size: ElementSize) -> Option<ResolvedSize> {
            let mut children = ::std::mem::replace(&mut self.children, Vec::with_capacity(0));
            let resolved_size = self.el.resize(size, Children { nodes, ids: &mut children });
            ::std::mem::replace(&mut self.children, children);
            resolved_size
        }
    }

    pub struct NodeSkeleton {
        body: Option<NodeBody>,
    }

    impl NodeSkeleton {
        pub fn new(element: Box<Element>) -> NodeSkeleton {
            NodeSkeleton {
                body: Some(NodeBody {
                    children: Vec::new(),
                    el: element,
                })
            }
        }

        pub fn element_mut(&mut self) -> &mut Element {
            self.body.as_mut().map(|b| &mut *b.el).expect("element_mut: encountered stolen value")
        }

        pub fn steal_body(&mut self) -> NodeBody {
            self.body.take().expect("steal_body: encountered stolen value")
        }

        pub fn restore_body(&mut self, body: NodeBody) {
            if let Some(_) = ::std::mem::replace(&mut self.body, Some(body)) {
                unreachable!("restore_body: encountered existing value")
            }
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