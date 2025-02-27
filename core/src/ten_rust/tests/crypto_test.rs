//
// Copyright © 2025 Agora
// This file is part of TEN Framework, an open source project.
// Licensed under the Apache License, Version 2.0, with certain conditions.
// Refer to the "LICENSE" file in the root directory for more information.
//

use ten_rust::crypto;

#[test]
fn test_aes_ctr_crypto() {
    let origin_str = "hello world";
    let key_str = "0123456789012345";
    let nonce_str = "0123456789012345";
    let mut data_vec = origin_str.as_bytes().to_vec();
    let mut algorithm = crypto::create_encryption_algorithm(
        "AES-CTR",
        &format!("{{\"key\": \"{}\", \"nonce\": \"{}\"}}", key_str, nonce_str),
    )
    .unwrap();
    algorithm.encrypt(&mut data_vec);
    println!("data: {:?}", data_vec);

    // decrypt
    algorithm.encrypt(&mut data_vec);

    let data_str = String::from_utf8(data_vec).unwrap();
    println!("data: {:?}", data_str);

    assert_eq!(data_str, origin_str);
}

#[test]
fn test_aes_ctr_decrypt() {
    // 密文的十六进制字符串
    let hex_str = "37189CD37F16FB259496C99C03A93E2F";

    // 将十六进制字符串转换为字节数组
    let mut decrypt_bytes = Vec::new();
    for i in 0..(hex_str.len() / 2) {
        let byte_str = &hex_str[i * 2..i * 2 + 2];
        let byte = u8::from_str_radix(byte_str, 16).expect("无效的十六进制字符串");
        decrypt_bytes.push(byte);
    }

    let key_str = "0123456789012345";
    let nonce_str = "0123456789012345";
    let mut data_vec = decrypt_bytes;
    let mut algorithm = crypto::create_encryption_algorithm(
        "AES-CTR",
        &format!("{{\"key\": \"{}\", \"nonce\": \"{}\"}}", key_str, nonce_str),
    )
    .unwrap();
    algorithm.encrypt(&mut data_vec);

    // 将解密后的字节转换为字符串
    let data_str = String::from_utf8(data_vec).unwrap_or_else(|e| {
        println!("解密后的数据不是有效的UTF-8字符串: {}", e);
        String::new()
    });
    println!("解密后的数据: {}", data_str);
}
