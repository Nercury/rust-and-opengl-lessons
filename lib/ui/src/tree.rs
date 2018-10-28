use std::marker::PhantomData;
use ::*;
use std::rc::Rc;
use std::cell::RefCell;

mod shared {
    use ::*;
    use std::collections::BTreeMap;

    pub struct Container {
        next_id: Ix,
        _root_id: Ix,
        nodes: BTreeMap<Ix, Node>,
    }

    impl Container {
        pub fn new_fill() -> Container {
            let root_id = Ix(0);
            let mut nodes = BTreeMap::new();

            nodes.insert(root_id, Node::Fill(
                NodeFill::new()
            ));

            Container {
                next_id: Ix(1),
                _root_id: root_id,
                nodes
            }
        }

        pub fn root_id(&self) -> Ix {
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

            calculate_and_apply_size(id, decision, &mut self.nodes)
        }
    }

    fn calculate_and_apply_size(id: Ix, resize_decision: ResizeDecision, nodes: &mut BTreeMap<Ix, Node>) -> Option<ResolvedSize> {
        match resize_decision {
            ResizeDecision::AutoFromChildrenVertical { stolen_children } => {
                let mut size = None;

                let resize_decisions = stolen_children.iter()
                    .map(|id| (*id, nodes.get_mut(id).map(|node| node.resize_decision(ElementSize::Auto))))
                    .collect::<Vec<_>>();

                for item in resize_decisions {
                    match item {
                        (child_id, Some(resize_decision)) => {
                            if let Some(resolved_child_size) = calculate_and_apply_size(child_id, resize_decision, nodes) {
                                size = match size {
                                    None => Some(resolved_child_size),
                                    Some(size) => Some(ResolvedSize { w: size.w, h: size.h + resolved_child_size.h }),
                                }
                            }
                        },
                        (child_id, None) => unreachable!("resolve_size: child {:?} does not exist for parent {:?}", child_id, id),
                    }
                }

                nodes.get_mut(&id).expect("expected item with missing children")
                    .swap_children(stolen_children);

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
    }
}

pub struct Fill {
    id: Ix,
    shared: Rc<RefCell<shared::Container>>,
}

impl Fill {
    pub fn root() -> Fill {
        let shared = Rc::new(RefCell::new(shared::Container::new_fill()));
        let root_id = shared.borrow().root_id();

        Fill {
            id: root_id,
            shared,
        }
    }

    pub fn resize(&mut self, size: ElementSize) -> Option<ResolvedSize> {
        self.shared.borrow_mut().resize(self.id, size)
    }

    pub fn add<T: Element>(&mut self, _element: T) -> Leaf<T> {
        Leaf::new()
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