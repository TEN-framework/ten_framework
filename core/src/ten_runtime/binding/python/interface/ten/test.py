#
# Copyright © 2024 Agora
# This file is part of TEN Framework, an open source project.
# Licensed under the Apache License, Version 2.0, with certain conditions.
# Refer to the "LICENSE" file in the root directory for more information.
#
from libten_runtime_python import _ExtensionTester


class ExtensionTester(_ExtensionTester):
    def add_addon_base_dir(self, base_dir: str) -> None:
        return _ExtensionTester.add_addon_base_dir(self, base_dir)
