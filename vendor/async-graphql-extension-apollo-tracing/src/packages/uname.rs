
cfg_if::cfg_if! {
    if #[cfg(any(target_arch = "wasm32", target_os = "windows"))] {
        pub fn uname() -> std::io::Result<String> {
            Err(std::io::Error::new(std::io::ErrorKind::NotFound, "not supported on wasm32"))
        }
    } else {
         pub fn uname() -> std::io::Result<String> {
            let x = uname::uname()?;
            Ok(format!(
                "{sysname} {version} {release} {machine} {nodename}",
                sysname = x.sysname,
                version = x.version,
                release = x.release,
                machine = x.machine,
                nodename = x.nodename
            ))
        }
    }
}
