#[macro_use]
extern crate vst;

use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use vst::buffer::AudioBuffer;

struct Plugin {
    params: Arc<PluginParameters>,
    state: State,
}

impl Default for Plugin {
    fn default() -> Plugin {
        Plugin {
            params: Arc::new(PluginParameters {
                samples_to_average: AtomicUsize::new(5),
            }),
            state: State {
                started: false,
                recent_samples: Default::default(),
                recent_samples_copy: Default::default(),
            },
        }
    }
}

struct PluginParameters {
    samples_to_average: AtomicUsize,
}

struct State {
    started: bool,
    recent_samples: Vec<f32>,
    recent_samples_copy: Vec<f32>,
}

impl vst::prelude::Plugin for Plugin {
    fn new(_host: vst::prelude::HostCallback) -> Self {
        Default::default()
    }

    fn process(&mut self, buffer: &mut AudioBuffer<f32>) {
        for (input_buffer, output_buffer) in buffer.zip() {
            self.state.started = false;
            self.state.recent_samples =
                vec![Default::default(); self.params.samples_to_average.load(Ordering::Relaxed)];

            for (input_sample, output_sample) in input_buffer.iter().zip(output_buffer) {
                if !self.state.started {
                    self.state.started = true;
                    self.state
                        .recent_samples
                        .iter_mut()
                        .for_each(|x| *x = *input_sample);
                } else {
                    self.state.recent_samples_copy = self.state.recent_samples.clone();

                    for item in self.state.recent_samples.iter_mut().enumerate() {
                        let (index, element): (usize, &mut f32) = item;
                        if index == self.params.samples_to_average.load(Ordering::Relaxed) - 1 {
                            *element = *input_sample;
                        } else {
                            *element = self.state.recent_samples_copy[index + 1];
                        }
                    }

                    *output_sample = (1.
                        + 0.01 * self.params.samples_to_average.load(Ordering::Relaxed) as f32)
                        * self.state.recent_samples.iter().sum::<f32>()
                        / self.params.samples_to_average.load(Ordering::Relaxed) as f32;
                }
            }
        }
    }

    fn get_parameter_object(&mut self) -> Arc<dyn vst::prelude::PluginParameters> {
        Arc::clone(&self.params) as Arc<dyn vst::prelude::PluginParameters>
    }

    fn get_info(&self) -> vst::prelude::Info {
        vst::prelude::Info {
            name: "VST".to_string(),
            unique_id: 666420, // used by hosts to differentiate between plugins
            category: vst::prelude::Category::Effect,

            inputs: 2,
            outputs: 2,
            parameters: 1,

            ..Default::default()
        }
    }
}

impl vst::prelude::PluginParameters for PluginParameters {
    fn get_parameter(&self, index: i32) -> f32 {
        match index {
            0 => self.samples_to_average.load(Ordering::Relaxed) as f32,
            _ => 0.0,
        }
    }

    fn get_parameter_name(&self, index: i32) -> String {
        match index {
            0 => "Samples to average",
            _ => "",
        }
        .to_string()
    }

    fn get_parameter_text(&self, index: i32) -> String {
        match index {
            0 => format!("{}", self.samples_to_average.load(Ordering::Relaxed)),
            _ => "".to_string(),
        }
    }

    fn set_parameter(&self, index: i32, value: f32) {
        if index == 0 {
            if value.clamp(0.0, 1.0) == 0.0 {
                self.samples_to_average.store(1, Ordering::Relaxed);
            } else {
                self.samples_to_average.store(
                    (value.clamp(0.0, 1.0) * 100.0).round() as usize,
                    Ordering::Relaxed,
                );
            }
        }
    }
}

plugin_main!(Plugin);
