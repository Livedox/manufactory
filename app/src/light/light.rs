use std::{cell::UnsafeCell, fmt, ops::Index, simd::{cmp::{SimdOrd, SimdPartialEq, SimdPartialOrd}, num::SimdUint, Simd}, sync::atomic::{AtomicU8, Ordering}};

use itertools::Itertools;
use russimp::light;

use crate::voxels::chunk::{CHUNK_VOLUME};
use super::useful::{has_greater_element, max_element_wise, saturated_sub_one};
use super::useful::zero_if_equal_elements;

// We are not worried about data races because nothing critical will happen and performance will increase.
#[repr(transparent)]
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
    // Do not change!
    pub const MAX: u8 = 15;
    // Do not change!
    pub const BIT_SHIFT: usize = (Self::MAX + 1).ilog2() as usize;

    pub const MAX_COLOR: f32 = 255.0;
    pub const R: [f32; 3] = [139.0, 87.0, 29.0];
    pub const G: [f32; 2] = [165.0, 90.0];
    pub const B: [f32; 2] = [165.0, 90.0];

    #[inline]
    pub const fn new(s: u8, r0: u8, r1: u8, r2: u8, g0: u8, g1: u8, b0: u8, b1: u8) -> Self {
        let first = s << Self::BIT_SHIFT | r0;
        let second = r1 << Self::BIT_SHIFT | r2;
        let third = g0 << Self::BIT_SHIFT | g1;
        let fourth = b0 << Self::BIT_SHIFT | b1;
        Self(UnsafeCell::new([first, second, third, fourth]))
    }

    #[inline]
    pub const fn with_sun(s: u8) -> Self {
        Self(UnsafeCell::new([s << Self::BIT_SHIFT, 0, 0, 0]))
    }

    #[inline] 
    pub const fn with_rgb(r: u8, g: u8, b: u8) -> Self {
        debug_assert!(r <= Self::MAX);
        Self(UnsafeCell::new([r, r<<Self::BIT_SHIFT|r,
            g<<Self::BIT_SHIFT|g, b<<Self::BIT_SHIFT|b]))
    }

    #[inline] 
    pub const fn with_srgb(s: u8, r: u8, g: u8, b: u8) -> Self {
        Self(UnsafeCell::new([s<<Self::BIT_SHIFT|r, r<<Self::BIT_SHIFT|r,
            g<<Self::BIT_SHIFT|g, b<<Self::BIT_SHIFT|b]))
    }

    #[inline(always)] unsafe fn get_unchecked(&self, index: usize) -> u8 {
        unsafe {*(*self.0.get()).get_unchecked(index)}
    }

    fn get(&self, index: usize) -> u8 {
        unsafe {(*self.0.get())[index]}
    }

    #[inline] pub fn get_channel(&self, channel: usize) -> u8 {unsafe {(*self.0.get())[channel]}}
    #[inline] pub fn get_red(&self) -> [u8; 3]   {
        let r0 = unsafe {self.get_unchecked(0)} & Self::MAX;
        let r12 = unsafe {self.get_unchecked(1)};
        let r2 = r12 & Self::MAX;
        let r1 = r12 >> Self::BIT_SHIFT;
        [r0, r1, r2]
    }
    #[inline] pub fn get_green(&self) -> [u8; 2] {
        let g01 = unsafe {self.get_unchecked(2)};
        let g1 = g01 & Self::MAX;
        let g0 = g01 >> Self::BIT_SHIFT;
        [g0, g1]
    }
    #[inline] pub fn get_blue(&self) -> [u8; 2] {
        let b01 = unsafe {self.get_unchecked(3)};
        let b1 = b01 & Self::MAX;
        let b0 = b01 >> Self::BIT_SHIFT;
        [b0, b1]
    }
    #[inline] pub fn get_sun(&self) -> u8 {
        unsafe {self.get_unchecked(0) >> Self::BIT_SHIFT}
    }


    // #[inline(always)] unsafe fn set_unchecked(&self, value: u8, channel: usize) {
    //     unsafe {
    //         let s = &mut (*self.0.get());
    //         *s.get_unchecked_mut(channel) = value;
    //     };
    // }
    // #[inline(always)]
    // pub fn set(&self, value: u8, channel: usize) {
    //     unsafe {
    //         let s = &mut (*self.0.get());
    //         s[channel] = value;
    //     };
    // }
    #[inline] pub fn set_rgb(&self, r: u8, g: u8, b: u8) {
        self.set_light(Light::with_srgb(self.get_sun(), r, g, b));
    }
    // #[inline] pub fn set_red(&self, value: u8) {
    //     unsafe {self.set_unchecked(value, 0)};
    // }
    // #[inline] pub fn set_green(&self, value: u8) {
    //     unsafe {self.set_unchecked(value, 1)};
    // }
    // #[inline] pub fn set_blue(&self, value: u8) {
    //     unsafe {self.set_unchecked(value, 2)};
    // }
    // #[inline] pub fn set_sun(&self, value: u8) {
    //     unsafe {self.set_unchecked(value, 3)};
    // }
    #[inline] pub fn set_sun(&self, value: u8) {
        let v = unsafe {(&mut *self.0.get()).get_unchecked_mut(0)};
        *v = *v & 0x0_F | (value << Self::BIT_SHIFT);
    }
    /// Returns colors in RGBS order from 0.0 to 1.0.
    pub fn get_normalized(&self) -> [f32; 4] {
        let red = self.get_red().into_iter().zip_eq(Self::R.into_iter())
            .map(|(r, max_r)| r as f32 * max_r / Self::MAX as f32 / Self::MAX_COLOR)
            .sum::<f32>();
        let green = self.get_green().into_iter().zip_eq(Self::G.into_iter())
            .map(|(g, max_g)| g as f32 * max_g / Self::MAX as f32 / Self::MAX_COLOR)
            .sum::<f32>();
        let blue = self.get_blue().into_iter().zip_eq(Self::B.into_iter())
            .map(|(b, max_b)| b as f32 * max_b / Self::MAX as f32 / Self::MAX_COLOR)
            .sum::<f32>();
        [red, green, blue, self.get_sun() as f32 / Self::MAX as f32]
    }

    #[inline(always)]
    pub fn to_number(&self) -> u32 {
        unsafe {std::mem::transmute::<[u8; 4], u32>(self.array())}
    }

    #[inline(always)]
    pub fn array(&self) -> [u8; 4] {
        unsafe {*self.0.get()}
    }

    /// Returns an array with elements [s, r1, g0, b0, r0, r2, g1, b1]
    pub fn all_array(&self) -> [u8; 8] {
        let colors = self.to_number();
        let c1 =  colors & 0x0F0F0F0F;
        let c0 = (colors & 0xF0F0F0F0) >> Self::BIT_SHIFT;
        unsafe {std::mem::transmute::<_, [u8; 8]>([c0, c1])}
    }

    pub fn from_all_array(arr: [u8; 8]) -> Self {
        let arr = unsafe {std::mem::transmute::<[u8; 8], [u32; 2]>(arr)};
        unsafe {std::mem::transmute::<u32, Self>(arr[0] << Self::BIT_SHIFT | arr[1])}
    }

    pub fn set_light(&self, light: Light) {
        unsafe {
            let l = &mut (*self.0.get());
            *l = light.array();
        };
    }

    pub fn max_element_wise(&self, light: Light) -> Light {
        let result = Simd::<u8, 8>::from_array(self.all_array())
            .simd_max(light.all_array().into());
        Self::from_all_array(result.into())
    }

    pub fn saturated_sub_one(&self) -> Light {
        let result = Simd::<u8, 8>::from_array(self.all_array())
            .saturating_sub(Simd::splat(1));
        Self::from_all_array(result.into())
    }

    /// If all elements are less than or equal to one, returns true
    pub fn all_le_one(&self) -> bool {
        (self.to_number() & 0xEE_EE_EE_EE) == 0
    }

    /// a: [1, 2, 3, 4], b: [1, 3, 4, 4] -> [0, 2, 3, 0]
    pub fn zero_if_equal_elements(&self, light: Light) -> Light {
        let s = Simd::<u8, 8>::from_array(self.all_array());
        let mask = s.simd_eq(Simd::<u8, 8>::from(light.all_array()));
        let result = mask.select(Simd::splat(0), s);
        Self::from_all_array(result.into())
    }

    pub fn has_greater_element(&self, light: Light) -> bool {
        Simd::<u8, 8>::from_array(self.all_array())
            .simd_gt(Simd::<u8, 8>::from_array(light.all_array()))
            .any()
    }
}

unsafe impl Send for Light {}
unsafe impl Sync for Light {}


impl fmt::Debug for Light {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", unsafe {*self.0.get()})
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn check_sun() {
        let light = Light::with_sun(15);

        assert_eq!(light.get_sun(), 15);
    }

    #[test]
    fn check_creation() {
        let light = Light::with_rgb(15, 15, 15);

        assert_eq!(light.get(0), 15);
        assert_eq!(light.get(1), 255);
        assert_eq!(light.get(2), 255);
        assert_eq!(light.get(3), 255);

        let light = Light::with_sun(15);
        assert_eq!(light.get(0), 0xF0);
    }

    #[test]
    fn check_colors() {
        let light = Light::with_rgb(15, 15, 15);
        assert_eq!(light.get_red(), [15, 15, 15]);
        assert_eq!(light.get_green(), [15, 15]);
        assert_eq!(light.get_blue(), [15, 15]);

        let light = Light::with_rgb(0, 0, 0);
        assert_eq!(light.get_red(), [0, 0, 0]);
        assert_eq!(light.get_green(), [0, 0]);
        assert_eq!(light.get_blue(), [0, 0]);
    }

    #[test]
    fn check_normalization() {
        let light = Light::with_rgb(0, 0, 0);
        let n = light.get_normalized();
        assert!(n[0] < 0.000001 && n[1] < 0.000001 && n[2] < 0.000001 && n[3] < 0.000001);

        let light = Light::with_sun(15);
        let n = light.get_normalized();
        assert!(n[0] < 0.000001 && n[1] < 0.000001 && n[2] < 0.000001 && n[3] > 0.999999);
    }

    #[test]
    fn zero_if_equal_elements() {
        let light1 = Light::new(15, 14, 13, 12, 1, 2, 3, 4);
        let light2 = Light::new(15, 14, 10, 12, 4, 3, 2, 1);

        assert_eq!(light1.zero_if_equal_elements(light2),
            Light::new(0, 0, 13, 0, 1, 2, 3, 4));
    }

    #[test]
    fn check_conversion() {
        let light = Light::new(1, 2, 3, 4, 5, 6, 7, 8);

        assert_eq!(light.all_array(), [1u8, 3, 5, 7, 2, 4, 6, 8]);
        assert_eq!(Light::from_all_array([1u8, 3, 5, 7, 2, 4, 6, 8]), light);
    }
}