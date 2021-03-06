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