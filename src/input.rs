//! Various input functions, structs, etc.
//!
//! Very incomplete currently


// pub(crate) fn parse_event(
//     buffer: &[u8],
//     input_available: bool,
// ) -> io::Result<Option<InternalEvent>> {
//     if buffer.is_empty() {
//         return Ok(None);
//     }

//     match buffer[0] {
//         b'\x1B' => {
//             if buffer.len() == 1 {
//                 if input_available {
//                     // Possible Esc sequence
//                     Ok(None)
//                 } else {
//                     Ok(Some(InternalEvent::Event(Event::Key(KeyCode::Esc.into()))))
//                 }
//             } else {
//                 match buffer[1] {
//                     b'O' => {
//                         if buffer.len() == 2 {
//                             Ok(None)
//                         } else {
//                             match buffer[2] {
//                                 b'D' => {
//                                     Ok(Some(InternalEvent::Event(Event::Key(KeyCode::Left.into()))))
//                                 }
//                                 b'C' => Ok(Some(InternalEvent::Event(Event::Key(
//                                     KeyCode::Right.into(),
//                                 )))),
//                                 b'A' => {
//                                     Ok(Some(InternalEvent::Event(Event::Key(KeyCode::Up.into()))))
//                                 }
//                                 b'B' => {
//                                     Ok(Some(InternalEvent::Event(Event::Key(KeyCode::Down.into()))))
//                                 }
//                                 b'H' => {
//                                     Ok(Some(InternalEvent::Event(Event::Key(KeyCode::Home.into()))))
//                                 }
//                                 b'F' => {
//                                     Ok(Some(InternalEvent::Event(Event::Key(KeyCode::End.into()))))
//                                 }
//                                 // F1-F4
//                                 val @ b'P'..=b'S' => Ok(Some(InternalEvent::Event(Event::Key(
//                                     KeyCode::F(1 + val - b'P').into(),
//                                 )))),
//                                 _ => Err(could_not_parse_event_error()),
//                             }
//                         }
//                     }
//                     b'[' => parse_csi(buffer),
//                     b'\x1B' => Ok(Some(InternalEvent::Event(Event::Key(KeyCode::Esc.into())))),
//                     _ => parse_event(&buffer[1..], input_available).map(|event_option| {
//                         event_option.map(|event| {
//                             if let InternalEvent::Event(Event::Key(key_event)) = event {
//                                 let mut alt_key_event = key_event;
//                                 alt_key_event.modifiers |= KeyModifiers::ALT;
//                                 InternalEvent::Event(Event::Key(alt_key_event))
//                             } else {
//                                 event
//                             }
//                         })
//                     }),
//                 }
//             }
//         }
//         b'\r' => Ok(Some(InternalEvent::Event(Event::Key(
//             KeyCode::Enter.into(),
//         )))),
//         // Issue #371: \n = 0xA, which is also the keycode for Ctrl+J. The only reason we get
//         // newlines as input is because the terminal converts \r into \n for us. When we
//         // enter raw mode, we disable that, so \n no longer has any meaning - it's better to
//         // use Ctrl+J. Waiting to handle it here means it gets picked up later
//         b'\n' if !crate::terminal::sys::is_raw_mode_enabled() => Ok(Some(InternalEvent::Event(
//             Event::Key(KeyCode::Enter.into()),
//         ))),
//         b'\t' => Ok(Some(InternalEvent::Event(Event::Key(KeyCode::Tab.into())))),
//         b'\x7F' => Ok(Some(InternalEvent::Event(Event::Key(
//             KeyCode::Backspace.into(),
//         )))),
//         c @ b'\x01'..=b'\x1A' => Ok(Some(InternalEvent::Event(Event::Key(KeyEvent::new(
//             KeyCode::Char((c - 0x1 + b'a') as char),
//             KeyModifiers::CONTROL,
//         ))))),
//         c @ b'\x1C'..=b'\x1F' => Ok(Some(InternalEvent::Event(Event::Key(KeyEvent::new(
//             KeyCode::Char((c - 0x1C + b'4') as char),
//             KeyModifiers::CONTROL,
//         ))))),
//         b'\0' => Ok(Some(InternalEvent::Event(Event::Key(KeyEvent::new(
//             KeyCode::Char(' '),
//             KeyModifiers::CONTROL,
//         ))))),
//         _ => parse_utf8_char(buffer).map(|maybe_char| {
//             maybe_char
//                 .map(KeyCode::Char)
//                 .map(char_code_to_event)
//                 .map(Event::Key)
//                 .map(InternalEvent::Event)
//         }),
//     }
// }

