use genio::bufio::BufWrite;

pub trait Encoding {
    fn encode_char<W: BufWrite>(writer: W, chr: char) -> Result<(), W::WriteError>;
    fn encode_str<W: BufWrite>(writer: W, string: &str) -> Result<(), W::WriteError>;
}

pub struct Encoder<W, E> {
    writer: W,
    phantom: ::core::marker::PhantomData<E>,
}

impl<W: BufWrite, E: Encoding> Encoder<W, E> {
    pub fn new(writer: W) -> Self {
        Encoder {
            writer,
            phantom: Default::default(),
        }
    }
}

impl<W: BufWrite, E: Encoding> ::Write for Encoder<W, E> {
    type Error = W::WriteError;

    fn write_char(&mut self, val: char) -> Result<(), Self::Error> {
        E::encode_char(&mut self.writer, val)
    }

    fn write_str(&mut self, val: &str) -> Result<(), Self::Error> {
        E::encode_str(&mut self.writer, val)
    }

    fn size_hint(&mut self, bytes: usize) {
        self.writer.size_hint(bytes)
    }

    fn uses_size_hint(&self) -> bool {
        self.writer.uses_size_hint()
    }
}

pub enum Utf8 {}

impl Encoding for Utf8 {
    fn encode_char<W: BufWrite>(mut writer: W, chr: char) -> Result<(), W::WriteError> {
        let mut buf = [0; 4];
        writer.write_all(chr.encode_utf8(&mut buf).as_bytes())
    }

    fn encode_str<W: BufWrite>(mut writer: W, string: &str) -> Result<(), W::WriteError> {
        writer.write_all(string.as_bytes())
    }
}
