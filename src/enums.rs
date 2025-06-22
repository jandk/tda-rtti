use crate::TypeInfoGenerated;
use crate::reader::{ProcessMemoryError, ProcessMemoryReader};
use serde::Serialize;
use std::ffi::c_char;

#[derive(Serialize)]
pub(crate) struct Enum {
    name: String,
    hash: u32,
    values: Vec<EnumValue>,
}

#[derive(Serialize)]
pub(crate) struct EnumValue {
    name: String,
    value: u64,
    hash: u64,
}

#[repr(C)]
#[derive(Debug)]
pub(crate) struct EnumTypeInfo {
    name: *const c_char,
    flags: u64,
    enum_type: EnumType,
    name_hash: u32,
    value_index_length: u32,
    values: *const EnumValueInfo,
    value_name_hashes: *const u64,
    enum_checksum: u64,
    value_index: *const i32,
}

#[repr(C)]
#[derive(Debug)]
pub(crate) struct EnumValueInfo {
    name: *const c_char,
    value: u64,
}

#[repr(C)]
#[derive(Debug)]
enum EnumType {
    EnumS8 = 0,
    EnumU8 = 1,
    EnumS16 = 2,
    EnumU16 = 3,
    EnumS32 = 4,
    EnumU32 = 5,
    EnumS64 = 6,
    EnumU64 = 7,
}

pub(crate) fn read_enums(
    reader: &ProcessMemoryReader,
    type_info_generated: &TypeInfoGenerated,
) -> Result<Vec<Enum>, ProcessMemoryError> {
    assert_eq!(size_of::<EnumTypeInfo>(), 64);
    assert_eq!(size_of::<EnumValueInfo>(), 16);

    let enum_type_infos = reader.read_structs::<EnumTypeInfo>(
        type_info_generated.enums as usize,
        type_info_generated.num_enums as usize - 1,
    )?;

    Ok(enum_type_infos
        .into_iter()
        .map(|info| read_enum(reader, &info).expect("Could not read enum"))
        .collect())
}

fn read_enum(
    reader: &ProcessMemoryReader,
    enum_type_info: &EnumTypeInfo,
) -> Result<Enum, ProcessMemoryError> {
    Ok(Enum {
        name: reader.read_cstring(enum_type_info.name as usize)?,
        hash: enum_type_info.name_hash,
        values: read_enum_values(reader, enum_type_info)?,
    })
}

fn read_enum_values(
    reader: &ProcessMemoryReader,
    enum_type_info: &EnumTypeInfo,
) -> Result<Vec<EnumValue>, ProcessMemoryError> {
    let values = reader.read_structs::<EnumValueInfo>(
        enum_type_info.values as usize,
        enum_type_info.value_index_length as usize,
    )?;
    let hashes = reader.read_structs::<u64>(
        enum_type_info.value_name_hashes as usize,
        enum_type_info.value_index_length as usize,
    )?;

    Ok(values
        .iter()
        .zip(&hashes)
        .filter_map(|(value, &hash)| read_enum_value(reader, value, hash).ok())
        .collect())
}

fn read_enum_value(
    reader: &ProcessMemoryReader,
    enum_value_info: &EnumValueInfo,
    enum_value_hash: u64,
) -> Result<EnumValue, ProcessMemoryError> {
    Ok(EnumValue {
        name: reader.read_cstring(enum_value_info.name as usize)?,
        value: enum_value_info.value,
        hash: enum_value_hash,
    })
}
