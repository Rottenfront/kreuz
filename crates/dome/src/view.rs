use super::*;

pub trait View {
    fn build(self, arena: &mut Arena) -> ViewId;
}