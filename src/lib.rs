
use midly::Smf;

fn parse_file(name: &str) -> Result<(Vec<u8>, Smf), String> {
    let bytes = std::fs::read(name)
        .map_err(|e| format!("Failed open: {:?}", e))?;
    let smf = Smf::parse(&bytes)
        .map_err(|e| format!("Failed parse: {:?}", e))?;
    Ok((bytes, smf))
}

#[cfg(test)]
mod tests {
    use super::parse_file;

    #[test]
    fn it_works() {
        let smf = parse_file("resources/MIDI_sample.mid");
        assert!(smf.is_ok());
    }
}
