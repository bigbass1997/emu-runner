use camino::Utf8PathBuf;
use crate::{EmulatorContext, Error};
use crate::includes::{BIZHAWK_BASH_DEFAULT, BIZHAWK_BASH_PRE290, copy_if_different};

#[derive(Debug, Clone, PartialEq)]
pub struct BizHawkContext {
    pub config: Option<Utf8PathBuf>,
    pub movie: Option<Utf8PathBuf>,
    pub lua: Option<Utf8PathBuf>,
    pub rom: Option<Utf8PathBuf>,
    pub working_dir: Utf8PathBuf,
}
impl EmulatorContext for BizHawkContext {
    fn cmd_name(&self) -> String {
        #[cfg(target_family = "unix")]
        { "bash".into() }
        
        #[cfg(target_family = "windows")]
        { "EmuHawk.exe".into() }
    }
    
    fn args(&self) -> Vec<String> {
        let mut args = Vec::with_capacity(5);
        
        #[cfg(target_family = "unix")]
        {
            args.push("start-bizhawk.sh".into());
        }
        
        if let Some(config) = self.config.as_ref() {
            args.push(format!("--config={config}"));
        }
        if let Some(movie) = self.movie.as_ref() {
            args.push(format!("--movie={movie}"));
        }
        if let Some(lua) = self.lua.as_ref() {
            args.push(format!("--lua={lua}"));
        }
        if let Some(rom) = self.rom.as_ref() {
            args.push(rom.to_string());
        }
        
        args
    }
    
    fn env(&self) -> Vec<(String, String)> {
        vec![]
    }

    fn prepare(&mut self) -> Result<(), Error> {
        // BizHawk accepts configs/movies/scripts/roms from anywhere,
        // so we only need to verify they exist.
        // However, since we change the working directory, and there's no
        // easy way to test if file exists relative to a different dir,
        // the paths _should_ be absolute, either originally or via the with_* functions.
        
        if let Some(config) = self.config.as_ref() {
            if !config.is_file() {
                return Err(Error::MissingConfig(config.clone()));
            }
            if !config.is_absolute() {
                return Err(Error::AbsolutePathFailed);
            }
        }
        if let Some(movie) = self.movie.as_ref() {
            if !movie.is_file() {
                return Err(Error::MissingMovie(movie.clone()));
            }
            if !movie.is_absolute() {
                return Err(Error::AbsolutePathFailed);
            }
        }
        if let Some(lua) = self.lua.as_ref() {
            if !lua.is_file() {
                return Err(Error::MissingLua(lua.clone()));
            }
            if !lua.is_absolute() {
                return Err(Error::AbsolutePathFailed);
            }
        }
        if let Some(rom) = self.rom.as_ref() {
            if !rom.is_file() {
                return Err(Error::MissingRom(rom.clone()));
            }
            if !rom.is_absolute() {
                return Err(Error::AbsolutePathFailed);
            }
        }
        
        // If unix, copy bash script and check for incompatible versions
        #[cfg(target_family = "unix")]
        {
            let bash = match self.detect_version() {
                Some(ver) => match ver.as_str() {
                    "2.8" | "2.8-rc1" | "2.7" | "2.6.3" | "2.6.2" | "2.6.1" | "2.6" => BIZHAWK_BASH_PRE290,
                    
                    "2.5.2" | "2.5.1" | "2.5.0" | "2.4.2" | "2.4.1" | "2.4" | "2.3.3"
                        | "2.3.2" | "2.3.1" | "2.3" | "2.2.2" | "2.2.1" | "2.2" | "2.1.1"
                        | "2.1.0" | "1.13.2" | "1.9.2" | "1.6.1" => return Err(Error::IncompatibleOSVersion),
                    
                    _ => BIZHAWK_BASH_DEFAULT,
                },
                None => BIZHAWK_BASH_DEFAULT
            };
            
            let mut path = self.working_dir.clone();
            path.push("start-bizhawk.sh");
            copy_if_different(bash, path)?;
        }
        
        Ok(())
    }

    fn working_dir(&self) -> Utf8PathBuf {
        self.working_dir.clone()
    }
}
impl BizHawkContext {
    /// Creates a new Context with default options.
    /// 
    /// If the path does not point to a directory, or a file within a directory, which contains `EmuHawk.exe`,
    /// an error will be returned.
    pub fn new<P: Into<Utf8PathBuf>>(working_dir: P) -> Result<Self, Error> {
        let mut working_dir = working_dir.into();
        if working_dir.is_file() {
            working_dir.pop();
        }
        
        working_dir = working_dir.canonicalize_utf8().unwrap_or(working_dir);
        
        let mut detect_exe = working_dir.clone();
        detect_exe.push("EmuHawk.exe");
        if working_dir.is_file() || !working_dir.exists() || !detect_exe.is_file() {
            return Err(Error::MissingExecutable(detect_exe));
        }
        
        Ok(Self {
            config: None,
            movie: None,
            lua: None,
            rom: None,
            working_dir,
        })
    }
    
    pub fn with_config<P: Into<Utf8PathBuf>>(self, config: P) -> Self {
        let config = config.into();
        Self {
            config: Some(config.canonicalize_utf8().unwrap_or_else(|_| config)),
            ..self
        }
    }
    
    pub fn with_movie<P: Into<Utf8PathBuf>>(self, movie: P) -> Self {
        let movie = movie.into();
        Self {
            movie: Some(movie.canonicalize_utf8().unwrap_or_else(|_| movie)),
            ..self
        }
    }
    
    pub fn with_lua<P: Into<Utf8PathBuf>>(self, lua: P) -> Self {
        let lua = lua.into();
        Self {
            lua: Some(lua.canonicalize_utf8().unwrap_or_else(|_| lua)),
            ..self
        }
    }
    
    pub fn with_rom<P: Into<Utf8PathBuf>>(self, rom: P) -> Self {
        let rom = rom.into();
        Self {
            rom: Some(rom.canonicalize_utf8().unwrap_or_else(|_| rom)),
            ..self
        }
    }
    
    /// Determines the emulator version by comparing the SHA1 checksum of `EmuHawk.exe`
    pub fn detect_version(&self) -> Option<String> {
        let mut exe = self.working_dir.clone();
        exe.push("EmuHawk.exe");
        let exe = std::fs::read(exe).unwrap();
        let sha1 = sha1_smol::Sha1::from(exe).digest().to_string().to_lowercase();
        
        Some(match sha1.as_str() {
            "ef7c4067cec01b60b89ff8c271f3c72a0b2d009f" => "2.9.1",
            "c9be7f8e4a05122e60545e8988920b710bd50ea7" => "2.9",
            "31a2fadd049957377358a0b7b1267f3d8ecebfd9" => "2.9-rc3",
            "a6ff6e02a05a0ec52695a7ec757aedcdc16e0192" => "2.9-rc2",
            "288e310c430cbcbc0881913efd82e91f16dc14dd" => "2.9-rc1",
            "88e476295d004a80ea514847c0d590879e7b3d88" => "2.8",
            "9d2738265a37e28813eeff08e41def697f58cbee" => "2.8-rc1",
            "eac6aa28589372d120e23e5b2f69b56c2542273b" => "2.7",
            "3261214b9991918c5224d27b6cf7d84f9acd3566" => "1.9.2",
            "202a0d945cd20a1b2e5021d3499ac7b5c2f5ca46" => "1.6.1",
            "7dd9dce90e16138ca38ef92cdb1270a378d21dad" => "2.6.3",
            "3668613ed1fc61f1dafde9b678e6a637da23d882" => "2.6.2",
            "115cb73156b4a288378fd00aa0fd982fb0c311c5" => "2.6.1",
            "307526d8171fa9aa2dfbf735aa1eca23425b829a" => "2.6",
            "7bcc6337005dba33fbc8a454cf7f669563f39e85" => "2.5.2",
            "410c423feef9666955b2a0d66c3b64c3e432988a" => "2.5.1",
            "d45a7348a8e5505b294df9add852787d04b569e4" => "2.5.0",
            "6e169792aebef5942c9fabd276c7d3e07e2c3196" => "2.4.2",
            "71d9bd1ae6d60b6fc7d3aebe474eae50995c29d7" => "2.4.1",
            "2668ef81bad2459a9a14a09a3a8d5ee2c6e9cbac" => "2.4",
            "1fbf1b672ddb4e98aef77a8edd5655149b4b4c72" => "2.3.3",
            "d9365fd6f1f979a52689979e5709b26dfef7dc09" => "2.3.2",
            "c2f4b95b86a11c472f7e8522598be644e9e05c6d" => "2.3.1",
            "b2072e0bdf4944d060c83f44df17e88da4007c81" => "2.3",
            "ab83cfd3ed5b9dc392d2b0d4aa1b99723c5bf4c9" => "1.13.2",
            "4c1599c7ed7e5216477454ac7fac0719f2ee6e66" => "2.2.2",
            "6095cb07bd79703527c01ad4f27ed4e907d2f030" => "2.2.1",
            "4e2ec35bff8798494d3cc0e22276f2456939257d" => "2.2",
            "fc372a78d03ca5229f3c125a9dff91a779a66b7a" => "2.1.1",
            "c2e31867428e03ba2ef23911d605163a7008d6a5" => "2.1.0",
            
            _ => return None
        }.to_string())
    }
}