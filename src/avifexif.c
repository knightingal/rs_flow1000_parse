// Copyright 2022 Google LLC
// SPDX-License-Identifier: BSD-2-Clause

#include "avifexif.h"
#include <avif/avif.h>


avifResult avifSetExifOrientation(avifRWData * exif, uint8_t orientation)
{
    size_t offset;
    const avifResult result = avifGetExifOrientationOffset(exif->data, exif->size, &offset);
    if (result != AVIF_RESULT_OK) {
        return result;
    }
    if (offset < exif->size) {
        exif->data[offset] = orientation;
        return AVIF_RESULT_OK;
    }
    // No Exif orientation was found.
    if (orientation == 1) {
        // The default orientation is 1, so if the given orientation is 1 too, do nothing.
        return AVIF_RESULT_OK;
    }
    // Adding an orientation tag to an Exif payload is involved.
    return AVIF_RESULT_NOT_IMPLEMENTED;
}
