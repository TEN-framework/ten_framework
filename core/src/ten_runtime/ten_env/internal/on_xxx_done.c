//
// This file is part of the TEN Framework project.
// See https://github.com/TEN-framework/ten_framework/LICENSE for license
// information.
//
#include "ten_runtime/ten_env/internal/on_xxx_done.h"

#include "include_internal/ten_runtime/addon/ten_env/on_xxx.h"
#include "include_internal/ten_runtime/app/app.h"
#include "include_internal/ten_runtime/engine/on_xxx.h"
#include "include_internal/ten_runtime/extension/extension.h"
#include "include_internal/ten_runtime/extension/on_xxx.h"
#include "include_internal/ten_runtime/extension_context/ten_env/on_xxx.h"
#include "include_internal/ten_runtime/extension_group/extension_group.h"
#include "include_internal/ten_runtime/extension_group/on_xxx.h"
#include "include_internal/ten_runtime/metadata/metadata_info.h"
#include "include_internal/ten_runtime/ten_env/ten_env.h"
#include "ten_utils/macro/check.h"
#include "ten_runtime/ten_env/ten_env.h"
#include "ten_utils/macro/mark.h"

bool ten_env_on_init_done(ten_env_t *self, ten_error_t *err) {
  TEN_ASSERT(self, "Invalid argument.");
  TEN_ASSERT(
      ten_env_check_integrity(
          self, self->attach_to != TEN_ENV_ATTACH_TO_ADDON ? true : false),
      "Invalid use of ten_env %p.", self);

  switch (self->attach_to) {
    case TEN_ENV_ATTACH_TO_EXTENSION:
      ten_extension_on_init_done(self);
      break;

    case TEN_ENV_ATTACH_TO_EXTENSION_GROUP:
      ten_extension_group_on_init_done(self);
      break;

    case TEN_ENV_ATTACH_TO_APP:
      ten_app_on_init_done(self);
      break;

    case TEN_ENV_ATTACH_TO_ADDON:
      ten_addon_on_init_done(self);
      break;

    default:
      TEN_ASSERT(0, "Should not happen.");
      break;
  }

  return true;
}

bool ten_env_on_deinit_done(ten_env_t *self, TEN_UNUSED ten_error_t *err) {
  TEN_ASSERT(self, "Invalid argument.");
  TEN_ASSERT(
      ten_env_check_integrity(
          self, self->attach_to != TEN_ENV_ATTACH_TO_ADDON ? true : false),
      "Invalid use of ten_env %p.", self);

  switch (self->attach_to) {
    case TEN_ENV_ATTACH_TO_ADDON:
      ten_addon_on_deinit_done(self);
      break;

    case TEN_ENV_ATTACH_TO_EXTENSION_GROUP:
      ten_extension_group_on_deinit_done(self);
      break;

    case TEN_ENV_ATTACH_TO_EXTENSION:
      ten_extension_on_deinit_done(self);
      break;

    case TEN_ENV_ATTACH_TO_APP:
      ten_app_on_deinit_done(self);
      break;

    default:
      TEN_ASSERT(0, "Should not happen.");
      break;
  }

  return true;
}

bool ten_env_on_create_extensions_done(ten_env_t *self, ten_list_t *extensions,
                                       TEN_UNUSED ten_error_t *err) {
  TEN_ASSERT(self, "Invalid argument.");
  TEN_ASSERT(ten_env_check_integrity(self, true), "Invalid use of ten_env %p.",
             self);
  TEN_ASSERT(self->attach_to == TEN_ENV_ATTACH_TO_EXTENSION_GROUP,
             "Should not happen.");

  ten_extension_group_t *extension_group =
      ten_env_get_attached_extension_group(self);
  TEN_ASSERT(extension_group &&
                 ten_extension_group_check_integrity(extension_group, true),
             "Should not happen.");

  ten_extension_group_on_create_extensions_done(extension_group, extensions);

  return true;
}

bool ten_env_on_destroy_extensions_done(ten_env_t *self,
                                        TEN_UNUSED ten_error_t *err) {
  TEN_ASSERT(self, "Invalid argument.");
  TEN_ASSERT(ten_env_check_integrity(self, true), "Invalid use of ten_env %p.",
             self);
  TEN_ASSERT(self->attach_to == TEN_ENV_ATTACH_TO_EXTENSION_GROUP,
             "Should not happen.");

  ten_extension_group_t *extension_group =
      ten_env_get_attached_extension_group(self);
  TEN_ASSERT(extension_group &&
                 ten_extension_group_check_integrity(extension_group, true),
             "Should not happen.");

  ten_extension_group_on_destroy_extensions_done(extension_group);

  return true;
}

bool ten_env_on_create_instance_done(ten_env_t *self, void *instance,
                                     void *context,
                                     TEN_UNUSED ten_error_t *err) {
  TEN_ASSERT(self, "Invalid argument.");
  TEN_ASSERT(
      ten_env_check_integrity(
          self, self->attach_to != TEN_ENV_ATTACH_TO_ADDON ? true : false),
      "Invalid use of ten_env %p.", self);

  switch (self->attach_to) {
    case TEN_ENV_ATTACH_TO_ADDON:
      ten_addon_on_create_instance_done(self, instance, context);
      break;

    default:
      TEN_ASSERT(0, "Should not happen.");
      break;
  }

  return true;
}

bool ten_env_on_destroy_instance_done(ten_env_t *self, void *context,
                                      TEN_UNUSED ten_error_t *err) {
  TEN_ASSERT(self, "Invalid argument.");
  TEN_ASSERT(
      ten_env_check_integrity(
          self, self->attach_to != TEN_ENV_ATTACH_TO_ADDON ? true : false),
      "Invalid use of ten_env %p.", self);

  switch (self->attach_to) {
    case TEN_ENV_ATTACH_TO_ADDON:
      ten_addon_on_destroy_instance_done(self, context);
      break;

    case TEN_ENV_ATTACH_TO_ENGINE:
      ten_extension_context_on_addon_destroy_extension_group_done(self,
                                                                  context);
      break;

    case TEN_ENV_ATTACH_TO_EXTENSION_GROUP:
      ten_extension_group_on_addon_destroy_extension_done(self, context);
      break;

    default:
      TEN_ASSERT(0, "Should not happen.");
      break;
  }

  return true;
}

bool ten_env_on_start_done(ten_env_t *self, TEN_UNUSED ten_error_t *err) {
  TEN_ASSERT(self, "Invalid argument.");
  TEN_ASSERT(ten_env_check_integrity(self, true), "Invalid use of ten_env %p.",
             self);
  TEN_ASSERT(self->attach_to == TEN_ENV_ATTACH_TO_EXTENSION,
             "Should not happen.");

  ten_extension_on_start_done(self);

  return true;
}

bool ten_env_on_stop_done(ten_env_t *self, TEN_UNUSED ten_error_t *err) {
  TEN_ASSERT(self, "Invalid argument.");
  TEN_ASSERT(ten_env_check_integrity(self, true), "Invalid use of ten_env %p.",
             self);
  TEN_ASSERT(self->attach_to == TEN_ENV_ATTACH_TO_EXTENSION,
             "Should not happen.");

  ten_extension_on_stop_done(self);

  return true;
}
