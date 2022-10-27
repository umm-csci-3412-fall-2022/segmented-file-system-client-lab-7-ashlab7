use super::packets::*;
use std::collections::HashMap;

pub struct File {
    file_id: FileId,
    name: Option<String>,
    segments: HashMap<PacketNumber, Vec<u8>>,
    max_segments: Option<PacketNumber>
}

impl File {
    pub fn new(file_id: FileId) -> Self {
        Self {
            file_id,
            name: None,
            segments: HashMap::new(),
            max_segments: None
        }
    }

    pub fn report_header_packet(&mut self, data: HeaderPacket) {
        self.name = Some(data.name);
    }

    pub fn report_data_packet(&mut self, data: DataPacket) {
        self.segments.insert(data.packet_number, data.data);
        if data.is_last {
            self.max_segments = Some(data.packet_number);
        }
    }

    pub fn is_done(&self) -> bool {
        if let Some(max_segments) = self.max_segments {
            self.name.is_some() && self.segments.len() == max_segments as usize + 1
        }
        else {
            false
        }
    }
}

impl std::fmt::Display for File {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "FileId {:02x}", self.file_id)?;

        if let Some(name) = &self.name {
            write!(f, " {}", name)?;
        }

        if let Some(max_segments) = self.max_segments {
            write!(f, " {} / {} segments", self.segments.len(), max_segments + 1)?;
        }
        else {
            write!(f, " {} segments", self.segments.len())?;    
        }

        Ok(())
    }   
}

pub struct Client {
    udp_socket: std::net::UdpSocket,
    in_progress_files: HashMap<FileId, File>,
    final_files: Vec<File>
}

impl Client {
    pub fn new(udp_socket: std::net::UdpSocket) -> Self {
        Self {
            udp_socket,
            in_progress_files: HashMap::new(),
            final_files: Vec::new(),
        }
    }

    pub fn send_request(&mut self) -> Result<(), String> {
        self.udp_socket.send(&[0]).map_err(|e| format!("unable to send request over socket {}", e))?;
        Ok(())
    }

    fn read_data(&mut self) -> Result<Vec<u8>, String> {
        let mut buf = [0; 1024 + 4];
        match self.udp_socket.recv_from(&mut buf) {
            Ok((size, _)) => Ok(buf[..size].to_vec()),
            Err(e) => Err(format!("unable to recieve data over socket {}", e))
        }
    }

    fn get_mut_file_id(&mut self, file_id: FileId) -> &mut File {
        if !self.in_progress_files.contains_key(&file_id) {
            self.in_progress_files.insert(file_id, File::new(file_id));
        }

        if let Some(file) = self.in_progress_files.get_mut(&file_id) {
            file
        }
        else {
            unreachable!()
        }
    }

    fn move_complete_files(&mut self) -> Result<(), String> {
        let mut transition_files = Vec::new();

        for file in self.in_progress_files.values_mut() {
            if file.is_done() {
                transition_files.push(file.file_id);
            }
        }

        for id in transition_files {
            if let Some(file) = self.in_progress_files.remove(&id) {
                self.final_files.push(file);
            }
        }

        Ok(())
    }

    pub fn recv_packet(&mut self) -> Result<(), String> {
        let data = self.read_data()?;

        if data.len() == 0 {
            return Err(format!("data packet has zero length"));
        }

        if data[0] & 1 > 0 {
            let packet = DataPacket::try_from(data)?;
            self.get_mut_file_id(packet.file_id).report_data_packet(packet);
        }
        else {
            let packet = HeaderPacket::try_from(data)?;
            self.get_mut_file_id(packet.file_id).report_header_packet(packet);
        }

        self.move_complete_files()?;

        Ok(())
    }

    pub fn file_count(&self) -> usize {
        self.final_files.len()
    }

    pub fn finalize_files(self) -> Result<(), String> {
        use std::io::prelude::*;

        for file in self.final_files {
            if let Some(filename) = file.name {
                let mut file_io = std::fs::File::create(&filename).map_err(|e| format!("unable to create file {}: {}", &filename, e))?;
            
                if let Some(last_packet) = file.max_segments {
                    for id in 0..=last_packet {
                        if let Some(data) = file.segments.get(&id) {
                            file_io.write_all(data).map_err(|e| format!("unable to write to file {}", e))?;
                        }
                        else {
                            return Err(format!("unable to write file {}, bad data at packet id {}", &filename, id));
                        }
                    }
                }
            }
            else {
                return Err(format!("unable to write file id {}, no name", file.file_id));
            }
        }

        Ok(())
    }

    pub fn print_line_length(&self) -> usize {
        2 + self.in_progress_files.len() + self.final_files.len()
    }
}

impl std::fmt::Display for Client {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "In Progress:")?;
        for file in self.in_progress_files.values() {
            writeln!(f, "  {}", file)?;
        }
        writeln!(f, "Done:")?;
        for file in self.final_files.iter() {
            if let Some(name) = &file.name {
                writeln!(f, "  {}", name)?;
            }
            else {
                writeln!(f, "  <BAD FILE>")?;
            }
        }
        Ok(())
    }   
}