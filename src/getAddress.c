#include "getAddress.h"
#include "utils.h"

static const bagl_element_t ui_address_nanos[] = {
    // type            userid    x    y   w    h  str rad fill      fg        bg      fid iid  txt   touchparams...       ]
    {{BAGL_RECTANGLE   , 0x00,   0,   0, 128,  32, 0, 0, BAGL_FILL, 0x000000, 0xFFFFFF, 0, 0}, NULL, 0, 0, 0, NULL, NULL, NULL},

    {{BAGL_ICON        , 0x00,   3,  12,   7,   7, 0, 0, 0        , 0xFFFFFF, 0x000000, 0, BAGL_GLYPH_ICON_CROSS  }, NULL, 0, 0, 0, NULL, NULL, NULL },
    {{BAGL_ICON        , 0x00, 117,  13,   8,   6, 0, 0, 0        , 0xFFFFFF, 0x000000, 0, BAGL_GLYPH_ICON_CHECK  }, NULL, 0, 0, 0, NULL, NULL, NULL },

    {{BAGL_LABELINE    , 0x01,   0,  12, 128,  12, 0, 0, 0        , 0xFFFFFF, 0x000000, BAGL_FONT_OPEN_SANS_EXTRABOLD_11px|BAGL_FONT_ALIGNMENT_CENTER, 0}, "Confirm", 0, 0, 0, NULL, NULL, NULL },
    {{BAGL_LABELINE    , 0x01,   0,  26, 128,  12, 0, 0, 0        , 0xFFFFFF, 0x000000, BAGL_FONT_OPEN_SANS_EXTRABOLD_11px|BAGL_FONT_ALIGNMENT_CENTER, 0}, "address", 0, 0, 0, NULL, NULL, NULL },

    {{BAGL_LABELINE    , 0x02,   0,  12, 128,  12, 0, 0, 0        , 0xFFFFFF, 0x000000, BAGL_FONT_OPEN_SANS_REGULAR_11px|BAGL_FONT_ALIGNMENT_CENTER, 0}, "Address", 0, 0, 0, NULL, NULL, NULL },
    {{BAGL_LABELINE    , 0x02,  23,  26,  82,  12, 0x80|10, 0, 0  , 0xFFFFFF, 0x000000, BAGL_FONT_OPEN_SANS_EXTRABOLD_11px|BAGL_FONT_ALIGNMENT_CENTER, 26  }, tmpCtx.addressContext.address, 0, 0, 0, NULL, NULL, NULL },
};

static unsigned int ui_address_prepro(const bagl_element_t* element) {
    if (element->component.userid > 0) {
        unsigned int display = (ux_step == element->component.userid - 1);
        if (display) {
            switch (element->component.userid) {
                case 1:
                    UX_CALLBACK_SET_INTERVAL(2000);
                    break;
                case 2:
                    UX_CALLBACK_SET_INTERVAL(MAX(3000, 1000 + bagl_label_roundtrip_duration_ms(element, 7)));
                    break;
            }
        }
        return display;
    }
    return 1;
}

static uint8_t set_result_get_address() {
    uint8_t tx = 0;
    uint8_t address_size = strlen(tmpCtx.addressContext.address);
    G_io_apdu_buffer[tx++] = address_size;
    os_memmove(G_io_apdu_buffer + tx, tmpCtx.addressContext.address, address_size);
    tx += address_size;
    return tx;
}

static unsigned int ui_address_nanos_button(unsigned int button_mask, unsigned int button_mask_counter) {
    switch(button_mask) {
        case BUTTON_EVT_RELEASED|BUTTON_LEFT: // CANCEL
            sendResponse(0, false);
            break;

        case BUTTON_EVT_RELEASED|BUTTON_RIGHT: { // OK
            sendResponse(set_result_get_address(), true);
            break;
        }
    }
    return 0;
}

void handleGetAddress(uint8_t p1, uint8_t p2, uint8_t *dataBuffer, uint16_t dataLength, volatile unsigned int *flags, volatile unsigned int *tx) {
    UNUSED(dataLength);
    UNUSED(p2);
    cx_ecfp_private_key_t privateKey;
    cx_ecfp_public_key_t publicKey;

    getPrivateKey(readUint32BE(dataBuffer), &privateKey);
    cx_ecfp_generate_pair(CX_CURVE_Ed25519, &publicKey, &privateKey, 1);
    os_memset(&privateKey, 0, sizeof(privateKey));
    getAeAddressStringFromKey(&publicKey, tmpCtx.addressContext.address);

    if (p1 == P1_NON_CONFIRM) {
        *tx = set_result_get_address();
        THROW(0x9000);
    } else {
        ux_step = 0;
        ux_step_count = 2;
        UX_DISPLAY(ui_address_nanos, ui_address_prepro);

        *flags |= IO_ASYNCH_REPLY;
    }
}
