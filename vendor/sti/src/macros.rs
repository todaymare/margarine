
#[macro_export]
macro_rules! static_assert {
    ($cond: expr) => {
        const _: () = assert!($cond);
    };
}

#[macro_export]
macro_rules! static_assert_eq {
    ($a: expr, $b: expr) => {
        const _: () = assert!($a == $b);
    };
}

#[macro_export]
macro_rules! static_assert_ne {
    ($a: expr, $b: expr) => {
        const _: () = assert!($a != $b);
    };
}


#[macro_export]
macro_rules! assert_anon {
    ($cond:expr) => {
        if !($cond) { $crate::os::abort() }
    };
}


#[macro_export]
macro_rules! fmt {
    ($($arg:tt)*) => {
        ::core::format_args!($($arg)*)
    }
}

#[macro_export]
macro_rules! write {
    ($dst:expr, $($arg:tt)*) => {{
        _ = ::core::fmt::Write::write_fmt($dst, ::core::format_args!($($arg)*));
    }};
}

#[macro_export]
macro_rules! write_checked {
    ($dst:expr, $($arg:tt)*) => {
        ::core::fmt::Write::write_fmt($dst, ::core::format_args!($($arg)*))
    };
}

#[macro_export]
macro_rules! format {
    ($($arg:tt)*) => {
        $crate::string::String::from_fmt(::core::format_args!($($arg)*))
    }
}

#[macro_export]
macro_rules! format_in {
    ($alloc:expr, $($arg:tt)*) => {
        $crate::string::String::from_fmt_in($alloc, ::core::format_args!($($arg)*))
    }
}


#[macro_export]
macro_rules! dbg {
    () => {
        $crate::println!("[{}:{}:{}]", $crate::file!(), $crate::line!(), $crate::column!())
    };

    ($val:expr $(,)?) => {
        // Use of `match` here is intentional because it affects the lifetimes
        // of temporaries - https://stackoverflow.com/a/48732525/1063961
        match $val {
            tmp => {
                $crate::println!("[{}:{}:{}] {} = {:#?}",
                    $crate::file!(), $crate::line!(), $crate::column!(), $crate::stringify!($val), &tmp);
                tmp
            }
        }
    };

    ($($val:expr),+ $(,)?) => {
        ($($crate::dbg!($val)),+,)
    };
}


#[macro_export]
macro_rules! assume {
    ($cond: expr) => {
        if !($cond) {
            $crate::hint::unreachable_unchecked();
        }
    };
}

/// erase lifetimes in `x: T`
///
/// this is safer than using transmute directly
/// as you're forced to specify the type
/// and you don't have to repeat it.
#[macro_export]
macro_rules! erase {
    ($T:ty, $x:expr) => {
        $crate::mem::transmute::<$T, $T>($x)
    };
}


// @todo: move this somewhere else.
#[macro_export]
macro_rules! define_bit_flags {
    ($name:ident($vis:vis $ty:ty)) => {
        #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
        pub struct $name($vis $ty);

        impl $name {
            #[inline]
            pub fn has(self, bit: Self) -> bool {
                self.0 & bit.0 != 0
            }

            #[inline]
            pub fn has_all(self, bits: Self) -> bool {
                self.0 & bits.0 == bits.0
            }

            #[inline]
            pub fn has_none(self, bits: Self) -> bool {
                self.0 & bits.0 == 0
            }
        }

        impl core::ops::BitOr for $name {
            type Output = Self;

            #[inline(always)]
            fn bitor(self, rhs: Self) -> Self::Output {
                Self(self.0 | rhs.0)
            }
        }

        impl core::ops::BitAnd for $name {
            type Output = Self;

            #[inline(always)]
            fn bitand(self, rhs: Self) -> Self::Output {
                Self(self.0 & rhs.0)
            }
        }
    };
}

