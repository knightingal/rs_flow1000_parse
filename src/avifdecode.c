#include <avif/avif.h>
#include <png.h>

#include <inttypes.h>
#include <stdio.h>
#include <string.h>
#include <stdlib.h>

typedef struct expandableBuff {
  avifRWData data;
  size_t written_len;
} expandableBuff;
expandableBuff* avifPNGWriteToMemory(const avifImage * avif, uint32_t requestedDepth, avifChromaUpsampling chromaUpsampling, int compressionLevel);

int avif_to_png(const char* avif_file_name) {
  avifBool rawColor = AVIF_FALSE;
  int pngCompressionLevel = -1; // -1 is a sentinel to avifPNGWrite() to skip calling png_set_compression_level()
  int requestedDepth = 0;
  avifChromaUpsampling chromaUpsampling = AVIF_CHROMA_UPSAMPLING_AUTOMATIC;

  avifDecoder * decoder = avifDecoderCreate();
  if (decoder == NULL) {
    fprintf(stderr, "Memory allocation failure\n");
    return 1;
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
    avifPNGWriteToMemory(imageView, requestedDepth, chromaUpsampling, pngCompressionLevel);
  }


cleanup:
  return 0;
}

