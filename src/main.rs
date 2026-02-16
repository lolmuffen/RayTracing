mod vector;
mod utils;

use crate::vector::Vec3;

fn main() {
    let vec: Vec3 = Vec3::new(0.0, 0.0, 0.0);
    println!("{0}, {1}, {2}", vec.x, vec.y, vec.z);
}
