//
// This file is part of TEN Framework, an open source project.
// Licensed under the Apache License, Version 2.0.
// See the LICENSE file for more information.
//
#include "ten_runtime/binding/cpp/detail/msg/cmd/cmd.h"
#include "ten_runtime/binding/cpp/ten.h"
#include "vosk_api.h"

namespace vosk_asr_cpp {

class vosk_asr_cpp_t : public ten::extension_t {
 public:
  explicit vosk_asr_cpp_t(const char *name) : extension_t(name) {}

  void on_init(ten::ten_env_t &ten_env) override {
    vosk_model = vosk_model_new("model");
    if (vosk_model == nullptr) {
      TEN_LOGE("Failed to load model, check if exists in the folder.");
      exit(EXIT_FAILURE);
    }

    auto sample_rate = ten_env.get_property_float32("sample_rate");

    vosk_recognizer = vosk_recognizer_new(vosk_model, sample_rate);
    if (vosk_recognizer == nullptr) {
      TEN_LOGE("Failed to create recognizer.");
      exit(EXIT_FAILURE);
    }

    ten_env.on_init_done();
  }

  void on_cmd(ten::ten_env_t &ten_env,
              std::unique_ptr<ten::cmd_t> cmd) override {
    auto cmd_result = ten::cmd_result_t::create(TEN_STATUS_CODE_OK);
    cmd_result->set_property("detail", "This is a demo");
    ten_env.return_result(std::move(cmd_result), std::move(cmd));
  }

  void on_audio_frame(ten::ten_env_t &ten_env,
                      std::unique_ptr<ten::audio_frame_t> frame) override {
    ten::buf_t locked_in_buf = frame->lock_buf();

    int is_final = vosk_recognizer_accept_waveform(
        vosk_recognizer, reinterpret_cast<const char *>(locked_in_buf.data()),
        static_cast<int>(locked_in_buf.size()));

    frame->unlock_buf(locked_in_buf);

    auto recognition_result = ten::data_t::create("recognition_result");

    const char *result = nullptr;
    if (is_final != 0) {
      result = vosk_recognizer_result(vosk_recognizer);
    } else {
      result = vosk_recognizer_partial_result(vosk_recognizer);
    }
    recognition_result->set_property("result", result);

    recognition_result->set_property("is_final", is_final);

    bool rc = ten_env.send_data(std::move(recognition_result));
    TEN_ASSERT(rc, "Should not happen.");
  }

  void on_stop(ten::ten_env_t &ten_env) override {
    // Extension stop.
    ten_env.on_stop_done();
  }

 private:
  VoskModel *vosk_model = nullptr;
  VoskRecognizer *vosk_recognizer = nullptr;
};

TEN_CPP_REGISTER_ADDON_AS_EXTENSION(vosk_asr_cpp, vosk_asr_cpp_t);

}  // namespace vosk_asr_cpp
