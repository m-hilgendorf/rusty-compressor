extern crate vst;
use vst::plugin::*;
use vst::buffer::AudioBuffer;
use vst::plugin_main;
use crossbeam::channel::{Receiver, bounded};
use std::sync::Arc;
use dsp::*;

mod params;
use params::*;

struct CompressorPlugin {
    channel : Option <Receiver<ParamType>>, 
    makeup  : f32, 
    dsp     : [Compressor; 2] // for stereo processing 
}

impl Default for CompressorPlugin {
    fn default() -> Self {
        Self {
            channel : None, 
            makeup  : 0.0, 
            dsp     : [Compressor::new()
                .with_attack(25.0)
                .with_release(50.0)
                .with_ratio(2.0)
                .with_threshold(0.0); 2]
        }
    }
}

impl Plugin for CompressorPlugin {
    fn get_info(&self) -> Info {
        Info {
            name : "Compressor".to_string(), 
            vendor : "Michael Hilgendorf".to_string(), 
            presets : 0, 
            parameters : 5,
            inputs : 2,     // stereo
            outputs : 2,    
            f64_precision : false,
            category : Category::Effect,
            unique_id : 0x2001_beef, 
            ..Default::default()
        } 
    }

    fn set_sample_rate(&mut self, rate : f32) {
        for dsp in &mut self.dsp { *dsp = dsp.with_sample_rate(rate) };
    }
    
    fn process (&mut self, buffer : &mut AudioBuffer<f32>) {
        // first we check if there are any pending parameter updates
        if let Some(channel) = &self.channel {
            while let Ok(change) = channel.try_recv() {
                match change {
                    ParamType::Threshold(t) => {
                        self.dsp[0].threshold = t;
                        self.dsp[1].threshold = t;
                    },
                    ParamType::Ratio(r) => {
                        self.dsp[0].ratio = r;
                        self.dsp[1].ratio = r;
                    },
                    ParamType::Attack(a) => {
                        self.dsp[0].set_attack(a);
                        self.dsp[1].set_attack(a);
                    },
                    ParamType::Release(r) => {
                        self.dsp[0].set_release(r);
                        self.dsp[1].set_release(r);
                    },
                    ParamType::Makeup(m) => { self.makeup = m; },
                }
            }
        }

        // then we do the signal processing 
        {
            let nchan = buffer
                .input_count()
                .min(buffer.output_count())
                .min(2); 

            let (inpt, outp) = buffer.split();
            for i in 0..nchan {
                let inpt = inpt.get(i).iter();
                let outp = outp.get_mut(i).iter_mut();
                let dsp  = &mut self.dsp[i];
            
                for (inpt, outp) in inpt.zip(outp) {
                    let (cmp, _, _, _) = dsp.compress (*inpt); 
                    *outp = cmp * 10.0f32.powf(self.makeup / 20.0);
                }
            }
        }
    }
    
    fn get_parameter_object(&mut self) -> Arc<dyn PluginParameters> {
        let (tx, rx) = bounded(2048);
        self.channel = Some(rx); 
        Arc::new(CompressorParameters::new(tx))
    }
}
plugin_main!(CompressorPlugin);