use core::{mem, slice, str};

use numtoa::NumToA;

use {Display, Fmt, Write};

macro_rules! impls {
    ($(($ix:ident, $N:expr)),+,) => {
        $(
            impl Fmt<Display> for $ix {
                fn fmt<W>(
                    &self,
                    writer: &mut W,
                    _strategy: Display,
                ) -> Result<(), W::Error>
                where
                    W: Write,
                {
                    unsafe {
                        // NOTE formatting negative integers requires one more
                        // byte than necessary. See mmstick/numtoa#8.
                        let mut buffer: [u8; $N + 1] = mem::uninitialized();

                        let start = self.numtoa(10, &mut buffer);

                        let ptr = buffer.as_ptr().offset(start as isize);
                        let len = buffer.len() - start;
                        let s = slice::from_raw_parts(ptr, len);
                        let s = str::from_utf8_unchecked(s);
                        writer.write_str(s)
                    }
                }

                fn size_hint(&self, _strategy: Display) -> usize {
                    $N
                }
            }
        )+
    }
}

impls! {
    (i8, 4),
    (i16, 6),
    (i32, 11),
    (i64, 20),
    (u8, 3),
    (u16, 5),
    (u32, 10),
    (u64, 20),
}
