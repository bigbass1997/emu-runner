use camino::Utf8PathBuf;
use crate::{EmulatorContext, Error};
use crate::includes::copy_if_different;

#[derive(Debug, Clone, PartialEq)]
pub struct FceuxContext {
    pub config: Option<Utf8PathBuf>,
    pub movie: Option<Utf8PathBuf>,
    pub lua: Option<Utf8PathBuf>,
    pub rom: Option<Utf8PathBuf>,
    pub working_dir: Utf8PathBuf,
}
impl EmulatorContext for FceuxContext {
    fn cmd_name(&self) -> String {
        #[cfg(target_family = "unix")]
        {
            match self.determine_executable() {
                Some(exe) if exe == "fceux" => "./fceux".into(),
                Some(_) => "wine".into(),
                None => "./fceux".into(),
            }
        }
        
        #[cfg(target_family = "windows")]
        {
            match self.determine_executable() {
                Some(exe) => exe,
                None => "fceux.exe".into()
            }
        }
    }
    
    fn args(&self) -> Vec<String> {
        let mut args = Vec::with_capacity(5);
        
        #[cfg(target_family = "unix")]
        {
            if self.cmd_name() == "wine" {
                args.push(self.determine_executable().unwrap());
            }
        }
        
        match self.determine_executable() {
            Some(exe) => match exe.as_str() {
                "fceux.exe" | "fceux64.exe" => {
                    if let Some(config) = self.config.as_ref() {
                        args.push("-cfg".into());
                        args.push(config.to_string());
                    }
                    if let Some(movie) = self.movie.as_ref() {
                        args.push("-playmovie".into());
                        args.push(movie.to_string());
                    }
                    if let Some(lua) = self.lua.as_ref() {
                        args.push("-lua".into());
                        args.push(lua.to_string());
                    }
                },
                "fceux" | "qfceux.exe" => {
                    if let Some(movie) = self.movie.as_ref() {
                        args.push("--playmov".into());
                        args.push(movie.to_string());
                    }
                    if let Some(lua) = self.lua.as_ref() {
                        args.push("--loadlua".into());
                        args.push(lua.to_string());
                    }
                },
                _ => ()
            },
            None => ()
        }
        
        if let Some(rom) = self.rom.as_ref() {
            args.push(rom.to_string());
        }
        
        args
    }
    
    fn env(&self) -> Vec<(String, String)> {
        let mut vars = vec![];

        #[cfg(target_family = "unix")]
        {
            if self.cmd_name() == "wine" {
                let mut prefix = self.working_dir();
                prefix.push(".wine/");
                
                vars.push(("WINEPREFIX".into(), prefix.to_string()));
            }
        }
        
        let mut home = self.working_dir();
        home.push(".fceux/");
        vars.push(("HOME".into(), home.to_string()));
        
        vars
    }
    
    fn prepare(&mut self) -> Result<(), Error> {
        // FCEUX accepts configs/movies/scripts/roms from anywhere,
        // so we only need to verify they exist.
        // However, since we change the working directory, and there's no
        // easy way to test if file exists relative to a different dir,
        // the paths _should_ be absolute, either originally or via the with_* functions.
        
        #[cfg(target_family = "windows")]
        {
            if cmd_name == "./fceux" {
                return Err(Error::IncompatibleOSVersion);
            }
        }
        
        if let Some(config) = self.config.as_ref() {
            // Preparing the config file is extremely messy.
            // - win32/win64 provides a CLI argument that is used.
            // - win64-QtSLD uses the fceux.cfg located beside the executable.
            // - compiled linux builds use $HOME/.fceux/fceux.cfg.
            //     (if $HOME isn't set, it's unclear what FCEUX does)
            
            if !config.is_file() {
                return Err(Error::MissingConfig(config.clone()));
            }
            
            if let Some(config) = self.config.as_ref() {
                if !config.is_file() {
                    return Err(Error::MissingConfig(config.clone()));
                }
                
                if let Some(exe) = self.determine_executable() {
                    let mut dest = self.working_dir();
                    if exe == "fceux" {
                        dest.push(".fceux/");
                        if !dest.is_dir() {
                            std::fs::create_dir_all(&dest)?;
                        }
                        dest.push("fceux.cfg");
                        
                        copy_if_different(&std::fs::read(config)?, dest)?;
                    } else if exe == "qfceux.exe" {
                        dest.push("fceux.cfg");
                        
                        copy_if_different(&std::fs::read(config)?, dest)?;
                    } else if !config.is_absolute() {
                        return Err(Error::AbsolutePathFailed);
                    }
                }
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
        
        Ok(())
    }
    
    fn working_dir(&self) -> Utf8PathBuf {
        self.working_dir.clone()
    }
}
impl FceuxContext {
    /// Creates a new Context with default options.
    /// 
    /// If the path does not point to a directory, or a file within a directory, which contains a valid FCEUX executable,
    /// an error will be returned.
    pub fn new<P: Into<Utf8PathBuf>>(working_dir: P) -> Result<Self, Error> {
        let mut working_dir = working_dir.into();
        if working_dir.is_file() {
            working_dir.pop();
        }
        
        working_dir = working_dir.canonicalize_utf8().unwrap_or(working_dir);
        
        if working_dir.is_file() || !working_dir.exists() {
            return Err(Error::MissingExecutable(working_dir));
        }
        
        let mut found = false;
        for exe in ["fceux.exe", "fceux64.exe", "qfceux.exe", "fceux"] {
            let mut path = working_dir.clone();
            path.push(exe);
            
            if path.is_file() {
                found = true;
                break;
            }
        }
        if !found {
            let mut path = working_dir.clone();
            path.push("fceux");
            return Err(Error::MissingExecutable(path));
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
    
    pub fn determine_executable(&self) -> Option<String> {
        let mut path = self.working_dir();
        path.push("fceux");
        if path.is_file() {
            return Some("fceux".into())
        }
        
        path.pop();
        path.push("fceux.exe");
        if path.is_file() {
            return Some("fceux.exe".into())
        }
        
        path.pop();
        path.push("fceux64.exe");
        if path.is_file() {
            return Some("fceux64.exe".into())
        }
        
        path.pop();
        path.push("qfceux.exe");
        if path.is_file() {
            return Some("qfceux.exe".into())
        }
        
        None
    }
}