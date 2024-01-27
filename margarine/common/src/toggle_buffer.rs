use std::fmt::Write;

pub struct ToggleBuffer<T: Write>(Option<T>);

impl<T: Write> ToggleBuffer<T> {
    pub fn new(buffer: Option<T>) -> Self { Self(buffer) }
}


impl<T: Write> Write for ToggleBuffer<T> {
    fn write_str(&mut self, s: &str) -> std::fmt::Result {
        if let Some(val) = &mut self.0 {
            val.write_str(s)
        } else { Ok(()) }
    }
}
