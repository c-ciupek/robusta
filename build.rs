use std::env;

fn main() {
    let result_jni_signature =
        env::var("RESULT_JNI_SIGNATURE").unwrap_or_else(|_| "LResult;".to_string());

    // Pass the value to the crate code
    println!(
        "cargo:rustc-env=RESULT_JNI_SIGNATURE={}",
        result_jni_signature
    );

    // Ensure recompilation if the environment variable changes
    println!("cargo:rerun-if-env-changed=RESULT_JNI_SIGNATURE");
}
