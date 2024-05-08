pub fn gen_url(id: &str) -> String {
    format!("https://anilist.co/api/v2/oauth/authorize?client_id={}&response_type=token", id)
}
