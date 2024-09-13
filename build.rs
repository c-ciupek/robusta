use std::{collections::HashSet, env, fs, path::Path};

const RESULT_SIGNATURE_ENV: &str = "RESULT_JNI_SIGNATURE";
const TUPLE_SIGNATURE_ENV: &str = "TUPLE_JNI_BASE_PATH";
const TUPLE_IMPL_STRING_ENV: &str = "TUPLE_IMPL_STRING";

const CONFIG_PATH: &str = "./src/convert/config.rs";
const TUPLE_PATH: &str = "./src/convert/tuple.rs";

const TUPLE_IMPORTS: &str = r#"
use std::str::FromStr;
use std::sync::OnceLock;

use jni::errors::Result;
use jni::objects::{GlobalRef, JClass, JFieldID, JMethodID, JObject, JValue};
use jni::signature::{JavaType, ReturnType, TypeSignature};
use jni::JNIEnv;

use super::{
    FromJavaValue, IntoJavaValue, JClassAccess, JavaValue, Signature, TryFromJavaValue,
    TryIntoJavaValue,
};
"#;

fn set_result_jni_config() {
    let result_jni_signature =
        env::var(RESULT_SIGNATURE_ENV).unwrap_or_else(|_| "LResult;".to_string());

    // Write the constant declaration to the file
    fs::write(
        CONFIG_PATH,
        format!(
            "pub const RESULT_JNI_SIGNATURE: &str = \"{}\";",
            result_jni_signature
        ),
    )
    .unwrap();

    println!("cargo:rerun-if-env-changed={}", RESULT_SIGNATURE_ENV);
}

fn parse_tuple_impl_string(impl_string: &str) -> HashSet<usize> {
    impl_string
        .split(",")
        .map(|num| num.parse::<usize>().unwrap())
        .collect()
}

fn get_tuple_sig(base_path: &str, dim: usize) -> String {
    format!("L{}Tuple{};", base_path, dim)
}

fn get_tuple_macro_param(dim: usize) -> String {
    (0..dim)
        .map(|idx| format!("(T{idx}, t{idx}, {idx}), "))
        .collect()
}

fn create_impl_tuple_macros() {
    let tuple_jni_base_path = env::var(TUPLE_SIGNATURE_ENV).unwrap_or_else(|_| "".to_string());
    let tuple_impl_string = env::var(TUPLE_IMPL_STRING_ENV).unwrap_or_else(|_| "".to_string());

    let impl_tuple_set = parse_tuple_impl_string(&tuple_impl_string);

    let mut tuple_macro_call_file = TUPLE_IMPORTS.to_string();

    for dim in impl_tuple_set.iter() {
        tuple_macro_call_file.push_str(&format!(
            r#"crate::convert::impl_tuple_complete!({}, {});"#,
            get_tuple_sig(&tuple_jni_base_path, *dim),
            get_tuple_macro_param(*dim)
        ));
    }

    // Write the tuple macro call to file
    fs::write(TUPLE_PATH, tuple_macro_call_file).unwrap();

    println!("cargo:rerun-if-env-changed={}", TUPLE_SIGNATURE_ENV);
    println!("cargo:rerun-if-env-changed={}", TUPLE_IMPL_STRING_ENV);
}

fn main() {
    #[cfg(feature = "jni_result")]
    set_result_jni_config();

    #[cfg(feature = "jni_tuple")]
    create_impl_tuple_macros();
}
