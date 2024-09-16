use std::{collections::HashSet, env, fs};

const CONFIG_PATH: &str = "./src/convert/config.rs";
const ALL_TUPLE_MACRO_TEMPLATE: &str = r#"
macro_rules! impl_all_tuples {
    () => {

#tuple_macro_calls

    };
}

pub(crate) use impl_all_tuples;
"#;

const RESULT_SIGNATURE_ENV: &str = "RESULT_JNI_SIGNATURE";

fn set_result_jni_config(config_str: &mut String) {
    let result_jni_signature =
        env::var(RESULT_SIGNATURE_ENV).unwrap_or_else(|_| "LResult;".to_string());

    config_str.push_str(&format!(
        "pub const RESULT_JNI_SIGNATURE: &str = \"{}\";\n",
        result_jni_signature
    ));

    println!("cargo:rerun-if-env-changed={}", RESULT_SIGNATURE_ENV);
}

struct TupleConfig();

impl TupleConfig {
    const SIGNATURE_ENV: &str = "TUPLE_JNI_PREFIX";
    const IMPL_STRING_ENV: &str = "TUPLE_IMPL_STRING";

    fn parse_tuple_impl_string(impl_string: &str) -> Vec<usize> {
        let dim_set: HashSet<usize> = impl_string
            .split(",")
            .filter_map(|num| num.trim().parse::<usize>().ok())
            .collect();

        let mut dim_vec: Vec<usize> = dim_set.into_iter().collect();
        dim_vec.sort_unstable();
        dim_vec
    }

    fn get_tuple_sig(jni_prefix: &str, dim: usize) -> String {
        format!("{}{};", jni_prefix, dim)
    }

    fn get_tuple_macro_param(dim: usize) -> String {
        (0..dim)
            .map(|idx| format!("(T{idx}, t{idx}, {idx}), "))
            .collect()
    }

    pub fn create_impl_tuple_macros(config_str: &mut String) {
        let tuple_jni_prefix =
            env::var(Self::SIGNATURE_ENV).unwrap_or_else(|_| "LTuple".to_string());

        // by default create tuple 0 to 12
        let mut tuple_impl_string = env::var(Self::IMPL_STRING_ENV)
            .unwrap_or_else(|_| "0,1,2,3,4,5,6,7,8,9,10,11,12".to_string());

        // always include 0
        tuple_impl_string.push_str(",0");

        let impl_tuple_vec = Self::parse_tuple_impl_string(&tuple_impl_string);

        let mut tuple_macro_calls = String::new();

        for dim in impl_tuple_vec.iter() {
            tuple_macro_calls.push_str(&format!(
                "\t\timpl_tuple_complete!(\"{}\", {});\n",
                Self::get_tuple_sig(&tuple_jni_prefix, *dim),
                Self::get_tuple_macro_param(*dim)
            ));
        }

        config_str
            .push_str(&ALL_TUPLE_MACRO_TEMPLATE.replace("#tuple_macro_calls", &tuple_macro_calls));

        println!("cargo:rerun-if-env-changed={}", Self::SIGNATURE_ENV);
        println!("cargo:rerun-if-env-changed={}", Self::IMPL_STRING_ENV);
    }
}

fn main() {
    let mut config_str = String::new();

    #[cfg(feature = "jni_result")]
    set_result_jni_config(&mut config_str);

    #[cfg(feature = "jni_tuple")]
    TupleConfig::create_impl_tuple_macros(&mut config_str);

    // write to config file
    fs::write(CONFIG_PATH, config_str).unwrap();
}
