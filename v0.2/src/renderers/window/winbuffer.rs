pub struct WinBuffer {
    pub width: usize,
    pub height: usize,
    pub data: Vec<u32>,
    pub depth: Vec<f32>,
}

impl WinBuffer {
    pub fn new(width: usize, height: usize) -> Self {
        let max = width * height;
        WinBuffer {
            width,
            height,
            data: vec![0; max],
            depth: vec![f32::INFINITY; max],
        }
    }
}
