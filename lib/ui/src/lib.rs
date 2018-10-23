pub mod schema;

pub mod mutator {
    use schema;
    use {Dim, Size, Color};
    use std::collections::BTreeMap;

    #[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd)]
    struct Ix(u32);

    impl Ix {
        fn inc(&mut self) -> Ix {
            let next_id = *self;
            self.0 += 1;
            next_id
        }
    }

    #[derive(Debug)]
    pub struct Mutator {
        next_id: Ix,
        root_id: Option<Ix>,
        nodes: BTreeMap<Ix, Node>,
    }

    impl Mutator {
        pub fn from_schema(schema: &schema::Root) -> Mutator {
            let mut items = Mutator {
                next_id: Ix(0),
                root_id: None,
                nodes: BTreeMap::new(),
            };

            items.replace_root(schema);

            items
        }

        /// Replaces root with a new schema, returns old root schema
        fn replace_root(&mut self, root: &schema::Root) -> Option<schema::Root> {
            let id = self.next_id.inc();

            let previous_root_schema = match self.root_id {
                Some(previous_root_id) => self.remove_root(previous_root_id),
                None => None,
            };

            let container_id = root.container
                .as_ref()
                .map(|c| self.insert_container(&**c));

            self.nodes.insert(id, Node::Root(RootNode {
                size: root.size,
                container: container_id,
            }));

            previous_root_schema
        }

        fn remove_root(&mut self, _id: Ix) -> Option<schema::Root> {
            None
        }

        fn insert_container(&mut self, c: &schema::Container) -> Ix {
            let id = self.next_id.inc();

            match c {
                schema::Container::PaneLeft(data) => {
                    self.nodes.insert(id, Node::ContainerPaneLeft(ContainerPaneLeft {
                        bg_color: data.bg_color,
                        offset: data.offset,
                        container: None,
                    }));
                }
            }

            id
        }
    }

    #[derive(Debug)]
    pub enum Node {
        Root(RootNode),
        ContainerPaneLeft(ContainerPaneLeft),
    }

    #[derive(Debug)]
    pub struct RootNode {
        size: Size,
        container: Option<Ix>,
    }

    #[derive(Debug)]
    pub struct ContainerPaneLeft {
        bg_color: Option<Color>,
        offset: Dim,
        container: Option<Ix>,
    }
}

#[derive(Debug, Copy, Clone)]
pub struct Pos {
    x: Dim,
    y: Dim,
}

impl Pos {
    pub fn new(x: Dim, y: Dim) -> Pos {
        Pos {
            x,
            y,
        }
    }
}

#[derive(Debug, Copy, Clone)]
pub struct Color {
    r: f32,
    g: f32,
    b: f32,
}

impl Color {
    pub fn new(r: f32, g: f32, b: f32) -> Color {
        Color { r, g, b }
    }
}

#[derive(Debug, Copy, Clone)]
pub struct Size {
    w: Dim,
    h: Dim,
}

impl Size {
    pub fn new(w: Dim, h: Dim) -> Size {
        Size {
            w,
            h,
        }
    }
}

#[derive(Debug, Copy, Clone)]
pub struct Dim {
    uu: f32,
}

impl From<f32> for Dim {
    fn from(other: f32) -> Self {
        Dim { uu: other }
    }
}

impl Dim {
    pub fn new(uu: f32) -> Dim {
        Dim {
            uu
        }
    }

    pub fn uu(&self) -> f32 {
        self.uu
    }
}

