mod math;
mod bubble;
mod edge;
mod createDataset;
mod forcelayout;
mod physics;
mod drawable;
mod project;

extern crate minifb;

use math::{Rect, Vector2};
use minifb::{Key, Scale, Window, WindowOptions};
use rand::random;
use raqote::{DrawTarget, SolidSource};

use forcelayout::*;
use drawable::*;

fn main() {
    let mut bubbles = createDataset::create_bubbles(50);
    let mut edges = createDataset::create_edges(bubbles.len());
    // println!("bubbles: {:?}", bubbles);
    // println!("edges: {:?}", edges);
    // println!("{:?}", edges.len());
    // forcelayout(&mut bubbles, &mut edges);

    let mut window_option = WindowOptions::default();
    window_option.resize = true;

    let mut window = Window::new(
        "forcelayout in rust - ESC to exit, Space to replay",
        640,
        640,
        window_option,
    )
    .unwrap_or_else(|e| {
        panic!("{}", e);
    });

    // Limit to max ~60 fps update rate
    // window.limit_update_rate(Some(std::time::Duration::from_micros(16600)));


    let size = window.get_size();
    let mut dt = DrawTarget::new(size.0 as i32, size.1 as i32);

    while window.is_open() && !window.is_key_down(Key::Escape) {
        let size = window.get_size();
        if (size.0 as i32) != dt.width() || (size.1 as i32) != dt.height() {
            dt = DrawTarget::new(size.0 as i32, size.1 as i32);
        }

        if window.is_key_down(Key::Space) {
            bubbles = createDataset::create_bubbles(50);
            edges = createDataset::create_edges(bubbles.len());
        }

        let padding = usize::min(size.0, size.1) as f64 * 0.1;

        let target_rect = Rect{origin: Vector2{x: padding, y: padding}, width: (size.0 - ((2.0*padding) as usize)) as f64, height: (size.1 - ((2.0*padding) as usize)) as f64};
        dt.clear(SolidSource::from_unpremultiplied_argb(0xff, 0xff, 0xff, 0xff));
        forcelayout(&mut bubbles, &mut edges);

        // println!("__________________________________");
        // println!("bubbles: {:?}", bubbles);
        // println!("edges: {:?}", edges);

    
        let b0 = &bubbles[0];
        let mut min_x = b0.position.x;
        let mut max_x = min_x;
        let mut min_y = b0.position.y;
        let mut max_y = min_y;

        // println!("bb {}, {}", bubbles[0].position.x, bubbles[0].position.y);
        // println!("bb {}, {}", bubbles[1].position.x, bubbles[1].position.y);

        for bubble in bubbles.iter() {
            let p = & bubble.position;
            min_x = min_x.min(p.x);
            max_x = max_x.max(p.x);
            min_y = min_y.min(p.y);
            max_y = max_y.max(p.y);
        }

        // println!("{} {} {} {}", min_x, max_x, min_y, max_y);

        let source_rect = Rect{origin: Vector2{x: min_x, y: min_y}, width: max_x - min_x, height: max_y - min_y};
        // println!("st {:?}", source_rect);
        

        // println!("s {} {}", scale_x, scale_y);


        for bubble in bubbles.iter() {
            bubble.draw(&mut dt, &source_rect, &target_rect);
        }
        for edge in edges.iter() {
            edge.draw(&mut dt, &source_rect, &target_rect);
        }

        // We unwrap here as we want this code to exit if it fails. Real applications may want to handle this in a different way
        window
            .update_with_buffer(dt.get_data(), dt.width() as usize, dt.height() as usize)
            .unwrap();
    }
}
