use crate::{Error, NoteRange};
use soundfont::{
    SfEnum, SoundFont2, Zone,
    data::{GeneratorAmount, GeneratorType},
};

pub fn validate_sf2_file(sf2: &SoundFont2) -> Result<(), Error> {
    if sf2.info.version.major != 2 {
        return Err(Error::User(format!(
            "Unsupported SF2 file version {}; only version 2 is supported",
            sf2.info.version.major
        )));
    }

    if !sf2.presets.is_empty() {
        println!("WARNING: SF2: File has presets; these will be ignored");
    }
    if sf2.instruments.is_empty() {
        return Err(Error::User("The SF2 file has no instruments".to_owned()));
    }
    Ok(())
}

pub fn log_opened_sf2(sf2: &SoundFont2) {
    println!(
        "SF2: Contains {} presets, {} instruments and {} samples",
        sf2.presets.len(),
        sf2.instruments.len(),
        sf2.sample_headers.len()
    );
}

pub fn note_range_for_zone(zone: &Zone) -> Result<NoteRange, Error> {
    for generator in zone.gen_list.iter() {
        if let SfEnum::Value(GeneratorType::KeyRange) = generator.ty {
            if let GeneratorAmount::Range(range) = generator.amount {
                return Ok(NoteRange::new_inclusive_range(range.low, range.high));
            }
        }
    }
    Err(Error::User(
        "No key range found in an instrument zone in the SF2 file".to_owned(),
    ))
}
