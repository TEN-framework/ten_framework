#
# Copyright © 2024 Agora
# This file is part of TEN Framework, an open source project.
# Licensed under the Apache License, Version 2.0, with certain conditions.
# Refer to the "LICENSE" file in the root directory for more information.
#
from libten_runtime_python import _Msg


class Msg(_Msg):
    get_name = _Msg.get_name
    set_dest = _Msg.set_dest
    get_property_to_json = _Msg.get_property_to_json
    set_property_from_json = _Msg.set_property_from_json
    set_property_string = _Msg.set_property_string
    get_property_string = _Msg.get_property_string
    get_property_int = _Msg.get_property_int
    set_property_int = _Msg.set_property_int
    get_property_bool = _Msg.get_property_bool
    set_property_bool = _Msg.set_property_bool
    get_property_float = _Msg.get_property_float
    set_property_float = _Msg.set_property_float
    get_property_buf = _Msg.get_property_buf
    set_property_buf = _Msg.set_property_buf
