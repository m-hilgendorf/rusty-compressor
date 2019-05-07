 #![allow(unreachable_code)]
use vst::plugin::*;
use crossbeam::channel::{Sender, TrySendError};
use vst::util::AtomicFloat;

#[derive(Copy,Clone,Debug)]
pub enum ParamType { 
    Threshold(f32), 
    Ratio(f32),
    Attack(f32),
    Release(f32),
    Makeup(f32),
}

pub struct CompressorParameters {
    threshold : AtomicFloat,
    ratio     : AtomicFloat,
    attack    : AtomicFloat,
    release   : AtomicFloat,
    makeup    : AtomicFloat,
    channel   : Sender<ParamType>
}

impl CompressorParameters {
    pub fn new(channel : Sender<ParamType>) -> Self {
        Self {
            threshold : AtomicFloat::new(Self::uncook(0, 0.0)),
            ratio     : AtomicFloat::new(Self::uncook(1, 2.0)),
            attack    : AtomicFloat::new(Self::uncook(2, 25.0)),
            release   : AtomicFloat::new(Self::uncook(3, 50.0)),
            makeup    : AtomicFloat::new(Self::uncook(4, 0.0)),
            channel 
        }
    }
    
    fn cook (index : i32, value : f32) -> f32 {
        match index {
            0 => -96.0 + (108.0) * value, 
            1 => 1.0   + 9.0 * value,
            2 => 0.1  + (1000.1) * value, 
            3 => 0.1  + (1000.1) * value,
            4 => 40.0 * value,
            _ => { unreachable!(); 0.0 }  
        }
    }

    fn uncook (index : i32, value : f32) -> f32 {
        match index {
            0 => (value + 96.0) / 108.0,
            1 => (value - 1.0) / 9.0, 
            2 => (value - 0.1) / 1000.1, 
            3 => (value - 0.1) / 1000.1,
            4 => value / 40.0,
            _ => {unreachable!(); 0.0 }
        }
    }
}

impl PluginParameters for CompressorParameters {
    fn get_parameter_label(&self, index: i32) -> String {
        println!("get parameter label");
        match index {
            0 => "dB".to_string(),
            1 => "".to_string(),
            2 => "ms".to_string(),
            3 => "ms".to_string(),
            4 => "dB".to_string(),
            _ => unreachable!()
        }
    }
    
    fn get_parameter_text(&self, index: i32) -> String {
        println!("get parameter text");
        let val = self.get_parameter(index);
        format!("{}", Self::cook(index, val))
    }

    fn get_parameter_name(&self, index: i32) -> String {
        println!("get parameter name");
        match index {
            0 => "Threshold".to_string(),
            1 => "Ratio".to_string(),
            2 => "Attack".to_string(),
            3 => "Release".to_string(),
            4 => "Makeup".to_string(),
            _ => unreachable!()
        }
    }

    fn get_parameter(&self, index: i32) -> f32 {
        println!("get parameter");
        match index {
            0 => self.threshold.get(),
            1 => self.ratio.get(),
            2 => self.attack.get(),
            3 => self.release.get(),
            4 => self.makeup.get(),
            _ => { unreachable!(); 0.0 }
        }
    }

    fn set_parameter(&self, index: i32, raw: f32) {
        println!("idx:{} -- val: {}", index, raw);
        let value = Self::cook(index, raw);

        let cooked = match index {
            0 => {
                self.threshold.set(raw);
                ParamType::Threshold(value)
            },
            1 => {
                self.ratio.set(raw);
                ParamType::Ratio(value)
            },
            2 => {
                self.attack.set(raw);
                ParamType::Attack(value)
            },
            3 => {
                self.release.set(raw);
                ParamType::Release(value)
            },
            4 => {
                self.makeup.set(raw);
                ParamType::Makeup(value)
            },
            _ => { unreachable!(); ParamType::Threshold(0.0) }  
        };
    
        'l: while let Err(e) = self.channel.try_send (cooked) {
            match e {
                TrySendError::Disconnected(_) => break 'l,
                _ => continue,
            };
        }
        let _err = self.channel.send (cooked);
    }

    fn can_be_automated(&self, _: i32) -> bool {
        println!("can be automated");
        true
    }

    fn string_to_parameter(&self, index: i32, text: String) -> bool {
        println!("string to parameter");
        self.set_parameter(index, Self::uncook(index, text.parse().expect("failed to parse string")));
        true
    }
}
