import pytest
import rlp
from hashlib import blake2b
from application_client.transaction import Transaction
from application_client.command_sender import CommandSender, Errors
from application_client.utils import create_ae_curve_path
from application_client.response_unpacker import unpack_sign_response
from ragger.bip import calculate_public_key_and_chaincode, CurveChoice
from utils import check_signature_validity
from ragger.error import ExceptionRAPDU


def test_sign_tx_short_tx(backend, scenario_navigator):
    tx = Transaction(
        tag=0x0C,
        vsn=0x01,
        sender=b'\x01\xf7^S\xf5x""zX\xb4c\t]m\xabe|\xab\x80Et\xbeb\xde\x0b\xe1\xf9Ry\xd0\x907',
        recipient=b'\x01\xf7^S\xf5x""zX\xb4c\t]m\xabe|\xab\x80Et\xbeb\xde\x0b\xe1\xf9Ry\xd0\x907',
        amount=0x1111D67BB1BB0000,
        fee=0x0F4C36200800,
        ttl=0x00,
        nonce=0x0A,
        payload=b"Lorem ipsum dolor sit amet",
    )
    run_sign_tx(backend, scenario_navigator, tx)


def test_sign_tx_short_tx_no_payload(backend, scenario_navigator):
    tx = Transaction(
        tag=0x0C,
        vsn=0x01,
        sender=b'\x01\xf7^S\xf5x""zX\xb4c\t]m\xabe|\xab\x80Et\xbeb\xde\x0b\xe1\xf9Ry\xd0\x907',
        recipient=b'\x01\xf7^S\xf5x""zX\xb4c\t]m\xabe|\xab\x80Et\xbeb\xde\x0b\xe1\xf9Ry\xd0\x907',
        amount=0x1111D67BB1BB0000,
        fee=0x0F4C36200800,
        ttl=0x00,
        nonce=0x0A,
        payload=b"",
    )
    run_sign_tx(backend, scenario_navigator, tx)


def test_sign_tx_short_tx_inner(backend, scenario_navigator):
    tx = Transaction(
        tag=0x0C,
        vsn=0x01,
        sender=b'\x01\xf7^S\xf5x""zX\xb4c\t]m\xabe|\xab\x80Et\xbeb\xde\x0b\xe1\xf9Ry\xd0\x907',
        recipient=b'\x01\xf7^S\xf5x""zX\xb4c\t]m\xabe|\xab\x80Et\xbeb\xde\x0b\xe1\xf9Ry\xd0\x907',
        amount=0x1111D67BB1BB0000,
        fee=0x0F4C36200800,
        ttl=0x00,
        nonce=0x0A,
        payload=b"Lorem ipsum dolor sit amet",
    )
    run_sign_tx(backend, scenario_navigator, tx, inner_tx=True)


def test_sign_tx_long_tx(backend, scenario_navigator):
    tx = Transaction(
        tag=0x0C,
        vsn=0x01,
        sender=b'\x01\xf7^S\xf5x""zX\xb4c\t]m\xabe|\xab\x80Et\xbeb\xde\x0b\xe1\xf9Ry\xd0\x907',
        recipient=b'\x01\xf7^S\xf5x""zX\xb4c\t]m\xabe|\xab\x80Et\xbeb\xde\x0b\xe1\xf9Ry\xd0\x907',
        amount=0x1111D67BB1BB0000,
        fee=0x0F4C36200800,
        ttl=0x00,
        nonce=0x0A,
        payload=(
            b"Lorem ipsum dolor sit amet, consectetur adipiscing elit. Praesent non"
            b" elit non ipsum tristique volutpat. Etiam aliquam neque nunc, et"
            b" iaculis mauris accumsan non. Suspendisse vestibulum ligula sed dui"
            b" viverra suscipit. Ut in nisl tempus, finibus ex id, blandit velit. "
            b"Cras vel congue ante."
        ),
    )
    run_sign_tx(backend, scenario_navigator, tx)


def test_sign_tx_refused(backend, scenario_navigator):
    tx = Transaction(
        tag=0x0C,
        vsn=0x01,
        sender=b'\x01\xf7^S\xf5x""zX\xb4c\t]m\xabe|\xab\x80Et\xbeb\xde\x0b\xe1\xf9Ry\xd0\x907',
        recipient=b'\x01\xf7^S\xf5x""zX\xb4c\t]m\xabe|\xab\x80Et\xbeb\xde\x0b\xe1\xf9Ry\xd0\x907',
        amount=0x1111D67BB1BB0000,
        fee=0x0F4C36200800,
        ttl=0x00,
        nonce=0x0A,
        payload=b"Lorem ipsum dolor sit",
    )
    run_sign_tx(backend, scenario_navigator, tx, approve=False)


def run_sign_tx(backend, scenario_navigator, tx, approve=True, inner_tx=False):
    # Use the app interface instead of raw interface
    client = CommandSender(backend)
    # Random value for account number since it's not important for this test
    account_number = 8
    # Fixed network id since it's not important for this test
    network_id = bytes.fromhex("61655f756174")

    if approve:
        path = create_ae_curve_path(account_number)
        public_key, _ = calculate_public_key_and_chaincode(
            CurveChoice.Ed25519Slip, path=path
        )

        rlp_tx = rlp.encode(Transaction.serialize(tx))
        blake2b_256 = blake2b(digest_size=32)
        blake2b_256.update(rlp_tx)
        if inner_tx:
            data_to_sign = network_id + b"-inner_tx" + blake2b_256.digest()
        else:
            data_to_sign = network_id + blake2b_256.digest()

        with client.sign_tx(
            account_number=account_number,
            inner_tx=inner_tx,
            network_id=network_id,
            transaction=tx,
        ):
            scenario_navigator.review_approve()

        response = client.get_async_response().data
        der_sig = unpack_sign_response(response)
        assert check_signature_validity(
            bytes.fromhex(public_key[2:]), der_sig, data_to_sign
        )
    else:
        with pytest.raises(ExceptionRAPDU) as e:
            with client.sign_tx(
                account_number=account_number,
                inner_tx=inner_tx,
                network_id=network_id,
                transaction=tx,
            ):
                scenario_navigator.review_reject()

        # Assert that we have received a refusal
        assert e.value.status == Errors.SW_DENY
        assert len(e.value.data) == 0
