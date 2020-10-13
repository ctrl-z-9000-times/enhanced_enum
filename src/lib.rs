/// TODO
#[doc(hidden)]
pub use paste;

#[doc(hidden)]
#[macro_export]
macro_rules! _count {
    () => (0_usize);
    ( $x:ident $($xs:ident)* ) => (1_usize + enhanced_enum::_count!($($xs)*));
}

/// TODO
#[macro_export]
macro_rules! enhanced_enum {
    ($name:ident $(,)? {$($variants:ident$(,)?)*}) => {

        #[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
        pub enum $name {
            $( $variants ),*
        }

        impl $name {
            pub const fn count() -> usize {enhanced_enum::_count!($($variants)*)}

            pub fn iter() -> impl std::iter::Iterator<Item=$name> {
                (0..Self::count()).map(|x| match x {
                    $( _ if x == $name::$variants as usize => $name::$variants, )*
                    _ => panic!()
                })
            }
        }

        impl std::fmt::Display for $name {
            fn fmt(&self, _f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                #[allow(unreachable_patterns)]
                match self {
                    $( $name::$variants => write!(_f, stringify!($variants)), )*
                    _ => panic!()
                }
            }
        }

        impl std::convert::TryFrom<usize> for $name {
            type Error = &'static str;
            fn try_from(value: usize) -> Result<Self, Self::Error> {
                match value {
                    $( _ if value == $name::$variants as usize => Ok($name::$variants), )*
                    _ => Err("Bad enum discriminant!")
                }
            }
        }

        impl std::convert::TryFrom<&str> for $name {
            type Error = &'static str;
            fn try_from(value: &str) -> Result<Self, Self::Error> {
                match value {
                    $( _ if value == $name::$variants.to_string() => Ok($name::$variants), )*
                    _ => Err("Unrecognised variant name!")
                }
            }
        }

        // impl pyo3::conversion::FromPyObject<'_> for $name {
        //     fn extract(obj: & pyo3::PyAny) -> std::result::Result<Self, pyo3::PyErr> {
        //         let string: String = obj.extract()?;
        //         use std::convert::TryFrom;
        //         return Ok($name::try_from(string.as_str())
        //             .map_err(|err|
        //                 pyo3::PyErr::new::<pyo3::exceptions::PyTypeError, _>(err.to_string()))?);
        //     }
        // }

        enhanced_enum::paste::paste! {
            #[derive(Debug)]
            pub struct [<$name Array>]<T> {
                data: [T; $name::count()]
            }

            impl<T> [<$name Array>]<T> {
                pub fn new<F>(initial_value: F) -> Self
                    where F: Fn($name) -> T
                {
                    use std::convert::TryFrom;
                    use std::mem::MaybeUninit;
                    let mut data: [T; $name::count()] = unsafe {
                        MaybeUninit::uninit().assume_init()
                    };
                    for (idx, elem) in data.iter_mut().enumerate() {
                        let mut value = initial_value($name::try_from(idx).unwrap());
                        std::mem::swap(&mut value, elem);
                        std::mem::forget(value);
                    }
                    return Self { data };
                }

                pub fn len(&self) -> usize { $name::count() }

                pub fn is_empty(&self) -> bool { self.len() == 0 }

                // TODO: Isn't there a trait for these?

                pub fn iter<'a>(&'a self) -> impl std::iter::Iterator<Item=&T> {
                    self.data.iter()
                }

                pub fn iter_mut<'a>(&'a mut self) -> impl std::iter::Iterator<Item=&mut T> {
                    self.data.iter_mut()
                }

                pub fn iter_enumerate<'a>(&'a self) -> impl std::iter::Iterator<Item=($name, &T)> {
                    use std::convert::TryFrom;
                    self.data.iter().enumerate().map(|(idx, v)| ($name::try_from(idx).unwrap(),v))
                }

                pub fn iter_mut_enumerate<'a>(&'a mut self) -> impl std::iter::Iterator<Item=($name, &mut T)> {
                    use std::convert::TryFrom;
                    self.data.iter_mut().enumerate().map(|(idx, v)| ($name::try_from(idx).unwrap(),v))
                }
            }

            impl<T> std::ops::Index<$name> for [<$name Array>]<T> {
                type Output = T;
                fn index(&self, x: $name) -> &Self::Output {
                    &self.data[x as usize]
                }
            }

            impl<T> std::ops::IndexMut<$name> for [<$name Array>]<T> {
                fn index_mut(&mut self, x: $name) -> &mut Self::Output {
                    &mut self.data[x as usize]
                }
            }
        }
    }
}
