use std::{fmt::Debug, cell::{RefCell, RefMut}, rc::{Rc, Weak}, collections::HashMap, time::{Duration, Instant}};
use wgpu::Instance;

use crate::{direction::{Direction, self}, voxels::chunk::Chunk, recipes::{recipe::{Recipes, ActiveRecipe}, storage::Storage, item::{PossibleItem, Item}, recipes::RECIPES}, player};

use super::{chunks::Chunks, assembling_machine::AssemblingMachine, transport_belt::TransportBelt, block::blocks::BLOCKS};

#[derive(Debug)]
pub enum PlayerUnlockableStorage {
    VoxelBox(Weak<RefCell<VoxelBox>>),
    Furnace(Weak<RefCell<Furnace>>),
    AssemblingMachine(Weak<RefCell<AssemblingMachine>>),
}


impl PlayerUnlockableStorage {
    pub fn to_storage(&self) -> Weak<RefCell<dyn Storage>> {
        match self {
            PlayerUnlockableStorage::VoxelBox(s) => s.clone(),
            PlayerUnlockableStorage::Furnace(s) => s.clone(),
            PlayerUnlockableStorage::AssemblingMachine(s) => s.clone(),
        }
    }
}


#[derive(Debug)]
pub struct VoxelData {
    pub id: u32,
    pub global_coords: (i32, i32, i32),

    pub additionally: Rc<VoxelAdditionalData>,
}

impl VoxelData {
    pub fn update(&self, chunks: *mut Chunks) {
        if self.id == 1 {return};
        self.additionally.update(&self.global_coords, chunks);
    }

    pub fn rotation_index(&self) -> Option<u32> {
        self.additionally.rotation_index()
    }

    pub fn player_unlockable_storage(&self) -> Option<PlayerUnlockableStorage> {
        self.additionally.player_unlockable_storage()
    }
}


#[derive(Debug)]
pub enum VoxelAdditionalData {
    Empty,
    Manipulator(Box<RefCell<Manipulator>>),
    Cowboy(Box<RefCell<Cowboy>>),
    VoxelBox(Rc<RefCell<VoxelBox>>),
    Furnace(Rc<RefCell<Furnace>>),
    Drill(Rc<RefCell<Drill>>),
    AssemblingMachine(Rc<RefCell<AssemblingMachine>>),
    TransportBelt(Rc<RefCell<TransportBelt>>),
}


impl VoxelAdditionalData {
    pub fn new_multiblock(id: u32, direction: &Direction, structure_coordinates: Vec<(i32, i32, i32)>) -> Self {
        match id {
            15 => Self::Drill(Rc::new(RefCell::new(Drill::new(structure_coordinates, direction)))),
            16 => Self::AssemblingMachine(Rc::new(RefCell::new(AssemblingMachine::new(structure_coordinates)))),
            _ => Self::Empty,
        }
    }

    pub fn new(id: u32, direction: &Direction) -> Self {
        match id {
            9 => Self::Manipulator(Box::new(RefCell::new(Manipulator::new(direction)))),
            12 => Self::Cowboy(Box::new(RefCell::new(Cowboy::new()))),
            13 => Self::VoxelBox(Rc::new(RefCell::new(VoxelBox::new()))),
            14 => Self::Furnace(Rc::new(RefCell::new(Furnace::new()))),
            17 => Self::TransportBelt(Rc::new(RefCell::new(TransportBelt::new(direction)))),
            _ => Self::Empty,
        }
    }


    pub fn transport_belt(&self) -> Option<Rc<RefCell<TransportBelt>>> {
        match self {
            VoxelAdditionalData::TransportBelt(b) => Some(b.clone()),
            _ => None,
        }
    }


    pub fn storage(&self) -> Option<Rc<RefCell<dyn Storage>>> {
        Some(match self {
            VoxelAdditionalData::VoxelBox(b) => b.clone(),
            VoxelAdditionalData::Furnace(f) => f.clone(),
            VoxelAdditionalData::AssemblingMachine(a) => a.clone(),
            VoxelAdditionalData::TransportBelt(c) => c.clone(),
            _ => return None,
        })
    }


    pub fn update(&self, coords: &(i32, i32, i32), chunks: *mut Chunks) {
        match self {
            Self::Manipulator(o) => o.borrow_mut().update(coords, chunks),
            Self::Drill(d) => d.borrow_mut().update(chunks),
            Self::Cowboy(o) => o.borrow_mut().update(),
            Self::Furnace(f) => f.borrow_mut().update(),
            Self::AssemblingMachine(a) => a.borrow_mut().update(chunks),
            Self::TransportBelt(c) => c.borrow_mut().update(coords, chunks),
            Self::Empty | Self::VoxelBox(_) => (),
        }
    }


    pub fn animation_progress(&self) -> Option<f32> {
        match self {
            Self::Manipulator(o)=> Some(o.borrow().animation_progress()),
            Self::Cowboy(o) => Some(o.borrow().animation_progress()),
            Self::Empty | Self::VoxelBox(_) | Self::Furnace(_) |
            Self::Drill(_) | Self::AssemblingMachine(_) | Self::TransportBelt(_) => None,
        }
    }


    pub fn rotation_index(&self) -> Option<u32> {
        match self {
            Self::Manipulator(o) => {Some(o.borrow().rotation_index())},
            Self::TransportBelt(o) => {Some(o.borrow().rotation_index())},
            Self::Drill(o) => {Some(o.borrow().rotation_index())},
            _ => None,
        }
    }


    pub fn player_unlockable_storage(&self) -> Option<PlayerUnlockableStorage> {
        match self {
            Self::VoxelBox(o)=> {Some(PlayerUnlockableStorage::VoxelBox(Rc::downgrade(o)))},
            Self::Furnace(o) => {Some(PlayerUnlockableStorage::Furnace(Rc::downgrade(o)))},
            Self::AssemblingMachine(o) =>  {Some(PlayerUnlockableStorage::AssemblingMachine(Rc::downgrade(o)))}
            _ => None,
        }
    } 
}

#[derive(Debug)]
pub struct Manipulator {
    progress: f32,
    item_id: Option<u32>,
    direction: [i8; 3],
}


impl Manipulator {
    pub fn new(direction: &Direction) -> Self {Self {
        progress: 0.0,
        item_id: None,
        direction: direction.simplify_to_one_greatest(true, false, true),
    }}

    pub fn update(&mut self, coords: &(i32, i32, i32), chunks: *mut Chunks) {
        if self.item_id.is_some() {self.progress += 0.1};
        if self.item_id.is_none() {self.progress -= 0.1};
        if self.progress > 1.0 {self.progress = 1.0};
        if self.progress < 0.0 {self.progress = 0.0};
        if self.item_id.is_none() && self.progress == 0.0 {
            let src_coords = (coords.0 - self.direction[0] as i32, coords.1, coords.2 - self.direction[2] as i32);
            let src = unsafe {
                chunks.as_mut().unwrap().mut_chunk_by_global(src_coords.0, src_coords.1, src_coords.2)
            };
            if let Some(src_chunk) = src {
                let Some(src_data) = src_chunk.mut_voxel_data(Chunks::local_coords(src_coords.0, src_coords.1, src_coords.2)) else {return};
                let Some(storage) = src_data.additionally.storage() else {return};
                if let Some(item) = storage.borrow_mut().take_first_existing(1) {
                    self.item_id = Some(item.0.id());
                };
            }
        }
        
        if self.item_id.is_some() && self.progress == 1.0 {
            let dst_coords = (coords.0 + self.direction[0] as i32, coords.1, coords.2 + self.direction[2] as i32);
            let dst = unsafe {
                chunks.as_mut().unwrap().mut_chunk_by_global(dst_coords.0, dst_coords.1, dst_coords.2)
            };
            if let Some(dst_chunk) = dst {
                let Some(dst_data) = dst_chunk.mut_voxel_data(Chunks::local_coords(dst_coords.0, dst_coords.1, dst_coords.2)) else {return};
                let Some(storage) = dst_data.additionally.storage() else {return};
                let result = storage.borrow_mut().add(&Item::new(self.item_id.unwrap(), 1), false).is_none();
                if result {
                    self.item_id = None;
                }
            }
        }
    }


    pub fn animation_progress(&self) -> f32 {
        self.progress
    }


    pub fn rotation_index(&self) -> u32 {
        if self.direction[0] < 0 {return 2};
        if self.direction[2] > 0 {return 3};
        if self.direction[2] < 0 {return 1};
        0
    }
}


#[derive(Debug)]
pub struct Cowboy {
    progress: f32,
}


impl Cowboy {
    pub fn new() -> Self {Self { progress: 0.0 }}
    pub fn update(&mut self) {
        self.progress += 0.1;
        if self.progress > 1.0 {self.progress %= 1.0};
    }


    pub fn animation_progress(&self) -> f32 {
        self.progress
    }
}


#[derive(Debug, Clone)]
pub struct VoxelBox {
    storage: [PossibleItem; 30]
}

impl VoxelBox {
    pub fn new() -> Self {
        let mut storage = [PossibleItem::new_none(); 30];
        storage[0] = PossibleItem::new(0, 10);
        Self { storage }
    }
}

impl Storage for VoxelBox {
    fn storage<'a>(&'a self) -> &'a [PossibleItem] {
        &self.storage
    }

    fn mut_storage<'a>(&'a mut self) -> &'a mut [PossibleItem] {
        &mut self.storage
    }
}


#[derive(Debug)]
pub struct Furnace {
    storage: [PossibleItem; 2],
    active_recipe: Option<ActiveRecipe>,
}


impl Furnace {
    pub fn new() -> Self {
        Self {
            storage: [PossibleItem::new_none(); 2],
            active_recipe: None
        }
    }

    pub fn update(&mut self) {
        let active_recipe_take = self.active_recipe.take();
        if let Some(active_recipe) = &active_recipe_take {
            let storage = self.mut_storage();
            if active_recipe.is_finished() && storage[1].is_possible_add(&active_recipe.recipe.result) {
                storage[1].try_add_item(&active_recipe.recipe.result);
                self.active_recipe = None;
            } else {
                self.active_recipe = active_recipe_take;
            }
        } else {
            let Some(item) = &self.storage[0].0 else {return};
            let Some(recipe) = RECIPES().furnace.first_by_ingredient(item.id()).cloned() else {return};
            self.active_recipe = self.start_recipe(&recipe);
        }
    }
}



impl Storage for Furnace {
    fn storage<'a>(&'a self) -> &'a [PossibleItem] {
        &self.storage
    }

    fn mut_storage<'a>(&'a mut self) -> &'a mut [PossibleItem] {
        &mut self.storage
    }

    fn take_first_existing(&mut self, max_count: u32) -> Option<(Item, usize)> {
        self.mut_storage()[1].try_take(max_count).and_then(|i| Some((i, 1)))
    }

    fn add(&mut self, item: &Item, _: bool) -> Option<Item> {
        if RECIPES().furnace.get_by_ingredient(item.id()).is_some() {
            return self.mut_storage()[0].try_add_item(item);
        }
        Some(item.clone())
    }

    fn is_item_exist(&self, item: &Item) -> bool {
        self.storage[0].contains(item.id()) >= item.count
    }
}


pub trait MultiBlock {
    fn structure_coordinates(&self) -> &[(i32, i32, i32)];
    fn mut_structure_coordinates(&mut self) -> &mut [(i32, i32, i32)];
}


#[derive(Debug)]
pub struct Drill {
    dir: [i8; 3],
    storage: [PossibleItem; 1],
    structure_coordinates: Vec<(i32, i32, i32)>,
    start: Instant,
}


impl Drill {
    const DURATION: Duration = Duration::new(1, 0);

    pub fn new(structure_coordinates: Vec<(i32, i32, i32)>, dir: &Direction) -> Self {Self {
        storage: [PossibleItem::new_none()],
        structure_coordinates,
        start: Instant::now(),
        dir: dir.simplify_to_one_greatest(true, false, true)
    }}

    pub fn update(&mut self, chunks: *mut Chunks) {
        let xyz = self.structure_coordinates[0];
        let voxels_data = unsafe {chunks.as_mut().unwrap().mut_voxels_data(Chunks::chunk_coords(xyz.0 - self.dir[0] as i32, xyz.1, xyz.2-self.dir[2] as i32))};
        if let Some(storage) = voxels_data
            .and_then(|vd| vd.get_mut(&Chunk::voxel_index(Chunks::local_coords(xyz.0 - self.dir[0] as i32, xyz.1, xyz.2-self.dir[2] as i32))))
            .and_then(|d| d.additionally.storage()) {
                if let Some(item) = self.storage[0].0.take() {
                    if let Some(r_item) = storage.borrow_mut().add(&item, false) {
                        self.storage[0].try_add_item(&r_item);
                    }
                }
            }

        if self.start.elapsed() < Self::DURATION {return}
        self.start = Instant::now();
        
        self.structure_coordinates.iter().for_each(|coord| {
            let voxel = unsafe {chunks.as_mut().unwrap().voxel_global(coord.0, coord.1 - 1, coord.2)};
            let Some(voxel) = voxel else {return};
            if let Some(item) = BLOCKS()[voxel.id as usize].ore() {
                self.storage[0].try_add_item(&item);
            }
        });
    }

    pub fn rotation_index(&self) -> u32 {
        if self.dir[2] > 0 {return 0};
        if self.dir[0] < 0 {return 3};
        if self.dir[2] < 0 {return 2};
        1
    }
}



impl Storage for Drill {
    fn storage<'a>(&'a self) -> &'a [PossibleItem] {
        &self.storage
    }

    fn mut_storage<'a>(&'a mut self) -> &'a mut [PossibleItem] {
        &mut self.storage
    }
}


impl MultiBlock for Drill {
    fn structure_coordinates(&self) -> &[(i32, i32, i32)] {
        &self.structure_coordinates
    }

    fn mut_structure_coordinates(&mut self) -> &mut [(i32, i32, i32)] {
        &mut self.structure_coordinates
    }
}