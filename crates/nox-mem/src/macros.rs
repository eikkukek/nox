#[macro_export]
macro_rules! const_assert {
    ($check:expr $(,$msg:tt)*) => {
        const _: () = assert!($check $(,$msg)*);
    };
}

#[macro_export]
macro_rules! size_of {
    ($t:ty) => {
        size_of::<$t>()
    };
}

#[macro_export]
macro_rules! align_of {
    ($t:ty) => {
        align_of::<$t>()
    };
}

#[macro_export]
macro_rules! slice {
    ($v:expr; $n:expr) =>(
        [$v; $n].as_slice()
    );
    [$($elem:expr),* $(,)?] => {
        [$($elem),*].as_slice()
    };
}

#[macro_export]
macro_rules! impl_traits {
    (
        for $type:ident $(<$($gen:tt $(: $bounds:tt)? $([$gen_q:ident])?),*>)?
        $trait_this:ident $(<$($trg_this:ty),+>)? $(for $(&$lifetime_this:lifetime)? $(mut &$lifetime_mut_this:lifetime)?)?
                $(where $($trbl_this:ty: $trbr_this:tt$(<$trbg_this:ty>)?),+)? =>
            $(type $stype_this:ident = $sty_this:ty;)*
            $(
                $(#[$macro_this:ident $(($macro_spec_this:ident))?])*
                fn $met_this:ident$(<$($met_g_this:tt $(: $met_gb_this:tt)?),*>)?($($arg_this:tt)*) $(-> $ret_this:ty)? $body_this:block
            )*
        ,
        $($trait:ident $(<$($trg:ty),+>)? $(for $(&$lifetime:lifetime)? $(mut &$lifetime_mut:lifetime)?)?
                $(where $($trbl:ty: $trbr:tt$(<$trbg:ty>)?),+)? =>
            $(type $stype:ident = $sty:ty;)*
            $(
                $(#[$macro:ident $(($macro_spec:ident))?])*
                fn $met:ident$(<$($met_g:tt $(: $met_gb:tt)?),*>)?($($arg:tt)*) $(-> $ret:ty)? $body:block
            )*
        ),*
        $(,)?
    ) =>
    {
        impl<$($($lifetime_this,)? $($lifetime_mut_this,)?)? $($($($gen_q)? $gen $(: $bounds)?),*)?> $trait_this $(<$($trg_this),+>)?
                for $($(&$lifetime_this)? $(&$lifetime_mut_this mut)?)? $type<$($($gen),*)?>
            $(
                where
                    $($trbl_this: $trbr_this$(<$trbg_this>)?),+
            )?
        {

            $(
                type $stype_this = $sty_this;
            )*

            $(
                $(#[$macro_this $(($macro_spec_this))?])*
                fn $met_this$(<$($met_g_this $(: $met_gb_this)?),*>)?($($arg_this)*) $(-> $ret_this)? $body_this
            )*
        }
        impl_traits! {
            for $type $(<$($gen $(: $bounds)? $([$gen_q])?),*>)?
            $($trait $(<$($trg),+>)? $(for $(&$lifetime)? $(mut &$lifetime_mut)?)? $(where $($trbl: $trbr$(<$trbg>)?),+)? =>
                $(type $stype = $sty;)*
                $(
                    $(#[$macro $(($macro_spec))?])*
                    fn $met$(<$($met_g $(: $met_gb)?),*>)?($($arg)*) $(-> $ret)? $body
                )*
            ),*
            ,
        }
    };
    (
        for $type:ident $(<$($gen:tt $(: $bounds:tt)? $([$gen_q:ident])?),*>)?
        ,
    ) =>
    {
    };
}

#[macro_export]
macro_rules! impl_inherent {
    (
        $type:ident $(<$($gen:tt $(: $bounds:tt)? $([$gen_q:ident])?),*>)? {
            $(type $stype:ident = $sty:ty;)*
            $(
                $(#[$macro_this:ident $(($macro_spec_this:ident))?])*
                $([$($qual:ident)+])? fn $fn:ident$(<$($t:tt),+>)?($($arg_this:tt)*) $(-> $ret_this:ty)?
                    $body_this:block
            )*
        }
    ) => {
        impl<$($($($gen_q)? $gen $(: $bounds)?),*)?> $type<$($($gen),*)?> {

            $(
                type $stype = $sty;
            )*

            $(
                $(#[$macro_this $(($macro_spec_this))?])*
                $($($qual)+)? fn $fn $(<$($t),+>)? ($($arg_this)*) $(-> $ret_this)?
                    $body_this
            )*
        }
    };
}
