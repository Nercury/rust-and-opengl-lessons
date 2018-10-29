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

        pub fn new_root_fill(&mut self) -> Ix {
            let root_id = self.next_id;

            self.next_id.inc();

            self.nodes.clear();
            self.nodes.insert(root_id, Node::Fill(
                NodeFill::new()
            ));

            self._root_id = Some(root_id);

            self.queues.send(Effect::Add { id: root_id, size: None });

            root_id
        }

        pub fn root_id(&self) -> Option<Ix> {
            self._root_id
        }

        pub fn get_node_fill_mut(&mut self, id: Ix) -> Option<&mut NodeFill> {
            self.nodes
                .get_mut(&id)
                .and_then(|node| match node {
                    Node::Fill(fill) => Some(fill),
                })
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
    }

    fn calculate_and_apply_size(id: Ix, resize_decision: ResizeDecision, nodes: &mut BTreeMap<Ix, Node>, queues: &mut Queues) -> Option<ResolvedSize> {
        match resize_decision {
            ResizeDecision::AutoFromChildrenVertical { stolen_children } => {
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
                        (child_id, None) => unreachable!("resolve_size: child {:?} does not exist for parent {:?}", child_id, id),
                    }
                }

                if let Some(node) = nodes.get_mut(&id) {
                    node.swap_children(stolen_children);

                    if let Some(size) = size {
                        node.apply_resize(&size);
                    }

                    queues.send(Effect::Resize { id, size: size.map(|v| (v.w, v.h)) })
                }

                size
            }
        }
    }

    pub enum ResizeDecision {
        AutoFromChildrenVertical { stolen_children: Vec<Ix> },
    }

    pub enum Node {
        Fill(NodeFill),
    }

    impl Node {
        pub fn resize_decision(&mut self, size: ElementSize) -> ResizeDecision {
            match self {
                Node::Fill(fill) => fill.resize_decision(size),
            }
        }

        pub fn apply_resize(&mut self, size: &ResolvedSize) {
            match self {
                Node::Fill(fill) => fill.apply_resize(size),
            }
        }

        pub fn swap_children(&mut self, new: Vec<Ix>) -> Vec<Ix> {
            match self {
                Node::Fill(fill) => fill.swap_children(new),
            }
        }
    }

    pub struct NodeFill {
        fixed_size: Option<(i32, i32)>,
        children: Vec<Ix>,
    }

    impl NodeFill {
        pub fn new() -> NodeFill {
            NodeFill {
                fixed_size: None,
                children: Vec::new(),
            }
        }

        pub fn swap_children(&mut self, new: Vec<Ix>) -> Vec<Ix> {
            ::std::mem::replace(&mut self.children, new)
        }

        pub fn resize_decision(&mut self, size: ElementSize) -> ResizeDecision {
            match size {
                ElementSize::Auto => ResizeDecision::AutoFromChildrenVertical { stolen_children: self.swap_children(vec![]) },
                _ => unimplemented!("handle other resize_decision cases")
            }
        }

        pub fn apply_resize(&mut self, size: &ResolvedSize) {

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

    pub fn create_root_fill(&self) -> Fill {
        Fill {
            id: self.shared.borrow_mut().new_root_fill(),
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
}

impl Drop for Events {
    fn drop(&mut self) {
        self.shared.borrow_mut().delete_queue(self.queue_id);
    }
}

pub struct Fill {
    id: Ix,
    shared: Rc<RefCell<shared::Container>>,
}

impl Fill {
    pub fn resize(&self, size: ElementSize) -> Option<ResolvedSize> {
        self.shared.borrow_mut().resize(self.id, size)
    }

    pub fn add<T: Element>(&self, _element: T) -> Leaf<T> {
        Leaf::new()
    }
}

impl Drop for Fill {
    fn drop(&mut self) {
        self.shared.borrow_mut().delete_node(self.id);
    }
}

pub struct Leaf<T> {
    _marker: PhantomData<T>,
}

impl<T> Leaf<T> {
    pub fn new() -> Leaf<T> {
        Leaf {
            _marker: PhantomData,
        }
    }
}