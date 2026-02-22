use derive_builder::Builder;
use crate::common::constants::SAMPLE_RATE;

use crate::note::constants::{DEFAULT_VOLUME, INIT_START_TIME};
use crate::note::note_trait::BuilderWrapper;

pub(crate) const BUF_STORAGE_SIZE: usize = (SAMPLE_RATE as usize * 2) as usize;

#[allow(dead_code)]
#[derive(Builder, Clone, Debug, PartialEq)]
#[builder(build_fn(skip))] // needed for custom build()
pub(crate) struct SampledNote {
    #[builder(default = "String::new()")]
    pub(crate) file_path: String,
    
    #[builder(default = "0", setter(skip))]
    pub(crate) buf_size: usize,
    
    #[builder(default = "0", setter(skip))]
    pub(crate) sample_index: usize,

    #[builder(default = "DEFAULT_VOLUME")]
    pub(crate) volume: f32,

    #[builder(default = "INIT_START_TIME")]
    pub(crate) start_time_ms: f32,

    #[builder(default = "INIT_START_TIME")]
    pub(crate) end_time_ms: f32,

    #[builder(default = "Vec::with_capacity(BUF_STORAGE_SIZE)", setter(skip))]
    sample_buf: Vec<f32>,
}

#[allow(dead_code)]
impl SampledNote {
    pub(crate) fn duration_ms(&self) -> f32 {
        self.end_time_ms - self.start_time_ms
    }

    pub(crate) fn next_sample(&mut self) -> f32 {
        if self.sample_index < self.buf_size {
            let sample = self.sample_buf[self.sample_index];
            self.sample_index += 1;
            sample
        } else {
            0.0
        }
    }
    
    // TODO Can now add range and "scrtach" kinds of access to the buffer
    
    pub(crate) fn get_sample_at(&self, index: usize) -> f32 {
        self.sample_buf[index]
    }

    // TODO remove unused arg buf_size
    pub(crate) fn set_sample_buf(&mut self, samples: &[f32]) {
        self.sample_buf = samples.try_into().unwrap();
        self.buf_size = samples.len();
        self.sample_index = 0;
    }

    pub(crate) fn append_sample(&mut self, sample: f32) {
        self.sample_buf.push(sample);
        self.buf_size += 1;
    }

    pub(crate) fn reverse(&mut self) {
        self.sample_buf.reverse();
    }

    pub(crate) fn chopped(&self, num_segments: usize) -> Vec<SampledNote> {
        let mut chopped_notes = Vec::with_capacity(num_segments);
        let segment_size = self.buf_size / num_segments;
        for i in 0..num_segments {
            let start = i * segment_size;
            let end = (i + 1) * segment_size;
            let mut chopped_note = self.clone();
            chopped_note.sample_buf = self.sample_buf[start..end].to_vec();
            chopped_note.buf_size = segment_size;
            chopped_notes.push(chopped_note);
        }
        chopped_notes
    }

    // TODO Support other algorithms besides linear interpolation, which is implemented here
    pub(crate) fn stretched(&self, stretch_factor: u8) -> SampledNote {
        let mut stretched_note: SampledNote = self.clone();
        let stretched_buf_size = self.buf_size * stretch_factor as usize;
        stretched_note.sample_buf = Vec::with_capacity(stretched_buf_size);
        stretched_note.buf_size = stretched_buf_size;
        for i in 0..self.buf_size - 1 {
            let start = self.sample_buf[i];
            let end = self.sample_buf[i + 1];
            let step = (end - start) / stretch_factor as f32;
            for j in 0..stretch_factor {
                stretched_note.sample_buf.push(start + j as f32 * step);
            }
        }

        stretched_note
    }
}

impl BuilderWrapper<SampledNote> for SampledNoteBuilder {
    fn new() -> SampledNote {
        SampledNoteBuilder::default().build().unwrap()
    }
}

impl SampledNoteBuilder {

    pub(crate) fn build(&mut self) -> Result<SampledNote, String> {
        let sample_index = 0;
        let volume = self.volume.unwrap_or(DEFAULT_VOLUME);
        let start_time_ms = self.start_time_ms.unwrap_or(INIT_START_TIME);
        let end_time_ms = self.end_time_ms.unwrap_or(INIT_START_TIME);

        let mut sample_buf: Vec<f32> = Vec::with_capacity(crate::note::sampled_note::BUF_STORAGE_SIZE);
        
        // Only try to read audio file if file_path is provided and not empty
        if let Some(file_path) = &self.file_path {
            if !file_path.is_empty() {
                let sample_data =
                    crate::audio_gen::audio_gen::read_audio_file(file_path).into_boxed_slice();
                for sample in sample_data.iter() {
                    sample_buf.push(*sample as f32);
                }
            }
        }
        let buf_size = sample_buf.len();
        
        Ok(
            SampledNote {
                file_path: self.file_path.take().unwrap_or_default(),
                buf_size,
                sample_index,
                volume,
                start_time_ms,
                end_time_ms,
                sample_buf,
            }
        )
    }
}

#[allow(dead_code)]
pub(crate) fn default_sample_note() -> SampledNote {
    SampledNoteBuilder::default().build().unwrap()
}