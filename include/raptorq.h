#ifndef _RAPTORQ_H
#define _RAPTORQ_H

#include <stdint.h>
#include <stdlib.h>

// NOTES
// - blockBufLen / inSymbolCount must be >= 64
// - call raptorq_initEncoder once before calling raptorq_encodeBlock
// - call raptorq_initEncoder and raptorq_encodeBlock from the same thread
// - raptorq_encodeBlock will panic if raptorq_initEncoder was not called first
// - packetBufLen must be constant for every call
// - symbolLen must be: 64, 128, 256, 512 or 1024
void raptorq_initEncoder (size_t blockBufLen, size_t inSymbolCount);
size_t raptorq_encodeBlock (uint8_t sbn, const uint8_t *blockBuf, uint8_t *outBuf, size_t outPacketCount);
size_t raptorq_decodePacket (const uint8_t *packetBuf, size_t packetBufLen, uint8_t *outBuf, size_t outSymbolCount);

#endif
