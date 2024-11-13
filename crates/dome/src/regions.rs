use kurbo::*;
use std::collections::HashMap;

use crate::*;

pub type Region = RoundedRect;

struct CalcData {
    min_size: SizeConstraints,
    max_size: SizeConstraints,
}

pub struct RegionCalc {
    arena: Arena,
    calc_data: HashMap<ViewId, CalcData>,
}

impl RegionCalc {
    pub fn new(arena: Arena) -> Self {
        Self {
            arena,
            calc_data: HashMap::new(),
        }
    }

    fn get_paragraph_size(&self, entity: &ParagraphEntity, width: f64) -> Region {
        let lines = entity.text.clone().lines();

        for line in lines {
            let mut x = line_x;
            let words = line
                .split(" ")
                .map(|s| String::from(s))
                .collect::<Vec<String>>();

            let mut is_first_in_line = true;

            for word in words {
                let layout = rc
                    .text()
                    .new_text_layout(if is_first_in_line {
                        word
                    } else {
                        " ".to_owned() + &word
                    })
                    .font(font.clone(), FONT_SIZE)
                    .default_attribute(TextAttribute::TextColor(color))
                    .build()?;
                is_first_in_line = false;

                let current_width = layout.size().width;
                if current_width + x > max_x {
                    line_x = x0;
                    x = line_x;
                    y += dy;
                }

                rc.draw_text(&layout, (x, y));

                x += current_width;
            }

            y += dy;
            line_x = x0;
        }
    }

    pub fn compute_regions(&mut self, root: ViewId, bounds: Region) -> Option<()> {
        self.compute_size_constraints(root);

        self.compute_region(root, bounds);

        Some(())
    }

    fn compute_region(&mut self, id: ViewId, bounds: Region) -> Option<()> {
        let node = self.arena.get_view(id)?;
        let styles = &node.styles;

        let initial_size = bounds.rect().size();
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
        let region = Region::new(
            origin_x,
            origin_y,
            size.width,
            size.height,
            styles.borders.radius,
        );

        let mut inner_region = {
            let mut rect = region.rect();
            rect.x0 += styles.padding.left + styles.borders.width;
            rect.x1 -= styles.padding.right + styles.borders.width;
            rect.y0 += styles.padding.top + styles.borders.width;
            rect.y1 -= styles.padding.bottom + styles.borders.width;
            rect.to_rounded_rect(0.0)
        };

        match &node.entity {
            Entity::Box(entity) => {
                let id = entity.inner.clone();
                self.compute_region(id, inner_region);
            }
            Entity::Scroll(entity) => {
                let id = entity.inner.clone();
                let offset = entity.offset.clone();
                let mut rect = inner_region.rect();
                rect.x0 -= offset.x;
                rect.y0 -= offset.y;
                inner_region = rect.to_rounded_rect(inner_region.radii());
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
                        width: Some(size.width + delta_width),
                        height: Some(size.height + delta_height),
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
        let mut x0 = region.rect().x0;
        let mut current_region = region;
        for (i, node) in nodes.iter().enumerate() {
            let width = widths[i];
            current_region = current_region.with_width(width);
            let mut rect = current_region.rect();
            rect.x0 = x0;
            current_region = rect.to_rounded_rect(current_region.radii());
            x0 += width;
            self.compute_region(*node, current_region);
        }
    }

    fn result_regions_y_stack(&mut self, nodes: Vec<ViewId>, heights: Vec<f64>, region: Region) {
        let mut y0 = region.rect().y0;
        let mut current_region = region;
        for (i, node) in nodes.iter().enumerate() {
            let height = heights[i];
            current_region = current_region.with_height(height);
            let mut rect = current_region.rect();
            rect.y0 = y0;
            current_region = rect.to_rounded_rect(current_region.radii());
            y0 += height;
            self.compute_region(*node, current_region);
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
            Entity::Image(entity) => {
                let size = rect_size(&entity.radii);
                SizeConstraints {
                    width: Some(size.width),
                    height: Some(size.height),
                }
            }
            Entity::Rect(entity) => {
                let size = rect_size(&entity.radii);
                SizeConstraints {
                    width: Some(size.width),
                    height: Some(size.height),
                }
            }
            Entity::Text(entity) => {
                let size = self.drawer.get_text_blob_size(entity);
                style_max_size = SizeConstraints {
                    width: Some(size.width),
                    height: Some(size.height),
                };
                SizeConstraints {
                    width: Some(size.width),
                    height: Some(size.height),
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

    pub fn destroy(self) -> Arena {
        self.arena
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
