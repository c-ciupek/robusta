//! Conversion facilities.
//! This module provides two trait families: [FromJavaValue]/[IntoJavaValue] (infallible conversions) and [TryFromJavaValue]/[TryIntoJavaValue] (fallible conversions),
//! similar to the ones found in the standard library.
//!
//! The `call_type` attribute controls which of the two conversion families is selected during code generation.
//! `call_type` is a per-function attribute.
//! Specific parameters that can be given to `call_type` can be found in the module documentation relative to the trait family ([safe] module for fallible conversions and [unchecked] module for infallible conversions)
//!
//! **If the `call_type` attribute is omitted, the fallible conversion trait family is chosen.**
//!
//! Example usage:
//! ```
//! use robusta_jni::bridge;
//!
//! #[bridge]
//! mod jni {
//!     #[package(com.example.robusta)]
//!     struct HelloWorld;
//!
//!     impl HelloWorld {
//!         #[call_type(unchecked)]
//!         pub extern "jni" fn special(mut input1: Vec<i32>, input2: i32) -> Vec<String> {
//!             input1.push(input2);
//!             input1.iter().map(ToString::to_string).collect()
//!         }
//!
//!         #[call_type(safe(exception_class = "java.lang.IllegalArgumentException", message = "invalid value"))]
//!         pub extern "jni" fn bar(foo: i32) -> ::robusta_jni::jni::errors::Result<i32> { Ok(foo) }
//!     }
//! }
//! ```
//!
//! # Raising exceptions from native code
//! If you want to have the option of throwing a Java exception from native code (conversion errors aside), you can
//! annotate your function signature with a [`jni::errors::Result<T>`] return type.
//!
//! When used with `#[call_type(safe)]`, if an `Err` is returned a Java exception is thrown (the one specified in the `call_type` attribute,
//! or `java.lang.RuntimeException` if omitted).
//!

use std::convert::TryFrom;
use std::str::FromStr;
use std::sync::OnceLock;

use jni::errors::Error;
use jni::objects::{
    GlobalRef, JClass, JFieldID, JMethodID, JObject, JStaticFieldID, JStaticMethodID, JString,
    JValue,
};
use jni::signature::ReturnType;
use jni::sys::{jboolean, jbyte, jchar, jdouble, jfloat, jint, jlong, jobject, jshort};
use jni::JNIEnv;
use paste::paste;

pub use field::*;
pub use robusta_codegen::Signature;
pub use safe::*;
pub use unchecked::*;

pub mod field;
pub mod safe;
pub mod unchecked;

/// A trait for types that are ffi-safe to use with JNI. It is implemented for primitives, [JObject](jni::objects::JObject) and [jobject](jni::sys::jobject).
/// Users that want automatic conversion should instead implement [FromJavaValue], [IntoJavaValue] and/or [TryFromJavaValue], [TryIntoJavaValue]
pub trait JavaValue<'env> {
    /// Convert instance to a [`JObject`].
    fn autobox(self, env: &JNIEnv<'env>) -> JObject<'env>;

    /// Convert [`JObject`] to the implementing type.
    fn unbox(s: JObject<'env>, env: &JNIEnv<'env>) -> Self;
}

/// This trait provides [type signatures](https://docs.oracle.com/en/java/javase/15/docs/specs/jni/types.html#type-signatures) for types.
/// It is necessary to support conversions to/from Java types.
///
/// While you can implement this trait manually, you should probably use the derive macro.
///
/// The derive macro requires a `#[package()]` attribute on implementing structs (most likely you already have that).
///
pub trait Signature {
    /// [Java type signature](https://docs.oracle.com/en/java/javase/15/docs/specs/jni/types.html#type-signatures) for the implementing type.
    const SIG_TYPE: &'static str;
}

macro_rules! jvalue_types {
    ($type:ty: $boxed:ident ($sig:ident) [$unbox_method:ident]) => {
        impl Signature for $type {
            const SIG_TYPE: &'static str = stringify!($sig);
        }

        impl<'env> JavaValue<'env> for $type {
            fn autobox(self, env: &JNIEnv<'env>) -> JObject<'env> {
                env.call_static_method_unchecked(concat!("java/lang/", stringify!($boxed)),
                    (concat!("java/lang/", stringify!($boxed)), "valueOf", concat!(stringify!(($sig)), "Ljava/lang/", stringify!($boxed), ";")),
                    ReturnType::from_str(concat!("Ljava/lang/", stringify!($boxed), ";")).unwrap(),
                    &[JValue::from(self).to_jni()]).unwrap().l().unwrap()
            }

            fn unbox(s: JObject<'env>, env: &JNIEnv<'env>) -> Self {
                paste!(Into::into(env.call_method_unchecked(s, (concat!("java/lang/", stringify!($boxed)), stringify!($unbox_method), concat!("()", stringify!($sig))), ReturnType::from_str(stringify!($sig)).unwrap(), &[])
                    .unwrap().[<$sig:lower>]()
                    .unwrap()))
            }
        }
    };

    ($type:ty: $boxed:ident ($sig:ident) [$unbox_method:ident], $($rest:ty: $rest_boxed:ident ($rest_sig:ident) [$unbox_method_rest:ident]),+) => {
        jvalue_types!($type: $boxed ($sig) [$unbox_method]);

        jvalue_types!($($rest: $rest_boxed ($rest_sig) [$unbox_method_rest]),+);
    }
}

jvalue_types! {
    jboolean: Boolean (Z) [booleanValue],
    jbyte: Byte (B) [byteValue],
    jchar: Character (C) [charValue],
    jdouble: Double (D) [doubleValue],
    jfloat: Float (F) [floatValue],
    jint: Integer (I) [intValue],
    jlong: Long (J) [longValue],
    jshort: Short (S) [shortValue]
}

pub trait JClassAccess<'env>: Signature {
    fn get_jclass(env: &JNIEnv<'env>) -> JClass<'env>;
    fn init_global_class_ref(env: &JNIEnv<'env>) -> GlobalRef {
        let class_name = &<Self as Signature>::SIG_TYPE[1..<Self as Signature>::SIG_TYPE.len() - 1];
        let class = env.find_class(class_name).unwrap();
        env.new_global_ref(class).unwrap()
    }
    fn get_field_id(env: &JNIEnv<'env>, name: &str, sig: &str) -> JFieldID {
        env.get_field_id(Self::get_jclass(env), name, sig).unwrap()
    }
    fn get_static_field_id(env: &JNIEnv<'env>, name: &str, sig: &str) -> JStaticFieldID {
        env.get_static_field_id(Self::get_jclass(env), name, sig)
            .unwrap()
    }
    fn get_method_id(env: &JNIEnv<'env>, name: &str, sig: &str) -> JMethodID {
        env.get_method_id(Self::get_jclass(env), name, sig).unwrap()
    }
    fn get_static_method_id(env: &JNIEnv<'env>, name: &str, sig: &str) -> JStaticMethodID {
        env.get_static_method_id(Self::get_jclass(env), name, sig)
            .unwrap()
    }
}

macro_rules! generate_get_jclass {
    () => {
        fn get_jclass(env: &JNIEnv<'env>) -> JClass<'env> {
            static JCLASS_REF: OnceLock<GlobalRef> = OnceLock::new();
            Into::into(
                env.new_local_ref(JCLASS_REF.get_or_init(|| Self::init_global_class_ref(env)))
                    .unwrap(),
            )
        }
    };
}

impl Signature for () {
    const SIG_TYPE: &'static str = "V";
}

impl<'env> JavaValue<'env> for () {
    fn autobox(self, _env: &JNIEnv<'env>) -> JObject<'env> {
        panic!("called `JavaValue::autobox` on unit value")
    }

    fn unbox(_s: JObject<'env>, _env: &JNIEnv<'env>) -> Self {}
}

impl<'env> Signature for JObject<'env> {
    const SIG_TYPE: &'static str = "Ljava/lang/Object;";
}

impl<'env> JavaValue<'env> for JObject<'env> {
    fn autobox(self, _env: &JNIEnv<'env>) -> JObject<'env> {
        self
    }

    fn unbox(s: JObject<'env>, _env: &JNIEnv<'env>) -> Self {
        s
    }
}

impl<'env> JavaValue<'env> for jobject {
    fn autobox(self, _env: &JNIEnv<'env>) -> JObject<'env> {
        unsafe { JObject::from_raw(self) }
    }

    fn unbox(s: JObject<'env>, _env: &JNIEnv<'env>) -> Self {
        s.into_raw()
    }
}

impl<'env> JClassAccess<'env> for JObject<'env> {
    generate_get_jclass!();
}

impl Signature for String {
    const SIG_TYPE: &'static str = "Ljava/lang/String;";
}

impl<'env> JClassAccess<'env> for String {
    generate_get_jclass!();
}

impl<'env> Signature for JString<'env> {
    const SIG_TYPE: &'static str = "Ljava/lang/String;";
}

impl<'env> JClassAccess<'env> for JString<'env> {
    generate_get_jclass!();
}

impl<'env> JavaValue<'env> for JString<'env> {
    fn autobox(self, _env: &JNIEnv<'env>) -> JObject<'env> {
        Into::into(self)
    }

    fn unbox(s: JObject<'env>, _env: &JNIEnv<'env>) -> Self {
        From::from(s)
    }
}

impl<T: Signature> Signature for Vec<T> {
    const SIG_TYPE: &'static str = "Ljava/util/ArrayList;";
}

impl<'env, T: Signature> JClassAccess<'env> for Vec<T> {
    generate_get_jclass!();
}

impl<T: Signature> Signature for jni::errors::Result<T> {
    const SIG_TYPE: &'static str = <T as Signature>::SIG_TYPE;
}

impl<'env, T: Signature> Signature for Option<T> {
    const SIG_TYPE: &'static str = T::SIG_TYPE;
}

impl<'env, Ok, Err> Signature for core::result::Result<Ok, Err>
where
    Ok: Signature,
    Err: Signature,
{
    const SIG_TYPE: &'static str = env!("RESULT_JNI_SIGNATURE");
}

impl<'env, Ok, Err> JClassAccess<'env> for core::result::Result<Ok, Err>
where
    Ok: Signature,
    Err: Signature,
{
    generate_get_jclass!();
}

pub struct JValueWrapper<'a>(pub JValue<'a>);

impl<'a> From<JValue<'a>> for JValueWrapper<'a> {
    fn from(v: JValue<'a>) -> Self {
        JValueWrapper(v)
    }
}

impl<'a> From<JValueWrapper<'a>> for JValue<'a> {
    fn from(v: JValueWrapper<'a>) -> Self {
        v.0
    }
}

impl<'a> TryFrom<JValueWrapper<'a>> for jboolean {
    type Error = jni::errors::Error;

    fn try_from(value: JValueWrapper<'a>) -> Result<Self, Self::Error> {
        match value.0 {
            JValue::Bool(b) => Ok(b),
            _ => Err(Error::WrongJValueType("bool", value.0.type_name()).into()),
        }
    }
}

impl<'a> TryFrom<JValueWrapper<'a>> for jbyte {
    type Error = jni::errors::Error;

    fn try_from(value: JValueWrapper<'a>) -> Result<Self, Self::Error> {
        match value.0 {
            JValue::Byte(b) => Ok(b),
            _ => Err(Error::WrongJValueType("byte", value.0.type_name()).into()),
        }
    }
}

impl<'a> TryFrom<JValueWrapper<'a>> for jchar {
    type Error = jni::errors::Error;

    fn try_from(value: JValueWrapper<'a>) -> Result<Self, Self::Error> {
        match value.0 {
            JValue::Char(c) => Ok(c),
            _ => Err(Error::WrongJValueType("char", value.0.type_name()).into()),
        }
    }
}

impl<'a> TryFrom<JValueWrapper<'a>> for jdouble {
    type Error = jni::errors::Error;

    fn try_from(value: JValueWrapper<'a>) -> Result<Self, Self::Error> {
        match value.0 {
            JValue::Double(d) => Ok(d),
            _ => Err(Error::WrongJValueType("double", value.0.type_name()).into()),
        }
    }
}

impl<'a> TryFrom<JValueWrapper<'a>> for jfloat {
    type Error = jni::errors::Error;

    fn try_from(value: JValueWrapper<'a>) -> Result<Self, Self::Error> {
        match value.0 {
            JValue::Float(f) => Ok(f),
            _ => Err(Error::WrongJValueType("float", value.0.type_name()).into()),
        }
    }
}

impl<'a> TryFrom<JValueWrapper<'a>> for jint {
    type Error = jni::errors::Error;

    fn try_from(value: JValueWrapper<'a>) -> Result<Self, Self::Error> {
        match value.0 {
            JValue::Int(i) => Ok(i),
            _ => Err(Error::WrongJValueType("int", value.0.type_name()).into()),
        }
    }
}

impl<'a> TryFrom<JValueWrapper<'a>> for jshort {
    type Error = jni::errors::Error;

    fn try_from(value: JValueWrapper<'a>) -> Result<Self, Self::Error> {
        match value.0 {
            JValue::Short(s) => Ok(s),
            _ => Err(Error::WrongJValueType("short", value.0.type_name()).into()),
        }
    }
}

impl<'a> TryFrom<JValueWrapper<'a>> for jlong {
    type Error = jni::errors::Error;

    fn try_from(value: JValueWrapper<'a>) -> Result<Self, Self::Error> {
        match value.0 {
            JValue::Long(l) => Ok(l),
            _ => Err(Error::WrongJValueType("long", value.0.type_name()).into()),
        }
    }
}

impl<'a> TryFrom<JValueWrapper<'a>> for () {
    type Error = jni::errors::Error;

    fn try_from(value: JValueWrapper<'a>) -> Result<Self, Self::Error> {
        match value.0 {
            JValue::Void => Ok(()),
            _ => Err(Error::WrongJValueType("void", value.0.type_name()).into()),
        }
    }
}

impl<'a> TryFrom<JValueWrapper<'a>> for JObject<'a> {
    type Error = jni::errors::Error;

    fn try_from(value: JValueWrapper<'a>) -> Result<Self, Self::Error> {
        match value.0 {
            JValue::Object(o) => Ok(o),
            _ => Err(Error::WrongJValueType("object", value.0.type_name()).into()),
        }
    }
}

impl<'a> TryFrom<JValueWrapper<'a>> for JString<'a> {
    type Error = jni::errors::Error;

    fn try_from(value: JValueWrapper<'a>) -> Result<Self, Self::Error> {
        match value.0 {
            JValue::Object(o) => Ok(From::from(o)),
            _ => Err(Error::WrongJValueType("string", value.0.type_name()).into()),
        }
    }
}
