use crate::TypeInfoGenerated;
use crate::reader::ProcessMemoryReader;
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
) -> windows::core::Result<Vec<TypeDef>> {
    assert_eq!(size_of::<TypeDefInfo>(), 32);

    let infos = reader.read_structs::<TypeDefInfo>(
        type_info_generated.typedefs as usize,
        type_info_generated.num_typedefs as usize,
    )?;

    Ok(infos
        .into_iter()
        .map(|info| read_typedef(reader, &info).unwrap())
        .collect())
}

fn read_typedef(
    reader: &ProcessMemoryReader,
    info: &TypeDefInfo,
) -> windows::core::Result<TypeDef> {
    Ok(TypeDef {
        name: reader.read_cstring(info.name as usize)?.unwrap(),
        r#type: reader.read_cstring(info.r#type as usize)?.unwrap(),
        ops: reader.read_cstring(info.ops as usize)?,
        size: info.size,
    })
}
