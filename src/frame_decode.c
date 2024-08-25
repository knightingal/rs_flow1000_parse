#include <stdio.h>
#include <stdlib.h>
#include <string.h>

#include <libavcodec/avcodec.h>
#include <libavcodec/avcodec.h>
#include <libavformat/avformat.h>

#define INBUF_SIZE 4096

static AVFormatContext *fmt_ctx;
static char* filename = "/home/knightingal/demo_video.mp4";
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


  av_seek_frame(fmt_ctx, 0, 0, AVSEEK_FLAG_BACKWARD);
  AVPacket* p_packet = av_packet_alloc();
  ret = av_read_frame(fmt_ctx, p_packet);
  printf("red=%d\n", ret);
  ret = avcodec_send_packet(dec_ctx, p_packet);
  printf("red=%d\n", ret);
  AVFrame *frame = av_frame_alloc();
  ret = avcodec_receive_frame(dec_ctx, frame);
  printf("red=%d\n", ret);
  return 0;
}