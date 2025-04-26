import rlp
from rlp.sedes import big_endian_int, binary


class Transaction(rlp.Serializable):
    fields = (
        ("tag", big_endian_int),
        ("vsn", big_endian_int),
        ("sender", binary),
        ("recipient", binary),
        ("amount", big_endian_int),
        ("fee", big_endian_int),
        ("ttl", big_endian_int),
        ("nonce", big_endian_int),
        ("payload", binary),
    )
