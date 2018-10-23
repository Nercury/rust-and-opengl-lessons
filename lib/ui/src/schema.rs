use {Dim, Size, Color};

pub struct Root {
    pub size: Size,
    pub container: Option<Box<Container>>,
}

impl Root {
    pub fn new(size: Size) -> Root {
        Root {
            size,
            container: None,
        }
    }

    pub fn with_container(mut self, container: Container) -> Root {
        self.container = Some(Box::new(container));
        self
    }
}

pub enum Container {
    PaneLeft(PaneLeft),
}

pub struct PaneLeft {
    pub bg_color: Option<Color>,
    pub offset: Dim,
    pub container: Option<Box<Container>>,
}

impl PaneLeft {
    pub fn new(offset: Dim) -> PaneLeft {
        PaneLeft {
            bg_color: None,
            offset,
            container: None,
        }
    }

    pub fn with_container(mut self, container: Container) -> PaneLeft {
        self.container = Some(Box::new(container));
        self
    }

    pub fn with_bg_color(mut self, color: Color) -> PaneLeft {
        self.bg_color = Some(color);
        self
    }
}
