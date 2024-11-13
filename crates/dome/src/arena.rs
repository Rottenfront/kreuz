use std::collections::HashMap;

use kurbo::RoundedRect;

use crate::{Entity, Region, UiNode};

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct ViewId(usize);

pub struct ViewData {
    view: UiNode,
    relative_region: Region,
    real_region: Region,
    prev_region: Region,
    is_dirty: bool,
}

#[derive(Default)]
pub struct Arena {
    last_view_id: usize,
    views: HashMap<ViewId, ViewData>,
    free_ids: Vec<ViewId>,
}

impl Arena {
    fn new_view_id(&mut self) -> ViewId {
        let id = self.last_view_id;
        self.last_view_id += 1;
        ViewId(id)
    }

    pub fn get_view(&self, id: ViewId) -> Option<&UiNode> {
        self.views.get(&id).map(|data| &data.view)
    }

    pub fn get_previuos_region(&self, id: ViewId) -> Option<RoundedRect> {
        self.views.get(&id).map(|data| data.prev_region)
    }

    pub fn get_relative_region(&self, id: ViewId) -> Option<RoundedRect> {
        self.views.get(&id).map(|data| data.relative_region)
    }

    pub fn get_real_region(&self, id: ViewId) -> Option<RoundedRect> {
        self.views.get(&id).map(|data| data.real_region)
    }

    pub fn get_dirty(&self, id: ViewId) -> Option<bool> {
        self.views.get(&id).map(|data| data.is_dirty)
    }

    pub fn get_view_mut(&mut self, id: ViewId) -> Option<&mut UiNode> {
        self.views.get_mut(&id).map(|data| &mut data.view)
    }

    pub fn set_previuos_region(&mut self, id: ViewId, region: RoundedRect) {
        self.views
            .get_mut(&id)
            .map(|data| data.prev_region = region);
    }

    pub fn set_relative_region(&mut self, id: ViewId, region: RoundedRect) {
        self.views
            .get_mut(&id)
            .map(|data| data.relative_region = region);
    }

    pub fn set_real_region(&mut self, id: ViewId, region: RoundedRect) {
        self.views
            .get_mut(&id)
            .map(|data| data.real_region = region);
    }

    pub fn set_dirty(&mut self, id: ViewId, dirty: bool) {
        self.views.get_mut(&id).map(|data| data.is_dirty = dirty);
    }

    pub fn push_view(&mut self, view: UiNode) -> ViewId {
        let id = self.new_view_id();
        self.views.insert(
            id,
            ViewData {
                view,
                is_dirty: true,
                relative_region: Default::default(),
                real_region: Default::default(),
                prev_region: Default::default(),
            },
        );
        id
    }

    pub fn remove_view(&mut self, id: ViewId) -> Option<ViewData> {
        if let Some(view) = self.views.remove(&id) {
            self.free_ids.push(id);
            Some(view)
        } else {
            None
        }
    }

    pub fn remove_view_verbose(&mut self, id: ViewId) -> HashMap<ViewId, ViewData> {
        if let Some(data) = self.remove_view(id) {
            let mut res = HashMap::new();
            match &data.view.entity {
                Entity::Box(entity) => {
                    for (k, v) in self.remove_view_verbose(entity.inner) {
                        res.insert(k, v);
                    }
                }
                Entity::Stack(entity) => {
                    for id in &entity.inner {
                        for (id, view) in self.remove_view_verbose(*id) {
                            res.insert(id, view);
                        }
                    }
                }
                Entity::Switch(entity) => {
                    for id in &entity.inner {
                        for (id, view) in self.remove_view_verbose(*id) {
                            res.insert(id, view);
                        }
                    }
                }
                _ => {}
            }
            res.insert(id, data);
            res
        } else {
            Default::default()
        }
    }
}
