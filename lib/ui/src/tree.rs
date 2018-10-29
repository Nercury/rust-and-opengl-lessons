use std::marker::PhantomData;
use ::*;
use std::rc::Rc;
use std::cell::RefCell;

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

    pub struct Container {
        queues: Queues,

        next_id: Ix,
        _root_id: Option<Ix>,
        nodes: BTreeMap<Ix, Node>,
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
                let children = removed.swap_children(vec![]);

                for child in children {
                    self.delete_node(child);
                }

                self.queues.send(Effect::Remove { id })
            }
        }

        pub fn new_root(&mut self, element: Box<Element>) -> Ix {
            let root_id = self.next_id;

            self.next_id.inc();

            self.nodes.clear();
            self.nodes.insert(root_id, Node::new(element));

            self._root_id = Some(root_id);

            self.queues.send(Effect::Add { id: root_id, size: None });

            root_id
        }

        pub fn root_id(&self) -> Option<Ix> {
            self._root_id
        }

        pub fn get_node_fill_mut(&mut self, id: Ix) -> Option<&mut (Element + 'static)> {
            self.nodes
                .get_mut(&id)
                .map(|node| &mut *node.element)
        }

        pub fn resize(&mut self, id: Ix, size: ElementSize) -> Option<ResolvedSize> {
            let decision = match self.nodes.get_mut(&id) {
                None => unreachable!("resolve_size: node {:?} missing", id),
                Some(node) => node.resize_decision(size),
            };

            calculate_and_apply_size(id, decision, &mut self.nodes, &mut self.queues)
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

    fn calculate_and_apply_size(id: Ix, resize_decision: ResizeDecision, nodes: &mut BTreeMap<Ix, Node>, queues: &mut Queues) -> Option<ResolvedSize> {
        match resize_decision {
            ResizeDecision::AutoFromChildrenVertical => {
                let stolen_children = if let Some(node) = nodes.get_mut(&id) {
                    node.swap_children(Vec::with_capacity(0))
                } else {
                    unreachable!("calculate_and_apply_size: node {:?} not found", id);
                };

                let mut size = None;

                let resize_decisions = stolen_children.iter()
                    .map(|id| (*id, nodes.get_mut(id).map(|node| node.resize_decision(ElementSize::Auto))))
                    .collect::<Vec<_>>();

                for item in resize_decisions {
                    match item {
                        (child_id, Some(resize_decision)) => {
                            if let Some(resolved_child_size) = calculate_and_apply_size(child_id, resize_decision, nodes, queues) {
                                size = match size {
                                    None => Some(resolved_child_size),
                                    Some(size) => Some(ResolvedSize { w: size.w, h: size.h + resolved_child_size.h }),
                                }
                            }
                        },
                        (child_id, None) => unreachable!("calculate_and_apply_size: child {:?} does not exist for parent {:?}", child_id, id),
                    }
                }

                if let Some(node) = nodes.get_mut(&id) {
                    node.swap_children(stolen_children);
                    queues.send(Effect::Resize { id, size: size.map(|v| (v.w, v.h)) })
                }

                size
            }
        }
    }

    pub struct Node {
        children: Vec<Ix>,
        element: Box<Element>,
    }

    impl Node {
        pub fn new(element: Box<Element>) -> Node {
            Node {
                children: Vec::new(),
                element,
            }
        }

        pub fn resize_decision(&mut self, size: ElementSize) -> ResizeDecision {
            self.element.resize_decision(size)
        }

        pub fn swap_children(&mut self, new: Vec<Ix>) -> Vec<Ix> {
            ::std::mem::replace(&mut self.children, new)
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