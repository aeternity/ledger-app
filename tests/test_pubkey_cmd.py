import pytest

from application_client.command_sender import CommandSender, Errors
from application_client.response_unpacker import unpack_get_address_response
from application_client.utils import create_ae_curve_path
from ragger.bip import calculate_public_key_and_chaincode, CurveChoice
from ragger.error import ExceptionRAPDU
from ragger.navigator import NavInsID, NavIns
from utils import ROOT_SCREENSHOT_PATH
from base58 import b58encode_check


# In this test we check that the GET_ADDRESS works in non-confirmation mode
def test_get_address_no_confirm(backend):
    for account_number in range(3):
        client = CommandSender(backend)
        response = client.get_address(account_number=account_number.to_bytes(4, 'big')).data
        _, address = unpack_get_address_response(response)

        path = create_ae_curve_path(account_number)
        public_key, _ = calculate_public_key_and_chaincode(CurveChoice.Ed25519Slip, path=path)
        ref_address = b'ak_' + b58encode_check(bytes.fromhex(public_key[2:]))

        assert address.decode('ascii') == ref_address.decode('ascii')


"""
# In this test we check that the GET_PUBLIC_KEY works in confirmation mode
def test_get_public_key_confirm_accepted(backend, scenario_navigator):
    client = CommandSender(backend)
    path = "m/44'/1'/0'/0/0"
    
    with client.get_address_with_confirmation(account_number=path):
        scenario_navigator.address_review_approve()
        
    response = client.get_async_response().data
    _, public_key, _, _ = unpack_get_public_key_response(response)

    ref_public_key, _ = calculate_public_key_and_chaincode(CurveChoice.Secp256k1, path=path)
    assert public_key.hex() == ref_public_key


# In this test we check that the GET_PUBLIC_KEY in confirmation mode replies an error if the user refuses
def test_get_public_key_confirm_refused(backend, scenario_navigator):
    client = CommandSender(backend)
    path = "m/44'/1'/0'/0/0"

    with pytest.raises(ExceptionRAPDU) as e:
        with client.get_address_with_confirmation(account_number=path):
            scenario_navigator.address_review_reject()

    # Assert that we have received a refusal
    assert e.value.status == Errors.SW_DENY
    assert len(e.value.data) == 0
"""
