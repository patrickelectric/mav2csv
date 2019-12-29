use std::io;
use std::sync::Mutex;

pub fn open(file_reader: &stdweb::web::FileReader) -> io::Result<FileConnection> {
    let content: Vec<u8> = match file_reader.result() {
        Some(stdweb::web::FileReaderResult::ArrayBuffer(buffer)) => buffer.into(),
        _ => {
            unreachable!();
        }
    };

    Ok(FileConnection {
        content: Mutex::new(io::Cursor::new(content)),
        protocol_version: mavlink::MavlinkVersion::V2,
    })
}

pub struct FileConnection {
    content: Mutex<io::Cursor<Vec<u8>>>,
    protocol_version: mavlink::MavlinkVersion,
}

impl mavlink::MavConnection for FileConnection {
    fn recv(&self) -> io::Result<(mavlink::MavHeader, mavlink::MavMessage)> {
        let mut content = self.content.lock().unwrap();

        loop {
            match mavlink::read_versioned_msg(&mut *content, self.protocol_version) {
                Ok((h, m)) => {
                    return Ok((h, m));
                }
                Err(e) => match e.kind() {
                    io::ErrorKind::UnexpectedEof => {
                        return Err(e);
                    }
                    _ => {}
                },
            }
        }
    }

    fn send(&self, _header: &mavlink::MavHeader, _data: &mavlink::MavMessage) -> io::Result<()> {
        Ok(())
    }

    fn set_protocol_version(&mut self, version: mavlink::MavlinkVersion) {
        self.protocol_version = version;
    }

    fn get_protocol_version(&self) -> mavlink::MavlinkVersion {
        self.protocol_version
    }
}
