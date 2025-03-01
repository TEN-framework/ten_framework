//
// Copyright © 2025 Agora
// This file is part of TEN Framework, an open source project.
// Licensed under the Apache License, Version 2.0, with certain conditions.
// Refer to the "LICENSE" file in the root directory for more information.
//
#include <cstring>

#include "gtest/gtest.h"
#include "ten_utils/lib/json.h"

TEST(JsonTest, Create) {  // NOLINT
  ten_json_t *json = ten_json_create();
  EXPECT_NE(json, nullptr);
  ten_json_destroy(json);
}

TEST(JsonTest, Init1) {  // NOLINT
  ten_json_t json;
  ten_json_init(&json);
  ten_json_deinit(&json);
}

TEST(JsonTest, Init2) {  // NOLINT
  ten_json_t json = TEN_JSON_INIT_VAL;
  ten_json_deinit(&json);
}

TEST(JsonTest, FromString) {  // NOLINT
  ten_json_t *json = ten_json_from_string("{\"a\": 1}", nullptr);
  EXPECT_NE(json, nullptr);
  ten_json_destroy(json);
}

TEST(JsonTest, ObjectPeekString) {  // NOLINT
  ten_json_t *json = ten_json_from_string(R"({"a": "hello"})", nullptr);
  EXPECT_NE(json, nullptr);

  const char *a = ten_json_object_peek_string(json, "a");
  EXPECT_NE(a, nullptr);
  EXPECT_STREQ(a, "hello");

  ten_json_destroy(json);
}

TEST(JsonTest, ObjectPeekObject) {  // NOLINT
  ten_json_t *json = ten_json_from_string(R"({"a": {"b": "hello"}})", nullptr);
  EXPECT_NE(json, nullptr);

  ten_json_t a = TEN_JSON_INIT_VAL;
  bool success = ten_json_object_peek(json, "a", &a);
  EXPECT_TRUE(success);

  const char *b = ten_json_object_peek_string(&a, "b");
  EXPECT_NE(b, nullptr);
  EXPECT_STREQ(b, "hello");

  ten_json_deinit(&a);
  ten_json_destroy(json);
}

TEST(JsonTest, ObjectPeekObjectForcibly) {  // NOLINT
  ten_json_t *json = ten_json_from_string(R"({})", nullptr);
  EXPECT_NE(json, nullptr);

  ten_json_t a = TEN_JSON_INIT_VAL;
  bool success = ten_json_object_peek_object_forcibly(json, "a", &a);
  EXPECT_TRUE(success);

  ten_json_deinit(&a);
  ten_json_destroy(json);
}

TEST(JsonTest, ObjectForeach) {  // NOLINT
  ten_json_t *json =
      ten_json_from_string(R"({"a": "hello", "b": "world"})", nullptr);
  EXPECT_NE(json, nullptr);

  const char *key = nullptr;
  ten_json_t *item = nullptr;
  ten_json_object_foreach(json, key, item) {
    EXPECT_NE(key, nullptr);
    EXPECT_NE(item, nullptr);
    if (ten_c_string_is_equal(key, "a")) {
      EXPECT_STREQ(ten_json_peek_string_value(item), "hello");
    } else if (ten_c_string_is_equal(key, "b")) {
      EXPECT_STREQ(ten_json_peek_string_value(item), "world");
    }
  }

  ten_json_destroy(json);
}

TEST(JsonTest, ArrayForeach) {  // NOLINT
  ten_json_t *json = ten_json_from_string(R"(["a", "hello"])", nullptr);
  EXPECT_NE(json, nullptr);

  size_t i = 0;
  ten_json_t *item = nullptr;
  ten_json_array_foreach(json, i, item) {
    if (i == 0) {
      EXPECT_STREQ(ten_json_peek_string_value(item), "a");
    } else if (i == 1) {
      EXPECT_STREQ(ten_json_peek_string_value(item), "hello");
    }
  }

  ten_json_destroy(json);
}

TEST(JsonTest, ObjectSetNew) {  // NOLINT
  ten_json_t *json = ten_json_create();
  EXPECT_NE(json, nullptr);

  ten_json_t *value = ten_json_create_string("hello");
  EXPECT_NE(value, nullptr);
  ten_json_object_set_new(json, "a", value);

  ten_json_destroy(value);
  ten_json_destroy(json);
}

TEST(JsonTest, ArrayAppendNew) {  // NOLINT
  ten_json_t *json = ten_json_create();
  EXPECT_NE(json, nullptr);

  ten_json_t *value = ten_json_create_string("hello");
  EXPECT_NE(value, nullptr);
  ten_json_array_append_new(json, value);

  ten_json_destroy(value);
  ten_json_destroy(json);
}
