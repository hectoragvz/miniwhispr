//! The Groq Transcriber: sends a Dictation's audio to Groq Whisper large-v3
//! and returns the Transcript (ADR 0002).

use std::time::{Duration, Instant};

use reqwest::blocking::Client;
use reqwest::blocking::multipart::{Form, Part};
use serde::Deserialize;

use crate::dictation::{Audio, SAMPLE_RATE, Transcriber};
use crate::key_store;

const URL: &str = "https://api.groq.com/openai/v1/audio/transcriptions";
const MODEL: &str = "whisper-large-v3";
/// Whisper reads the prompt as text said just before the audio. Saying it in
/// both languages steers detection toward English or Spanish without fixing
/// either, so short Spanish Dictations are not taken for Portuguese.
const PROMPT: &str =
    "The speaker talks in English or Spanish. El hablante habla en inglés o en español.";
const CONNECT_TIMEOUT: Duration = Duration::from_secs(10);
/// Whisper's own rule for silence, from its reference `transcribe()`: a
/// segment is left out when Whisper thinks it is probably not speech and is
/// not confident in the words it heard. On silence Whisper makes up words
/// such as "END" or "Thank you.", and this is how they are caught.
const NO_SPEECH_THRESHOLD: f64 = 0.6;
const LOG_PROB_THRESHOLD: f64 = -1.0;

pub struct GroqTranscriber {
    client: Client,
}

impl GroqTranscriber {
    pub fn new() -> Self {
        let client = Client::builder()
            .connect_timeout(CONNECT_TIMEOUT)
            .build()
            .expect("could not set up the Groq HTTP client");
        Self { client }
    }
}

/// Groq's `verbose_json` answer, which splits the Transcript into segments.
#[derive(Deserialize)]
struct Transcription {
    segments: Vec<Segment>,
}

#[derive(Deserialize)]
struct Segment {
    text: String,
    no_speech_prob: f64,
    avg_logprob: f64,
}

impl Transcriber for GroqTranscriber {
    fn transcribe(&mut self, audio: &Audio) -> Result<String, String> {
        let key = key_store::require()?;
        let file = Part::bytes(wav(audio))
            .file_name("dictation.wav")
            .mime_str("audio/wav")
            .map_err(|e| e.to_string())?;
        let form = Form::new()
            .text("model", MODEL)
            .text("prompt", PROMPT)
            .text("response_format", "verbose_json")
            .part("file", file);
        let started = Instant::now();
        let response = self
            .client
            .post(URL)
            .timeout(timeout(audio))
            .bearer_auth(key)
            .multipart(form)
            .send()
            .map_err(|e| format!("Could not reach Groq: {e}"))?;
        let status = response.status();
        if !status.is_success() {
            let body = response.text().unwrap_or_default();
            return Err(format!("Groq transcription failed with {status}: {body}"));
        }
        let transcription: Transcription = response
            .json()
            .map_err(|e| format!("Unexpected answer from Groq: {e}"))?;
        // Only lengths: the Transcript itself never goes to the log, which is
        // written to disk.
        log::info!(
            "Transcribed {:.1} s of audio in {:.1} s",
            audio.samples.len() as f64 / f64::from(SAMPLE_RATE),
            started.elapsed().as_secs_f64(),
        );
        Ok(speech(transcription))
    }
}

/// The Transcript without the segments Whisper takes for silence. Empty when
/// there was no speech at all.
fn speech(transcription: Transcription) -> String {
    let text: String = transcription
        .segments
        .into_iter()
        .filter(|s| {
            !(s.no_speech_prob > NO_SPEECH_THRESHOLD && s.avg_logprob <= LOG_PROB_THRESHOLD)
        })
        .map(|s| s.text)
        .collect();
    // Whisper starts each segment with a space.
    text.trim().to_string()
}

/// How long a transcription may take: 30 s, plus 15 s per minute of audio
/// for the upload, so a 10-minute Dictation gets 3 minutes.
fn timeout(audio: &Audio) -> Duration {
    let seconds = audio.samples.len() as f64 / f64::from(SAMPLE_RATE);
    Duration::from_secs(30) + Duration::from_secs_f64(seconds / 4.0)
}

/// Encodes `audio` as a WAV file: 16 kHz mono 16-bit PCM. Ten minutes is
/// 19.2 MB, within Groq's 25 MB upload limit, and WAV needs no decoding on
/// Groq's side, which keeps latency low.
fn wav(audio: &Audio) -> Vec<u8> {
    const BYTES_PER_SAMPLE: u32 = 2;
    let data_size = audio.samples.len() as u32 * BYTES_PER_SAMPLE;
    let mut wav = Vec::with_capacity(44 + data_size as usize);
    wav.extend_from_slice(b"RIFF");
    wav.extend_from_slice(&(36 + data_size).to_le_bytes());
    wav.extend_from_slice(b"WAVE");

    wav.extend_from_slice(b"fmt ");
    wav.extend_from_slice(&16u32.to_le_bytes());
    wav.extend_from_slice(&1u16.to_le_bytes()); // PCM
    wav.extend_from_slice(&1u16.to_le_bytes()); // mono
    wav.extend_from_slice(&SAMPLE_RATE.to_le_bytes());
    wav.extend_from_slice(&(SAMPLE_RATE * BYTES_PER_SAMPLE).to_le_bytes());
    wav.extend_from_slice(&(BYTES_PER_SAMPLE as u16).to_le_bytes());
    wav.extend_from_slice(&16u16.to_le_bytes()); // bits per sample

    wav.extend_from_slice(b"data");
    wav.extend_from_slice(&data_size.to_le_bytes());
    for sample in &audio.samples {
        wav.extend_from_slice(&sample.to_le_bytes());
    }
    wav
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dictation::MAX_DICTATION;

    /// Groq's upload limit on the free tier.
    const UPLOAD_LIMIT: usize = 25 * 1024 * 1024;

    #[test]
    fn a_ten_minute_dictation_fits_in_one_upload() {
        let samples = SAMPLE_RATE as usize * MAX_DICTATION.as_secs() as usize;
        let audio = Audio {
            samples: vec![0; samples],
        };

        assert!(wav(&audio).len() < UPLOAD_LIMIT);
    }

    #[test]
    fn the_wav_is_16_khz_mono_16_bit() {
        let audio = Audio {
            samples: vec![1, -2],
        };

        let expected: &[u8] = &[
            b'R', b'I', b'F', b'F', 40, 0, 0, 0, // file size minus 8
            b'W', b'A', b'V', b'E', //
            b'f', b'm', b't', b' ', 16, 0, 0, 0, // format chunk size
            1, 0, // PCM
            1, 0, // mono
            0x80, 0x3e, 0, 0, // 16 000 samples a second
            0x00, 0x7d, 0, 0, // 32 000 bytes a second
            2, 0, // 2 bytes per sample
            16, 0, // 16 bits per sample
            b'd', b'a', b't', b'a', 4, 0, 0, 0, // data size
            1, 0, 0xfe, 0xff, // the samples, little-endian
        ];
        assert_eq!(wav(&audio), expected);
    }

    #[test]
    fn segments_whisper_takes_for_silence_are_left_out() {
        let transcription: Transcription = serde_json::from_str(
            r#"{
                "text": " Let's meet tomorrow. END Thank you.",
                "segments": [
                    { "text": " Let's meet tomorrow.", "no_speech_prob": 0.01, "avg_logprob": -0.2 },
                    { "text": " END", "no_speech_prob": 0.9, "avg_logprob": -1.4 },
                    { "text": " Thank you.", "no_speech_prob": 0.7, "avg_logprob": -0.3 }
                ]
            }"#,
        )
        .unwrap();

        assert_eq!(speech(transcription), "Let's meet tomorrow. Thank you.");
    }

    #[test]
    fn a_dictation_of_only_silence_has_no_speech() {
        let transcription: Transcription = serde_json::from_str(
            r#"{
                "text": " END",
                "segments": [{ "text": " END", "no_speech_prob": 0.95, "avg_logprob": -1.1 }]
            }"#,
        )
        .unwrap();

        assert_eq!(speech(transcription), "");
    }
}
