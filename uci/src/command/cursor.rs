use std::str::Chars;

#[derive(Debug, Clone)]
pub struct Cursor<'line>(Chars<'line>);

impl<'line> Cursor<'line> {
    pub fn new(line: &'line str) -> Self {
        Self(line.chars())
    }
    pub fn until_token(&mut self, stop: &str) -> &'line str {
        let s = self.0.as_str();
        let before = self.len();
        let mut after = self.len();
        while !self.is_empty() {
            self.skip_whitespace();
            after = self.len();
            if self.next_token() == Some(stop) {
                break;
            }
        }
        &s[..(before - after)]
    }
    pub fn next_token(&mut self) -> Option<&'line str> {
        self.skip_whitespace();
        if self.is_empty() {
            return None;
        }
        let s = self.0.as_str();
        let before = self.len();
        self.skip_token();
        let slice = &s[..(before - self.len())];
        Some(slice)
    }
    pub fn is_empty(&self) -> bool {
        self.peek().is_none()
    }
    fn skip_token(&mut self) {
        self.skip(|ch| !ch.is_whitespace());
    }
    fn skip_whitespace(&mut self) {
        self.skip(|ch| ch.is_whitespace());
    }
    fn skip(&mut self, mut p: impl FnMut(char) -> bool) {
        while self.peek().is_some_and(&mut p) {
            self.0.next();
        }
    }
    fn len(&self) -> usize {
        self.0.as_str().len()
    }
    fn peek(&self) -> Option<char> {
        self.0.clone().next()
    }
}
