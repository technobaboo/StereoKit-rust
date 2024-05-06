use openxr_sys::{
    pfn::{
        CreateVirtualKeyboardMETA, CreateVirtualKeyboardSpaceMETA, DestroyVirtualKeyboardMETA, GetSystemProperties,
        GetVirtualKeyboardDirtyTexturesMETA, GetVirtualKeyboardModelAnimationStatesMETA, GetVirtualKeyboardScaleMETA,
        GetVirtualKeyboardTextureDataMETA, SendVirtualKeyboardInputMETA, SetVirtualKeyboardModelVisibilityMETA,
        SuggestVirtualKeyboardLocationMETA,
    },
    Bool32, Instance, Posef, Result, Session, Space, StructureType, SystemGraphicsProperties, SystemId,
    SystemProperties, SystemTrackingProperties, SystemVirtualKeyboardPropertiesMETA, VirtualKeyboardCreateInfoMETA,
    VirtualKeyboardLocationTypeMETA, VirtualKeyboardMETA, VirtualKeyboardSpaceCreateInfoMETA, FALSE,
    MAX_SYSTEM_NAME_SIZE,
};

use crate::{
    sk::{IStepper, MainThreadToken, SkInfo, StepperAction, StepperId},
    system::{Backend, BackendOpenXR, BackendXRType, Log},
};
use std::{cell::RefCell, ffi::c_void, ptr::null_mut, rc::Rc};

///
///
///  This is a rust copycat of https://github.com/StereoKit/StereoKit/blob/master/Examples/StereoKitTest/Tools/PassthroughFBExt.cs
///
///

/// The StepperAction to trigger with the value "0"/"1" to Show/Hide the keyboard.
pub const KEYBOARD_SHOW: &str = "KeyboardShow";

/// Use PassthroughFbExt::new(true) instead of Default if you want to have it at start up.
///
///
/// ```ignore
/// // The folowing line must be added before initializing sk:
/// BackendOpenXR::request_ext("XR_FB_passthrough");
/// let (sk, event_loop) = settings.init().unwrap();
///
/// // Launch the stepper as follow :
/// let mut passthrough = false;
/// let passthrough_enabled = BackendOpenXR::ext_enabled("XR_FB_passthrough");
/// if passthrough_enabled {
///      sk.push_action(StepperAction::add_default::<PassthroughFbExt>("PassthroughFbExt"));
///      Log::diag("Passthrough Disabled !!")
///  } else {
///      Log::diag("No Passthrough !!")
///  }
///
///  // Activate/Deactivate the stepper as follow :
///  if passthrough_enabled && passthrough != new_passthrough_value {
///      passthrough = new_passthrough_value;
///          let mut string_value = "0";
///          if passthrough {
///              Log::diag("Activate passthrough");
///              string_value = "1";
///          } else {
///              Log::diag("Deactivate passthrough");
///          }
///          sk.push_action(StepperAction::event("main".into(), PASSTHROUGH_FLIP, string_value))
///      }
///  }
/// ```

pub struct VirtualKbdMETA {
    id: StepperId,
    sk_info: Option<Rc<RefCell<SkInfo>>>,
    ext_available: bool,
    enabled: bool,
    enable_on_init: bool,
    virtual_kbd: VirtualKeyboardMETA,
    kbd_space: Space,
    xr_get_system_properties: Option<GetSystemProperties>,
    xr_create_virtual_kbd: Option<CreateVirtualKeyboardMETA>,
    xr_destroy_virtual_kdb: Option<DestroyVirtualKeyboardMETA>,
    xr_create_virtual_kbd_space: Option<CreateVirtualKeyboardSpaceMETA>,
    xr_suggest_virtual_kbd_location: Option<SuggestVirtualKeyboardLocationMETA>,
    xr_get_virtual_kbd_scale: Option<GetVirtualKeyboardScaleMETA>,
    xr_set_virtual_kbd_model_visibility: Option<SetVirtualKeyboardModelVisibilityMETA>,
    xr_get_virtual_kbd_model_animation_states: Option<GetVirtualKeyboardModelAnimationStatesMETA>,
    xr_get_virtual_kbd_dirty_textures: Option<GetVirtualKeyboardDirtyTexturesMETA>,
    xr_get_virtual_kbd_texture_data: Option<GetVirtualKeyboardTextureDataMETA>,
    xr_send_virtual_kbd_input: Option<SendVirtualKeyboardInputMETA>,
}

unsafe impl Send for VirtualKbdMETA {}

impl Default for VirtualKbdMETA {
    fn default() -> Self {
        Self {
            id: "PassthroughFbExt".to_string(),
            sk_info: None,
            ext_available: false,
            enabled: false,
            enable_on_init: false,
            virtual_kbd: VirtualKeyboardMETA::NULL,
            kbd_space: Space::from_raw(0),
            xr_get_system_properties: BackendOpenXR::get_function::<GetSystemProperties>("xrGetSystemProperties"),
            xr_create_virtual_kbd: BackendOpenXR::get_function::<CreateVirtualKeyboardMETA>(
                "xrCreateVirtualKeyboardMETA",
            ),
            xr_destroy_virtual_kdb: BackendOpenXR::get_function::<DestroyVirtualKeyboardMETA>(
                "xrDestroyVirtualKeyboardMETA",
            ),
            xr_create_virtual_kbd_space: BackendOpenXR::get_function::<CreateVirtualKeyboardSpaceMETA>(
                "xrCreateVirtualKeyboardSpaceMETA",
            ),
            xr_suggest_virtual_kbd_location: BackendOpenXR::get_function::<SuggestVirtualKeyboardLocationMETA>(
                "xrSuggestVirtualKeyboardLocationMETA",
            ),
            xr_get_virtual_kbd_scale: BackendOpenXR::get_function::<GetVirtualKeyboardScaleMETA>(
                "xrGetVirtualKeyboardScaleMETA",
            ),
            xr_set_virtual_kbd_model_visibility: BackendOpenXR::get_function::<SetVirtualKeyboardModelVisibilityMETA>(
                "xrSetVirtualKeyboardModelVisibilityMETA",
            ),
            xr_get_virtual_kbd_model_animation_states: BackendOpenXR::get_function::<
                GetVirtualKeyboardModelAnimationStatesMETA,
            >("xrGetVirtualKeyboardModelAnimationStatesMETA"),
            xr_get_virtual_kbd_dirty_textures: BackendOpenXR::get_function::<GetVirtualKeyboardDirtyTexturesMETA>(
                "xrGetVirtualKeyboardDirtyTexturesMETA",
            ),
            xr_get_virtual_kbd_texture_data: BackendOpenXR::get_function::<GetVirtualKeyboardTextureDataMETA>(
                "xrGetVirtualKeyboardTextureDataMETA",
            ),
            xr_send_virtual_kbd_input: BackendOpenXR::get_function::<SendVirtualKeyboardInputMETA>(
                "xrSendVirtualKeyboardInputMETA",
            ),
        }
    }
}

/// All the code here run in the main thread
impl IStepper for VirtualKbdMETA {
    fn initialize(&mut self, id: StepperId, sk_info: Rc<RefCell<SkInfo>>) -> bool {
        self.id = id;
        self.sk_info = Some(sk_info);

        self.ext_available = Backend::xr_type() == BackendXRType::OpenXR
            && BackendOpenXR::ext_enabled("XR_META_virtual_keyboard")
            && self.load_binding()
            && self.init_kbd();

        self.ext_available
    }

    fn enabled(&self) -> bool {
        self.enabled
    }

    fn step(&mut self, token: &MainThreadToken) {
        // Here with enable/disable the passthrough
        for e in token.get_event_report().iter() {
            if let StepperAction::Event(_, key, value) = e {
                if key.eq(KEYBOARD_SHOW) {
                    if value == "0" {
                        self.enable(false)
                    } else {
                        self.enable(true)
                    }
                }
            }
        }
        if self.enabled() {

            // let mut layer = CompositionLayerPassthroughFB {
            //     ty: StructureType::COMPOSITION_LAYER_PASSTHROUGH_FB,
            //     next: null_mut(),
            //     space: Space::from_raw(0),
            //     flags: CompositionLayerFlags::BLEND_TEXTURE_SOURCE_ALPHA,
            //     layer_handle: unsafe { *self.active_layer },
            // };
            // BackendOpenXR::add_composition_layer(&mut layer, -1);
        }
    }

    fn shutdown(&mut self) {
        if self.enabled {
            self.enable(false);
            if self.ext_available {
                unsafe { self.xr_destroy_virtual_kdb.unwrap()(self.virtual_kbd) };
            }
        };
    }
}

impl VirtualKbdMETA {
    /// Use this if you don't want passthrough at init.
    pub fn new(enabled: bool) -> Self {
        Self { enable_on_init: enabled, ..Default::default() }
    }

    pub fn enable(&mut self, value: bool) {
        if self.ext_available && self.enabled != value {
            self.enabled = value;
        }
    }

    fn init_kbd(&mut self) -> bool {
        let mut virtual_kbd_props = SystemVirtualKeyboardPropertiesMETA {
            ty: StructureType::SYSTEM_VIRTUAL_KEYBOARD_PROPERTIES_META,
            next: null_mut(),
            supports_virtual_keyboard: Bool32::from_raw(1),
        };

        let mut system_properties = SystemProperties {
            ty: StructureType::SYSTEM_PROPERTIES,
            next: &mut virtual_kbd_props as *mut _ as *mut c_void,
            system_id: SystemId::NULL,
            vendor_id: 0,
            system_name: [0; MAX_SYSTEM_NAME_SIZE],
            graphics_properties: SystemGraphicsProperties {
                max_swapchain_image_height: 0,
                max_swapchain_image_width: 0,
                max_layer_count: 0,
            },
            tracking_properties: SystemTrackingProperties {
                orientation_tracking: Bool32::from_raw(0),
                position_tracking: Bool32::from_raw(0),
            },
        };

        match unsafe {
            self.xr_get_system_properties.unwrap()(
                Instance::from_raw(BackendOpenXR::instance()),
                SystemId::from_raw(BackendOpenXR::system_id()),
                &mut system_properties,
            )
        } {
            Result::SUCCESS => {
                if virtual_kbd_props.supports_virtual_keyboard == FALSE {
                    Log::err("xrGetSystemProperties returns that supports_virtual_keybord is XrFalse");
                    return false;
                }
            }
            otherwise => {
                Log::err(format!("xrGetSystemProperties failed: {otherwise}"));
                return false;
            }
        }

        //let kbd_model_key = RenderModelKeyFB::from_raw(0);
        match unsafe {
            self.xr_create_virtual_kbd.unwrap()(
                Session::from_raw(BackendOpenXR::session()),
                &VirtualKeyboardCreateInfoMETA {
                    ty: StructureType::VIRTUAL_KEYBOARD_CREATE_INFO_META,
                    next: null_mut(),
                },
                &mut self.virtual_kbd,
            )
        } {
            Result::SUCCESS => {}
            otherwise => {
                Log::err(format!("xrCreateVirtualKeyboardMETA failed: {otherwise}"));
                return false;
            }
        }

        match unsafe {
            self.xr_create_virtual_kbd_space.unwrap()(
                Session::from_raw(BackendOpenXR::session()),
                self.virtual_kbd,
                &VirtualKeyboardSpaceCreateInfoMETA {
                    ty: StructureType::VIRTUAL_KEYBOARD_SPACE_CREATE_INFO_META,
                    next: null_mut(),
                    location_type: VirtualKeyboardLocationTypeMETA::CUSTOM,
                    space: Space::from_raw(BackendOpenXR::space()),
                    pose_in_space: Posef::IDENTITY,
                },
                &mut self.kbd_space,
            )
        } {
            Result::SUCCESS => {}
            otherwise => {
                Log::err(format!("XrVirtualKeyboardSpaceCreateInfoMETA failed: {otherwise}"));
                return false;
            }
        }

        self.enable(self.enable_on_init);
        // !!!!!! true
        todo!("Success !!! we can move on about virtual_kbd");
    }

    /// Check if all the binded functions are ready.
    fn load_binding(&mut self) -> bool {
        self.xr_get_system_properties.is_some()
            && self.xr_create_virtual_kbd.is_some()
            && self.xr_destroy_virtual_kdb.is_some()
            && self.xr_create_virtual_kbd_space.is_some()
            && self.xr_suggest_virtual_kbd_location.is_some()
            && self.xr_get_virtual_kbd_scale.is_some()
            && self.xr_set_virtual_kbd_model_visibility.is_some()
            && self.xr_get_virtual_kbd_model_animation_states.is_some()
            && self.xr_get_virtual_kbd_dirty_textures.is_some()
            && self.xr_get_virtual_kbd_texture_data.is_some()
            && self.xr_send_virtual_kbd_input.is_some()
    }
}
