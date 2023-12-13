/// Only for constant size types,
/// Only for repr(C)
pub trait AsFromBytes: Sized + Clone {
    const SIZE: usize = std::mem::size_of::<Self>();
    #[inline(always)]
    fn as_bytes(&self) -> &[u8] {
        let slf: *const Self = self;
        unsafe { std::slice::from_raw_parts(slf.cast::<u8>(), Self::SIZE) }
    }

    #[inline]
    fn from_bytes(bytes: &[u8]) -> Self {
        assert_eq!(bytes.len(), Self::SIZE);
        let ptr = bytes.as_ptr() as *const Self;
        unsafe {ptr.as_ref()}.unwrap().clone()
    }

    #[inline(always)]
    fn size() -> usize {Self::SIZE}
}

#[inline]
pub fn cast_bytes_from_vec<T>(data: &Vec<T>) -> &[u8] {
    let slice = &data[..];
    let slf: *const T = slice.as_ptr();
    let len = std::mem::size_of_val(slice);
    unsafe { std::slice::from_raw_parts(slf.cast::<u8>(), len) }
}

#[inline]
pub fn cast_vec_from_bytes<T: Clone>(bytes: &[u8]) -> Vec<T> {
    let ptr = bytes.as_ptr() as *const T;
    let len = bytes.len() / std::mem::size_of::<T>();
    let temp_slice = unsafe { std::slice::from_raw_parts(ptr, len) };
    temp_slice.to_vec()
}

impl AsFromBytes for u8 {}
impl AsFromBytes for u16 {}
impl AsFromBytes for u32 {}
impl AsFromBytes for u64 {}

impl AsFromBytes for i8 {}
impl AsFromBytes for i16 {}
impl AsFromBytes for i32 {}
impl AsFromBytes for i64 {}

impl AsFromBytes for f32 {}
impl AsFromBytes for f64 {}

impl<const N: usize> AsFromBytes for [u32; N] {}

pub trait BytesCoder: Sized {
    fn encode_bytes(&self) -> Box<[u8]>;
    fn decode_bytes(bytes: &[u8]) -> Self;
}