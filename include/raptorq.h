#ifndef _RAPTORQ_H
#define _RAPTORQ_H

#include <stdint.h>
#include <stdlib.h>

// NOTE: blockBufLen / inSymbolCount must be >= 64
// symbolLen must be: 64, 128, 256, 512 or 1024
size_t raptorq_encodeBlock (uint8_t sbn, const uint8_t *blockBuf, size_t blockBufLen, size_t inSymbolCount, uint8_t *outBuf, size_t outPacketCount);
size_t raptorq_decodePacket (const uint8_t *packetBuf, size_t packetBufLen, uint8_t *outBuf, size_t outSymbolCount);

#endif
