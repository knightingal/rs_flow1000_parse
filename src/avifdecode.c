#include <avif/avif.h>
#include <png.h>
#include "avifdecode.h"

#include <inttypes.h>
#include <stdio.h>
#include <string.h>
#include <stdlib.h>

/*
gcc -shared -fPIC avifpng.c  avifutil.c avifexif.c avifdecode.c -o libavifdecode.so -lavif -lpng
 */

expandableBuff* avif_to_png(const char* avif_file_name) {
  avifBool rawColor = AVIF_FALSE;
  int pngCompressionLevel = -1; // -1 is a sentinel to avifPNGWrite() to skip calling png_set_compression_level()
  int requestedDepth = 0;
  expandableBuff *p_buff = NULL;
  avifChromaUpsampling chromaUpsampling = AVIF_CHROMA_UPSAMPLING_AUTOMATIC;

  avifDecoder * decoder = avifDecoderCreate();
  if (decoder == NULL) {
    fprintf(stderr, "Memory allocation failure\n");
    return p_buff;
  }

  avifResult result = avifDecoderSetIOFile(decoder, avif_file_name);
  if (result != AVIF_RESULT_OK) {
    fprintf(stderr, "Cannot open file for read: %s\n", avif_file_name);
    goto cleanup;
  }
  result = avifDecoderParse(decoder);
  if (result != AVIF_RESULT_OK) {
    fprintf(stderr, "Failed to decode image: %s\n", avifResultToString(result));
    goto cleanup;
  }

  printf("Parsed AVIF: %ux%u (%ubpc)\n", decoder->image->width, decoder->image->height, decoder->image->depth);
  if (avifDecoderNthImage(decoder, 0) == AVIF_RESULT_OK) {
    avifImage * imageView = decoder->image;
    p_buff = avifPNGWriteToMemory(imageView, requestedDepth, chromaUpsampling, pngCompressionLevel);
  }


cleanup:
  avifDecoderDestroy(decoder);
  return p_buff;
}

