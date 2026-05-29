#[manganis::ffi("ios/WarmmyImagePicker")]
extern "Swift" {
    pub type WarmmyImagePicker;
    pub fn native_pick_images(this: &WarmmyImagePicker);
}

pub fn pick_images() -> Result<(), &'static str> {
    let picker = WarmmyImagePicker::new()?;
    native_pick_images(&picker)
}
