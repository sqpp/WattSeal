#[cfg(target_os = "windows")]
const WINDOWS_ICON_PATH: &str = "resources/icon.ico";

fn main() {
    #[cfg(target_os = "windows")]
    {
        let mut res = winres::WindowsResource::new();
        res.set_icon(WINDOWS_ICON_PATH);
        res.compile().unwrap();
    }
}
