#ifndef _RAPTORQ_H
#define _RAPTORQ_H

#ifdef __cplusplus
extern "C" {
#endif

#include <stdint.h>
#include <stdlib.h>

// NOTES
// - blockBufLen / inSymbolCount must be >= 64
// - raptorq_initEncoder returns an opaque pointer handle, pass it back to raptorq_encodeBlock and raptorq_deinitEncoder
// - handle must not be used between threads; each thread should call raptorq_initEncoder / raptorq_deinitEncoder
// - likewise for raptorq_initDecoder, raptorq_decodePacket and raptorq_deinitDecoder
// - symbolLen is packetBufLen - 4
// - for raptorq_decodePacket, allocate for outBuf (symbolLen * outSymbolCount) bytes
// - symbolLen must be: 64, 128, 256, 512 or 1024
void *raptorq_initEncoder (size_t blockBufLen, size_t inSymbolCount);
void *raptorq_initDecoder (size_t packetBufLen, size_t outSymbolCount);
void raptorq_deinitEncoder (void *handle);
void raptorq_deinitDecoder (void *handle);
size_t raptorq_encodeBlock (const void *handle, uint8_t sbn, const uint8_t *blockBuf, uint8_t *outBuf, size_t outPacketCount);
size_t raptorq_decodePacket (void *handle, const uint8_t *packetBuf, uint8_t *outBuf);

#ifdef __cplusplus
}
#endif

#endif
