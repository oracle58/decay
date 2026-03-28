/// Minimal binary serialization. No external dependencies.
use std::io::{self, Read, Write};

pub trait Serialize {
    fn serialize(&self, w: &mut dyn Write) -> io::Result<()>;
}

pub trait Deserialize: Sized {
    fn deserialize(r: &mut dyn Read) -> io::Result<Self>;
}

impl Serialize for i8 {
    fn serialize(&self, w: &mut dyn Write) -> io::Result<()> {
        w.write_all(&self.to_le_bytes())
    }
}
impl Deserialize for i8 {
    fn deserialize(r: &mut dyn Read) -> io::Result<Self> {
        let mut buf = [0u8; 1];
        r.read_exact(&mut buf)?;
        Ok(i8::from_le_bytes(buf))
    }
}

impl Serialize for i16 {
    fn serialize(&self, w: &mut dyn Write) -> io::Result<()> {
        w.write_all(&self.to_le_bytes())
    }
}
impl Deserialize for i16 {
    fn deserialize(r: &mut dyn Read) -> io::Result<Self> {
        let mut buf = [0u8; 2];
        r.read_exact(&mut buf)?;
        Ok(i16::from_le_bytes(buf))
    }
}

impl Serialize for u8 {
    fn serialize(&self, w: &mut dyn Write) -> io::Result<()> {
        w.write_all(&[*self])
    }
}
impl Deserialize for u8 {
    fn deserialize(r: &mut dyn Read) -> io::Result<Self> {
        let mut buf = [0u8; 1];
        r.read_exact(&mut buf)?;
        Ok(buf[0])
    }
}

impl Serialize for u16 {
    fn serialize(&self, w: &mut dyn Write) -> io::Result<()> {
        w.write_all(&self.to_le_bytes())
    }
}
impl Deserialize for u16 {
    fn deserialize(r: &mut dyn Read) -> io::Result<Self> {
        let mut buf = [0u8; 2];
        r.read_exact(&mut buf)?;
        Ok(u16::from_le_bytes(buf))
    }
}

impl Serialize for u32 {
    fn serialize(&self, w: &mut dyn Write) -> io::Result<()> {
        w.write_all(&self.to_le_bytes())
    }
}
impl Deserialize for u32 {
    fn deserialize(r: &mut dyn Read) -> io::Result<Self> {
        let mut buf = [0u8; 4];
        r.read_exact(&mut buf)?;
        Ok(u32::from_le_bytes(buf))
    }
}

impl Serialize for u64 {
    fn serialize(&self, w: &mut dyn Write) -> io::Result<()> {
        w.write_all(&self.to_le_bytes())
    }
}
impl Deserialize for u64 {
    fn deserialize(r: &mut dyn Read) -> io::Result<Self> {
        let mut buf = [0u8; 8];
        r.read_exact(&mut buf)?;
        Ok(u64::from_le_bytes(buf))
    }
}

impl Serialize for i32 {
    fn serialize(&self, w: &mut dyn Write) -> io::Result<()> {
        w.write_all(&self.to_le_bytes())
    }
}
impl Deserialize for i32 {
    fn deserialize(r: &mut dyn Read) -> io::Result<Self> {
        let mut buf = [0u8; 4];
        r.read_exact(&mut buf)?;
        Ok(i32::from_le_bytes(buf))
    }
}

impl Serialize for f32 {
    fn serialize(&self, w: &mut dyn Write) -> io::Result<()> {
        w.write_all(&self.to_le_bytes())
    }
}
impl Deserialize for f32 {
    fn deserialize(r: &mut dyn Read) -> io::Result<Self> {
        let mut buf = [0u8; 4];
        r.read_exact(&mut buf)?;
        Ok(f32::from_le_bytes(buf))
    }
}

impl Serialize for bool {
    fn serialize(&self, w: &mut dyn Write) -> io::Result<()> {
        w.write_all(&[*self as u8])
    }
}
impl Deserialize for bool {
    fn deserialize(r: &mut dyn Read) -> io::Result<Self> {
        let mut buf = [0u8; 1];
        r.read_exact(&mut buf)?;
        Ok(buf[0] != 0)
    }
}

impl Serialize for String {
    fn serialize(&self, w: &mut dyn Write) -> io::Result<()> {
        (self.len() as u32).serialize(w)?;
        w.write_all(self.as_bytes())
    }
}
impl Deserialize for String {
    fn deserialize(r: &mut dyn Read) -> io::Result<Self> {
        let len = u32::deserialize(r)? as usize;
        let mut buf = vec![0u8; len];
        r.read_exact(&mut buf)?;
        String::from_utf8(buf).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))
    }
}

impl<T: Serialize> Serialize for Vec<T> {
    fn serialize(&self, w: &mut dyn Write) -> io::Result<()> {
        (self.len() as u32).serialize(w)?;
        for item in self {
            item.serialize(w)?;
        }
        Ok(())
    }
}
impl<T: Deserialize> Deserialize for Vec<T> {
    fn deserialize(r: &mut dyn Read) -> io::Result<Self> {
        let len = u32::deserialize(r)? as usize;
        let mut v = Vec::with_capacity(len);
        for _ in 0..len {
            v.push(T::deserialize(r)?);
        }
        Ok(v)
    }
}

pub fn save_to_file<T: Serialize>(path: &str, data: &T) -> io::Result<()> {
    let mut file = std::fs::File::create(path)?;
    data.serialize(&mut file)
}

pub fn load_from_file<T: Deserialize>(path: &str) -> io::Result<T> {
    let mut file = std::fs::File::open(path)?;
    T::deserialize(&mut file)
}
