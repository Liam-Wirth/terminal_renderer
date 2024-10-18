use nalgebra::Vector3;

pub struct Entity {
    pub vertices: Vec<Vector3<f64>>,
}

impl Entity {
    pub fn new(vertices: Vec<Vector3<f64>>) -> Self {
        Entity { vertices }
    }
}

