use crate::TypeInfoGenerated;
use crate::reader::{ProcessMemoryError, ProcessMemoryReader};
use serde::Serialize;
use std::ffi::c_char;
use std::option::Option;

#[derive(Serialize)]
pub struct Class {
    name: String,
    super_type: Option<String>,
    hash: u32,
    size: u32,
    template_parms: Vec<ClassVariable>,
    variables: Vec<ClassVariable>,
    checksum: u64,
    meta_data: Option<String>,
}

#[derive(Serialize)]
pub struct ClassVariable {
    r#type: String,
    name: String,
    ops: Option<String>,
    offset: u32,
    size: u32,
    flags: u64,
    comment: Option<String>,
    hash: Option<u64>,
}

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct ClassTypeInfo {
    name: *const c_char,
    super_type: *const c_char,
    super_type_type_info_tools_index: u32,
    name_hash: u32,
    size: u32,
    template_parms: *const ClassVariableInfo,
    variables: *const ClassVariableInfo,
    variable_name_hashes: *const u64,
    class_checksum: u64,
    create_instance: usize,
    placement_create_instance: usize,
    meta_data: *const ClassMetaDataInfo,
}

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct ClassVariableInfo {
    r#type: *const c_char,
    ops: *const c_char,
    name: *const c_char,
    offset: u32,
    size: u32,
    class_type_info_tools_index: u32,
    enum_type_info_tools_index: u32,
    flags: u64,
    comment: *const c_char,
    get: usize,
    set: usize,
    reallocate: usize,
    merge: usize,
}

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct ClassMetaDataInfo {
    meta_data: *const c_char,
}

const _: () = assert!(size_of::<ClassTypeInfo>() == 88);
const _: () = assert!(size_of::<ClassVariableInfo>() == 88);

pub fn read_classes(
    reader: &ProcessMemoryReader,
    type_info_generated: &TypeInfoGenerated,
) -> Result<Vec<Class>, ProcessMemoryError> {
    let class_type_infos = reader.read_structs::<ClassTypeInfo>(
        type_info_generated.classes,
        type_info_generated.num_classes.saturating_sub(1),
    )?;

    class_type_infos
        .iter()
        .map(|info| read_class(reader, info))
        .collect()
}

fn read_class(
    reader: &ProcessMemoryReader,
    class_type_info: &ClassTypeInfo,
) -> Result<Class, ProcessMemoryError> {
    let meta_data: Option<String> = if class_type_info.meta_data.is_null() {
        None
    } else {
        let meta_data_info = reader.read_struct(class_type_info.meta_data)?;
        if meta_data_info.meta_data.is_null() {
            None
        } else {
            Some(reader.read_cstring(meta_data_info.meta_data)?)
        }
    };

    let template_parms = if class_type_info.template_parms.is_null() {
        Vec::new()
    } else {
        read_class_template_parms(reader, class_type_info)?
    };

    let variables = if class_type_info.variables.is_null() {
        Vec::new()
    } else {
        read_class_variables(reader, class_type_info)?
    };

    Ok(Class {
        name: reader.read_cstring(class_type_info.name)?,
        super_type: reader.read_cstring(class_type_info.super_type).ok(),
        hash: class_type_info.name_hash,
        size: class_type_info.size,
        template_parms,
        variables,
        checksum: class_type_info.class_checksum,
        meta_data,
    })
}

fn ptr_or_else<T, R, F1, F2>(ptr: *const T, if_null: F1, if_not_null: F2) -> R
where
    F1: FnOnce() -> R,
    F2: FnOnce(*const T) -> R,
{
    if ptr.is_null() {
        if_null()
    } else {
        if_not_null(ptr)
    }
}

fn read_class_template_parms(
    reader: &ProcessMemoryReader,
    class_type_info: &ClassTypeInfo,
) -> Result<Vec<ClassVariable>, ProcessMemoryError> {
    let mut result = Vec::new();
    for i in 0.. {
        let variable = reader.read_struct(class_type_info.template_parms.wrapping_add(i))?;
        if variable.r#type.is_null() {
            break;
        }

        result.push(read_class_variable(reader, &variable, None)?);
    }
    Ok(result)
}

fn read_class_variables(
    reader: &ProcessMemoryReader,
    class_type_info: &ClassTypeInfo,
) -> Result<Vec<ClassVariable>, ProcessMemoryError> {
    let mut result = Vec::new();
    for i in 0.. {
        let variable = reader.read_struct(class_type_info.variables.wrapping_add(i))?;
        if variable.r#type.is_null() {
            break;
        }

        let hash = reader.read_struct(class_type_info.variable_name_hashes.wrapping_add(i))?;
        result.push(read_class_variable(reader, &variable, Some(hash))?);
    }
    Ok(result)
}

fn read_class_variable(
    reader: &ProcessMemoryReader,
    info: &ClassVariableInfo,
    hash: Option<u64>,
) -> Result<ClassVariable, ProcessMemoryError> {
    Ok(ClassVariable {
        r#type: reader.read_cstring(info.r#type)?,
        name: reader.read_cstring(info.name)?,
        ops: reader.read_cstring(info.ops).ok(),
        offset: info.offset,
        size: info.size,
        flags: info.flags,
        comment: reader.read_cstring(info.comment).ok(),
        hash,
    })
}
