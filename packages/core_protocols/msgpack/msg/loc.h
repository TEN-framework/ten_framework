//
// This file is part of TEN Framework, an open source project.
// Licensed under the Apache License, Version 2.0.
// See the LICENSE file for more information.
//
#pragma once

#include "ten_runtime/ten_config.h"

#include <assert.h>
#include <stdbool.h>

#include "include_internal/ten_runtime/common/loc.h"
#include "ten_utils/lib/string.h"

typedef struct msgpack_packer msgpack_packer;
typedef struct msgpack_unpacker msgpack_unpacker;
typedef struct msgpack_unpacked msgpack_unpacked;

TEN_RUNTIME_PRIVATE_API void ten_msgpack_loc_serialize(ten_loc_t *self,
                                                       msgpack_packer *pck);

TEN_RUNTIME_PRIVATE_API void ten_msgpack_loc_deserialize(
    ten_loc_t *self, msgpack_unpacker *unpacker, msgpack_unpacked *unpacked);

TEN_RUNTIME_PRIVATE_API void ten_msgpack_loc_list_serialize(
    ten_list_t *self, msgpack_packer *pck);

TEN_RUNTIME_PRIVATE_API ten_list_t ten_msgpack_loc_list_deserialize(
    msgpack_unpacker *unpacker, msgpack_unpacked *unpacked);
