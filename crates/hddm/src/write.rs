use std::io::Write;

use crate::{HddmResult, xdr::XdrWriter};

pub struct HddmWriter<W: Write> {
    xdr: XdrWriter<W>,
}

impl<W: Write> HddmWriter<W> {
    pub fn new(inner: W) -> Self {
        Self {
            xdr: XdrWriter::new(inner),
        }
    }

    pub fn flush(&mut self) -> HddmResult<()> {
        Ok(self.xdr.inner.flush()?)
    }

    pub fn write_i32(&mut self, x: i32) -> HddmResult<()> {
        Ok(self.xdr.write_i32(x)?)
    }

    pub fn write_u32(&mut self, x: u32) -> HddmResult<()> {
        Ok(self.xdr.write_u32(x)?)
    }

    pub fn write_i64(&mut self, x: i64) -> HddmResult<()> {
        Ok(self.xdr.write_i64(x)?)
    }

    pub fn write_u64(&mut self, x: u64) -> HddmResult<()> {
        Ok(self.xdr.write_u64(x)?)
    }

    pub fn write_f32(&mut self, x: f32) -> HddmResult<()> {
        Ok(self.xdr.write_f32(x)?)
    }

    pub fn write_f64(&mut self, x: f64) -> HddmResult<()> {
        Ok(self.xdr.write_f64(x)?)
    }

    pub fn write_bool(&mut self, x: bool) -> HddmResult<()> {
        Ok(self.xdr.write_bool(x)?)
    }

    pub fn write_string(&mut self, s: &str) -> HddmResult<()> {
        Ok(self.xdr.write_string(s)?)
    }

    pub fn write_element<F>(&mut self, f: F) -> HddmResult<()>
    where
        F: FnOnce(&mut HddmWriter<Vec<u8>>) -> HddmResult<()>,
    {
        let mut payload_writer = HddmWriter::new(Vec::new());
        f(&mut payload_writer)?;

        let payload = payload_writer.into_inner();

        self.write_i32(payload.len() as i32)?;
        self.xdr.write_all(&payload)?;

        Ok(())
    }

    pub fn write_list<T: HddmWrite>(&mut self, items: &[T]) -> HddmResult<()> {
        self.write_element(|w| {
            if !items.is_empty() {
                w.write_i32(items.len() as i32)?;

                for item in items {
                    item.write_contents(w)?;
                }
            }

            Ok(())
        })
    }

    pub fn write_link<T: HddmWrite>(&mut self, item: &Option<T>) -> HddmResult<()> {
        self.write_element(|w| {
            if let Some(item) = item {
                item.write_contents(w)?;
            }

            Ok(())
        })
    }

    pub fn into_inner(self) -> W {
        self.xdr.into_inner()
    }
}

pub trait HddmWrite {
    fn write_contents<W: Write>(&self, w: &mut HddmWriter<W>) -> HddmResult<()>;

    fn write_hddm<W: Write>(&self, w: &mut HddmWriter<W>) -> HddmResult<()> {
        w.write_element(|w| self.write_contents(w))
    }
}

pub trait HddmPrimitiveWrite {
    fn write_primitive<W: Write>(&self, w: &mut HddmWriter<W>) -> HddmResult<()>;
}

impl HddmPrimitiveWrite for i32 {
    fn write_primitive<W: Write>(&self, w: &mut HddmWriter<W>) -> HddmResult<()> {
        w.write_i32(*self)
    }
}

impl HddmPrimitiveWrite for u32 {
    fn write_primitive<W: Write>(&self, w: &mut HddmWriter<W>) -> HddmResult<()> {
        w.write_u32(*self)
    }
}

impl HddmPrimitiveWrite for i64 {
    fn write_primitive<W: Write>(&self, w: &mut HddmWriter<W>) -> HddmResult<()> {
        w.write_i64(*self)
    }
}

impl HddmPrimitiveWrite for u64 {
    fn write_primitive<W: Write>(&self, w: &mut HddmWriter<W>) -> HddmResult<()> {
        w.write_u64(*self)
    }
}

impl HddmPrimitiveWrite for f32 {
    fn write_primitive<W: Write>(&self, w: &mut HddmWriter<W>) -> HddmResult<()> {
        w.write_f32(*self)
    }
}

impl HddmPrimitiveWrite for f64 {
    fn write_primitive<W: Write>(&self, w: &mut HddmWriter<W>) -> HddmResult<()> {
        w.write_f64(*self)
    }
}

impl HddmPrimitiveWrite for bool {
    fn write_primitive<W: Write>(&self, w: &mut HddmWriter<W>) -> HddmResult<()> {
        w.write_bool(*self)
    }
}

impl HddmPrimitiveWrite for String {
    fn write_primitive<W: Write>(&self, w: &mut HddmWriter<W>) -> HddmResult<()> {
        w.write_string(self)
    }
}
