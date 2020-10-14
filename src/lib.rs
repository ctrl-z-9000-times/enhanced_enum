/*! ## Enhanced Fieldless Enumerations and Associated Array Types

In Rust, enumerations can contain data fields which is a powerful language feature.
However not all enums have data fields. Fieldless enums are simply a list of variants.
This crate provides many features for fieldless enums
which are difficult or impossible to provide for enums with data fields.

This crate contains a single item: the `enhanced_enum!` macro which generates an `enum`.

```
enhanced_enum::enhanced_enum!(YourEnum { A, B, C });
```

Translates to:

```ignore
pub enum YourEnum {
    A,
    B,
    C
}

impl YourEnum {
    ...
}

/// Custom wrapper around an array of length `YourEnum::len()`.
/// This array can only be indexed by `YourEnum`.
pub struct YourEnumArray<T> {
    ...
}
```

### Features

* Enhanced enums implement many common traits:
    + `Debug`, `Display`,
    + `Copy`, `Clone`,
    + `PartialEq`, `Eq`, `PartialOrd`, `Ord`,
    + `Hash`

* Iterate through all variants of your enhanced enum with `YourEnum::iter()`.
* Count the number of variants with `YourEnum::count()` or `YourEnum::len()`.

* Make an array which can only be indexed by your enum.
The `enhanced_enum!` macro generates a wrapper around a standard array,
and this custom array type implements a very similar API to a standard array.
The name of the new array type is the enum name with the word "Array" appended.

* Convert between integers, strings, and enhanced enums.
    + `YourEnum::try_from(usize)`
    Also works with `u32` and `64`.
    + `YourEnum::try_from(&str)`
    Note that the string must exactly match a variant name, or else this returns an error.
    + `your_enum as usize`.
    + `your_enum.to_string() -> String`
    + `your_enum.to_str() -> &'static str`

* Interface with Python via the `pyo3` library.
Currently this only implements a converting from python strings to rust.
This is optionally compiled.
To opt-in: build the enhanced_enum crate using the feature flag "pyo3".

### Examples

A histogram for counting DNA nucleotides.
This re-implements the example from the documentation for the
[Trait std::ops::Index](https://doc.rust-lang.org/std/ops/trait.Index.html).

```
enhanced_enum::enhanced_enum!(Nucleotide {
    A,
    C,
    G,
    T,
});

let nucleotide_count = NucleotideArray::<usize>::new_with(|x| match x {
    Nucleotide::A => 14,
    Nucleotide::C => 9,
    Nucleotide::G => 10,
    Nucleotide::T => 12
});
assert_eq!(nucleotide_count[Nucleotide::A], 14);
assert_eq!(nucleotide_count[Nucleotide::C], 9);
assert_eq!(nucleotide_count[Nucleotide::G], 10);
assert_eq!(nucleotide_count[Nucleotide::T], 12);
```
*/

// TODO: Allow the user to put doc-strings on their enums.

// TODO: Arrays should implement the following traits:
//          Display if T is also Display.
//          AsRef, AsMut, Borrow, BorrowMut.

// TODO: Analyze the assembly output and verify that the array access methods
// are not bounds checked. If they are then manually do `get_unchecked` BC the
// enums are always valid indexes into the array.

/// Define a new Enhanced Enum.
///
/// Usage: `enhanced_enum!( EnumName { List, Of, Variants } );`
///
/// See the documentation on the generated types for the full documentation of
/// enhanced enums and associated arrays.
#[macro_export]
macro_rules! enhanced_enum {
    ($name:ident $(,)? {$($variants:ident$(,)?)*}) => {

        #[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
        pub enum $name {
            $( $variants ),*
        }

        impl $name {
            /// Number of variants in this enumeration.
            pub const fn count() -> usize {enhanced_enum::count!($($variants)*)}

            /// Number of variants in this enumeration.
            pub const fn len() -> usize { $name::count() }

            pub const fn is_empty() -> bool { $name::len() == 0 }

            /// Iterate over all variants in this enum, in sorted order.
            pub fn iter() -> impl std::iter::Iterator<Item=$name> {
                (0..Self::count()).map(|x| match x {
                    $( _ if x == $name::$variants as usize => $name::$variants, )*
                    _ => panic!()
                })
            }

            pub fn to_str(&self) -> &'static str {
                match self {
                    $( $name::$variants => stringify!($variants), )*
                }
            }
        }

        impl std::fmt::Display for $name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(f, "{:?}", self)
            }
        }

        impl std::convert::TryFrom<u32> for $name {
            type Error = &'static str;
            fn try_from(value: u32) -> Result<Self, Self::Error> {
                if cfg!(debug_assertions) && u32::try_from($name::count() - 1).is_err() {
                    panic!("Too many enum variants to fit inside a u32!");
                }
                match value {
                    $( _ if value == $name::$variants as u32 => Ok($name::$variants), )*
                    _ => Err("Bad enum discriminant!")
                }
            }
        }

        impl std::convert::TryFrom<u64> for $name {
            type Error = &'static str;
            fn try_from(value: u64) -> Result<Self, Self::Error> {
                let value = u32::try_from(value).unwrap();
                $name::try_from(value)
            }
        }

        impl std::convert::TryFrom<usize> for $name {
            type Error = &'static str;
            fn try_from(value: usize) -> Result<Self, Self::Error> {
                let value = u32::try_from(value).unwrap();
                $name::try_from(value)
            }
        }

        impl std::convert::TryFrom<&str> for $name {
            type Error = &'static str;
            fn try_from(value: &str) -> Result<Self, Self::Error> {
                match value {
                    $( _ if value == $name::$variants.to_string() => Ok($name::$variants), )*
                    _ => Err("Unrecognized variant name!")
                }
            }
        }

        enhanced_enum::pyo3_traits!($name, {$($variants,)*});

        enhanced_enum::paste::paste! {
            /// Container to associate each enum variant with a datum.
            ///
            /// This is an array type and it implements much of the array API,
            /// with a few caveats and exceptions. The foremost deviation from
            /// the standard array API is that this container must be indexed
            /// using variants of the correct enum type.
            pub struct [<$name Array>]<T> {
                data: [T; $name::count()]
            }

            impl<T> [<$name Array>]<T> {
                /// Create a new array filled with the given value.
                pub fn new(initial_value: T) -> Self where T: Clone {
                    Self::new_with(|_| initial_value.clone())
                }

                /// Create a new array using a closure to associate each enum
                /// variant with its initial value.
                pub fn new_with<F>(initial_value: F) -> Self
                    where F: Fn($name) -> T
                {
                    use std::convert::TryFrom;
                    use std::mem::{MaybeUninit, forget, replace};
                    let mut data: [T; $name::count()] = unsafe {
                        MaybeUninit::uninit().assume_init()
                    };
                    for (idx, elem) in data.iter_mut().enumerate() {
                        forget(replace(elem, initial_value($name::try_from(idx).unwrap())));
                    }
                    return Self { data };
                }

                pub const fn len(&self) -> usize { $name::count() }

                pub const fn is_empty(&self) -> bool { self.len() == 0 }

                pub fn iter<'a>(&'a self) -> impl std::iter::Iterator<Item=&T> {
                    self.data.iter()
                }

                pub fn iter_mut<'a>(&'a mut self) -> impl std::iter::Iterator<Item=&mut T> {
                    self.data.iter_mut()
                }

                /// Iterate and Enumerate, where Enumerate yields enum variants instead of usize.
                pub fn iter_enumerate<'a>(&'a self) -> impl std::iter::Iterator<Item=($name, &T)> {
                    use std::convert::TryFrom;
                    self.data.iter().enumerate().map(|(idx, v)| ($name::try_from(idx).unwrap(),v))
                }

                /// Iterate and Enumerate, where Enumerate yields enum variants instead of usize.
                pub fn iter_mut_enumerate<'a>(&'a mut self) -> impl std::iter::Iterator<Item=($name, &mut T)> {
                    use std::convert::TryFrom;
                    self.data.iter_mut().enumerate().map(|(idx, v)| ($name::try_from(idx).unwrap(),v))
                }

                /// Returns an array like self, with function f applied to each element.
                pub fn map<F, Q>(&self, f: F) -> [<$name Array>]<Q> where F: Fn(&T) -> Q {
                    [<$name Array>]::new_with(|x| f(&self[x]))
                }

                /// Returns true if the array contains an element with the given value.
                pub fn contains(&self, x: &T) -> bool where T: PartialEq<T> {
                    self.data.contains(x)
                }
            }

            impl<T> std::fmt::Debug for [<$name Array>]<T> where T: std::fmt::Debug {
                fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                    // TODO: Make this pretty. Print the variant names along
                    // side the array contents.
                    self.data.fmt(f)
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

            impl<'a, T> std::iter::IntoIterator for &'a [<$name Array>]<T> {
                type Item = &'a T;
                type IntoIter = std::slice::Iter<'a, T>;
                fn into_iter(self) -> std::slice::Iter<'a, T> {
                    self.data.iter()
                }
            }

            impl<'a, T> std::iter::IntoIterator for &'a mut [<$name Array>]<T> {
                type Item = &'a mut T;
                type IntoIter = std::slice::IterMut<'a, T>;
                fn into_iter(self) -> std::slice::IterMut<'a, T> {
                    self.data.iter_mut()
                }
            }

            impl<T> Copy for [<$name Array>]<T> where T: Copy {}

            impl<T> Clone for [<$name Array>]<T> where T: Clone {
                fn clone(&self) -> Self {
                    Self { data: self.data.clone() }
                }
            }

            impl<T> PartialEq for [<$name Array>]<T> where T: PartialEq {
                fn eq(&self, other: &Self) -> bool {
                    self.data == other.data
                }
            }

            impl<T> Eq for [<$name Array>]<T> where T: Eq {}

            impl<T> PartialOrd for [<$name Array>]<T> where T: PartialOrd {
                fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
                    self.data.partial_cmp(&other.data)
                }
            }

            impl<T> Ord for [<$name Array>]<T> where T: Ord {
                fn cmp(&self, other: &Self) -> std::cmp::Ordering {
                    self.data.cmp(&other.data)
                }
            }

            impl<T> std::hash::Hash for [<$name Array>]<T> where T: std::hash::Hash {
                fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
                    self.data.hash(state);
                }
            }
        }
    }
}

#[doc(hidden)]
pub use paste;

#[doc(hidden)]
#[macro_export]
macro_rules! count {
    () => (0_usize);
    ( $x:ident $($xs:ident)* ) => (1_usize + enhanced_enum::count!($($xs)*));
}

#[cfg(not(feature = "pyo3"))]
#[doc(hidden)]
#[macro_export]
macro_rules! pyo3_traits {
    ($name:ident $(,)? {$($variants:ident$(,)?)*}) => {};
}
#[cfg(feature = "pyo3")]
#[doc(hidden)]
#[macro_export]
macro_rules! pyo3_traits {
    ($name:ident $(,)? {$($variants:ident$(,)?)*}) => {
        impl pyo3::conversion::FromPyObject<'_> for $name {
            fn extract(obj: &pyo3::PyAny) -> std::result::Result<Self, pyo3::PyErr> {
                let string: String = obj.extract()?;
                use std::convert::TryFrom;
                return Ok($name::try_from(string.as_str()).map_err(|err| {
                    pyo3::PyErr::new::<pyo3::exceptions::PyTypeError, _>(err.to_string())
                })?);
            }
        }
    };
}
