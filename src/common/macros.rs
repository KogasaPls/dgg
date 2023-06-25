#[macro_export]
macro_rules! include_resource {
    ($($path:literal),*, $ty:ty) => {
        serde_json::from_str::<$ty>(include_resource!($($path),*)).unwrap()
    };

    ($($path:expr),*) => {
        include_str!(concat_path!(
            env!("CARGO_MANIFEST_DIR"),
            "resources",
            $($path),*
        ))
    };
}

#[cfg(not(target_os = "windows"))]
concat_with::concat_impl! {
    #[macro_export]
    concat_path => "/",
}

#[cfg(target_os = "windows")]
concat_with::concat_impl! {
    #[macro_export]
    concat_path => "\\",
}
