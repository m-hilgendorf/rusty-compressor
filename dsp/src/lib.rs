 #[derive(Copy,Clone,Debug)]
 pub struct Compressor {
        pub peak_at   : f32, //< variables for the peak follower 
        pub peak_rt   : f32, 
        pub peak_avg  : f32, 

        pub gain_at   : f32, //< variables for the gain smoother
        pub gain_rt   : f32, 
        pub gain_avg  : f32, 

        pub threshold : f32, //< gain calculation vars 
        pub ratio     : f32,

        pub sample_rate : f32, //< we need to hold onto these
        pub att_ms : f32,      //  values because the sample rate
        pub rel_ms : f32,      //  may change. 
 }

impl Compressor {
    pub fn calc_tau (&self, time_ms : f32) -> f32 {
        1.0 - (-2200.0 / (time_ms * self.sample_rate)).exp()
    }

    fn ar_avg (avg : &mut f32,  at : f32, rt : f32, input : f32) -> f32 {
        let tau = if input > *avg { at } else { rt };
        *avg = (1.0 - tau) * (*avg) + tau * input;
        *avg
    }

    fn gain_calc (input : f32, threshold : f32, ratio : f32) -> f32 {
        let db   = 20.0f32 * input.abs().log10();
        let gain = ((1.0 - 1.0 / ratio) * (threshold - db)).min(0.0);
        10.0f32.powf (0.05 * gain)
    }

    pub fn compress (&mut self, input : f32) -> (f32, f32, f32, f32) {
        let peak     = Self::ar_avg   (&mut self.peak_avg, self.peak_at, self.peak_rt, input.abs());
        let gain     = Self::gain_calc(peak, self.threshold, self.ratio);
        let smoothed = Self::ar_avg   (&mut self.gain_avg, self.gain_rt, self.gain_at, gain);
        let outp = smoothed * input;
        (outp, peak, gain, smoothed)
    }

     // initialization
     pub fn new() -> Self {
         Self {
            peak_at : 0.01, 
            peak_rt : 0.1,
            peak_avg : 0.0,
            gain_at : 0.01,
            gain_rt : 0.1, gain_avg : 1.0,
            threshold : 0.0, ratio : 2.0,
            sample_rate : 48000.0,
            att_ms : 10.0,
            rel_ms : 50.0,
         }.with_sample_rate(48000.0)
     }

     pub fn with_sample_rate(mut self, fs : f32) -> Self {
         self.sample_rate = fs;
         self.peak_at = self.calc_tau(0.01);
         self.peak_rt = self.calc_tau(10.0);
         self.gain_at = self.calc_tau(self.att_ms);
         self.gain_rt = self.calc_tau(self.rel_ms);
         self
     }

     pub fn with_attack(mut self, att_ms : f32) -> Self {
         self.att_ms = att_ms;
         self.gain_at = self.calc_tau(att_ms);
         self
     }

     pub fn with_release(mut self, rel_ms : f32) -> Self {
         self.rel_ms = rel_ms;
         self.gain_rt = self.calc_tau(rel_ms);
         self
     }

     pub fn with_threshold (mut self, thresh : f32) -> Self {
         self.threshold = thresh;
         self
     }

     pub fn with_ratio (mut self, ratio : f32) -> Self {
         self.ratio = ratio;
         self
     }

     pub fn reset(&mut self) {
         self.gain_avg = 1.0;
         self.peak_avg = 0.0;
     }

     pub fn set_attack(&mut self, att :f32) {
         self.att_ms = att;
         self.gain_at = self.calc_tau(att);
     }

     pub fn set_release(&mut self, rel :f32) {
         self.rel_ms = rel;
         self.gain_rt = self.calc_tau(rel);
     }
 }

#[cfg(test)]
mod test_signals;

#[cfg(test)]
mod tests {
    use super::*;
    use test_signals::*;
    
    #[test]
    fn model_reference () {
        let test = |measured : &[f32], expected : &[f32]| {
            let thresh = 0.00001; //-100dB
            let scl = measured.len() as f32;
            let diff = measured
                .iter()
                .zip(expected.iter())
                .fold(0.0, |d, it| (it.0 - it.1).abs() + d) / scl;
            println!("avg d= {}", diff);
            assert!(diff <= thresh);
        };

        let mut comp = Compressor::new()
            .with_sample_rate(48000.0)
            .with_ratio(10.0)
            .with_threshold(-6.0)
            .with_attack(1.3)
            .with_release(2.6);

        let (mut outp, mut gain, mut peak, mut smoothed) = (Vec::new(), Vec::new(), Vec::new(), Vec::new());
        for x in CASE_1_INPUT.iter() {
             let (o, p, g, s) = comp.compress(*x);
             outp.push(o);
            gain.push(g);
             peak.push(p);
             smoothed.push(s);
        }

        test(&outp, &CASE_1_OUTP);
        test(&gain, &CASE_1_GAIN);
        test(&peak, &CASE_1_PEAK);
        test(&smoothed, &CASE_1_SMOOTHED); test(&smoothed, &CASE_2_SMOOTHED);
    }
}
