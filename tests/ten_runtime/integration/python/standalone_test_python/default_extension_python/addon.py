#
# Copyright © 2024 Agora
# This file is part of TEN Framework, an open source project.
# Licensed under the Apache License, Version 2.0, with certain conditions.
# Refer to the "LICENSE" file in the root directory for more information.
#
from ten import (
    Addon,
    register_addon_as_extension,
    TenEnv,
)
from .extension import DefaultExtension
from .log import logger


@register_addon_as_extension("default_extension_python")
class DefaultExtensionAddon(Addon):

    def on_create_instance(self, ten_env: TenEnv, name: str, context) -> None:
        logger.info("DefaultExtensionAddon on_create_instance")
        ten_env.on_create_instance_done(DefaultExtension(name), context)