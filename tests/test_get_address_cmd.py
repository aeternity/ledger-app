import pytest

from application_client.command_sender import CommandSender, Errors
from application_client.response_unpacker import unpack_get_address_response
from application_client.utils import UINT32_MAX, create_ae_curve_path
from ragger.bip import calculate_public_key_and_chaincode, CurveChoice
from ragger.error import ExceptionRAPDU
from ragger.navigator import NavInsID, NavIns
from utils import ROOT_SCREENSHOT_PATH
from base58 import b58encode_check
from random import randint


# In this test we check that the GET_ADDRESS works in non-confirmation mode
def test_get_address_no_confirm(backend):
    for account_number in [randint(0, UINT32_MAX) for _ in range(5)]:
        client = CommandSender(backend)
        response = client.get_address(account_number=account_number.to_bytes(4, 'big')).data
        _, address = unpack_get_address_response(response)

        path = create_ae_curve_path(account_number)
        public_key, _ = calculate_public_key_and_chaincode(CurveChoice.Ed25519Slip, path=path)
        ref_address = b'ak_' + b58encode_check(bytes.fromhex(public_key[2:]))

        assert address.decode('ascii') == ref_address.decode('ascii')


# In this test we check that the GET_ADDRESS works in confirmation mode
def test_get_address_confirm_accepted(backend, scenario_navigator):
    client = CommandSender(backend)
    account_number = 20

    with client.get_address_with_confirmation(account_number=account_number.to_bytes(4, 'big')):
        scenario_navigator.address_review_approve()

    response = client.get_async_response().data
    _, address = unpack_get_address_response(response)

    path = create_ae_curve_path(account_number)
    public_key, _ = calculate_public_key_and_chaincode(CurveChoice.Ed25519Slip, path=path)
    ref_address = b'ak_' + b58encode_check(bytes.fromhex(public_key[2:]))

    assert address.decode('ascii') == ref_address.decode('ascii')


# In this test we check that the GET_ADDRESS in confirmation mode replies an error if the user refuses
def test_get_address_confirm_refused(backend, scenario_navigator):
    client = CommandSender(backend)
    account_number = 20

    with pytest.raises(ExceptionRAPDU) as e:
        with client.get_address_with_confirmation(account_number=account_number.to_bytes(4, 'big')):
            scenario_navigator.address_review_reject()

    # Assert that we have received a refusal
    assert e.value.status == Errors.SW_DENY
    assert len(e.value.data) == 0
