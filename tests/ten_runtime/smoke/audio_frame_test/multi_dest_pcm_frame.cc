//
// This file is part of the TEN Framework project.
// See https://github.com/TEN-framework/ten_framework/LICENSE for license
// information.
//
#include <cstdlib>
#include <nlohmann/json.hpp>
#include <string>

#include "gtest/gtest.h"
#include "include_internal/ten_runtime/binding/cpp/ten.h"
#include "ten_utils/lib/thread.h"
#include "ten_utils/macro/check.h"
#include "tests/common/client/cpp/msgpack_tcp.h"
#include "tests/ten_runtime/smoke/extension_test/util/binding/cpp/check.h"

#define SAMPLE_RATE 16000
#define NUM_OF_CHANNELS 1

namespace {

class test_extension_1 : public ten::extension_t {
 public:
  explicit test_extension_1(const std::string &name) : ten::extension_t(name) {}

  static std::unique_ptr<ten::audio_frame_t> createEmptyAudioFrame(
      int sample_rate, int num_channels) {
    int sampleSize = sizeof(int16_t) * num_channels;
    int samplesPer10ms = sample_rate / 100;
    int sendBytes = sampleSize * samplesPer10ms;

    auto audio_frame = ten::audio_frame_t::create("audio_frame");
    audio_frame->alloc_buf(sendBytes);
    audio_frame->set_data_fmt(TEN_AUDIO_FRAME_DATA_FMT_INTERLEAVE);
    audio_frame->set_bytes_per_sample(2);
    audio_frame->set_sample_rate(sample_rate);
    audio_frame->set_number_of_channels(num_channels);
    audio_frame->set_samples_per_channel(samplesPer10ms);

    return audio_frame;
  }

  void on_cmd(ten::ten_env_t &ten_env,
              std::unique_ptr<ten::cmd_t> cmd) override {
    nlohmann::json json = nlohmann::json::parse(cmd->to_json());
    if (json["_ten"]["name"] == "dispatch_data") {
      auto audio_frame = createEmptyAudioFrame(SAMPLE_RATE, NUM_OF_CHANNELS);
      audio_frame->set_property("test_prop", "test_prop_value");

      ten_env.send_audio_frame(std::move(audio_frame));

      auto cmd_result = ten::cmd_result_t::create(TEN_STATUS_CODE_OK);
      cmd_result->set_property("detail", "done");
      ten_env.return_result(std::move(cmd_result), std::move(cmd));
    }
  }
};

class test_extension_2 : public ten::extension_t {
 public:
  explicit test_extension_2(const std::string &name) : ten::extension_t(name) {}

  void on_audio_frame(
      TEN_UNUSED ten::ten_env_t &ten_env,
      std::unique_ptr<ten::audio_frame_t> audio_frame) override {
    auto test_value = audio_frame->get_property_string("test_prop");
    TEN_ASSERT(test_value == "test_prop_value", "test_prop_value not match");

    if (audio_frame->get_number_of_channels() == NUM_OF_CHANNELS &&
        audio_frame->get_sample_rate() == SAMPLE_RATE) {
      received = true;
    }
  }

  void on_cmd(ten::ten_env_t &ten_env,
              std::unique_ptr<ten::cmd_t> cmd) override {
    nlohmann::json json = nlohmann::json::parse(cmd->to_json());
    if (json["_ten"]["name"] == "check_received") {
      if (received) {
        auto cmd_result = ten::cmd_result_t::create(TEN_STATUS_CODE_OK);
        cmd_result->set_property("detail", "received confirmed");
        ten_env.return_result(std::move(cmd_result), std::move(cmd));
      } else {
        auto cmd_result = ten::cmd_result_t::create(TEN_STATUS_CODE_ERROR);
        cmd_result->set_property("detail", "received failed");
        ten_env.return_result(std::move(cmd_result), std::move(cmd));
      }
    }
  }

 private:
  bool received{false};
};

class test_extension_3 : public ten::extension_t {
 public:
  explicit test_extension_3(const std::string &name) : ten::extension_t(name) {}

  void on_audio_frame(
      TEN_UNUSED ten::ten_env_t &ten_env,
      std::unique_ptr<ten::audio_frame_t> audio_frame) override {
    auto test_value = audio_frame->get_property_string("test_prop");
    TEN_ASSERT(test_value == "test_prop_value", "test_prop_value not match");

    if (audio_frame->get_number_of_channels() == NUM_OF_CHANNELS &&
        audio_frame->get_sample_rate() == SAMPLE_RATE) {
      received = true;
    }
  }

  void on_cmd(ten::ten_env_t &ten_env,
              std::unique_ptr<ten::cmd_t> cmd) override {
    nlohmann::json json = nlohmann::json::parse(cmd->to_json());
    if (json["_ten"]["name"] == "check_received") {
      if (received) {
        auto cmd_result = ten::cmd_result_t::create(TEN_STATUS_CODE_OK);
        cmd_result->set_property("detail", "received confirmed");
        ten_env.return_result(std::move(cmd_result), std::move(cmd));
      } else {
        auto cmd_result = ten::cmd_result_t::create(TEN_STATUS_CODE_ERROR);
        cmd_result->set_property("detail", "received failed");
        ten_env.return_result(std::move(cmd_result), std::move(cmd));
      }
    }
  }

 private:
  bool received{false};
};

class test_app : public ten::app_t {
 public:
  void on_init(ten::ten_env_t &ten_env) override {
    bool rc = ten_env.init_property_from_json(
        // clang-format off
                 R"({
                      "_ten": {
                        "uri": "msgpack://127.0.0.1:8001/",
                        "log_level": 1
                      }
                    })"
        // clang-format on
        ,
        nullptr);
    ASSERT_EQ(rc, true);

    ten_env.on_init_done();
  }
};

void *test_app_thread_main(TEN_UNUSED void *args) {
  auto *app = new test_app();
  app->run();
  delete app;

  return nullptr;
}

TEN_CPP_REGISTER_ADDON_AS_EXTENSION(multi_dest_audio_frame__extension_1,
                                    test_extension_1);
TEN_CPP_REGISTER_ADDON_AS_EXTENSION(multi_dest_audio_frame__extension_2,
                                    test_extension_2);
TEN_CPP_REGISTER_ADDON_AS_EXTENSION(multi_dest_audio_frame__extension_3,
                                    test_extension_3);

}  // namespace

TEST(AudioFrameTest, MultiDestAudioFrame) {  // NOLINT
  // Start app.
  auto *app_thread =
      ten_thread_create("app thread", test_app_thread_main, nullptr);

  // Create a client and connect to the app.
  auto *client = new ten::msgpack_tcp_client_t("msgpack://127.0.0.1:8001/");

  // Send graph.
  nlohmann::json resp = client->send_json_and_recv_resp_in_json(
      R"({
           "_ten": {
             "type": "start_graph",
             "seq_id": "55",
             "nodes": [{
               "type": "extension_group",
               "name": "test extension group",
               "addon": "default_extension_group",
               "app": "msgpack://127.0.0.1:8001/"
             },{
               "type": "extension",
               "name": "extension 1",
               "addon": "multi_dest_audio_frame__extension_1",
               "app": "msgpack://127.0.0.1:8001/",
               "extension_group": "test extension group"
             },{
               "type": "extension",
               "name": "extension 2",
               "addon": "multi_dest_audio_frame__extension_2",
               "app": "msgpack://127.0.0.1:8001/",
               "extension_group": "test extension group"
             },{
               "type": "extension",
               "name": "extension 3",
               "addon": "multi_dest_audio_frame__extension_3",
               "app": "msgpack://127.0.0.1:8001/",
               "extension_group": "test extension group"
             }],
             "connections": [{
               "app": "msgpack://127.0.0.1:8001/",
               "extension_group": "test extension group",
               "extension": "extension 1",
               "audio_frame": [{
                 "name": "audio_frame",
                 "dest": [{
                   "app": "msgpack://127.0.0.1:8001/",
                   "extension_group": "test extension group",
                   "extension": "extension 2"
                 },{
                   "app": "msgpack://127.0.0.1:8001/",
                   "extension_group": "test extension group",
                   "extension": "extension 3"
                 }]
               }]
             }]
           }
         })"_json);
  ten_test::check_status_code_is(resp, TEN_STATUS_CODE_OK);

  // Send a user-defined 'dispatch_data' command.
  resp = client->send_json_and_recv_resp_in_json(
      R"({
           "_ten": {
             "name": "dispatch_data",
             "seq_id": "137",
             "dest": [{
               "app": "msgpack://127.0.0.1:8001/",
               "extension_group": "test extension group",
               "extension": "extension 1"
             }]
           }
         })"_json);
  ten_test::check_result_is(resp, "137", TEN_STATUS_CODE_OK, "done");

  resp = client->send_json_and_recv_resp_in_json(
      R"({
           "_ten": {
             "name": "check_received",
             "seq_id": "137",
             "dest": [{
               "app": "msgpack://127.0.0.1:8001/",
               "extension_group": "test extension group",
               "extension": "extension 2"
             }]
           }
         })"_json);
  ten_test::check_result_is(resp, "137", TEN_STATUS_CODE_OK,
                            "received confirmed");

  resp = client->send_json_and_recv_resp_in_json(
      R"({
           "_ten": {
             "name": "check_received",
             "seq_id": "137",
             "dest": [{
               "app": "msgpack://127.0.0.1:8001/",
               "extension_group": "test extension group",
               "extension": "extension 3"
             }]
           }
         })"_json);
  ten_test::check_result_is(resp, "137", TEN_STATUS_CODE_OK,
                            "received confirmed");

  delete client;

  ten_thread_join(app_thread, -1);
}