mod init_cell;

pub unsafe trait CellToken {

    type Identifier;

    fn identifier(&self) -> Self::Identifier;

    fn validate(&self, id: &Self::Identifier);
}

pub use init_cell::InitCell;

#[macro_export]
macro_rules! singleton_cell_token {
    ($vis:vis $ident:ident) => {
        $crate::paste!(
            #[allow(non_snake_case)]
            mod [<__ $ident Mod __>] {

                use core::sync::atomic;

                pub struct $ident(());

                static LOCK: atomic::AtomicBool = atomic::AtomicBool::new(false);

                impl $ident {

                    pub fn new() -> Option<Self> {
                        if LOCK.load(atomic::Ordering::Relaxed) {
                            None
                        } else {
                            LOCK.store(true, atomic::Ordering::Relaxed);
                            Some($ident(()))
                        }
                    }
                }

                unsafe impl $crate::cell::CellToken for $ident {

                    type Identifier = ();

                    #[inline(always)]
                    fn identifier(&self) -> Self::Identifier {
                        ()
                    }
                    
                    #[inline(always)]
                    fn validate(&self, id: &Self::Identifier) {}
                }

                impl Drop for $ident {

                    fn drop(&mut self) {
                        LOCK.store(false, atomic::Ordering::Relaxed);
                    }
                }
            }
            $vis use [<__ $ident Mod __>]::$ident;
        );
    };
}
