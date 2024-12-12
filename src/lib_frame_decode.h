#include <stdint.h>

int frame_decode(const char *name_path, const char *dest_path);

int frame_decode_with_param(const char *url, const char *dest_path);

struct snapshot_st
{
  uint8_t *buff;
  int buff_len;
};

struct snapshot_st snapshot_video(const char *name_path, const uint64_t snap_time);

struct video_meta_info
{
  int width;
  int height;
  int frame_rate;
  int video_frame_count;
  int duratoin;
};

struct video_meta_info *video_meta_info(const char *url);