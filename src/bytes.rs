pub enum ByteSize {
    Fixed(u32),
    Dynamic
}

pub trait ConstByteInterpretation: where Self: Sized {
    fn to_bytes(&self) -> Box<[u8]>;
    fn from_bytes(data: &[u8]) -> Self;

    fn size(&self) -> u32;
}

pub trait DynByteInterpretation {
    fn to_bytes(&self) -> Box<[u8]>;
    fn from_bytes(data: &[u8]) -> Self;
}

pub trait NumFromBytes {
    fn from_bytes(data: &[u8]) -> Self;
}

impl NumFromBytes for i32 {
    fn from_bytes(data: &[u8]) -> Self {
        assert!(data.len() == 4);
        unsafe {
            i32::from_le_bytes([
                *data.get_unchecked(0),
                *data.get_unchecked(1),
                *data.get_unchecked(2),
                *data.get_unchecked(3)])
        }
    }
}

impl NumFromBytes for u32 {
    fn from_bytes(data: &[u8]) -> Self {
        assert!(data.len() == 4);
        unsafe {
            u32::from_le_bytes([
                *data.get_unchecked(0),
                *data.get_unchecked(1),
                *data.get_unchecked(2),
                *data.get_unchecked(3)])
        }
    }
}

impl NumFromBytes for u64 {
    fn from_bytes(data: &[u8]) -> Self {
        assert!(data.len() == 8);
        unsafe {
            u64::from_le_bytes([
                *data.get_unchecked(0),
                *data.get_unchecked(1),
                *data.get_unchecked(2),
                *data.get_unchecked(3),
                *data.get_unchecked(4),
                *data.get_unchecked(5),
                *data.get_unchecked(6),
                *data.get_unchecked(7)])
        }
    }
}

/// I'm not sure if I should use this function
pub unsafe fn any_as_u8_slice<T: Sized>(p: &T) -> &[u8] {
    ::core::slice::from_raw_parts(
        (p as *const T) as *const u8,
        ::core::mem::size_of::<T>(),
    )
}