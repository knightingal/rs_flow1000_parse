#ifndef LIBAVIF_APPS_SHARED_AVIFUTIL_H
#define LIBAVIF_APPS_SHARED_AVIFUTIL_H

#include <avif/avif.h>


avifResult avifApplyTransforms(avifRGBImage * dstView, avifRGBImage * srcImage, const avifImage * avif);

void avifRGBImageSetViewRect(avifRGBImage * dstImage, const avifRGBImage * srcImage, const avifCropRect * cropRect);

avifResult avifRGBImageRotate(avifRGBImage * dstImage, const avifRGBImage * srcImage, const avifImageRotation * rotation);

avifResult avifRGBImageMirror(avifRGBImage * image, const avifImageMirror * mirror);

#endif // ifndef LIBAVIF_APPS_SHARED_AVIFUTIL_H