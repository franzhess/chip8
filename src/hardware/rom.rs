use std::{fs, io, path::Path};

pub fn load(path: &Path) -> Result<Vec<u8>, io::Error> {
    fs::read(path).map_err(|error| {
        io::Error::new(
            error.kind(),
            format!("cannot read ROM '{}': {error}", path.display()),
        )
    })
}
