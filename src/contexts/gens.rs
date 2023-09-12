use camino::Utf8PathBuf;
use crate::{EmulatorContext, Error};
use crate::includes::copy_if_different;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum GensVersion {
    Ver11A,
    Ver11B,
    GitA2425B5,
}

#[derive(Debug, Clone, PartialEq)]
pub struct GensContext {
    pub version: GensVersion,
    pub start_paused: bool,
    pub rom: Option<Utf8PathBuf>,
    pub movie: Option<Utf8PathBuf>,
    pub lua: Option<Utf8PathBuf>,
    pub working_dir: Utf8PathBuf,
}
impl EmulatorContext for GensContext {
    fn cmd_name(&self) -> String {
        #[cfg(target_family = "unix")]
        { "wine".into() }
        
        #[cfg(target_family = "windows")]
        { "Gens.exe".into() }
    }
    
    fn args(&self) -> Vec<String> {
        let mut args = Vec::with_capacity(5);
        
        #[cfg(target_family = "unix")]
        {
            let mut executable = self.working_dir.clone();
            executable.push("Gens.exe");
            args.push(executable.to_string());
        }
        
        use GensVersion::*;
        match self.version {
            Ver11A | Ver11B | GitA2425B5 => { // TODO: verify for correctness
                if self.start_paused {
                    args.push("-pause".into());
                    args.push("0".into());
                }
                if let Some(rom) = self.rom.as_ref() {
                    args.push("-rom".into());
                    args.push(rom.to_string());
                }
                if let Some(movie) = self.movie.as_ref() {
                    args.push("-play".into());
                    args.push(movie.to_string());
                }
                if let Some(lua) = self.lua.as_ref() {
                    args.push("-lua".into());
                    args.push(lua.to_string());
                }
            },
        }
        
        args
    }
    
    fn env(&self) -> Vec<(String, String)> {
        let mut vars = vec![];

        #[cfg(target_family = "unix")]
        {
            let mut prefix = self.working_dir.clone();
            prefix.push(".wine/");
            
            vars.push(("WINEPREFIX".into(), prefix.to_string()));
        }
        
        vars
    }
    
    fn prepare(&mut self) -> Result<(), Error> {
        // Gens has inconsistent requirements for where files exist
        
        if let Some(rom) = self.rom.as_ref() {
            if !rom.is_file() {
                return Err(Error::MissingRom(rom.clone()));
            }
            let mut dest = self.working_dir.clone();
            dest.push(rom.file_name().unwrap());
            
            copy_if_different(&std::fs::read(rom)?, dest)?;
            
            self.rom = Some(rom.file_name().unwrap().into());
        }
        if let Some(movie) = self.movie.as_ref() {
            if !movie.is_file() {
                return Err(Error::MissingMovie(movie.clone()));
            }
            
            // movie path can be outside working dir, but must be absolute
            if !movie.is_absolute() {
                let mut dest = self.working_dir.clone();
                dest.push(movie.file_name().unwrap());
                
                copy_if_different(&std::fs::read(movie)?, dest)?;
                
                self.movie = Some(movie.file_name().unwrap().into());
            }
        }
        if let Some(lua) = self.lua.as_ref() {
            if !lua.is_file() {
                return Err(Error::MissingLua(lua.clone()));
            }
            
            // lua path can be outside working dir, but must be absolute
            if !lua.is_absolute() {
                let mut dest = self.working_dir.clone();
                dest.push(lua.file_name().unwrap());
                
                copy_if_different(&std::fs::read(lua)?, dest)?;
                
                self.lua = Some(lua.file_name().unwrap().into());
            }
        }
        
        Ok(())
    }
    
    fn working_dir(&self) -> Utf8PathBuf {
        self.working_dir.clone()
    }
}
impl GensContext {
    /// Creates a new Context with default options.
    /// 
    /// If the path does not point to a directory, or a file within a directory, which contains `Gens.exe`,
    /// an error message will be returned.
    pub fn new<P: Into<Utf8PathBuf>>(working_dir: P, version: GensVersion) -> Result<Self, Error> {
        let mut working_dir = working_dir.into();
        if working_dir.is_file() {
            working_dir.pop();
        }
        
        working_dir = working_dir.canonicalize_utf8().unwrap_or(working_dir);
        
        let mut detect_exe = working_dir.clone();
        detect_exe.push("Gens.exe");
        if working_dir.is_file() || !working_dir.exists() || !detect_exe.is_file() {
            return Err(Error::MissingExecutable(detect_exe));
        }
        
        Ok(Self {
            version,
            start_paused: false,
            rom: None,
            movie: None,
            lua: None,
            working_dir,
        })
    }
    
    pub fn with_pause(self, start_paused: bool) -> Self {
        Self {
            start_paused,
            ..self
        }
    }
    
    pub fn with_rom<P: Into<Utf8PathBuf>>(self, rom: P) -> Self {
        Self {
            rom: Some(rom.into()),
            ..self
        }
    }
    
    pub fn with_movie<P: Into<Utf8PathBuf>>(self, movie: P) -> Self {
        Self {
            movie: Some(movie.into()),
            ..self
        }
    }
    
    pub fn with_lua<P: Into<Utf8PathBuf>>(self, lua: P) -> Self {
        Self {
            lua: Some(lua.into()),
            ..self
        }
    }
}