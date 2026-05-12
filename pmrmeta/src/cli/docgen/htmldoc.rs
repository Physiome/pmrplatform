use html5ever::{
    tendril::{ByteTendril, ReadExt},
    tokenizer::{BufferQueue, TagKind, Token, TokenSink, TokenSinkResult, Tokenizer},
};
use std::{
    cell::RefCell,
    io::Read,
};

#[derive(Clone, Copy, Default)]
enum State {
    #[default]
    Standby,
    Capturing,
    Completed,
}

#[derive(Default)]
struct Sink {
    title: RefCell<Vec<String>>,
    state: RefCell<State>,
}

impl TokenSink for Sink {
    type Handle = ();

    /// Each processed token will be handled by this method
    fn process_token(&self, token: Token, _line_number: u64) -> TokenSinkResult<()> {
        let mut state = self.state.borrow_mut();
        match (*state, token) {
            (State::Standby, Token::TagToken(tag))
                if tag.kind == TagKind::StartTag && tag.name.to_string() == "title"
            => {
                *state = State::Capturing;
            }
            (State::Capturing, Token::CharacterTokens(s)) => {
                self.title.borrow_mut().push(s.to_string())
            }
            (State::Capturing, Token::TagToken(tag))
                if tag.kind == TagKind::EndTag && tag.name.to_string() == "title"
            => {
                *state = State::Completed;
            }
            _ => {}
        }
        TokenSinkResult::Continue
    }
}

pub(super) fn parse_title(mut reader: impl Read) -> anyhow::Result<String> {
    let mut chunk = ByteTendril::new();
    reader.read_to_tendril(&mut chunk)?;
    let input = BufferQueue::default();
    input.push_back(chunk.try_reinterpret().expect("reinterpret should have succeeded here"));

    let tok = Tokenizer::new(Sink::default(), Default::default());
    let _ = tok.feed(&input);
    tok.end();
    Ok(tok.sink.title.borrow().join(" "))
}
