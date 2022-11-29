const API_ROOT: &str = "https://api.mojang.com";

pub(crate) fn usernames(username: &str) -> String {
    format!("{API_ROOT}/users/profiles/minecraft/{username}")
}

pub(crate) fn uids(uuid: &str) -> String {
    format!("{API_ROOT}/user/profile/{uuid}")
}
