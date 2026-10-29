use crate::classes::{Class, ClassTypeInfo, read_classes};
use crate::enums::{Enum, EnumTypeInfo, read_enums};
use crate::reader::{ProcessMemoryError, ProcessMemoryReader};
use crate::typedefs::{TypeDef, TypeDefInfo};
use serde::Serialize;
use std::ffi::c_char;
use std::num::ParseIntError;
use std::process::ExitCode;
use std::ptr;
use thiserror::Error;

mod classes;
mod enums;
mod reader;
mod typedefs;

const _: () = assert!(size_of::<usize>() == 8, "Only works on 64 bit");
const _: () = assert!(size_of::<TypeInfoGenerated>() == 88);

const OUTPUT_PATH: &str = "idlib.json";

const TYPE_INFO_ADDRESSES: [usize; 2] = [0x0000_0001_4675_ABC0, 0x0000_0001_4678_D9D0];

#[derive(Debug, Error)]
enum AppError {
    #[error("PID must be an integer ({0})")]
    InvalidPid(#[from] ParseIntError),
    #[error("Could not open process ({0})")]
    OpenProcess(windows::core::Error),
    #[error("Could not read type info ({0})")]
    ReadTypeInfo(#[from] ProcessMemoryError),
    #[error("Could not create {OUTPUT_PATH} ({0})")]
    CreateOutput(#[from] std::io::Error),
    #[error("Could not serialize type infos ({0})")]
    Serialize(#[from] serde_json::Error),
}

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().collect();

    let Some(pid) = args.get(1) else {
        let program = args.first().map_or("tda-rtti", String::as_str);
        eprintln!("Usage: {program} <pid>");
        return ExitCode::FAILURE;
    };

    match run(pid) {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("Error: {error}");
            ExitCode::FAILURE
        }
    }
}

fn run(pid: &str) -> Result<(), AppError> {
    let reader = ProcessMemoryReader::new(pid.parse::<u32>()?).map_err(AppError::OpenProcess)?;

    let type_infos = TYPE_INFO_ADDRESSES
        .into_iter()
        .map(|address| read_type_info(&reader, ptr::without_provenance(address)))
        .collect::<Result<Vec<TypeInfo>, ProcessMemoryError>>()?;

    let writer = std::io::BufWriter::new(std::fs::File::create(OUTPUT_PATH)?);
    serde_json::to_writer_pretty(writer, &type_infos)?;

    Ok(())
}

fn read_type_info(
    reader: &ProcessMemoryReader,
    address: *const TypeInfoGenerated,
) -> Result<TypeInfo, ProcessMemoryError> {
    let type_info_generated = reader.read_struct(address)?;
    println!("{type_info_generated:#?}");

    let project_name = reader.read_cstring(type_info_generated.project_name)?;
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
#[derive(Clone, Copy, Debug)]
struct TypeInfoGenerated {
    project_name: *const c_char,
    enums: *const EnumTypeInfo,
    num_enums: u32,
    classes: *const ClassTypeInfo,
    num_classes: u32,
    typedefs: *const TypeDefInfo,
    num_typedefs: u32,
    render_model_ctors: *const u8,
    num_render_model_ctors: u32,
    logic_custom_event_declarations: *const u8,
    num_logic_custom_event_declarations: u32,
}
