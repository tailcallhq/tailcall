#[cfg(feature = "compression")]
use libflate::gzip;

#[cfg(feature = "compression")]
const TARGET_LOG_COMPRESSION: &str = "apollo-studio-extension-compression";

#[cfg(feature = "compression")]
pub fn compress(msg: Vec<u8>) -> Result<Vec<u8>, std::io::Error> {
    let mut encoder = gzip::Encoder::new(Vec::new()).unwrap();
    let mut msg = std::io::Cursor::new(msg);

    match std::io::copy(&mut msg, &mut encoder) {
        Ok(_) => {}
        Err(e) => {
            error!(target: TARGET_LOG_COMPRESSION, message = "An issue happened while GZIP compression", err = ?e);
            return Err(e);
        }
    };

    encoder.finish().into_result()
}

#[cfg(not(feature = "compression"))]
pub fn compress(msg: Vec<u8>) -> Result<Vec<u8>, std::io::Error> {
    Ok::<Vec<u8>, std::io::Error>(msg)
}
