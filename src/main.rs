extern crate derive_builder;
extern crate musicxml;

mod audio_gen;
mod common;
mod effect;
mod envelope;
mod midi;
mod note;
mod sequence;
mod track;
mod composition;
mod meter;
mod dsl;
mod compositions;

use musicxml::read_score_timewise;
// use crate::compositions::dsl_1;
// use crate::compositions::computer_punk_001;
// use crate::compositions::computer_punk_003;

fn main() {

    match read_score_timewise(
            "/Users/markweiss/iCloud Drive (Archive)/Documents/projects/music/In C/Terry_Riley_-_In_C.mxl") {
        Ok(score) => {
            for measure in score.content.measure {
                for measure_element in measure.content {
                    // Access the Part content (which contains PartElements)
                    if let musicxml::elements::MeasureElement::Part(part) = &measure_element {
                        // println!("Part ID: {:?}", part.attributes.id);
                        
                        // Iterate through each PartElement in the content array
                        for part_element in &part.content {
                            if let musicxml::elements::PartElement::Note(note) = part_element {
                                println!("Note: {:?}", note);

                                // Pattern match to extract the note type content
                                // r#type needed because 'type' is a reserved word in Rust
                                if let Some(note_type) = &note.content.r#type {
                                    // println!("Note type: {:?}", note_type.content);

                                    // Pattern match to extract the pitch value
                                    if let musicxml::elements::NoteType::Normal(normal_info) = &note.content.info {
                                        // Use string matching on debug output to identify pitch types
                                        let audible_str = format!("{:?}", normal_info.audible);
                                        if audible_str.starts_with("Pitch(") {
                                            // println!("Found pitched note");
                                            // Extract pitch info using debug format parsing
                                            if let Some(step_start) = audible_str.find("step: Step { attributes: (), content: ") {
                                                if let Some(step_end) = audible_str[step_start..].find(" }") {
                                                    let step_part = &audible_str[step_start + 38..step_start + step_end];
                                                    // println!("Pitch step: {}", step_part);
                                                }
                                            }
                                            // Look for octave pattern: "Octave(4)"
                                            if let Some(octave_start) = audible_str.find("Octave(") {
                                                let search_start = octave_start + 7; // "Octave(" is 7 characters
                                                if let Some(octave_end_relative) = audible_str[search_start..].find(")") {
                                                    let end_pos = search_start + octave_end_relative;
                                                    let octave_part = &audible_str[search_start..end_pos];
                                                    // println!("Pitch octave: {}", octave_part);
                                                }
                                            }
                                        } else if audible_str.starts_with("Unpitched(") {
                                            // Skip unpitched notes - do not process them
                                            // println!("Skipping unpitched note");
                                        } else if audible_str.starts_with("Rest(") {
                                            // println!("Found rest note");
                                            // println!("Rest duration: {:?}", note_type.content);
                                        }
                                    }
                                }

                            }
                            // println!("Part element: {:?}", part_element);
                        }
                    }
                }
            }
        },
        Err(e) => println!("Error reading MusicXML file: {}", e),
    }
    // dsl_1::play();
    // computer_punk_001::play();
    // computer_punk_003::play();
}
