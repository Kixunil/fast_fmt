use ::*;
use ::void::Void;

impl Fmt for ::std::string::String {
    fn fmt<W: Write>(&self, writer: &mut W, strategy: &Display) -> Result<(), W::Error> {
        (**self).fmt(writer, strategy)
    }

    fn size_hint(&self, _: &Display) -> usize {
        self.len()
    }
}

impl Write for ::std::string::String {
    type Error = Void;

    fn write_char(&mut self, val: char) -> Result<(), Self::Error> {
        self.push(val);
        Ok(())
    }

    fn write_str(&mut self, val: &str) -> Result<(), Self::Error> {
        self.push_str(val);
        Ok(())
    }

    fn size_hint(&mut self, bytes: usize) {
        self.reserve(bytes);
    }

    fn uses_size_hint(&self) -> bool {
        true
    }
}
