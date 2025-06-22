#[macro_export]
macro_rules! impl_traits {
    (
        for $type:ident $(<$($gen:tt $(: $bounds:tt)? $([$gen_q:ident])?),*>)?
        $trait_this:ident $(<$($trt_this:tt),*>)? $(&$lifetime_this:tt $($mut_this:ident)?)? =>
            $(type $stype_this:ident = $sty_this:ty;)*
            $(
                $(#[$macro_this:ident $(($macro_spec_this:ident))?])*
                fn $met_this:ident($($arg_this:tt)*) -> $ret_this:ty $body_this:block
            )*
        ,
        $($trait:ident $(<$($trt:tt),*>)? $(&$lifetime:tt $($mut:ident)?)? =>
            $(type $stype:ident = $sty:ty;)*
            $(
                $(#[$macro:ident $(($macro_spec:ident))?])*
                fn $met:ident($($arg:tt)*) -> $ret:ty $body:block
            )*
        ),*
        $(,)?
    ) =>
    {
        impl<$($lifetime_this,)? $($($($gen_q)? $gen $(: $bounds)?),*)?> $trait_this $(<$($trt_this),*>)? for $(&$lifetime_this $($mut_this)?)? $type<$($($gen),*)?> {

            $(
                type $stype_this = $sty_this;
            )*

            $(
                $(#[$macro_this $(($macro_spec_this))?])*
                fn $met_this($($arg_this)*) -> $ret_this $body_this
            )*
        }
        crate::impl_traits! {
            for $type $(<$($gen $(: $bounds)? $([$gen_q])?),*>)?
            $($trait $(<$($trt),*>)? $(&$lifetime $($mut)?)? =>
                $(type $stype = $sty;)*
                $(
                    $(#[$macro $(($macro_spec))?])*
                    fn $met($($arg)*) -> $ret $body
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
