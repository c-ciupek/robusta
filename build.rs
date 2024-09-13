use std::{collections::HashSet, env, fs};

const RESULT_SIGNATURE_ENV: &str = "RESULT_JNI_SIGNATURE";
const TUPLE_SIGNATURE_ENV: &str = "TUPLE_JNI_BASE_PATH";
const TUPLE_IMPL_STRING_ENV: &str = "TUPLE_IMPL_STRING";

const CONFIG_PATH: &str = "./src/convert/config.rs";

const ALL_TUPLE_MACRO_TEMPLATE: &str = r#"
macro_rules! impl_all_tuples {
    () => {
        #tuple_macro_calls
    };
}

pub(crate) use impl_all_tuples;
"#;

fn set_result_jni_config(config_str: &mut String) {
    let result_jni_signature =
        env::var(RESULT_SIGNATURE_ENV).unwrap_or_else(|_| "LResult;".to_string());

    config_str.push_str(&format!(
        "pub const RESULT_JNI_SIGNATURE: &str = \"{}\";",
        result_jni_signature
    ));

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

fn create_impl_tuple_macros(config_str: &mut String) {
    let tuple_jni_base_path = env::var(TUPLE_SIGNATURE_ENV).unwrap_or_else(|_| "".to_string());
    let tuple_impl_string =
        env::var(TUPLE_IMPL_STRING_ENV).unwrap_or_else(|_| "1,2,3,4,5,6,7,8,9,10".to_string());

    let impl_tuple_set = parse_tuple_impl_string(&tuple_impl_string);

    let mut tuple_macro_calls = String::new();

    for dim in impl_tuple_set.iter() {
        tuple_macro_calls.push_str(&format!(
            "crate::convert::impl_tuple_complete!(\"{}\", {});\n",
            get_tuple_sig(&tuple_jni_base_path, *dim),
            get_tuple_macro_param(*dim)
        ));
    }

    config_str
        .push_str(&ALL_TUPLE_MACRO_TEMPLATE.replace("#tuple_macro_calls", &tuple_macro_calls));

    println!("cargo:rerun-if-env-changed={}", TUPLE_SIGNATURE_ENV);
    println!("cargo:rerun-if-env-changed={}", TUPLE_IMPL_STRING_ENV);
}

fn main() {
    let mut config_str = String::new();

    #[cfg(feature = "jni_result")]
    set_result_jni_config(&mut config_str);

    #[cfg(feature = "jni_tuple")]
    create_impl_tuple_macros(&mut config_str);

    // write to config file
    fs::write(CONFIG_PATH, config_str).unwrap();
}
