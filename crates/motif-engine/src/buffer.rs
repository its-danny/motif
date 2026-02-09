use std::ops::Range;

/// A fixed-capacity audio buffer. Allocated once, reused every callback.
///
/// PLANAR layout: each channel is a contiguous slice.
///     channel_data[0] = [L0, L1, L2, ...]
///     channel_data[1] = [R0, R1, R2, ...]
#[derive(Debug)]
pub struct AudioBuffer {
    data: Vec<Vec<f32>>,
    frames: usize,
}

impl AudioBuffer {
    pub fn new(channels: usize, max_frames: usize) -> Self {
        Self {
            data: (0..channels).map(|_| vec![0.0; max_frames]).collect(),
            frames: 0,
        }
    }

    /// Prepare for a new callback.
    pub fn prepare(&mut self, frames: usize) {
        self.frames = frames;

        for channel in &mut self.data {
            for sample in &mut channel[..frames] {
                *sample = 0.0;
            }
        }
    }

    pub fn frames(&self) -> usize {
        self.frames
    }

    pub fn channels(&self) -> usize {
        self.data.len()
    }

    /// Access a channel as a contiguous slice.
    pub fn channel(&self, ch: usize) -> &[f32] {
        &self.data[ch][..self.frames]
    }

    /// Mutable access to a channel.
    pub fn channel_mut(&mut self, ch: usize) -> &mut [f32] {
        &mut self.data[ch][..self.frames]
    }

    /// Access a sub-range of frames for one channel.
    pub fn channel_range_mut(&mut self, channel: usize, range: Range<usize>) -> &mut [f32] {
        &mut self.data[channel][range]
    }

    /// Additive mix. Sums other's samples into self. Used for combining
    /// track outputs into the master bus.
    pub fn mix_from(&mut self, other: &AudioBuffer) {
        debug_assert_eq!(self.frames, other.frames);
        debug_assert_eq!(self.channels(), other.channels());

        for channel in 0..self.channels() {
            let destination = &mut self.data[channel][..self.frames];
            let source = &other.data[channel][..self.frames];

            for idx in 0..self.frames {
                destination[idx] += source[idx];
            }
        }
    }

    /// Assumes stereo (channels 0 and 1). Used by GainPanNode.
    pub fn apply_stereo_gain(&mut self, gain_l: f32, gain_r: f32) {
        let frames = self.frames;

        let left = &mut self.data[0][..frames];
        for source in left.iter_mut() {
            *source *= gain_l;
        }

        let right = &mut self.data[1][..frames];
        for source in right.iter_mut() {
            *source *= gain_r;
        }
    }

    /// Convert planar →  interleaved for cpal output callback.
    pub fn write_interleaved(&self, output: &mut [f32]) {
        let channels = self.channels();

        for frame in 0..self.frames {
            for channel in 0..channels {
                output[frame * channels + channel] = self.data[channel][frame];
            }
        }
    }

    /// Borrow two channels mutably at once via split_at_mut.
    /// Needed for stereo DSP where both channels are written simultaneously.
    pub fn two_channels_mut(&mut self, ch_a: usize, ch_b: usize) -> (&mut [f32], &mut [f32]) {
        assert_ne!(ch_a, ch_b);

        let frames = self.frames;

        if ch_a < ch_b {
            let (left, right) = self.data.split_at_mut(ch_b);

            (&mut left[ch_a][..frames], &mut right[0][..frames])
        } else {
            let (left, right) = self.data.split_at_mut(ch_a);

            (&mut right[0][..frames], &mut left[ch_b][..frames])
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new() {
        let buffer = AudioBuffer::new(2, 512);

        assert_eq!(buffer.channels(), 2);
        assert_eq!(buffer.frames(), 0)
    }

    #[test]
    fn prepare() {
        let mut buffer = AudioBuffer::new(2, 512);
        buffer.prepare(256);

        let empty = vec![0.0_f32; 256];

        assert_eq!(buffer.frames(), 256);
        assert_eq!(buffer.channel(0), empty.as_slice());
        assert_eq!(buffer.channel(1), empty.as_slice());
    }

    #[test]
    fn channel_write_and_read() {
        let mut buffer = AudioBuffer::new(2, 512);
        buffer.prepare(4);

        let left = buffer.channel_mut(0);
        left[0] = 1.0;
        left[1] = 2.0;
        left[2] = 3.0;
        left[3] = 4.0;

        let right = buffer.channel_mut(1);
        right[0] = 5.0;
        right[1] = 6.0;
        right[2] = 7.0;
        right[3] = 8.0;

        assert_eq!(buffer.channel(0), &[1.0, 2.0, 3.0, 4.0]);
        assert_eq!(buffer.channel(1), &[5.0, 6.0, 7.0, 8.0]);
    }

    #[test]
    fn channel_slice_length_matches_frames_not_max() {
        let mut buffer = AudioBuffer::new(2, 512);
        buffer.prepare(64);

        assert_eq!(buffer.channel(0).len(), 64);
        assert_eq!(buffer.channel_mut(0).len(), 64);
    }

    #[test]
    fn mix_from_adds_values() {
        let mut destination = AudioBuffer::new(2, 4);
        destination.prepare(4);

        for sample in destination.channel_mut(0) {
            *sample = 0.5;
        }

        for sample in destination.channel_mut(1) {
            *sample = 0.5;
        }

        let mut source = AudioBuffer::new(2, 4);
        source.prepare(4);

        for sample in source.channel_mut(0) {
            *sample = 0.3;
        }

        for sample in source.channel_mut(1) {
            *sample = 0.3;
        }

        destination.mix_from(&source);

        for &sample in destination.channel(0) {
            assert!((sample - 0.8).abs() < 1e-6);
        }

        for &sample in destination.channel(1) {
            assert!((sample - 0.8).abs() < 1e-6);
        }
    }

    #[test]
    fn mix_from_into_zeroed_equals_source() {
        let mut destination = AudioBuffer::new(2, 4);
        destination.prepare(4);

        let mut source = AudioBuffer::new(2, 4);
        source.prepare(4);
        source.channel_mut(0).copy_from_slice(&[1.0, 2.0, 3.0, 4.0]);
        source.channel_mut(1).copy_from_slice(&[5.0, 6.0, 7.0, 8.0]);

        destination.mix_from(&source);

        assert_eq!(destination.channel(0), source.channel(0));
        assert_eq!(destination.channel(1), source.channel(1));
    }

    #[test]
    fn mix_from_is_additive() {
        let mut destination = AudioBuffer::new(2, 4);
        destination.prepare(4);

        let mut source = AudioBuffer::new(2, 4);
        source.prepare(4);

        for sample in source.channel_mut(0) {
            *sample = 1.0;
        }

        for sample in source.channel_mut(1) {
            *sample = 2.0;
        }

        destination.mix_from(&source);
        destination.mix_from(&source);

        for &sample in destination.channel(0) {
            assert!((sample - 2.0).abs() < 1e-6);
        }

        for &sample in destination.channel(1) {
            assert!((sample - 4.0).abs() < 1e-6);
        }
    }

    #[test]
    fn apply_stereo_gain_half_left() {
        let mut buffer = AudioBuffer::new(2, 4);
        buffer.prepare(4);

        for sample in buffer.channel_mut(0) {
            *sample = 1.0;
        }

        for sample in buffer.channel_mut(1) {
            *sample = 1.0;
        }

        buffer.apply_stereo_gain(0.5, 1.0);

        for &sample in buffer.channel(0) {
            assert!((sample - 0.5).abs() < 1e-6);
        }

        for &sample in buffer.channel(1) {
            assert!((sample - 1.0).abs() < 1e-6);
        }
    }

    #[test]
    fn apply_stereo_gain_zero() {
        let mut buffer = AudioBuffer::new(2, 4);
        buffer.prepare(4);

        for sample in buffer.channel_mut(0) {
            *sample = 1.0;
        }

        for sample in buffer.channel_mut(1) {
            *sample = 1.0;
        }

        buffer.apply_stereo_gain(0.0, 0.0);

        for &sample in buffer.channel(0) {
            assert_eq!(sample, 0.0);
        }

        for &sample in buffer.channel(1) {
            assert_eq!(sample, 0.0);
        }
    }

    #[test]
    fn apply_stereo_gain_unity() {
        let mut buffer = AudioBuffer::new(2, 4);
        buffer.prepare(4);

        buffer.channel_mut(0).copy_from_slice(&[0.1, 0.2, 0.3, 0.4]);
        buffer.channel_mut(1).copy_from_slice(&[0.5, 0.6, 0.7, 0.8]);

        buffer.apply_stereo_gain(1.0, 1.0);

        assert_eq!(buffer.channel(0), &[0.1, 0.2, 0.3, 0.4]);
        assert_eq!(buffer.channel(1), &[0.5, 0.6, 0.7, 0.8]);
    }

    #[test]
    fn write_interleaved_basic() {
        let mut buffer = AudioBuffer::new(2, 4);
        buffer.prepare(3);
        buffer.channel_mut(0).copy_from_slice(&[1.0, 2.0, 3.0]);
        buffer.channel_mut(1).copy_from_slice(&[4.0, 5.0, 6.0]);

        let mut output = vec![0.0_f32; 6];
        buffer.write_interleaved(&mut output);

        assert_eq!(output, &[1.0, 4.0, 2.0, 5.0, 3.0, 6.0]);
    }

    #[test]
    fn write_interleaved_single_frame() {
        let mut buffer = AudioBuffer::new(2, 4);
        buffer.prepare(1);
        buffer.channel_mut(0)[0] = 1.0;
        buffer.channel_mut(1)[0] = 2.0;

        let mut output = vec![0.0_f32; 2];
        buffer.write_interleaved(&mut output);

        assert_eq!(output, &[1.0, 2.0]);
    }

    #[test]
    fn two_channels_mut_independent() {
        let mut buffer = AudioBuffer::new(2, 4);
        buffer.prepare(4);

        let (left, right) = buffer.two_channels_mut(0, 1);

        for sample in left.iter_mut() {
            *sample = 1.0;
        }

        for sample in right.iter_mut() {
            *sample = 2.0;
        }

        assert_eq!(buffer.channel(0), &[1.0, 1.0, 1.0, 1.0]);
        assert_eq!(buffer.channel(1), &[2.0, 2.0, 2.0, 2.0]);
    }

    #[test]
    fn two_channels_mut_reversed_order() {
        let mut buffer = AudioBuffer::new(2, 4);
        buffer.prepare(4);

        let (right, left) = buffer.two_channels_mut(1, 0);

        for sample in left.iter_mut() {
            *sample = 3.0;
        }

        for sample in right.iter_mut() {
            *sample = 4.0;
        }

        assert_eq!(buffer.channel(0), &[3.0, 3.0, 3.0, 3.0]);
        assert_eq!(buffer.channel(1), &[4.0, 4.0, 4.0, 4.0]);
    }

    #[test]
    #[should_panic]
    fn two_channels_mut_same_channel_panics() {
        let mut buffer = AudioBuffer::new(2, 4);
        buffer.prepare(4);
        buffer.two_channels_mut(0, 0);
    }

    #[test]
    fn channel_range_mut_partial_write() {
        let mut buffer = AudioBuffer::new(2, 8);
        buffer.prepare(8);

        let slice = buffer.channel_range_mut(0, 2..5);
        slice[0] = 1.0;
        slice[1] = 2.0;
        slice[2] = 3.0;

        let full = buffer.channel(0);

        assert_eq!(full, &[0.0, 0.0, 1.0, 2.0, 3.0, 0.0, 0.0, 0.0]);
    }
}
