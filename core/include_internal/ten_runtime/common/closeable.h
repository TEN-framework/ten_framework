//
// Copyright © 2024 Agora
// This file is part of TEN Framework, an open source project.
// Licensed under the Apache License, Version 2.0, with certain conditions.
// Refer to the "LICENSE" file in the root directory for more information.
//
#pragma once

#include "ten_runtime/ten_config.h"

#include "ten_utils/container/list.h"
#include "ten_utils/lib/signature.h"
#include "ten_utils/sanitizer/thread_check.h"

#define TEN_CLOSEABLE_SIGNATURE 0x7263656c6f736564U

typedef struct ten_closeable_t ten_closeable_t;

/**
 * @brief This function is used to notify others (i.e., @a
 * who_have_interest_on_me) who are interested in the 'on_closed_all_done' event
 * of mine.
 *
 * @param self The instance who implements the 'ten_closeable_t' interface.
 * @param who_have_interest_on_me The instance who is interested in the
 * 'on_closed_all_done' event of mine.
 * @param on_closed_all_done_data The data bound to the 'on_closed_all_done'
 * callback.
 *
 * @note @a self might be destroyed by its owner after its owner receives the
 * 'on_closed_all_done' event, so it's not safe for others to access the memory
 * of @a self in this callback.
 */
typedef void (*ten_closeable_on_closed_all_done_func_t)(
    ten_closeable_t *self, void *who_have_interest_on_me,
    void *on_closed_all_done_data);

typedef struct ten_closeable_on_closed_all_done_item_t {
  void *who_have_interest_on_me;
  ten_closeable_on_closed_all_done_func_t on_closed_all_done_cb;
  void *on_closed_all_done_data;
} ten_closeable_on_closed_all_done_item_t;

/**
 * @brief This structure is used to collect information about which other
 * instances, as well as their interest in which closing events of a
 * ten_closeable_t instance.
 *
 * @note Someone is not always interested in all closing events of mine, so we
 * prefer to use three different queues here.
 */
typedef struct ten_closeable_be_notified_resources_t {
  // Is interested in 'closed_all_done' event. The type of the element is
  // 'ten_closeable_on_closed_all_done_item_t'.
  ten_list_t on_closed_all_done_queue;
} ten_closeable_be_notified_resources_t;

/**
 * @brief The standard interface for closing a resource.
 *
 * Basically, if A wants to be closed, there are some stages during the closing
 * flow.
 *
 * 1) Stage: Make a 'intend_to_close' announcement.
 *    In this stage, a 'ten_closeable_t' tries to announce the intend_to_close
 *    event to all others who are interested in this event.
 *
 * 2) Stage: Determine the closing root.
 *    In this stage, all possible closing roots will be determined.
 *
 * 3) Stage: Start to close.
 *    In this stage, the closing flow will be started from all roots.
 *
 * 4) Stage: Close owned 'ten_closeable_t' resources.
 *    In this stage, all owned 'ten_closeable_t' resources of a
 *    'ten_closeable_t' are closed.
 *
 * 5) Stage: Close owned non-'ten_closeable_t' resources.
 *    In this stage, all owned non-'ten_closeable_t' resources of a
 *    'ten_closeable_t' are closed.
 *
 * 6) Stage: On closed, make a 'closed' announcement.
 *    When a 'ten_closeable_t' is closed completed, it will notify others, who
 *    are interested in my 'closed' event, that I am closed.
 *
 * 7) Stage: All subscribers interested in the 'on_closed' event have completed
 *    handling the 'on_closed' event, then make a 'on_closed_all_done'
 *    announcement. In other words, when a 'ten_closeable_t' receives all the
 *    'on_closed_done' callbacks, it will notify others who are interested in my
 *    'on_closed_all_done' event. Please note that this 'ten_closeable_t' might
 *    be destroyed in this stage by its owner.
 */
typedef struct ten_closeable_t {
  ten_signature_t signature;

  // All operations _must_ be called in the same thread.
  ten_sanitizer_thread_check_t thread_check;

  // The offset of this 'ten_closeable_t' object in the implementation who
  // implements the 'ten_closeable_t' interface. It will be used to get the raw
  // pointer of the implementation.
  //
  // We want to use the following coding style to ensure that the
  // 'implementation' is paired one-to-one with 'ten_closeable_t'.
  //
  // - The 'ten_closeable_t' is an embedded member of 'implementation' rather
  //   than a pointer.
  //
  // Ex:
  //   struct some_impl_t {
  //     // other fields.
  //
  //     ten_closeable_t closeable;
  //
  //     // other fields.
  //   }
  //
  // The above example is similar to the following codes in other programming
  // language:
  //
  //   class some_impl_t implements ten_closeable_t;
  //
  // We want to represent the following two key points:
  //
  // 1. The implementation is a kind of 'ten_closeable_t'.
  //
  // 2. One 'ten_closeable_t' could only be paired with one implementation. The
  //    implementation is closing when its embedded 'ten_closeable_t' is
  //    closing; the implementation is closed when its embedded
  //    'ten_closeable_t' is closed.
  //
  // So we keep the offset rather than the raw pointer here.
  ptrdiff_t offset_in_impl;

  // This list is used to store those who are interested in my 'closed' events.
  ten_closeable_be_notified_resources_t be_notified_resources;
} ten_closeable_t;

TEN_RUNTIME_API bool ten_closeable_check_integrity(ten_closeable_t *self,
                                                   bool thread_check);

/**
 * @example An implementation whose name is 'some_impl_t' and the name of the
 * 'ten_closeable_t' field in it is 'closeable'. The calling of this function in
 * the implementation will be as follows:
 *
 * ten_closeable_init(&impl->closeable,
 *   offsetof(some_impl_t, closeable)))
 */
TEN_RUNTIME_API void ten_closeable_init(ten_closeable_t *self,
                                        ptrdiff_t offset);

TEN_RUNTIME_API void ten_closeable_deinit(ten_closeable_t *self);

/**
 * @note Add @a who_have_interest_on_me who is interested in my various closing
 * event to the closeable management of mine (i.e., @a self).
 *
 * @param self The instance who implements the 'ten_closeable_t' interface.
 * @param who_have_interest_on_me The instance who is interested in various
 * closing event of @a self.
 */
TEN_RUNTIME_API void ten_closeable_add_be_notified(
    ten_closeable_t *self, void *who_have_interest_on_me,
    // closed_all_done event.
    ten_closeable_on_closed_all_done_func_t on_closed_all_done_cb,
    void *on_closed_all_done_data);

TEN_RUNTIME_API void ten_closeable_action_to_close_myself_done(
    ten_closeable_t *self, void *on_close_myself_data);
