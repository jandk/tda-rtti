use crate::TypeInfoGenerated;
use crate::reader::{ProcessMemoryError, ProcessMemoryReader};
use serde::Serialize;
use std::ffi::c_char;

const _: () = assert!(size_of::<TypeDefInfo>() == 32);

#[derive(Serialize)]
pub struct TypeDef {
    name: String,
    r#type: String,
    ops: Option<String>,
    size: u32,
}

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct TypeDefInfo {
    name: *const c_char,
    r#type: *const c_char,
    ops: *const c_char,
    size: u32,
}

pub fn read_typedefs(
    reader: &ProcessMemoryReader,
    type_info_generated: &TypeInfoGenerated,
) -> Result<Vec<TypeDef>, ProcessMemoryError> {
    let infos = reader.read_structs::<TypeDefInfo>(
        type_info_generated.typedefs,
        type_info_generated.num_typedefs,
    )?;

    infos
        .iter()
        .map(|info| read_typedef(reader, info))
        .collect()
}

fn read_typedef(
    reader: &ProcessMemoryReader,
    info: &TypeDefInfo,
) -> Result<TypeDef, ProcessMemoryError> {
    Ok(TypeDef {
        name: reader.read_cstring(info.name)?,
        r#type: reader.read_cstring(info.r#type)?,
        ops: reader.read_cstring(info.ops).ok(),
        size: info.size,
    })
}
