//
// Copyright © 2024 Agora
// This file is part of TEN Framework, an open source project.
// Licensed under the Apache License, Version 2.0, with certain conditions.
// Refer to https://github.com/TEN-framework/ten_framework/LICENSE for more
// information.
//
macro_rules! tman_verbose_println {
    ($tman_config:expr, $($arg:tt)*) => {
        if $tman_config.verbose {
            println!($($arg)*);
        }
    };
}

pub(crate) use tman_verbose_println;
