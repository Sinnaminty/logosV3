#![allow(non_camel_case_types, non_snake_case, non_upper_case_globals)]
use crate::types::Result;
use std::{ffi::CString, path::Path, ptr};
// Generated in build.rs as OUT_DIR/dectalk_bindings.rs
include!(concat!(env!("OUT_DIR"), "/dectalk_bindings.rs"));

#[derive(Debug)]
enum DectalkError {
    MmError {
        error_code: MMRESULT,
        loc: std::panic::Location<'static>,
    },
    NullHandleError,
}

impl std::fmt::Display for DectalkError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DectalkError::MmError { error_code, loc } => {
                write!(f, "DectalkMmError {} at {}", error_code, loc)
            }
            DectalkError::NullHandleError => write!(f, "DectalkNullHandleError"),
        }
    }
}

impl std::error::Error for DectalkError {}

#[track_caller]
fn check_mm(code: MMRESULT) -> Result<()> {
    if code == 0 {
        Ok(())
    } else {
        let loc = std::panic::Location::caller();

        let e = DectalkError::MmError {
            error_code: (code),
            loc: (*loc),
        };
        Err(Box::new(e))
    }
}

#[derive(Debug)]
pub struct Dectalk {
    handle: LPTTS_HANDLE_T,
}

pub enum WaveFormat {
    //  Mono 8-bit, 11.025 kHz sample rate
    //    DT_1M08 = 1,
    ///  Mono 16-bit, 11.025 kHz sample rate
    DT_1M16 = 4,
    // Mono 8-bit, m-law 8 kHz sample rate
    //   DT_08M08 = 0x1000,
}

impl Dectalk {
    /// Start DECtalk. Uses no window/callback and default device options.
    /// For headless use (WAV/memory output), we keep device options at 0 and
    /// direct output to a wave file using `TextToSpeechOpenWaveOutFile`.
    pub fn new() -> Result<Self> {
        log::debug!("Dectalk::new()");
        let mut h: LPTTS_HANDLE_T = ptr::null_mut();
        let rc = unsafe { TextToSpeechStartup(&mut h as *mut LPTTS_HANDLE_T, 0, 0, None, 0) };
        check_mm(rc)?;
        if h.is_null() {
            return Err(Box::new(DectalkError::NullHandleError));
        }
        Ok(Self { handle: h })
    }

    pub fn set_rate(&self, wpm: u32) -> Result<()> {
        check_mm(unsafe { TextToSpeechSetRate(self.handle, wpm as DWORD) })
    }

    /// DECtalk speakers are numeric IDs; consult caps/docs for mapping.
    pub fn set_speaker(&self, speaker_id: u32) -> Result<()> {
        check_mm(unsafe { TextToSpeechSetSpeaker(self.handle, speaker_id as SPEAKER_T) })
    }

    /// Set language by numeric code (e.g., TTS_AMERICAN_ENGLISH = 1)
    pub fn set_language(&self, lang_code: u32) -> Result<()> {
        check_mm(unsafe { TextToSpeechSetLanguage(self.handle, lang_code as LANGUAGE_T) })
    }

    /// Write synthesized audio directly to a WAV file on disk.
    /// `format` is a DECtalk wave format code; 0 typically selects a default.
    pub fn speak_to_wav(
        &self,
        text: &str,
        out_path: impl AsRef<Path>,
        format: WaveFormat,
    ) -> Result<()> {
        // Open the wave file output
        let cpath = CString::new(out_path.as_ref().to_string_lossy().as_bytes())?;
        check_mm(unsafe {
            TextToSpeechOpenWaveOutFile(self.handle, cpath.as_ptr() as *mut i8, format as DWORD)
        })?;

        // Speak the text in normal mode (ASCII expected by this entry point)
        let mut bytes = text.as_bytes().to_vec();
        bytes.retain(|&b| b != 0); // strip interior NULs
        let ctext = CString::new(bytes)?;
        check_mm(unsafe { TextToSpeechSpeak(self.handle, ctext.as_ptr() as *mut i8, TTS_NORMAL) })?;

        // Wait until all queued speech is done
        check_mm(unsafe { TextToSpeechSync(self.handle) })?;

        // Close the wave file
        check_mm(unsafe { TextToSpeechCloseWaveOutFile(self.handle) })
    }

    /// Pause/resume helpers
    pub fn pause(&self) -> Result<()> {
        check_mm(unsafe { TextToSpeechPause(self.handle) })
    }
    pub fn resume(&self) -> Result<()> {
        check_mm(unsafe { TextToSpeechResume(self.handle) })
    }
}

impl Drop for Dectalk {
    fn drop(&mut self) {
        if !self.handle.is_null() {
            unsafe {
                let _ = TextToSpeechShutdown(self.handle);
            }
        }
    }
}
