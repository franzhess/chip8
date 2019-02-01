use sdl2::audio::*;
use sdl2::Sdl;

pub struct Sound {
  device: AudioDevice<SquareWave>
}

impl Sound {
  pub fn new(sdl: &Sdl) -> Sound {
    let audio_subsystem = sdl.audio().unwrap();

    let desired_spec = AudioSpecDesired {
      freq: Some(44100),
      channels: Some(1),  // mono
      samples: None,       // default sample size
    };

    let device = audio_subsystem.open_playback(None, &desired_spec, |spec| {
      // initialize the audio callback
      SquareWave {
        phase_inc: 440.0 / spec.freq as f32,
        phase: 0.0,
        volume: 0.25,
      }
    }).unwrap();

    Sound {
      device
    }
  }

  pub fn play(&mut self) {
    match self.device.status() {
      AudioStatus::Paused | AudioStatus::Stopped => self.device.resume(),
      _ => ()
    }
  }

  pub fn stop(&mut self) {
    match self.device.status() {
      AudioStatus::Playing => self.device.pause(),
      _ => ()
    }
  }
}

struct SquareWave {
  phase_inc: f32,
  phase: f32,
  volume: f32,
}

impl AudioCallback for SquareWave {
  type Channel = f32;

  fn callback(&mut self, out: &mut [f32]) {
    // Generate a square wave
    for x in out.iter_mut() {
      *x = self.volume * if self.phase < 0.5 { 1.0 } else { -1.0 };
      self.phase = (self.phase + self.phase_inc) % 1.0;
    }
  }
}