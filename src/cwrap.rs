extern crate libc;
use libc::size_t;
use std::boxed::Box;
use std::{ptr, slice, iter};
use crate::base::{EncodingPacket, ObjectTransmissionInformation};
use crate::decoder::{SourceBlockDecoder};
use crate::encoder::{SourceBlockEncoder, SourceBlockEncodingPlan};

pub struct RaptorqEncodeHandle {
  block_buf_len: size_t,
  in_symbol_count: size_t,
  config: ObjectTransmissionInformation,
  plan: SourceBlockEncodingPlan
}

pub struct RaptorqDecodeHandle {
  packet_buf_len: size_t,
  out_buf_len: u64,
  config: ObjectTransmissionInformation,
  decoders: Vec<SourceBlockDecoder>
}

/////////////////////
// init, deinit
/////////////////////

#[no_mangle]
pub unsafe extern "C" fn raptorq_initEncoder(block_buf_len: size_t, in_symbol_count: size_t) -> *mut RaptorqEncodeHandle {
  let handle = RaptorqEncodeHandle {
    block_buf_len,
    in_symbol_count,
    config: ObjectTransmissionInformation::with_defaults(block_buf_len as u64, (block_buf_len / in_symbol_count) as u16),
    plan: SourceBlockEncodingPlan::generate(in_symbol_count as u16)
  };

  Box::into_raw(Box::new(handle))
}

#[no_mangle]
pub unsafe extern "C" fn raptorq_initDecoder(packet_buf_len: size_t, out_symbol_count: size_t) -> *mut RaptorqDecodeHandle {
  let out_symbol_len = packet_buf_len - 4;
  let out_buf_len = (out_symbol_len * out_symbol_count) as u64;

  let handle = RaptorqDecodeHandle {
    packet_buf_len,
    out_buf_len,
    config: ObjectTransmissionInformation::with_defaults(out_buf_len, out_symbol_len as u16),
    decoders: (0..256).map(|_| { SourceBlockDecoder::new_blank() }).collect()
  };

  Box::into_raw(Box::new(handle))
}

#[no_mangle]
pub unsafe extern "C" fn raptorq_deinitEncoder(handle: *mut RaptorqEncodeHandle) {
  drop(Box::from_raw(handle));
}

#[no_mangle]
pub unsafe extern "C" fn raptorq_deinitDecoder(handle: *mut RaptorqDecodeHandle) {
  drop(Box::from_raw(handle));
}

/////////////////////
// public
/////////////////////

#[no_mangle]
pub unsafe extern "C" fn raptorq_encodeBlock(handle: *const RaptorqEncodeHandle, sbn: u8, block_buf: *const u8, out_buf: *mut u8, out_packet_count: size_t) -> size_t {
  let handle_ref: &RaptorqEncodeHandle = &*handle;
  let in_slice = slice::from_raw_parts(block_buf, handle_ref.block_buf_len);
  let encoder = SourceBlockEncoder::with_encoding_plan2(sbn, &handle_ref.config, in_slice, &handle_ref.plan);

  let mut encoded_packets = vec![];
  let repair_packets_count = (out_packet_count - handle_ref.in_symbol_count) as u32;
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
pub unsafe extern "C" fn raptorq_decodePacket(handle: *mut RaptorqDecodeHandle, packet_buf: *const u8, out_buf: *mut u8) -> size_t {
  let handle_ref: &mut RaptorqDecodeHandle = &mut *handle;
  let packet_slice = slice::from_raw_parts(packet_buf, handle_ref.packet_buf_len);
  let packet = EncodingPacket::deserialize(packet_slice);
  let block_number = packet.payload_id.source_block_number();

  // The erase head (block_number_ahead) is put the maximum distance away from the write head (block_number).
  let block_number_ahead = ((block_number as usize) + 128) % 256;

  handle_ref.decoders[block_number_ahead as usize].reset(block_number_ahead as u8, &handle_ref.config, handle_ref.out_buf_len);

  let result = if handle_ref.decoders[block_number as usize].decoded {
    None
  } else {
    handle_ref.decoders[block_number as usize].decode(iter::once(packet))
  };

  match result {
    Some(out_vec) => {
      ptr::copy(out_vec.as_ptr(), out_buf, out_vec.len());
      out_vec.len()
    },
    None => 0
  }
}
