use crate::{
    capacity_policy::{CapacityPolicy, Dyn}, conditional::{Conditional, True}, global_alloc::{GlobalAlloc, GLOBAL_ALLOC}, AllocVec
};

pub type GlobalVec<T> = super::alloc_vec::AllocVec<'static, T, GlobalAlloc, Dyn, True>;

impl<T> GlobalVec<T> {

    pub fn new() -> Self {
        Self {
            data: NonNull::dangling(),
            capacity: 0,
            len: 0,
            alloc: OptionAlloc::Some(&GLOBAL_ALLOC),
            _markers: PhantomData,
        }
    }

    pub fn with_capacity(
        capacity: usize,
    ) -> Result<Self, CapacityError> {
        if capacity == 0 {
            return Err(InvalidReservation {
                current: 0, requested: 0
            })
        }
        let true_capacity =
            if <Self as Vector<T>>::CapacityPol::power_of_two() {
                capacity.next_power_of_two()
            }
            else {
                capacity
            };
        let data = unsafe { GLOBAL_ALLOC
            .allocate_uninit(true_capacity)
            .ok_or_else(|| {
                if size_of::<T>() == 0 {
                    ZeroSizedElement
                }
                else {
                    AllocFailed { new_capacity: true_capacity }
                }
            })?
        };
        Ok(Self {
            data,
            capacity: true_capacity,
            len: 0,
            alloc: OptionAlloc::Some(&GLOBAL_ALLOC),
            _markers: PhantomData,
        })
    }

    pub fn with_len(
        len: usize,
        value: T,
    ) -> Result<Self, CapacityError>
        where
            T: Clone
    {
        if len == 0 {
            return Err(InvalidReservation {
                current: 0, requested: 0
            })
        }
        let capacity =
            if <Self as Vector<T>>::CapacityPol::power_of_two() {
                len.next_power_of_two()
            }
            else {
                len
            };
        let data = unsafe { GLOBAL_ALLOC
            .allocate_uninit(capacity)
            .ok_or_else(|| {
                if size_of::<T>() == 0 {
                    ZeroSizedElement
                }
                else {
                    AllocFailed { new_capacity: capacity }
                }
            })?
        };
        for i in 0..len {
            unsafe { data.add(i).write(value.clone()) };
        }
        Ok(Self {
            data,
            capacity,
            len,
            alloc: OptionAlloc::Some(&GLOBAL_ALLOC),
            _markers: PhantomData,
        })
    }

    pub fn with_len_with<F>(
        len: usize,
        mut f: F,
    ) -> Result<Self, CapacityError>
        where
            F: FnMut() -> T,
    {
        if len == 0 {
            return Err(InvalidReservation {
                current: 0, requested: 0
            })
        }
        let capacity =
            if <Self as Vector<T>>::CapacityPol::power_of_two() {
                len.next_power_of_two()
            }
            else {
                len
            };
        let data = unsafe { GLOBAL_ALLOC
            .allocate_uninit(capacity)
            .ok_or_else(|| {
                if size_of::<T>() == 0 {
                    ZeroSizedElement
                }
                else {
                    AllocFailed { new_capacity: capacity }
                }
            })?
        };
        for i in 0..len {
            unsafe { data.add(i).write(f()) };
        }
        Ok(Self {
            data,
            capacity,
            len,
            alloc: OptionAlloc::Some(&GLOBAL_ALLOC),
            _markers: PhantomData,
        })
    }
}
