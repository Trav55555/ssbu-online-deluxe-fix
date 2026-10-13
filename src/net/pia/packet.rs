#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct PiaCustomNetPacket {
    pub latency_bits: u8,
    pub render_profile_settings_bits: u16,
}

impl PiaCustomNetPacket {
    pub const LEN: usize = 4;

    pub fn decode(data: &[u8]) -> Option<Self> {
        let data = data.get(..Self::LEN)?;
        let bits = u16::from_le_bytes([data[2], data[3]]);
        let render_profile_settings_bits = match data[0] {
            2 => bits,
            // v3 uses bit 14 for FPS boost, not the v2 overclock profile.
            3 => bits & 0x3fff,
            _ => return None,
        };
        Some(Self {
            latency_bits: data[1],
            render_profile_settings_bits,
        })
    }

    pub fn encode(self) -> [u8; Self::LEN] {
        let bits = self.render_profile_settings_bits.to_le_bytes();
        [2, self.latency_bits, bits[0], bits[1]]
    }
}

#[cfg(test)]
mod tests {
    use super::PiaCustomNetPacket;

    #[test]
    fn legacy_wire_fixture_preserves_overclock_bits() {
        let packet = PiaCustomNetPacket::decode(&[2, 25, 0x06, 0x68]).unwrap();
        assert_eq!(packet.latency_bits, 25);
        assert_eq!(packet.render_profile_settings_bits, 0x6806);
    }

    #[test]
    fn upstream_v3_fixture_discards_boost_and_reserved_bits() {
        let packet = PiaCustomNetPacket::decode(&[3, 5, 0x06, 0xe8]).unwrap();
        assert_eq!(packet.latency_bits, 5);
        assert_eq!(packet.render_profile_settings_bits, 0x2806);
    }

    #[test]
    fn emitted_packet_remains_compatible_with_v2_readers() {
        let packet = PiaCustomNetPacket {
            latency_bits: 0xff,
            render_profile_settings_bits: 0x900b,
        };
        assert_eq!(packet.encode(), [2, 0xff, 0x0b, 0x90]);
    }

    #[test]
    fn truncated_and_unknown_protocol_packets_are_rejected() {
        for len in 0..4 {
            assert!(PiaCustomNetPacket::decode(&[2, 5, 0x0b, 0x10][..len]).is_none());
        }
        for version in [0, 1, 4, 255] {
            assert!(PiaCustomNetPacket::decode(&[version, 5, 0x0b, 0x10]).is_none());
        }
    }

    #[test]
    fn callback_padding_does_not_change_the_packet() {
        let packet = PiaCustomNetPacket::decode(&[2, 7, 0x0b, 0x10, 0xaa]).unwrap();
        assert_eq!(packet.latency_bits, 7);
        assert_eq!(packet.render_profile_settings_bits, 0x100b);
    }
}
