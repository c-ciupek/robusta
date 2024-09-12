use std::{env, fs, path::Path};

fn main() {
    let result_jni_signature =
        env::var("RESULT_JNI_SIGNATURE").unwrap_or_else(|_| "LResult;".to_string());

    let config_path = Path::new("./src/convert/config.rs");

    // Write the constant declaration to the file
    fs::write(
        config_path,
        format!(
            "pub const RESULT_JNI_SIGNATURE: &str = \"{}\";",
            result_jni_signature
        ),
    )
    .unwrap();

    // Ensure recompilation if the environment variable changes
    println!("cargo:rerun-if-env-changed=RESULT_JNI_SIGNATURE");
}
