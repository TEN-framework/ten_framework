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
#include "ten_utils/lib/mutex.h"
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

static void ten_closeable_action_to_close_myself_init(
    ten_closeable_action_to_close_myself_t *self) {
  TEN_ASSERT(self, "Invalid argument.");

  self->action_to_close_myself_cb = NULL;
  self->action_to_close_myself_data = NULL;
}

static void ten_closeable_be_notified_resources_init(
    ten_closeable_be_notified_resources_t *self) {
  TEN_ASSERT(self, "Invalid argument.");

  self->on_closed_done_mode = TEN_CLOSEABLE_ON_CLOSED_DONE_MODE_IN_OWN_THREAD;
  self->on_closed_done_cb = NULL;

  self->expected_on_closed_done_count = 0;
  self->on_closed_done_mutex = NULL;

  if (self->on_closed_done_mode ==
      TEN_CLOSEABLE_ON_CLOSED_DONE_MODE_OUT_OWN_THREAD) {
    self->on_closed_done_mutex = ten_mutex_create();
  }

  ten_list_init(&self->on_intend_to_close_queue);
  ten_list_init(&self->on_closed_queue);
  ten_list_init(&self->on_closed_all_done_queue);
}

void ten_closeable_init(ten_closeable_t *self, ptrdiff_t offset) {
  TEN_ASSERT(self, "Invalid argument.");

  ten_signature_set(&self->signature, (ten_signature_t)TEN_CLOSEABLE_SIGNATURE);
  ten_sanitizer_thread_check_init_with_current_thread(&self->thread_check);

  self->state = TEN_CLOSEABLE_STATE_ALIVE;
  self->offset_in_impl = offset;

  ten_closeable_action_to_close_myself_init(&self->action_to_close_myself);

  ten_closeable_be_notified_resources_init(&self->be_notified_resources);
}

static void ten_closeable_be_notified_resources_deinit(
    ten_closeable_be_notified_resources_t *self) {
  TEN_ASSERT(self, "Invalid argument.");

  ten_list_clear(&self->on_intend_to_close_queue);
  ten_list_clear(&self->on_closed_queue);
  ten_list_clear(&self->on_closed_all_done_queue);

  if (self->on_closed_done_mutex) {
    ten_mutex_destroy(self->on_closed_done_mutex);
    self->on_closed_done_mutex = NULL;
  }
}

void ten_closeable_deinit(ten_closeable_t *self) {
  // TEN_NOLINTNEXTLINE(thread-check)
  // thread-check: The belonging thread (i.e., ten_thread_t object) maybe
  // already destroyed, so do not check thread integrity.
  TEN_ASSERT(self && ten_closeable_check_integrity(self, false),
             "Invalid argument.");

  ten_signature_set(&self->signature, 0);
  ten_sanitizer_thread_check_deinit(&self->thread_check);

  self->state = TEN_CLOSEABLE_STATE_ALIVE;

  ten_closeable_be_notified_resources_deinit(&self->be_notified_resources);
}

void ten_closeable_set_action_to_close_myself(
    ten_closeable_t *self,
    ten_closeable_action_to_close_myself_func_t action_to_close_myself_cb,
    void *action_to_close_myself_data) {
  TEN_ASSERT(self && ten_closeable_check_integrity(self, true),
             "Invalid argument.");

  self->action_to_close_myself.action_to_close_myself_cb =
      action_to_close_myself_cb;
  self->action_to_close_myself.action_to_close_myself_data =
      action_to_close_myself_data;
}

static void ten_closeable_on_closed_all_done(ten_closeable_t *self) {
  TEN_ASSERT(self, "Should not happen.");

  bool is_thread_safe = self->be_notified_resources.on_closed_done_mode ==
                        TEN_CLOSEABLE_ON_CLOSED_DONE_MODE_IN_OWN_THREAD;
  TEN_ASSERT(ten_closeable_check_integrity(self, is_thread_safe),
             "Invalid argument.");

  ten_list_t on_closed_all_done_queue = TEN_LIST_INIT_VAL;

  // Because on_closed_done() may be invoked in different threads than the one
  // where ten_closeable_t resides, it is necessary to determine whether to
  // perform lock/unlock operations based on thread safety principles.
  if (is_thread_safe) {
    ten_list_swap(&on_closed_all_done_queue,
                  &self->be_notified_resources.on_closed_all_done_queue);
  } else {
    TEN_ASSERT(self->be_notified_resources.on_closed_done_mutex,
               "Should not happen.");

    TEN_DO_WITH_MUTEX_LOCK(self->be_notified_resources.on_closed_done_mutex, {
      ten_list_swap(&on_closed_all_done_queue,
                    &self->be_notified_resources.on_closed_all_done_queue);
    });
  }

  ten_list_foreach (&on_closed_all_done_queue, iter) {
    ten_closeable_on_closed_all_done_item_t *item =
        ten_ptr_listnode_get(iter.node);
    TEN_ASSERT(item && item->on_closed_all_done_cb, "Should not happen.");

    item->on_closed_all_done_cb(self, item->who_have_interest_on_me,
                                item->on_closed_all_done_data);
  };

  ten_list_clear(&on_closed_all_done_queue);
}

static void ten_closeable_on_closed_done(ten_closeable_t *self,
                                         void *who_have_interest_on_me,
                                         void *on_closed_data) {
  TEN_ASSERT(self && ten_closeable_check_integrity(self, true),
             "Invalid argument.");

  ten_list_t *on_closed_queue = &self->be_notified_resources.on_closed_queue;

  bool found = false;
  ten_list_foreach (on_closed_queue, iter) {
    ten_closeable_on_closed_item_t *item = ten_ptr_listnode_get(iter.node);
    TEN_ASSERT(item && item->on_closed_cb, "Should not happen.");

    if (item->who_have_interest_on_me == who_have_interest_on_me &&
        item->on_closed_data == on_closed_data) {
      found = true;
      ten_list_remove_node(on_closed_queue, iter.node);

      break;
    }
  }

  TEN_ASSERT(found, "Should not happen.");

  if (ten_list_is_empty(on_closed_queue)) {
    // All other resources who are interested in my 'closed' event handle the
    // 'closed' event completely.
    ten_closeable_on_closed_all_done(self);
  }
}

static void ten_closeable_on_closed_done_out_of_thread(
    ten_closeable_t *self, TEN_UNUSED void *who_have_interest_on_me,
    TEN_UNUSED void *on_closed_data) {
  // TEN_NOLINTNEXTLINE(thread-check)
  // thread-check: This function is always called in threads other than the
  // thread where ten_closeable_t resides.
  TEN_ASSERT(self && ten_closeable_check_integrity(self, false),
             "Invalid argument.");
  TEN_ASSERT(self->be_notified_resources.on_closed_done_mode ==
                 TEN_CLOSEABLE_ON_CLOSED_DONE_MODE_OUT_OWN_THREAD,
             "Should not happen.");

  size_t remain_on_closed_done_count = 0;
  TEN_DO_WITH_MUTEX_LOCK(self->be_notified_resources.on_closed_done_mutex, {
    remain_on_closed_done_count =
        --self->be_notified_resources.expected_on_closed_done_count;
  });

  if (remain_on_closed_done_count == 0) {
    ten_closeable_on_closed_all_done(self);
  }
}

void ten_closeable_action_to_close_myself_done(
    ten_closeable_t *self, TEN_UNUSED void *on_close_myself_data) {
  TEN_ASSERT(self && ten_closeable_check_integrity(self, true),
             "Invalid argument.");

  self->state = TEN_CLOSEABLE_STATE_CLOSED;

  if (ten_list_is_empty(&self->be_notified_resources.on_closed_queue)) {
    ten_closeable_on_closed_all_done(self);
    return;
  }

  // Under different modes, the logic for determining the receipt of all
  // on_closed_done() is different.
  if (self->be_notified_resources.on_closed_done_mode ==
      TEN_CLOSEABLE_ON_CLOSED_DONE_MODE_OUT_OWN_THREAD) {
    self->be_notified_resources.expected_on_closed_done_count =
        ten_list_size(&self->be_notified_resources.on_closed_queue);

    self->be_notified_resources.on_closed_done_cb =
        ten_closeable_on_closed_done_out_of_thread;
  } else {
    self->be_notified_resources.on_closed_done_cb =
        ten_closeable_on_closed_done;
  }

  // Notify others who are interested in my 'closed' event.
  ten_list_foreach (&self->be_notified_resources.on_closed_queue, iter) {
    ten_closeable_on_closed_item_t *item = ten_ptr_listnode_get(iter.node);
    TEN_ASSERT(item && item->on_closed_cb, "Should not happen.");

    item->on_closed_cb(self, item->who_have_interest_on_me,
                       item->on_closed_data,
                       self->be_notified_resources.on_closed_done_cb);
  }
}

/**
 * @brief All the underlying resources are closed, so I could start to close
 * myself.
 */
static void ten_closeable_do_close(ten_closeable_t *self) {
  TEN_ASSERT(self && ten_closeable_check_integrity(self, true),
             "Invalid argument.");

  if (self->action_to_close_myself.action_to_close_myself_cb) {
    self->action_to_close_myself.action_to_close_myself_cb(
        self, self->action_to_close_myself.action_to_close_myself_data,
        ten_closeable_action_to_close_myself_done);
  } else {
    // There is no customized close_myself actions, so call
    // 'action_to_close_myself_done' directly.
    ten_closeable_action_to_close_myself_done(self, NULL);
  }
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

void ten_closeable_close(ten_closeable_t *self) {
  TEN_ASSERT(self && ten_closeable_check_integrity(self, true),
             "Invalid argument.");
  TEN_ASSERT(self->state < TEN_CLOSEABLE_STATE_CLOSING,
             "The 'close()' function could be called only once to ensure that "
             "all the resources are closed from top to bottom.");

  self->state = TEN_CLOSEABLE_STATE_CLOSING;

  ten_closeable_do_close(self);
}

bool ten_closeable_is_closed(ten_closeable_t *self) {
  TEN_ASSERT(self && ten_closeable_check_integrity(self, true),
             "Invalid argument.");

  return self->state == TEN_CLOSEABLE_STATE_CLOSED;
}
