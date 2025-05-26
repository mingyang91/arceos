// Legacy SBI console implementation - no need for memory address conversion

/// The maximum number of bytes that can be read at once.
const MAX_RW_SIZE: usize = 256;

/// Writes a byte to the console.
pub fn putchar(c: u8) {
    sbi_rt::legacy::console_putchar(c as usize);
}

/// Tries to write bytes to the console from input u8 slice.
/// Returns the number of bytes written.
fn try_write_bytes(bytes: &[u8]) -> usize {
    // Legacy SBI doesn't have bulk write, so write byte by byte
    let len = bytes.len().min(MAX_RW_SIZE);
    for &byte in &bytes[..len] {
        sbi_rt::legacy::console_putchar(byte as usize);
    }
    len
}

/// Writes bytes to the console from input u8 slice.
pub fn write_bytes(bytes: &[u8]) {
    let mut write_len = 0;
    while write_len < bytes.len() {
        let len = try_write_bytes(&bytes[write_len..]);
        if len == 0 {
            break;
        }
        write_len += len;
    }
}

/// Reads bytes from the console into the given mutable slice.
/// Returns the number of bytes read.
pub fn read_bytes(bytes: &mut [u8]) -> usize {
    // Legacy SBI doesn't have bulk read, so read byte by byte
    let max_len = bytes.len().min(MAX_RW_SIZE);
    for i in 0..max_len {
        let ch = sbi_rt::legacy::console_getchar();
        if ch == usize::MAX {
            // No character available
            return i;
        }
        bytes[i] = ch as u8;
        // Stop on newline or carriage return
        if ch == b'\n' as usize || ch == b'\r' as usize {
            return i + 1;
        }
    }
    max_len
}
