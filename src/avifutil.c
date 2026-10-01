#include <stdio.h>
#include "avifutil.h"
#include <string.h>

avifResult avifRGBImageRotate(avifRGBImage * dstImage, const avifRGBImage * srcImage, const avifImageRotation * rotation)
{
    const uint32_t bytesPerPixel = avifRGBImagePixelSize(srcImage);
    const uint8_t angle = rotation->angle;
    const uint32_t newWidth = (angle == 0 || angle == 2) ? srcImage->width : srcImage->height;
    const uint32_t newHeight = (angle == 0 || angle == 2) ? srcImage->height : srcImage->width;
    *dstImage = *srcImage;
    dstImage->width = newWidth;
    dstImage->height = newHeight;
    dstImage->pixels = NULL;
    avifResult result = avifRGBImageAllocatePixels(dstImage);
    if (result != AVIF_RESULT_OK) {
        return result;
    }

    if (rotation->angle == 0) {
        const size_t bytesPerRow = (size_t)bytesPerPixel * srcImage->width;
        // 0 degrees. Just copy the rows as is.
        for (uint32_t j = 0; j < srcImage->height; ++j) {
            memcpy(dstImage->pixels + ((size_t)j * dstImage->rowBytes), srcImage->pixels + ((size_t)j * srcImage->rowBytes), bytesPerRow);
        }
    } else if (rotation->angle == 1) {
        // 90 degrees anti-clockwise.
        for (uint32_t j = 0; j < srcImage->height; ++j) {
            for (uint32_t i = 0; i < srcImage->width; ++i) {
                // Source pixel at (i, j) goes to destination pixel at (j, srcImage->width - 1 - i).
                memcpy(dstImage->pixels + ((size_t)(srcImage->width - 1 - i) * dstImage->rowBytes) + ((size_t)j * bytesPerPixel),
                       srcImage->pixels + ((size_t)j * srcImage->rowBytes) + ((size_t)i * bytesPerPixel),
                       bytesPerPixel);
            }
        }
    } else if (rotation->angle == 2) {
        // 180 degrees.
        for (uint32_t j = 0; j < srcImage->height; ++j) {
            for (uint32_t i = 0; i < srcImage->width; ++i) {
                // Source pixel at (i, j) goes to destination pixel at (srcImage->width - 1 - i, srcImage->height - 1 - j).
                memcpy(dstImage->pixels + ((size_t)(srcImage->height - 1 - j) * dstImage->rowBytes) +
                           ((size_t)(srcImage->width - 1 - i) * bytesPerPixel),
                       srcImage->pixels + ((size_t)j * srcImage->rowBytes) + ((size_t)i * bytesPerPixel),
                       bytesPerPixel);
            }
        }
    } else if (rotation->angle == 3) {
        // 90 degrees clockwise.
        for (uint32_t j = 0; j < srcImage->height; ++j) {
            for (uint32_t i = 0; i < srcImage->width; ++i) {
                // Source pixel at (i, j) goes to destination pixel at (srcImage->width - 1 - i, j).
                memcpy(dstImage->pixels + ((size_t)i * dstImage->rowBytes) + ((size_t)(srcImage->height - 1 - j) * bytesPerPixel),
                       srcImage->pixels + ((size_t)j * srcImage->rowBytes) + ((size_t)i * bytesPerPixel),
                       bytesPerPixel);
            }
        }
    } else {
        return AVIF_RESULT_INVALID_ARGUMENT; // Invalid angle.
    }
    return AVIF_RESULT_OK;
}

avifResult avifRGBImageMirror(avifRGBImage * image, const avifImageMirror * mirror)
{
    if (mirror->axis == 0) { // Horizontal axis.
        const uint32_t bytesPerPixel = avifRGBImagePixelSize(image);
        // May be less than image->rowBytes e.g. if image is a cropped view.
        const size_t bytesPerRowToMove = (size_t)bytesPerPixel * image->width;
        // Top-to-bottom
        uint8_t * tempRow = (uint8_t *)avifAlloc(bytesPerRowToMove);
        if (!tempRow) {
            return AVIF_RESULT_OUT_OF_MEMORY;
        }
        for (uint32_t y = 0; y < image->height / 2; ++y) {
            uint8_t * row1 = &image->pixels[(size_t)y * image->rowBytes];
            uint8_t * row2 = &image->pixels[(size_t)(image->height - 1 - y) * image->rowBytes];
            memcpy(tempRow, row1, bytesPerRowToMove);
            memcpy(row1, row2, bytesPerRowToMove);
            memcpy(row2, tempRow, bytesPerRowToMove);
        }
        avifFree(tempRow);
    } else if (mirror->axis == 1) { // Vertical axis.
        const uint32_t bytesPerPixel = avifRGBImagePixelSize(image);
        uint8_t tempPixel[8]; // Max pixel size should be 8 bytes (RGBA 16-bit).
        if (bytesPerPixel > sizeof(tempPixel)) {
            return AVIF_RESULT_INVALID_ARGUMENT;
        }
        for (uint32_t y = 0; y < image->height; ++y) {
            uint8_t * row = &image->pixels[(size_t)y * image->rowBytes];
            for (uint32_t x = 0; x < image->width / 2; ++x) {
                uint8_t * pixel1 = &row[(size_t)x * bytesPerPixel];
                uint8_t * pixel2 = &row[(size_t)(image->width - 1 - x) * bytesPerPixel];
                memcpy(tempPixel, pixel1, bytesPerPixel);
                memcpy(pixel1, pixel2, bytesPerPixel);
                memcpy(pixel2, tempPixel, bytesPerPixel);
            }
        }
    } else {
        return AVIF_RESULT_INVALID_ARGUMENT; // Invalid axis value.
    }

    return AVIF_RESULT_OK;
}

void avifRGBImageSetViewRect(avifRGBImage * dstImage, const avifRGBImage * srcImage, const avifCropRect * cropRect)
{
    memset(dstImage, 0, sizeof(avifRGBImage));
    dstImage->width = cropRect->width;
    dstImage->height = cropRect->height;
    dstImage->depth = srcImage->depth;
    dstImage->format = srcImage->format;
    dstImage->alphaPremultiplied = srcImage->alphaPremultiplied;
    dstImage->isFloat = srcImage->isFloat;
    const uint32_t bytesPerPixel = avifRGBImagePixelSize(srcImage);
    // This should not overflow if cropRect is a valid crop of the image.
    const size_t offset = (size_t)cropRect->y * srcImage->rowBytes + (size_t)cropRect->x * bytesPerPixel;
    dstImage->pixels = srcImage->pixels + offset;
    dstImage->rowBytes = srcImage->rowBytes;
}

avifResult avifApplyTransforms(avifRGBImage * dstView, avifRGBImage * srcImage, const avifImage * avif)
{
    // ISO/IEC 23000-22 (MIAF), Section 7.3.6.7:
    //  These properties, if used, shall be indicated to be applied in the following order:
    //  clean aperture first, then rotation, then mirror.
    *dstView = *srcImage;
    if (avif->transformFlags & AVIF_TRANSFORM_CLAP) {
        avifCropRect cropRect;
        avifDiagnostics diag;
        if (avifCropRectFromCleanApertureBox(&cropRect, &avif->clap, avif->width, avif->height, &diag) &&
            (cropRect.x != 0 || cropRect.y != 0 || cropRect.width != avif->width || cropRect.height != avif->height)) {
            avifRGBImageSetViewRect(dstView, srcImage, &cropRect);
        } else {
            fprintf(stderr, "Invalid clean aperture box\n");
            return AVIF_RESULT_INVALID_ARGUMENT;
        }
    }
    if (avif->transformFlags & AVIF_TRANSFORM_IROT && avif->irot.angle != 0) {
        avifRGBImage tmpRgbImage;
        avifResult result = avifRGBImageRotate(&tmpRgbImage, dstView, &avif->irot);
        if (result != AVIF_RESULT_OK) {
            fprintf(stderr, "Failed to apply rotation\n");
            avifRGBImageFreePixels(&tmpRgbImage);
            return result;
        }
        // We assume that srcImage owned its pixels and free them before replacing it with tmpRgbImage.
        avifRGBImageFreePixels(srcImage);
        *srcImage = tmpRgbImage;
        *dstView = *srcImage;
    }
    if (avif->transformFlags & AVIF_TRANSFORM_IMIR) {
        avifResult result = avifRGBImageMirror(dstView, &avif->imir);
        if (result != AVIF_RESULT_OK) {
            fprintf(stderr, "Failed to apply mirror\n");
            return result;
        }
    }
    return AVIF_RESULT_OK;
}