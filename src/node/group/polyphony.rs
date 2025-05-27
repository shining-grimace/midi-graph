use crate::{Error, Event, Message, Node};

struct Voice {
    pub current_note: Option<u8>,
    pub source: Box<dyn Node + Send + 'static>,
}

pub struct Polyphony {
    node_id: u64,
    voices: Vec<Voice>,
    next_on_index: usize,
}

impl Polyphony {
    pub fn new(
        node_id: Option<u64>,
        max_voices: usize,
        consumer: Box<dyn Node + Send + 'static>,
    ) -> Result<Self, Error> {
        if max_voices < 1 {
            return Err(Error::User(format!(
                "Cannot form Polyphony with {} voices",
                max_voices
            )));
        }
        let mut voices = (0..(max_voices - 1))
            .map(|_| {
                consumer.duplicate().map(|source| Voice {
                    current_note: None,
                    source,
                })
            })
            .collect::<Result<Vec<Voice>, Error>>()?;
        voices.push(Voice {
            current_note: None,
            source: consumer,
        });
        Ok(Self {
            node_id: node_id.unwrap_or_else(<Self as Node>::new_node_id),
            voices,
            next_on_index: 0,
        })
    }
}

impl Node for Polyphony {
    fn get_node_id(&self) -> u64 {
        self.node_id
    }

    fn set_node_id(&mut self, node_id: u64) {
        self.node_id = node_id;
    }

    fn duplicate(&self) -> Result<Box<dyn Node + Send + 'static>, Error> {
        let voices = self
            .voices
            .iter()
            .map(|voice| {
                voice.source.duplicate().map(|source| Voice {
                    current_note: None,
                    source,
                })
            })
            .collect::<Result<Vec<Voice>, Error>>()?;
        let polyphony = Self {
            node_id: self.node_id,
            voices,
            next_on_index: 0,
        };
        Ok(Box::new(polyphony))
    }

    fn on_event(&mut self, event: &Message) {
        let was_consumed = if event.target.influences(self.node_id) {
            match event.data {
                Event::NoteOn { note, .. } => {
                    if let Some(index) = self
                        .voices
                        .iter()
                        .position(|voice| voice.current_note.is_none())
                    {
                        self.voices[index].current_note = Some(note);
                        self.voices[index].source.on_event(event);
                    }
                    true
                }
                Event::NoteOff { note, .. } => {
                    if let Some(index) = self
                        .voices
                        .iter()
                        .position(|voice| match voice.current_note {
                            Some(current_note) => current_note == note,
                            None => false
                        })
                    {
                        self.voices[index].source.on_event(event);
                        self.voices[index].current_note = None;
                    }
                    true
                }
                _ => false,
            }
        } else {
            true
        };
        if event.target.propagates_from(self.node_id, was_consumed) {
            for voice in self.voices.iter_mut() {
                voice.source.on_event(event);
            }
        }
    }

    fn fill_buffer(&mut self, buffer: &mut [f32]) {
        for voice in self.voices.iter_mut() {
            voice.source.fill_buffer(buffer);
        }
    }

    fn replace_children(
        &mut self,
        children: &[Box<dyn Node + Send + 'static>],
    ) -> Result<(), Error> {
        if children.len() != 1 {
            return Err(Error::User(
                "Polyphony requires one child which will be duplicated as needed".to_owned(),
            ));
        }

        self.voices = (0..(self.voices.len()))
            .map(|_| {
                children[0].duplicate().map(|source| Voice {
                    current_note: None,
                    source,
                })
            })
            .collect::<Result<Vec<Voice>, Error>>()?;
        self.next_on_index = 0;
        Ok(())
    }
}
