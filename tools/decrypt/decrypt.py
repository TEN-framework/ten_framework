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
    Decrypt a file using AES-128-CTR mode

    Args:
        input_file: Path to the source file to decrypt
        output_file: Path to the destination file after decryption
        key: AES key (16 bytes), string or bytes format
        nonce: Nonce used for CTR mode (typically 8-12 bytes), string or bytes format
        nonce_length: Number of bytes used for nonce, remaining bytes for counter, default is 8
    """
    # Ensure key and nonce are in bytes format
    if isinstance(key, str):
        key = key.encode("utf-8")
    if isinstance(nonce, str):
        nonce = nonce.encode("utf-8")

    # Validate key length
    if len(key) != 16:
        raise ValueError(f"Key must be 16 bytes (current length: {len(key)} bytes)")

    # Determine nonce_length
    if nonce_length is None:
        if len(nonce) <= 8:
            nonce_length = len(nonce)
        else:
            nonce_length = 8  # Default to 8 bytes for nonce

    # Create the correct initial counter value
    if len(nonce) == 16:
        # Already a complete initial counter value
        initial_value = nonce
    else:
        # Need to build initial counter value
        if len(nonce) < nonce_length:
            raise ValueError(
                f"Nonce length ({len(nonce)}) is less than specified nonce_length ({nonce_length})"
            )

        # Take the first nonce_length bytes of nonce
        real_nonce = nonce[:nonce_length]
        # Build initial counter value: nonce + initial counter(0) padded to 16 bytes
        counter_part = b"\x00" * (16 - nonce_length)
        # Convert memoryview to bytes, then concatenate
        initial_value = bytes(real_nonce) + counter_part

    # Create AES-CTR decryptor
    cipher = Cipher(
        algorithms.AES(key), modes.CTR(initial_value), backend=default_backend()
    )
    decryptor = cipher.decryptor()

    # Read and decrypt data in chunks
    buffer_size = 64 * 1024  # 64KB buffer

    with open(input_file, "rb") as in_file, open(output_file, "wb") as out_file:
        while True:
            data = in_file.read(buffer_size)
            if not data:
                break
            decrypted_data = decryptor.update(data)
            out_file.write(decrypted_data)

        # Finalize the decryption process
        out_file.write(decryptor.finalize())

    print(f"File successfully decrypted: {output_file}")


def main() -> None:
    """Parse command line arguments and execute decryption operation"""
    parser = argparse.ArgumentParser(description="AES-128-CTR file decryption tool")
    parser.add_argument(
        "--input", "-i", required=True, help="Path to the source file to decrypt"
    )
    parser.add_argument(
        "--key", "-k", required=True, help="AES key (16 byte string)"
    )
    parser.add_argument(
        "--nonce", "-n", required=True, help="Nonce string for CTR mode"
    )
    parser.add_argument(
        "--output", "-o", required=True, help="Path to the destination file after decryption"
    )

    args = parser.parse_args()

    # Check if source file exists
    if not os.path.exists(args.input):
        print(f"Error: Source file '{args.input}' does not exist")
        return

    # Ensure destination file directory exists
    output_dir = os.path.dirname(args.output)
    if output_dir and not os.path.exists(output_dir):
        os.makedirs(output_dir)

    try:
        decrypt_file(
            args.input, args.output, args.key, args.nonce, nonce_length=8
        )
    except Exception as e:
        print(f"Error during decryption: {e}")


if __name__ == "__main__":
    main()
