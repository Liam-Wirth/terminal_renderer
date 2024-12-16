use glam::Vec2;
use crate::core::Pixel;

// TODO: Implement a rasterizer struct that takes in that set_pixel closure as an argument or something this is fucking huge
// basically shift all the logic over here and then require that each pipeline has a rasterizer
pub fn bresenham(
    mut start: Vec2,
    mut end: Vec2,
    pixel: Pixel,
    mut set_pixel: impl FnMut(Vec2, f32, Pixel)
) {
    let mut steep = false;
    
    // Convert to integers for the algorithm
    let mut x0 = start.x as i32;
    let mut y0 = start.y as i32;
    let mut x1 = end.x as i32;
    let mut y1 = end.y as i32;

    if (x0 - x1).abs() < (y0 - y1).abs() {
        std::mem::swap(&mut x0, &mut y0);
        std::mem::swap(&mut x1, &mut y1);
        steep = true;
    }
    
    if x0 > x1 {
        std::mem::swap(&mut x0, &mut x1);
        std::mem::swap(&mut y0, &mut y1);
    }

    let dx = x1 - x0;
    let dy = y1 - y0;
    let derror2 = dy.abs() * 2;
    let mut error2 = 0;
    let mut y = y0;

    for x in x0..=x1 {
        let pos = if steep {
            Vec2::new(y as f32, x as f32)
        } else {
            Vec2::new(x as f32, y as f32)
        };
        
        set_pixel(pos, 0.0, pixel);
        
        error2 += derror2;
        if error2 > dx {
            y += if y1 > y0 { 1 } else { -1 };
            error2 -= dx * 2;
        }
    }
}
