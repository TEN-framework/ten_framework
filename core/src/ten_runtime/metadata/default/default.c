//
// This file is part of the TEN Framework project.
// See https://github.com/TEN-framework/ten_framework/LICENSE for license
// information.
//
#include "include_internal/ten_runtime/metadata/metadata.h"
#include "ten_utils/lib/error.h"
#include "ten_utils/lib/path.h"

void ten_set_default_manifest_info(const char *base_dir,
                                   ten_metadata_info_t *manifest,
                                   TEN_UNUSED ten_error_t *err) {
  TEN_ASSERT(manifest, "Should not happen.");

  if (!base_dir || !strlen(base_dir)) {
    return;
  }

  ten_string_t manifest_json_file_path;
  ten_string_init_formatted(&manifest_json_file_path, "%s/manifest.json",
                            base_dir);
  ten_path_to_system_flavor(&manifest_json_file_path);
  if (ten_path_exists(ten_string_get_raw_str(&manifest_json_file_path))) {
    ten_metadata_info_set(manifest, TEN_METADATA_JSON_FILENAME,
                          ten_string_get_raw_str(&manifest_json_file_path));
  }
  ten_string_deinit(&manifest_json_file_path);
}

void ten_set_default_property_info(const char *base_dir,
                                   ten_metadata_info_t *property,
                                   TEN_UNUSED ten_error_t *err) {
  TEN_ASSERT(property, "Should not happen.");

  if (!base_dir || !strlen(base_dir)) {
    return;
  }

  ten_string_t property_json_file_path;
  ten_string_init_formatted(&property_json_file_path, "%s/property.json",
                            base_dir);
  ten_path_to_system_flavor(&property_json_file_path);
  if (ten_path_exists(ten_string_get_raw_str(&property_json_file_path))) {
    ten_metadata_info_set(property, TEN_METADATA_JSON_FILENAME,
                          ten_string_get_raw_str(&property_json_file_path));
  }
  ten_string_deinit(&property_json_file_path);
}
