use std::{
    error::Error,
    io::{BufRead, BufReader},
    process::{Command, Stdio},
};

use tmdb_client::TmdbClient;

mod models;
mod parser;

pub fn detect_devices(command: &str) -> Result<Vec<models::Device>, Box<dyn Error>> {
    let process = Command::new(command).args(["-r", "--cache=1", "info", "disc:999"]).stdout(Stdio::piped()).spawn()?;

    let stdout = BufReader::new(process.stdout.ok_or("failed to capture stdout")?);

    let mut devices: Vec<models::Device> = Vec::new();

    for line in stdout.lines() {
        let columns = parser::parse_csv_line(&line?);

        if columns[0].starts_with("DRV:") {
            let device_type = columns[4].trim().to_string();
            let name: String = columns[5].trim().to_string();
            let path = columns[6].trim().to_string();

            if name.len() > 0 && device_type.len() > 0 && path.len() > 0 {
                devices.push(models::Device { name, device_type, path });
            }
        }
    }

    if devices.len() == 0 {
        return Err("no devices found".into());
    }

    Ok(devices)
}

pub struct Ripper {
    command: String,
    device: String,
    langs: Vec<String>,
    tmdb_key: String,
    pub disc: Option<models::Disc>,
}

impl Ripper {
    pub fn new(command: &str, device: &str, langs: Vec<&str>, tmdb_key: &str) -> Self {
        Self {
            command: command.to_string(),
            device: device.to_string(),
            langs: langs.iter().map(|x| x.to_string()).collect(),
            tmdb_key: tmdb_key.to_string(),
            disc: None,
        }
    }

    pub fn read_properties(&mut self) -> Result<(), Box<dyn Error>> {
        let process = Command::new(self.command.to_owned())
            .args(["-r", "info", format!("dev:{}", self.device).as_str()])
            .stdout(Stdio::piped())
            .spawn()?;

        let stdout = BufReader::new(process.stdout.ok_or("failed to capture stdout")?);

        let mut disc = models::Disc::default();
        let mut stream_type = models::StreamType::Video;

        let mut audio_stream_id: isize = -1;
        let mut subtitle_stream_id: isize = -1;

        for line in stdout.lines() {
            let columns = parser::parse_csv_line(&line?);

            match columns[0].as_str() {
                x if x.starts_with("CINFO:") => {
                    let code: usize = x.trim_start_matches("CINFO:").parse()?;
                    let value = columns.get(2).ok_or("missing value")?.to_string();

                    match code {
                        1 => disc.disc_type = value,
                        2 => disc.name = value,
                        28 => disc.metadata_language_code = value,
                        29 => disc.metadata_language_name = value,
                        30 => disc.tree_info = value,
                        31 => disc.panel_title = value,
                        32 => disc.volume_name = value,
                        33 => disc.order_weight = value.parse()?,
                        _ => println!("unhandled disc code: {}", code),
                    }
                }
                x if x.starts_with("TINFO:") => {
                    let id: usize = x.trim_start_matches("TINFO:").parse()?;
                    let code: usize = columns.get(1).ok_or("missing code")?.parse()?;
                    let value = columns.get(3).ok_or("missing value")?.to_string();

                    if disc.titles.len() <= id {
                        disc.titles.resize(id + 1, models::Title::default());
                    }

                    match code {
                        2 => disc.titles[id].name = value,
                        8 => disc.titles[id].chapter_count = value.parse()?,
                        9 => disc.titles[id].duration = parser::parse_duration_to_seconds(value.as_str()).unwrap_or(0),
                        10 => disc.titles[id].disk_size = value,
                        11 => disc.titles[id].disk_size_bytes = value.parse()?,
                        16 => disc.titles[id].source_file_name = value,
                        25 => disc.titles[id].segments_count = value.parse()?,
                        26 => disc.titles[id].segments_map = value,
                        27 => disc.titles[id].output_file_name = value,
                        28 => disc.titles[id].metadata_language_code = value,
                        29 => disc.titles[id].metadata_language_name = value,
                        30 => disc.titles[id].tree_info = value,
                        31 => disc.titles[id].panel_title = value,
                        33 => disc.titles[id].order_weight = value.parse()?,
                        _ => println!("unhandled title code: {}", code),
                    }
                }
                x if x.starts_with("SINFO:") => {
                    let title_id: usize = x.trim_start_matches("SINFO:").parse()?;
                    let code: usize = columns.get(2).ok_or("missing code")?.parse()?;
                    let value = columns.get(4).ok_or("missing value")?.to_string();

                    if code == 1 {
                        match value.as_str() {
                            "Video" => {
                                stream_type = models::StreamType::Video;
                                audio_stream_id = -1;
                                subtitle_stream_id = -1;
                            }
                            "Audio" => {
                                stream_type = models::StreamType::Audio;
                                audio_stream_id += 1;
                            }
                            "Subtitles" => {
                                stream_type = models::StreamType::Subtitle;
                                subtitle_stream_id += 1;
                            }
                            _ => println!("unhandled stream type: {}", value),
                        }
                    }

                    match stream_type {
                        models::StreamType::Video => {
                            let stream_ref = &mut disc.titles[title_id].video_stream;

                            match code {
                                01 => stream_ref.stream_type = value,
                                05 => stream_ref.codec_id = value,
                                06 => stream_ref.codec_short = value,
                                07 => stream_ref.codec_long = value,
                                19 => stream_ref.video_size = value,
                                20 => stream_ref.video_aspect_ratio = value,
                                21 => stream_ref.video_frame_rate = value,
                                22 => stream_ref.stream_flags = value,
                                28 => stream_ref.metadata_language_code = value,
                                29 => stream_ref.metadata_language_name = value,
                                30 => stream_ref.tree_info = value,
                                31 => stream_ref.panel_title = value,
                                33 => stream_ref.order_weight = value.parse()?,
                                38 => stream_ref.mkv_flags = value,
                                42 => stream_ref.output_conversion_type = value,
                                _ => println!("unhandled subtitle code: {}", code),
                            }
                        }
                        models::StreamType::Audio => {
                            let audio_stream_id = audio_stream_id as usize;

                            let streams_ref = &mut disc.titles[title_id].audio_streams;

                            if streams_ref.len() <= audio_stream_id {
                                streams_ref.resize(audio_stream_id + 1, models::AudioStream::default());
                            }

                            let stream_ref = &mut streams_ref[audio_stream_id];

                            match code {
                                01 => stream_ref.stream_type = value,
                                02 => stream_ref.name = value,
                                03 => stream_ref.lang_code = value,
                                04 => stream_ref.lang_name = value,
                                05 => stream_ref.codec_id = value,
                                06 => stream_ref.codec_short = value,
                                07 => stream_ref.codec_long = value,
                                13 => stream_ref.bitrate = value,
                                14 => stream_ref.audio_channels_count = value.parse()?,
                                17 => stream_ref.audio_sample_rate = value.parse()?,
                                18 => stream_ref.audio_sample_size = value.parse()?,
                                22 => stream_ref.stream_flags = value,
                                28 => stream_ref.metadata_language_code = value,
                                29 => stream_ref.metadata_language_name = value,
                                30 => stream_ref.tree_info = value,
                                31 => stream_ref.panel_title = value,
                                33 => stream_ref.order_weight = value.parse()?,
                                38 => stream_ref.mkv_flags = value,
                                39 => stream_ref.mkv_flags_text = value,
                                40 => stream_ref.audio_channel_layout_name = value,
                                42 => stream_ref.output_conversion_type = value,
                                _ => println!("unhandled subtitle code: {}", code),
                            }
                        }
                        models::StreamType::Subtitle => {
                            let subtitle_stream_id = subtitle_stream_id as usize;

                            let streams_ref = &mut disc.titles[title_id].subtitle_streams;

                            if streams_ref.len() <= subtitle_stream_id {
                                streams_ref.resize(subtitle_stream_id + 1, models::SubtitleStream::default());
                            }

                            let stream_ref = &mut streams_ref[subtitle_stream_id];

                            match code {
                                01 => stream_ref.stream_type = value,
                                03 => stream_ref.lang_code = value,
                                04 => stream_ref.lang_name = value,
                                05 => stream_ref.codec_id = value,
                                06 => stream_ref.codec_short = value,
                                07 => stream_ref.codec_long = value,
                                22 => stream_ref.stream_flags = value,
                                28 => stream_ref.metadata_language_code = value,
                                29 => stream_ref.metadata_language_name = value,
                                30 => stream_ref.tree_info = value,
                                31 => stream_ref.panel_title = value,
                                33 => stream_ref.order_weight = value.parse()?,
                                38 => stream_ref.mkv_flags = value,
                                39 => stream_ref.mkv_flags_text = value,
                                42 => stream_ref.output_conversion_type = value,
                                _ => println!("unhandled subtitle code: {}", code),
                            }
                        }
                    }
                }
                _ => {}
            }
        }

        self.disc = Some(disc);

        Ok(())
    }

    pub async fn filter_movie_candidates(&mut self, tmdb_id: u32) -> Result<(), Box<dyn Error>> {
        let movie = TmdbClient::new(self.tmdb_key.as_str()).get_movie(tmdb_id).await?;

        let filtered_titles = self
            .disc
            .as_ref()
            .expect("disc not read in yet")
            .titles
            .iter()
            .filter(|title| {
                if title.audio_streams.len() == 0 {
                    return false;
                }

                let satisfies_language = title.audio_streams.iter().any(|stream| self.langs.contains(&stream.lang_code));

                if !satisfies_language {
                    return false;
                }

                let title_runtime = title.duration as f32;
                let actual_runtime = (movie.runtime * 60) as f32;

                title_runtime >= actual_runtime * 0.9 && title_runtime <= actual_runtime * 1.1
            })
            .cloned()
            .collect::<Vec<_>>();

        self.disc.as_mut().unwrap().titles = filtered_titles;

        Ok(())
    }

    pub async fn filter_tv_series_candidates(&mut self, tmdb_id: u32, season: u16, episodes: Vec<u16>) -> Result<(), Box<dyn Error>> {
        let tv_series = TmdbClient::new(self.tmdb_key.as_str()).get_tv_series(tmdb_id).await?;

        let episode_runtimes = tv_series.seasons[(season - 1) as usize]
            .episodes
            .iter()
            .filter(|episode| episodes.contains(&episode.episode_number))
            .cloned()
            .map(|episode| (episode.runtime * 60) as f32)
            .collect::<Vec<_>>();

        let filtered_titles = self
            .disc
            .as_ref()
            .expect("disc not read in yet")
            .titles
            .iter()
            .filter(|title| {
                if title.audio_streams.len() == 0 {
                    return false;
                }

                let satisfies_language = title.audio_streams.iter().any(|stream| self.langs.contains(&stream.lang_code));

                if !satisfies_language {
                    return false;
                }

                let title_runtime = title.duration as f32;

                episode_runtimes
                    .iter()
                    .any(|runtime| title_runtime >= *runtime * 0.9 && title_runtime <= *runtime * 1.1)
            })
            .cloned()
            .collect::<Vec<_>>();

        self.disc.as_mut().unwrap().titles = filtered_titles;

        Ok(())
    }
}