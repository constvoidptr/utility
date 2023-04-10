//! Small wrapper around the Windows SDK for TTS

use std::fs::File;
use std::io::{BufWriter, Write};
use std::path::Path;
use windows::core::Error as WindowsError;
use windows::Media::SpeechSynthesis::{SpeechSynthesizer, VoiceInformation};
use windows::Storage::Streams::DataReader;

/// Spoken text in .wav format
pub struct Spoken {
    buf: Vec<u8>,
}

impl Spoken {
    pub fn into_inner(self) -> Vec<u8> {
        self.buf
    }

    pub fn save(&self, path: impl AsRef<Path>) -> std::io::Result<()> {
        let file = File::create(path)?;
        let mut wtr = BufWriter::with_capacity(self.buf.len(), file);
        wtr.write_all(&self.buf)?;
        Ok(())
    }
}

pub struct Synthesizer {
    inner: SpeechSynthesizer,
}

impl Synthesizer {
    pub fn new() -> Result<Self, WindowsError> {
        Ok(Self {
            inner: SpeechSynthesizer::new()?,
        })
    }

    pub fn with_voice(voice: &VoiceInformation) -> Result<Self, WindowsError> {
        let synthesizer = SpeechSynthesizer::new()?;
        synthesizer.SetVoice(voice)?;
        Ok(Self { inner: synthesizer })
    }

    pub fn voices() -> Result<Vec<VoiceInformation>, WindowsError> {
        Ok(Vec::from_iter(SpeechSynthesizer::AllVoices()?))
    }

    pub async fn say(&self, text: &str) -> Result<Spoken, WindowsError> {
        // Convert the text to the windows HSTRING type
        let text = windows::core::HSTRING::from(text);

        // Create stream
        let stream = self.inner.SynthesizeTextToStreamAsync(&text)?.await?;
        let stream_size = stream.Size()?;

        // Create data reader
        let data_reader = DataReader::CreateDataReader(&stream.GetInputStreamAt(0)?)?;
        data_reader.LoadAsync(stream_size as u32)?.await?;

        // Allocate buffer and write into it
        let mut buf = vec![0u8; stream_size as usize];
        data_reader.ReadBytes(&mut buf)?;

        // Release the system resources
        data_reader.Close()?;
        stream.Close()?;

        Ok(Spoken { buf })
    }

    pub fn close(self) -> Result<(), (Self, WindowsError)> {
        match self.inner.Close() {
            Ok(_) => Ok(()),
            Err(err) => Err((self, err)),
        }
    }
}

impl Drop for Synthesizer {
    fn drop(&mut self) {
        let _ = self.inner.Close();
    }
}
