#!/usr/bin/env python3
# -*- coding: utf-8 -*-

import argparse
import os
from typing import Union, Tuple
from pathlib import Path

from cryptography.hazmat.primitives.ciphers import Cipher, algorithms, modes
from cryptography.hazmat.backends import default_backend


def parse_log_header(data: bytes) -> Tuple[bool, int]:
    """
    解析单条日志的头部(5字节)

    Args:
        data: 包含头部的字节数据

    Returns:
        (是否有效, 数据长度)
    """
    if len(data) < 5:
        return False, 0

    # 检查魔数
    if data[0] != 0xFF or data[1] != 0xFF:
        return False, 0

    # 提取版本和校验位
    version = data[2] & 0x3F  # 前6位
    parity_bit = (data[2] >> 7) & 0x01  # 第8位

    # 计算实际校验值
    calculated_parity = 0
    for i in range(5):
        # 暂时将第8位清零再计算
        byte_val = data[i]
        if i == 2:
            byte_val &= 0x7F
        calculated_parity ^= byte_val

    calculated_parity &= 0x01

    # 检查校验位
    if parity_bit != calculated_parity:
        return False, 0

    # 获取数据长度
    data_len = (data[3] << 8) | data[4]

    return True, data_len


def create_decryptor(key: bytes, iv: bytes):
    """
    创建新的AES-CTR解密器

    Args:
        key: AES密钥(16字节)
        iv: CTR模式的初始化向量(16字节)

    Returns:
        新的解密器对象
    """
    cipher = Cipher(
        algorithms.AES(key), modes.CTR(iv), backend=default_backend()
    )
    return cipher.decryptor()


def decrypt_log_file(
    input_file: Union[str, Path],
    output_file: Union[str, Path],
    key: Union[str, bytes],
    iv: Union[str, bytes],
) -> None:
    """
    解密包含多条加密日志的文件，每条日志都有自己的5字节头部
    每条日志独立加密，需要为每条日志重新创建解密器

    Args:
        input_file: 要解密的源文件路径
        output_file: 解密后的目标文件路径
        key: AES密钥(16字节)，字符串或字节格式
        iv: CTR模式的初始化向量(16字节)，字符串或字节格式
    """
    # 确保密钥和IV为字节格式
    if isinstance(key, str):
        key = key.encode("utf-8")
    if isinstance(iv, str):
        iv = iv.encode("utf-8")

    # 验证密钥长度
    if len(key) != 16:
        raise ValueError(f"密钥必须为16字节(当前长度: {len(key)}字节)")

    # 验证IV长度
    if len(iv) != 16:
        raise ValueError(f"IV必须为16字节(当前长度: {len(iv)}字节)")

    # 打开输入和输出文件
    with open(input_file, "rb") as in_file, open(output_file, "wb") as out_file:
        log_count = 0
        total_bytes_decrypted = 0

        while True:
            # 读取头部(5字节)
            header = in_file.read(5)
            if not header or len(header) < 5:
                # 文件结束或数据不完整
                break

            valid, data_len = parse_log_header(header)
            if not valid:
                print(
                    f"警告: 在偏移量 {in_file.tell() - 5} 处发现无效的日志头部，尝试继续..."
                )
                # 尝试找到下一个可能的日志头部
                in_file.seek(
                    -4, os.SEEK_CUR
                )  # 回退4个字节，保留最后一个字节用于下一次检查
                continue

            # 读取单条日志的数据部分
            log_data = in_file.read(data_len)
            if len(log_data) < data_len:
                print(
                    f"警告: 日志数据不完整，预期 {data_len} 字节，实际 {len(log_data)} 字节"
                )

            if log_data:
                # 为每条日志创建新的解密器
                decryptor = create_decryptor(key, iv)

                # 解密这条日志
                decrypted_data = decryptor.update(log_data)
                final_data = decryptor.finalize()  # 确保处理所有数据

                if final_data:
                    decrypted_data += final_data

                out_file.write(decrypted_data)
                total_bytes_decrypted += len(decrypted_data)
                log_count += 1

    print(
        f"解密完成: 共处理 {log_count} 条日志记录，解密 {total_bytes_decrypted} 字节的数据"
    )


def main() -> None:
    """解析命令行参数并执行解密操作"""
    parser = argparse.ArgumentParser(description="日志文件解密工具")
    parser.add_argument(
        "--input", "-i", required=True, help="要解密的源文件路径"
    )
    parser.add_argument(
        "--key", "-k", required=True, help="AES密钥(16字节字符串)"
    )
    parser.add_argument(
        "--iv", "-v", required=True, help="CTR模式的初始化向量(16字节)"
    )
    parser.add_argument(
        "--output", "-o", required=True, help="解密后的目标文件路径"
    )

    args = parser.parse_args()

    # 检查源文件是否存在
    if not os.path.exists(args.input):
        print(f"错误: 源文件'{args.input}'不存在")
        return

    # 确保目标文件目录存在
    output_dir = os.path.dirname(args.output)
    if output_dir and not os.path.exists(output_dir):
        os.makedirs(output_dir)

    try:
        decrypt_log_file(args.input, args.output, args.key, args.iv)
    except Exception as e:
        print(f"解密过程中出错: {e}")


if __name__ == "__main__":
    main()
