//
// This file is part of the TEN Framework project.
// Licensed under the Apache License, Version 2.0.
// See the LICENSE file for more information.
//
#pragma once

#include <atomic>

#include "ten_runtime/binding/cpp/ten.h"
#include "ten_utils/lib/event.h"
#include "ten_utils/lib/thread.h"

namespace ten {
namespace ffmpeg_extension {

class demuxer_t;

class demuxer_thread_t {
 public:
  demuxer_thread_t(ten::ten_env_proxy_t *ten_env_proxy,
                   std::unique_ptr<ten::cmd_t> start_cmd,
                   extension_t *extension, std::string &input_stream_loc);

  ~demuxer_thread_t();

  demuxer_thread_t(const demuxer_thread_t &other) = delete;
  demuxer_thread_t(demuxer_thread_t &&other) = delete;

  demuxer_thread_t &operator=(const demuxer_thread_t &other) = delete;
  demuxer_thread_t &operator=(demuxer_thread_t &&other) = delete;

  void start();
  void wait_for_start();

  void start_demuxing();

  void stop() { stop_ = true; }
  void wait_for_stop();
  bool is_stopped() const { return stop_; }

 private:
  friend class demuxer_t;
  friend void *demuxer_thread_main(void *self_);

  void send_image_eof();
  void send_audio_eof();
  void reply_to_start_cmd(bool success = true);
  bool create_demuxer();
  void wait_for_demuxer();
  void notify_completed(bool success = true);

  ten::ten_env_proxy_t *ten_env_proxy_;
  ten::extension_t *extension_;

  std::atomic<bool> stop_;

  demuxer_t *demuxer_;
  ten_thread_t *demuxer_thread;
  ten_event_t *demuxer_thread_is_started;
  ten_event_t *ready_for_demuxer;

  std::string input_stream_loc_;
  std::unique_ptr<ten::cmd_t> start_cmd_;
};

}  // namespace ffmpeg_extension
}  // namespace ten
