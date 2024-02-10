#[macro_export]
/// If the value is Ok or Some,
/// then the Result or Option will be returned from the entire calling function;
/// otherwise, it does nothing.
macro_rules! rev_qumark {
    ( $val:expr ) => {
        if $val.map_or(false, |_| true) {
            return $val;
        }
    };
}

#[macro_export]
macro_rules! vec_none {
    ( $len:expr ) => {
        {
            let mut vec = Vec::new();
            vec.resize_with($len, || None);
            vec
        }
    };
}


/// Do this:
/// ```rust, ignore
/// fn player_unlockable(&self) -> Option<Weak<Mutex<dyn PlayerUnlockable>>> {
///     let tmp: Arc<Mutex<dyn PlayerUnlockable>> = self.clone();
///     Some(Arc::downgrade(&tmp))
/// }
/// ```
#[macro_export]
macro_rules! player_unlockable {
    () => {
        fn player_unlockable(&self) -> Option<Weak<Mutex<dyn PlayerUnlockable>>> {
            let tmp: Arc<Mutex<dyn PlayerUnlockable>> = self.clone();
            Some(Arc::downgrade(&tmp))
        }
    };
}

/// Do this:
/// ```rust, ignore
/// impl LiveVoxelDesiarialize for $type {
///     fn deserialize(bytes: &[u8]) -> Box<dyn LiveVoxel> {
///         Box::new(bincode::deserialize::<Self>(bytes)
///             .expect(concat!("Deserialization error on type: ", stringify!($type))))
///         }
///     }
/// }
/// ```
#[macro_export]
macro_rules! live_voxel_default_deserialize {
    ( $type:ty ) => {
        impl LiveVoxelDesiarialize for $type {
            fn deserialize(bytes: &[u8]) -> Box<dyn LiveVoxel> {
                Box::new(bincode::deserialize::<Self>(bytes)
                    .expect(concat!("Deserialization error on type: ", stringify!($type))))
            }
        }
    };
}