use crate::classes::{Class, ClassTypeInfo, read_classes};
use crate::enums::{Enum, EnumTypeInfo, read_enums};
use crate::reader::{ProcessMemoryError, ProcessMemoryReader};
use crate::typedefs::{TypeDef, TypeDefInfo};
use serde::Serialize;
use std::ffi::c_char;
use std::fs::File;
use std::io::BufWriter;

mod classes;
mod enums;
mod reader;
mod typedefs;

fn main() {
    assert_eq!(size_of::<TypeInfoGenerated>(), 88);

    let addresses: Vec<usize> = vec![0x146310F50, 0x14633FD00];

    let reader = ProcessMemoryReader::new(6068).expect("Could not create process memory reader");
    let type_infos: Vec<TypeInfo> = addresses
        .into_iter()
        .map(|address| read_type_info(&reader, address).expect("Could not read type info"))
        .collect();

    let writer = BufWriter::new(File::create("idlib.json").expect("Could not create idlib.json"));
    serde_json::to_writer_pretty(writer, &type_infos).expect("Error serializing type_infos.");
}

fn read_type_info(
    reader: &ProcessMemoryReader,
    address: usize,
) -> Result<TypeInfo, ProcessMemoryError> {
    let type_info_generated = reader.read_struct::<TypeInfoGenerated>(address)?;
    println!("{:#?}", type_info_generated);

    let project_name = reader.read_cstring(type_info_generated.project_name as usize)?;
    let classes = read_classes(reader, &type_info_generated)?;
    let enums = read_enums(reader, &type_info_generated)?;
    // let typedefs = read_typedefs(reader, &type_info_generated)?;

    Ok(TypeInfo {
        project_name,
        classes,
        enums,
        typedefs: Vec::new(),
    })
}

#[derive(Serialize)]
struct TypeInfo {
    project_name: String,
    classes: Vec<Class>,
    enums: Vec<Enum>,
    typedefs: Vec<TypeDef>,
}

#[repr(C)]
#[derive(Clone, Debug)]
struct TypeInfoGenerated {
    project_name: *const c_char,
    enums: *const EnumTypeInfo,
    num_enums: i32,
    classes: *const ClassTypeInfo,
    num_classes: i32,
    typedefs: *const TypeDefInfo,
    num_typedefs: i32,
    render_model_ctors: *const u8,
    num_render_model_ctors: i32,
    logic_custom_event_declarations: *const u8,
    num_logic_custom_event_declarations: i32,
}
