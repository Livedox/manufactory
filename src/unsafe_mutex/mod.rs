use std::{cell::UnsafeCell, sync::atomic::AtomicU32, ops::{Deref, DerefMut}, fmt};
use core::sync::atomic::Ordering::{Relaxed, Release, Acquire};
use atomic_wait::{wait, wake_one, wake_all};

mod poison;
use poison::{TryLockError, LockResult, TryLockResult};

const WAIT: u32 = 0b01000000_00000000_00000000_00000000;//1_073_741_824 dec
const LOCK: u32 = 0b10000000_00000000_00000000_00000000;//2_147_483_648 dec


pub struct UnsafeMutex<T: ?Sized> {
    pub state: AtomicU32,
    wake_counter: AtomicU32,
    poison: poison::Flag,
    data: UnsafeCell<T>,
}

impl<T> UnsafeMutex<T> {
    pub fn new(t: T) -> UnsafeMutex<T> {
        UnsafeMutex {
            state: AtomicU32::new(0),
            wake_counter: AtomicU32::new(0),
            poison: poison::Flag::new(),
            data: UnsafeCell::new(t),
        }
    }
}

impl<T: ?Sized> UnsafeMutex<T> {
    /// If immediately = false, this can sometimes work as immediately true.
    pub fn lock_unsafe(&self, immediately: bool) -> LockResult<UnsafeGuard<T>> {
        let mut state = self.state.load(Relaxed);
        loop {
            if state < WAIT || (immediately && state < LOCK) {
                // There may or may not be bugs here
                // I'm too dumb to figure it out.
                let current = self.state.load(Acquire);
                match (current < LOCK, self.state.compare_exchange_weak(current, current+1, Acquire, Relaxed)) {
                    (true, Ok(_)) => {
                        assert_ne!((current+1)&(WAIT-1), WAIT-1, "Too many locks");
                        return unsafe { UnsafeGuard::new(self) };
                    },
                    (false, Ok(_)) => {state = current; continue;},
                    (_, Err(s)) => {state = s; continue;},
                }
            }
            wait(&self.state, state);
            state = self.state.load(Relaxed);
        }
    }


    pub fn lock(&self) -> LockResult<SafeGuard<T>> {
        let mut state = self.state.load(Relaxed);
        loop {
            if state < WAIT {
                match self.state.compare_exchange_weak(
                    state, state|WAIT, Acquire, Relaxed
                ) {
                    Ok(_) => {},
                    Err(s) => {state = s; continue;},
                }
            }
            if state == WAIT {
                match self.state.compare_exchange_weak(
                    WAIT, LOCK, Acquire, Relaxed
                ) {
                    Ok(_) => return unsafe { SafeGuard::new(self) },
                    Err(s) => {state = s; continue;},
                }; 
            }
            let c = self.wake_counter.load(Acquire);
            state = self.state.load(Relaxed);
            if state != WAIT {
                wait(&self.wake_counter, c);
                state = self.state.load(Relaxed);
            }
        }
    }


    pub fn try_lock(&self) -> TryLockResult<SafeGuard<T>> {
        let state = self.state.load(Relaxed);
        if state < WAIT {
            let _ = self.state.compare_exchange(
                state, state|WAIT, Acquire, Relaxed
            );
        }
        match self.state.compare_exchange(
            WAIT, LOCK, Acquire, Relaxed
        ) {
            Ok(_) => Ok(unsafe { SafeGuard::new(self) }?),
            Err(_) => {Err(TryLockError::WouldBlock)},
        }
    }

    #[inline]
    pub fn is_poisoned(&self) -> bool {
        self.poison.get()
    }

    #[inline]
    pub fn clear_poison(&self) {
        self.poison.clear();
    }

    pub fn into_inner(self) -> LockResult<T>
    where
        T: Sized,
    {
        let data = self.data.into_inner();
        poison::map_result(self.poison.borrow(), |()| data)
    }

    pub fn get_mut(&mut self) -> LockResult<&mut T> {
        let data = self.data.get_mut();
        poison::map_result(self.poison.borrow(), |()| data)
    }
}


unsafe impl<T: ?Sized + Send> Sync for UnsafeMutex<T> {}
unsafe impl<T: ?Sized + Send> Send for UnsafeMutex<T> {}


impl<T> From<T> for UnsafeMutex<T> {
    /// Creates a new mutex in an unlocked state ready for use.
    /// This is equivalent to [`Mutex::new`].
    fn from(t: T) -> Self {
        UnsafeMutex::new(t)
    }
}


impl<T: ?Sized + Default> Default for UnsafeMutex<T> {
    /// Creates a `Mutex<T>`, with the `Default` value for T.
    fn default() -> UnsafeMutex<T> {
        UnsafeMutex::new(Default::default())
    }
}


impl<T: ?Sized + fmt::Debug> fmt::Debug for UnsafeMutex<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut d = f.debug_struct("Mutex");
        match self.try_lock() {
            Ok(guard) => {
                d.field("data", &&*guard);
            }
            Err(TryLockError::Poisoned(err)) => {
                d.field("data", &&**err.get_ref());
            }
            Err(TryLockError::WouldBlock) => {
                d.field("data", &format_args!("<locked>"));
            }
        }
        d.field("poisoned", &self.poison.get());
        d.finish_non_exhaustive()
    }
}


pub struct UnsafeGuard<'a, T: ?Sized + 'a> {
    mutex: &'a UnsafeMutex<T>,
    poison: poison::Guard,
}

impl<'a, T: ?Sized> UnsafeGuard<'a, T> {
    unsafe fn new(mutex: &'a UnsafeMutex<T>) -> LockResult<UnsafeGuard<'a, T>> {
        poison::map_result(mutex.poison.guard(), |guard| UnsafeGuard {
            mutex, poison: guard
        })
    }
}

impl<T: ?Sized> Deref for UnsafeGuard<'_, T> {
    type Target = T;
    fn deref(&self) -> &T {
        unsafe { &*self.mutex.data.get() }
    }
}

impl<T: ?Sized> DerefMut for UnsafeGuard<'_, T> {
    fn deref_mut(&mut self) -> &mut T {
        unsafe { &mut *self.mutex.data.get() }
    }
}

impl<T: ?Sized> Drop for UnsafeGuard<'_, T> {
    fn drop(&mut self) {
        self.mutex.poison.done(&self.poison);
        let state = self.mutex.state.fetch_sub(1, Release);
        if state == 1 || state == WAIT+1 {
            self.mutex.wake_counter.fetch_add(1, Release);
            wake_one(&self.mutex.wake_counter);
        }
    }
}

impl<T: ?Sized + fmt::Debug> fmt::Debug for UnsafeGuard<'_, T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(&**self, f)
    }
}

pub struct SafeGuard<'a, T: ?Sized + 'a> {
    mutex: &'a UnsafeMutex<T>,
    poison: poison::Guard,
}

impl<'a, T: ?Sized> SafeGuard<'a, T> {
    unsafe fn new(mutex: &'a UnsafeMutex<T>) -> LockResult<SafeGuard<'a, T>> {
        poison::map_result(mutex.poison.guard(), |guard| SafeGuard {
            mutex, poison: guard
        })
    }
}

impl<T: ?Sized> Deref for SafeGuard<'_, T> {
    type Target = T;
    fn deref(&self) -> &T {
        unsafe { &*self.mutex.data.get() }
    }
}

impl<T: ?Sized> DerefMut for SafeGuard<'_, T> {
    fn deref_mut(&mut self) -> &mut T {
        unsafe { &mut *self.mutex.data.get() }
    }
}

impl<T: ?Sized> Drop for SafeGuard<'_, T> {
    fn drop(&mut self) {
        self.mutex.poison.done(&self.poison);
        self.mutex.state.store(0, Release);
        self.mutex.wake_counter.fetch_add(1, Release);
        wake_one(&self.mutex.wake_counter);
        wake_all(&self.mutex.state);
    }
}

impl<T: ?Sized + fmt::Debug> fmt::Debug for SafeGuard<'_, T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(&**self, f)
    }
}