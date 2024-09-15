use std::str::FromStr;
use std::sync::OnceLock;

use jni::errors::Result;
use jni::objects::{GlobalRef, JClass, JFieldID, JMethodID, JObject, JStaticFieldID, JValue};
use jni::signature::{JavaType, ReturnType, TypeSignature};
use jni::JNIEnv;

use crate::{impl_jclass_access, impl_signature};

use super::{
    FromJavaValue, IntoJavaValue, JClassAccess, JavaValue, Signature, TryFromJavaValue,
    TryIntoJavaValue,
};

macro_rules! impl_tuple_signature {
    ($sig_type:expr, $($T:ident),+ $(,)?) => {
        impl_signature!($sig_type, ($($T,)+), $($T,)+);
        impl_jclass_access!(($($T,)+), $($T,)+);
    };
}

macro_rules! count_tuple_elements {
    () => { 0 };
    ($T:ident $(,$rest:ident)*) => { 1 + count_tuple_elements!($($rest),*) };
}

macro_rules! impl_tuple_conversion {
    ($(($T:ident, $t:ident, $idx:tt)),+ $(,)?) => {
        impl_tuple_conversion!(safe, $(($T, $t, $idx)),+);
        impl_tuple_conversion!(unchecked, $(($T, $t, $idx)),+);
    };

    // safe conversion
    (safe, $(($T:ident, $t:ident, $idx:tt)),+ $(,)?) => {

        impl<'env, $($T),+> TryIntoJavaValue<'env> for ($($T,)+)
        where
            $($T: TryIntoJavaValue<'env>,)+
        {
            type Target = JObject<'env>;

            fn try_into(self, env: &JNIEnv<'env>) -> Result<Self::Target> {
                static CTOR_ID: OnceLock<JMethodID> = OnceLock::new();
                let ctor_id = CTOR_ID.get_or_init(|| {
                    let param_sig = "Ljava/lang/Object;".repeat(count_tuple_elements!($($T),+));
                    Self::get_method_id(env, "<init>", &format!("({})V", param_sig))
                });


                let java_tuple = env.new_object_unchecked(
                    Self::get_jclass(env),
                    *ctor_id,
                    &[
                        $(JValue::Object($T::Target::autobox($T::try_into(self.$idx, env)?, env))),+
                    ],
                )?;
                Ok(java_tuple)
            }
        }

        impl<'env: 'borrow, 'borrow, $($T),+> TryFromJavaValue<'env, 'borrow> for ($($T,)+)
        where
            $($T: TryFromJavaValue<'env, 'borrow>,)+
        {
            type Source = JObject<'env>;

            fn try_from(s: Self::Source, env: &'borrow JNIEnv<'env>) -> Result<Self> {
                $(
                    let $t = {
                        static FIELD_ID: OnceLock<JFieldID> = OnceLock::new();
                        println!("filed_name: {}, signature:  {}", stringify!($t), <$T as Signature>::SIG_TYPE);
                        let field_id = FIELD_ID.get_or_init(|| Self::get_field_id(env, stringify!($t), <$T as Signature>::SIG_TYPE));

                        $T::try_from(
                            $T::Source::unbox(
                                env.get_field_unchecked(s, *field_id, Self::get_return_type())?.l()?,
                                env,
                            ),
                            env,
                        )?
                    };
                )+

                Ok(($($t,)+))
            }
        }
    };

    // unchecked conversion
    (unchecked, $(($T:ident, $t:ident, $idx:tt)),+ $(,)?) => {

        impl<'env, $($T),+> IntoJavaValue<'env> for ($($T,)+)
        where
            $($T: IntoJavaValue<'env>,)+
        {
            type Target = JObject<'env>;

            fn into(self, env: &JNIEnv<'env>) -> Self::Target {
                static CTOR_ID: OnceLock<JMethodID> = OnceLock::new();
                let ctor_id = CTOR_ID.get_or_init(|| {
                    let param_sig = "Ljava/lang/Object;".repeat(count_tuple_elements!($($T),+));
                    Self::get_method_id(env, "<init>", &format!("({})V", param_sig))
                });


                env.new_object_unchecked(
                    Self::get_jclass(env),
                    *ctor_id,
                    &[
                        $(JValue::Object($T::Target::autobox($T::into(self.$idx, env), env))),+
                    ],
                ).unwrap()
            }
        }

        impl<'env: 'borrow, 'borrow, $($T),+> FromJavaValue<'env, 'borrow> for ($($T,)+)
        where
            $($T: FromJavaValue<'env, 'borrow>,)+
        {
            type Source = JObject<'env>;

            fn from(s: Self::Source, env: &'borrow JNIEnv<'env>) -> Self {
                $(
                    let $t = {
                        static FIELD_ID: OnceLock<JFieldID> = OnceLock::new();
                        let field_id = FIELD_ID.get_or_init(|| Self::get_field_id(env, stringify!($t), <$T as Signature>::SIG_TYPE));

                        $T::from(
                            $T::Source::unbox(
                                env.get_field_unchecked(s, *field_id, Self::get_return_type()).unwrap().l().unwrap(),
                                env,
                            ),
                            env,
                        )
                    };
                )+

                ($($t,)+)
            }
        }
    };
}

macro_rules! impl_tuple_conversion_tuple0 {
    () => {
        impl<'env> JavaValue<'env> for () {
            fn autobox(self, env: &JNIEnv<'env>) -> JObject<'env> {
                static STATIC_FIELD_ID: OnceLock<JStaticFieldID> = OnceLock::new();
                let static_field_id = STATIC_FIELD_ID.get_or_init(|| {
                    JavaTuple0::get_static_field_id(
                        env,
                        "INSTANCE",
                        <JavaTuple0 as Signature>::SIG_TYPE,
                    )
                });

                env.get_static_field_unchecked(
                    JavaTuple0::get_jclass(env),
                    *static_field_id,
                    JavaTuple0::get_java_type(),
                )
                .unwrap()
                .l()
                .unwrap()
            }

            fn unbox(_s: JObject<'env>, _env: &JNIEnv<'env>) -> Self {}
        }
    };
}

macro_rules! impl_tuple_complete {

    ($sig_type:expr $(,)?) => {
        struct JavaTuple0();
        impl_signature!($sig_type, JavaTuple0);
        impl_jclass_access!(JavaTuple0);
        impl_tuple_conversion_tuple0!();
    };

    ($sig_type:expr, $(($T:ident, $t:ident, $idx:tt)),+ $(,)?) => {
        impl_tuple_signature!($sig_type, $($T,)+);
        impl_tuple_conversion!($(($T, $t, $idx)),+);
    };
}

// call macro for implementing all tuples specified in build
crate::convert::config::impl_all_tuples!();
