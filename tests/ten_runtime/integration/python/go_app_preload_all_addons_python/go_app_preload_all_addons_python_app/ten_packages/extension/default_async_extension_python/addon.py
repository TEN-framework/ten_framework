#
# This file is part of TEN Framework, an open source project.
# Licensed under the Apache License, Version 2.0.
# See the LICENSE file for more information.
#
import time
from ten import (
    Addon,
    register_addon_as_extension,
    TenEnv,
)
from .extension import DefaultAsyncExtension

# Sleep 3 seconds to mock the long import time of the addon.
time.sleep(3)
print("default_extension_python addon loaded")

@register_addon_as_extension("default_async_extension_python")
class DefaultAsyncExtensionAddon(Addon):
    def on_create_instance(self, ten_env: TenEnv, name: str, context) -> None:
        ten_env.log_info("on_create_instance")
        ten_env.on_create_instance_done(DefaultAsyncExtension(name), context)
