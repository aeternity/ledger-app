import rlp
from enum import IntEnum
from typing import Generator, List, Optional
from contextlib import contextmanager

from ragger.backend.interface import BackendInterface, RAPDU
from application_client.transaction import Transaction
# from ragger.bip import pack_derivation_path


MAX_APDU_LEN: int = 255

CLA: int = 0xE0


class P1(IntEnum):
    P1_START = 0x00
    P1_MORE = 0x80
    P1_CONFIRM_NOT_NEEDED = 0x00
    P1_CONFIRM_NEEDED = 0x01


P2: int = 0x00


class InsType(IntEnum):
    GET_ADDRESS = 0x02
    SIGN_TX = 0x04
    GET_VERSION = 0x06
    SIGN_MSG = 0x08
    SIGN_DATA = 0x0A


class Errors(IntEnum):
    SW_DENY = 0x6985
    SW_WRONG_P1P2 = 0x6A86
    SW_INS_NOT_SUPPORTED = 0x6D00
    SW_CLA_NOT_SUPPORTED = 0x6E00
    SW_WRONG_APDU_LENGTH = 0x6E03
    SW_WRONG_RESPONSE_LENGTH = 0xB000
    SW_DISPLAY_BIP32_PATH_FAIL = 0xB001
    SW_DISPLAY_ADDRESS_FAIL = 0xB002
    SW_DISPLAY_AMOUNT_FAIL = 0xB003
    SW_WRONG_TX_LENGTH = 0xB004
    SW_TX_PARSING_FAIL = 0xB005
    SW_TX_HASH_FAIL = 0xB006
    SW_BAD_STATE = 0xB007
    SW_SIGNATURE_FAIL = 0xB008
    SW_MSG_WRONG_LENGTH = (0xB100,)
    SW_MSG_HASH_FAIL = (0xB101,)
    SW_MSG_SIGN_FAIL = (0xB102,)
    SW_GET_ADDRESS_PARSING_FAIL = (0xB200,)


def split_message(message: bytes, max_size: int) -> List[bytes]:
    return [message[x : x + max_size] for x in range(0, len(message), max_size)]


class CommandSender:
    def __init__(self, backend: BackendInterface) -> None:
        self.backend = backend

    def get_version(self) -> RAPDU:
        return self.backend.exchange(cla=CLA, ins=InsType.GET_VERSION)

    def get_address(self, account_number: bytes) -> RAPDU:
        return self.backend.exchange(
            cla=CLA,
            ins=InsType.GET_ADDRESS,
            p1=P1.P1_CONFIRM_NOT_NEEDED,
            p2=P2,
            data=account_number,
        )

    @contextmanager
    def get_address_with_confirmation(
        self, account_number: bytes
    ) -> Generator[None, None, None]:
        with self.backend.exchange_async(
            cla=CLA,
            ins=InsType.GET_ADDRESS,
            p1=P1.P1_CONFIRM_NEEDED,
            p2=P2,
            data=account_number,
        ) as response:
            yield response

    @contextmanager
    def sign_msg(
        self, account_number: int, message: str
    ) -> Generator[None, None, None]:
        with self.backend.exchange_async(
            cla=CLA,
            ins=InsType.SIGN_MSG,
            data=account_number.to_bytes(4, "big")
            + len(message.encode()).to_bytes(4, "big")
            + message.encode(),
        ) as response:
            yield response

    @contextmanager
    def sign_data(
        self, account_number: int, data: bytes
    ) -> Generator[None, None, None]:
        with self.backend.exchange_async(
            cla=CLA,
            ins=InsType.SIGN_DATA,
            data=account_number.to_bytes(4, "big")
            + len(data).to_bytes(4, "big")
            + data,
        ) as response:
            yield response

    @contextmanager
    def sign_tx(
        self, account_number: int, inner_tx: bool, network_id: bytes, transaction: Transaction
    ) -> Generator[None, None, None]:
        tx_rlp = rlp.encode(Transaction.serialize(transaction))
        with self.backend.exchange_async(
            cla=CLA,
            ins=InsType.SIGN_TX,
            data=account_number.to_bytes(4, "big")
            + len(tx_rlp).to_bytes(4, "big")
            + inner_tx.to_bytes(1, "big")
            + len(network_id).to_bytes(1, "big")
            + network_id
            + tx_rlp,
        ) as response:
            yield response

    # @contextmanager
    # def sign_tx(self, path: str, transaction: bytes) -> Generator[None, None, None]:
    #    self.backend.exchange(cla=CLA,
    #                          ins=InsType.SIGN_TX,
    #                          p1=P1.P1_START,
    #                          p2=P2.P2_MORE,
    #                          data=pack_derivation_path(path))
    #    messages = split_message(transaction, MAX_APDU_LEN)
    #    idx: int = P1.P1_START + 1

    #    for msg in messages[:-1]:
    #        self.backend.exchange(cla=CLA,
    #                              ins=InsType.SIGN_TX,
    #                              p1=idx,
    #                              p2=P2.P2_MORE,
    #                              data=msg)
    #        idx += 1

    #    with self.backend.exchange_async(cla=CLA,
    #                                     ins=InsType.SIGN_TX,
    #                                     p1=idx,
    #                                     p2=P2.P2_LAST,
    #                                     data=messages[-1]) as response:
    #        yield response

    def get_async_response(self) -> Optional[RAPDU]:
        return self.backend.last_async_response
