use camino::Utf8Path;
use sha1_smol::Sha1;

pub const BIZHAWK_BASH_DEFAULT: &'static [u8] = include_bytes!("includes/start-bizhawk.sh");
pub const BIZHAWK_BASH_PRE290: &'static [u8] = include_bytes!("includes/start-bizhawk-pre290.sh");

/// Writes data to the destination path, replacing if destination file exists and SHA1 mismatches.
/// 
/// Does _not_ create missing parent directories!
pub fn copy_if_different<P: AsRef<Utf8Path>>(data: &[u8], dest: P) -> std::io::Result<()> {
    let dest = dest.as_ref();
    
    if dest.is_file() {
        let old = std::fs::read(&dest)?;
        let hash_new = Sha1::from(&data).digest();
        let hash_old = Sha1::from(&old).digest();
        
        if hash_new == hash_old {
            return Ok(());
        }
    }
    
    std::fs::write(dest, data)
}