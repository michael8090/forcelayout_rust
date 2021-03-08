use crate::{mesh::Mesh};
pub trait Drawable {
    fn get_mesh() -> Mesh;
}