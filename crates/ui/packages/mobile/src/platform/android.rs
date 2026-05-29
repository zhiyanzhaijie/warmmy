const _: () = {
    const RUSTLS_PLATFORM_VERIFIER_ANDROID: manganis::android::AndroidArtifactMetadata =
        manganis::android::AndroidArtifactMetadata::new(
            "rustls-platform-verifier",
            concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/android/libs/rustls-platform-verifier-0.1.1.aar"
            ),
            "",
        );

    #[used]
    static __RUSTLS_PLATFORM_VERIFIER_ANDROID_ARTIFACT: &'static [u8] = {
        const BUFFER: manganis::android::metadata::AndroidMetadataBuffer =
            manganis::android::metadata::serialize_android_metadata(
                &RUSTLS_PLATFORM_VERIFIER_ANDROID,
            );
        const BYTES: &[u8] = BUFFER.as_ref();
        const LEN: usize = BYTES.len();

        #[unsafe(export_name = "__ASSETS__warmmy_rustls_platform_verifier_android")]
        #[used]
        static LINK_SECTION: [u8; LEN] = manganis::android::macro_helpers::copy_bytes(BYTES);
        &LINK_SECTION
    };
};

pub fn init() {
    init_tls_verifier();
}

fn init_tls_verifier() {
    use ::jni::objects::JObject;
    use ::jni::JavaVM;

    let ctx = ndk_context::android_context();
    let vm = unsafe { JavaVM::from_raw(ctx.vm().cast()) };
    let context = ctx.context() as jni::sys::jobject;
    if context.is_null() {
        eprintln!("failed to initialize Android TLS verifier: missing Android context");
        return;
    }

    let result = vm.attach_current_thread(|env| {
        let context = unsafe { JObject::from_raw(env, context) };
        rustls_platform_verifier::android::init_with_env(env, context)
    });

    if let Err(err) = result {
        eprintln!("failed to initialize Android TLS verifier: init failed: {err}");
    }
}
