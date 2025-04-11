#[cfg(target_os = "windows")]
pub fn message_box_error(message: &str) {
    use windows::{
        Win32::UI::WindowsAndMessaging::{MB_ICONERROR, MessageBoxW},
        core::{PCWSTR, w},
    };

    let msg = to_utf16_with_nul(message);
    unsafe {
        MessageBoxW(None, PCWSTR(msg.as_ptr()), w!("Ree Pak GUI Error"), MB_ICONERROR);
    }
}

#[cfg(target_os = "windows")]
fn to_utf16_with_nul(s: &str) -> Vec<u16> {
    s.encode_utf16().chain(Some(0)).collect::<Vec<_>>()
}
