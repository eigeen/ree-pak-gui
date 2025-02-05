use windows::{
    core::{w, PCWSTR},
    Win32::UI::WindowsAndMessaging::{MessageBoxW, MB_ICONERROR},
};

pub fn message_box_error(message: &str) {
    let msg = to_utf16_with_nul(message);
    unsafe {
        MessageBoxW(None, PCWSTR(msg.as_ptr()), w!("Ree Pak GUI Error"), MB_ICONERROR);
    }
}

fn to_utf16_with_nul(s: &str) -> Vec<u16> {
    s.encode_utf16().chain(Some(0)).collect::<Vec<_>>()
}
