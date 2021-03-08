use crate::math::{Rect, Vector2};

pub fn fit_into_view(v: &Vector2, rect_source: &Rect, rect_target: &Rect) -> Vector2 {
    let mut scale_x = 1.0;
    if rect_source.width > 0.0 {
        scale_x = rect_target.width / rect_source.width;
    }
    
    let mut scale_y = 1.0;
    if rect_source.height > 0.0 {
        scale_y = rect_target.height / rect_source.height;
    }
    
    let mut p = v.clone();
    p.x = (p.x - rect_source.origin.x ) * scale_x + rect_target.origin.x;
    p.y = (p.y - rect_source.origin.y) * scale_y + rect_target.origin.y;
    p
}

// it's interesting that the projection method for direction vectors should 
// be considered about how they're meant to be used
// the same thing happens when we take care of mesh normals
// note that I keep the len the same after the projection
pub fn project_direction_vector(v: &Vector2, source_rect_width: f32, source_rect_height: f32, target_rect_width: f32, target_rect_height: f32) -> Vector2 {
    let mut scale_x = 1.0;
    if source_rect_width > 0.0 {
        scale_x = target_rect_width / source_rect_width;
    }
    
    let mut scale_y = 1.0;
    if source_rect_height > 0.0 {
        scale_y = target_rect_height / source_rect_height;
    }

    let len = v.len();
    
    v.mul(&Vector2{x: scale_x, y: scale_y}).norm().mul_s(len)
}
