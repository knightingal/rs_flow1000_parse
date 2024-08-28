#include <stdio.h>
#include <stdlib.h>
#include <string.h>

#include <libavcodec/avcodec.h>
#include <libavcodec/avcodec.h>
#include <libavformat/avformat.h>
#include <libavutil/imgutils.h>
#include <libavcodec/codec_id.h>
#include <libswscale/swscale.h>

#define INBUF_SIZE 4096

static AVFormatContext *fmt_ctx;
static char* filename = "/home/knightingal/demo_video.mp4";
// static char* output_file = "/home/knightingal/demo_video_1.jpg";
static FILE *output_file = NULL;

static int frame_to_image(AVFrame* frame, enum AVCodecID code_id, uint8_t* outbuf, size_t out_buf_size) {
  int ret = 0;
  AVPacket pkt;
  AVCodec* codec;
  AVCodecContext* ctx = NULL;
  AVFrame* rgb_frame = NULL;
  uint8_t* buffer = NULL;
  struct SwsContext* swsContext = NULL;
  av_init_packet(&pkt);
  codec = avcodec_find_encoder(code_id);
  if (!codec) {
    printf("codec non found\n");
    return -1;
  }
  if (!codec->pix_fmts) {
    printf("codec non support pix_fmt\n");
    
    return -1;
  }
  ctx = avcodec_alloc_context3(codec);
  ctx->bit_rate = 3000000;
  ctx->width = frame->width;
  ctx->height = frame->height;
  ctx->time_base.num = 1;
  ctx->time_base.den = 25;
  ctx->gop_size = 10;
  ctx->max_b_frames = 0;
  ctx->pix_fmt = *codec->pix_fmts;
  ret = avcodec_open2(ctx, codec, NULL);
  if (ret < 0) {
    printf("avcodec_open2 failed");
    return -1;
  }
  if (frame->format != ctx->pix_fmt) {
    rgb_frame = av_frame_alloc();
    swsContext = sws_getContext(frame->width, frame->height, 
      (enum AVPixelFormat)frame->format, frame->width, frame->height, 
      ctx->pix_fmt, 1, NULL, NULL, NULL
    );
    if (!swsContext) {
      printf("sws_getContext failed\n");
      return -1;
    }
    int buffer_size = av_image_get_buffer_size(ctx->pix_fmt, frame->width, frame->height, 1) * 2;
    buffer = (unsigned char*)av_malloc(buffer_size);
    av_image_fill_arrays(rgb_frame->data, rgb_frame->linesize, buffer, ctx->pix_fmt, frame->width, frame->height, 1);
    if ((ret = sws_scale(swsContext, (const uint8_t * const *)frame->data, frame->linesize, 0, frame->height, rgb_frame->data, rgb_frame->linesize)) < 0) {
      printf("sws_scale failed\n");
    }
    rgb_frame->format = ctx->pix_fmt;
    rgb_frame->width = ctx->width;
    rgb_frame->height = ctx->height;
    ret = avcodec_send_frame(ctx, rgb_frame);
  } else {
    ret = avcodec_send_frame(ctx, frame);
  }
  if (ret < 0) {
    printf("avcodec_send_frame failed\n");
  }
  ret = avcodec_receive_packet(ctx, &pkt);
  if (ret < 0) {
    printf("avcodec_receive_packet failed\n");
  }
  if (pkt.size > 0 && pkt.size <= out_buf_size) {
    memcpy(outbuf, pkt.data, pkt.size);
  }
  ret = pkt.size;


  return ret;

}



int main(int argc, char **argv) {
  const AVCodec *codec;
  AVCodecParserContext *parser;
  AVCodecContext *c= NULL;
  FILE *f;
  uint8_t inbuf[INBUF_SIZE + AV_INPUT_BUFFER_PADDING_SIZE];
  uint8_t *data;
  size_t   data_size;
  int ret;
  int eof;
  AVPacket *pkt;
  pkt = av_packet_alloc();
  ret = avformat_open_input(&fmt_ctx, filename, NULL, NULL);
  printf("red=%d\n", ret);
  output_file = fopen("/home/knightingal/demo_video_1.jpg", "w+b");

  ret = avformat_find_stream_info(fmt_ctx, 0);
  printf("red=%d\n", ret);
  av_dump_format(fmt_ctx, 0, filename, 0);
  int count = fmt_ctx->nb_streams;
  printf("number=%d\n", count);
  int video_stream_index = -1;
  int audio_stream_index = -1;
  AVCodecContext *dec_ctx;

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
      if(in_stream->avg_frame_rate.den != 0 && in_stream->avg_frame_rate.num != 0)
      {
        frame_rate = in_stream->avg_frame_rate.num/in_stream->avg_frame_rate.den; 
      }
      int video_frame_count = in_stream->nb_frames; 
      printf("width=%d, height=%d, frame_rate=%d, video_frame_count=%d\n", width, height, frame_rate, video_frame_count);
      const AVCodec* codec = avcodec_find_decoder(in_stream->codecpar->codec_id);
      const char* codec_name = codec->long_name;
      printf("codec_name=%s\n", codec_name);
      AVCodecParameters* para = avcodec_parameters_alloc();

      dec_ctx = avcodec_alloc_context3(codec);
      printf("dec_ctx=%p\n", dec_ctx);
      avcodec_parameters_to_context(dec_ctx, in_stream->codecpar);
      ret = avcodec_open2(dec_ctx, codec, NULL);
      printf("red=%d\n", ret);
    }  
  }
  printf("video_stream_index=%d, audio_stream_index=%d\n", video_stream_index, audio_stream_index);


  av_seek_frame(fmt_ctx, 0, 60000000, AVSEEK_FLAG_BACKWARD);
  AVPacket* p_packet = av_packet_alloc();
  while (1) {
    ret = av_read_frame(fmt_ctx, p_packet);
    printf("red=%d\n", ret);
    ret = avcodec_send_packet(dec_ctx, p_packet);
    printf("red=%d\n", ret);
    AVFrame *frame = av_frame_alloc();
  
    /* code */
    ret = avcodec_receive_frame(dec_ctx, frame);
    printf("red=%d\n", ret);
    if (ret == 0) {
      printf("read succ \n");
      int w = frame->width;
      int h = frame->height;
      printf("w=%d, h=%d\n", w, h);
      int size = av_image_get_buffer_size(AV_PIX_FMT_BGRA, frame->width,
                                        frame->height, 64);
      printf("size=%d\n", size);
      uint8_t *buffer = av_malloc(size);
      if (!buffer) {
          printf("Can not alloc buffer\n");
          ret = AVERROR(ENOMEM);
          break;;
      }
      ret = frame_to_image(frame, AV_CODEC_ID_MJPEG, buffer, size);
      if (ret < 0) {
        printf("Can not copy image to buffer\n");
        break;
      }
      if ((ret = fwrite(buffer, 1, ret, output_file)) < 0) {
            fprintf(stderr, "Failed to dump raw data.\n");
            break;
      }
      
      break;
    }
  }

  
  return 0;
}

