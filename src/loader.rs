use super::rlua;
// use super::includedir;

use std::str;

include!(concat!(env!("OUT_DIR"), "/data.rs"));

/// Loads all libraries in the data path.
pub fn load_all(vm: &rlua::Lua) -> Result<(), String> {
    for f in FILES.file_names() {
        if let Err(e) = load_lib(&vm, f) {
            return Err(format!("failed to load {}: {}", f, e));
        }
    }

    Ok(())
}

/// Attempts to load the bindata with the given filename into the given
/// rlua::Lua vm.
pub fn load_lib(vm: &rlua::Lua, lib: &str) -> Result<(), String> {
    let lbs = FILES
        .get(lib)
        .expect("failed to get binary data")
        .into_owned();
    let lstr = str::from_utf8(&lbs).expect("failed to parse UTF-8");

    let llib = vm.load(lstr, None)
        .expect("Lua load failed")
        .call::<(), rlua::Table>(())
        .expect("Lua require failed");

    Ok(vm.globals().set("inspect", llib).expect(
        "failed to set library",
    ))
}

#[cfg(test)]
mod test {
    #[test]
    fn test_load_lib() {
        assert_eq!(2, 2);
    }
}
