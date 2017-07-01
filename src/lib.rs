//! This crate provides formatting similar to `core::fmt` but it's faster, more flexible and
//! provides safer error handling.
//!
//! The core traits of this crate are `Write` and `Fmt<S>`. `S` represents a formatting strategy.
//! There are multiple formatting strategies and users of this crate can define their own. This is
//! similar to different traits in `core::fmt`, like `Display`, `Debug`...
//! 
//! A formatting strategy can also hold relevant configuration for given formatting. E.g. whether
//! HEX dump should be uppercase or lowercase.

#![no_std]
#![cfg_attr(feature = "nightly", feature(test))]

#[cfg(feature = "with_std")]
extern crate std;

extern crate numtoa;
extern crate void;

mod int_impls;

#[cfg(feature = "with_std")]
mod std_impls;

#[macro_use]
mod macros;

pub mod consts;
pub mod transform;

use transform::{Transform, Transformer, Transformed};

/// Represents types to which characters may be written.
///
/// The difference between this trait and byte writers (such as `std::io::Write` or `genio::Write`)
/// is that this guarantees valid UTF-8 encoding - it allows implementing this trait on `String`.
pub trait Write {
    /// Type of error returned if write fails.
    type Error;

    /// Writes single char.
    ///
    /// If this operation fails the state of underlying writer is unspecified. Re-trying is
    /// therefore impossible.
    fn write_char(&mut self, val: char) -> Result<(), Self::Error>;

    /// Writes whole string.
    ///
    /// By default, this just iterates and writes all characters. The implementors are encouraged
    /// to override this and provide faster implementation if possible.
    ///
    /// If this operation fails the state of underlying writer is unspecified. Re-trying is
    /// therefore impossible.
    fn write_str(&mut self, val: &str) -> Result<(), Self::Error> {
        for c in val.chars() {
            self.write_char(c)?;
        }
        Ok(())
    }

    /// Hints that implementor should allocate enough space so that string containing `bytes` UTF-8
    /// bytes can be stored in it.
    ///
    /// Warning: the number of bytes (chars) actually written might differ from this number! The
    /// writer must NOT fail if it does!
    fn size_hint(&mut self, bytes: usize);

    /// Tells the user whether the size hint is actually used. This allows the user to skip
    /// calculation of the hint.
    ///
    /// By default this returns false but `size_hint` method is still mandatory to prevent people
    /// implementing this trait from forgetting about the size hint.
    fn uses_size_hint(&self) -> bool {
        false
    }

    /// Combinator for creating transformed writer.
    fn transform<T: Transform>(self, transformation: T) -> Transformer<T, Self> where Self: Sized {
        Transformer::new(transformation, self)
    }
}

impl<'a, W: Write> Write for &'a mut W {
    type Error = W::Error;

    fn write_char(&mut self, val: char) -> Result<(), Self::Error> {
        (*self).write_char(val)
    }

    fn write_str(&mut self, val: &str) -> Result<(), Self::Error> {
        (*self).write_str(val)
    }

    fn size_hint(&mut self, bytes: usize) {
        (*self).size_hint(bytes)
    }

    fn uses_size_hint(&self) -> bool {
        (**self).uses_size_hint()
    }
}

/// The formatting trait. Represents types that can be formated.
///
/// This trait is much like `core::fmt::Display`, `core::fmt::Debug` and other similar traits from
/// `core::fmt`, but instead of many traits it is a single parametrized trait.
///
/// The `S` type parameter is formatting strategy and it defaults to `Display`.
pub trait Fmt<S = Display> {
    /// The implementor should write itself into `writer` inside this function.
    fn fmt<W: Write>(&self, writer: &mut W, strategy: &S) -> Result<(), W::Error>;

    /// The implementor should estimate how many bytes would it's representation have in UTF-8 if
    /// formated using specific strategy.
    ///
    /// If the implementor knows maximum possible size, it should return it.
    /// If the implementor doesn't know maximum possible size, it should return minimum possible
    /// size. (0 is always valid minimum)
    fn size_hint(&self, strategy: &S) -> usize;

    /// Combinator for transforming the value,
    fn transformed<T: Transform>(self, transformation: T) -> transform::Transformed<Self, T> where Self: Sized {
        Transformed::new(self, transformation)
    }
}

impl<'a, S, T: ?Sized + Fmt<S>> Fmt<S> for &'a T {
    fn fmt<W: Write>(&self, writer: &mut W, strategy: &S) -> Result<(), W::Error> {
        (*self).fmt(writer, strategy)
    }

    fn size_hint(&self, strategy: &S) -> usize {
        (*self).size_hint(strategy)
    }
}

/*
 * Any way to achieve this without conflict?
impl<'a, S, T: Fmt<S>> Fmt<&'a S> for T {
    fn fmt<W: Write>(&self, writer: &mut W, strategy: &S) -> Result<(), W::Error> {
        <Self as Fmt<S>>::fmt(self, writer, *strategy)
    }

    fn size_hint(&self, strategy: &S) -> usize {
        <Self as Fmt<S>>::size_hint(self, *strategy)
    }
}
*/

/// Pair of value and a strategy that implements `Fmt`. This allows combining many different
/// strategies in single chain.
pub struct Instantiated<'a, T, S: 'a> {
    value: T,
    strategy: &'a S,
}

impl<'a, S, T: Fmt<S>> Instantiated<'a, T, S> {
    pub fn new(value: T, strategy: &'a S) -> Self {
        Instantiated {
            value,
            strategy,
        }
    }

    pub fn chain<O: Fmt>(self, other: O) -> Chain<Self, O> {
        Chain::new(self, other)
    }
}

impl<'a, S, T: Fmt<S>> Fmt for Instantiated<'a, T, S> {
    fn fmt<W: Write>(&self, writer: &mut W, _strategy: &Display) -> Result<(), W::Error> {
        self.value.fmt(writer, self.strategy)
    }

    fn size_hint(&self, _strategy: &Display) -> usize {
        self.value.size_hint(self.strategy)
    }
}

/// Two values chained together, so they can be concatenated.
pub struct Chain<T0, T1> {
    val0: T0,
    val1: T1,
}

impl<T0: Fmt, T1: Fmt> Chain<T0, T1> {
    pub fn new(val0: T0, val1: T1) -> Self {
        Chain {
            val0,
            val1,
        }
    }

    pub fn chain<O: Fmt>(self, other: O) -> Chain<Self, O> {
        Chain::new(self, other)
    }
}

impl<S, T0: Fmt<S>, T1: Fmt<S>> Fmt<S> for Chain<T0, T1> {
    fn fmt<W: Write>(&self, writer: &mut W, strategy: &S) -> Result<(), W::Error> {
        self.val0.fmt(writer, strategy)?;
        self.val1.fmt(writer, strategy)
    }

    fn size_hint(&self, strategy: &S) -> usize {
        self.val0.size_hint(strategy) + self.val1.size_hint(strategy)
    }
}

/// Empty type that never writes anything.
///
/// This is mostly a helper for macros.
#[derive(Debug, Default, Copy, Clone)]
pub struct Empty;

impl Empty {
    pub fn chain<O: Fmt>(self, other: O) -> Chain<Self, O> {
        Chain::new(self, other)
    }
}

impl<S> Fmt<S> for Empty {
    fn fmt<W: Write>(&self, _writer: &mut W, _strategy: &S) -> Result<(), W::Error> {
        Ok(())
    }

    fn size_hint(&self, _strategy: &S) -> usize {
        0
    }
}

/// Represents strategy with same semantics as `core::fmt::Display`.
#[derive(Debug, Default, Copy, Clone)]
pub struct Display;

/// Represents strategy with same semantics as `core::fmt::Debug`.
#[derive(Debug, Default, Copy, Clone)]
pub struct Debug;

impl Fmt<Display> for str {
    fn fmt<W: Write>(&self, writer: &mut W, _strategy: &Display) -> Result<(), W::Error> {
        writer.write_str(self)
    }

    fn size_hint(&self, _strategy: &Display) -> usize {
        self.len()
    }
}

impl Fmt<Display> for char {
    fn fmt<W: Write>(&self, writer: &mut W, _strategy: &Display) -> Result<(), W::Error> {
        writer.write_char(*self)
    }

    fn size_hint(&self, _strategy: &Display) -> usize {
        self.len_utf8()
    }
}

/// Error type indicating that buffer was too small.
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct BufferOverflow;

impl ::core::fmt::Debug for BufferOverflow {
    fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
        write!(f, "Attempt to write past buffer")
    }
}

impl<'a> Write for &'a mut [u8] {
    type Error = BufferOverflow;

    fn write_char(&mut self, val: char) -> Result<(), Self::Error> {
        let mut buf = [0; 4];
        let buf = val.encode_utf8(&mut buf);
        let buf = buf.as_bytes();
        if buf.len() <= self.len() {
            let (first, second) = ::core::mem::replace(self, &mut []).split_at_mut(buf.len());
            first.copy_from_slice(buf);
            *self = second;
            Ok(())
        } else {
            Err(BufferOverflow)
        }
    }

    fn size_hint(&mut self, _bytes: usize) {}
}

#[cfg(test)]
mod tests {
    const TEST_STR: &str = "Hello world1";

    #[test]
    fn slice_write() {
        use ::Write;

        let mut buf = [0u8; 42];
        {
            let mut buf: &mut [u8] = &mut buf;
            buf.write_str(TEST_STR).unwrap();
        }
        assert_eq!(&buf[0..TEST_STR.len()], TEST_STR.as_bytes());
    }

    #[test]
    fn buffer_overflow() {
        use ::Write;
        use ::BufferOverflow;

        let mut buf = [0u8; 5];
        let mut buf: &mut [u8] = &mut buf;
        assert_eq!(buf.write_str(TEST_STR), Err(BufferOverflow));

        let mut buf = &mut buf[0..0];
        assert_eq!(buf.write_str(TEST_STR), Err(BufferOverflow));
    }

    #[test]
    fn str_fmt() {
        use ::Fmt;
        use ::Display;

        let mut buf = [0u8; 42];
        {
            let mut buf: &mut [u8] = &mut buf;
            TEST_STR.fmt(&mut buf, &Display).unwrap();
        }
        assert_eq!(&buf[0..TEST_STR.len()], TEST_STR.as_bytes());
    }

    #[test]
    fn instantiated_fmt() {
        use ::Fmt;
        use ::Display;
        use ::Instantiated;

        let mut buf = [0u8; 42];
        {
            let mut buf: &mut [u8] = &mut buf;
            let display = Display;
            let inst = Instantiated::new(TEST_STR, &display);
            inst.fmt(&mut buf, &Display).unwrap();
        }
        assert_eq!(&buf[0..TEST_STR.len()], TEST_STR.as_bytes());
    }

    #[test]
    fn chain() {
        use ::Fmt;
        use ::Display;
        use ::Instantiated;

        let mut buf = [0u8; 42];
        {
            let mut buf: &mut [u8] = &mut buf;
            let display = Display;
            let inst = Instantiated::new(TEST_STR, &display);
            let chain = inst.chain(TEST_STR);
            chain.fmt(&mut buf, &Display).unwrap();
        }
        assert_eq!(&buf[0..TEST_STR.len()], TEST_STR.as_bytes());
        assert_eq!(&buf[TEST_STR.len()..(TEST_STR.len() * 2)], TEST_STR.as_bytes());
    }

    #[test]
    fn transform() {
        use ::transform::Transform;
        use ::Write;

        struct Upper;

        impl Transform for Upper {
            fn transform_char<W: Write>(&self, writer: &mut W, c: char) -> Result<(), W::Error> {
                for c in c.to_uppercase() {
                    writer.write_char(c)?;
                }
                Ok(())
            }

            fn transform_size_hint(&self, bytes: usize) -> usize {
                bytes
            }
        }

        {
            let mut buf = [0u8; 42];
            {
                let buf = &mut buf;
                let mut buf = buf.transform(Upper);

                fwrite!(&mut buf, "Hello world!").unwrap();
            }
            assert_eq!(&buf[0..12], "HELLO WORLD!".as_bytes());
        }

        {
            let mut buf = [0u8; 42];
            {
                let mut buf: &mut [u8] = &mut buf;
                fwrite!(&mut buf, "Hello", " world!".transformed(Upper)).unwrap();
            }
            assert_eq!(&buf[0..12], "Hello WORLD!".as_bytes());
        }
    }
}

#[cfg(feature = "nightly")]
#[cfg(feature = "with_std")]
#[cfg(test)]
mod bench {
    extern crate test;
    use self::test::Bencher;

    const TEST_STR: &str = "Hello world1";

    #[bench]
    fn bench_fast_fmt(bencher: &mut Bencher) {
        bencher.iter(|| {
            let mut string = ::std::string::String::new();
            fwrite!(&mut string, TEST_STR, TEST_STR).unwrap();
            string
        });
    }

    #[bench]
    fn bench_core_fmt(bencher: &mut Bencher) {
        use core::fmt::Write;
        bencher.iter(|| {
            let mut string = ::std::string::String::new();
            write!(&mut string, "{}{}", TEST_STR, TEST_STR).unwrap();
            string
        });
    }
}
