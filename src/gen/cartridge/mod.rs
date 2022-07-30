use std::io::Read;

use simple_error::{SimpleError, SimpleResult};

pub fn read(src: &mut dyn Read, _save_data: Option<&mut dyn Read>) -> SimpleResult<Box<[u8]>> {
    let mut contents = Vec::new();
    src.read_to_end(&mut contents)
        .expect("error reading source");
    if contents[0x100..0x104] != [0x53, 0x45, 0x47, 0x41] {
        return Err(SimpleError::new("Not a Genesis/Mega Drive file."));
    }
    Ok(contents.into_boxed_slice())
}