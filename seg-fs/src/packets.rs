pub type FileId = u8;
pub type StatusByte = u8;
pub type PacketNumber = u16;

/// Header Packet Structure
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HeaderPacket {
    pub file_id: u8,
    pub name: String
}

/// Data Packet Structure
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DataPacket {
    pub is_last: bool,
    pub file_id: u8,
    pub packet_number: PacketNumber,
    pub data: Vec<u8>
}

impl std::convert::TryFrom<Vec<u8>> for HeaderPacket {
    type Error = String;

    fn try_from(data: Vec<u8>) -> Result<Self, Self::Error> {
        if data.len() < 2 {
            return Err(format!("cannot parse header packet from data with length {}", data.len()));
        }

        if data[0] & 0b1 > 0 {
            return Err(format!("cannot parse header packet from data packet"));
        }

        let file_id = data[1];

        if data.len() == 2 {
            return Err(format!("cannot parse header packet with empty file name"))
        }

        let data = std::str::from_utf8(&data[2..])
                        .map_err(|e| format!("filename is not valid utf8: '{}'", e))?;

        Ok(HeaderPacket {
            file_id,
            name: data.to_string()
        })
    }   
}

impl std::convert::TryFrom<Vec<u8>> for DataPacket {
    type Error = String;

    fn try_from(data: Vec<u8>) -> Result<Self, Self::Error> {
        if data.len() < 2 {
            return Err(format!("cannot parse data packet from data with length {}", data.len()));
        }

        if data[0] & 0b1 == 0 {
            return Err(format!("cannot parse data packet from header packet"));
        }

        let file_id = data[1];
        let packet_number = u16::from_be_bytes([data[2], data[3]]);

        if data.len() == 4 {
            return Err(format!("cannot parse data packet with empty data"))
        }

        let file_data = data[4..].to_vec();

        Ok(DataPacket {
            is_last: data[0] & 0b10 > 0,
            file_id,
            packet_number,
            data: file_data
        })
    }   
}

#[test]
fn data_packet_decode() {
    // Test buffers which are too small
    assert!(DataPacket::try_from(vec![]).is_err());
    assert!(DataPacket::try_from(vec![1]).is_err());
    assert!(DataPacket::try_from(vec![5]).is_err());
    assert!(DataPacket::try_from(vec![6]).is_err());
    assert!(DataPacket::try_from(vec![0]).is_err());

    // Test buffers which contain header packets
    assert!(DataPacket::try_from(vec![6, 5]).is_err());
    assert!(DataPacket::try_from(vec![0, 5, b't', b'e', b's', b't']).is_err());

    // Test buffers which do not contain a file name
    assert!(DataPacket::try_from(vec![0, 5, 0, 0]).is_err());

    // Actually test some valid buffers
    assert_eq!(DataPacket::try_from(vec![3, 42, 0, 0, b'h', b'e', b'l', b'l', b'o']).unwrap(),
               DataPacket { is_last: true, file_id: 42, packet_number: 0, data: vec![b'h', b'e', b'l', b'l', b'o'] });
    assert_eq!(DataPacket::try_from(vec![65, 0xaa, 0xaa, 0x55, 0, 1, 2, 3, 4, 5]).unwrap(),
               DataPacket { is_last: false, file_id: 0xaa, packet_number: 0xaa55, data: vec![0, 1, 2, 3, 4, 5] });
}

#[test]
fn header_packet_decode() {
    // Test buffers which are too small
    assert!(HeaderPacket::try_from(vec![]).is_err());
    assert!(HeaderPacket::try_from(vec![1]).is_err());
    assert!(HeaderPacket::try_from(vec![5]).is_err());
    assert!(HeaderPacket::try_from(vec![6]).is_err());
    assert!(HeaderPacket::try_from(vec![0]).is_err());

    // Test buffers which contain data packets
    assert!(HeaderPacket::try_from(vec![65, 0xaa, 0xaa, 0x55, 0, 1, 2, 3, 4, 5]).is_err());
    assert!(HeaderPacket::try_from(vec![3, 42, 0, 0, b'h', b'e', b'l', b'l', b'o']).is_err());

    // Test buffers which do not contain a file name
    assert!(HeaderPacket::try_from(vec![0, 5]).is_err());

    // Test buffers which are not valid utf8
    assert!(HeaderPacket::try_from(vec![0, 5, 0xff]).is_err());

    // Actually test some valid buffers
    assert_eq!(HeaderPacket::try_from(vec![0, 5, b't', b'e', b's', b't']).unwrap(),
               HeaderPacket { file_id: 5, name: String::from("test") });
}