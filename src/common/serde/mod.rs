use serde::de::DeserializeOwned;
use serde::Serialize;
use std::fmt::Debug;

pub fn serialize_deserialize_does_nothing<T>(obj: T) -> anyhow::Result<T>
where
    T: Serialize + DeserializeOwned + PartialEq + Debug,
{
    let obj_json = serde_json::to_string(&obj)?;
    let obj_json_deserialized = serde_json::from_str::<T>(&obj_json)?;

    assert_eq!(
        obj, obj_json_deserialized,
        "Object changed after serialization-deserialization"
    );

    Ok(obj)
}

pub fn deserialized_sample_equals_expected<T>(expected_value: T, json: &str) -> anyhow::Result<()>
where
    T: DeserializeOwned + PartialEq + Debug,
{
    let deserialized = serde_json::from_str::<T>(json)?;
    assert_eq!(
        deserialized, expected_value,
        "Deserialized object differs from sample: {:?}",
        deserialized
    );

    Ok(())
}

#[macro_export]
macro_rules! test_serde {
    (
        $ty:ty, (
        $obj:expr,
        $expected:expr)) => {
        #[test]
        fn serialize_deserialize_does_nothing() -> anyhow::Result<()> {
            let obj = $obj;

            $crate::common::serde::serialize_deserialize_does_nothing::<$ty>(obj)?;

            Ok(())
        }

        #[test]
        fn deserialized_equals_expected() -> anyhow::Result<()> {
            let expected_value = $obj;
            let json = $expected;

            $crate::common::serde::deserialized_sample_equals_expected::<$ty>(
                expected_value,
                json,
            )?;

            Ok(())
        }
    };
}

pub mod color;
pub mod datetime;
