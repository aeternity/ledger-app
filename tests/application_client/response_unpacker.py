from typing import Tuple
from struct import unpack


# remainder, data_len, data
def pop_sized_buf_from_buffer(buffer: bytes, size: int) -> Tuple[bytes, bytes]:
    return buffer[size:], buffer[0:size]


# remainder, data_len, data
def pop_size_prefixed_buf_from_buf(buffer: bytes) -> Tuple[bytes, int, bytes]:
    data_len = buffer[0]
    return buffer[1 + data_len :], data_len, buffer[1 : data_len + 1]


# Unpack from response:
# response = MAJOR (1)
#            MINOR (1)
#            PATCH (1)
def unpack_get_version_response(response: bytes) -> Tuple[int, int, int]:
    assert len(response) == 3
    major, minor, patch = unpack("BBB", response)
    return (major, minor, patch)


# Unpack from response:
# response = pub_key_len (1)
#            pub_key (var)
def unpack_get_address_response(response: bytes) -> Tuple[int, bytes]:
    response, address_len, address = pop_size_prefixed_buf_from_buf(response)

    assert len(response) == 0
    return address_len, address


# Unpack from response:
# response = sig (64)
def unpack_sign_response(response: bytes) -> Tuple[bytes]:
    response, sig = pop_sized_buf_from_buffer(response, 64)

    assert len(response) == 0

    return sig
