//
// Copyright © 2024 Agora
// This file is part of TEN Framework, an open source project.
// Licensed under the Apache License, Version 2.0, with certain conditions.
// Refer to the "LICENSE" file in the root directory for more information.
//
#include "include_internal/ten_runtime/common/closeable.h"

#include "ten_utils/container/list.h"
#include "ten_utils/container/list_node.h"
#include "ten_utils/container/list_node_ptr.h"
#include "ten_utils/container/list_ptr.h"
#include "ten_utils/lib/signature.h"
#include "ten_utils/macro/check.h"
#include "ten_utils/macro/mark.h"
#include "ten_utils/macro/memory.h"
#include "ten_utils/sanitizer/thread_check.h"

bool ten_closeable_check_integrity(ten_closeable_t *self, bool thread_check) {
  TEN_ASSERT(self, "Invalid argument.");

  if (ten_signature_get(&self->signature) != TEN_CLOSEABLE_SIGNATURE) {
    return false;
  }

  if (thread_check) {
    return ten_sanitizer_thread_check_do_check(&self->thread_check);
  }

  return true;
}

static void ten_closeable_be_notified_resources_init(
    ten_closeable_be_notified_resources_t *self) {
  TEN_ASSERT(self, "Invalid argument.");

  ten_list_init(&self->on_closed_all_done_queue);
}

void ten_closeable_init(ten_closeable_t *self, ptrdiff_t offset) {
  TEN_ASSERT(self, "Invalid argument.");

  ten_signature_set(&self->signature, (ten_signature_t)TEN_CLOSEABLE_SIGNATURE);
  ten_sanitizer_thread_check_init_with_current_thread(&self->thread_check);

  self->offset_in_impl = offset;

  ten_closeable_be_notified_resources_init(&self->be_notified_resources);
}

static void ten_closeable_be_notified_resources_deinit(
    ten_closeable_be_notified_resources_t *self) {
  TEN_ASSERT(self, "Invalid argument.");

  ten_list_clear(&self->on_closed_all_done_queue);
}

void ten_closeable_deinit(ten_closeable_t *self) {
  // TEN_NOLINTNEXTLINE(thread-check)
  // thread-check: The belonging thread (i.e., ten_thread_t object) maybe
  // already destroyed, so do not check thread integrity.
  TEN_ASSERT(self && ten_closeable_check_integrity(self, false),
             "Invalid argument.");

  ten_signature_set(&self->signature, 0);
  ten_sanitizer_thread_check_deinit(&self->thread_check);

  ten_closeable_be_notified_resources_deinit(&self->be_notified_resources);
}

static void ten_closeable_on_closed_all_done(ten_closeable_t *self) {
  TEN_ASSERT(self, "Should not happen.");
  TEN_ASSERT(ten_closeable_check_integrity(self, true), "Invalid argument.");

  ten_list_t on_closed_all_done_queue = TEN_LIST_INIT_VAL;

  ten_list_swap(&on_closed_all_done_queue,
                &self->be_notified_resources.on_closed_all_done_queue);

  ten_list_foreach (&on_closed_all_done_queue, iter) {
    ten_closeable_on_closed_all_done_item_t *item =
        ten_ptr_listnode_get(iter.node);
    TEN_ASSERT(item && item->on_closed_all_done_cb, "Should not happen.");

    item->on_closed_all_done_cb(self, item->who_have_interest_on_me,
                                item->on_closed_all_done_data);
  };

  ten_list_clear(&on_closed_all_done_queue);
}

void ten_closeable_action_to_close_myself_done(
    ten_closeable_t *self, TEN_UNUSED void *on_close_myself_data) {
  TEN_ASSERT(self && ten_closeable_check_integrity(self, true),
             "Invalid argument.");

  ten_closeable_on_closed_all_done(self);
}

static void ten_closeable_on_closed_all_done_item_destroy(
    ten_closeable_on_closed_all_done_item_t *self) {
  TEN_ASSERT(self, "Invalid argument.");

  TEN_FREE(self);
}

void ten_closeable_add_be_notified(
    ten_closeable_t *self, void *who_have_interest_on_me,
    ten_closeable_on_closed_all_done_func_t on_closed_all_done_cb,
    void *on_closed_all_done_data) {
  TEN_ASSERT(self && ten_closeable_check_integrity(self, true),
             "Invalid argument.");

  if (on_closed_all_done_cb) {
    ten_closeable_on_closed_all_done_item_t *item =
        TEN_MALLOC(sizeof(ten_closeable_on_closed_all_done_item_t));
    TEN_ASSERT(item, "Failed to allocate memory.");

    item->who_have_interest_on_me = who_have_interest_on_me;
    item->on_closed_all_done_cb = on_closed_all_done_cb;
    item->on_closed_all_done_data = on_closed_all_done_data;

    ten_list_push_ptr_back(
        &self->be_notified_resources.on_closed_all_done_queue, item,
        (ten_ptr_listnode_destroy_func_t)
            ten_closeable_on_closed_all_done_item_destroy);
  }
}
