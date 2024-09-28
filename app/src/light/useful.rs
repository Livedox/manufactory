use std::arch::asm;

#[cfg(all(any(target_arch = "x86", target_arch = "x86_64")))]
#[inline]
// Work by default on "rustc default"
pub fn max_element_wise(a: [u8; 4], b: [u8; 4]) -> [u8; 4] {
    let mut a = unsafe {std::mem::transmute::<[u8; 4], u32>(a)};
    let b = unsafe {std::mem::transmute::<[u8; 4], u32>(b)};
    unsafe {
        asm!(
            "movd    xmm0, {a:e}",
            "movd    xmm1, {b:e}",
            "pmaxub  xmm1, xmm0",
            "movd    {a:e}, xmm1",
            a = inout(reg) a,
            b = in(reg) b,
        );
    }
    unsafe {std::mem::transmute::<u32, [u8; 4]>(a)}
}

#[cfg(not(all(any(target_arch = "x86", target_arch = "x86_64"))))]
pub fn max_element_wise(a: [u8; 4], b: [u8; 4]) -> [u8; 4] {
    [a[0].max(b[0]),
    a[1].max(b[1]),
    a[2].max(b[2]),
    a[3].max(b[3])]
}

#[cfg(all(any(target_arch = "x86", target_arch = "x86_64")))]
#[inline]
// Work by default on "rustc default"
pub fn saturated_sub_one(a: [u8; 4]) -> [u8; 4] {
    let mut a = unsafe {std::mem::transmute::<[u8; 4], u32>(a)};
    let b: u32 = 0x1_01_01_01; // [1u8, 1u8, 1u8, 1u8];
    unsafe {
        asm!(
            "movd    xmm0, {a:e}",
            "movd    xmm1, {b:e}",
            "psubusb xmm0, xmm1",
            "movd    {a:e}, xmm0",
            a = inout(reg) a,
            b = in(reg) b,
        );
    }
    unsafe {std::mem::transmute::<u32, [u8; 4]>(a)}
}

#[cfg(not(all(any(target_arch = "x86", target_arch = "x86_64"))))]
pub fn saturated_sub_one(a: [u8; 4]) -> [u8; 4] {
    [a[0].saturating_sub(1),
    a[1].saturating_sub(1),
    a[2].saturating_sub(1),
    a[3].saturating_sub(1)]
}

#[cfg(all(any(target_arch = "x86", target_arch = "x86_64")))]
pub fn zero_if_equal_elements(a: [u8; 4], b: [u8; 4]) -> [u8; 4] {
    let mut a = unsafe {std::mem::transmute::<[u8; 4], u32>(a)};
    let b = unsafe {std::mem::transmute::<[u8; 4], u32>(b)};
    unsafe {
        asm!(
            "movd    xmm0, {a:e}",
            "movd    xmm1, {b:e}",
            "pcmpeqb xmm1, xmm0",
            "pandn   xmm1, xmm0",
            "movd   {a:e}, xmm1",
            a = inout(reg) a,
            b = in(reg) b,
        )
    }
    unsafe {std::mem::transmute::<u32, [u8; 4]>(a)}
}

#[cfg(not(all(any(target_arch = "x86", target_arch = "x86_64"))))]
pub fn zero_if_equal_elements(mut a: [u8; 4], b: [u8; 4]) -> [u8; 4] {
    for i in 0..4 {if a[i] == b[i] {a[i] = 0;}}
    a
}

#[cfg(all(any(target_arch = "x86", target_arch = "x86_64")))]
pub fn has_greater_element(a: [u8; 4], b: [u8; 4]) -> bool {
    let mut a = unsafe {std::mem::transmute::<[u8; 4], u32>(a)};
    let b = unsafe {std::mem::transmute::<[u8; 4], u32>(b)};
    unsafe {
        asm!(
            "movd    xmm0, {b:e}",
            "movd    xmm1, {a:e}",
            "pcmpgtb xmm1, xmm0",
            "movd   {a:e}, xmm1",
            a = inout(reg) a,
            b = in(reg) b,
        )
    }
    a != 0
}

#[cfg(not(all(any(target_arch = "x86", target_arch = "x86_64"))))]
pub fn has_greater_element(a: [u8; 4], b: [u8; 4]) -> bool {
    for i in 0..4 {if a[i] > b[i] {return true;}}
    false
}
