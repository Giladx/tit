fn main() {
    #[cfg(windows)]
    {
        winresource::WindowsResource::new()
            .set_icon("img/tit.ico")
            .compile()
            .expect("failed to embed the TIT.RUN Windows icon");
    }
}
