use crate::TypeInfoGenerated;
use crate::reader::{ProcessMemoryError, ProcessMemoryReader};
use serde::Serialize;
use std::ffi::c_char;

#[derive(Serialize)]
pub(crate) struct TypeDef {
    name: String,
    r#type: String,
    ops: Option<String>,
    size: u32,
}

#[repr(C)]
#[derive(Debug)]
pub(crate) struct TypeDefInfo {
    name: *const c_char,
    r#type: *const c_char,
    ops: *const c_char,
    size: u32,
}

pub(crate) fn read_typedefs(
    reader: &ProcessMemoryReader,
    type_info_generated: &TypeInfoGenerated,
) -> Result<Vec<TypeDef>, ProcessMemoryError> {
    assert_eq!(size_of::<TypeDefInfo>(), 32);

    let infos = reader.read_structs::<TypeDefInfo>(
        type_info_generated.typedefs as usize,
        type_info_generated.num_typedefs as usize,
    )?;

    Ok(infos
        .into_iter()
        .map(|info| read_typedef(reader, &info).expect("Could not read typedef"))
        .collect())
}

fn read_typedef(
    reader: &ProcessMemoryReader,
    info: &TypeDefInfo,
) -> Result<TypeDef, ProcessMemoryError> {
    Ok(TypeDef {
        name: reader.read_cstring(info.name as usize)?,
        r#type: reader.read_cstring(info.r#type as usize)?,
        ops: reader.read_cstring(info.ops as usize).ok(),
        size: info.size,
    })
}
