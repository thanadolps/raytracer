use serde::Serialize;

pub mod aabb;
pub mod cell_vec;
#[cfg(test)]
mod fuzzing;
pub mod proxy_serialize;

pub fn print_ron(x: &impl Serialize) {
    println!("{}", ron::to_string(x).unwrap());
}
