//
// Copyright © 2024 Agora
// This file is part of TEN Framework, an open source project.
// Licensed under the Apache License, Version 2.0, with certain conditions.
// Refer to the "LICENSE" file in the root directory for more information.
//
import ten_addon from '../ten_addon'

export class TenEnv {
    getPropertyToJson(path: string): string {
        return ten_addon.ten_nodejs_ten_env_get_property_to_json(path);
    }

    setPropertyFromJson(path: string, jsonStr: string): void {
        ten_addon.ten_nodejs_ten_env_set_property_from_json(path, jsonStr);
    }
}

ten_addon.ten_nodejs_ten_env_register_class(TenEnv);