import pytest
from application_client.command_sender import CommandSender, Errors
from application_client.response_unpacker import unpack_sign_response
from application_client.utils import create_ae_curve_path
from ragger.bip import calculate_public_key_and_chaincode, CurveChoice
from ragger.error import ExceptionRAPDU
from utils import check_signature_validity


def test_sign_data_empty(backend, scenario_navigator):
    data = "".encode()
    run_sign_data(backend, scenario_navigator, data)


def test_sign_data_short_utf(backend, scenario_navigator):
    data = "Lorem ipsum dolor sit amet, consectetur adipiscing".encode()
    run_sign_data(backend, scenario_navigator, data)


def test_sign_data_short_non_utf(backend, scenario_navigator):
    data = b"\xc0\xab\xf5@\xfb\x03\xc1\x05\x13\xf7x\xf5\xfb\xff\\\xfa\xb2_\xf7\xceU\xa2\xfb\x87\xff\x0bu}\xfa\xcfH\xf8\xfb\x90\xfd\xa1X\xfem\xf8\xc5\xb7\xe7\xf5\xfa\xd7\xc0\xff\xcd\xe1"
    run_sign_data(backend, scenario_navigator, data)


def test_sign_data_short_non_ascii(backend, scenario_navigator):
    data = "مرحبا".encode()
    run_sign_data(backend, scenario_navigator, data)


def test_sign_data_long_utf(backend, scenario_navigator):
    data = "Lorem ipsum dolor sit amet, consectetur adipiscing elit.".encode()
    run_sign_data(backend, scenario_navigator, data)


def test_sign_data_long_non_utf(backend, scenario_navigator):
    data = b"\xfc\xf6\xfe\xfe}\xff\x81\xfe\xe8\xfdM\ru\xf1\x8c\x91\xf6\t\x1av\xf5\x98ch\xd8\xf7\x9d\xfb\x9a\xfaC\xff\xf5\xf6\x1d\xc0\xad\xf5L\xf5\xef\xc1\x82\xfe\x1b\xa7\xf7\xfa\xff\xfa\xf5[3\x8e\x06"
    run_sign_data(backend, scenario_navigator, data)


def test_sign_data_reject(backend, scenario_navigator):
    data = "Lorem ipsum dolor sit amet".encode()
    run_sign_data(backend, scenario_navigator, data, approve=False)


def run_sign_data(backend, scenario_navigator, data_to_sign, approve=True):
    # Random value for account number since it's not important for this test
    account_number = 15

    client = CommandSender(backend)
    if approve:
        with client.sign_data(account_number=account_number, data=data_to_sign):
            scenario_navigator.review_approve()

        response = client.get_async_response().data
        der_sig = unpack_sign_response(response)
        path = create_ae_curve_path(account_number)
        public_key, _ = calculate_public_key_and_chaincode(
            CurveChoice.Ed25519Slip, path=path
        )

        assert check_signature_validity(
            bytes.fromhex(public_key[2:]), der_sig, data_to_sign
        )
    else:
        with pytest.raises(ExceptionRAPDU) as e:
            with client.sign_data(account_number=account_number, data=data_to_sign):
                scenario_navigator.review_reject()

        # Assert that we have received a refusal
        assert e.value.status == Errors.SW_DENY
        assert len(e.value.data) == 0
