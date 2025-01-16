use kurbo::*;
use std::collections::HashMap;

use crate::*;

pub type Region = Rect;

pub struct CalcData {
    min_size: SizeConstraints,
    max_size: SizeConstraints,
}

pub struct RegionCalc<'a, 'b> {
    pub arena: &'a mut Arena,
    pub text: &'b mut SimpleText,
    pub calc_data: HashMap<ViewId, CalcData>,
}

impl<'a, 'b> RegionCalc<'a, 'b> {
    pub fn new(arena: &'a mut Arena, text: &'b mut SimpleText) -> Self {
        Self {
            arena,
            text,
            calc_data: HashMap::new(),
        }
    }

    fn get_paragraph_size(&self, entity: &ParagraphEntity, max_width: f64) -> Region {
        // let text_run = self.text.make_font_run(entity.styles.size as f32, None);
        // let line_height = text_run.get_line_height();
        // let space_width = text_run.get_char_data(' ').width;
        // let height = entity
        //     .text
        //     .lines()
        //     .map(|line| {
        //         line.split_whitespace()
        //             .fold((0.0, line_height), |(x, height), word| {
        //                 let width = text_run.get_word_width(word);
        //                 if x + width > max_width as _ {
        //                     (width + space_width, height + line_height)
        //                 } else {
        //                     (x + width + space_width, height)
        //                 }
        //             })
        //             .1
        //     })
        //     .fold(0.0, |acc, e| acc + e);
        // Region::new(0.0, 0.0, max_width, height as _)
        Default::default()
    }

    fn get_text_blob_size(&self, entity: &TextEntity) -> Region {
        // let text_run = self.text.make_font_run(entity.styles.size as f32, None);
        // let line_height = text_run.get_line_height();
        // let space_width = text_run.get_char_data(' ').width;
        // let (max_width, height) = entity
        //     .text
        //     .lines()
        //     .map(|line| {
        //         line.split_whitespace().fold(0.0, |x, word| {
        //             let width = text_run.get_word_width(word);

        //             if x != 0.0 {
        //                 x + width + space_width
        //             } else {
        //                 width
        //             }
        //         })
        //     })
        //     .fold((0.0f32, 0.0), |(max_width, height), e| {
        //         (max_width.max(e), height + line_height)
        //     });
        // Region::new(0.0, 0.0, max_width as _, height as _)
        Default::default()
    }

    pub fn compute_regions(&mut self, root: ViewId, bounds: Region) -> Option<()> {
        self.compute_size_constraints(root);

        self.compute_region(root, bounds);

        Some(())
    }

    fn compute_region(&mut self, id: ViewId, bounds: Region) -> Option<()> {
        let node = self.arena.get_view(id)?;
        let styles = &node.styles;

        let initial_size = bounds.size();
        let size = if styles.size.zip {
            self.calc_data.get(&id)?.min_size.apply_min(initial_size)
        } else {
            self.calc_data.get(&id)?.max_size.apply_max(initial_size)
        };
        let origin = bounds.origin();
        let delta_width = size.width - initial_size.width;
        let delta_height = size.height - initial_size.height;
        let origin_x = match styles.h_align {
            HAlign::Left => origin.x,
            HAlign::Center => origin.x + delta_width / 2.0,
            HAlign::Right => origin.x + delta_width,
        };
        let origin_y = match styles.v_align {
            VAlign::Top => origin.y,
            VAlign::Center => origin.y + delta_height / 2.0,
            VAlign::Bottom => origin.y + delta_height,
        };
        let region = Region::new(origin_x, origin_y, size.width, size.height);

        let mut inner_region = {
            let mut rect = region;
            rect.x0 += styles.padding.left + styles.borders.width;
            rect.x1 -= styles.padding.right + styles.borders.width;
            rect.y0 += styles.padding.top + styles.borders.width;
            rect.y1 -= styles.padding.bottom + styles.borders.width;
            rect
        };

        match &node.entity {
            Entity::Box(entity) => {
                let id = entity.inner.clone();
                self.compute_region(id, inner_region);
            }
            Entity::Scroll(entity) => {
                let id = entity.inner.clone();
                let offset = entity.offset.clone();
                inner_region.x0 -= offset.x;
                inner_region.y0 -= offset.y;
                self.compute_region(id, inner_region);
            }
            Entity::Switch(entity) => {
                let id = entity.inner[entity.mode].clone();
                self.compute_region(id, inner_region);
            }
            Entity::Stack(entity) => {
                let ids = entity.inner.clone();
                let direction = entity.direction;
                self.compute_stack_inner_region(direction, ids, inner_region);
            }
            _ => {}
        }

        self.arena.set_relative_region(id, region);

        Some(())
    }

    fn compute_stack_inner_region(
        &mut self,
        direction: StackDirection,
        nodes: Vec<ViewId>,
        bounds: Region,
    ) -> Option<()> {
        match direction {
            StackDirection::X => {
                let mut min_widths = vec![];
                let mut min_width_sum = 0.0;
                let mut max_widths = vec![];
                let mut max_width_sum = 0.0;
                for id in &nodes {
                    let data = self.calc_data.get(id);
                    if let Some(data) = data {
                        if let Some(width) = data.min_size.width {
                            min_width_sum += width;
                            min_widths.push(width);
                        } else {
                            min_widths.push(0.0);
                        }
                        if let Some(width) = data.max_size.width {
                            max_width_sum += width;
                            max_widths.push(width);
                        } else {
                            max_widths.push(0.0);
                        }
                    } else {
                        min_widths.push(0.0);
                        max_widths.push(0.0);
                    }
                }

                if min_width_sum >= bounds.width() {
                    self.result_regions_x_stack(nodes, min_widths, bounds);
                    return Some(());
                }

                if max_width_sum <= bounds.width() {
                    self.result_regions_x_stack(nodes, max_widths, bounds);
                    return Some(());
                }
                let mut left_count = nodes.len();
                let mut have_size = Vec::new();
                have_size.resize(left_count, false);
                let mut region_width = bounds.width();
                let mut avg_width = region_width / left_count as f64;

                let mut widths = min_widths.clone();
                loop {
                    let mut modified = false;
                    for (i, width) in min_widths.iter().enumerate() {
                        if !have_size[i] && *width > avg_width {
                            have_size[i] = true;
                            region_width -= width;
                            left_count -= 1;
                            avg_width = region_width / left_count as f64;
                            modified = true;
                        }
                    }
                    if !modified {
                        break;
                    }
                }
                loop {
                    let mut modified = false;
                    for (i, width) in max_widths.iter().enumerate() {
                        if !have_size[i] && *width < avg_width {
                            widths[i] = *width;
                            have_size[i] = true;
                            region_width -= width;
                            left_count -= 1;
                            avg_width = region_width / left_count as f64;
                            modified = true;
                        }
                    }
                    if !modified {
                        break;
                    }
                }
                for (i, have) in have_size.iter().enumerate() {
                    if *have {
                        continue;
                    }
                    widths[i] = avg_width;
                }
                self.result_regions_x_stack(nodes, widths, bounds);
            }
            StackDirection::Y => {
                self.recompute_paragraph_size_with_width(&nodes, bounds.width());

                let mut min_heights = vec![];
                let mut min_height_sum = 0.0;
                let mut max_heights = vec![];
                let mut max_height_sum = 0.0;
                for id in &nodes {
                    let data = self.calc_data.get(id);
                    if let Some(data) = data {
                        if let Some(height) = data.min_size.height {
                            min_height_sum += height;
                            min_heights.push(height);
                        } else {
                            min_heights.push(0.0);
                        }
                        if let Some(height) = data.max_size.height {
                            max_height_sum += height;
                            max_heights.push(height);
                        } else {
                            max_heights.push(0.0);
                        }
                    } else {
                        min_heights.push(0.0);
                        max_heights.push(0.0);
                    }
                }

                if min_height_sum >= bounds.height() {
                    self.result_regions_y_stack(nodes, min_heights, bounds);
                    return Some(());
                }

                if max_height_sum <= bounds.height() {
                    self.result_regions_y_stack(nodes, max_heights, bounds);
                    return Some(());
                }
                let mut left_count = nodes.len();
                let mut have_size = Vec::new();
                have_size.resize(left_count, false);
                let mut region_height = bounds.height();
                let mut avg_height = region_height / left_count as f64;

                let mut heights = min_heights.clone();
                loop {
                    let mut modified = false;
                    for (i, height) in min_heights.iter().enumerate() {
                        if !have_size[i] && *height > avg_height {
                            have_size[i] = true;
                            region_height -= height;
                            left_count -= 1;
                            avg_height = region_height / left_count as f64;
                            modified = true;
                        }
                    }
                    if !modified {
                        break;
                    }
                }
                loop {
                    let mut modified = false;
                    for (i, height) in max_heights.iter().enumerate() {
                        if !have_size[i] && *height < avg_height {
                            heights[i] = *height;
                            have_size[i] = true;
                            region_height -= height;
                            left_count -= 1;
                            avg_height = region_height / left_count as f64;
                            modified = true;
                        }
                    }
                    if !modified {
                        break;
                    }
                }
                for (i, have) in have_size.iter().enumerate() {
                    if *have {
                        continue;
                    }
                    heights[i] = avg_height;
                }
                self.result_regions_y_stack(nodes, heights, bounds);
            }
            StackDirection::Z => {
                for id in nodes {
                    self.compute_region(id, bounds);
                }
            }
        }

        Some(())
    }

    fn recompute_paragraph_size_with_width(&mut self, nodes: &Vec<ViewId>, width: f64) {
        for id in nodes {
            let node = self.arena.get_view(*id).unwrap();
            let delta_width = node.styles.padding.left
                + node.styles.padding.right
                + node.styles.borders.width * 2.0;
            let width = width - delta_width;
            match &node.entity {
                Entity::Paragraph(entity) => {
                    let delta_height = node.styles.padding.top
                        + node.styles.padding.bottom
                        + node.styles.borders.width * 2.0;
                    let size = self.get_paragraph_size(entity, width);
                    let min_size = SizeConstraints {
                        width: Some(size.width() + delta_width),
                        height: Some(size.height() + delta_height),
                    };
                    if let Some(constraints) = self.calc_data.get_mut(id) {
                        constraints.min_size = min_size;
                    }
                }
                Entity::Scroll(entity) => {
                    let id = entity.inner.clone();
                    let h_scroll_enabled = entity.h_enabled;
                    if h_scroll_enabled {
                        self.recompute_paragraph_size_with_width(&vec![id], width);
                    }
                }
                Entity::Box(entity) => {
                    let id = entity.inner.clone();
                    self.recompute_paragraph_size_with_width(&vec![id], width);
                }
                Entity::Stack(entity) => {
                    let direction = entity.direction.clone();
                    if direction != StackDirection::X {
                        let ids = entity.inner.clone();
                        self.recompute_paragraph_size_with_width(&ids, width);
                    }
                }
                Entity::Switch(entity) => {
                    let id = entity.inner[entity.mode].clone();
                    self.recompute_paragraph_size_with_width(&vec![id], width);
                }
                _ => (),
            }
        }
    }

    fn result_regions_x_stack(&mut self, nodes: Vec<ViewId>, widths: Vec<f64>, region: Region) {
        let mut x0 = region.x0;
        for (i, node) in nodes.iter().enumerate() {
            let width = widths[i];
            let mut rect = region.with_size((width, region.height()));
            rect.x0 = x0;
            x0 += width;
            self.compute_region(*node, rect);
        }
    }

    fn result_regions_y_stack(&mut self, nodes: Vec<ViewId>, heights: Vec<f64>, region: Region) {
        let mut y0 = region.y0;
        for (i, node) in nodes.iter().enumerate() {
            let height = heights[i];
            let mut rect = region.with_size((region.width(), height));
            rect.y0 = y0;
            y0 += height;
            self.compute_region(*node, rect);
        }
    }

    fn compute_size_constraints(&mut self, node: ViewId) -> Option<()> {
        let styles = self.arena.get_view(node)?.styles.clone();
        let (style_min_size, mut style_max_size) = (styles.size.min_size, styles.size.max_size);
        let border_width = styles.borders.width;
        let padding = styles.padding;
        let mut is_scrollable = false;

        let min_size = match &self.arena.get_view(node)?.entity {
            Entity::Box(entity) => {
                let id = entity.inner.clone();
                self.compute_size_constraints(id);
                self.calc_data.get(&id)?.min_size
            }
            Entity::Scroll(entity) => {
                let id = entity.inner.clone();
                is_scrollable = !entity.enable_inner_min_size;
                self.compute_size_constraints(id);
                if is_scrollable {
                    style_min_size
                } else {
                    self.calc_data.get(&id)?.min_size
                }
            }
            Entity::Stack(entity) => {
                let nodes = entity.inner.clone();
                let direction = entity.direction.clone();
                self.compute_stack_size_constraints(direction, nodes)
            }
            Entity::Switch(entity) => {
                let id = entity.inner[entity.mode].clone();
                self.compute_size_constraints(id);
                self.calc_data.get(&id)?.min_size
            }
            Entity::Image(_) => SizeConstraints {
                width: None,
                height: None,
            },
            Entity::Rect(entity) => {
                let size = rect_size(&entity.radii);
                SizeConstraints {
                    width: Some(size.width),
                    height: Some(size.height),
                }
            }
            Entity::Text(entity) => {
                let size = self.get_text_blob_size(entity);
                style_max_size = SizeConstraints {
                    width: Some(size.width()),
                    height: Some(size.height()),
                };
                SizeConstraints {
                    width: Some(size.width()),
                    height: Some(size.height()),
                }
            }
            Entity::Paragraph(entity) => {
                let height = text_height(&entity.text, &entity.styles);
                SizeConstraints {
                    width: None,
                    height: Some(height),
                }
            }
            _ => Default::default(),
        };

        // if it is scrollable, and the flag `enable_inner_min_size` is disable, we should not
        // care about inner size constraints
        let min_size = if is_scrollable {
            style_min_size
        } else {
            SizeConstraints {
                width: sum_min_size(
                    min_size.width,
                    border_width * 2.0 + padding.left + padding.right,
                ),
                height: sum_min_size(
                    min_size.height,
                    border_width * 2.0 + padding.top + padding.bottom,
                ),
            }
            .max(style_min_size)
        };

        let max_size = SizeConstraints {
            width: sum_max_size(
                style_max_size.width,
                border_width * 2.0 + padding.left + padding.right,
            ),
            height: sum_max_size(
                style_max_size.height,
                border_width * 2.0 + padding.top + padding.bottom,
            ),
        };

        let calc_data = CalcData { min_size, max_size };
        self.calc_data.insert(node, calc_data);

        Some(())
    }

    fn compute_stack_size_constraints(
        &mut self,
        direction: StackDirection,
        nodes: Vec<ViewId>,
    ) -> SizeConstraints {
        let mut sizes = vec![];
        for node in nodes {
            self.compute_size_constraints(node);
            if let Some(data) = self.calc_data.get(&node) {
                sizes.push(data.min_size);
            }
        }
        let mut result = SizeConstraints::default();
        match direction {
            StackDirection::X => {
                for size in sizes {
                    result = SizeConstraints {
                        width: sum_options(result.width, size.width),
                        height: max_options(result.height, size.height),
                    };
                }
            }
            StackDirection::Y => {
                for size in sizes {
                    result = SizeConstraints {
                        width: max_options(result.width, size.width),
                        height: sum_options(result.height, size.height),
                    };
                }
            }
            StackDirection::Z => {
                for size in sizes {
                    result = SizeConstraints {
                        width: max_options(result.width, size.width),
                        height: max_options(result.height, size.height),
                    };
                }
            }
        }
        result
    }
}

fn rect_size(rect_radii: &RoundedRectRadii) -> Size {
    Size::new(
        (rect_radii.top_left + rect_radii.top_right)
            .max(rect_radii.bottom_left + rect_radii.bottom_right),
        (rect_radii.top_left + rect_radii.bottom_left)
            .max(rect_radii.top_right + rect_radii.bottom_right),
    )
}

fn sum_max_size(width: Option<f64>, border: f64) -> Option<f64> {
    if let Some(width) = width {
        Some(width + border)
    } else {
        None
    }
}

fn sum_min_size(width: Option<f64>, border: f64) -> Option<f64> {
    Some(border + width.unwrap_or(0.0))
}

fn text_height(text: &String, styles: &TextStyles) -> f64 {
    let mut lines = 1;
    for ch in text.chars() {
        if ch == '\n' {
            lines += 1;
        }
    }
    lines as f64 * styles.size
}
