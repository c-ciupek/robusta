use std::sync::OnceLock;

use super::{
    config, FromJavaValue, IntoJavaValue, JClassAccess, JavaValue, Signature, TryFromJavaValue,
    TryIntoJavaValue,
};
use jni::errors::Result;
use jni::objects::{JFieldID, JMethodID, JObject, JValue};
use jni::signature::{JavaType, Primitive, ReturnType, TypeSignature};
use jni::JNIEnv;
use std::str::FromStr;

crate::impl_signature!(config::RESULT_JNI_SIGNATURE, core::result::Result<Ok, Err>, Ok, Err);
crate::impl_jclass_access!(core::result::Result<Ok, Err>, Ok, Err);

// safe implementation
impl<'env, Ok, Err> IntoJavaValue<'env> for core::result::Result<Ok, Err>
where
    Ok: IntoJavaValue<'env>,
    Err: IntoJavaValue<'env>,
{
    type Target = JObject<'env>;
    fn into(self, env: &JNIEnv<'env>) -> Self::Target {
        static CTOR_ID: OnceLock<JMethodID> = OnceLock::new();
        let ctor_id =
            CTOR_ID.get_or_init(|| Self::get_method_id(env, "<init>", "(BLjava/lang/Object;)V"));
        let (tag, value) = match self {
            Ok(ok) => {
                let ok_value = Ok::Target::autobox(Ok::into(ok, env), env);
                (0i8, ok_value)
            }
            Err(err) => {
                let err_value = Err::Target::autobox(Err::into(err, env), env);
                (1i8, err_value)
            }
        };
        env.new_object_unchecked(
            Self::get_jclass(env),
            *ctor_id,
            &[JValue::Byte(tag), JValue::Object(value)],
        )
        .unwrap()
    }
}

impl<'env: 'borrow, 'borrow, Ok, Err> FromJavaValue<'env, 'borrow> for core::result::Result<Ok, Err>
where
    Ok: FromJavaValue<'env, 'borrow>,
    Err: FromJavaValue<'env, 'borrow>,
{
    type Source = JObject<'env>;
    fn from(s: Self::Source, env: &'borrow JNIEnv<'env>) -> Self {
        static TAG_FIELD_ID: OnceLock<JFieldID> = OnceLock::new();
        static VALUE_FIELD_ID: OnceLock<JFieldID> = OnceLock::new();
        let tag_field_id = TAG_FIELD_ID.get_or_init(|| Self::get_field_id(env, "tag", "B"));
        let tag = env
            .get_field_unchecked(s, *tag_field_id, ReturnType::Primitive(Primitive::Byte))
            .unwrap()
            .b()
            .unwrap();
        let value_field_id =
            VALUE_FIELD_ID.get_or_init(|| Self::get_field_id(env, "value", "Ljava/lang/Object;"));
        let value = env
            .get_field_unchecked(s, *value_field_id, ReturnType::Object)
            .unwrap()
            .l()
            .unwrap();

        match tag {
            0 => {
                let ok = Ok::from(Ok::Source::unbox(value, env), env);
                core::result::Result::Ok(ok)
            }
            1 => {
                let err = Err::from(Err::Source::unbox(value, env), env);
                core::result::Result::Err(err)
            }
            _ => unreachable!(),
        }
    }
}

// unchecked implementation
impl<'env, Ok, Err> TryIntoJavaValue<'env> for core::result::Result<Ok, Err>
where
    Ok: TryIntoJavaValue<'env>,
    Err: TryIntoJavaValue<'env>,
{
    type Target = JObject<'env>;
    fn try_into(self, env: &JNIEnv<'env>) -> Result<Self::Target> {
        static CTOR_ID: OnceLock<JMethodID> = OnceLock::new();
        let ctor_id =
            CTOR_ID.get_or_init(|| Self::get_method_id(env, "<init>", "(BLjava/lang/Object;)V"));
        let (tag, value) = match self {
            Ok(ok) => {
                let ok_value = Ok::Target::autobox(Ok::try_into(ok, env)?, env);
                (0i8, ok_value)
            }
            Err(err) => {
                let err_value = Err::Target::autobox(Err::try_into(err, env)?, env);
                (1i8, err_value)
            }
        };
        let result_object = env.new_object_unchecked(
            Self::get_jclass(env),
            *ctor_id,
            &[JValue::Byte(tag), JValue::Object(value)],
        )?;
        Ok(result_object)
    }
}

impl<'env: 'borrow, 'borrow, Ok, Err> TryFromJavaValue<'env, 'borrow>
    for core::result::Result<Ok, Err>
where
    Ok: TryFromJavaValue<'env, 'borrow>,
    Err: TryFromJavaValue<'env, 'borrow>,
{
    type Source = JObject<'env>;
    fn try_from(s: Self::Source, env: &'borrow JNIEnv<'env>) -> Result<Self> {
        static TAG_FIELD_ID: OnceLock<JFieldID> = OnceLock::new();
        static VALUE_FIELD_ID: OnceLock<JFieldID> = OnceLock::new();
        let tag_field_id = TAG_FIELD_ID.get_or_init(|| Self::get_field_id(env, "tag", "B"));
        let tag = env
            .get_field_unchecked(s, *tag_field_id, ReturnType::Primitive(Primitive::Byte))?
            .b()?;
        let value_field_id =
            VALUE_FIELD_ID.get_or_init(|| Self::get_field_id(env, "value", "Ljava/lang/Object;"));
        let value = env
            .get_field_unchecked(s, *value_field_id, ReturnType::Object)?
            .l()?;
        let result = match tag {
            0 => {
                let ok = Ok::try_from(Ok::Source::unbox(value, env), env)?;
                core::result::Result::Ok(ok)
            }
            1 => {
                let err = Err::try_from(Err::Source::unbox(value, env), env)?;
                core::result::Result::Err(err)
            }
            _ => unreachable!(),
        };
        Ok(result)
    }
}
