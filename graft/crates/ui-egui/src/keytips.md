# crates/ui-egui/src/keytips.rs

- Action · enum · L6-L10 — pub enum Action
- KeyTips · struct · L13-L18 — pub struct KeyTips
- prefix · function · L21-L23 — pub fn prefix(&self) -> Option<&str>
- clear · function · L25-L29 — pub fn clear(&mut self)
- toggle · function · L31-L34 — fn toggle(&mut self)
- process · function · L37-L132 — pub fn process(&mut self, ctx: &Context, enabled: bool) -> Vec<Action>
- finish · function · L134-L137 — fn finish(&mut self, action: Action, actions: &mut Vec<Action>)
- command_modifier · function · L140-L142 — fn command_modifier(m: Modifiers) -> bool
- text_pairs · function · L145-L162 — fn text_pairs(events: &[Event]) -> Vec<(usize, usize)>
- tests · module · L165-L208 — mod tests
- key · function · L168-L170 — fn key(key: Key, pressed: bool, modifiers: Modifiers) -> Event
- active_mode_preserves_modified_text_before_key · function · L173-L193 — fn active_mode_preserves_modified_text_before_key()
- shift_between_alt_press_and_release_does_not_activate · function · L196-L207 — fn shift_between_alt_press_and_release_does_not_activate()
