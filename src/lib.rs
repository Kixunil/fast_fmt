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

pub trait Write {
    type Error;

    fn write_char(&mut self, val: char) -> Result<(), Self::Error>;

    fn write_str(&mut self, val: &str) -> Result<(), Self::Error> {
        for c in val.chars() {
            self.write_char(c)?;
        }
        Ok(())
    }

    fn size_hint(&mut self, bytes: usize);
    fn uses_size_hint(&self) -> bool {
        false
    }
}

pub trait Fmt<S = Display> {
    fn fmt<W: Write>(&self, writer: &mut W, strategy: &S) -> Result<(), W::Error>;

    fn size_hint(&self, strategy: &S) -> usize;
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

#[derive(Debug, Default, Copy, Clone)]
pub struct Display;

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
