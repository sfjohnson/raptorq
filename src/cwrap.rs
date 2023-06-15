extern crate libc;
use libc::size_t;
use std::sync::Mutex;
use std::{ptr, slice, iter};
use crate::base::{EncodingPacket, ObjectTransmissionInformation};
use crate::decoder::{SourceBlockDecoder};
use crate::encoder::{SourceBlockEncoder, SourceBlockEncodingPlan};

static mut BLOCK_BUF_LEN: Option<size_t> = None;
static mut IN_SYMBOL_COUNT: Option<size_t> = None;
static mut CONFIG: Option<ObjectTransmissionInformation> = None;
static mut PLAN: Option<SourceBlockEncodingPlan> = None;

#[no_mangle]
pub unsafe extern "C" fn raptorq_initEncoder(block_buf_len: size_t, in_symbol_count: size_t) {
  BLOCK_BUF_LEN = Some(block_buf_len);
  IN_SYMBOL_COUNT = Some(in_symbol_count);
  CONFIG = Some(ObjectTransmissionInformation::with_defaults(block_buf_len as u64, (block_buf_len / in_symbol_count) as u16));
  PLAN = Some(SourceBlockEncodingPlan::generate(in_symbol_count as u16));
}

#[no_mangle]
pub unsafe extern "C" fn raptorq_encodeBlock(sbn: u8, block_buf: *const u8, out_buf: *mut u8, out_packet_count: size_t) -> size_t {
  let in_slice = slice::from_raw_parts(block_buf, BLOCK_BUF_LEN.unwrap());
  let encoder = SourceBlockEncoder::with_encoding_plan2(sbn, CONFIG.as_mut().unwrap(), in_slice, PLAN.as_mut().unwrap());

  let mut encoded_packets = vec![];
  let repair_packets_count = (out_packet_count - IN_SYMBOL_COUNT.unwrap()) as u32;
  encoded_packets.extend(encoder.source_packets());
  encoded_packets.extend(encoder.repair_packets(0, repair_packets_count));
  let out_vec: Vec<u8> = encoded_packets
    .iter()
    .flat_map(|packet| packet.serialize())
    .collect();

  ptr::copy(out_vec.as_ptr(), out_buf, out_vec.len());
  out_vec.len()
}

#[no_mangle]
pub unsafe extern "C" fn raptorq_decodePacket(packet_buf: *const u8, packet_buf_len: size_t, out_buf: *mut u8, out_symbol_count: size_t) -> size_t {
  lazy_static! {
    static ref DECODERS_MUTEX: Mutex<Vec<SourceBlockDecoder>> = Mutex::new((0..256).map(|_| {
      SourceBlockDecoder::new_blank()
    }).collect());
  }

  let packet_slice = slice::from_raw_parts(packet_buf, packet_buf_len);
  let packet = EncodingPacket::deserialize(packet_slice);
  let block_number = packet.payload_id.source_block_number();
  let mut decoders = DECODERS_MUTEX.lock().unwrap();

  // The erase head (block_number_ahead) is put the maximum distance away from the write head (block_number).
  let block_number_ahead = ((block_number as usize) + 128) % 256;

  // NOTE: breaks unless packet_buf_len is constant for every packet
  let out_symbol_len = packet_buf_len - 4;
  let out_buf_len = (out_symbol_len * out_symbol_count) as u64;
  let config = ObjectTransmissionInformation::with_defaults(out_buf_len, out_symbol_len as u16);
  decoders[block_number_ahead as usize].reset(block_number_ahead as u8, &config, out_buf_len);

  let result = if decoders[block_number as usize].decoded {
    None
  } else {
    decoders[block_number as usize].decode(iter::once(packet))
  };

  match result {
    Some(out_vec) => {
      ptr::copy(out_vec.as_ptr(), out_buf, out_vec.len());
      out_vec.len()
    },
    None => 0
  }
}
