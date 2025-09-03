use derive_builder::Builder;
use std::collections::VecDeque;
use std::sync::{Arc, RwLock};
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::Mutex;
use std::collections::HashMap;
use std::sync::LazyLock;

use crate::common::constants::SAMPLES_PER_MS;

pub(crate) const PREDELAY_BUFFER_SIZE: usize = 20;

static DEFAULT_DELAY_ID: usize = 0;
static DEFAULT_DELAY_MIX: f32 = 1.0;
static DEFAULT_DELAY_DECAY: f32 = 0.5;
static DEFAULT_INTERVAL_DURATION_MS: f32 = 100.0;
static DEFAULT_DELAY_DURATION_MS: f32 = 20.0;
static DEFAULT_NUM_REPEATS: usize = 4;
static ACTIVE_SAMPLE_MANAGERS: LazyLock<Mutex<HashMap<usize, Vec<SampleManager>>>> = 
    LazyLock::new(|| Mutex::new(HashMap::new()));
static SAMPLE_MANAGER_ID_COUNTER: LazyLock<Mutex<usize>> = LazyLock::new(|| Mutex::new(0));
static MAX_NUM_ACTIVE_SAMPLE_MANAGERS: usize = 4;

struct SampleManagerConfig {
    id: usize,
    sm_id: usize,
    sample_buffer_size: usize,
    delay_windows: Vec<bool>,
    num_delay_windows: usize,
    num_predelay_samples: usize,
    sample_buffer_read_index: usize,
    sample_buffer_write_index: usize,
    init_buffer_index: usize,
    cur_delay_window: usize,
    delay_windows_index: usize,
}

fn add_sample_manager(config: SampleManagerConfig) {
    
    let mut map = ACTIVE_SAMPLE_MANAGERS.lock().unwrap();
    let sample_managers = map.entry(config.id).or_default();
    sample_managers.push(
        SampleManager {
            id: config.sm_id,
            sample_buffer_size: config.sample_buffer_size,
            sample_buffer: Arc::new(RwLock::new(VecDeque::with_capacity(config.sample_buffer_size))),
            delay_windows: config.delay_windows.clone(),
            num_delay_windows: config.num_delay_windows,
            num_predelay_samples: config.num_predelay_samples,
            sample_buffer_read_index: AtomicUsize::new(config.sample_buffer_read_index),
            sample_buffer_write_index: AtomicUsize::new(config.sample_buffer_write_index),
            init_buffer_index: AtomicUsize::new(config.init_buffer_index),
            cur_delay_window: AtomicUsize::new(config.cur_delay_window),
            delay_windows_index: AtomicUsize::new(config.delay_windows_index),
            is_full: AtomicBool::new(false),
            is_active: AtomicBool::new(true),
            is_pre_delay: AtomicBool::new(true),
            is_in_delay_window: AtomicBool::new(true),
            is_in_interval: AtomicBool::new(false),
            has_spawned: AtomicBool::new(false),
        }
    );
}

fn next_sample_manager_id() -> usize {
    let mut counter = SAMPLE_MANAGER_ID_COUNTER.lock().unwrap();
    let id = *counter;
    *counter += 1;
    id
}

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
#[derive(Debug)]
pub(crate) struct SampleManager {

    // the id of the sample manager
    id: usize,
    // the size of the delay sample buffer
    sample_buffer_size: usize,
    sample_buffer: Arc<RwLock<VecDeque<f32>>>,
    // boundaries of sample indexes in delay windows or in intervals between delay windows
    // true if in delay window, false if in interval
    delay_windows: Vec<bool>,
    num_delay_windows: usize,
    num_predelay_samples: usize,
    // the current index for reading the next delay sample from the buffer
    sample_buffer_read_index: AtomicUsize,
    // the current index for writing the next delay sample from the buffer
    sample_buffer_write_index: AtomicUsize,
    // leader buffer, we don't start reading and incrementing other buffers until we have written
    // this many initializing samples
    init_buffer_index: AtomicUsize,
    // which delay window we are in, used to calculate decay factor
    cur_delay_window: AtomicUsize,
    // position in bit vector of entire length of all delay windows
    delay_windows_index: AtomicUsize,
    // false if the sample manager can still write more samples
    is_full: AtomicBool,
    // true if the sample manager hasn't finished going through its delay windows
    is_active: AtomicBool,
    // true if the sample manager is in the pre-delay buffer
    is_pre_delay: AtomicBool,
    // true if the sample manager is in a delay window
    is_in_delay_window: AtomicBool,
    // true if the sample manager is in an interval
    is_in_interval: AtomicBool,
    // true if the sample manager has spawned a new sample manager; this will happend once
    // for each Manager when it gets full, up to the global limit of MAX_NUM_ACTIVE_SAMPLE_MANAGERS
    has_spawned: AtomicBool,
}

#[allow(dead_code)]
impl SampleManager {
 
    pub(crate) fn next_sample(&mut self, sample: f32) -> f32 {
        let mut delay_sample = 0.0f32;
        
        // if we are in the pre-delay buffer, increment the write index, add the sample to the
        // buffer and return 0
        if self.is_pre_delay.load(Ordering::SeqCst) {
            let mut buffer = self.sample_buffer.write().unwrap();
            buffer.push_back(sample);
            self.sample_buffer_write_index.fetch_add(1, Ordering::SeqCst);
            if self.sample_buffer_write_index.load(Ordering::SeqCst) == PREDELAY_BUFFER_SIZE {
                self.is_pre_delay.store(false, Ordering::SeqCst);
            }
            return 0f32;
        }
        
        // if the buffer holding the samples being repeated in each delay window is not full,
        // add the sample to the buffer
        if !self.is_full.load(Ordering::SeqCst) {
            let mut buffer = self.sample_buffer.write().unwrap();
            buffer.push_back(sample);
            self.sample_buffer_write_index.fetch_add(1, Ordering::SeqCst);
            if self.sample_buffer_write_index.load(Ordering::SeqCst) ==
                    self.sample_buffer_size - self.num_predelay_samples {
                self.is_full.store(true, Ordering::SeqCst);
            }
        }
        
        // check if we are in a delay window or an interval by checking current delay window value
        if self.delay_windows[self.delay_windows_index.load(Ordering::SeqCst)] {
            {
                let buffer = self.sample_buffer.read().unwrap();
                delay_sample =
                    *buffer.get(self.sample_buffer_read_index.load(Ordering::SeqCst) %
                                self.sample_buffer_size).unwrap_or(&0.0);
            }
            // If this is the first sample in the delay window, increment the delay window index
            if self.sample_buffer_read_index.load(Ordering::SeqCst) == 0 {
                self.cur_delay_window.fetch_add(1, Ordering::SeqCst);
            }
            self.sample_buffer_read_index.fetch_add(1, Ordering::SeqCst);
        }
        
        // check for reaching the end of the delay windows
        if !self.is_pre_delay.load(Ordering::SeqCst) {
            self.delay_windows_index.fetch_add(1, Ordering::SeqCst);
        }
        if self.delay_windows_index.load(Ordering::SeqCst) ==
                self.delay_windows.len() - self.num_predelay_samples {
            self.reset();
        }

        delay_sample 
    }
    
    // DO NOT reset has_spawned, it is used to track if the sample manager has spawned a new
    // sample manager; this will happen once for each Manager when it gets full, up to the
    // global limit of MAX_NUM_ACTIVE_SAMPLE_MANAGERS
    pub(crate) fn reset(&mut self) {
        self.sample_buffer_read_index.store(0, Ordering::SeqCst);
        self.sample_buffer_write_index.store(0, Ordering::SeqCst);
        self.init_buffer_index.store(0, Ordering::SeqCst);
        self.cur_delay_window.store(0, Ordering::SeqCst);
        self.delay_windows_index.store(0, Ordering::SeqCst);
        self.is_full.store(false, Ordering::SeqCst);
        self.is_active.store(true, Ordering::SeqCst);
        self.is_pre_delay.store(true, Ordering::SeqCst);
        self.is_in_delay_window.store(true, Ordering::SeqCst);
        self.is_in_interval.store(false, Ordering::SeqCst);

        let mut buffer =
            self.sample_buffer.write().unwrap();
        buffer.clear();
    }

    pub(crate) fn dump_print(&self) {
        if self.is_active.load(Ordering::SeqCst) {
            println!("--------------------------------");
            println!("id: {}", self.id);
            println!("sample_buffer_size: {}", self.sample_buffer_size);
            println!("sample_buffer_read_index: {}", self.sample_buffer_read_index.load(Ordering::SeqCst));
            println!("sample_buffer_write_index: {}", self.sample_buffer_write_index.load(Ordering::SeqCst));
            println!("num_delay_windows: {}", self.num_delay_windows);
            println!("num_predelay_samples: {}", self.num_predelay_samples);
            println!("init_buffer_index: {}", self.init_buffer_index.load(Ordering::SeqCst));
            println!("cur_delay_window: {}", self.cur_delay_window.load(Ordering::SeqCst));
            println!("delay_windows_index: {}", self.delay_windows_index.load(Ordering::SeqCst));
            println!("is_full: {}", self.is_full.load(Ordering::SeqCst));
            println!("is_active: {}", self.is_active.load(Ordering::SeqCst));
            println!("is_pre_delay: {}", self.is_pre_delay.load(Ordering::SeqCst));
            println!("is_in_delay_window: {}", self.is_in_delay_window.load(Ordering::SeqCst));
            println!("is_in_interval: {}", self.is_in_interval.load(Ordering::SeqCst));
            println!("has_spawned: {}", self.has_spawned.load(Ordering::SeqCst));
            println!("--------------------------------");
        }
    }
}

impl Clone for SampleManager {
    fn clone(&self) -> Self {
        SampleManager {
            id: self.id,
            sample_buffer_size: self.sample_buffer_size,
            sample_buffer: Arc::new(RwLock::new(VecDeque::with_capacity(self.sample_buffer_size))),
            delay_windows: self.delay_windows.clone(),
            num_delay_windows: self.num_delay_windows,
            num_predelay_samples: self.num_predelay_samples,
            sample_buffer_read_index: AtomicUsize::new(self.sample_buffer_read_index.load(Ordering::SeqCst)),
            sample_buffer_write_index: AtomicUsize::new(self.sample_buffer_write_index.load(Ordering::SeqCst)),
            init_buffer_index: AtomicUsize::new(self.init_buffer_index.load(Ordering::SeqCst)),
            cur_delay_window: AtomicUsize::new(self.cur_delay_window.load(Ordering::SeqCst)),
            delay_windows_index: AtomicUsize::new(self.delay_windows_index.load(Ordering::SeqCst)),
            is_full: AtomicBool::new(self.is_full.load(Ordering::SeqCst)),
            is_active: AtomicBool::new(self.is_active.load(Ordering::SeqCst)),
            is_pre_delay: AtomicBool::new(self.is_pre_delay.load(Ordering::SeqCst)),
            is_in_delay_window: AtomicBool::new(self.is_in_delay_window.load(Ordering::SeqCst)),
            is_in_interval: AtomicBool::new(self.is_in_interval.load(Ordering::SeqCst)),
            has_spawned: AtomicBool::new(self.has_spawned.load(Ordering::SeqCst)),
        }
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
        if in_window {
            delay_windows.push(true);
        } else {
            delay_windows.push(false);
        }
        
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

        let sample_manager_id_counter = 0;
        let sample_manager_is_full_counter = 0;
        let duration_num_samples = duration_ms as usize * SAMPLES_PER_MS as usize;
        let interval_num_samples = interval_ms as usize * SAMPLES_PER_MS as usize;
        
        // initialize the delay with one active SampleManager
        add_sample_manager(SampleManagerConfig {
            id,
            sm_id: next_sample_manager_id(),
            sample_buffer_size: duration_num_samples,
            delay_windows: build_delay_windows(duration_num_samples, interval_num_samples, num_repeats),
            num_delay_windows: num_repeats,
            num_predelay_samples,
            sample_buffer_read_index: 0,
            sample_buffer_write_index: 0,
            init_buffer_index: 0,
            cur_delay_window: 0,
            delay_windows_index: 0,
        });

        let mix_complement = 1.0 - mix;
        
        Ok(
            Delay {
                // public
                id,
                mix,
                decay,
                interval_ms,
                duration_ms,
                num_repeats,
                num_predelay_samples,
                num_concurrent_sample_managers,
                // private
                sample_manager_id_counter,
                sample_manager_is_full_counter,
                mix_complement,
                // window_size,
                delay_windows: build_delay_windows(duration_num_samples, interval_num_samples,
                                                   num_repeats),
                duration_num_samples,
                interval_num_samples,
            }
        )
    }
}

#[allow(dead_code)]
impl Delay {
    
    pub(crate) fn apply_effect(&mut self, sample: f32, _sample_clock: f32) -> f32 {
        let delay_sample = Arc::new(Mutex::new(0.0f32));
        let num_delay_samples = AtomicUsize::new(0);
        let push = AtomicBool::new(false);
        
        // Process all samples under one lock
        {
            let mut managers = ACTIVE_SAMPLE_MANAGERS.lock().unwrap();
            if let Some(sample_managers) = managers.get_mut(&self.id) {
                for sample_manager in sample_managers.iter_mut() {
                    let next_sample = sample_manager.next_sample(sample) *
                        self.decay.powi(sample_manager.cur_delay_window.load(Ordering::SeqCst) as i32);
                    
                    // Update delay sample under one lock
                    {
                        let mut current = delay_sample.lock().unwrap();
                        *current += next_sample;
                    }
                    
                    num_delay_samples.fetch_add(1, Ordering::SeqCst);
                    
                    if !sample_manager.has_spawned.load(Ordering::SeqCst) &&
                            sample_manager.is_full.load(Ordering::SeqCst) {
                        sample_manager.has_spawned.store(true, Ordering::SeqCst);
                        push.store(true, Ordering::SeqCst);
                    }
                }
            }
        }

        // Get final value under one lock
        let final_value = {
            let mut value = delay_sample.lock().unwrap();
            if num_delay_samples.load(Ordering::SeqCst) > 0 {
                *value /= num_delay_samples.load(Ordering::SeqCst) as f32;
            }
            *value
        };

        // Add new manager outside the lock
        // enforce global limit on number of active sample managers
        if push.load(Ordering::SeqCst) &&
                *SAMPLE_MANAGER_ID_COUNTER.lock().unwrap() < self.num_concurrent_sample_managers {
            add_sample_manager(SampleManagerConfig {
                id: self.id,
                sm_id: next_sample_manager_id(),
                sample_buffer_size: self.duration_num_samples,
                delay_windows: self.delay_windows.clone(),
                num_delay_windows: self.num_repeats,
                num_predelay_samples: self.num_predelay_samples,
                sample_buffer_read_index: 0,
                sample_buffer_write_index: 0,
                init_buffer_index: 0,
                cur_delay_window: 0,
                delay_windows_index: 0,
            });
        }
        push.store(false, Ordering::SeqCst);

        self.mix_complement * sample + (self.mix * final_value)

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
