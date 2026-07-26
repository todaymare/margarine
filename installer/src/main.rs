use std::{fs::create_dir_all, path::Path};

use flate2::read::GzDecoder;
use margarine_installer::{executable_name, path_bin, path_cache, path_lib, path_share, static_library_name};
use tar::Archive;
use tempfile::tempfile;

const MARGARINE_INSTALL_VERSION : &str = env!("MARGARINE_INSTALL_VERSION");
const MARGARINE_INSTALL_TARGET : &str = env!("MARGARINE_INSTALL_TARGET");
const MARGARINE_INSTALL_PAYLOAD : &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/payload.tar.gz"));

fn main() {
    println!("installing margarine@{MARGARINE_INSTALL_VERSION}-{MARGARINE_INSTALL_TARGET}");

    let file = GzDecoder::new(MARGARINE_INSTALL_PAYLOAD);
    let mut archive = Archive::new(file);

    let mut temp_dir = tempfile::Builder::new()
        .prefix("margarine-")
        .tempdir().unwrap();
    archive.unpack(temp_dir.path()).unwrap();

    temp_dir.disable_cleanup(true);

    println!("finalising");

    create_dir_all(path_lib(MARGARINE_INSTALL_TARGET)).unwrap();
    create_dir_all(path_bin()).unwrap();
    create_dir_all(path_cache()).unwrap();

    std::fs::copy(temp_dir.path().join(static_library_name()), path_lib(MARGARINE_INSTALL_TARGET).join(static_library_name())).unwrap();
    std::fs::copy(temp_dir.path().join("margarine"), path_bin().join(executable_name())).unwrap();

    let _ = std::fs::remove_dir_all(path_share());
    copy_dir_all(
        temp_dir.path().join("share"),
        path_share(),
    ).unwrap();

    println!("installed margarine at {}", path_bin().join(executable_name()).to_string_lossy());


}



fn copy_dir_all(src: impl AsRef<Path>, dst: impl AsRef<Path>) -> std::io::Result<()> {
    std::fs::create_dir_all(&dst)?;

    for entry in std::fs::read_dir(src)? {
        let entry = entry?;
        let ty = entry.file_type()?;

        let dest = dst.as_ref().join(entry.file_name());

        if ty.is_dir() {
            copy_dir_all(entry.path(), dest)?;
        } else {
            std::fs::copy(entry.path(), dest)?;
        }
    }

    Ok(())
}
