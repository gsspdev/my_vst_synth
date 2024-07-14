#[macro_use]
extern crate vst;

use vst::api::{Events, Supported};
use vst::buffer::AudioBuffer;
use vst::event::Event;
use vst::plugin::{Category, Info, Plugin, PluginParameters};
use vst::util::AtomicFloat;
use vst::prelude::HostCallback;


use std::sync::Arc;
use std::f32::consts::PI;
use rand::Rng;

// Oscillator type enum
#[derive(Clone, Copy)]
enum OscType {
    Sine,
    Saw,
    Square,
    Triangle,
    Noise,
}

enum EnvelopeStage {
    Idle,
    Attack,
    Decay,
    Sustain,
    Release,
}

// Oscillator struct
struct Oscillator {
    phase: f32,
    freq: f32,
    osc_type: OscType,
}

impl Oscillator {
    fn new(osc_type: OscType) -> Oscillator {
        Oscillator {
            phase: 0.0,
            freq: 440.0,
            osc_type,
        }
    }

    fn generate_sample(&mut self, sample_rate: f32) -> f32 {
        let output = match self.osc_type {
            OscType::Sine => (self.phase * 2.0 * PI).sin(),
            OscType::Saw => 1.0 - (2.0 * self.phase),
            OscType::Square => if self.phase < 0.5 { 1.0 } else { -1.0 },
            OscType::Triangle => {
                if self.phase < 0.5 {
                    4.0 * self.phase - 1.0
                } else {
                    3.0 - 4.0 * self.phase
                }
            },
            OscType::Noise => rand::thread_rng().gen_range(-1.0..1.0),
        };

        self.phase += self.freq / sample_rate;
        if self.phase >= 1.0 {
            self.phase -= 1.0;
        }

        output
    }
}    

// ADSR envelope
struct Envelope {
    attack: f32,
    decay: f32,
    sustain: f32,
    release: f32,
    stage: EnvelopeStage,
    level: f32,
    sample_rate: f32,
}

impl Envelope {
    fn new(sample_rate: f32, ) -> Envelope {
        Envelope {
            attack: 0.01,
            decay: 0.1,
            sustain: 0.5,
            release: 0.2,
            stage: EnvelopeStage::Idle,
            level: 0.0,
            sample_rate,
        }
    }
    fn trigger(&mut self) {
        self.stage = EnvelopeStage::Attack;
        self.level = 0.0;
    }

    fn d_stage(&mut self) {
        self.stage = EnvelopeStage::Decay;
    }

    fn s_stage(&mut self) {
        self.stage = EnvelopeStage::Decay;
    }

    fn r_stage(&mut self) {
        self.stage = EnvelopeStage::Release;
    }

    fn mod_atk(&mut self, new_value: f32) -> f32 {
        self.attack =  new_value;
        new_value
    }

    fn mod_dec(&mut self, new_value: f32) -> f32 {
        self.decay = new_value;
        new_value
    }

    fn mod_sus(&mut self, new_value: f32) -> f32 {
        self.sustain = new_value;
        new_value
    }

    fn mod_rel(&mut self, new_value: f32) -> f32 {
        self.release = new_value;
        new_value
    }
    fn process(&mut self) -> f32 {
        match self.stage {
            EnvelopeStage::Idle => self.level = 0.0,
            EnvelopeStage::Attack => {
                self.level += 1.0 / (self.attack * self.sample_rate);
                if self.level >= 1.0 {
                    self.level = 1.0;
                    self.stage = EnvelopeStage::Decay;
                }
            }
            EnvelopeStage::Decay => {
                self.level -= (1.0 - self.sustain) / (self.decay * self.sample_rate);
                if self.level <= self.sustain {
                    self.level = self.sustain;
                    self.stage = EnvelopeStage::Sustain;
                }
            }
            EnvelopeStage::Sustain => (),
            EnvelopeStage::Release => {
                self.level -= self.level / (self.release * self.sample_rate);
                if self.level <= 0.001 {
                    self.level = 0.0;
                    self.stage = EnvelopeStage::Idle;
                }
            }
        }
        self.level
    }
}

// Simple low-pass filter
struct LowPassFilter {
    cutoff: f32,
    resonance: f32,
    y1: f32,
    y2: f32,
}

impl LowPassFilter {
    fn new() -> LowPassFilter {
        LowPassFilter {
            cutoff: 1000.0,
            resonance: 0.5,
            y1: 0.0,
            y2: 0.0,
        }
    }

    fn process(&mut self, input: f32, sample_rate: f32) -> f32 {
        let c = 2.0 * PI * self.cutoff / sample_rate;
        let _r = 1.0 / (2.0 * (1.0 - self.resonance));
        let k = c / (1.0 + c);

        let output = input * k + self.y1 * (1.0 - k);
        self.y1 = output * k + self.y2 * (1.0 - k);
        self.y2 = output;

        output
    }
}

// LFO (Low Frequency Oscillator)
struct LFO {
    phase: f32,
    freq: f32,
}

impl LFO {
    fn new() -> LFO {
        LFO {
            phase: 0.0,
            freq: 2.4,
        }
    }

    fn process(&mut self, sample_rate: f32) -> f32 {
        let output = (self.phase * 2.0 * PI).sin();
        self.phase += self.freq / sample_rate;
        if self.phase >= 1.0 {
            self.phase -= 1.0;
        }
        output
    }
}

struct MySynth {
    sample_rate: f32,
    oscillators: [Oscillator; 5],
    envelope: Envelope,
    filter: LowPassFilter,
    lfo: LFO,
    note: u8,
    note_on: bool,
    params: Arc<SynthParams>,
}

struct SynthParams {
    osc1_freq: AtomicFloat,
    osc2_freq: AtomicFloat,
    filter_cutoff: AtomicFloat,
    filter_resonance: AtomicFloat,
    lfo_freq: AtomicFloat,
    lfo_amount: AtomicFloat,
}

impl Default for MySynth {
    fn default() -> Self {
        MySynth {
            sample_rate: 44100.0,
            oscillators: [
                Oscillator::new(OscType::Sine),
                Oscillator::new(OscType::Saw),
                Oscillator::new(OscType::Square),
                Oscillator::new(OscType::Triangle),
                Oscillator::new(OscType::Noise),
            ],
            envelope: Envelope::new(44100.0),
            filter: LowPassFilter::new(),
            lfo: LFO::new(),
            note: 0,
            note_on: false,
            params: Arc::new(SynthParams {
                osc1_freq: AtomicFloat::new(440.0),
                osc2_freq: AtomicFloat::new(440.0),
                filter_cutoff: AtomicFloat::new(1000.0),
                filter_resonance: AtomicFloat::new(0.5),
                lfo_freq: AtomicFloat::new(1.0),
                lfo_amount: AtomicFloat::new(0.5),
            }),
        }
    }
}

impl Plugin for MySynth {
    fn new(_host: HostCallback) -> Self {
        MySynth::default()
    }

    fn get_info(&self) -> Info {
        Info {
            name: "Advanced VST Synth".to_string(),
            vendor: "Your Name".to_string(),
            unique_id: 1234,
            category: Category::Synth,
            inputs: 0,
            outputs: 2,
            parameters: 6,
            ..Default::default()
        }
    }

    fn init(&mut self) {
        self.sample_rate = 44100.0;
    }

    fn process(&mut self, buffer: &mut AudioBuffer<f32>) {
        let samples = buffer.samples();
        let (_, mut outputs) = buffer.split();
        let output_count = outputs.len();

        for sample_idx in 0..samples {
            let mut output = 0.0;

            if self.note_on {
                for osc in &mut self.oscillators {
                    output += osc.generate_sample(self.sample_rate);
                }
                output *= 0.5; // Mix oscillators

                output *= self.envelope.process();

                // Apply LFO to filter cutoff
                let lfo_value = self.lfo.process(self.sample_rate);
                let cutoff_mod = self.params.filter_cutoff.get() * (1.0 + lfo_value * self.params.lfo_amount.get());
                self.filter.cutoff = cutoff_mod.clamp(20.0, 20000.0);

                output = self.filter.process(output, self.sample_rate);
            }

            for buf_idx in 0..output_count {
                outputs[buf_idx][sample_idx] = output;
            }
        }
    }

    fn process_events(&mut self, events: &Events) {
        for event in events.events() {
            match event {
                Event::Midi(ev) => {
                    match ev.data[0] {
                        128 => { // Note Off
                            if ev.data[1] == self.note {
                                self.envelope.r_stage();
                            }
                        },
                        144 => { // Note On
                            self.note = ev.data[1];
                            let freq = midi_pitch_to_freq(self.note);
                            self.oscillators[0].freq = freq;
                            self.oscillators[1].freq = freq * 1.01; // Slight detuning for second oscillator
                            self.envelope.trigger();
                            self.note_on = true;
                        },
                        _ => (),
                    }
                },
                _ => (),
            }
        }
    }

    fn can_do(&self, can_do: vst::plugin::CanDo) -> vst::api::Supported {
        match can_do {
            vst::plugin::CanDo::ReceiveMidiEvent => Supported::Yes,
            _ => Supported::Maybe
        }
    }

    fn get_parameter_object(&mut self) -> Arc<dyn PluginParameters> {
        Arc::clone(&self.params) as Arc<dyn PluginParameters>
    }
}

impl PluginParameters for SynthParams {
    fn get_parameter(&self, index: i32) -> f32 {
        match index {
            0 => self.osc1_freq.get() / 20000.0,
            1 => self.osc2_freq.get() / 20000.0,
            2 => self.filter_cutoff.get() / 20000.0,
            3 => self.filter_resonance.get(),
            4 => self.lfo_freq.get() / 10.0,
            5 => self.lfo_amount.get(),
            _ => 0.0,
        }
    }

    fn set_parameter(&self, index: i32, value: f32) {
        match index {
            0 => self.osc1_freq.set(value * 20000.0),
            1 => self.osc2_freq.set(value * 20000.0),
            2 => self.filter_cutoff.set(value * 20000.0),
            3 => self.filter_resonance.set(value),
            4 => self.lfo_freq.set(value * 20.0),
            5 => self.lfo_amount.set(value * 2.0),
            _ => (),
        }
    }

    fn get_parameter_name(&self, index: i32) -> String {
        match index {
            0 => "Osc1 Freq".to_string(),
            1 => "Osc2 Freq".to_string(),
            2 => "Filter Cutoff".to_string(),
            3 => "Filter Resonance".to_string(),
            4 => "LFO Freq".to_string(),
            5 => "LFO Amount".to_string(),
            _ => "".to_string(),
        }
    }
}

fn midi_pitch_to_freq(pitch: u8) -> f32 {
    const A4_PITCH: i8 = 69;
    const A4_FREQ: f32 = 440.0;

    (f32::from(pitch as i8 - A4_PITCH) / 12.0).exp2() * A4_FREQ
}

plugin_main!(MySynth);
