use std::any::type_name;
use windows::Win32::Foundation::{CloseHandle, HANDLE};
use windows::Win32::System::Diagnostics::Debug::ReadProcessMemory;
use windows::Win32::System::Threading::{OpenProcess, PROCESS_QUERY_INFORMATION, PROCESS_VM_READ};

pub struct ProcessMemoryReader {
    process_handle: HANDLE,
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

    pub fn read_bytes(&self, address: usize, size: usize) -> windows::core::Result<Vec<u8>> {
        let mut buffer = vec![0u8; size];
        let mut bytes_read = 0;

        unsafe {
            ReadProcessMemory(
                self.process_handle,
                address as *const _,
                buffer.as_mut_ptr() as *mut _,
                size,
                Some(&mut bytes_read),
            )?;
        }

        buffer.truncate(bytes_read);
        Ok(buffer)
    }

    pub fn read_struct<T>(&self, address: usize) -> windows::core::Result<T> {
        let bytes = self.read_bytes(address, size_of::<T>())?;

        if bytes.len() != size_of::<T>() {
            eprintln!("Failed to read struct of type {}", type_name::<T>());
            return Err(windows::core::Error::from_win32());
        }

        let struct_data = unsafe { std::ptr::read(bytes.as_ptr() as *const T) };

        Ok(struct_data)
    }

    pub fn read_structs<T>(&self, address: usize, count: usize) -> windows::core::Result<Vec<T>> {
        let struct_size = size_of::<T>();
        let total_size = struct_size * count;
        let bytes = self.read_bytes(address, total_size)?;

        if bytes.len() != total_size {
            eprintln!(
                "Failed to read {} structs of type {}",
                count,
                type_name::<T>()
            );
            return Err(windows::core::Error::from_win32());
        }

        let mut results = Vec::with_capacity(count);
        for i in 0..count {
            let offset = i * struct_size;
            let struct_data = unsafe { std::ptr::read(bytes.as_ptr().add(offset) as *const T) };
            results.push(struct_data);
        }
        Ok(results)
    }

    pub fn read_cstring(&self, address: usize) -> windows::core::Result<Option<String>> {
        if address == 0 {
            return Ok(None);
        }
        let bytes = self
            .read_bytes(address, 1024)
            .expect(format!("Failed to read CString at {}", address).as_str());

        let null_pos = bytes.iter().position(|&c| c == 0).unwrap_or(bytes.len());
        if null_pos == 0 {
            return Ok(None);
        }
        let string_bytes = &bytes[..null_pos];

        Ok(Some(String::from_utf8_lossy(string_bytes).to_string()))
    }
}

impl Drop for ProcessMemoryReader {
    fn drop(&mut self) {
        unsafe {
            let _ = CloseHandle(self.process_handle);
        }
    }
}
