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

const WIDTH: usize = 640;
const HEIGHT: usize = 360;

fn main() {
    let mut bubbles = createDataset::create_bubbles(50);
    let mut edges = createDataset::create_edges(bubbles.len());
    // println!("bubbles: {:?}", bubbles);
    // println!("edges: {:?}", edges);
    // println!("{:?}", edges.len());
    // forcelayout(&mut bubbles, &mut edges);

    let mut window = Window::new(
        "forcelayout in rust - ESC to exit",
        WIDTH,
        HEIGHT,
        WindowOptions::default(),
    )
    .unwrap_or_else(|e| {
        panic!("{}", e);
    });

    // Limit to max ~60 fps update rate
    // window.limit_update_rate(Some(std::time::Duration::from_micros(16600)));

    let size = window.get_size();

    let target_rect = Rect{origin: Vector2{x: 60.0, y: 60.0}, width: (size.0 - 120) as f64, height: (size.1 - 120) as f64};

    let mut dt = DrawTarget::new(size.0 as i32, size.1 as i32);

    while window.is_open() && !window.is_key_down(Key::Escape) {
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
            let s = 0.0;
            let p0_x = p.x - s;
            let p0_y = p.y - s;
            let p1_x = p.x + s;
            let p1_y = p.y + s;
            if min_x > p0_x {
                min_x = p0_x;
            }
            if max_x < p1_x {
                max_x = p1_x;
            }
            if min_y > p0_y {
                min_y = p0_y;
            }
            if max_y < p1_y {
                max_y = p1_y;
            }
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
            .update_with_buffer(dt.get_data(), WIDTH, HEIGHT)
            .unwrap();
    }
}
