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

                pub struct $ident(());

                static INSTANCE: std::sync::OnceLock<$ident> = std::sync::OnceLock::new();

                impl $ident {

                    pub fn new() -> Option<Self> {
                        if INSTANCE.get().is_none()
                        {
                            INSTANCE.get_or_init(|| $ident(()));
                            Some($ident(()))
                        } else {
                            None
                        }
                    }
                }

                unsafe impl $crate::cell::CellToken for $ident {

                    type Identifier = ();

                    fn identifier(&self) -> Self::Identifier {
                        ()
                    }

                    fn validate(&self, id: &Self::Identifier) {}
                }
            }
            $vis use [<__ $ident Mod __>]::$ident;
        );
    };
}
