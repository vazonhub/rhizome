fn main() {
    #[cfg(target_os = "android")]
    {
        println!("cargo:rustc-link-lib=static=rhizome");
    }

    #[cfg(target_os = "ios")]
    {
        println!("cargo:rustc-link-lib=static=rhizome");
    }
}