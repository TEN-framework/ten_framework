//
// Copyright © 2025 Agora
// This file is part of TEN Framework, an open source project.
// Licensed under the Apache License, Version 2.0, with certain conditions.
// Refer to the "LICENSE" file in the root directory for more information.
//
#include "include_internal/ten_runtime/app/close.h"

#include "include_internal/ten_runtime/app/app.h"
#include "include_internal/ten_runtime/app/endpoint.h"
#include "include_internal/ten_runtime/app/engine_interface.h"
#include "include_internal/ten_runtime/app/msg_interface/common.h"
#include "include_internal/ten_runtime/connection/connection.h"
#include "include_internal/ten_runtime/engine/engine.h"
#include "include_internal/ten_runtime/engine/msg_interface/common.h"
#include "include_internal/ten_runtime/msg/msg.h"
#include "include_internal/ten_runtime/protocol/protocol.h"
#include "ten_runtime/app/app.h"
#include "ten_runtime/msg/cmd_result/cmd_result.h"
#include "ten_utils/io/runloop.h"
#include "ten_utils/lib/smart_ptr.h"
#include "ten_utils/lib/string.h"
#include "ten_utils/macro/check.h"
#include "ten_utils/macro/mark.h"
#include "ten_utils/sanitizer/thread_check.h"

static bool ten_app_has_no_work(ten_app_t *self) {
  TEN_ASSERT(self && ten_app_check_integrity(self, true), "Should not happen.");

  if (ten_list_is_empty(&self->engines) &&
      ten_list_is_empty(&self->orphan_connections)) {
    return true;
  }

  return false;
}

static bool ten_app_could_be_close(ten_app_t *self) {
  TEN_ASSERT(self && ten_app_check_integrity(self, true), "Should not happen.");

  if (ten_app_has_no_work(self) && ten_app_is_endpoint_closed(self)) {
    return true;
  }

  return false;
}

static void ten_app_proceed_to_close(ten_app_t *self) {
  TEN_ASSERT(self && ten_app_check_integrity(self, true), "Should not happen.");

  if (!ten_app_could_be_close(self)) {
    TEN_LOGD("[%s] Could not close alive app.", ten_app_get_uri(self));
    return;
  }
  TEN_LOGD("[%s] Close app.", ten_app_get_uri(self));

  ten_app_on_deinit(self);
}

static void ten_app_close_sync(ten_app_t *self) {
  TEN_ASSERT(self && ten_app_check_integrity(self, true), "Should not happen.");

  TEN_LOGD("[%s] Try to close app.", ten_app_get_uri(self));

  ten_list_foreach (&self->engines, iter) {
    ten_engine_close_async(ten_ptr_listnode_get(iter.node));
  }

  ten_list_foreach (&self->orphan_connections, iter) {
    ten_connection_close(ten_ptr_listnode_get(iter.node));
  }

  if (self->endpoint_protocol) {
    ten_protocol_close(self->endpoint_protocol);
  }
}

static void ten_app_close_task(void *app_, TEN_UNUSED void *arg) {
  ten_app_t *app = (ten_app_t *)app_;
  TEN_ASSERT(app_ && ten_app_check_integrity(app_, true), "Should not happen.");

  // The app might be closed due to the problems during creation, ex: some
  // property is invalid. And all resources have not been created yet.
  if (ten_app_could_be_close(app)) {
    ten_app_proceed_to_close(app);
    return;
  }

  ten_app_close_sync(app);
}

bool ten_app_close(ten_app_t *self, TEN_UNUSED ten_error_t *err) {
  // TEN_NOLINTNEXTLINE(thread-check)
  // thread-check: this function is used to be called in threads other than the
  // app thread, so the whole function is protected by a lock.
  TEN_ASSERT(self && ten_app_check_integrity(self, false),
             "Should not happen.");

  ten_mutex_lock(self->state_lock);

  if (self->state >= TEN_APP_STATE_CLOSING) {
    TEN_LOGD("[%s] App has been signaled to close.", ten_app_get_uri(self));
    goto done;
  }

  TEN_LOGD("[%s] Try to close app.", ten_app_get_uri(self));

  self->state = TEN_APP_STATE_CLOSING;

  int rc = ten_runloop_post_task_tail(ten_app_get_attached_runloop(self),
                                      ten_app_close_task, self, NULL);
  TEN_ASSERT(!rc, "Should not happen.");

done:
  ten_mutex_unlock(self->state_lock);
  return true;
}

bool ten_app_is_closing(ten_app_t *self) {
  // TEN_NOLINTNEXTLINE(thread-check)
  // thread-check: this function is used to be called in threads other than the
  // app thread, so the whole function is protected by a lock.
  TEN_ASSERT(self && ten_app_check_integrity(self, false),
             "Should not happen.");

  bool is_closing = false;

  ten_mutex_lock(self->state_lock);
  is_closing = self->state >= TEN_APP_STATE_CLOSING ? true : false;
  ten_mutex_unlock(self->state_lock);

  return is_closing;
}

void ten_app_check_termination_when_engine_closed(ten_app_t *self,
                                                  ten_engine_t *engine) {
  TEN_ASSERT(self && ten_app_check_integrity(self, true), "Should not happen.");

  if (engine->has_own_loop) {
    // Wait for the engine thread to be reclaimed. Because the engine thread
    // should have been terminated, this operation should be very fast.
    TEN_LOGD("App waiting engine thread be reclaimed.");

    TEN_UNUSED int rc = ten_thread_join(
        ten_sanitizer_thread_check_get_belonging_thread(&engine->thread_check),
        -1);
    TEN_ASSERT(!rc, "Should not happen.");

    TEN_LOGD(
        "Engine thread is reclaimed, and after this point, modify fields of "
        "'engine' is safe.");
  }

  if (engine->cmd_stop_graph != NULL) {
    const char *src_graph_id = ten_msg_get_src_graph_id(engine->cmd_stop_graph);
    if (!ten_string_is_equal_c_str(&engine->graph_id, src_graph_id)) {
      // This engine is _not_ suicidal.

      ten_shared_ptr_t *ret_cmd = ten_cmd_result_create_from_cmd(
          TEN_STATUS_CODE_OK, engine->cmd_stop_graph);
      ten_msg_set_property(ret_cmd, "detail",
                           ten_value_create_string("close engine done"), NULL);

      ten_app_push_to_in_msgs_queue(self, ret_cmd);

      ten_shared_ptr_destroy(ret_cmd);
    }
  }

  ten_app_del_engine(self, engine);

  // In the case of the engine has its own thread, the engine thread has already
  // been reclaimed now, so it's safe to destroy the engine object here.
  ten_engine_destroy(engine);

  if (self->long_running_mode) {
    TEN_LOGD("[%s] Don't close App due to it's in long running mode.",
             ten_app_get_uri(self));
  } else {
    // Here, we do not rely on whether the app has any remaining orphan
    // connections to decide whether to shut it down. This is because if a bad
    // client connects to the app but does not perform any subsequent actions,
    // such as failing to send a `start_graph` command, the orphan connection
    // will remain indefinitely. Consequently, if the decision to close the app
    // is based on whether there are orphan connections, the app may never be
    // able to shut down. If you want to prevent the app from terminating itself
    // simply because there are no engines at a certain moment, you can use the
    // app's `long_running_mode` mechanism. Once `long_running_mode` is enabled,
    // the app must be explicitly closed using the `close_app` command. This is
    // both normal and reasonable behavior.
    if (ten_list_is_empty(&self->engines)) {
      ten_app_close(self, NULL);
    }
  }

  if (ten_app_is_closing(self)) {
    ten_app_proceed_to_close(self);
  }
}

void ten_app_on_orphan_connection_closed(ten_connection_t *connection,
                                         TEN_UNUSED void *on_closed_data) {
  TEN_ASSERT(connection && ten_connection_check_integrity(connection, true),
             "Should not happen.");

  ten_app_t *self = connection->attached_target.app;
  TEN_ASSERT(self && ten_app_check_integrity(self, true), "Should not happen.");

  TEN_LOGD("[%s] Orphan connection %p closed", ten_app_get_uri(self),
           connection);

  ten_app_del_orphan_connection(self, connection);
  ten_connection_destroy(connection);

  // Check if the app is in the closing phase.
  if (ten_app_is_closing(self)) {
    TEN_LOGD("[%s] App is closing, check to see if it could proceed.",
             ten_app_get_uri(self));
    ten_app_proceed_to_close(self);
  } else {
    // If 'connection' is an orphan connection, it means the connection is not
    // attached to an engine, and the TEN app should _not_ be closed due to an
    // strange connection like this, otherwise, the TEN app will be very
    // fragile, anyone could simply connect to the TEN app, and close the app
    // through disconnection.
  }
}

void ten_app_on_protocol_closed(TEN_UNUSED ten_protocol_t *protocol,
                                void *on_closed_data) {
  ten_app_t *self = (ten_app_t *)on_closed_data;
  TEN_ASSERT(self && ten_app_check_integrity(self, true), "Should not happen.");

  if (ten_app_is_closing(self)) {
    ten_app_proceed_to_close(self);
  }
}
