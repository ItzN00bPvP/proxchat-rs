use glam::{DVec3, IVec3};
use lazy_static::lazy_static;
use std::collections::HashMap;

lazy_static! {
    static ref ALL_OFFSETS: Vec<IVec3> = ProxChatGenerator::create_all_offsets();
    static ref ALL_OFFSETS_LOOKUP_MAP: HashMap<IVec3, isize> = ProxChatGenerator::create_offset_lookup_map();
    pub static ref PACKET_PDU_MAGIC: [usize; 2] = [
        ProxChatGenerator::get_max_prox_data_unit() - 1,
        ProxChatGenerator::get_max_prox_data_unit() - 19,
    ];
}

trait IVec3centervec3d {
    fn center_vec3d(&self) -> DVec3;
}
impl IVec3centervec3d for IVec3 {
    fn center_vec3d(&self) -> DVec3 {
        DVec3::new(self.x as f64 + 0.5, self.y as f64 + 0.5, self.z as f64 + 0.5)
    }
}
pub struct ProxChatGenerator;
impl ProxChatGenerator {
    fn create_all_offsets() -> Vec<IVec3> {
        // All tested offsets have to be in reach of all these positions
        let mut origin_positions: Vec<DVec3> = vec![];
        for x_offset in &[0, 1] {
            for y_offset in &[0, 1] {
                for z_offset in &[0, 1] {
                    origin_positions.push(DVec3::new(*x_offset as f64, *y_offset as f64, *z_offset as f64));
                }
            }
        }

        // Find all offsets that can be interacted with
        let mut offsets: Vec<IVec3> = vec![];
        let any_axis_offsets = [0, 1, -1, 2, -2, 3, -3, 4, -4, 5, -5, 6, -6];
        for &x in &any_axis_offsets {
            for &y in &any_axis_offsets {
                'offsetloop:
                for &z in &any_axis_offsets {
                    let offset = IVec3::new(x, y, z);
                    let offset_center = offset.center_vec3d();
                    for origin in &origin_positions {
                        if origin.distance_squared(offset_center) > 6.0 * 6.0 {
                            continue 'offsetloop; // Too far to interact with
                        }
                    }
                    offsets.push(offset);
                }
            }
        }
        offsets
    }


    fn create_offset_lookup_map() -> HashMap<IVec3, isize> {
        let mut lookup_map = HashMap::new();
        for (index, offset) in ALL_OFFSETS.iter().enumerate() {
            lookup_map.insert(offset.clone(), index as isize);
        }
        lookup_map
    }
    pub fn get_storable_bit_count() -> i32 {
        if ALL_OFFSETS.len() == 0 { return 0; }
        (ALL_OFFSETS.len() as f64).log2().floor() as i32
    }

    pub fn get_max_prox_data_unit() -> usize {
        ALL_OFFSETS.len()
    }
}


pub const PACKET_ID_CHAT: u16 = 1;
pub const PACKET_ID_PATPAT_PATENTITY: u16 = 2;
pub const PACKET_ID_EMOTECRAFT: u16 = 3;

pub struct ProxChatEncoder;
impl ProxChatEncoder {

    /// returns a list of block offsets
    pub fn encode_prox_chat_packet(message: String) -> Vec<IVec3> {
        let id = PACKET_ID_CHAT;
        let mut data = vec![00, message.len() as u8];
        data.append(&mut message.as_bytes().to_vec());
        let proxdata =  Self::encode_prox_packet(id, &*data);

        let mut proxdataunits = PACKET_PDU_MAGIC.iter().map(|&pdu| pdu as u32).collect::<Vec<u32>>();
        proxdataunits.append(&mut Self::bytes_to_prox_data_units(&proxdata));
        Self::data_units_to_offsets(proxdataunits)
    }


    pub fn encode_prox_packet(id: u16, data: &[u8]) -> Vec<u8> {
        let length = 2 + data.len() as i32; // Id is 2 bytes
        let mut bout = Vec::with_capacity((3 + length) as usize);

        // Write length
        bout.push(((length >> 16) & 0xFF) as u8);
        bout.push(((length >> 8) & 0xFF) as u8);
        bout.push((length & 0xFF) as u8);

        // Write id
        bout.push(((id >> 8) & 0xFF) as u8);
        bout.push((id & 0xFF) as u8);

        // Write data
        bout.extend_from_slice(data);

        bout
    }

    pub fn bytes_to_prox_data_units(input: &[u8]) -> Vec<u32> {
        if input.is_empty() { return vec![]; }

        let mut output = vec![];
        let highest_bit: i32 = ProxChatGenerator::get_storable_bit_count();
        let mut current_offset_index: i32 = 0;
        let mut current_offset_index_pos: i32 = 0;
        for &input_byte in input {
            for i in 0..8 {
                let input_bit: u32 = (input_byte as u32 >> i as u32) & 1;

                current_offset_index |= (input_bit << current_offset_index_pos) as i32;
                current_offset_index_pos += 1;

                if current_offset_index_pos == highest_bit {
                    output.push(current_offset_index as u32);
                    current_offset_index = 0;
                    current_offset_index_pos = 0;
                }
            }
        }

        if current_offset_index_pos > 0 {
            output.push(current_offset_index as u32);
        }

        output
    }

    pub fn data_units_to_offsets(data_units: Vec<u32>) -> Vec<IVec3> {
        let mut offsets = vec![];
        for &data_unit in data_units.iter() {
            offsets.push(ALL_OFFSETS[data_unit as usize]);
        }
        offsets
    }
}