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

crate::convert::config::impl_all_tuples!();
