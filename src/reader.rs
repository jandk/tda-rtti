use crate::reader::ProcessMemoryError::{CouldNotReadProcessMemory, InvalidString, NotEnoughBytes};
use std::ffi::{c_char, c_void};
use std::num::TryFromIntError;
use std::ptr;
use std::string::FromUtf8Error;
use thiserror::Error;
use windows::Win32::Foundation::{CloseHandle, HANDLE};
use windows::Win32::System::Diagnostics::Debug::ReadProcessMemory;
use windows::Win32::System::Threading::{OpenProcess, PROCESS_QUERY_INFORMATION, PROCESS_VM_READ};

const MAX_CSTRING_LEN: usize = 4096;

pub struct ProcessMemoryReader {
    process_handle: HANDLE,
}

#[derive(Debug, Error)]
pub enum ProcessMemoryError {
    #[error("Could not read process memory")]
    CouldNotReadProcessMemory(windows::core::Error),
    #[error("Not enough bytes read, expected {expected} but read {actual}")]
    NotEnoughBytes { expected: usize, actual: usize },
    #[error("Invalid UTF8 String ({0})")]
    InvalidString(#[from] FromUtf8Error),
    #[error("String required but was null")]
    EmptyString,
    #[error("Reading {count} elements of {size} bytes overflows the address space")]
    SizeOverflow { count: usize, size: usize },
    #[error("Count does not fit in a pointer-sized integer ({0})")]
    CountTooLarge(#[from] TryFromIntError),
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

        Ok(Self { process_handle })
    }

    fn read_bytes(&self, address: usize, size: usize) -> Result<Vec<u8>, ProcessMemoryError> {
        let mut buffer = vec![0u8; size];
        let mut bytes_read = 0usize;

        unsafe {
            ReadProcessMemory(
                self.process_handle,
                ptr::without_provenance(address),
                buffer.as_mut_ptr().cast::<c_void>(),
                size,
                Some(&raw mut bytes_read),
            )
            .map_err(CouldNotReadProcessMemory)?;
        }

        if bytes_read != size {
            return Err(NotEnoughBytes {
                expected: size,
                actual: bytes_read,
            });
        }

        Ok(buffer)
    }

    pub fn read_struct<T: Copy>(&self, address: *const T) -> Result<T, ProcessMemoryError> {
        let bytes = self.read_bytes(address.addr(), size_of::<T>())?;
        Ok(unsafe { bytes.as_ptr().cast::<T>().read_unaligned() })
    }

    pub fn read_structs<T: Copy>(
        &self,
        address: *const T,
        count: u32,
    ) -> Result<Vec<T>, ProcessMemoryError> {
        let count = usize::try_from(count)?;
        let size = size_of::<T>();
        let total = size
            .checked_mul(count)
            .ok_or(ProcessMemoryError::SizeOverflow { count, size })?;
        let bytes = self.read_bytes(address.addr(), total)?;

        Ok((0..count)
            // Safety: the buffer holds exactly `count` values of `size` bytes each.
            .map(|i| unsafe { bytes.as_ptr().cast::<T>().add(i).read_unaligned() })
            .collect())
    }

    pub fn read_cstring(&self, address: *const c_char) -> Result<String, ProcessMemoryError> {
        if address.is_null() {
            return Err(ProcessMemoryError::EmptyString);
        }

        let bytes = self.read_bytes(address.addr(), MAX_CSTRING_LEN)?;
        let text = bytes.split(|&c| c == 0).next().unwrap_or_default();
        String::from_utf8(text.to_vec()).map_err(InvalidString)
    }
}

impl Drop for ProcessMemoryReader {
    fn drop(&mut self) {
        unsafe {
            let _ = CloseHandle(self.process_handle);
        }
    }
}
