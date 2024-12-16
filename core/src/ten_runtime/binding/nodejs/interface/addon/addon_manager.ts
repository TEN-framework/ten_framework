//
// Copyright © 2024 Agora
// This file is part of TEN Framework, an open source project.
// Licensed under the Apache License, Version 2.0, with certain conditions.
// Refer to the "LICENSE" file in the root directory for more information.
//

import { Addon } from "./addon";
import ten_addon from "../ten_addon";

type Ctor<T> = {
    new(): T;
    prototype: T;
}

type addonRegisterHandler = (registerContext: any) => void;

export class AddonManager {
    private static _registry: Map<string, addonRegisterHandler> = new Map();

    static _set_register_handler(name: string, handler: addonRegisterHandler) {
        AddonManager._registry.set(name, handler);
    }


}


export function RegisterAddonAsExtension(
    name: string
): <T extends Ctor<Addon>>(klass: T) => T {
    return function <T extends Ctor<Addon>>(klass: T): T {

        function registerHandler(registerContext: any) {
            ten_addon.ten_nodejs_register_addon_as_extension(
                name,
                klass,
            );
        }

        AddonManager._set_register_handler(name, registerHandler);

        return klass;
    };
}