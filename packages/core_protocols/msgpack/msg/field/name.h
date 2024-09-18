//
// This file is part of TEN Framework, an open source project.
// Licensed under the Apache License, Version 2.0.
// See the LICENSE file for more information.
//
#pragma once

#include "ten_runtime/ten_config.h"

#include "msgpack/pack.h"
#include "msgpack/unpack.h"

typedef struct ten_msg_t ten_msg_t;

TEN_RUNTIME_PRIVATE_API void ten_msgpack_msg_name_serialize(
    ten_msg_t *self, msgpack_packer *pck);

TEN_RUNTIME_PRIVATE_API bool ten_msgpack_msg_name_deserialize(
    ten_msg_t *self, msgpack_unpacker *unpacker, msgpack_unpacked *unpacked);
