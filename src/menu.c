#include "menu.h"
#include "os.h"

volatile uint8_t dataAllowed;
volatile uint8_t contractDetails;

#if defined(TARGET_NANOS)

#ifdef HAVE_U2F

static const ux_menu_entry_t menu_main[];
static const ux_menu_entry_t menu_settings[];
static const ux_menu_entry_t menu_settings_data[];
static const ux_menu_entry_t menu_settings_details[];

// change the setting
static void menu_settings_data_change(unsigned int enabled) {
    dataAllowed = enabled;
    nvm_write(&N_storage.dataAllowed, (void*)&dataAllowed, sizeof(uint8_t));
    // go back to the menu entry
    UX_MENU_DISPLAY(0, menu_settings, NULL);
}

static void menu_settings_details_change(unsigned int enabled) {
    contractDetails = enabled;
    nvm_write(&N_storage.contractDetails, (void*)&contractDetails, sizeof(uint8_t));
    // go back to the menu entry
    UX_MENU_DISPLAY(0, menu_settings, NULL);
}

// show the currently activated entry
static void menu_settings_data_init(unsigned int ignored) {
    UNUSED(ignored);
    UX_MENU_DISPLAY(N_storage.dataAllowed?1:0, menu_settings_data, NULL);
}

static void menu_settings_details_init(unsigned int ignored) {
    UNUSED(ignored);
    UX_MENU_DISPLAY(N_storage.contractDetails?1:0, menu_settings_details, NULL);
}

static const ux_menu_entry_t menu_settings_data[] = {
    {NULL, menu_settings_data_change, 0, NULL, "No", NULL, 0, 0},
    {NULL, menu_settings_data_change, 1, NULL, "Yes", NULL, 0, 0},
    UX_MENU_END
};

static const ux_menu_entry_t menu_settings_details[] = {
    {NULL, menu_settings_details_change, 0, NULL, "No", NULL, 0, 0},
    {NULL, menu_settings_details_change, 1, NULL, "Yes", NULL, 0, 0},
    UX_MENU_END
};

static const ux_menu_entry_t menu_settings[] = {
    {NULL, menu_settings_data_init, 0, NULL, "Contract data", NULL, 0, 0},
    {NULL, menu_settings_details_init, 0, NULL, "Display data", NULL, 0, 0},
    {menu_main, NULL, 1, &C_icon_back, "Back", NULL, 61, 40},
    UX_MENU_END
};
#endif // HAVE_U2F

static const ux_menu_entry_t menu_about[] = {
    {NULL, NULL, 0, NULL, "Version", APPVERSION , 0, 0},
    {menu_main, NULL, 2, &C_icon_back, "Back", NULL, 61, 40},
    UX_MENU_END
};

static const ux_menu_entry_t menu_main[] = {
    //{NULL, NULL, 0, &NAME3(C_nanos_badge_, CHAINID, ), "Use wallet to", "view accounts", 33, 12},
    {NULL, NULL, 0, NULL, "Use wallet to", "view accounts", 0, 0},
    {menu_settings, NULL, 0, NULL, "Settings", NULL, 0, 0},
    {menu_about, NULL, 0, NULL, "About", NULL, 0, 0},
    {NULL, os_sched_exit, 0, &C_icon_dashboard, "Quit app", NULL, 50, 29},
    UX_MENU_END
};

#elif defined(TARGET_NANOX)

void display_settings(void);
void switch_settings_contract_data(void);
void switch_settings_display_data(void);

//////////////////////////////////////////////////////////////////////


const char* settings_submenu_getter(unsigned int idx);
void settings_submenu_selector(unsigned int idx);


//////////////////////////////////////////////////////////////////////////////////////
// Enable contract data submenu:

void settings_contract_data_change(unsigned int enabled) {
    nvm_write((void *)&N_storage.dataAllowed, &enabled, 1);
    ui_idle();
}

const char* const settings_contract_data_getter_values[] = {
  "No",
  "Yes",
  "Back"
};

const char* settings_contract_data_getter(unsigned int idx) {
  if (idx < ARRAYLEN(settings_contract_data_getter_values)) {
    return settings_contract_data_getter_values[idx];
  }
  return NULL;
}

void settings_contract_data_selector(unsigned int idx) {
  switch(idx) {
    case 0:
      settings_contract_data_change(0);
      break;
    case 1:
      settings_contract_data_change(1);
      break;
    default:
      ux_menulist_init(0, settings_submenu_getter, settings_submenu_selector);
  }
}

//////////////////////////////////////////////////////////////////////////////////////
// Display contract data submenu:

void settings_display_data_change(unsigned int enabled) {
    nvm_write((void *)&N_storage.contractDetails, &enabled, 1);
    ui_idle();
}

const char* const settings_display_data_getter_values[] = {
  "No",
  "Yes",
  "Back"
};

const char* settings_display_data_getter(unsigned int idx) {
  if (idx < ARRAYLEN(settings_display_data_getter_values)) {
    return settings_display_data_getter_values[idx];
  }
  return NULL;
}

void settings_display_data_selector(unsigned int idx) {
  switch(idx) {
    case 0:
      settings_display_data_change(0);
      break;
    case 1:
      settings_display_data_change(1);
      break;
    default:
      ux_menulist_init(0, settings_submenu_getter, settings_submenu_selector);
  }
}

//////////////////////////////////////////////////////////////////////////////////////
// Settings menu:

const char* const settings_submenu_getter_values[] = {
  "Contract data",
  "Display data",
  "Back",
};

const char* settings_submenu_getter(unsigned int idx) {
  if (idx < ARRAYLEN(settings_submenu_getter_values)) {
    return settings_submenu_getter_values[idx];
  }
  return NULL;
}

void settings_submenu_selector(unsigned int idx) {
  switch(idx) {
    case 0:
      ux_menulist_init_select(0, settings_contract_data_getter, settings_contract_data_selector, N_storage.dataAllowed);
      break;
    case 1:
      ux_menulist_init_select(0, settings_display_data_getter, settings_display_data_selector, N_storage.contractDetails);
      break;
    default:
      ui_idle();
  }
}

//////////////////////////////////////////////////////////////////////
UX_STEP_NOCB(
    ux_idle_flow_1_step,
    nn,
    {
      "Application",
      "is ready",
    });
UX_STEP_VALID(
    ux_idle_flow_2_step,
    pb,
    ux_menulist_init(0, settings_submenu_getter, settings_submenu_selector),
    {
      &C_icon_coggle,
      "Settings",
    });
UX_STEP_NOCB(
    ux_idle_flow_3_step,
    bn,
    {
      "Version",
      APPVERSION,
    });
UX_STEP_VALID(
    ux_idle_flow_4_step,
    pb,
    os_sched_exit(-1),
    {
      &C_icon_dashboard_x,
      "Quit",
    });
UX_FLOW(ux_idle_flow,
  &ux_idle_flow_1_step,
  &ux_idle_flow_2_step,
  &ux_idle_flow_3_step,
  &ux_idle_flow_4_step
);


#endif

void ui_idle(void) {
#if defined(TARGET_NANOS)
    UX_MENU_DISPLAY(0, menu_main, NULL);
#elif defined(TARGET_NANOX)
    // reserve a display stack slot if none yet
    if(G_ux.stack_count == 0) {
        ux_stack_push();
    }
    ux_flow_init(0, ux_idle_flow, NULL);
#endif // #if TARGET_ID
}
