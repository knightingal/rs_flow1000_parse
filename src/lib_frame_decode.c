#include <stdio.h>
#include <stdlib.h>
#include <string.h>

#include <libavcodec/avcodec.h>
#include <libavcodec/avcodec.h>
#include <libavformat/avformat.h>
#include <libavutil/imgutils.h>
#include <libavcodec/codec_id.h>
#include <libswscale/swscale.h>
#include "lib_frame_decode.h"

#define INBUF_SIZE 4096
#define PIC_NUM 16

static AVFormatContext *fmt_ctx;
static char *FILE_NAME = "/home/knightingal/demo_video.mp4";
static char* DEST_URL = "demo_video_1.png";
// static char* output_file = "/home/knightingal/demo_video_1.jpg";
static FILE *output_file = NULL;


static AVFrame *frame_to_rgb_buff41(AVFrame *frame, uint32_t index, AVCodecContext *ctx, uint8_t *dest_buff)
{
  printf("index=%d\n", index);
  int ret = 0;
  AVFrame *rgb_frame = NULL;
  uint8_t *buffer = NULL;
  struct SwsContext *sws_context = NULL;
  int dest_width = frame->width / 4;
  int dest_height = frame->height / 4;
  rgb_frame = av_frame_alloc();
  sws_context = sws_getContext(frame->width, frame->height,
                               (enum AVPixelFormat)frame->format, dest_width, dest_height,
                               ctx->pix_fmt, 1, NULL, NULL, NULL);
  int buffer_size = av_image_get_buffer_size(ctx->pix_fmt, frame->width, frame->height, 1) * 2;
  buffer = (unsigned char *)av_malloc(buffer_size);
  av_image_fill_arrays(rgb_frame->data, rgb_frame->linesize, buffer, ctx->pix_fmt, frame->width, frame->height, 1);

  if ((ret = sws_scale(sws_context, (const uint8_t *const *)frame->data, frame->linesize, 0, frame->height, rgb_frame->data, rgb_frame->linesize)) < 0)
  {
    printf("sws_scale failed\n");
  }

  uint32_t x = index % 4;
  uint32_t y = index / 4;
  size_t width_offset = dest_width * 3 * x;
  size_t height_offset = dest_height * rgb_frame->linesize[0] * y;
  size_t rgb_data_size = rgb_frame->linesize[0] * frame->height;
  if (dest_buff == NULL)
  {
    dest_buff = (uint8_t *)av_malloc(rgb_data_size);
  }
  for (int line = 0; line < dest_height; line++)
  {
    memcpy(dest_buff + height_offset + line * rgb_frame->linesize[0] + width_offset, rgb_frame->data[0] + line * rgb_frame->linesize[0], dest_width * 3);
  }
  if (index == 0)
  {
    rgb_frame->data[0] = dest_buff;
    rgb_frame->format = ctx->pix_fmt;
    rgb_frame->width = ctx->width;
    rgb_frame->height = ctx->height;
    return rgb_frame;
  }
  else
  {
    av_frame_unref(rgb_frame);
    av_frame_free(&rgb_frame);
    return NULL;
  }
}

static AVFrame *frame_to_rgb_buff(AVFrame *frame, uint32_t index, AVCodecContext *ctx, uint8_t *dest_buff)
{
  printf("index=%d\n", index);
  int ret = 0;
  AVFrame *rgb_frame = NULL;
  uint8_t *buffer = NULL;
  struct SwsContext *sws_context = NULL;
  int dest_width = frame->width / 4;
  int dest_height = frame->height / 4;
  rgb_frame = av_frame_alloc();
  sws_context = sws_getContext(frame->width, frame->height,
                               (enum AVPixelFormat)frame->format, dest_width, dest_height,
                               ctx->pix_fmt, 1, NULL, NULL, NULL);
  int buffer_size = av_image_get_buffer_size(ctx->pix_fmt, frame->width, frame->height, 1) * 2;
  buffer = (unsigned char *)av_malloc(buffer_size);
  av_image_fill_arrays(rgb_frame->data, rgb_frame->linesize, buffer, ctx->pix_fmt, frame->width, frame->height, 1);

  if ((ret = sws_scale(sws_context, (const uint8_t *const *)frame->data, frame->linesize, 0, frame->height, rgb_frame->data, rgb_frame->linesize)) < 0)
  {
    printf("sws_scale failed\n");
  }

  uint32_t x = index % 4;
  uint32_t y = index / 4;
  size_t width_offset = dest_width * 3 * x;
  size_t height_offset = dest_height * rgb_frame->linesize[0] * y;
  size_t rgb_data_size = rgb_frame->linesize[0] * frame->height;
  if (dest_buff == NULL)
  {
    dest_buff = (uint8_t *)av_malloc(rgb_data_size);
  }
  for (int line = 0; line < dest_height; line++)
  {
    memcpy(dest_buff + height_offset + line * rgb_frame->linesize[0] + width_offset, rgb_frame->data[0] + line * rgb_frame->linesize[0], dest_width * 3);
  }
  if (index == 0)
  {
    rgb_frame->data[0] = dest_buff;
    rgb_frame->format = ctx->pix_fmt;
    rgb_frame->width = ctx->width;
    rgb_frame->height = ctx->height;
    printf("finish frame_to_rgb_buff\n");
    return rgb_frame;
  }
  else
  {
    av_frame_unref(rgb_frame);
    av_frame_free(&rgb_frame);
    printf("finish frame_to_rgb_buff\n");
    return NULL;
  }
}

static int frame_array_to_image41(AVFrame **frame_array, enum AVCodecID code_id, uint8_t *outbuf, size_t out_buf_size)
{
  int ret = 0;
  AVPacket *pkt = av_packet_alloc();
  const AVCodec *codec = NULL;
  AVCodecContext *ctx = NULL;
  AVFrame *rgb_frame = NULL;
  uint8_t *buffer = NULL;
  struct SwsContext *sws_context = NULL;
  codec = avcodec_find_encoder(code_id);
  ctx = avcodec_alloc_context3(codec);

  const enum AVPixelFormat *pix_fmts;
  avcodec_get_supported_config(NULL, codec, AV_CODEC_CONFIG_PIX_FORMAT, 0, (const void **) &pix_fmts, NULL);
  int dest_width = frame_array[0]->width;
  int dest_height = frame_array[0]->height;
  ctx->width = frame_array[0]->width / 4;
  ctx->height = frame_array[0]->height / 4;
  ctx->bit_rate = 3000000;
  ctx->time_base.num = 1;
  ctx->time_base.den = 25;
  ctx->gop_size = 10;
  ctx->max_b_frames = 0;
  ctx->pix_fmt = *pix_fmts;
  ret = avcodec_open2(ctx, codec, NULL);
  rgb_frame = frame_to_rgb_buff(frame_array[0], 0, ctx, NULL);
  rgb_frame->format = ctx->pix_fmt;
  rgb_frame->width = ctx->width ;
  rgb_frame->height = ctx->height ;
  // for (int i = 1; i < PIC_NUM; i++)
  // {
  //   frame_to_rgb_buff(frame_array[i], i, ctx, rgb_frame->data[0]);
  // }
  ret = avcodec_send_frame(ctx, rgb_frame);
  ret = avcodec_receive_packet(ctx, pkt);
  printf("start memcpy\n");
  memcpy(outbuf, pkt->data, pkt->size);
  ret = pkt->size;
  if (rgb_frame)
  {
    av_frame_unref(rgb_frame);
    av_frame_free(&rgb_frame);
  }
  if (ctx)
  {
    avcodec_close(ctx);
    avcodec_free_context(&ctx);
  }
  if (pkt) {
    av_packet_free(&pkt);
  }
  printf("finish frame_array_to_image41\n");
  return ret;
}

static int frame_array_to_image(AVFrame **frame_array, enum AVCodecID code_id, uint8_t *outbuf, size_t out_buf_size)
{
  int ret = 0;
  AVPacket *pkt = av_packet_alloc();
  const AVCodec *codec = NULL;
  AVCodecContext *ctx = NULL;
  AVFrame *rgb_frame = NULL;
  uint8_t *buffer = NULL;
  struct SwsContext *sws_context = NULL;
  codec = avcodec_find_encoder(code_id);
  ctx = avcodec_alloc_context3(codec);

  const enum AVPixelFormat *pix_fmts;
  avcodec_get_supported_config(NULL, codec, AV_CODEC_CONFIG_PIX_FORMAT, 0, (const void **) &pix_fmts, NULL);
  int dest_width = frame_array[0]->width;
  int dest_height = frame_array[0]->height;
  ctx->width = frame_array[0]->width;
  ctx->height = frame_array[0]->height;
  ctx->bit_rate = 3000000;
  ctx->time_base.num = 1;
  ctx->time_base.den = 25;
  ctx->gop_size = 10;
  ctx->max_b_frames = 0;
  ctx->pix_fmt = *pix_fmts;
  ret = avcodec_open2(ctx, codec, NULL);
  rgb_frame = frame_to_rgb_buff(frame_array[0], 0, ctx, NULL);
  rgb_frame->format = ctx->pix_fmt;
  rgb_frame->width = ctx->width;
  rgb_frame->height = ctx->height;
  for (int i = 1; i < PIC_NUM; i++)
  {
    frame_to_rgb_buff(frame_array[i], i, ctx, rgb_frame->data[0]);
  }
  ret = avcodec_send_frame(ctx, rgb_frame);
  ret = avcodec_receive_packet(ctx, pkt);
  memcpy(outbuf, pkt->data, pkt->size);
  ret = pkt->size;
  if (rgb_frame)
  {
    av_frame_unref(rgb_frame);
    av_frame_free(&rgb_frame);
  }
  if (ctx)
  {
    avcodec_close(ctx);
    avcodec_free_context(&ctx);
  }
  if (pkt) {
    av_packet_free(&pkt);
  }
  return ret;
}

static int frame_to_image(AVFrame *frame, enum AVCodecID code_id, uint8_t *outbuf, size_t out_buf_size)
{
  int ret = 0;
  AVPacket *pkt = av_packet_alloc();
  const AVCodec *codec = NULL;
  AVCodecContext *ctx = NULL;
  AVFrame *rgb_frame = NULL;
  uint8_t *buffer = NULL;
  struct SwsContext *sws_context = NULL;
  codec = avcodec_find_encoder(code_id);
  if (!codec)
  {
    ret = -1;
    printf("codec non found\n");
    goto error;
  }
  const enum AVPixelFormat *pix_fmts;
  ret = avcodec_get_supported_config(NULL, codec, AV_CODEC_CONFIG_PIX_FORMAT, 0, (const void **) &pix_fmts, NULL);
  if (ret < 0)
  {
    ret = -1;
    printf("codec non support pix_fmt\n");
    goto error;
  }
  ctx = avcodec_alloc_context3(codec);
  // int dest_width = frame->width / 4;
  // int dest_height = frame->height / 4;
  int dest_width = frame->width;
  int dest_height = frame->height;
  ctx->width = frame->width;
  ctx->height = frame->height;
  ctx->bit_rate = 3000000;
  ctx->time_base.num = 1;
  ctx->time_base.den = 25;
  ctx->gop_size = 10;
  ctx->max_b_frames = 0;
  ctx->pix_fmt = *pix_fmts;
  ret = avcodec_open2(ctx, codec, NULL);
  if (ret < 0)
  {
    printf("avcodec_open2 failed");
    goto error;
  }
  if (frame->format != ctx->pix_fmt)
  {
    rgb_frame = av_frame_alloc();
    sws_context = sws_getContext(frame->width, frame->height,
                                 (enum AVPixelFormat)frame->format, dest_width, dest_height,
                                 ctx->pix_fmt, 1, NULL, NULL, NULL);
    if (!sws_context)
    {
      printf("sws_getContext failed\n");
      ret = -1;
      goto error;
    }
    int buffer_size = av_image_get_buffer_size(ctx->pix_fmt, frame->width, frame->height, 1) * 2;
    buffer = (unsigned char *)av_malloc(buffer_size);
    av_image_fill_arrays(rgb_frame->data, rgb_frame->linesize, buffer, ctx->pix_fmt, frame->width, frame->height, 1);
    if ((ret = sws_scale(sws_context, (const uint8_t *const *)frame->data, frame->linesize, 0, frame->height, rgb_frame->data, rgb_frame->linesize)) < 0)
    {
      printf("sws_scale failed\n");
    }
    size_t rgb_size = rgb_frame->linesize[0] * frame->height;
    uint8_t *rgb_buffer = malloc(rgb_size);
    memset(rgb_buffer, 0, rgb_size);
    memcpy(rgb_buffer, rgb_frame->data[0], rgb_size);
    FILE *rgb_file = fopen("/home/knightingal/demo_video.mp4.bin", "w+b");

    fwrite(rgb_buffer, 1, rgb_size, rgb_file);

    rgb_frame->format = ctx->pix_fmt;
    rgb_frame->width = ctx->width;
    rgb_frame->height = ctx->height;
    ret = avcodec_send_frame(ctx, rgb_frame);
  }
  else
  {
    ret = avcodec_send_frame(ctx, frame);
  }
  if (ret < 0)
  {
    printf("avcodec_send_frame failed\n");
  }
  ret = avcodec_receive_packet(ctx, pkt);
  if (ret < 0)
  {
    printf("avcodec_receive_packet failed\n");
  }
  if (pkt->size > 0 && pkt->size <= out_buf_size)
  {
    memcpy(outbuf, pkt->data, pkt->size);
  }
  ret = pkt->size;

error:
  if (sws_context)
  {
    sws_freeContext(sws_context);
  }
  if (rgb_frame)
  {
    av_frame_unref(rgb_frame);
    av_frame_free(&rgb_frame);
  }
  if (buffer)
  {
    av_free(buffer);
  }
  if (ctx)
  {
    avcodec_close(ctx);
    avcodec_free_context(&ctx);
  }
  if (pkt) {
    av_packet_free(&pkt);
  }
  return ret;
}

struct video_meta_info* video_meta_info(const char* name_path) {
  struct video_meta_info* p_video_meta_info = malloc(sizeof(struct video_meta_info));

  int ret;
  int eof;
  const char *filename;
  if (name_path != NULL) {
    filename = name_path;
  } else {
    filename = FILE_NAME;
  }
  
  ret = avformat_open_input(&fmt_ctx, filename, NULL, NULL);
  printf("red=%d\n", ret);

  ret = avformat_find_stream_info(fmt_ctx, 0);
  printf("red=%d\n", ret);
  av_dump_format(fmt_ctx, 0, filename, 0);
  int count = fmt_ctx->nb_streams;
  printf("number=%d\n", count);
  int video_stream_index = -1;
  int audio_stream_index = -1;
  int i_duratoin;
  for (int i = 0; i < fmt_ctx->nb_streams; i++)
  {
    AVStream *in_stream = fmt_ctx->streams[i];
    if (in_stream->codecpar->codec_type == AVMEDIA_TYPE_VIDEO)
    {
      video_stream_index = i;
    }
    else if (in_stream->codecpar->codec_type == AVMEDIA_TYPE_AUDIO)
    {
      audio_stream_index = i;
    }

    if (in_stream->codecpar->codec_type == AVMEDIA_TYPE_VIDEO)
    {
      int width = in_stream->codecpar->width;
      int height = in_stream->codecpar->height;
      int frame_rate;
      if (in_stream->avg_frame_rate.den != 0 && in_stream->avg_frame_rate.num != 0)
      {
        frame_rate = in_stream->avg_frame_rate.num / in_stream->avg_frame_rate.den;
      }
      int video_frame_count = in_stream->nb_frames;
      printf("width=%d, height=%d, frame_rate=%d, video_frame_count=%d\n", width, height, frame_rate, video_frame_count);
      float f_duration = (float)video_frame_count / ((float)(in_stream->avg_frame_rate.num) / (float)(in_stream->avg_frame_rate.den));
      i_duratoin = (int)f_duration;
      printf("duration=%d\n", i_duratoin);

      p_video_meta_info->width = width;
      p_video_meta_info->height = height;
      p_video_meta_info->frame_rate = frame_rate;
      p_video_meta_info->video_frame_count = video_frame_count;
      p_video_meta_info->duratoin = i_duratoin;
    }
  }

  avformat_close_input(&fmt_ctx);
  return p_video_meta_info;
}

struct snapshot_st snapshot_video(const char* name_path, const uint64_t snap_time) {
  int ret;
  int eof;
  const char *filename;
  if (name_path != NULL) {
    filename = name_path;
  } else {
    filename = FILE_NAME;
  }

  ret = avformat_open_input(&fmt_ctx, filename, NULL, NULL);
  printf("red=%d\n", ret);

  ret = avformat_find_stream_info(fmt_ctx, 0);
  printf("red=%d\n", ret);
  av_dump_format(fmt_ctx, 0, filename, 0);
  int count = fmt_ctx->nb_streams;
  printf("number=%d\n", count);
  int video_stream_index = -1;
  int audio_stream_index = -1;
  AVCodecContext *dec_ctx;
  const AVCodec *codec;
  AVStream *video_in_stream;
  int i_duratoin;
  for (int i = 0; i < fmt_ctx->nb_streams; i++)
  {
    AVStream *in_stream = fmt_ctx->streams[i];
    if (in_stream->codecpar->codec_type == AVMEDIA_TYPE_VIDEO)
    {
      video_stream_index = i;
    }
    else if (in_stream->codecpar->codec_type == AVMEDIA_TYPE_AUDIO)
    {
      audio_stream_index = i;
    }

    if (in_stream->codecpar->codec_type == AVMEDIA_TYPE_VIDEO)
    {
      video_in_stream = in_stream;
      int width = in_stream->codecpar->width;
      int height = in_stream->codecpar->height;
      int frame_rate;
      if (in_stream->avg_frame_rate.den != 0 && in_stream->avg_frame_rate.num != 0)
      {
        frame_rate = in_stream->avg_frame_rate.num / in_stream->avg_frame_rate.den;
      }
      int video_frame_count = in_stream->nb_frames;
      printf("width=%d, height=%d, frame_rate=%d, video_frame_count=%d\n", width, height, frame_rate, video_frame_count);
      float f_duration = (float)video_frame_count / ((float)(in_stream->avg_frame_rate.num) / (float)(in_stream->avg_frame_rate.den));
      i_duratoin = (int)f_duration;
      printf("duration=%d\n", i_duratoin);
      codec = avcodec_find_decoder(in_stream->codecpar->codec_id);
      const char *codec_name = codec->long_name;
      printf("codec_name=%s\n", codec_name);
      printf("red=%d\n", ret);
    }
  }
  printf("video_stream_index=%d, audio_stream_index=%d\n", video_stream_index, audio_stream_index);

  AVFrame *frame_array[1];
  dec_ctx = avcodec_alloc_context3(codec);
  avcodec_parameters_to_context(dec_ctx, video_in_stream->codecpar);
  ret = avcodec_open2(dec_ctx, codec, NULL);

  int64_t timestamp = (int64_t)(snap_time) * 1000000l;

  printf("i=%d, timestamp=%lld\n", 0, timestamp);
  av_seek_frame(fmt_ctx, -1, timestamp, AVSEEK_FLAG_BACKWARD);
  AVPacket *p_packet = av_packet_alloc();
  while (1)
  {
    ret = av_read_frame(fmt_ctx, p_packet);
    printf("red=%d\n", ret);
    ret = avcodec_send_packet(dec_ctx, p_packet);
    printf("red=%d\n", ret);
    AVFrame *frame = av_frame_alloc();

    /* code */
    ret = avcodec_receive_frame(dec_ctx, frame);
    printf("red=%d\n", ret);
    if (ret == 0)
    {
      frame_array[0] = frame;
      printf("read succ \n");
      int w = frame->width;
      int h = frame->height;
      printf("w=%d, h=%d\n", w, h);

      break;
    }
  }
  av_packet_free(&p_packet);
  avcodec_flush_buffers(dec_ctx);

  avcodec_close(dec_ctx);
  avcodec_free_context(&dec_ctx);

  int size = av_image_get_buffer_size(AV_PIX_FMT_BGRA, frame_array[0]->width,
                                      frame_array[0]->height, 64);

  printf("size=%d\n", size);
  uint8_t *buffer = av_malloc(size);
  if (!buffer)
  {
    printf("Can not alloc buffer\n");
    ret = AVERROR(ENOMEM);
  }
  ret = frame_array_to_image41(frame_array, AV_CODEC_ID_PNG, buffer, size);

  for (int i = 0; i < 1; i++)
  {
    av_frame_free(&frame_array[i]);
  }

  // output_file = fopen(DEST_URL, "w+b");
  // if (fwrite(buffer, 1, ret, output_file) < 0)
  // {
  //   fprintf(stderr, "Failed to dump raw data.\n");
  // }
  // av_free(buffer);
  

  // fclose(output_file);
  free(dec_ctx);
  avformat_close_input(&fmt_ctx);
  struct snapshot_st st = {
    buffer, ret
  };

  return st;
}

int frame_decode(const char* name_path, const char *dest_path)
{
  int ret;
  int eof;
  const char *filename;
  if (name_path != NULL) {
    filename = name_path;
  } else {
    filename = FILE_NAME;
  }
  
  if (dest_path == NULL) {
    dest_path = DEST_URL;
  }

  ret = avformat_open_input(&fmt_ctx, filename, NULL, NULL);
  printf("red=%d\n", ret);

  ret = avformat_find_stream_info(fmt_ctx, 0);
  printf("red=%d\n", ret);
  av_dump_format(fmt_ctx, 0, filename, 0);
  int count = fmt_ctx->nb_streams;
  printf("number=%d\n", count);
  int video_stream_index = -1;
  int audio_stream_index = -1;
  AVCodecContext *dec_ctx;
  const AVCodec *codec;
  AVStream *video_in_stream;
  int i_duratoin;
  for (int i = 0; i < fmt_ctx->nb_streams; i++)
  {
    AVStream *in_stream = fmt_ctx->streams[i];
    if (in_stream->codecpar->codec_type == AVMEDIA_TYPE_VIDEO)
    {
      video_stream_index = i;
    }
    else if (in_stream->codecpar->codec_type == AVMEDIA_TYPE_AUDIO)
    {
      audio_stream_index = i;
    }

    if (in_stream->codecpar->codec_type == AVMEDIA_TYPE_VIDEO)
    {
      video_in_stream = in_stream;
      int width = in_stream->codecpar->width;
      int height = in_stream->codecpar->height;
      int frame_rate;
      if (in_stream->avg_frame_rate.den != 0 && in_stream->avg_frame_rate.num != 0)
      {
        frame_rate = in_stream->avg_frame_rate.num / in_stream->avg_frame_rate.den;
      }
      int video_frame_count = in_stream->nb_frames;
      printf("width=%d, height=%d, frame_rate=%d, video_frame_count=%d\n", width, height, frame_rate, video_frame_count);
      float f_duration = (float)video_frame_count / ((float)(in_stream->avg_frame_rate.num) / (float)(in_stream->avg_frame_rate.den));
      i_duratoin = (int)f_duration;
      printf("duration=%d\n", i_duratoin);
      codec = avcodec_find_decoder(in_stream->codecpar->codec_id);
      const char *codec_name = codec->long_name;
      printf("codec_name=%s\n", codec_name);
      // AVCodecParameters* para = avcodec_parameters_alloc();

      // dec_ctx = avcodec_alloc_context3(codec);
      // printf("dec_ctx=%p\n", dec_ctx);
      // avcodec_parameters_to_context(dec_ctx, in_stream->codecpar);
      // ret = avcodec_open2(dec_ctx, codec, NULL);
      printf("red=%d\n", ret);
    }
  }
  printf("video_stream_index=%d, audio_stream_index=%d\n", video_stream_index, audio_stream_index);
  int sub_duration = i_duratoin / (PIC_NUM + 2);
  printf("sub_duration=%d", sub_duration);
  AVFrame *frame_array[PIC_NUM];
  dec_ctx = avcodec_alloc_context3(codec);
  avcodec_parameters_to_context(dec_ctx, video_in_stream->codecpar);
  ret = avcodec_open2(dec_ctx, codec, NULL);
  for (int i = 0; i < PIC_NUM; i++)
  {
    int64_t timestamp = (int64_t)((i + 1) * sub_duration) * 1000000l;
    printf("i=%d, timestamp=%lld\n", i, timestamp);
    av_seek_frame(fmt_ctx, -1, timestamp, AVSEEK_FLAG_BACKWARD);
    AVPacket *p_packet = av_packet_alloc();
    while (1)
    {
      ret = av_read_frame(fmt_ctx, p_packet);
      printf("red=%d\n", ret);
      ret = avcodec_send_packet(dec_ctx, p_packet);
      printf("red=%d\n", ret);
      AVFrame *frame = av_frame_alloc();

      /* code */
      ret = avcodec_receive_frame(dec_ctx, frame);
      printf("red=%d\n", ret);
      if (ret == 0)
      {
        frame_array[i] = frame;
        printf("read succ \n");
        int w = frame->width;
        int h = frame->height;
        printf("w=%d, h=%d\n", w, h);

        break;
      }
    }
    av_packet_free(&p_packet);
    avcodec_flush_buffers(dec_ctx);
  }
  avcodec_close(dec_ctx);
  avcodec_free_context(&dec_ctx);

  int size = av_image_get_buffer_size(AV_PIX_FMT_BGRA, frame_array[0]->width,
                                      frame_array[0]->height, 64);
  printf("size=%d\n", size);
  uint8_t *buffer = av_malloc(size);
  if (!buffer)
  {
    printf("Can not alloc buffer\n");
    ret = AVERROR(ENOMEM);
  }
  ret = frame_array_to_image(frame_array, AV_CODEC_ID_PNG, buffer, size);
  for (int i = 0; i < 16; i++)
  {
    av_frame_free(&frame_array[i]);
  }

  output_file = fopen(dest_path, "w+b");
  if (fwrite(buffer, 1, ret, output_file) < 0)
  {
    fprintf(stderr, "Failed to dump raw data.\n");
  }
  av_free(buffer);

  fclose(output_file);
  free(dec_ctx);
  avformat_close_input(&fmt_ctx);

  return 0;
}


int frame_decode_with_param(const char *url, const char* dest_url) {
  printf("url:%s\n",url);
  return frame_decode(url, dest_url);
}