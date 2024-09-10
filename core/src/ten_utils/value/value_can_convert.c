//
// This file is part of the TEN Framework project.
// See https://github.com/TEN-framework/ten_framework/LICENSE for license
// information.
//
#include <float.h>
#include <limits.h>
#include <math.h>
#include <stdbool.h>
#include <stdint.h>

// The range of int8 is from INT8_MIN (-128) ~ INT8_MAX (127). So the range of
// floating number which could fit into int8 is '-128.0 ~ 127.xx'.
// - 2^8 == 128, so if a floating point number is less than (not equal to)
//   128.0, the result of casting it to int8_t would be fit into int8_t.
// - -2^8 == -128, so if a floating point number is larger than or equal to
//   128.0, the result of casting it to int8_t would be fit into int8_t.
bool can_convert_float64_to_int8(double a) {
  return (a >= -ldexp(1.0, 7) && a < ldexp(1.0, 7));
}

// -32768 ~ 32767 => -32768.0 ~ 32767.xx
bool can_convert_float64_to_int16(double a) {
  return (a >= -ldexp(1.0, 15) && a < ldexp(1.0, 15));
}

bool can_convert_float64_to_int32(double a) {
  return (a >= -ldexp(1.0, 31) && a < ldexp(1.0, 31));
}

bool can_convert_float64_to_int64(double a) {
  return (a >= -ldexp(1.0, 63) && a < ldexp(1.0, 63));
}

bool can_convert_float64_to_uint8(double a) {
  return (a >= 0.0 && a < ldexp(1.0, 8));
}

bool can_convert_float64_to_uint16(double a) {
  return (a >= 0.0 && a < ldexp(1.0, 16));
}

bool can_convert_float64_to_uint32(double a) {
  return (a >= 0.0 && a < ldexp(1.0, 32));
}

bool can_convert_float64_to_uint64(double a) {
  return (a >= 0.0 && a < ldexp(1.0, 64));
}

bool can_convert_int32_to_float32(int32_t value) {
  if (value > (int32_t)FLT_MAX || value < -(int32_t)FLT_MAX) {
    return false;
  }

  float float_value = (float)value;
  int32_t converted_back = (int32_t)float_value;

  return value == converted_back;
}

bool can_convert_int64_to_float32(int64_t value) {
  if (value > (int64_t)FLT_MAX || value < -(int64_t)FLT_MAX) {
    return false;
  }

  float float_value = (float)value;
  int64_t converted_back = (int64_t)float_value;

  return value == converted_back;
}

bool can_convert_uint32_to_float32(uint32_t value) {
  if (value > (uint32_t)FLT_MAX) {
    return false;
  }

  float float_value = (float)value;
  uint32_t converted_back = (uint32_t)float_value;

  return value == converted_back;
}

bool can_convert_uint64_to_float32(uint64_t value) {
  if (value > (uint64_t)FLT_MAX) {
    return false;
  }

  float float_value = (float)value;
  uint64_t converted_back = (uint64_t)float_value;

  return value == converted_back;
}

bool can_convert_int32_to_float64(int32_t value) {
  double double_value = (double)value;
  int32_t converted_back = (int32_t)double_value;
  return value == converted_back;
}

bool can_convert_int64_to_float64(int64_t value) {
  double double_value = (double)value;
  int64_t converted_back = (int64_t)double_value;
  return value == converted_back;
}

bool can_convert_uint32_to_float64(uint32_t value) {
  double double_value = (double)value;
  uint32_t converted_back = (uint32_t)double_value;
  return value == converted_back;
}

bool can_convert_uint64_to_float64(uint64_t value) {
  double double_value = (double)value;
  uint64_t converted_back = (uint64_t)double_value;
  return value == converted_back;
}
