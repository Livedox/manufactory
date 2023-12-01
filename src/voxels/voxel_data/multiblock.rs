use crate::world::global_coords::GlobalCoords;

pub trait MultiBlock {
    fn structure_coordinates(&self) -> &[GlobalCoords];
    fn mut_structure_coordinates(&mut self) -> &mut [GlobalCoords];
}