// Copyright 2019 Joe Drago. All rights reserved.
// SPDX-License-Identifier: BSD-2-Clause

#ifndef LIBAVIF_APPS_SHARED_AVIFPNG_H
#define LIBAVIF_APPS_SHARED_AVIFPNG_H

#include "avif/avif.h"

#ifdef __cplusplus
extern "C" {
#endif


typedef struct expandableBuff {
  avifRWData data;
  size_t written_len;
} expandableBuff;

expandableBuff* avifPNGWriteToMemory(
  const avifImage * avif, 
  uint32_t requestedDepth, 
  avifChromaUpsampling chromaUpsampling, 
  int compressionLevel
);

#ifdef __cplusplus
} // extern "C"
#endif

#endif // ifndef LIBAVIF_APPS_SHARED_AVIFPNG_H
