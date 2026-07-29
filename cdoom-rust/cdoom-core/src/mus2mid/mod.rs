//! MUS-to-MIDI conversion ported from `chocolate-doom/src/mus2mid.c`.

use std::sync::Mutex;

const NUM_CHANNELS: usize = 16;
const MIDI_PERCUSSION_CHAN: i32 = 9;
const MUS_PERCUSSION_CHAN: i32 = 15;

const MUS_RELEASE_KEY: u8 = 0x00;
const MUS_PRESS_KEY: u8 = 0x10;
const MUS_PITCH_WHEEL: u8 = 0x20;
const MUS_SYSTEM_EVENT: u8 = 0x30;
const MUS_CHANGE_CONTROLLER: u8 = 0x40;
const MUS_SCORE_END: u8 = 0x60;

const MIDI_RELEASE_KEY: u8 = 0x80;
const MIDI_PRESS_KEY: u8 = 0x90;
const MIDI_CHANGE_CONTROLLER: u8 = 0xB0;
const MIDI_CHANGE_PATCH: u8 = 0xC0;
const MIDI_PITCH_WHEEL: u8 = 0xE0;

const MIDI_HEADER: &[u8] = &[
    b'M', b'T', b'h', b'd', 0x00, 0x00, 0x00, 0x06, 0x00, 0x00, 0x00, 0x01, 0x00, 0x46, b'M', b'T',
    b'r', b'k', 0x00, 0x00, 0x00, 0x00,
];

const CONTROLLER_MAP: [u8; 15] = [
    0x00, 0x20, 0x01, 0x07, 0x0A, 0x0B, 0x5B, 0x5D, 0x40, 0x43, 0x78, 0x7B, 0x7E, 0x7F, 0x79,
];

static CHANNEL_VELOCITIES: Mutex<[u8; NUM_CHANNELS]> = Mutex::new([127; NUM_CHANNELS]);

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub struct Mus2MidError;

struct Reader<'a> {
    data: &'a [u8],
    position: usize,
}

impl<'a> Reader<'a> {
    fn new(data: &'a [u8]) -> Self {
        Self { data, position: 0 }
    }

    fn read_u8(&mut self) -> Result<u8, Mus2MidError> {
        let value = self.data.get(self.position).copied().ok_or(Mus2MidError)?;
        self.position += 1;
        Ok(value)
    }

    fn read_u16_le(&mut self) -> Result<u16, Mus2MidError> {
        let low = self.read_u8()?;
        let high = self.read_u8()?;
        Ok(u16::from_le_bytes([low, high]))
    }

    fn seek_set(&mut self, position: usize) -> Result<(), Mus2MidError> {
        if position < self.data.len() {
            self.position = position;
            Ok(())
        } else {
            Err(Mus2MidError)
        }
    }
}

struct Converter<'a> {
    reader: Reader<'a>,
    output: Vec<u8>,
    channel_velocities: &'a mut [u8; NUM_CHANNELS],
    channel_map: [i32; NUM_CHANNELS],
    queued_time: u32,
    track_size: u32,
}

impl<'a> Converter<'a> {
    fn new(input: &'a [u8], channel_velocities: &'a mut [u8; NUM_CHANNELS]) -> Self {
        Self {
            reader: Reader::new(input),
            output: Vec::new(),
            channel_velocities,
            channel_map: [-1; NUM_CHANNELS],
            queued_time: 0,
            track_size: 0,
        }
    }

    fn convert(mut self) -> Result<Vec<u8>, Mus2MidError> {
        let header = self.read_mus_header()?;
        self.reader.seek_set(usize::from(header.score_start))?;

        self.output.extend_from_slice(MIDI_HEADER);
        self.track_size = 0;

        let mut hit_score_end = false;

        while !hit_score_end {
            while !hit_score_end {
                let event_descriptor = self.reader.read_u8()?;
                let channel = self.get_midi_channel(i32::from(event_descriptor & 0x0F))?;
                let event = event_descriptor & 0x70;

                match event {
                    MUS_RELEASE_KEY => {
                        let key = self.reader.read_u8()?;
                        self.write_release_key(channel, key);
                    }
                    MUS_PRESS_KEY => {
                        let mut key = self.reader.read_u8()?;

                        if key & 0x80 != 0 {
                            self.channel_velocities[channel as usize] =
                                self.reader.read_u8()? & 0x7F;
                        }

                        key &= 0x7F;
                        self.write_press_key(
                            channel,
                            key,
                            self.channel_velocities[channel as usize],
                        );
                    }
                    MUS_PITCH_WHEEL => {
                        let key = self.reader.read_u8()?;
                        self.write_pitch_wheel(channel, i16::from(key) * 64);
                    }
                    MUS_SYSTEM_EVENT => {
                        let controller_number = self.reader.read_u8()?;
                        if !(10..=14).contains(&controller_number) {
                            return Err(Mus2MidError);
                        }

                        self.write_change_controller_valueless(
                            channel,
                            CONTROLLER_MAP[usize::from(controller_number)],
                        );
                    }
                    MUS_CHANGE_CONTROLLER => {
                        let controller_number = self.reader.read_u8()?;
                        let controller_value = self.reader.read_u8()?;

                        if controller_number == 0 {
                            self.write_change_patch(channel, controller_value);
                        } else {
                            if !(1..=9).contains(&controller_number) {
                                return Err(Mus2MidError);
                            }

                            self.write_change_controller_valued(
                                channel,
                                CONTROLLER_MAP[usize::from(controller_number)],
                                controller_value,
                            );
                        }
                    }
                    MUS_SCORE_END => {
                        hit_score_end = true;
                    }
                    _ => return Err(Mus2MidError),
                }

                if event_descriptor & 0x80 != 0 {
                    break;
                }
            }

            if !hit_score_end {
                let mut time_delay = 0u32;
                loop {
                    let working = self.reader.read_u8()?;
                    time_delay = time_delay
                        .wrapping_mul(128)
                        .wrapping_add(u32::from(working & 0x7F));

                    if working & 0x80 == 0 {
                        break;
                    }
                }
                self.queued_time = self.queued_time.wrapping_add(time_delay);
            }
        }

        self.write_end_track();
        self.patch_track_size();

        Ok(self.output)
    }

    fn read_mus_header(&mut self) -> Result<MusHeader, Mus2MidError> {
        for _ in 0..4 {
            self.reader.read_u8()?;
        }

        let _score_length = self.reader.read_u16_le()?;
        let score_start = self.reader.read_u16_le()?;
        let _primary_channels = self.reader.read_u16_le()?;
        let _secondary_channels = self.reader.read_u16_le()?;
        let _instrument_count = self.reader.read_u16_le()?;

        Ok(MusHeader { score_start })
    }

    fn write_time(&mut self, mut time: u32) {
        let mut buffer = time & 0x7F;

        loop {
            time >>= 7;
            if time == 0 {
                break;
            }

            buffer = buffer.wrapping_shl(8);
            buffer |= (time & 0x7F) | 0x80;
        }

        loop {
            let write_value = (buffer & 0xFF) as u8;
            self.output.push(write_value);
            self.track_size = self.track_size.wrapping_add(1);

            if buffer & 0x80 != 0 {
                buffer >>= 8;
            } else {
                self.queued_time = 0;
                break;
            }
        }
    }

    fn write_end_track(&mut self) {
        self.write_time(self.queued_time);
        self.output.extend_from_slice(&[0xFF, 0x2F, 0x00]);
        self.track_size = self.track_size.wrapping_add(3);
    }

    fn write_press_key(&mut self, channel: i32, key: u8, velocity: u8) {
        self.write_time(self.queued_time);
        self.output.push(MIDI_PRESS_KEY | channel as u8);
        self.output.push(key & 0x7F);
        self.output.push(velocity & 0x7F);
        self.track_size = self.track_size.wrapping_add(3);
    }

    fn write_release_key(&mut self, channel: i32, key: u8) {
        self.write_time(self.queued_time);
        self.output.push(MIDI_RELEASE_KEY | channel as u8);
        self.output.push(key & 0x7F);
        self.output.push(0);
        self.track_size = self.track_size.wrapping_add(3);
    }

    fn write_pitch_wheel(&mut self, channel: i32, wheel: i16) {
        self.write_time(self.queued_time);
        self.output.push(MIDI_PITCH_WHEEL | channel as u8);
        self.output.push((wheel & 0x7F) as u8);
        self.output.push(((wheel >> 7) & 0x7F) as u8);
        self.track_size = self.track_size.wrapping_add(3);
    }

    fn write_change_patch(&mut self, channel: i32, patch: u8) {
        self.write_time(self.queued_time);
        self.output.push(MIDI_CHANGE_PATCH | channel as u8);
        self.output.push(patch & 0x7F);
        self.track_size = self.track_size.wrapping_add(2);
    }

    fn write_change_controller_valued(&mut self, channel: i32, control: u8, value: u8) {
        self.write_time(self.queued_time);
        self.output.push(MIDI_CHANGE_CONTROLLER | channel as u8);
        self.output.push(control & 0x7F);
        self.output
            .push(if value & 0x80 != 0 { 0x7F } else { value });
        self.track_size = self.track_size.wrapping_add(3);
    }

    fn write_change_controller_valueless(&mut self, channel: i32, control: u8) {
        self.write_change_controller_valued(channel, control, 0);
    }

    fn allocate_midi_channel(&self) -> i32 {
        let max = self.channel_map.iter().copied().max().unwrap_or(-1);
        let mut result = max + 1;

        if result == MIDI_PERCUSSION_CHAN {
            result += 1;
        }

        result
    }

    fn get_midi_channel(&mut self, mus_channel: i32) -> Result<i32, Mus2MidError> {
        if mus_channel == MUS_PERCUSSION_CHAN {
            Ok(MIDI_PERCUSSION_CHAN)
        } else {
            let index = usize::try_from(mus_channel).map_err(|_| Mus2MidError)?;
            if index >= NUM_CHANNELS {
                return Err(Mus2MidError);
            }

            if self.channel_map[index] == -1 {
                self.channel_map[index] = self.allocate_midi_channel();
                self.write_change_controller_valueless(self.channel_map[index], 0x7B);
            }

            Ok(self.channel_map[index])
        }
    }

    fn patch_track_size(&mut self) {
        let bytes = self.track_size.to_be_bytes();
        self.output[18..22].copy_from_slice(&bytes);
    }
}

struct MusHeader {
    score_start: u16,
}

pub fn convert_mus_to_midi(input: &[u8]) -> Result<Vec<u8>, Mus2MidError> {
    let mut channel_velocities = CHANNEL_VELOCITIES.lock().map_err(|_| Mus2MidError)?;
    Converter::new(input, &mut channel_velocities).convert()
}

#[cfg(test)]
mod tests {
    use super::*;

    const SIMPLE_MUS: &[u8] = &[
        b'M', b'U', b'S', 0x1A, 0x07, 0x00, 0x0E, 0x00, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x90,
        0xBC, 0x64, 0x0A, 0x80, 0x3C, 0x05, 0x60,
    ];

    const SIMPLE_MIDI: &[u8] = &[
        b'M', b'T', b'h', b'd', 0x00, 0x00, 0x00, 0x06, 0x00, 0x00, 0x00, 0x01, 0x00, 0x46, b'M',
        b'T', b'r', b'k', 0x00, 0x00, 0x00, 0x10, 0x00, 0xB0, 0x7B, 0x00, 0x00, 0x90, 0x3C, 0x64,
        0x0A, 0x80, 0x3C, 0x00, 0x05, 0xFF, 0x2F, 0x00,
    ];

    #[test]
    fn converts_simple_mus_to_expected_midi_bytes() {
        assert_eq!(convert_mus_to_midi(SIMPLE_MUS).unwrap(), SIMPLE_MIDI);
    }

    #[test]
    fn rejects_truncated_header() {
        assert_eq!(convert_mus_to_midi(&SIMPLE_MUS[..6]), Err(Mus2MidError));
    }

    #[test]
    fn rejects_invalid_controller_number() {
        let mut input = SIMPLE_MUS.to_vec();
        input[14] = 0xC0;
        input[15] = 0x0A;

        assert_eq!(convert_mus_to_midi(&input), Err(Mus2MidError));
    }
}
