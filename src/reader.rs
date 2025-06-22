use crate::reader::ProcessMemoryError::{CouldNotReadProcessMemory, InvalidString, NotEnoughBytes};
use std::string::FromUtf8Error;
use thiserror::Error;
use windows::Win32::Foundation::{CloseHandle, HANDLE};
use windows::Win32::System::Diagnostics::Debug::ReadProcessMemory;
use windows::Win32::System::Threading::{OpenProcess, PROCESS_QUERY_INFORMATION, PROCESS_VM_READ};

pub struct ProcessMemoryReader {
    process_handle: HANDLE,
}

#[derive(Debug, Error)]
pub(crate) enum ProcessMemoryError {
    #[error("Could not read process memory")]
    CouldNotReadProcessMemory(windows::core::Error),
    #[error("Not enough bytes read, expected {expected} but read {actual}")]
    NotEnoughBytes { expected: usize, actual: usize },
    #[error("Invalid UTF8 String ({0})")]
    InvalidString(#[from] FromUtf8Error),
    #[error("String required but was null")]
    EmptyString,
}

impl ProcessMemoryReader {
    pub fn new(process_id: u32) -> windows::core::Result<Self> {
        let process_handle = unsafe {
            OpenProcess(
                PROCESS_VM_READ | PROCESS_QUERY_INFORMATION,
                false,
                process_id,
            )?
        };

        Ok(ProcessMemoryReader { process_handle })
    }

    pub fn read_bytes(&self, address: usize, size: usize) -> Result<Vec<u8>, ProcessMemoryError> {
        let mut buffer = vec![0u8; size];
        let mut bytes_read = 0;

        unsafe {
            ReadProcessMemory(
                self.process_handle,
                address as *const _,
                buffer.as_mut_ptr() as *mut _,
                size,
                Some(&mut bytes_read),
            )
                .map_err(CouldNotReadProcessMemory)?;
        }

        if bytes_read != size {
            return Err(NotEnoughBytes {
                expected: size,
                actual: bytes_read,
            });
        }

        buffer.truncate(bytes_read);
        Ok(buffer)
    }

    pub fn read_struct<T: Copy>(&self, address: usize) -> Result<T, ProcessMemoryError> {
        let bytes = self.read_bytes(address, size_of::<T>())?;
        let struct_data = unsafe { std::ptr::read(bytes.as_ptr() as *const T) };
        Ok(struct_data)
    }

    pub fn read_structs<T: Copy>(
        &self,
        address: usize,
        count: usize,
    ) -> Result<Vec<T>, ProcessMemoryError> {
        let struct_size = size_of::<T>();
        let bytes = self.read_bytes(address, struct_size * count)?;

        let results: Vec<T> = (0..count)
            .map(|i| unsafe { std::ptr::read(bytes.as_ptr().add(i * struct_size) as *const T) })
            .collect();
        Ok(results)
    }

    pub fn read_cstring(&self, address: usize) -> Result<String, ProcessMemoryError> {
        if address == 0 {
            return Err(ProcessMemoryError::EmptyString);
        }

        let bytes = self.read_bytes(address, 1024)?;
        let null_index = bytes.iter().position(|&c| c == 0).unwrap_or(bytes.len());
        String::from_utf8(bytes[..null_index].to_vec()).map_err(InvalidString)
    }
}

impl Drop for ProcessMemoryReader {
    fn drop(&mut self) {
        unsafe {
            let _ = CloseHandle(self.process_handle);
        }
    }
}
