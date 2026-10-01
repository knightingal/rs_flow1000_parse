// Copyright 2022 Google LLC
// SPDX-License-Identifier: BSD-2-Clause

#ifndef LIBAVIF_APPS_SHARED_AVIFEXIF_H
#define LIBAVIF_APPS_SHARED_AVIFEXIF_H

#include <avif/avif.h>

#ifdef __cplusplus
extern "C" {
#endif


// Attempts to parse the Exif payload until the orientation is found, then sets it to the given value.
avifResult avifSetExifOrientation(avifRWData * exif, uint8_t orientation);

#ifdef __cplusplus
} // extern "C"
#endif

#endif // ifndef LIBAVIF_APPS_SHARED_AVIFEXIF_H
