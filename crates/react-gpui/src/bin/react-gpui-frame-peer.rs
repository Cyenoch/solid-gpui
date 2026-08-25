use std::io::{self, BufReader, BufWriter};

use react_gpui::{
    EVENT_MESSAGE, EVENT_PRESS, Event, KIND_VIEW, Node, Snapshot, read_frame, write_frame,
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let snapshot = Snapshot::new(1, 1, 0, 1, vec![Node::new(1, 0, 0, KIND_VIEW)]);
    let stdout = io::stdout();
    let mut stdout = BufWriter::new(stdout.lock());
    write_frame(&mut stdout, &snapshot.encode()?)?;

    let stdin = io::stdin();
    let mut stdin = BufReader::new(stdin.lock());
    let Some(payload) = read_frame(&mut stdin)? else {
        return Err("event peer exited before receiving an event".into());
    };
    let event = Event::decode(&payload)?;
    if event.message != EVENT_MESSAGE || event.event_type != EVENT_PRESS {
        return Err("fixture received an unexpected event".into());
    }
    Ok(())
}
