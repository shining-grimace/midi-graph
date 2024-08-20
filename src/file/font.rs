use crate::{util::wav_from_i16_samples, Error, NoteRange, SoundFont, SoundFontBuilder};
use byteorder::{LittleEndian, ReadBytesExt};
use soundfont::{
    data::{GeneratorAmount, GeneratorType},
    SoundFont2, Zone,
};
use std::{
    fs::File,
    io::{BufReader, Seek, SeekFrom},
};

pub fn soundfont_from_file(file_name: &str, instrument_index: usize) -> Result<SoundFont, Error> {
    let file = File::open(file_name)?;
    let mut reader = BufReader::new(file);
    let sf2 = SoundFont2::load(&mut reader)?;
    validate_sf2_file(&sf2)?;
    #[cfg(debug_assertions)]
    log_opened_sf2(&sf2);

    let sample_chunk_metadata = &sf2
        .sample_data
        .smpl
        .ok_or_else(|| Error::User("Cannot read SF2 sample header".to_owned()))?;
    let Some(instrument) = sf2.instruments.get(instrument_index) else {
        return Err(Error::User(format!(
            "Index {} out of bounds ({} instruments in SF2 file)",
            instrument_index,
            sf2.instruments.len()
        )));
    };
    #[cfg(debug_assertions)]
    println!("SF2: Using instrument from file: {:?}", &instrument.header);

    let mut soundfont_builder = SoundFontBuilder::new();
    for zone in instrument.zones.iter() {
        let Some(sample_index) = zone.sample() else {
            println!("WARNING: SF2: Sample index not found for instrument zone");
            continue;
        };
        let Some(sample_header) = sf2.sample_headers.get(*sample_index as usize) else {
            println!(
                "WARNING: SF2: Sample index {} not found matching instrument zone",
                sample_index
            );
            continue;
        };

        let sample_position = sample_chunk_metadata.offset() + sample_header.start as u64;
        let sample_length = sample_header.end as u64 - sample_position;
        let sample_data = load_sample(&mut reader, sample_position, sample_length)?;
        let note_range = note_range_for_zone(&zone)?;
        soundfont_builder = soundfont_builder.add_range(note_range, || {
            let source = wav_from_i16_samples(&sample_header, &sample_data).unwrap();
            Box::new(source)
        });
    }
    Ok(soundfont_builder.build())
}

fn validate_sf2_file(sf2: &SoundFont2) -> Result<(), Error> {
    if sf2.info.version.major != 2 {
        return Err(Error::User(format!(
            "ERROR: SF2: Unsupported SF2 file version: {}",
            sf2.info.version.major
        )));
    }

    if sf2.presets.len() > 0 {
        println!("WARNING: SF2: File has presets; these will be ignored");
    }
    if sf2.instruments.is_empty() {
        return Err(Error::User(
            "ERROR: SF2: File has no instruments".to_owned(),
        ));
    }
    Ok(())
}

fn load_sample(
    reader: &mut BufReader<File>,
    sample_position: u64,
    sample_length: u64,
) -> Result<Vec<i16>, Error> {
    reader.seek(SeekFrom::Start(sample_position))?;
    let mut sample_data = vec![0i16; sample_length as usize / 2];
    reader.read_i16_into::<LittleEndian>(&mut sample_data)?;
    Ok(sample_data)
}

fn note_range_for_zone(zone: &Zone) -> Result<NoteRange, Error> {
    for generator in zone.gen_list.iter() {
        match generator.ty {
            GeneratorType::KeyRange => match generator.amount {
                GeneratorAmount::Range(range) => {
                    return Ok(NoteRange::new_inclusive_range(range.low, range.high));
                }
                _ => {}
            },
            _ => {}
        }
    }
    Err(Error::User(
        "SF2: No key range found in instrument zone".to_owned(),
    ))
}

#[cfg(debug_assertions)]
fn log_opened_sf2(sf2: &SoundFont2) {
    println!(
        "SF2: Contains {} presets, {} instruments and {} samples",
        sf2.presets.len(),
        sf2.instruments.len(),
        sf2.sample_headers.len()
    );
}
