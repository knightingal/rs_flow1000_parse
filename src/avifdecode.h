#ifndef LIBAVIF_DECODE_H
#define LIBAVIF_DECODE_H

#include "avifpng.h"

expandableBuff* avif_to_png(const char* avif_file_name);
#endif