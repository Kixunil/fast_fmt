//! This module contains traits and types used for transforming text into other text.
//!
//! Possible use cases are: escaping, URL encoding...

use Write;
use Fmt;

/// Represents a transformation of text to another text.
///
/// It abstracts transforming a single value and a whole input to the writer.
pub trait Transform {
    /// Writes transformed char into provided writer.
    fn transform_char<W: Write>(&self, writer: &mut W, c: char) -> Result<(), W::Error>;

    /// Writes transformed string into provided writer.
    fn transform_str<W: Write>(&self, writer: &mut W, s: &str) -> Result<(), W::Error> {
        for c in s.chars() {
            self.transform_char(writer, c)?;
        }
        Ok(())
    }

    /// Calculates new size hint based on how transformation affects output size.
    ///
    /// It should always be maximum of possible scenarios. If maximum can't be determined, the
    /// `bytes` value should be returned unchanged.
    fn transform_size_hint(&self, bytes: usize) -> usize;
}

impl<'a, T: Transform> Transform for &'a T {
    fn transform_char<W: Write>(&self, writer: &mut W, c: char) -> Result<(), W::Error> {
        (*self).transform_char(writer, c)
    }

    fn transform_str<W: Write>(&self, writer: &mut W, s: &str) -> Result<(), W::Error> {
        (*self).transform_str(writer, s)
    }

    fn transform_size_hint(&self, bytes: usize) -> usize {
        (*self).transform_size_hint(bytes)
    }
}

/// Transformation attached to a writer, transforming everything written into the writer.
pub struct Transformer<T: Transform, W: Write> {
    transformation: T,
    writer: W,
}

impl<T: Transform, W: Write> Transformer<T, W> {
    /// Attaches transformation to the writer.
    pub fn new(transformation: T, writer: W) -> Self {
        Transformer {
            transformation,
            writer,
        }
    }
}

impl<T: Transform, W: Write> Write for Transformer<T, W> {
    type Error = W::Error;

    fn write_char(&mut self, c: char) -> Result<(), Self::Error> {
        self.transformation.transform_char(&mut self.writer, c)
    }

    fn write_str(&mut self, s: &str) -> Result<(), Self::Error> {
        self.transformation.transform_str(&mut self.writer, s)
    }

    fn size_hint(&mut self, bytes: usize) {
        self.writer.size_hint(self.transformation.transform_size_hint(bytes))
    }

    fn uses_size_hint(&self) -> bool {
        self.writer.uses_size_hint()
    }
}

/*
 * Unfortunatelly, this requires specialization.
pub struct TransformStrategy<'a, T: Transform, S: 'a> {
    transformation: T,
    strategy: &'a S,
}

impl<'a, T: Transform, S> TransformStrategy<'a, T, S> {
    pub fn new(transformation: T, strategy: &'a S) -> Self {
        TransformStrategy {
            transformation,
            strategy,
        }
    }
}

impl<'a, T: Transform, S, U: Fmt<S>> Fmt<TransformStrategy<'a, T, S>> for U {
    fn fmt<W: Write>(&self, writer: &mut W, strategy: &TransformStrategy<'a, T, S>) -> Result<(), W::Error> {
        let mut writer = Transformer::new(strategy, writer);
        self.fmt(&mut writer, strategy.strategy)
    }

    fn size_hint(&self, strategy: &TransformStrategy<'a, T, S>) -> usize {
        strategy.transform_size_hint(self.size_hint(strategy.strategy))
    }
}
*/

/// Transformation attached to a value, transforming given vale when formatting.
pub struct Transformed<V, T: Transform> {
    value: V,
    transformation: T,
}

impl<V, T: Transform> Transformed<V, T> {
    /// Attaches transformation to the value.
    pub fn new(value: V, transformation: T) -> Self {
        Transformed {
            value,
            transformation,
        }
    }
}

impl<S, V: Fmt<S>, T: Transform> Fmt<S> for Transformed<V, T> {
    fn fmt<W: Write>(&self, writer: &mut W, strategy: &S) -> Result<(), W::Error> {
        let mut writer = Transformer::new(&self.transformation, writer);
        self.value.fmt(&mut writer, strategy)
    }

    fn size_hint(&self, strategy: &S) -> usize {
        self.transformation.transform_size_hint(self.value.size_hint(strategy))
    }
}
