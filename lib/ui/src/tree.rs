pub use self::shared::{Base, LastResolvedSize, ResizeFlow};
use std::cell::RefCell;
use std::marker::PhantomData;
use std::rc::Rc;
use *;

mod shared {
    use na;
    use std::collections::{BTreeMap, BTreeSet, VecDeque};
    use queues::*;
    use fonts::Fonts;
    use std::cell::RefCell;
    use std::rc::Rc;
    use *;

    struct LayoutingOptions {
        force_equal_child_size: bool,
    }

    impl Default for LayoutingOptions {
        fn default() -> Self {
            LayoutingOptions {
                force_equal_child_size: true,
            }
        }
    }

    #[derive(Copy, Clone, Debug)]
    pub enum LastResolvedSize {
        ElementSizeHidden,
        ElementSizeAuto(Option<ResolvedSize>),
        ElementSizeFixed {
            w: i32,
            h: i32,
            size: Option<ResolvedSize>,
        },
    }

    impl LastResolvedSize {
        pub fn to_box_size(&self) -> BoxSize {
            match *self {
                LastResolvedSize::ElementSizeHidden => BoxSize::Hidden,
                LastResolvedSize::ElementSizeAuto(_) => BoxSize::Auto,
                LastResolvedSize::ElementSizeFixed { w, h, .. } => BoxSize::Fixed { w, h },
            }
        }
    }

    pub struct Base<'a> {
        id: Ix,
        container: &'a mut Container,
        children: &'a mut Children,

        resize_flow: ResizeFlow,
        resize_flow_output: ResizeFlowOutput,
        _box_size: BoxSize,

        window_scale: f32,
    }

    impl<'a> Base<'a> {
        fn new<'x>(
            id: Ix,
            container: &'x mut Container,
            children: &'x mut Children,
            resize_flow: ResizeFlow,
            box_size: BoxSize,
            window_scale: f32
        ) -> Base<'x> {
            Base {
                id,
                container,
                children,

                _box_size: box_size,

                resize_flow,
                resize_flow_output: match resize_flow {
                    ResizeFlow::ParentIsResizing => ResizeFlowOutput::ParentIsResizingNoResolve,
                    ResizeFlow::ParentIsNotResizing => {
                        ResizeFlowOutput::ParentIsNotResizingNoSizeUpdate
                    }
                },

                window_scale,
            }
        }

        #[inline(always)]
        pub fn box_size(&self) -> BoxSize {
            self._box_size
        }

        #[inline(always)]
        pub fn scale(&self) -> f32 {
            self.window_scale
        }

        /// Forces a resize after any update action (excluding the resize).
        pub fn invalidate_size(&mut self) {
            match self.resize_flow {
                ResizeFlow::ParentIsResizing => (),
                ResizeFlow::ParentIsNotResizing => {
                    self.resize_flow_output = ResizeFlowOutput::ParentIsNotResizingSizeInvalidated
                }
            }
        }

        /// Marks the size of this element resolved, and the resize action as executed.
        pub fn resolve_size(&mut self, size: Option<ResolvedSize>) {
            match self.resize_flow {
                ResizeFlow::ParentIsResizing => match size {
                    Some(size) => {
                        self.resize_flow_output = ResizeFlowOutput::ParentIsResizingResolved(size)
                    }
                    None => {
                        self.resize_flow_output = ResizeFlowOutput::ParentIsResizingResolvedNone
                    }
                },
                ResizeFlow::ParentIsNotResizing => self.resize_flow_output = ResizeFlowOutput::ParentIsNotResizingSizeInvalidated,
            }
        }

        pub fn enable_update(&mut self, state: bool) {
            if let Some(ref mut set) = self.container
                .update_set
                .as_mut() {
                if state {
                    set.insert(self.id);
                } else {
                    set.remove(&self.id);
                }
            } else {
                self.container.update_set_actions.push_back(if state {
                    SetAction::Add(self.id)
                } else {
                    SetAction::Remove(self.id)
                })
            }
        }

        pub fn enable_actions(&mut self, state: bool) {
            if let Some(ref mut set) = self.container
                .action_set
                .as_mut() {
                if state {
                    set.insert(self.id);
                } else {
                    set.remove(&self.id);
                }
            } else {
                self.container.action_set_actions.push_back(if state {
                    SetAction::Add(self.id)
                } else {
                    SetAction::Remove(self.id)
                })
            }
        }

        #[inline(always)]
        pub fn add<E: Element + 'static>(&mut self, element: E) -> Ix {
            self.add_boxed(Box::new(element) as Box<Element>)
        }

        pub fn add_boxed(&mut self, element: Box<Element>) -> Ix {
            let id = self
                .container
                .add_node(self.id, element);
            self.children.items.insert(id, Child::new(id));
            self.invalidate_size();
            id
        }

        pub fn layout_empty(&mut self) {
            self.children_mut(|_, mut child| {
                child.hide();
            });

            self.resolve_size(None);
        }

        pub fn layout_auto_sized_list(&mut self, margin: i32, flow: FlowDirection) {
            let mut flow_forward = margin;
            let flow_side_offset = margin;

            let mut flow_width = None;

            self.children_mut(|_i, mut child| {
                let actual_size = child.element_resize(BoxSize::Auto);
                if let Some(size) = actual_size {
                    let (element_flow_w, element_flow_val) = size.to_flow(flow);

                    flow_width = match flow_width {
                        None => Some(element_flow_w),
                        Some(w) => if element_flow_w > w {
                            Some(element_flow_w)
                        } else {
                            Some(w)
                        },
                    };

                    child.set_translation(ResolvedSize::from_flow(flow, flow_side_offset, flow_forward));

                    flow_forward += element_flow_val + margin;
                }
            });

            if let Some(w) = flow_width {
                self.resolve_size(Some(ResolvedSize::from_flow(flow, w + margin * 2, flow_forward)));
            } else {
                self.resolve_size(None);
            }
        }

        pub fn layout_equally_sized_fill_list(&mut self, margin: i32, size: ResolvedSize, flow: FlowDirection) {
            let options = LayoutingOptions::default();
            let children_len = self.children_len();

            if children_len == 0 {
                return self.layout_empty();
            }

            let (w, h) = size.to_flow(flow);

            let w_without_margin = w - margin * 2;
            let h_without_margin = h - margin * 2 - margin * (children_len as i32 - 1);

            if w_without_margin <= 0 || h_without_margin <= 0 {
                return self.layout_empty();
            }

            let child_h = h_without_margin / children_len as i32;
            if child_h == 0 {
                return self.layout_empty();
            }

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

                let asked_size = ResolvedSize::from_flow(flow, set_w, set_h); ;
                let _actual_size = child.element_resize(BoxSize::Fixed { w: asked_size.w, h: asked_size.h }); // layout ignores actual size

                child.set_translation(ResolvedSize::from_flow(flow, offset_x, offset_y));

                next_child_offset_y += set_h + margin;
            });

            self.resolve_size(Some(ResolvedSize::from_flow(flow, w, h)));
        }

        pub fn layout_vertical(&mut self, margin: i32) {
            match self.box_size() {
                BoxSize::Hidden => self.layout_empty(),
                BoxSize::Auto => self.layout_auto_sized_list(margin, FlowDirection::Vertical),
                BoxSize::Fixed { w, h } => self.layout_equally_sized_fill_list(margin, ResolvedSize { w, h }, FlowDirection::Vertical),
            }
        }

        pub fn layout_horizontal(&mut self, margin: i32) {
            match self.box_size() {
                BoxSize::Hidden => self.layout_empty(),
                BoxSize::Auto => self.layout_auto_sized_list(margin, FlowDirection::Horizontal),
                BoxSize::Fixed { w, h } => self.layout_equally_sized_fill_list(margin, ResolvedSize { w, h }, FlowDirection::Horizontal),
            }
        }

        #[inline(always)]
        pub fn children_len(&self) -> usize {
            self.children.items.len()
        }

        pub fn children_mut<F>(&mut self, mut fun: F)
        where
            F: for<'r> FnMut(usize, ChildIterItemMut<'r>),
        {
            // this method uses internal iterator because I failed to make it external

            for (i, (_, child)) in self.children.items.iter_mut().enumerate() {
                fun(
                    i,
                    ChildIterItemMut {
                        child,
                        container: self.container,
                        window_scale: self.window_scale,
                    },
                );
            }
        }

        pub fn primitives(&mut self) -> &mut Primitives {
            if self.children.primitives.is_none() {
                let window_scale = self.window_scale;
                let primitives = Primitives::new(self.container.fonts(), window_scale);
                self.children.primitives = Some(primitives.clone());
            }

            self.children.primitives.as_mut().unwrap()
        }
    }

    #[derive(Copy, Clone, Debug)]
    pub enum ResizeFlow {
        ParentIsResizing,
        ParentIsNotResizing,
    }

    #[derive(Copy, Clone, Debug)]
    enum ResizeFlowOutput {
        ParentIsResizingNoResolve,
        ParentIsResizingResolvedNone,
        ParentIsResizingResolved(ResolvedSize),
        ParentIsNotResizingNoSizeUpdate,
        ParentIsNotResizingSizeInvalidated,
    }

    pub struct ChildIterItemMut<'a> {
        child: &'a mut Child,
        container: &'a mut Container,
        window_scale: f32,
    }

    impl<'a> ChildIterItemMut<'a> {
        pub fn element_resize(&mut self, size: BoxSize) -> Option<ResolvedSize> {
            self.container.resize(self.child.id, size, self.window_scale)
        }

        pub fn set_translation(&mut self, size: ResolvedSize) {
            if size != self.child.translation2d || !self.child.transform_propagated {
                self.child.translation2d = size;
                self.propagate_transform();
            }
        }

        pub fn element_transform(&mut self, transform: &na::Projective3<f32>) {
            self.child.transform = transform.clone();
            self.propagate_transform();
        }

        fn propagate_transform(&mut self) {
            let transform = na::Translation3::<f32>::new(
                self.child.translation2d.w as f32,
                self.child.translation2d.h as f32,
                0.0,
            );
            self.container
                .transform(self.child.id, &(self.child.transform * na::convert::<_, na::Projective3<_>>(transform)));
            self.child.transform_propagated = true;
        }

        #[inline(always)]
        pub fn hide(&mut self) {
            self.container.hide(self.child.id, self.window_scale);
        }
    }

    pub enum PivotPoint {
        Fractional { x: f32, y: f32 },
        Fixed { x: i32, y: i32 },
    }

    pub struct Child {
        id: Ix,
        translation2d: ResolvedSize,
        transform: na::Projective3<f32>,
        transform_propagated: bool,
    }

    impl Child {
        pub fn new(id: Ix) -> Child {
            Child {
                id,
                translation2d: ResolvedSize { w: 0, h: 0 },
                transform: na::Projective3::<f32>::identity(),
                transform_propagated: false,
            }
        }

        #[inline(always)]
        pub fn invalidate_transform_propagation(&mut self) {
            self.transform_propagated = false;
        }
    }

    pub struct Children {
        items: BTreeMap<Ix, Child>,
        primitives: Option<primitives::Primitives>,
    }

    impl Children {
        pub fn empty() -> Children {
            Children {
                items: BTreeMap::new(),
                primitives: None,
            }
        }

        #[inline(always)]
        pub fn remove(&mut self, id: Ix) {
            self.items.remove(&id);
        }
    }

    enum SetAction {
        Remove(Ix),
        Add(Ix),
    }

    pub struct Container {
        queues: Rc<RefCell<Queues>>,
        _fonts: Fonts,

        next_id: Ix,
        _root_id: Option<Ix>,
        nodes: BTreeMap<Ix, NodeSkeleton>,

        update_set: Option<BTreeSet<Ix>>,
        action_set: Option<BTreeSet<Ix>>,

        update_set_actions: VecDeque<SetAction>,
        action_set_actions: VecDeque<SetAction>,
    }

    impl Container {
        pub fn new() -> Container {
            Container {
                queues: Rc::new(RefCell::new(Queues::new())),
                _fonts: Fonts::new(),

                next_id: Ix(0),
                _root_id: None,
                nodes: BTreeMap::new(),

                update_set: Some(BTreeSet::new()),
                action_set: Some(BTreeSet::new()),

                update_set_actions: VecDeque::with_capacity(32),
                action_set_actions: VecDeque::with_capacity(32),
            }
        }

        #[inline(always)]
        pub fn fonts(&self) -> &Fonts {
            &self._fonts
        }

        #[inline(always)]
        fn mutate<IA, I, O, OA, InputFunT, MutFunT, OutputFunT>(
            &mut self,
            id: Ix,
            input_arg: IA,
            mut input_fun: InputFunT, // input_arg comes in, I comes out (access to NodeSkeleton and Queues)
            mut mut_fun: MutFunT,     // I comes in, O comes out (access to Container and Body)
            mut output_fun: OutputFunT, // O comes in, OA is returned (access to NodeSkeleton and Queues)
        ) -> OA
        where
            InputFunT: FnMut(&mut NodeSkeleton, &Rc<RefCell<Queues>>, IA) -> I,
            MutFunT: FnMut(&mut NodeBody, &mut Container, I) -> O,
            OutputFunT: FnMut(&mut NodeSkeleton, &Rc<RefCell<Queues>>, O) -> OA,
        {
            let (mut body, input) = {
                let skeleton = self
                    .nodes
                    .get_mut(&id)
                    .expect("mutate 1: self.nodes.get_mut(&id)");
                let input = input_fun(skeleton, &self.queues, input_arg);
                let body = skeleton.steal_body();
                (body, input)
            };

            let output = mut_fun(&mut body, self, input);

            let skeleton = self
                .nodes
                .get_mut(&id)
                .expect("mutate 2: self.nodes.get_mut(&id)");
            skeleton.restore_body(body);

            output_fun(skeleton, &self.queues, output)
        }

        pub fn delete_node(&mut self, id: Ix) {
            if let Some(mut removed) = self.nodes.remove(&id) {
                let body = removed.steal_body();

                for (child_id, _) in body.children.items {
                    self.delete_node(child_id);
                }

                let parent_id = removed.parent_id;

                if let Some(parent_id) = parent_id {
                    if let Some(parent) = self.nodes.get_mut(&parent_id) {
                        parent.body.as_mut().expect("delete_node: parent.body")
                            .children.remove(id);
                    }
                }

                self.queues.borrow_mut().send(Effect::Remove { id })
            }
        }

        pub fn new_root(&mut self, element: Box<Element>, window_scale: f32) -> Ix {
            let root_id = self.next_id.inc();

            self.queues.borrow_mut().send(Effect::Add {
                id: root_id,
                parent_id: None,
            });

            let mut skeleton =
                NodeSkeleton::new(None,Children::empty(), &na::Projective3::identity(), element, window_scale);
            let mut body = skeleton.steal_body();

            self.nodes.clear();
            self.nodes.insert(root_id, skeleton);
            self._root_id = Some(root_id);

            {
                let mut base = Base::new(
                    root_id,
                    self,
                    &mut body.children,
                    ResizeFlow::ParentIsNotResizing,
                    BoxSize::Hidden,
                    window_scale
                );
                body.el.inflate(&mut base);
            }

            let skeleton = self
                .nodes
                .get_mut(&root_id)
                .expect("new_root: self.nodes.get_mut(&root_id)");
            skeleton.restore_body(body);

            self.queues.borrow_mut().send(Effect::Transform {
                id: root_id,
                absolute_transform: Some(na::Projective3::identity()),
            });

            root_id
        }

        pub fn add_node(&mut self, parent_id: Ix, element: Box<Element>) -> Ix {
            let id = self.next_id.inc();

            self.queues.borrow_mut().send(Effect::Add {
                id,
                parent_id: Some(parent_id),
            });

            let (parent_absolute_transform, window_scale) = {
                let parent = self
                    .nodes
                    .get(&parent_id)
                    .expect("add_node 1: self.nodes.get(&parent_id)");
                (parent.absolute_transform(), parent.window_scale)
            };

            let mut skeleton =
                NodeSkeleton::new(Some(parent_id), Children::empty(), &parent_absolute_transform, element, window_scale);
            let mut body = skeleton.steal_body();

            self.nodes.insert(id, skeleton);

            {
                let mut base = Base::new(
                    id,
                    self,
                    &mut body.children,
                    ResizeFlow::ParentIsNotResizing,
                    BoxSize::Hidden,
                    window_scale
                );
                body.el.inflate(&mut base);
            }

            let skeleton = self
                .nodes
                .get_mut(&id)
                .expect("add_node 2: self.nodes.get_mut(&id)");
            skeleton.restore_body(body);

            id
        }

        #[inline(always)]
        pub fn root_id(&self) -> Option<Ix> {
            self._root_id
        }

        #[inline(always)]
        pub fn get_node_mut(&mut self, id: Ix) -> Option<&mut Element> {
            self.nodes.get_mut(&id).map(|node| node.element_mut())
        }

        pub fn resize(&mut self, id: Ix, box_size: BoxSize, window_scale: f32) -> Option<ResolvedSize> {
            self.mutate(
                id,
                box_size,
                |skeleton, _q, size| (skeleton.last_resolved_size, skeleton.window_scale, size),
                |body, container, (last_resolved_size, last_window_scale, box_size)| {
                    let window_scale_changed = !approx_equal(last_window_scale, window_scale, 4);
                    match (window_scale_changed, last_resolved_size, box_size) {
                        (false, Some(LastResolvedSize::ElementSizeHidden), BoxSize::Hidden) => (LastResolvedSize::ElementSizeHidden, None, true, None),
                        (false, Some(LastResolvedSize::ElementSizeAuto(resolved_size)), BoxSize::Auto) => (LastResolvedSize::ElementSizeAuto(resolved_size), resolved_size, true, None),
                        (false, Some(LastResolvedSize::ElementSizeFixed { w, h, size }), BoxSize::Fixed { w: new_w, h: new_h }) if w == new_w && h == new_h => (LastResolvedSize::ElementSizeFixed { w, h, size }, size, true, None),
                        (_, _, box_size) => {
                            if window_scale_changed {
                                body.update_window_scale_for_primitives(window_scale);
                            }

                            let mut base = Base::new(id, container, &mut body.children, ResizeFlow::ParentIsResizing, box_size, window_scale);
                            body.el.resize(&mut base);

                            let resolved_size = match base.resize_flow_output {
                                ResizeFlowOutput::ParentIsResizingResolved(size) => Some(size),
                                ResizeFlowOutput::ParentIsResizingResolvedNone => None,
                                ResizeFlowOutput::ParentIsResizingNoResolve => None,
                                ResizeFlowOutput::ParentIsNotResizingSizeInvalidated => unreachable!("resize should not receive ParentIsNotResizing[..] from resize_flow_output"),
                                ResizeFlowOutput::ParentIsNotResizingNoSizeUpdate => unreachable!("resize should not receive ParentIsNotResizing[..] from resize_flow_output"),
                            };

                            (match box_size {
                                BoxSize::Hidden => LastResolvedSize::ElementSizeHidden,
                                BoxSize::Auto => LastResolvedSize::ElementSizeAuto(resolved_size),
                                BoxSize::Fixed { w, h } => LastResolvedSize::ElementSizeFixed { w, h, size: resolved_size },
                            }, resolved_size, false, if window_scale_changed {
                                Some(window_scale)
                            } else {
                                None
                            })
                        }
                    }
                },
                |skeleton, q, (last_resolved_size, resolved_size, skip_update, new_window_scale)| {
                    if !skip_update || new_window_scale.is_some() {
                        match resolved_size {
                            None => {
                                skeleton.body.as_mut().map(|b| b.hide_primitives(&mut q.borrow_mut()));
                            }
                            _ => {
                                let absolute_transform = skeleton.absolute_transform();
                                skeleton.body.as_mut().map(|b| {
                                    b.sync_primitives(&absolute_transform, &mut q.borrow_mut())
                                });
                            }
                        }
                        q.borrow_mut().send(Effect::Resize { id, size: resolved_size.map(|s| (s.w, s.h)) });
                        skeleton.last_resolved_size = Some(last_resolved_size);
                        if let Some(window_scale) = new_window_scale {
                            skeleton.window_scale = window_scale;
                        }
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
                    (skeleton.resolved_size_is_invisible(), skeleton.absolute_transform())
                },
                |body, container, (has_no_resolved_size, absolute_transform)| {
                    if !has_no_resolved_size {
                        for (child_id, _) in &body.children.items {
                            container.parent_transform(*child_id, &absolute_transform);
                        }
                        body.sync_primitives(&absolute_transform, &mut container.queues.borrow_mut());
                        Some(absolute_transform)
                    } else {
                        None
                    }
                },
                |_skeleton, q, absolute_transform| {
                    q.borrow_mut().send(Effect::Transform {
                        id,
                        absolute_transform,
                    });
                },
            )
        }

        pub fn parent_transform(&mut self, id: Ix, parent_transform: &na::Projective3<f32>) {
            self.mutate(
                id,
                parent_transform,
                |skeleton, _q, parent_transform| {
                    skeleton.parent_transform = parent_transform.clone();
                    (skeleton.resolved_size_is_invisible(), skeleton.absolute_transform())
                },
                |body, container, (has_no_resolved_size, absolute_transform)| {
                    if !has_no_resolved_size {
                        for (child_id, _) in &body.children.items {
                            container.parent_transform(*child_id, &absolute_transform);
                        }
                        body.sync_primitives(&absolute_transform, &mut container.queues.borrow_mut());
                        Some(absolute_transform)
                    } else {
                        None
                    }
                },
                |_skeleton, q, absolute_transform| {
                    q.borrow_mut().send(Effect::Transform {
                        id,
                        absolute_transform,
                    });
                },
            )
        }

        #[inline(always)]
        pub fn hide(&mut self, id: Ix, window_scale: f32) {
            self.resize(id, BoxSize::Hidden, window_scale);
        }

        #[inline(always)]
        pub fn create_queue(&mut self) -> Ix {
            self.queues.borrow_mut().create_queue()
        }

        #[inline(always)]
        pub fn delete_queue(&mut self, id: Ix) {
            self.queues.borrow_mut().delete_queue(id);
        }

        pub fn drain_queue_into(&self, id: Ix, output: &mut Vec<Effect>) {
            if let Some(queue) = self.queues.borrow_mut().get_queue_mut(id) {
                output.extend(queue.drain(..))
            }
        }

        pub fn send_action(&mut self, action: UiAction) {
            let update_list = ::std::mem::replace(&mut self.action_set, None)
                .expect("update: iteration reentry error");

            self.update_template(&update_list, |el, base, arg| {
                el.action(base, arg)
            }, action);

            ::std::mem::replace(&mut self.action_set, Some(update_list));

            let set = self.action_set.as_mut().unwrap();
            while let Some(a) = self.action_set_actions.pop_front() {
                match a {
                    SetAction::Add(ix) => set.insert(ix),
                    SetAction::Remove(ix) => set.remove(&ix),
                };
            }
        }

        pub fn update(&mut self, delta: f32) {
            let update_list = ::std::mem::replace(&mut self.update_set, None)
                .expect("update: iteration reentry error");

            self.update_template(&update_list, |el, base, arg| {
                el.update(base, arg)
            }, delta);

            ::std::mem::replace(&mut self.update_set, Some(update_list));

            let set = self.update_set.as_mut().unwrap();
            while let Some(a) = self.update_set_actions.pop_front() {
                match a {
                    SetAction::Add(ix) => set.insert(ix),
                    SetAction::Remove(ix) => set.remove(&ix),
                };
            }
        }

        #[inline(always)]
        pub fn update_template<A, F>(&mut self, update_list: &BTreeSet<Ix>, mut fun: F, arg: A)
            where F: FnMut(&mut Element, &mut Base, A), A: Copy
        {
            enum ResizeAction {
                None,
                InvalidateSize,
            }

            #[derive(Debug)]
            enum ResizeParentsAction {
                None,
                ResizeParents,
            }

            for id in update_list {
                let (parent_id, post_mutate_resize) = self.mutate(
                    *id,
                    &mut fun,
                    |skeleton, _q, fun| (skeleton.last_resolved_size, skeleton.window_scale, fun),
                    |body, container, (last_resolved_size, window_scale, fun)| {
                        let box_size = match last_resolved_size {
                            None => BoxSize::Hidden,
                            Some(last_resolved_size) => last_resolved_size.to_box_size(),
                        };

                        let resize_flow_output = {
                            let mut base = Base::new(
                                *id, container, &mut body.children,
                                ResizeFlow::ParentIsNotResizing,
                                box_size,
                                window_scale
                            );
                            fun(&mut *body.el, &mut base, arg);
                            base.resize_flow_output
                        };

                        (box_size, match resize_flow_output {
                            ResizeFlowOutput::ParentIsNotResizingNoSizeUpdate =>
                                ResizeAction::None,
                            ResizeFlowOutput::ParentIsNotResizingSizeInvalidated =>
                                ResizeAction::InvalidateSize,
                            ResizeFlowOutput::ParentIsResizingResolved(_) => unreachable!("non resize should not receive ParentIsResizing[..] from resize_flow_output"),
                            ResizeFlowOutput::ParentIsResizingResolvedNone => unreachable!("non resize should not receive ParentIsResizing[..] from resize_flow_output"),
                            ResizeFlowOutput::ParentIsResizingNoResolve => unreachable!("non resize should not receive ParentIsResizing[..] from resize_flow_output"),
                        })
                    },
                    |skeleton, q, (box_size, action)| {
                        match box_size {
                            BoxSize::Hidden => (),
                            _ => {
                                let absolute_transform = skeleton.absolute_transform();
                                skeleton.body.as_mut().map(|b| b.sync_invalidated_primitives(&absolute_transform, &mut q.borrow_mut()));
                            }
                        }

                        (skeleton.parent_id, match action {
                            ResizeAction::None => ResizeParentsAction::None,
                            ResizeAction::InvalidateSize => {
                                skeleton.last_resolved_size = None;
                                ResizeParentsAction::ResizeParents
                            },
                        })
                    },
                );

                if let ResizeParentsAction::ResizeParents = post_mutate_resize {
                    // invalidate parent node sizes

                    let mut root = None;
                    let mut parent_id = parent_id;

                    while let Some(id) = parent_id {
                        let node = self.nodes.get_mut(&id)
                            .expect("update: self.nodes.get_mut(&id)");
                        node.invalidate_transform_propagation();

                        if let Some(_) = node.parent_id {
                            node.last_resolved_size = None;
                        } else {
                            root = Some((id, node.last_resolved_size.map(|s| s.to_box_size()), node.window_scale));
                            node.last_resolved_size = None;
                        };

                        parent_id = node.parent_id;
                    }

                    // reflow from root

                    if let Some((root_id, Some(box_size), window_scale)) = root {
                        self.resize(root_id, box_size, window_scale);
                    }
                }
            }
        }
    }

    pub struct NodeBody {
        children: Children,
        el: Box<Element>,
    }

    use std::cell::RefMut;

    impl NodeBody {
        pub fn invalidate_transform_propagation(&mut self) {
            for child in self.children.items.values_mut() {
                child.invalidate_transform_propagation();
            }
        }

        pub fn update_window_scale_for_primitives(&mut self, window_scale: f32) {
            if let Some(ref mut primitives) = self.children.primitives {
                let mut shared = primitives.shared.borrow_mut();
                shared.set_window_scale(window_scale);
            }
        }

        pub fn hide_primitives(&mut self, queues: &mut RefMut<Queues>) {
            if let Some(ref mut primitives) = self.children.primitives {
                let mut shared = primitives.shared.borrow_mut();

                for buffer in shared.added_text_buffers() {
                    queues.send(Effect::TextAdd {
                        buffer: buffer.weak_ref()
                    });
                }

                for buffer in shared.buffers_keep_invalidated() {
                    queues.send(Effect::TextTransform {
                        buffer_id: buffer.id(),
                        absolute_transform: None,
                    });
                }

                for buffer_id in shared.removed_text_buffers() {
                    queues.send(Effect::TextRemove {
                        buffer_id
                    });
                }
            }
        }

        pub fn sync_primitives(&mut self, absolute_transform: &na::Projective3<f32>, queues: &mut RefMut<Queues>) {
            if let Some(ref mut primitives) = self.children.primitives {
                let mut shared = primitives.shared.borrow_mut();
                for buffer in shared.added_text_buffers() {
                    queues.send(Effect::TextAdd {
                        buffer: buffer.weak_ref()
                    });
                }

                for buffer in shared.buffers() {
                    queues.send(Effect::TextTransform {
                        buffer_id: buffer.id(),
                        absolute_transform: Some(buffer.absolute_transform(absolute_transform)),
                    });
                }

                for buffer_id in shared.removed_text_buffers() {
                    queues.send(Effect::TextRemove {
                        buffer_id
                    });
                }
            }
        }

        pub fn sync_invalidated_primitives(&mut self, absolute_transform: &na::Projective3<f32>, queues: &mut RefMut<Queues>) {
            if let Some(ref mut primitives) = self.children.primitives {
                let mut shared = primitives.shared.borrow_mut();
                for buffer in shared.added_text_buffers() {
                    queues.send(Effect::TextAdd {
                        buffer: buffer.weak_ref()
                    });
                }

                for buffer in shared.only_invalidated_buffers() {
                    queues.send(Effect::TextTransform {
                        buffer_id: buffer.id(),
                        absolute_transform: Some(buffer.absolute_transform(absolute_transform)),
                    });
                }

                for buffer_id in shared.removed_text_buffers() {
                    queues.send(Effect::TextRemove {
                        buffer_id
                    });
                }
            }
        }
    }

    pub struct NodeSkeleton {
        last_resolved_size: Option<LastResolvedSize>,
        parent_transform: na::Projective3<f32>,
        relative_transform: na::Projective3<f32>,
        parent_id: Option<Ix>,
        body: Option<NodeBody>,
        window_scale: f32,
    }

    impl NodeSkeleton {
        pub fn new(
            parent_id: Option<Ix>,
            children: Children,
            parent_transform: &na::Projective3<f32>,
            element: Box<Element>,
            window_scale: f32,
        ) -> NodeSkeleton {
            NodeSkeleton {
                last_resolved_size: None,
                parent_transform: parent_transform.clone(),
                relative_transform: na::Projective3::identity(),
                parent_id,
                body: Some(NodeBody {
                    children,
                    el: element,
                }),
                window_scale
            }
        }

        #[inline(always)]
        pub fn invalidate_transform_propagation(&mut self) {
            self.body
                .as_mut()
                .expect("invalidate_transform_propagation: encountered stolen body")
                .invalidate_transform_propagation();
        }

        pub fn resolved_size_is_invisible(&self) -> bool {
            match self.last_resolved_size {
                None => true,
                Some(LastResolvedSize::ElementSizeHidden) => true,
                _ => false,
            }
        }

        #[inline(always)]
        pub fn absolute_transform(&self) -> na::Projective3<f32> {
            &self.parent_transform * &self.relative_transform
        }

        #[inline(always)]
        pub fn steal_body(&mut self) -> NodeBody {
            self.body
                .take()
                .expect("steal_body: encountered stolen value")
        }

        #[inline(always)]
        pub fn restore_body(&mut self, body: NodeBody) {
            if let Some(_) = ::std::mem::replace(&mut self.body, Some(body)) {
                unreachable!("restore_body: encountered existing value")
            }
        }

        #[inline(always)]
        pub fn element_mut(&mut self) -> &mut Element {
            self.body
                .as_mut()
                .map(|b| &mut *b.el)
                .expect("element_mut: encountered stolen value")
        }
    }
}

pub struct Tree {
    shared: Rc<RefCell<shared::Container>>,
}

impl Tree {
    pub fn new() -> Tree {
        let shared = Rc::new(RefCell::new(shared::Container::new()));

        Tree { shared }
    }

    pub fn create_root<T: Element + 'static>(&self, element: T, window_scale: f32) -> Leaf<T> {
        Leaf {
            _marker: PhantomData,
            id: self
                .shared
                .borrow_mut()
                .new_root(Box::new(element) as Box<Element>, window_scale),
            shared: self.shared.clone(),
        }
    }

    #[inline(always)]
    pub fn update(&self, delta: f32) {
        self.shared.borrow_mut().update(delta)
    }

    #[inline(always)]
    pub fn send_action(&self, action: UiAction) {
        self.shared.borrow_mut().send_action(action)
    }

    #[inline(always)]
    pub fn events(&self) -> Events {
        Events::new(&self.shared)
    }

    #[inline(always)]
    pub fn fonts(&self) -> fonts::Fonts {
        self.shared.borrow().fonts().clone()
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
        let shared = self.shared.borrow_mut();
        shared.drain_queue_into(self.queue_id, output);
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
    pub fn resize(&self, size: BoxSize, window_scale: f32) -> Option<ResolvedSize> {
        self.shared.borrow_mut().resize(self.id, size, window_scale)
    }

    pub fn send_action(&self, action: UiAction) {
        self.shared.borrow_mut().send_action(action)
    }
}

impl<T> Drop for Leaf<T> {
    fn drop(&mut self) {
        self.shared.borrow_mut().delete_node(self.id);
    }
}
