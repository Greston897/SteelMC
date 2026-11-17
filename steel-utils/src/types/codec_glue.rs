use std::io::{self, Cursor, Write};

use simdnbt::owned::NbtTag;

use crate::{
    codec::VarInt,
    hash::{ComponentHasher, HashComponent},
    nbt::NbtNumeric as _,
    serial::{ReadFrom, WriteTo},
};

/// A raw block state ID. Using the registry this ID can be resolved into a
/// block and its current property values.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct BlockStateId(pub u16);

impl WriteTo for BlockStateId {
    fn write(&self, writer: &mut impl Write) -> io::Result<()> {
        VarInt(i32::from(self.0)).write(writer)
    }
}

impl ReadFrom for BlockStateId {
    fn read(data: &mut Cursor<&[u8]>) -> io::Result<Self> {
        let id = VarInt::read(data)?.0;
        let value = u16::try_from(id).map_err(|_| {
            io::Error::new(
                io::ErrorKind::InvalidData,
                format!("block state id out of u16 range: {id}"),
            )
        })?;
        Ok(Self(value))
    }
}

impl HashComponent for BlockStateId {
    fn hash_component(&self, hasher: &mut ComponentHasher) {
        hasher.put_int(i32::from(self.0));
    }
}

impl simdnbt::ToNbtTag for BlockStateId {
    fn to_nbt_tag(self) -> NbtTag {
        NbtTag::Int(i32::from(self.0))
    }
}

impl simdnbt::FromNbtTag for BlockStateId {
    fn from_nbt_tag(tag: simdnbt::borrow::NbtTag<'_, '_>) -> Option<Self> {
        let value = u16::try_from(tag.codec_i32()?).ok()?;
        Some(Self(value))
    }
}
