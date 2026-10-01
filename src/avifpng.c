#include "avifpng.h"
#include "avifutil.h"
#include "avifexif.h"

#include "png.h"

#include <ctype.h>
#include <limits.h>
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

void 
png_memory_flush(png_structp png_ptr)
{
  // DO nothing
}


expandableBuff* init_expandableBuff(void) {
  expandableBuff* pBuff = (expandableBuff *)avifAlloc(sizeof(expandableBuff));
  
  pBuff->written_len = 0;
  pBuff->data.data = (uint8_t *)avifAlloc(1024);
  pBuff->data.size = 1024;

  return pBuff;
}

void free_expandableBuff(expandableBuff* pBuff) {
  if (pBuff == NULL) {
    return;
  }

  avifFree(pBuff->data.data);
  avifFree(pBuff);
}


void 
png_memory_write_data(png_structp png_ptr, png_bytep data, size_t length)
{
  printf("write data %d\n", length);
  expandableBuff* io_ptr = png_get_io_ptr(png_ptr);
  size_t pre_write_len = io_ptr->written_len + length;
  if (pre_write_len > io_ptr->data.size) {
    size_t target_size = io_ptr->data.size;
    do {
      target_size *= 2;
    } while (target_size <= pre_write_len);
    if (avifRWDataRealloc(&io_ptr->data, target_size)!= AVIF_RESULT_OK) {
      fprintf(stderr, "Error : out of memory\n");
      return;
    }
  }

  memcpy(io_ptr->data.data + io_ptr->written_len, data, length);
  io_ptr->written_len = pre_write_len;
}




expandableBuff* avifPNGWriteToMemory(const avifImage * avif, uint32_t requestedDepth, avifChromaUpsampling chromaUpsampling, int compressionLevel)
{
    printf("avifPNGWriteToMemory\n");
    volatile avifBool writeResult = AVIF_FALSE;
    png_structp png = NULL;
    png_infop info = NULL;
    avifRWData xmp = { NULL, 0 };
    png_bytep * volatile rowPointers = NULL;
    expandableBuff *p_buff = NULL;

    avifRGBImage rgbData;
    memset(&rgbData, 0, sizeof(avifRGBImage));

    volatile int rgbDepth = requestedDepth;
    if (rgbDepth == 0) {
        rgbDepth = (avif->depth > 8) ? 16 : 8;
    }
    if (avif->matrixCoefficients == AVIF_MATRIX_COEFFICIENTS_YCGCO_RO) {
        fprintf(stderr, "AVIF_MATRIX_COEFFICIENTS_YCGCO_RO cannot be used with PNG because it has an even bit depth.\n");
        goto cleanup;
    }
    if (avif->matrixCoefficients == AVIF_MATRIX_COEFFICIENTS_YCGCO_RE) {
        if (avif->depth != 10) {
            fprintf(stderr, "avif->depth must be 10 bits and not %u.\n", avif->depth);
            goto cleanup;
        }
        if (requestedDepth && requestedDepth != 8) {
            fprintf(stderr, "Cannot request %u bits for YCgCo-Re as it only works for 8 bits.\n", requestedDepth);
            goto cleanup;
        }

        rgbDepth = 8;
    }

    volatile avifBool hasTransforms = avif->transformFlags & (AVIF_TRANSFORM_CLAP | AVIF_TRANSFORM_IROT | AVIF_TRANSFORM_IMIR);
    volatile avifBool copyYPlane = (avif->yuvFormat == AVIF_PIXEL_FORMAT_YUV400) && !avif->alphaPlane && (avif->depth == 8) &&
                                   (rgbDepth == 8) && !hasTransforms;

    volatile int colorType;
    if (copyYPlane) {
        colorType = PNG_COLOR_TYPE_GRAY;
    } else {
        avifRGBImageSetDefaults(&rgbData, avif);
        rgbData.depth = rgbDepth;
        if (avif->yuvFormat == AVIF_PIXEL_FORMAT_YUV400 && avif->alphaPlane) {
            colorType = PNG_COLOR_TYPE_GRAY_ALPHA;
            rgbData.format = AVIF_RGB_FORMAT_GRAYA;
        } else if (avif->yuvFormat == AVIF_PIXEL_FORMAT_YUV400 && !avif->alphaPlane) {
            colorType = PNG_COLOR_TYPE_GRAY;
            rgbData.format = AVIF_RGB_FORMAT_GRAY;
        } else {
            rgbData.chromaUpsampling = chromaUpsampling;
            colorType = PNG_COLOR_TYPE_RGBA;
            if (avifImageIsOpaque(avif)) {
                colorType = PNG_COLOR_TYPE_RGB;
                rgbData.format = AVIF_RGB_FORMAT_RGB;
            }
        }
        if (avifRGBImageAllocatePixels(&rgbData) != AVIF_RESULT_OK) {
            fprintf(stderr, "Conversion to RGB failed: (out of memory)\n");
            goto cleanup;
        }
        if (avifImageYUVToRGB(avif, &rgbData) != AVIF_RESULT_OK) {
            fprintf(stderr, "Conversion to RGB failed\n");
            goto cleanup;
        }
    }

    // rgbView is a view on rgbData. avifApplyTransforms() may modify rgbData.
    avifRGBImage rgbView = rgbData;
    avifResult transformResult = avifApplyTransforms(&rgbView, &rgbData, avif);
    if (transformResult != AVIF_RESULT_OK) {
        if (transformResult == AVIF_RESULT_INVALID_ARGUMENT) {
            fprintf(stderr, "Warning, ignoring invalid transforms (clap/irot/imir)\n");
        } else {
            fprintf(stderr, "Failed to apply transforms: %s\n", avifResultToString(transformResult));
            goto cleanup;
        }
    }


    png = png_create_write_struct(PNG_LIBPNG_VER_STRING, NULL, NULL, NULL);
    if (!png) {
        fprintf(stderr, "Cannot init libpng (png)\n");
        goto cleanup;
    }

    png_set_benign_errors(png, 1); // Treat benign errors as warnings.

    info = png_create_info_struct(png);
    if (!info) {
        fprintf(stderr, "Cannot init libpng (info)\n");
        goto cleanup;
    }

    if (setjmp(png_jmpbuf(png))) {
        fprintf(stderr, "Error writing PNG\n");
        goto cleanup;
    }

    // png_init_io(png, f);
    // TODO: implement function write_data_fn 
    p_buff = init_expandableBuff();
    png_set_write_fn(png, p_buff, png_memory_write_data, png_memory_flush);

    // Don't bother complaining about ICC profile's contents when transferring from AVIF to PNG.
    // It is up to the enduser to decide if they want to keep their ICC profiles or not.
#if defined(PNG_SKIP_sRGB_CHECK_PROFILE) && defined(PNG_SET_OPTION_SUPPORTED) // See libpng-manual.txt, section XII.
    png_set_option(png, PNG_SKIP_sRGB_CHECK_PROFILE, PNG_OPTION_ON);
#endif

    if (compressionLevel >= 0) {
        png_set_compression_level(png, compressionLevel);
    }

    volatile uint32_t width = copyYPlane ? avif->width : rgbView.width;
    volatile uint32_t height = copyYPlane ? avif->height : rgbView.height;

    png_set_IHDR(png, info, width, height, rgbDepth, colorType, PNG_INTERLACE_NONE, PNG_COMPRESSION_TYPE_DEFAULT, PNG_FILTER_TYPE_DEFAULT);

    const avifBool hasIcc = avif->icc.data && (avif->icc.size > 0);
    if (hasIcc) {
        // If there is an ICC profile, the CICP values are irrelevant and only the ICC profile
        // is written. If we could extract the primaries/transfer curve from the ICC profile,
        // then they could be written in cHRM/gAMA chunks.
        png_set_iCCP(png, info, "libavif", 0, avif->icc.data, (png_uint_32)avif->icc.size);
    } else {
        const avifBool isSrgb = (avif->colorPrimaries == AVIF_COLOR_PRIMARIES_SRGB) &&
                                (avif->transferCharacteristics == AVIF_TRANSFER_CHARACTERISTICS_SRGB);
        if (isSrgb) {
            png_set_sRGB_gAMA_and_cHRM(png, info, PNG_sRGB_INTENT_PERCEPTUAL);
        } else {
            if (avif->colorPrimaries != AVIF_COLOR_PRIMARIES_UNKNOWN && avif->colorPrimaries != AVIF_COLOR_PRIMARIES_UNSPECIFIED) {
                float primariesCoords[8];
                avifColorPrimariesGetValues(avif->colorPrimaries, primariesCoords);
                png_set_cHRM(png,
                             info,
                             primariesCoords[6],
                             primariesCoords[7],
                             primariesCoords[0],
                             primariesCoords[1],
                             primariesCoords[2],
                             primariesCoords[3],
                             primariesCoords[4],
                             primariesCoords[5]);
            }
            float gamma;
            // Write the transfer characteristics IF it can be represented as a
            // simple gamma value. Most transfer characteristics cannot be
            // represented this way. Viewers that support the cICP chunk can use
            // that instead, but older viewers might show incorrect colors.
            if (avifTransferCharacteristicsGetGamma(avif->transferCharacteristics, &gamma) == AVIF_RESULT_OK) {
                png_set_gAMA(png, info, 1.0f / gamma);
            }
        }
    }

    png_text texts[2];
    int numTextMetadataChunks = 0;
    if (avif->exif.data && (avif->exif.size > 0)) {
        if (avif->exif.size > UINT32_MAX) {
            fprintf(stderr, "Error writing PNG: Exif metadata is too big\n");
            goto cleanup;
        }
        // Make a copy of the Exif data to avoid modifying the input image when setting orientation.
        avifRWData exif = { NULL, 0 };
        if (avifRWDataRealloc(&exif, avif->exif.size) != AVIF_RESULT_OK) {
            fprintf(stderr, "Error writing PNG metadata: out of memory\n");
            goto cleanup;
        }
        memcpy(exif.data, avif->exif.data, avif->exif.size);
        // We already rotated the pixels if necessary in avifApplyTransforms(), so we set the orientation to 1 (no rotation, no mirror).
        avifResult result = avifSetExifOrientation(&exif, 1);
        if (result != AVIF_RESULT_OK) {
            if (result == AVIF_RESULT_INVALID_EXIF_PAYLOAD || result == AVIF_RESULT_NOT_IMPLEMENTED) {
                // Either the Exif is invalid, or it doesn't have an orientation field.
                // If it's invalid, we can consider it as equivalent to not having an orientation.
                // In both cases, we can ignore the error.
            } else {
                fprintf(stderr, "Error writing PNG metadata: %s\n", avifResultToString(result));
                avifRWDataFree(&exif);
                goto cleanup;
            }
        }
        png_set_eXIf_1(png, info, (png_uint_32)exif.size, exif.data);
        avifRWDataFree(&exif);
    }
    if (avif->xmp.data && (avif->xmp.size > 0)) {
        // The iTXt XMP payload may not contain a zero byte according to section 4.2.3.3 of
        // the PNG specification, version 1.2.
        // The chunk is given to libpng as is. Bytes after a zero byte may be stripped.

        // Providing the length through png_text.itxt_length does not work.
        // The given png_text.text string must end with a zero byte.
        if (avif->xmp.size >= SIZE_MAX) {
            fprintf(stderr, "Error writing PNG: XMP metadata is too big\n");
            goto cleanup;
        }
        if (avifRWDataRealloc(&xmp, avif->xmp.size + 1) != AVIF_RESULT_OK) {
            fprintf(stderr, "Error writing PNG: out of memory\n");
            goto cleanup;
        }
        memcpy(xmp.data, avif->xmp.data, avif->xmp.size);
        xmp.data[avif->xmp.size] = '\0';
        png_text * text = &texts[numTextMetadataChunks++];
        memset(text, 0, sizeof(*text));
        text->compression = PNG_ITXT_COMPRESSION_NONE;
        text->key = "XML:com.adobe.xmp";
        text->text = (char *)xmp.data;
        text->itxt_length = xmp.size;
    }
    if (numTextMetadataChunks != 0) {
        png_set_text(png, info, texts, numTextMetadataChunks);
    }

    png_write_info(png, info);

    // Custom chunk writing, must appear after png_write_info.
    // With AVIF, an ICC profile takes priority over CICP, but with PNG files, CICP takes priority over ICC.
    // Therefore CICP should only be written if there is no ICC profile.
    if (!hasIcc) {
        const png_byte cicp[5] = "cICP";
        const png_byte cicpData[4] = { (png_byte)avif->colorPrimaries,
                                       (png_byte)avif->transferCharacteristics,
                                       AVIF_MATRIX_COEFFICIENTS_IDENTITY,
                                       1 /*full range*/ };
        png_write_chunk(png, cicp, cicpData, 4);
    }

    rowPointers = (png_bytep *)malloc(sizeof(png_bytep) * height);
    if (rowPointers == NULL) {
        fprintf(stderr, "Error writing PNG: memory allocation failure");
        goto cleanup;
    }
    uint8_t * row;
    uint32_t rowBytes;
    if (copyYPlane) {
        row = avif->yuvPlanes[AVIF_CHAN_Y];
        rowBytes = avif->yuvRowBytes[AVIF_CHAN_Y];
    } else {
        row = rgbView.pixels;
        rowBytes = rgbView.rowBytes;
    }
    for (uint32_t y = 0; y < height; ++y) {
        rowPointers[y] = row;
        row += rowBytes;
    }

    if (rgbDepth > 8) {
        png_set_swap(png);
    }

    png_write_image(png, rowPointers);
    png_write_end(png, NULL);

    writeResult = AVIF_TRUE;
    FILE* f = fopen("output.png", "wb");
    fwrite(p_buff->data.data, 1, p_buff->written_len, f);
    fflush(f);
    fclose(f);
    printf("Wrote PNG\n");
cleanup:
    if (png) {
        png_destroy_write_struct(&png, &info);
    }
    // free_ExpandableBuff(p_buff);
    avifRWDataFree(&xmp);
    if (rowPointers) {
        free(rowPointers);
    }
    avifRGBImageFreePixels(&rgbData);
    return p_buff;
}