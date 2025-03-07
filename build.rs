fn main() {
    #[cfg(windows)]
    {
        let mut res = winres::WindowsResource::new();
        res.set_icon("assets/icon.ico"); // Icon file
        res.compile().unwrap();
    }
}
