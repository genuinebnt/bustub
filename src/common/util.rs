use std::os::unix::fs::FileExt;

pub fn read_exact_at(file: &std::fs::File, mut buf: &mut [u8], mut offset: u64) -> std::io::Result<u64> {
    let mut bytes_read = 0;

    while !buf.is_empty() {
        match file.read_at(buf, offset)? {
            0 => break,
            n => {
                offset += n as u64;
                bytes_read += n as u64;
                buf = &mut buf[n..];
            }
        }
    }

    Ok(bytes_read)
}
