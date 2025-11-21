#![allow(non_camel_case_types, non_snake_case, non_upper_case_globals)]
use crate::pawthos::types::Result;
use std::{
    ffi::{CString, NulError, c_void},
    path::Path,
    ptr::{self, NonNull},
};
// Generated in build.rs as OUT_DIR/dectalk_bindings.rs
include!(concat!(env!("OUT_DIR"), "/dectalk_bindings.rs"));

#[derive(Debug)]
pub enum DectalkError {
    Mm {
        error_code: MMRESULT,
        loc: std::panic::Location<'static>,
    },
    NullHandle,
    FfiNul(std::ffi::NulError),
}

impl std::fmt::Display for DectalkError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DectalkError::Mm { error_code, loc } => {
                write!(f, "DectalkMmError {} at {}", error_code, loc)
            }
            DectalkError::NullHandle => write!(f, "DectalkNullHandleError"),
            DectalkError::FfiNul(e) => write!(f, "Nul byte found in string: {}", e),
        }
    }
}

impl From<NulError> for DectalkError {
    fn from(e: NulError) -> Self {
        DectalkError::FfiNul(e)
    }
}

impl std::error::Error for DectalkError {}

#[track_caller]
fn check_mm(code: MMRESULT) -> Result<(), DectalkError> {
    if code == 0 {
        Ok(())
    } else {
        let loc = std::panic::Location::caller();

        let e = DectalkError::Mm {
            error_code: (code),
            loc: (*loc),
        };
        Err(e)
    }
}

#[derive(Debug)]
pub struct Dectalk {
    handle: NonNull<c_void>,
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
    pub fn new() -> Result<Self, DectalkError> {
        log::debug!("Dectalk::new()");
        let mut raw: LPTTS_HANDLE_T = ptr::null_mut();
        // SAFETY: TextToSpeechStartup is safe because we know that LPTTS_HANDLE_T is a proper null_mut pointer
        // before the function call, and we check if the handle is null afterwards.
        // for more information on the TextToSpeechStarup, visit https://dectalk.github.io/dectalk/dectalk.htm
        let rc = unsafe { TextToSpeechStartup(&mut raw, 0, 0, None, 0) };
        check_mm(rc)?;

        let handle = NonNull::new(raw).ok_or(DectalkError::NullHandle)?;
        Ok(Self { handle })
    }

    pub fn set_rate(&self, wpm: u32) -> Result<(), DectalkError> {
        check_mm(unsafe { TextToSpeechSetRate(self.handle.as_ptr(), wpm as DWORD) })?;
        Ok(())
    }
    pub fn speak_to_wav(
        &self,
        text: &str,
        out_path: impl AsRef<Path>,
        format: WaveFormat,
    ) -> Result<(), DectalkError> {
        // Open the wave file output
        let cpath = CString::new(out_path.as_ref().to_string_lossy().as_bytes())?;
        check_mm(unsafe {
            TextToSpeechOpenWaveOutFile(
                self.handle.as_ptr(),
                cpath.as_ptr() as *mut i8,
                format as DWORD,
            )
        })?;

        // Speak the text in normal mode (ASCII expected by this entry point)
        let mut bytes = text.as_bytes().to_vec();
        bytes.retain(|&b| b != 0); // strip interior NULs
        let ctext = CString::new(bytes)?;
        check_mm(unsafe {
            TextToSpeechSpeak(self.handle.as_ptr(), ctext.as_ptr() as *mut i8, TTS_NORMAL)
        })?;

        // Wait until all queued speech is done
        check_mm(unsafe { TextToSpeechSync(self.handle.as_ptr()) })?;

        // Close the wave file
        check_mm(unsafe { TextToSpeechCloseWaveOutFile(self.handle.as_ptr()) })?;
        Ok(())
    }
}

impl Drop for Dectalk {
    fn drop(&mut self) {
        unsafe {
            let _ = TextToSpeechShutdown(self.handle.as_ptr());
        }
    }
}
