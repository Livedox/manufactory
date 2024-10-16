use std::arch::asm;

#[cfg(all(any(target_arch = "x86", target_arch = "x86_64")))]
#[inline]
// Work by default on "rustc default"
pub fn max_element_wise(a: [u8; 8], b: [u8; 8]) -> [u8; 8] {
    let mut a = unsafe {std::mem::transmute::<[u8; 8], u64>(a)};
    let b = unsafe {std::mem::transmute::<[u8; 8], u64>(b)};
    unsafe {
        asm!(
            "movd    xmm0, {a}",
            "movd    xmm1, {b}",
            "pmaxub  xmm1, xmm0",
            "movd    {a}, xmm1",
            a = inout(reg) a,
            b = in(reg) b,
        );
    }
    unsafe {std::mem::transmute::<u64, [u8; 8]>(a)}
}

#[cfg(not(all(any(target_arch = "x86", target_arch = "x86_64"))))]
pub fn max_element_wise(a: [u8; 8], b: [u8; 8]) -> [u8; 8] {
    [a[0].max(b[0]),
    a[1].max(b[1]),
    a[2].max(b[2]),
    a[3].max(b[3]),
    a[4].max(b[4]),
    a[5].max(b[5]),
    a[6].max(b[6]),
    a[7].max(b[7])]
}

#[cfg(all(any(target_arch = "x86", target_arch = "x86_64")))]
#[inline]
// Work by default on "rustc default"
pub fn saturated_sub_one(a: [u8; 8]) -> [u8; 8] {
    let mut a = unsafe {std::mem::transmute::<[u8; 8], u64>(a)};
    let b: u64 = 0x01_01_01_01_01_01_01_01; // [1u8, 1u8, 1u8, 1u8, 1u8, 1u8, 1u8, 1u8];
    unsafe {
        asm!(
            "movd    xmm0, {a}",
            "movd    xmm1, {b}",
            "psubusb xmm0, xmm1",
            "movd    {a}, xmm0",
            a = inout(reg) a,
            b = in(reg) b,
        );
    }
    unsafe {std::mem::transmute::<u64, [u8; 8]>(a)}
}

#[cfg(not(all(any(target_arch = "x86", target_arch = "x86_64"))))]
pub fn saturated_sub_one(mut a: [u8; 8]) -> [u8; 8] {
    for i in 0..8 {a[i].saturating_sub(1)}
    a
}

#[cfg(all(any(target_arch = "x86", target_arch = "x86_64")))]
pub fn zero_if_equal_elements(a: [u8; 8], b: [u8; 8]) -> [u8; 8] {
    let mut a = unsafe {std::mem::transmute::<[u8; 8], u64>(a)};
    let b = unsafe {std::mem::transmute::<[u8; 8], u64>(b)};
    unsafe {
        asm!(
            "movd    xmm0, {a}",
            "movd    xmm1, {b}",
            "pcmpeqb xmm1, xmm0",
            "pandn   xmm1, xmm0",
            "movd   {a}, xmm1",
            a = inout(reg) a,
            b = in(reg) b,
        )
    }
    unsafe {std::mem::transmute::<u64, [u8; 8]>(a)}
}

#[cfg(not(all(any(target_arch = "x86", target_arch = "x86_64"))))]
pub fn zero_if_equal_elements(mut a: [u8; 8], b: [u8; 8]) -> [u8; 8] {
    for i in 0..8 {if a[i] == b[i] {a[i] = 0;}}
    a
}

#[cfg(all(any(target_arch = "x86", target_arch = "x86_64")))]
pub fn has_greater_element(a: [u8; 8], b: [u8; 8]) -> bool {
    let mut a = unsafe {std::mem::transmute::<[u8; 8], u64>(a)};
    let b = unsafe {std::mem::transmute::<[u8; 8], u64>(b)};
    unsafe {
        asm!(
            "movd    xmm0, {b}",
            "movd    xmm1, {a}",
            "pcmpgtb xmm1, xmm0",
            "movd   {a}, xmm1",
            a = inout(reg) a,
            b = in(reg) b,
        )
    }
    a != 0
}

#[cfg(not(all(any(target_arch = "x86", target_arch = "x86_64"))))]
pub fn has_greater_element(a: [u8; 8], b: [u8; 8]) -> bool {
    for i in 0..8 {if a[i] > b[i] {return true;}}
    false
}
