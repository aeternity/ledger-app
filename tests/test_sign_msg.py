import pytest
from hashlib import blake2b
from application_client.command_sender import CommandSender, Errors
from application_client.response_unpacker import unpack_sign_response
from application_client.utils import create_ae_curve_path
from ragger.bip import calculate_public_key_and_chaincode, CurveChoice
from ragger.error import ExceptionRAPDU
from utils import check_signature_validity, varint_encode


def test_sign_empty_message(backend, scenario_navigator):
    message = ""
    run_sign_message(backend, scenario_navigator, message)


def test_sign_short_message(backend, scenario_navigator):
    message = "Lorem ipsum dolor sit amet"
    run_sign_message(backend, scenario_navigator, message)


def test_sign_long_message(backend, scenario_navigator):
    # 247 bytes long (maximum length for a message)
    message = (
        "Lorem ipsum dolor sit amet, consectetur adipiscing elit. In vitae turpis"
        " at sapien aliquam aliquam. Mauris scelerisque ac nunc id facilisis."
        " Suspendisse tristique ultricies semper. Nam sollicitudin odio quis"
        " mauris dignissim consectetur. Vestibulu"
    )

    assert len(message.encode()) == 247

    run_sign_message(backend, scenario_navigator, message)


def test_sign_invalid_message(backend, scenario_navigator):
    message = "مرحبا"
    run_sign_message(backend, scenario_navigator, message)


def test_sign_message_reject(backend, scenario_navigator):
    message = "Lorem ipsum dolor sit amet, consectetur adipiscing elit."
    run_sign_message(backend, scenario_navigator, message, approve=False)


def run_sign_message(backend, scenario_navigator, message, approve=True):
    # Random value for account number since it's not important for this test
    account_number = 42

    client = CommandSender(backend)
    if approve:
        with client.sign_msg(account_number=account_number, message=message):
            scenario_navigator.review_approve()

        response = client.get_async_response().data
        der_sig = unpack_sign_response(response)

        sign_magic = "aeternity Signed Message:\n"
        data_to_sign = (
            len(sign_magic).to_bytes(1, "big")
            + sign_magic.encode()
            + varint_encode(len(message.encode()))
            + message.encode()
        )
        blake2b_256 = blake2b(digest_size=32)
        blake2b_256.update(data_to_sign)
        data_to_sign = blake2b_256.digest()

        path = create_ae_curve_path(account_number)
        public_key, _ = calculate_public_key_and_chaincode(
            CurveChoice.Ed25519Slip, path=path
        )

        assert check_signature_validity(
            bytes.fromhex(public_key[2:]), der_sig, data_to_sign
        )
    else:
        with pytest.raises(ExceptionRAPDU) as e:
            with client.sign_msg(account_number=account_number, message=message):
                scenario_navigator.review_reject()

        # Assert that we have received a refusal
        assert e.value.status == Errors.SW_DENY
        assert len(e.value.data) == 0
