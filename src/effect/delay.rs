use derive_builder::Builder;
use std::collections::VecDeque;

use crate::common::constants::SAMPLES_PER_MS;

pub(crate) const PREDELAY_BUFFER_SIZE: usize = 20;

static DEFAULT_DELAY_ID: usize = 0;
static DEFAULT_DELAY_MIX: f32 = 1.0;
static DEFAULT_DELAY_DECAY: f32 = 0.5;
static DEFAULT_INTERVAL_DURATION_MS: f32 = 100.0;
static DEFAULT_DELAY_DURATION_MS: f32 = 20.0;
static DEFAULT_NUM_REPEATS: usize = 4;
static MAX_NUM_ACTIVE_SAMPLE_MANAGERS: usize = 4;

// delay_buf: [************************************************************************* ...]
//             | duration_ms | interval_ms | duration_ms | interval_ms | duration_ms | ...
// there are num_repeats number of duration_ms sections
// duration_ms sections are width in samples of the delay window, i.e. length of each delay event
// interval_ms sections are width in samples of the silence between delay events
// as each sample comes in, insert_index updates the delay buffer rolling forward modulo
// as each sample comes in, the current delay_index is checked to see if it is in a delay window
// once the index gets to the end of the delay window, num_repeats increments. If the window
//  has repeated num_repeats times, it's put back in the pool. If it has not, a new window is
//  is pulled from the pool and it starts recording samples

#[allow(dead_code)]
#[derive(Clone, Debug)]
struct SampleManager {
    id: usize,
    sample_buffer_size: usize,
    sample_buffer: VecDeque<f32>,
    delay_windows: Vec<bool>,
    num_delay_windows: usize,
    num_predelay_samples: usize,
    sample_buffer_read_index: usize,
    sample_buffer_write_index: usize,
    init_buffer_index: usize,
    cur_delay_window: usize,
    delay_windows_index: usize,
    is_full: bool,
    is_active: bool,
    is_pre_delay: bool,
    is_in_delay_window: bool,
    is_in_interval: bool,
    has_spawned: bool,
}

impl PartialEq for SampleManager {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id &&
        self.sample_buffer_size == other.sample_buffer_size &&
        self.num_delay_windows == other.num_delay_windows &&
        self.num_predelay_samples == other.num_predelay_samples
    }
}

#[allow(dead_code)]
impl SampleManager {
    fn new(id: usize, sample_buffer_size: usize, delay_windows: Vec<bool>,
           num_delay_windows: usize, num_predelay_samples: usize) -> Self {
        SampleManager {
            id,
            sample_buffer_size,
            sample_buffer: VecDeque::with_capacity(sample_buffer_size),
            delay_windows,
            num_delay_windows,
            num_predelay_samples,
            sample_buffer_read_index: 0,
            sample_buffer_write_index: 0,
            init_buffer_index: 0,
            cur_delay_window: 0,
            delay_windows_index: 0,
            is_full: false,
            is_active: true,
            is_pre_delay: true,
            is_in_delay_window: true,
            is_in_interval: false,
            has_spawned: false,
        }
    }

    fn next_sample(&mut self, sample: f32) -> f32 {
        let mut delay_sample = 0.0f32;

        // if we are in the pre-delay buffer, increment the write index, add the sample to the
        // buffer and return 0
        if self.is_pre_delay {
            self.sample_buffer.push_back(sample);
            self.sample_buffer_write_index += 1;
            if self.sample_buffer_write_index == PREDELAY_BUFFER_SIZE {
                self.is_pre_delay = false;
            }
            return 0.0;
        }

        // if the buffer holding the samples being repeated in each delay window is not full,
        // add the sample to the buffer
        if !self.is_full {
            self.sample_buffer.push_back(sample);
            self.sample_buffer_write_index += 1;
            if self.sample_buffer_write_index == self.sample_buffer_size - self.num_predelay_samples {
                self.is_full = true;
            }
        }

        // check if we are in a delay window or an interval by checking current delay window value
        if self.delay_windows[self.delay_windows_index] {
            delay_sample = *self.sample_buffer
                .get(self.sample_buffer_read_index % self.sample_buffer_size)
                .unwrap_or(&0.0);
            // If this is the first sample in the delay window, increment the delay window index
            if self.sample_buffer_read_index == 0 {
                self.cur_delay_window += 1;
            }
            self.sample_buffer_read_index += 1;
        }

        // check for reaching the end of the delay windows
        if !self.is_pre_delay {
            self.delay_windows_index += 1;
        }
        if self.delay_windows_index == self.delay_windows.len() - self.num_predelay_samples {
            self.reset();
        }

        delay_sample
    }

    // DO NOT reset has_spawned, it is used to track if the sample manager has spawned a new
    // sample manager; this will happen once for each Manager when it gets full, up to the
    // global limit of MAX_NUM_ACTIVE_SAMPLE_MANAGERS
    fn reset(&mut self) {
        self.sample_buffer_read_index = 0;
        self.sample_buffer_write_index = 0;
        self.init_buffer_index = 0;
        self.cur_delay_window = 0;
        self.delay_windows_index = 0;
        self.is_full = false;
        self.is_active = true;
        self.is_pre_delay = true;
        self.is_in_delay_window = true;
        self.is_in_interval = false;
        self.sample_buffer.clear();
    }
}

#[allow(dead_code)]
#[derive(Builder, Clone, Debug, PartialEq)]
#[builder(build_fn(skip))]
pub(crate) struct Delay {

    id: usize,

    // master level at which sample events are mixed into final output
    pub(crate) mix: f32,

    // factor for how much each sample event decays in magnitude from the previous one
    pub(crate) decay: f32,

    // duration of the silence between sample events
    pub(crate) interval_ms: f32,

    // duration of each sample event
    pub(crate) duration_ms: f32,

    // the number of sample events
    pub(crate) num_repeats: usize,

    // number of samples in the pre-delay buffer
    pub(crate) num_predelay_samples: usize,

    // the number of concurrent sample managers allowed
    pub(crate) num_concurrent_sample_managers: usize,

    #[builder(field(private))]
    sample_manager_id_counter: usize,

    #[builder(field(private))]
    sample_manager_is_full_counter: usize,

    // complement of mix, private compute at build time because it's constant
    #[builder(field(private))]
    mix_complement: f32,

    // boundaries of sample indexes in delay windows or in intervals between delay windows
    #[builder(field(private))]
    delay_windows: Vec<bool>,

    #[builder(field(private))]
    duration_num_samples: usize,

    #[builder(field(private))]
    interval_num_samples: usize,

    // Per-instance sample managers (no global state)
    #[builder(field(private))]
    sample_managers: Vec<SampleManager>,

    // Precomputed decay factors per window
    #[builder(field(private))]
    decay_table: Vec<f32>,
}

// build the delay windows vectors, just the length of the sequence of indexes in each delay
// window and the interval between delay windows, set to true or false. To allow fast lookup
// of whether we add a delay sample as we iterate through samples
fn build_delay_windows(duration_num_samples: usize, interval_num_samples: usize,
                       num_repeats: usize) -> Vec<bool> {

    let mut delay_windows = Vec::new();
    let samples_total = (duration_num_samples * num_repeats) +
        (interval_num_samples * num_repeats - 1);

    let mut in_window = true;
    let mut in_window_index: usize = 0;
    for _ in 0..samples_total {
        delay_windows.push(in_window);

        in_window_index += 1;
        if in_window && in_window_index == duration_num_samples {
            in_window = false;
            in_window_index = 0;
        } else if !in_window && in_window_index == interval_num_samples {
            in_window = true;
            in_window_index = 0;
        }
    }

    delay_windows
}

#[allow(dead_code)]
impl DelayBuilder {

    pub(crate) fn build(&mut self) -> Result<Delay, String> {
        let id = self.id.unwrap_or(DEFAULT_DELAY_ID);
        let mix = self.mix.unwrap_or(DEFAULT_DELAY_MIX);
        let decay = self.decay.unwrap_or(DEFAULT_DELAY_DECAY);
        let interval_ms = self.interval_ms.unwrap_or(DEFAULT_INTERVAL_DURATION_MS);
        let duration_ms = self.duration_ms.unwrap_or(DEFAULT_DELAY_DURATION_MS);
        let num_repeats = self.num_repeats.unwrap_or(DEFAULT_NUM_REPEATS);
        let num_predelay_samples =
            self.num_predelay_samples.unwrap_or(PREDELAY_BUFFER_SIZE);
        let num_concurrent_sample_managers =
            self.num_concurrent_sample_managers.unwrap_or(MAX_NUM_ACTIVE_SAMPLE_MANAGERS);

        let duration_num_samples = duration_ms as usize * SAMPLES_PER_MS as usize;
        let interval_num_samples = interval_ms as usize * SAMPLES_PER_MS as usize;
        let delay_windows = build_delay_windows(duration_num_samples, interval_num_samples, num_repeats);
        let mix_complement = 1.0 - mix;

        // Precompute decay table
        let decay_table: Vec<f32> = (0..=num_repeats as i32 + 1)
            .map(|i| decay.powi(i))
            .collect();

        // Initialize with one sample manager
        let initial_manager = SampleManager::new(
            0, duration_num_samples, delay_windows.clone(), num_repeats, num_predelay_samples,
        );

        Ok(
            Delay {
                id,
                mix,
                decay,
                interval_ms,
                duration_ms,
                num_repeats,
                num_predelay_samples,
                num_concurrent_sample_managers,
                sample_manager_id_counter: 1,
                sample_manager_is_full_counter: 0,
                mix_complement,
                delay_windows,
                duration_num_samples,
                interval_num_samples,
                sample_managers: vec![initial_manager],
                decay_table,
            }
        )
    }
}

#[allow(dead_code)]
impl Delay {

    pub(crate) fn apply_effect(&mut self, sample: f32, _sample_clock: f32) -> f32 {
        let mut delay_sample_sum = 0.0f32;
        let mut num_delay_samples = 0usize;
        let mut should_spawn = false;

        for sample_manager in self.sample_managers.iter_mut() {
            let decay_factor = self.decay_table
                .get(sample_manager.cur_delay_window)
                .copied()
                .unwrap_or_else(|| self.decay.powi(sample_manager.cur_delay_window as i32));
            let next_sample = sample_manager.next_sample(sample) * decay_factor;

            delay_sample_sum += next_sample;
            num_delay_samples += 1;

            if !sample_manager.has_spawned && sample_manager.is_full {
                sample_manager.has_spawned = true;
                should_spawn = true;
            }
        }

        if num_delay_samples > 0 {
            delay_sample_sum /= num_delay_samples as f32;
        }

        // Add new manager if needed, enforcing per-instance limit
        if should_spawn && self.sample_managers.len() < self.num_concurrent_sample_managers {
            let new_id = self.sample_manager_id_counter;
            self.sample_manager_id_counter += 1;
            self.sample_managers.push(SampleManager::new(
                new_id, self.duration_num_samples, self.delay_windows.clone(),
                self.num_repeats, self.num_predelay_samples,
            ));
        }

        self.mix_complement * sample + (self.mix * delay_sample_sum)
    }
}

#[allow(dead_code)]
pub(crate) fn default_delay() -> Delay {
    DelayBuilder::default()
        .build().unwrap()
}

#[allow(dead_code)]
pub(crate) fn no_op_delay() -> Delay {
    DelayBuilder::default()
        .num_repeats(0)
        .build().unwrap()
}
