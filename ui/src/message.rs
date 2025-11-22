use crate::pages::Page;

#[derive(Debug, Clone)]
pub enum Message {
    Tick,
    NavigateTo(Page),
}
