/// Creates a "SmallBox" for a given trait object.
///
/// # Example
/// ``` rust
/// nox_mem::smallbox!(
///     /// Supports doc comments
///     pub MySmallBox: Display
/// );
/// // No allocation
/// let hello = MySmallBox::<32>::new("hello".to_string());
/// // Heap allocation since size of String > 1
/// let world = MySmallBox::<1>::new("world".to_string());
/// assert_eq!(
///     format!("{}, {}", &*hello, &*world),
///     "hello, world"
/// );
#[macro_export]
macro_rules! smallbox {
    (
        $(#[doc = $doc:literal])*
        $vis:vis $name:ident: $bounds:ident
    ) => {

        $(#[doc = $doc])*
        $vis struct $name<const N_BUF: usize> {
            data: [u8; N_BUF],
            ptr: (*const (), *const ()),
            _marker: core::marker::PhantomData<dyn $bounds>,
        }

        impl<const N_BUF: usize> $name<N_BUF> {

            pub fn new<T: $bounds + 'static>(x: T) -> Self {
                if size_of_val(&x) <= N_BUF {
                    let mut data = [0u8; N_BUF];
                    let ptr = data
                        .as_mut_ptr()
                        .cast::<T>();
                    unsafe {
                        ptr.write(x);
                    }
                    let mut ptr = unsafe {
                        core::mem::transmute::<*mut (dyn $bounds), (*const (), *const ())>(
                            ptr
                        )
                    };
                    ptr.0 = core::ptr::null();
                    Self {
                        data,
                        ptr,
                        _marker: core::marker::PhantomData,
                    }
                } else {
                    let ptr = unsafe {
                        std::alloc::alloc(std::alloc::Layout::new::<T>())
                        .cast::<T>()
                    };
                    unsafe {
                        ptr.write(x);
                    }
                    let ptr = unsafe {
                        core::mem::transmute::<*mut (dyn $bounds), (*const (), *const ())>(ptr)
                    };
                    Self {
                        data: [0u8; N_BUF],
                        ptr,
                        _marker: core::marker::PhantomData,
                    }
                }
            }
        }

        impl<const N_BUF: usize> core::ops::Deref for $name<N_BUF> {

            type Target = dyn $bounds;

            fn deref(&self) -> &Self::Target {
                let mut ptr = self.ptr;
                if ptr.0.is_null() {
                    ptr.0 = self.data
                        .as_ptr()
                        .cast();
                    unsafe {
                        core::mem::transmute::<(*const (), *const ()), &dyn $bounds>(
                            ptr
                        )
                    }
                } else {
                    unsafe {
                        core::mem::transmute::<(*const (), *const()), &dyn $bounds>(
                            ptr
                        )
                    }
                }
            }
        }

        impl<const N_BUF: usize> core::ops::DerefMut for $name<N_BUF> {

            fn deref_mut(&mut self) -> &mut Self::Target {
                let mut ptr = self.ptr;
                if ptr.0.is_null() {
                    ptr.0 = self.data
                        .as_ptr()
                        .cast();
                    unsafe {
                        core::mem::transmute::<(*const (), *const ()), &mut dyn $bounds>(
                            ptr
                        )
                    }
                } else {
                    unsafe {
                        core::mem::transmute::<(*const (), *const()), &mut dyn $bounds>(
                            ptr
                        )
                    }
                }
            }
        }

        impl<const N_BUF: usize> Drop for $name<N_BUF> {

            fn drop(&mut self) {
                let mut ptr = self.ptr; 
                if ptr.0.is_null() {
                    ptr.0 = self.data
                        .as_ptr()
                        .cast();
                    unsafe {
                        let ptr = core::mem::transmute::<(*const (), *const ()), *mut dyn $bounds>(ptr);
                        ptr.drop_in_place();
                    }
               } else {
                    unsafe {
                        let ptr = core::mem::transmute::<(*const (), *const ()), *mut dyn $bounds>(ptr);
                        let (size, align) = {
                            let x = ptr.as_ref().unwrap_unchecked();
                            (size_of_val(x), align_of_val(x))
                        };
                        ptr.drop_in_place();
                        std::alloc::dealloc(ptr.cast(), std::alloc::Layout::from_size_align(
                            size, align,
                        ).unwrap());
                    }
               }
            }
        }
    };
}
