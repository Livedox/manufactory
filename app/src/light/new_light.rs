use std::{cell::UnsafeCell, ops::Index, sync::atomic::{AtomicU8, Ordering}};

use russimp::light;

use crate::voxels::new_chunk::{LocalCoord, CHUNK_VOLUME};

use super::useful::{has_greater_element, max_element_wise, saturated_sub_one};
use super::useful::zero_if_equal_elements;

// We are not worried about data races because nothing critical will happen and performance will increase.
#[repr(transparent)]
#[derive(Debug)]
pub struct Light(pub UnsafeCell<[u8; 4]>);
impl Default for Light {
    #[inline]
    fn default() -> Self {Self(unsafe {std::mem::zeroed()})}
}

impl Clone for Light {
    #[inline]
    fn clone(&self) -> Self {
        unsafe {std::mem::transmute::<_, Self>(*self.0.get().cast::<u32>())}
    }
}

impl PartialEq for Light {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self.to_number() == other.to_number()
    }
}

impl Eq for Light {}

impl Light {
    const MAX_VALUE: u8 = 15;
    #[inline] pub fn new(r: u8, g: u8, b: u8, s: u8) -> Self {
        Self(UnsafeCell::new([r, g, b, s]))
    }

    #[inline(always)] pub unsafe fn get_unchecked(&self, channel: usize) -> u8 {
        unsafe {*(*self.0.get()).get_unchecked(channel)}
    }
    #[inline] pub fn get_channel(&self, channel: usize) -> u8 {unsafe {(*self.0.get())[channel]}}
    #[inline] pub fn get_red(&self) -> u8   {unsafe {self.get_unchecked(0)}}
    #[inline] pub fn get_green(&self) -> u8 {unsafe {self.get_unchecked(1)}}
    #[inline] pub fn get_blue(&self) -> u8  {unsafe {self.get_unchecked(2)}}
    #[inline] pub fn get_sun(&self) -> u8   {unsafe {self.get_unchecked(3)}}


    #[inline(always)] pub unsafe fn set_unchecked(&self, value: u8, channel: usize) {
        unsafe {
            let s = &mut (*self.0.get());
            *s.get_unchecked_mut(channel) = value;
        };
    }
    #[inline(always)]
    pub fn set(&self, value: u8, channel: usize) {
        unsafe {
            let s = &mut (*self.0.get());
            s[channel] = value;
        };
    }
    #[inline] pub fn set_rgb(&self, r: u8, g: u8, b: u8) {
        self.set_red(r);
        self.set_red(g);
        self.set_red(b);
    }
    #[inline] pub fn set_red(&self, value: u8) {
        unsafe {self.set_unchecked(value, 0)};
    }
    #[inline] pub fn set_green(&self, value: u8) {
        unsafe {self.set_unchecked(value, 1)};
    }
    #[inline] pub fn set_blue(&self, value: u8) {
        unsafe {self.set_unchecked(value, 2)};
    }
    #[inline] pub fn set_sun(&self, value: u8) {
        unsafe {self.set_unchecked(value, 3)};
    }

    pub fn get_normalized(&self) -> [f32; 4] {
        [self.get_red() as f32 / Self::MAX_VALUE as f32,
         self.get_green() as f32 / Self::MAX_VALUE as f32,
         self.get_blue() as f32 / Self::MAX_VALUE as f32,
         self.get_sun() as f32 / Self::MAX_VALUE as f32]
    }

    #[inline(always)]
    pub fn to_number(&self) -> u32 {
        unsafe {*self.0.get().cast::<u32>()}
    }

    #[inline(always)]
    pub fn array(&self) -> [u8; 4] {
        unsafe {*self.0.get()}
    }

    pub fn set_light(&self, light: Light) {
        unsafe {
            let l = &mut (*self.0.get());
            *l = light.array();
        };
    }

    pub fn max_element_wise(&self, light: Light) -> Light {
        unsafe {std::mem::transmute::<_, Self>(max_element_wise(self.array(), light.array()))}
    }

    pub fn saturated_sub_one(&self) -> Light {
        unsafe {std::mem::transmute::<_, Self>(saturated_sub_one(self.array()))}
    }

    /// If all elements are less than or equal to one, returns true
    pub fn all_le_one(&self) -> bool {
        (self.to_number() & 0b11111110_11111110_11111110_11111110) == 0
    }

    /// a: [1, 2, 3, 4], b: [1, 3, 4, 4] -> [0, 2, 3, 0]
    pub fn zero_if_equal_elements(&self, light: Light) -> Light {
        unsafe {std::mem::transmute::<_, Self>(
            zero_if_equal_elements(self.array(), light.array()))
        }
    }

    pub fn has_greater_element(&self, light: Light) -> bool {
        has_greater_element(self.array(), light.array())
    }
}