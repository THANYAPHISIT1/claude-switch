// Future: Windows Credential Manager implementation

use crate::security::token::Profile;

#[allow(dead_code)]
pub fn restore(_profile: &Profile) -> Result<bool, String> {
    unimplemented!("Windows Credential Manager not yet implemented")
}

#[allow(dead_code)]
pub fn backup(_profile: &Profile) -> Result<bool, String> {
    unimplemented!("Windows Credential Manager not yet implemented")
}
