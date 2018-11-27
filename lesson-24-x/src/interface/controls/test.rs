use ui::*;
use na;
use super::presentation::*;

pub struct TestPresentation {
    svg: Option<primitives::Svg>,
}

impl TestPresentation {
    pub fn new() -> TestPresentation {
        TestPresentation {
            svg: None
        }
    }
}

impl Element for TestPresentation {
    fn inflate(&mut self, base: &mut Base) {
        if let Ok(svg) = base.primitives().svg("test.svg") {
            self.svg = Some(svg);
        }
    }

    fn update(&mut self, _base: &mut Base, _delta: f32) {
    }
}

