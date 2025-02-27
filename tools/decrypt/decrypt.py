#!/usr/bin/env python3
# -*- coding: utf-8 -*-

import argparse
import os
from typing import Union, Optional
from pathlib import Path

from cryptography.hazmat.primitives.ciphers import Cipher, algorithms, modes
from cryptography.hazmat.backends import default_backend


def decrypt_file(
    input_file: Union[str, Path],
    output_file: Union[str, Path],
    key: Union[str, bytes],
    nonce: Union[str, bytes],
    nonce_length: Optional[int] = None,
) -> None:
    """
    使用AES-128-CTR模式解密文件

    参数:
        input_file: 要解密的源文件路径
        output_file: 解密后的目标文件路径
        key: AES密钥 (16字节)，字符串或字节格式
        nonce: CTR模式使用的nonce (通常8-12字节)，字符串或字节格式
        nonce_length: nonce占用的字节数，剩余部分用于计数器，默认为8
    """
    # 确保key和nonce是字节格式
    if isinstance(key, str):
        key = key.encode("utf-8")
    if isinstance(nonce, str):
        nonce = nonce.encode("utf-8")

    # 验证key长度
    if len(key) != 16:
        raise ValueError(f"密钥必须为16字节 (当前长度: {len(key)}字节)")

    # 确定nonce_length
    if nonce_length is None:
        if len(nonce) <= 8:
            nonce_length = len(nonce)
        else:
            nonce_length = 8  # 默认使用8字节作为nonce

    # 创建正确的初始计数器值
    if len(nonce) == 16:
        # 已经是完整的初始计数器值
        initial_value = nonce
    else:
        # 需要构建初始计数器值
        if len(nonce) < nonce_length:
            raise ValueError(
                f"Nonce长度({len(nonce)})小于指定的nonce_length({nonce_length})"
            )

        # 取nonce的前nonce_length字节
        real_nonce = nonce[:nonce_length]
        # 构建初始计数器值: nonce + 初始计数器(0)填充到16字节
        counter_part = b"\x00" * (16 - nonce_length)
        # 将memoryview转换为bytes，然后再连接
        initial_value = bytes(real_nonce) + counter_part

    # 创建AES-CTR解密器
    cipher = Cipher(
        algorithms.AES(key), modes.CTR(initial_value), backend=default_backend()
    )
    decryptor = cipher.decryptor()

    # 分批读取和解密数据
    buffer_size = 64 * 1024  # 64KB缓冲区

    with open(input_file, "rb") as in_file, open(output_file, "wb") as out_file:
        while True:
            data = in_file.read(buffer_size)
            if not data:
                break
            decrypted_data = decryptor.update(data)
            out_file.write(decrypted_data)

        # 完成解密过程
        out_file.write(decryptor.finalize())

    print(f"文件已成功解密: {output_file}")


def main() -> None:
    """解析命令行参数并执行解密操作"""
    parser = argparse.ArgumentParser(description="AES-128-CTR文件解密工具")
    parser.add_argument(
        "--input", "-i", required=True, help="要解密的源文件路径"
    )
    parser.add_argument(
        "--key", "-k", required=True, help="AES密钥 (16字节字符串)"
    )
    parser.add_argument(
        "--nonce", "-n", required=True, help="CTR模式的nonce字符串"
    )
    parser.add_argument(
        "--output", "-o", required=True, help="解密后的目标文件路径"
    )
    parser.add_argument(
        "--nonce-length",
        "-l",
        type=int,
        help="nonce占用的字节数，默认为8",
        default=8,
    )

    args = parser.parse_args()

    # 检查源文件是否存在
    if not os.path.exists(args.input):
        print(f"错误: 源文件 '{args.input}' 不存在")
        return

    # 确保目标文件目录存在
    output_dir = os.path.dirname(args.output)
    if output_dir and not os.path.exists(output_dir):
        os.makedirs(output_dir)

    try:
        decrypt_file(
            args.input, args.output, args.key, args.nonce, args.nonce_length
        )
    except Exception as e:
        print(f"解密过程中出错: {e}")


if __name__ == "__main__":
    main()
