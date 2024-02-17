use std::fmt::Display;

pub struct HexDisplay<'a>(&'a [u8]);

impl<'a> HexDisplay<'a> {
    pub fn new(inner: &'a [u8]) -> Self {
        Self(inner)
    }
}

impl Display for HexDisplay<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for byte in self.0 {
            write!(f, "{byte:0>2x}")?;
        }
        Ok(())
    }
}
